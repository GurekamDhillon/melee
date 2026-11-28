//! DOL referee, native-records mode (docs/rust-port/notes/referee.md section 10).
//!
//! A Rust function carrying `melee_core::referee_native!` is checked in the running match: before the Rust call, the
//! retail function of the same name runs from main.dol (plus a Gecko list, when one is configured) in the Gekko-exact
//! interpreter on the engine's live records. Records are reached through `remap::GuestSpace`: the call's arguments
//! (an `HSD_GObj`, ...) get guest handles and every pointer the retail code follows registers the native object it
//! names, so retail reads and writes the very objects the engine simulates. Engine globals the retail code reads at
//! their retail addresses come from a sync table (player slots, stage data, ftCommonData, the RNG seed, the gobj
//! lists, the pad queue): native records mapped at their retail addresses, or read-only guest copies.
//!
//! Every host byte retail stores is journaled; after the run the journal gives "what retail wrote" and is undone, the
//! Rust function then runs on the untouched state, and the two are compared: the return value, every byte retail
//! wrote, and every byte Rust changed in a record retail touched that retail did not write.
//!
//! Not checkable (counted with the address and its symbol, never a pass): a read of .bss or of a mutable-looking
//! .data/.sdata object outside the sync table, a store to the image or to a read-only sync copy, an unmapped address,
//! a native field without a GC counterpart, a native callback without a retail address, an unsupported instruction,
//! the budget.
//!
//! Callees are interpreted. A boundary callee (default `Fighter_ChangeMotionState`; `MELEE_REFEREE_BOUNDARY`) ends the
//! retail run at its entry: the outcome is "calls X with these arguments", compared with the state and arguments of
//! the Rust run's own call of X (its `referee_boundary!`). A native callee (`MELEE_REFEREE_NATIVE_CALLEES`, only
//! functions whose effect the sync table covers: HSD_Randi, HSD_Randf) runs as the engine's function instead.
//!
//! Env: `MELEE_REFEREE_NATIVE=<report file>` turns it on; `MELEE_REFEREE_FNS=a,b`; `MELEE_REFEREE_MAX` (checks per
//! function, default 2000); `MELEE_REFEREE_GECKO=none|replay|ini:<path>` (default: replay when the replay has a list);
//! `MELEE_REFEREE_FROM`/`MELEE_REFEREE_TO` (frame window). The report holds engine values and addresses, no disc bytes.
#![allow(static_mut_refs, clippy::type_complexity)]

use melee_core::referee::{Arg, NativeHooks, Ret};
use mex_import::dol::{Dol, Sym};
use mex_import::ppc::ir::{decode, Op};
use mex_import::ppc::{Bridge, Cpu, Fault, Machine, Mem, Step};
use mex_import::referee::{boot_gqr, conditional};
use mex_import::remap::{GuestSpace, Loc};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::io::Write;

const R13: u32 = 0x804D_B6A0;
const R2: u32 = 0x804D_F9E0;
const STACK_BASE: u32 = 0x8170_0000;
const STACK_SIZE: u32 = 0x10000;
/// Guest copies the sync table builds (gobj list heads, ...).
const COPY_BASE: u32 = 0x8580_0000;
/// The low guest range: the DOL image, the Gecko area, the stack. Accesses there go to a sync region when one covers
/// them, else to the image; everything above is GuestSpace's (native record handles).
const LOW_END: u32 = 0x8180_0000;
const BUDGET: u64 = 3_000_000;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum IKind {
    Text,
    Const,
    Data,
    Bss,
}

struct Img {
    base: u32,
    bytes: Vec<u8>,
    kind: IKind,
}

#[derive(Default)]
struct FnStats {
    calls: u64,
    compared: u64,
    matched: u64,
    mismatched: u64,
    notcheck: u64,
    boundary: BTreeMap<String, u64>,
    reasons: BTreeMap<String, (u64, i32)>,
    mismatches: Vec<String>,
    insns: u64,
    data_reads: BTreeSet<String>,
    gecko_reads: BTreeSet<u32>,
    pad_stores: u64,
}

/// How a retail run ended.
enum ROut {
    Ret { r3: u32, f1: u64 },
    Boundary { name: &'static str, gpr: [u32; 8], fpr: [u64; 4] },
    NotCheckable(String),
}

struct Open {
    name: &'static str,
    frame: i32,
    out: ROut,
    /// journaled host bytes: native address -> (byte before, byte retail left, guest address)
    wrote: BTreeMap<usize, (u8, u8, u32)>,
    /// watched native objects: (native address, native size, record, guest base) and their bytes before the Rust run
    watched: Vec<(usize, usize, &'static str, u32)>,
    before: Vec<Vec<u8>>,
    /// Rust's state at its first boundary call: (name, args, journal-address bytes, watched bytes)
    rust_b: Option<(&'static str, Vec<Arg>, Vec<u8>, Vec<Vec<u8>>)>,
    boundaries: Vec<&'static str>,
}

struct St {
    img: Vec<Img>,
    syms: Vec<Sym>,
    funcs: Vec<(u32, u32, usize)>,
    by_name: HashMap<String, u32>,
    text: Vec<(u32, u32)>,
    gecko_area: (u32, u32),
    gecko_desc: String,
    only: Option<Vec<String>>,
    max: u64,
    from: i32,
    to: i32,
    boundary_names: Vec<&'static str>,
    native_callees: Vec<&'static str>,
    boundary_at: HashMap<u32, &'static str>,
    native_at: HashMap<u32, &'static str>,
    space: GuestSpace,
    stats: BTreeMap<&'static str, FnStats>,
    cov: BTreeMap<u32, (u64, u64)>,
    open: Option<Open>,
    out: std::path::PathBuf,
    /// read-only sync copies (guest ranges)
    ro: Vec<(u32, u32, &'static str)>,
    /// sync regions mapped onto native memory: (guest base, len, name)
    synced: Vec<(u32, u32, &'static str)>,
    inv: HashMap<&'static str, Vec<Option<u32>>>,
    retail_fn: HashMap<usize, u32>,
    copy_next: u32,
    /// UCF 0.84's pad buffer in the Gecko area (block of the C2 at 0x8006B460, + 4)
    ucf084_buf: Option<u32>,
    /// image bytes the boot initialisers wrote (static ctors, class info inits): readable as constants
    init_written: std::collections::HashSet<u32>,
}

static mut ST: Option<St> = None;

fn st() -> Option<&'static mut St> {
    unsafe { ST.as_mut() }
}

fn leak(s: &str) -> &'static str {
    Box::leak(s.to_string().into_boxed_str())
}

/// Turn the native-records mode on (MELEE_REFEREE_NATIVE) for a replay whose Gecko list is `gecko` (raw list bytes,
/// may be empty). Called once by the replay runner before the match.
pub fn init(gecko: &[u8]) {
    let Some(out) = std::env::var_os("MELEE_REFEREE_NATIVE") else { return };
    let Some(path) = Dol::find() else {
        eprintln!("livereferee: no main.dol (REFEREE_DOL)");
        return;
    };
    let mut dol = match Dol::load(&path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("livereferee: {e}");
            return;
        }
    };
    let syms = match mex_import::dol::symbols() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("livereferee: {e}");
            return;
        }
    };
    // the Gecko list: the replay's own (default when it has one), a Dolphin INI, or none
    let mode = std::env::var("MELEE_REFEREE_GECKO").unwrap_or_else(|_| if gecko.is_empty() { "none".into() } else { "replay".into() });
    let codes: Vec<mex_import::gecko::Code> = if mode == "replay" {
        let lines = gecko.chunks_exact(8).map(|c| (u32::from_be_bytes(c[0..4].try_into().unwrap()), u32::from_be_bytes(c[4..8].try_into().unwrap()))).collect();
        vec![mex_import::gecko::Code { name: "replay list".into(), lines }]
    } else if let Some(p) = mode.strip_prefix("ini:") {
        match std::fs::read_to_string(p) {
            Ok(t) => mex_import::gecko::parse_ini(&t, false),
            Err(e) => {
                eprintln!("livereferee: {p}: {e}");
                return;
            }
        }
    } else {
        vec![]
    };
    let rep = mex_import::gecko::apply(&mut dol, &codes);
    let gecko_desc = format!("gecko={mode} codes={} writes={} inserts={} outside={} skipped={}", codes.len(), rep.writes, rep.inserts.len(), rep.outside.len(), rep.skipped.len());
    let gecko_area = dol.sections.iter().find(|s| s.addr == mex_import::gecko::AREA_BASE).map(|s| (s.addr, s.addr + s.bytes.len() as u32)).unwrap_or((0, 0));
    let (img, text) = image(&dol, &syms);
    let mut funcs: Vec<(u32, u32, usize)> = syms.iter().enumerate().filter(|(_, s)| s.function && s.size > 0).map(|(i, s)| (s.addr, s.addr + s.size, i)).collect();
    funcs.sort();
    let by_name: HashMap<String, u32> = syms.iter().filter(|s| s.function).map(|s| (s.name.clone(), s.addr)).collect();
    let list = |k: &str, d: &[&str]| -> Vec<&'static str> {
        match std::env::var(k) {
            Ok(v) => v.split(',').map(str::trim).filter(|s| !s.is_empty()).map(leak).collect(),
            Err(_) => d.iter().map(|s| leak(s)).collect(),
        }
    };
    let boundary_names = list("MELEE_REFEREE_BOUNDARY", &["Fighter_ChangeMotionState"]);
    let native_callees = list("MELEE_REFEREE_NATIVE_CALLEES", &[]);
    let at = |names: &[&'static str]| -> HashMap<u32, &'static str> { names.iter().filter_map(|n| by_name.get(*n).map(|&a| (a, *n))).collect() };
    let boundary_at = at(&boundary_names);
    let native_at = at(&native_callees);
    let only = std::env::var("MELEE_REFEREE_FNS").ok().map(|v| v.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect());
    let num = |k: &str, d: i64| std::env::var(k).ok().and_then(|v| v.parse().ok()).unwrap_or(d);
    let mut space = GuestSpace::new();
    space.void_pointee = Some(void_pointee);
    unsafe {
        ST = Some(St {
            img,
            syms,
            funcs,
            by_name,
            text,
            gecko_area,
            gecko_desc,
            only,
            max: num("MELEE_REFEREE_MAX", 2000) as u64,
            from: num("MELEE_REFEREE_FROM", i32::MIN as i64) as i32,
            to: num("MELEE_REFEREE_TO", i32::MAX as i64) as i32,
            boundary_names,
            native_callees,
            boundary_at,
            native_at,
            space,
            stats: BTreeMap::new(),
            cov: BTreeMap::new(),
            open: None,
            out: out.into(),
            ro: vec![],
            synced: vec![],
            inv: HashMap::new(),
            retail_fn: HashMap::new(),
            copy_next: COPY_BASE,
            ucf084_buf: rep.inserts.iter().find(|i| i.0 == 0x8006_B460).map(|i| i.1 + 4),
            init_written: Default::default(),
        });
    }
    let s = st().unwrap();
    s.space.fn_guest = Some(fn_guest);
    s.space.fn_native = Some(fn_native);
    build_retail_fn_map(s, &dol);
    boot_init(s);
    melee_core::referee::set_native(NativeHooks { wants, pre, post, boundary });
    eprintln!("livereferee: on ({}); boundaries {:?}", s.gecko_desc, s.boundary_names);
}

/// The DOL (patched) as guest regions, each section kind named from symbols.txt; the bss range as zeros.
fn image(dol: &Dol, syms: &[Sym]) -> (Vec<Img>, Vec<(u32, u32)>) {
    let sec_name = |lo: u32, hi: u32| -> String {
        let mut n: HashMap<&str, u32> = HashMap::new();
        for s in syms.iter().filter(|s| s.addr >= lo && s.addr < hi) {
            *n.entry(&s.section).or_default() += 1;
        }
        n.into_iter().max_by_key(|e| e.1).map(|e| e.0.to_string()).unwrap_or_default()
    };
    let mut img = vec![];
    let mut text = vec![];
    for s in &dol.sections {
        let hi = s.addr + s.bytes.len() as u32;
        let kind = if s.text {
            IKind::Text
        } else {
            match sec_name(s.addr, hi).as_str() {
                ".data" | ".sdata" => IKind::Data,
                ".bss" | ".sbss" => IKind::Bss,
                _ => IKind::Const,
            }
        };
        if kind == IKind::Text {
            text.push((s.addr, hi));
        }
        img.push(Img { base: s.addr, bytes: s.bytes.clone(), kind });
    }
    let (blo, bhi) = (dol.bss_addr, dol.bss_addr + dol.bss_size);
    let mut lo = blo;
    while lo < bhi {
        if let Some(e) = img.iter().find(|r| lo >= r.base && lo < r.base + r.bytes.len() as u32).map(|r| r.base + r.bytes.len() as u32) {
            lo = e;
            continue;
        }
        let next = img.iter().filter(|r| r.base > lo && r.base < bhi).map(|r| r.base).min().unwrap_or(bhi);
        // .sbss2 is constant zeros
        let k = match syms.iter().find(|s| s.addr >= lo && s.addr < next).map(|s| s.section.as_str()) {
            Some(".sbss2") => IKind::Const,
            _ => IKind::Bss,
        };
        img.push(Img { base: lo, bytes: vec![0; (next - lo) as usize], kind: k });
        lo = next;
    }
    (img, text)
}

/// Native callbacks the retail code may read out of a record (fp->cb.*, MotionState rows), by their retail address.
fn build_retail_fn_map(s: &mut St, dol: &Dol) {
    for n in crate::mex_bridge::GOBJ_FN_NAMES {
        if let (Some(f), Some(&a)) = (crate::mex_bridge::gobj_fn(n), s.by_name.get(*n)) {
            s.retail_fn.insert(f as usize, a);
        }
    }
    for (f, n) in melee_fighter::tu::fighter::fighter_proc_fns() {
        if let Some(&a) = s.by_name.get(n) {
            s.retail_fn.insert(f, a);
        }
    }
    let _ = dol;
}

/// The common MotionState table (fp->x1C, 341 rows at ftData_MotionStateList) row callbacks by retail address: mapped
/// the first time a fighter is bound.
unsafe fn map_common_motion_table(s: &mut St, native: *const melee_types::MotionState) {
    static mut DONE: bool = false;
    if native.is_null() || DONE {
        return;
    }
    DONE = true;
    let a = 0x803C_2800u32;
    for k in 0..341usize {
        let r = &*native.add(k);
        let cbs = [r.anim_cb, r.input_cb, r.phys_cb, r.coll_cb, r.cam_cb];
        for (j, cb) in cbs.iter().enumerate() {
            let o = a + (k * 0x20 + 0xC + 4 * j) as u32;
            if let (Some(f), Some(w)) = (cb, img_word(s, o)) {
                if w != 0 {
                    s.retail_fn.entry(*f as usize).or_insert(w);
                }
            }
        }
    }
}

fn img_word(s: &St, a: u32) -> Option<u32> {
    let r = s.img.iter().find(|r| a >= r.base && a + 4 <= r.base + r.bytes.len() as u32)?;
    let o = (a - r.base) as usize;
    Some(u32::from_be_bytes(r.bytes[o..o + 4].try_into().unwrap()))
}

fn fn_guest(p: usize) -> Option<u32> {
    st()?.retail_fn.get(&p).copied()
}

fn fn_native(a: u32) -> Option<usize> {
    let n = mex_import::layout::vanilla_name(a)?;
    crate::mex_bridge::gobj_fn(n).map(|f| f as usize)
}

/// What an untyped pointer field holds (as mex_guest's): HSD_GObj.user_data by class, hsd_obj the root joint,
/// HSD_JObj.u by the joint's flags.
fn void_pointee(rec: &'static str, off: u32, obj: *mut u8, _p: usize) -> Option<&'static str> {
    if rec == "HSD_JObj" && off == 0x18 {
        let f = unsafe { (*(obj as *mut melee_types::HSD_JObj)).flags };
        return Some(if f & (1 << 5) != 0 {
            "_HSD_SList"
        } else if f & (1 << 14) != 0 {
            "HSD_Spline"
        } else {
            "HSD_DObj"
        });
    }
    if rec != "HSD_GObj" {
        return None;
    }
    let g = obj as *mut melee_types::HSD_GObj;
    match off {
        0x28 => Some("HSD_JObj"),
        0x2C => match unsafe { (*g).classifier } as i32 {
            melee_types::HSD_GOBJ_CLASS_FIGHTER => Some("Fighter"),
            melee_types::HSD_GOBJ_CLASS_ITEM => Some("Item"),
            _ => None,
        },
        _ => None,
    }
}

fn sym_at(s: &St, a: u32) -> String {
    let i = s.syms.partition_point(|x| x.addr <= a);
    if let Some(x) = i.checked_sub(1).map(|i| &s.syms[i]) {
        if a < x.addr + x.size.max(1) || x.size == 0 && a - x.addr < 0x100 {
            return format!("{}+{:#x}", x.name, a - x.addr);
        }
    }
    if a >= s.gecko_area.0 && a < s.gecko_area.1 {
        return format!("gecko-area+{:#x}", a - s.gecko_area.0);
    }
    format!("{a:08X}")
}

fn func_at(s: &St, a: u32) -> Option<&Sym> {
    let i = s.funcs.partition_point(|f| f.0 <= a);
    let f = s.funcs.get(i.checked_sub(1)?)?;
    (a < f.1).then(|| &s.syms[f.2])
}

fn where_(s: &St, pc: u32) -> String {
    match func_at(s, pc) {
        Some(f) => format!("{}+{:#x}", f.name, pc - f.addr),
        None => sym_at(s, pc),
    }
}

// ---- the sync table --------------------------------------------------------------------------------------------

/// Map the engine globals the retail code reads at their retail addresses. Rebuilt before each check (cheap: native
/// regions are registrations, the copies are a few hundred bytes).
unsafe fn sync(s: &mut St) {
    s.ro.clear();
    s.synced.clear();
    s.copy_next = COPY_BASE;
    let sp = &mut s.space;
    // player slots (StaticPlayer[6], 0x80453080) and the stage (StageInfo, 0x8049E6C8): native records in place
    {
        let ps = core::ptr::addr_of_mut!(melee_world::pl::player_slots) as *mut melee_types::StaticPlayer;
        let gs = sp.layouts.gc_size("_StaticPlayer").unwrap_or(0xE90);
        for i in 0..6u32 {
            if let Err(e) = sp.add_native(Some(0x8045_3080 + i * gs), "_StaticPlayer", ps.add(i as usize) as *mut u8) {
                once(&format!("player_slots: {e}"));
            }
        }
        s.synced.push((0x8045_3080, 6 * gs, "player_slots"));
        let si = core::ptr::addr_of_mut!(melee_world::tu::ground::stage_info) as *mut u8;
        match sp.add_native(Some(0x8049_E6C8), "StageInfo", si) {
            Ok(_) => s.synced.push((0x8049_E6C8, sp.layouts.gc_size("StageInfo").unwrap_or(0x748), "stage_info")),
            Err(e) => once(&format!("stage_info: {e}")),
        }
        // the match rules (StartMeleeRules, inside gm's `controller` block at 0x8046DB68: gm_8016B0FC reads +2)
        let r = melee_modes::tu::gmvs::gm_GetStartMeleeRules() as *mut u8;
        if !r.is_null() {
            match sp.add_native(Some(0x8046_DB68), "StartMeleeRules", r) {
                Ok(_) => s.synced.push((0x8046_DB68, 0x60, "StartMeleeRules")),
                Err(e) => once(&format!("StartMeleeRules: {e}")),
            }
        }
    }
    // scalar statics of mpLib / mpColl the collision code reads and writes: native words in place (journaled)
    {
        use melee_world::tu::{mpcoll as c, mplib as l};
        let words: [(u32, *mut u8, u32, &'static str); 15] = [
            (0x804D_6490, core::ptr::addr_of_mut!(c::mpColl_804D6490_max_x) as *mut u8, 4, "mpColl_804D6490_max_x"),
            (0x804D_6494, core::ptr::addr_of_mut!(c::mpColl_804D6494_line_id) as *mut u8, 4, "mpColl_804D6494_line_id"),
            (0x804D_6498, core::ptr::addr_of_mut!(c::mpColl_804D6498_flags) as *mut u8, 4, "mpColl_804D6498_flags"),
            (0x804D_649C, core::ptr::addr_of_mut!(c::mpColl_IsEcbTiny) as *mut u8, 4, "mpColl_IsEcbTiny"),
            (0x804D_64AC, core::ptr::addr_of_mut!(c::mpColl_804D64AC) as *mut u8, 4, "mpColl_804D64AC"),
            (0x804D_64B0, core::ptr::addr_of_mut!(l::didCheckBounding) as *mut u8, 4, "didCheckBounding"),
            (0x804D_64CC, core::ptr::addr_of_mut!(l::mpLib_804D64CC) as *mut u8, 4, "mpLib_804D64CC"),
            (0x804D_64D0, core::ptr::addr_of_mut!(l::mpLib_804D64D0) as *mut u8, 4, "mpLib_804D64D0"),
            (0x804D_64D4, core::ptr::addr_of_mut!(l::mpLib_804D64D4) as *mut u8, 4, "mpLib_804D64D4"),
            (0x804D_64D8, core::ptr::addr_of_mut!(l::mpLib_804D64D8) as *mut u8, 4, "mpLib_804D64D8"),
            (0x804D_64DC, core::ptr::addr_of_mut!(l::mpLib_804D64DC) as *mut u8, 4, "mpLib_804D64DC"),
            (0x804D_64E0, core::ptr::addr_of_mut!(l::mpLib_804D64E0) as *mut u8, 4, "mpLib_804D64E0"),
            (0x804D_64E4, core::ptr::addr_of_mut!(l::mpLib_804D64E4) as *mut u8, 4, "mpLib_804D64E4"),
            (0x8045_8868, core::ptr::addr_of_mut!(l::mpLib_80458868) as *mut u8, 0x20, "mpLib_80458868"),
            (0x8045_8888, core::ptr::addr_of_mut!(l::mpLib_80458888) as *mut u8, 0x600, "mpLib_80458888"),
        ];
        for (a, p, n, name) in words {
            sp.add_words(a, p, n);
            s.synced.push((a, n, name));
        }
    }
    // the RNG: `seed` (0x804D5F90) is the engine's current seed word, HSD_RandSeedPtr (0x804D5F94) points at it
    {
        let p = melee_fighter::rng::HSD_RandSeedPtr() as *mut u8;
        sp.add_words(0x804D_5F90, p, 4);
        s.synced.push((0x804D_5F90, 4, "seed"));
        sp.add_bytes(0x804D_5F94, 0x804D_5F90u32.to_be_bytes().to_vec());
        s.ro.push((0x804D_5F94, 4, "HSD_RandSeedPtr"));
    }
    // pointer globals: guest words holding the native record's handle (read-only)
    use melee_world::tu::mplib as l;
    let ptrs: [(u32, usize, &'static str, &'static str); 10] = [
        (0x804D_6554, melee_runtime::shared::p_ftCommonData as usize, "ftCommonData", "p_ftCommonData"),
        (0x804D_64FC, melee_runtime::shared::Fighter_804D64FC as usize, "Fighter_804D64FC_t", "Fighter_804D64FC"),
        (0x804D_6D28, melee_items::tu::it_3F14::it_804D6D28 as usize, "ItemCommonData", "it_804D6D28"),
        (0x804D_64B4, l::mpLib_804D64B4 as usize, "MapCollData", "mpLib_804D64B4"),
        (0x804D_64B8, l::groundCollVtx as usize, "CollVtx", "groundCollVtx"),
        (0x804D_64BC, l::groundCollLine as usize, "CollLine", "groundCollLine"),
        (0x804D_64C0, l::groundCollJoint as usize, "CollJoint", "groundCollJoint"),
        (0x804D_64C4, l::jointListStart as usize, "CollJoint", "jointListStart"),
        (0x804D_64C8, l::jointListEnd as usize, "CollJoint", "jointListEnd"),
        (0x804D_6544, melee_fighter::hooks::FT_PARTS_TABLE as usize, "", "ftPartsTable"),
    ];
    for (a, p, rec, name) in ptrs {
        let h = if rec.is_empty() {
            // a table of record pointers (ftPartsTable: FighterPartsTable*[kind]): a guest copy of handles
            table_copy(sp, &mut s.copy_next, &mut s.ro, p, "FighterPartsTable", melee_types::Ft_Kind_Max as usize, name)
        } else {
            sp.handle(p, Some(rec)).unwrap_or(0)
        };
        sp.add_bytes(a, h.to_be_bytes().to_vec());
        s.ro.push((a, 4, name));
    }
    // HSD_GObjPLinkHead (0x804D782C): a guest table of the list heads' handles (read-only)
    {
        use melee_runtime::tu::gobj_native as gn;
        if !gn::HSD_GObjPLinkHead.is_null() {
            let n = (gn::HSD_GObjLibInitData.p_link_max as u32 + 1).min(64);
            let mut b = vec![0u8; 4 * n as usize];
            for i in 0..n {
                let g = *gn::HSD_GObjPLinkHead.add(i as usize);
                let h = sp.handle(g as usize, Some("HSD_GObj")).unwrap_or(0);
                b[4 * i as usize..4 * i as usize + 4].copy_from_slice(&h.to_be_bytes());
            }
            let t = s.copy_next;
            s.copy_next += 0x100;
            sp.add_bytes(t, b);
            s.ro.push((t, 4 * n, "HSD_GObjPLinkHead[]"));
            sp.add_bytes(0x804D_782C, t.to_be_bytes().to_vec());
            s.ro.push((0x804D_782C, 4, "HSD_GObjPLinkHead"));
        }
    }
    // the HSD pad queue the UCF 0.7x/0.8 codes read (HSD_PadLibData 0x804C1F78: qread at +1, queue pointer at +8; the
    // PADStatus queue at 0x8046B108, 5 entries x 4 ports x 12 bytes): built from the engine's raw-stick history of the
    // replay (Replay_RawStickBack: back 0 = this frame), entry (qread - 1 - back) mod 5. Read-only.
    {
        let mut lib = vec![0u8; 0x34];
        lib[0] = 5;
        lib[1] = 1;
        lib[8..12].copy_from_slice(&0x8046_B108u32.to_be_bytes());
        sp.add_bytes(0x804C_1F78, lib);
        s.ro.push((0x804C_1F78, 0x34, "HSD_PadLibData"));
        let mut q = vec![0u8; 5 * 48];
        for back in 0..5i32 {
            let e = (0 - back).rem_euclid(5) as usize;
            for port in 0..4i32 {
                let o = e * 48 + port as usize * 12;
                for (k, which) in [(2usize, 0i32), (3, 1), (4, 2), (5, 3)] {
                    q[o + k] = melee_fighter::platform::Replay_RawStickBack(port, which, back) as i8 as u8;
                }
            }
        }
        sp.add_bytes(0x8046_B108, q);
        s.ro.push((0x8046_B108, 5 * 48, "HSD_PadLibData.queue"));
    }
    // UCF 0.84's pad buffer, kept by "Pad Buffer + 1.0 Cardinals" in its own inserted block (C2 8006B460, block + 4,
    // 4 ports x 12 bytes: (x, y) pairs of a 4-deep ring, the ring index at +8, the flick counter at +9; the other
    // 0.84 codes find it through the hook's branch word): the engine's ftUcf_Pad. Read-only.
    if let Some(b) = s.ucf084_buf {
        let pads = &*core::ptr::addr_of!(melee_fighter::fighter::UCF_PAD);
        let mut q = vec![0u8; 48];
        for (p, u) in pads.iter().enumerate() {
            for i in 0..4 {
                q[p * 12 + 2 * i] = u.x[i] as u8;
                q[p * 12 + 2 * i + 1] = u.y[i] as u8;
            }
            q[p * 12 + 8] = u.idx;
            q[p * 12 + 9] = u.counter;
        }
        s.space.add_bytes(b, q);
        s.ro.push((b, 48, "UCF 0.84 pad buffer"));
    }
}

/// A guest copy of a native table of `n` record pointers: their handles (read-only); returns its guest address.
unsafe fn table_copy(sp: &mut GuestSpace, next: &mut u32, ro: &mut Vec<(u32, u32, &'static str)>, p: usize, rec: &'static str, n: usize, name: &'static str) -> u32 {
    if p == 0 {
        return 0;
    }
    let t = p as *const usize;
    let mut b = vec![0u8; 4 * n];
    for i in 0..n {
        let h = sp.handle(*t.add(i), Some(rec)).unwrap_or(0);
        b[4 * i..4 * i + 4].copy_from_slice(&h.to_be_bytes());
    }
    let a = *next;
    *next += ((4 * n as u32) + 0xFF) & !0xFF;
    sp.add_bytes(a, b);
    ro.push((a, 4 * n as u32, name));
    a
}

fn once(msg: &str) {
    static SEEN: std::sync::Mutex<Vec<String>> = std::sync::Mutex::new(Vec::new());
    if let Ok(mut v) = SEEN.lock() {
        if !v.iter().any(|m| m == msg) {
            eprintln!("livereferee: {msg}");
            v.push(msg.to_string());
        }
    }
}

/// .data objects read as constants (filled at build time, never written by the game during a match): the stage
/// collision parameter tables mpLib_800569EC looks up by stage kind.
const CONST_DATA: [&str; 2] = ["mpLib_803B", "Unit01"];

/// The boot initialisers the referee runs once on the image, .data/.bss writable: the static ctors (`_ctors`, MSL's
/// trig tables) and the HSD class infos the retail code calls through (JObjInfoInit -> hsdJObj's methods). Every
/// byte they write is then readable as a constant (`init_written`).
fn boot_init(s: &mut St) {
    let mut entries: Vec<u32> = vec![];
    if let Some(t) = s.syms.iter().find(|x| x.name == "_ctors").map(|x| x.addr) {
        let mut a = t;
        while let Some(f) = img_word(s, a).filter(|&f| f != 0) {
            entries.push(f);
            a += 4;
        }
    }
    for n in ["JObjInfoInit"] {
        if let Some(&a) = s.by_name.get(n) {
            entries.push(a);
        }
    }
    for f in entries {
        let mut mem = InitMem { img: &mut s.img, written: &mut s.init_written, stack: vec![0; STACK_SIZE as usize] };
        let mut cpu = Cpu::default();
        cpu.gpr[1] = STACK_BASE + STACK_SIZE - 0x100;
        cpu.gpr[2] = R2;
        cpu.gpr[13] = R13;
        cpu.pc = f;
        let mut m = Machine::new_gekko(boot_gqr(cpu));
        let mut br = TextOnly(s.text.clone());
        let r = m.run(&mut mem, &mut br, BUDGET as u32);
        if r.is_err() {
            eprintln!("livereferee: boot initialiser {f:08X} did not finish: {r:?}");
        }
    }
    // the native JObj class info is the retail hsdJObj (its methods are the retail functions JObjInfoInit set)
    if let Some(a) = s.syms.iter().find(|x| x.name == "hsdJObj").map(|x| x.addr) {
        s.retail_fn.insert(unsafe { core::ptr::addr_of_mut!(melee_runtime::tu::jobj::hsdJObj) } as usize, a);
    }
    eprintln!("livereferee: boot initialisers wrote {} image bytes", s.init_written.len());
}

struct InitMem<'a> {
    img: &'a mut Vec<Img>,
    written: &'a mut std::collections::HashSet<u32>,
    stack: Vec<u8>,
}

impl InitMem<'_> {
    fn at(&mut self, ea: u32, n: u32) -> Result<(&mut [u8], bool), Fault> {
        if ea >= STACK_BASE && ea + n <= STACK_BASE + STACK_SIZE {
            let o = (ea - STACK_BASE) as usize;
            return Ok((&mut self.stack[o..o + n as usize], true));
        }
        let r = self.img.iter_mut().find(|r| ea >= r.base && ea as u64 + n as u64 <= r.base as u64 + r.bytes.len() as u64).ok_or(Fault::Mem(ea))?;
        let w = matches!(r.kind, IKind::Data | IKind::Bss);
        let o = (ea - r.base) as usize;
        Ok((&mut r.bytes[o..o + n as usize], w))
    }
    fn ld(&mut self, ea: u32, n: u32) -> Result<u64, Fault> {
        let (b, _) = self.at(ea, n)?;
        Ok(b.iter().fold(0u64, |v, x| (v << 8) | *x as u64))
    }
    fn st(&mut self, ea: u32, n: u32, v: u64) -> Result<(), Fault> {
        let stack = ea >= STACK_BASE && ea < STACK_BASE + STACK_SIZE;
        let (b, w) = self.at(ea, n)?;
        if !w {
            return Err(Fault::Mem(ea));
        }
        for k in 0..n as usize {
            b[k] = (v >> (8 * (n as usize - 1 - k))) as u8;
        }
        if !stack {
            for k in 0..n {
                self.written.insert(ea + k);
            }
        }
        Ok(())
    }
}

impl Mem for InitMem<'_> {
    fn ld8(&mut self, ea: u32) -> Result<u8, Fault> {
        self.ld(ea, 1).map(|v| v as u8)
    }
    fn ld16(&mut self, ea: u32) -> Result<u16, Fault> {
        self.ld(ea, 2).map(|v| v as u16)
    }
    fn ld32(&mut self, ea: u32) -> Result<u32, Fault> {
        self.ld(ea, 4).map(|v| v as u32)
    }
    fn ld64(&mut self, ea: u32) -> Result<u64, Fault> {
        self.ld(ea, 8)
    }
    fn st8(&mut self, ea: u32, v: u8) -> Result<(), Fault> {
        self.st(ea, 1, v as u64)
    }
    fn st16(&mut self, ea: u32, v: u16) -> Result<(), Fault> {
        self.st(ea, 2, v as u64)
    }
    fn st32(&mut self, ea: u32, v: u32) -> Result<(), Fault> {
        self.st(ea, 4, v as u64)
    }
    fn st64(&mut self, ea: u32, v: u64) -> Result<(), Fault> {
        self.st(ea, 8, v)
    }
    fn fetch(&mut self, ea: u32) -> Result<u32, Fault> {
        let r = self.img.iter().find(|r| r.kind == IKind::Text && ea >= r.base && ea + 4 <= r.base + r.bytes.len() as u32).ok_or(Fault::Fetch(ea))?;
        let o = (ea - r.base) as usize;
        Ok(u32::from_be_bytes(r.bytes[o..o + 4].try_into().unwrap()))
    }
}

// ---- guest memory ---------------------------------------------------------------------------------------------

struct Live<'a> {
    s: &'a mut St,
    nc: Option<String>,
    data_reads: BTreeSet<String>,
    gecko_reads: BTreeSet<u32>,
}

impl Live<'_> {
    fn img(&self, ea: u32, n: u32) -> Option<&Img> {
        self.s.img.iter().find(|r| ea >= r.base && ea as u64 + n as u64 <= r.base as u64 + r.bytes.len() as u64)
    }
    fn fail(&mut self, why: String) -> Fault {
        self.nc.get_or_insert(why);
        Fault::Mem(0)
    }
    fn ld(&mut self, ea: u32, n: u32) -> Result<u64, Fault> {
        if (0x8000_0000..LOW_END).contains(&ea) && !self.s.space.covers(ea, n) {
            let Some(r) = self.img(ea, n) else {
                let w = sym_at(self.s, ea);
                return Err(self.fail(format!("read unmapped {ea:08X} ({w})")));
            };
            let o = (ea - r.base) as usize;
            let v = r.bytes[o..o + n as usize].iter().fold(0u64, |v, b| (v << 8) | *b as u64);
            match r.kind {
                IKind::Text => {
                    if ea >= self.s.gecko_area.0 && ea < self.s.gecko_area.1 {
                        self.gecko_reads.insert(ea);
                    }
                }
                IKind::Const => {}
                IKind::Data => {
                    // constant-in-practice .data: compiler tables (@NNN: jump tables, literal pools), strings, words
                    // holding code addresses (callback tables); anything else is a mutable global
                    let w = sym_at(self.s, ea);
                    let is_text = n == 4 && self.s.text.iter().any(|&(lo, hi)| (v as u32) >= lo && (v as u32) < hi);
                    let init = (0..n).all(|k| self.s.init_written.contains(&(ea + k)));
                    if w.starts_with('@') || w.contains("assert_msg") || w.contains("...data") || is_text || init || CONST_DATA.iter().any(|c| w.starts_with(c)) {
                        self.data_reads.insert(w.split('+').next().unwrap_or("").to_string());
                    } else {
                        return Err(self.fail(format!("read .data {ea:08X} ({w})")));
                    }
                }
                IKind::Bss if (0..n).all(|k| self.s.init_written.contains(&(ea + k))) => {}
                IKind::Bss => {
                    let w = sym_at(self.s, ea);
                    return Err(self.fail(format!("read .bss {ea:08X} ({w})")));
                }
            }
            return Ok(v);
        }
        let r = match n {
            1 => self.s.space.ld8(ea).map(|v| v as u64),
            2 => self.s.space.ld16(ea).map(|v| v as u64),
            4 => self.s.space.ld32(ea).map(|v| v as u64),
            _ => self.s.space.ld64(ea),
        };
        r.map_err(|_| {
            let d = self.s.space.describe(ea).map(|(r, o)| format!("{r}+{o:#x}")).unwrap_or_else(|| sym_at(self.s, ea));
            self.fail(format!("read faulted {ea:08X} ({d})"))
        })
    }
    fn st(&mut self, ea: u32, n: u32, v: u64) -> Result<(), Fault> {
        if (0x8000_0000..LOW_END).contains(&ea) {
            if let Some(&(_, _, name)) = self.s.ro.iter().find(|r| ea >= r.0 && ea < r.0 + r.1) {
                return Err(self.fail(format!("store to read-only sync copy {name} {ea:08X}")));
            }
            if !self.s.space.covers(ea, n) {
                let k = self.img(ea, n).map(|r| format!("{:?}", r.kind)).unwrap_or("unmapped".into());
                let w = sym_at(self.s, ea);
                return Err(self.fail(format!("store to {k} {ea:08X} ({w})")));
            }
        }
        let r = match n {
            1 => self.s.space.st8(ea, v as u8),
            2 => self.s.space.st16(ea, v as u16),
            4 => self.s.space.st32(ea, v as u32),
            _ => self.s.space.st64(ea, v),
        };
        r.map_err(|_| {
            let d = self.s.space.describe(ea).map(|(r, o)| format!("{r}+{o:#x}")).unwrap_or_else(|| sym_at(self.s, ea));
            self.fail(format!("store faulted {ea:08X} ({d})"))
        })
    }
}

impl Mem for Live<'_> {
    fn ld8(&mut self, ea: u32) -> Result<u8, Fault> {
        self.ld(ea, 1).map(|v| v as u8)
    }
    fn ld16(&mut self, ea: u32) -> Result<u16, Fault> {
        self.ld(ea, 2).map(|v| v as u16)
    }
    fn ld32(&mut self, ea: u32) -> Result<u32, Fault> {
        self.ld(ea, 4).map(|v| v as u32)
    }
    fn ld64(&mut self, ea: u32) -> Result<u64, Fault> {
        self.ld(ea, 8)
    }
    fn st8(&mut self, ea: u32, v: u8) -> Result<(), Fault> {
        self.st(ea, 1, v as u64)
    }
    fn st16(&mut self, ea: u32, v: u16) -> Result<(), Fault> {
        self.st(ea, 2, v as u64)
    }
    fn st32(&mut self, ea: u32, v: u32) -> Result<(), Fault> {
        self.st(ea, 4, v as u64)
    }
    fn st64(&mut self, ea: u32, v: u64) -> Result<(), Fault> {
        self.st(ea, 8, v)
    }
    fn fetch(&mut self, ea: u32) -> Result<u32, Fault> {
        match self.img(ea, 4) {
            Some(r) if r.kind == IKind::Text => {
                let o = (ea - r.base) as usize;
                Ok(u32::from_be_bytes(r.bytes[o..o + 4].try_into().unwrap()))
            }
            _ => Err(self.fail(format!("fetch outside the text {ea:08X}"))),
        }
    }
}

struct TextOnly(Vec<(u32, u32)>);
impl<M: Mem> Bridge<M> for TextOnly {
    fn in_blob(&self, a: u32) -> bool {
        self.0.iter().any(|&(lo, hi)| a >= lo && a < hi)
    }
    fn call(&mut self, _c: &mut Cpu, _m: &mut M, t: u32) -> Result<(), Fault> {
        Err(Fault::Unresolved(t))
    }
}

// ---- the hooks ---------------------------------------------------------------------------------------------------

fn frame() -> i32 {
    melee_core::ct::DBG_FRAME.load(std::sync::atomic::Ordering::Relaxed)
}

fn wants(name: &'static str) -> bool {
    let Some(s) = st() else { return false };
    if s.open.is_some() {
        return false;
    }
    let f = frame();
    if f < s.from || f > s.to {
        return false;
    }
    if let Some(o) = &s.only {
        if !o.iter().any(|n| n == name) {
            return false;
        }
    }
    s.stats.get(name).is_none_or(|x| x.calls < s.max)
}

/// Run the retail function on the live records, keep what it wrote, undo it.
fn pre(name: &'static str, args: &[Arg]) -> Option<usize> {
    let s = st()?;
    if s.open.is_some() {
        return None;
    }
    let fr = frame();
    let Some(&addr) = s.by_name.get(name) else {
        let e = s.stats.entry(name).or_default();
        e.calls += 1;
        e.notcheck += 1;
        *e.reasons.entry("no retail symbol".into()).or_insert((0, fr)) = (1, fr);
        return None;
    };
    s.stats.entry(name).or_default().calls += 1;
    s.space.clear();
    s.space.fn_guest = Some(fn_guest);
    s.space.fn_native = Some(fn_native);
    s.space.void_pointee = Some(void_pointee);
    s.space.add_bytes(STACK_BASE, vec![0; STACK_SIZE as usize]);
    unsafe { sync(s) };
    // arguments
    let mut cpu = Cpu::default();
    let (mut gi, mut fi) = (3usize, 1usize);
    let mut bad = None;
    for a in args {
        match *a {
            Arg::I(v) => {
                cpu.gpr[gi] = v;
                gi += 1;
            }
            Arg::F(v) => {
                cpu.fpr[fi] = (v as f64).to_bits();
                fi += 1;
            }
            Arg::R(p, rec) => {
                if rec == "HSD_GObj" && !p.is_null() {
                    let g = p as *mut melee_types::HSD_GObj;
                    if unsafe { (*g).classifier } as i32 == melee_types::HSD_GOBJ_CLASS_FIGHTER {
                        let fp = unsafe { (*g).user_data } as *mut melee_types::Fighter;
                        if !fp.is_null() {
                            unsafe { map_common_motion_table(s, (*fp).x1C_actionStateList) };
                        }
                    }
                }
                match s.space.handle(p as usize, Some(rec)) {
                    Ok(h) => cpu.gpr[gi] = h,
                    Err(_) => bad = Some(format!("argument {rec} has no guest handle")),
                }
                gi += 1;
            }
            // a pointer to native f32 words (Mtx, Vec): mapped in place as native words (journaled like a record)
            Arg::P(p, len) => {
                if p.is_null() {
                    cpu.gpr[gi] = 0;
                } else {
                    let base = 0x8590_0000 + gi as u32 * 0x1000;
                    unsafe { s.space.add_words(base, p as *mut u8, len as u32) };
                    cpu.gpr[gi] = base;
                }
                gi += 1;
            }
        }
    }
    cpu.gpr[1] = STACK_BASE + STACK_SIZE - 0x100;
    cpu.gpr[2] = R2;
    cpu.gpr[13] = R13;
    cpu.lr = 0;
    cpu.pc = addr;
    s.space.journal = Some(vec![]);
    let mut m = Machine::new_gekko(boot_gqr(cpu));
    let mut br = TextOnly(s.text.clone());
    let mut n = 0u64;
    let mut out = None;
    let mut cov: Vec<(u32, bool)> = vec![];
    let (mut data_reads, mut gecko_reads) = (BTreeSet::new(), BTreeSet::new());
    if let Some(b) = bad {
        out = Some(ROut::NotCheckable(b));
    }
    // MELEE_REFEREE_TRACE=<frame>: every retail instruction of the checks on that frame (debugging a mismatch)
    let trace = std::env::var("MELEE_REFEREE_TRACE").ok().and_then(|v| v.parse::<i32>().ok()) == Some(fr);
    let boundary_at = std::mem::take(&mut s.boundary_at);
    let native_at = std::mem::take(&mut s.native_at);
    while out.is_none() {
        if n >= BUDGET {
            out = Some(ROut::NotCheckable("budget".into()));
            break;
        }
        let ip = m.cpu.pc;
        if n > 0 {
            if let Some(&b) = boundary_at.get(&ip) {
                let c = &m.cpu;
                let mut gpr = [0u32; 8];
                gpr.copy_from_slice(&c.gpr[3..11]);
                let fpr = [c.fpr[1], c.fpr[2], c.fpr[3], c.fpr[4]];
                out = Some(ROut::Boundary { name: b, gpr, fpr });
                break;
            }
            if let Some(stub) = os_stub(s, ip) {
                m.cpu.gpr[3] = stub;
                m.cpu.pc = m.cpu.lr;
                if m.cpu.pc == 0 {
                    out = Some(ROut::Ret { r3: m.cpu.gpr[3], f1: m.cpu.fpr[1] });
                    break;
                }
                n += 1;
                continue;
            }
            if let Some(&nc) = native_at.get(&ip) {
                if let Err(e) = unsafe { native_callee(s, nc, &mut m.cpu) } {
                    out = Some(ROut::NotCheckable(e));
                    break;
                }
                m.cpu.pc = m.cpu.lr;
                if m.cpu.pc == 0 {
                    out = Some(ROut::Ret { r3: m.cpu.gpr[3], f1: m.cpu.fpr[1] });
                    break;
                }
                n += 1;
                continue;
            }
        }
        n += 1;
        let mut live = Live { s, nc: None, data_reads: std::mem::take(&mut data_reads), gecko_reads: std::mem::take(&mut gecko_reads) };
        let w = match live.fetch(ip) {
            Ok(w) => w,
            Err(_) => {
                out = Some(ROut::NotCheckable(live.nc.take().unwrap_or_default()));
                data_reads = live.data_reads;
                gecko_reads = live.gecko_reads;
                break;
            }
        };
        let op = decode(w, ip);
        let r = m.step(&mut live, &mut br);
        if trace {
            let c = &m.cpu;
            eprintln!("rtrace {} {ip:08X} {op:?} r3 {:08X} r4 {:08X} r0 {:08X} f0 {:e} f1 {:e} f2 {:e} cr {:08X}", where_(live.s, ip), c.gpr[3], c.gpr[4], c.gpr[0], f64::from_bits(c.fpr[0]), f64::from_bits(c.fpr[1]), f64::from_bits(c.fpr[2]), c.cr);
        }
        let nc = live.nc.take();
        data_reads = live.data_reads;
        gecko_reads = live.gecko_reads;
        if conditional(&op) && r.is_ok() {
            let taken = match r {
                Ok(Step::Return) => true,
                _ => m.cpu.pc != ip.wrapping_add(4),
            };
            cov.push((ip, taken));
        }
        match r {
            Ok(Step::Continue) => {}
            Ok(Step::Return) => out = Some(ROut::Ret { r3: m.cpu.gpr[3], f1: m.cpu.fpr[1] }),
            Err(e) => {
                let wh = where_(s, ip);
                out = Some(ROut::NotCheckable(match (nc, e) {
                    (Some(why), _) => format!("{why} at {wh}"),
                    (None, Fault::BadOpcode(..)) => format!("unsupported instruction at {wh}"),
                    (None, Fault::Unresolved(t)) => format!("call out of the text {t:08X} at {wh}"),
                    (None, other) => format!("{other:?} at {wh}"),
                }));
            }
        }
    }
    s.boundary_at = boundary_at;
    s.native_at = native_at;
    let out = out.unwrap();
    // what retail wrote, then undo it (in reverse store order)
    let j = s.space.journal.take().unwrap_or_default();
    let mut wrote: BTreeMap<usize, (u8, u8, u32)> = BTreeMap::new();
    for &(p, old, ea) in &j {
        wrote.entry(p).or_insert((old, 0, ea));
    }
    for (p, e) in wrote.iter_mut() {
        e.1 = unsafe { *(*p as *const u8) };
    }
    for &(p, old, _) in j.iter().rev() {
        unsafe { *(p as *mut u8) = old };
    }
    let e = s.stats.entry(name).or_default();
    e.insns += n;
    e.pad_stores += s.space.pad_stores as u64;
    e.data_reads.extend(data_reads);
    e.gecko_reads.extend(gecko_reads);
    if let ROut::NotCheckable(why) = &out {
        e.notcheck += 1;
        let k = e.reasons.entry(why.clone()).or_insert((0, fr));
        k.0 += 1;
        return None;
    }
    for (ip, t) in cov {
        let c = s.cov.entry(ip).or_default();
        if t {
            c.0 += 1;
        } else {
            c.1 += 1;
        }
    }
    // the native objects retail reached: watched for Rust-only writes
    let mut watched = vec![];
    for r in &s.space.regions {
        if let mex_import::remap::Backing::Native { rec, ptr, .. } = &r.backing {
            if let Some(ns) = s.space.layouts.native_size(rec) {
                if !watched.iter().any(|w: &(usize, usize, &'static str, u32)| w.0 == *ptr as usize && w.2 == *rec) {
                    watched.push((*ptr as usize, ns, *rec, r.base));
                }
            }
        }
    }
    let before = watched.iter().map(|w| unsafe { core::slice::from_raw_parts(w.0 as *const u8, w.1) }.to_vec()).collect();
    let boundaries = s.boundary_names.clone();
    s.open = Some(Open { name, frame: fr, out, wrote, watched, before, rust_b: None, boundaries });
    Some(1)
}

/// OS interrupt masking (supervisor SPRs the interpreter does not run; no game state): OSDisableInterrupts returns
/// "were enabled" (1), OSRestoreInterrupts / OSEnableInterrupts the same; the caller's state is untouched.
fn os_stub(s: &St, ip: u32) -> Option<u32> {
    let f = func_at(s, ip)?;
    if f.addr != ip {
        return None;
    }
    matches!(f.name.as_str(), "OSDisableInterrupts" | "OSRestoreInterrupts" | "OSEnableInterrupts").then_some(1)
}

/// A native callee: the engine's function on the registers, its sync-table effects journaled.
unsafe fn native_callee(s: &mut St, name: &str, cpu: &mut Cpu) -> Result<(), String> {
    let seed = melee_fighter::rng::HSD_RandSeedPtr() as *mut u8;
    let old: Vec<u8> = (0..4).map(|k| *seed.add(k)).collect();
    match name {
        "HSD_Randi" => cpu.gpr[3] = melee_fighter::rng::HSD_Randi(cpu.gpr[3] as i32) as u32,
        "HSD_Randf" => cpu.fpr[1] = (melee_fighter::rng::HSD_Randf() as f64).to_bits(),
        _ => return Err(format!("native callee {name} not in the registry")),
    }
    if let Some(j) = &mut s.space.journal {
        for (k, &o) in old.iter().enumerate() {
            j.push((seed as usize + k, o, 0x804D_5F90 + k as u32));
        }
    }
    Ok(())
}

fn snap(o: &Open) -> (Vec<u8>, Vec<Vec<u8>>) {
    let a = o.wrote.keys().map(|p| unsafe { *(*p as *const u8) }).collect();
    let w = o.watched.iter().map(|w| unsafe { core::slice::from_raw_parts(w.0 as *const u8, w.1) }.to_vec()).collect();
    (a, w)
}

fn boundary(name: &'static str, args: &[Arg]) {
    let Some(s) = st() else { return };
    let Some(o) = s.open.as_mut() else { return };
    if o.rust_b.is_some() || !o.boundaries.contains(&name) {
        return;
    }
    let (a, w) = snap(o);
    o.rust_b = Some((name, args.to_vec(), a, w));
}

/// GC offset of native byte `nat` of `rec` (inverse of the layout tables), cached per record.
fn gc_of(s: &mut St, rec: &'static str, nat: usize) -> Option<u32> {
    if !s.inv.contains_key(rec) {
        let ns = s.space.layouts.native_size(rec).unwrap_or(0);
        let gs = s.space.layouts.gc_size(rec).unwrap_or(0);
        let mut v = vec![None; ns];
        for g in 0..gs {
            match s.space.layouts.locate(rec, g) {
                Loc::Byte(b) | Loc::Bool(b) if b < ns => v[b] = v[b].or(Some(g)),
                Loc::Ptr(b, 0, _) => {
                    for k in 0..core::mem::size_of::<usize>() {
                        if b + k < ns {
                            v[b + k] = v[b + k].or(Some(g));
                        }
                    }
                }
                _ => {}
            }
        }
        s.inv.insert(rec, v);
    }
    s.inv[rec].get(nat).copied().flatten()
}

fn post(_t: usize, ret: Ret) {
    let Some(s) = st() else { return };
    let Some(mut o) = s.open.take() else { return };
    // MELEE_REFEREE_NEGCTL=1: flip one bit of the first byte retail wrote (a negative control: must be a mismatch)
    if std::env::var_os("MELEE_REFEREE_NEGCTL").is_some() {
        if let Some(e) = o.wrote.values_mut().next() {
            e.1 ^= 1;
        }
    }
    let mut mism: Vec<String> = vec![];
    // Rust's state: at its first boundary call, else now
    let (rust_bytes, rust_watched) = match &o.rust_b {
        Some((_, _, a, w)) => (a.clone(), w.clone()),
        None => snap(&o),
    };
    match (&o.out, &o.rust_b) {
        (ROut::Ret { r3, f1 }, None) => match ret {
            Ret::Void => {}
            Ret::I(v) => {
                if v != *r3 {
                    mism.push(format!("return r3: rust {v:08X} retail {r3:08X}"));
                }
            }
            Ret::F(v) => {
                let r = (f64::from_bits(*f1) as f32).to_bits();
                if r != v.to_bits() {
                    mism.push(format!("return f1: rust {:08X} ({v}) retail {r:08X} ({})", v.to_bits(), f32::from_bits(r)));
                }
            }
            Ret::P(p) => {
                let h = s.space.native(*r3).unwrap_or(usize::MAX);
                if h != p as usize {
                    mism.push(format!("return pointer: rust {:x} retail handle {r3:08X} -> {h:x}", p as usize));
                }
            }
        },
        (ROut::Ret { .. }, Some((b, ..))) => mism.push(format!("rust calls {b}, retail returns without it")),
        (ROut::Boundary { name, .. }, None) => mism.push(format!("retail calls {name}, rust returns without it")),
        (ROut::Boundary { name, gpr, fpr }, Some((b, args, ..))) => {
            if name != b {
                mism.push(format!("boundary: retail calls {name}, rust {b}"));
            } else {
                let (mut gi, mut fi) = (0usize, 0usize);
                for (k, a) in args.iter().enumerate() {
                    match *a {
                        Arg::I(v) => {
                            if gi < 8 && gpr[gi] != v {
                                mism.push(format!("{name} arg{k}: rust {v:08X} retail {:08X}", gpr[gi]));
                            }
                            gi += 1;
                        }
                        Arg::R(p, _) => {
                            let h = if gi < 8 { s.space.native(gpr[gi]).unwrap_or(usize::MAX) } else { usize::MAX };
                            if h != p as usize {
                                mism.push(format!("{name} arg{k}: rust {:x} retail handle -> {h:x}", p as usize));
                            }
                            gi += 1;
                        }
                        Arg::F(v) => {
                            let r = if fi < 4 { (f64::from_bits(fpr[fi]) as f32).to_bits() } else { 0 };
                            if r != v.to_bits() {
                                mism.push(format!("{name} arg{k}: rust {:08X} ({v}) retail {r:08X} ({})", v.to_bits(), f32::from_bits(r)));
                            }
                            fi += 1;
                        }
                        Arg::P(..) => {}
                    }
                }
            }
        }
        (ROut::NotCheckable(_), _) => {}
    }
    // every byte retail wrote: grouped per (guest field word)
    let mut by_field: BTreeMap<u32, (Vec<u8>, Vec<u8>)> = BTreeMap::new();
    for (i, (_p, (_old, rv, ea))) in o.wrote.iter().enumerate() {
        let ru = rust_bytes[i];
        if ru != *rv {
            let e = by_field.entry(ea & !3).or_default();
            e.0.push(*rv);
            e.1.push(ru);
        }
    }
    for (ea, _) in by_field.iter().take(12) {
        let d = s.space.describe(*ea).map(|(r, off)| format!("{r}+{off:#x}")).unwrap_or_else(|| sym_at(s, *ea));
        let words: Vec<(u32, u8, u8)> = o.wrote.iter().enumerate().filter(|(_, (_, (_, _, e)))| e & !3 == *ea).map(|(i, (_, (_, rv, e)))| (*e, *rv, rust_bytes[i])).collect();
        let (mut rt, mut ru) = (String::new(), String::new());
        for (e, rv, uv) in &words {
            rt += &format!("{:x}:{rv:02X} ", e & 3);
            ru += &format!("{:x}:{uv:02X} ", e & 3);
        }
        mism.push(format!("{d}: retail [{}] rust [{}]", rt.trim(), ru.trim()));
    }
    if by_field.len() > 12 {
        mism.push(format!("... {} more fields", by_field.len() - 12));
    }
    // Rust-only writes in the watched objects
    let written: BTreeSet<usize> = o.wrote.keys().copied().collect();
    let mut only: BTreeMap<String, u32> = BTreeMap::new();
    for (wi, w) in o.watched.iter().enumerate() {
        let (b, a) = (&o.before[wi], &rust_watched[wi]);
        for k in 0..w.1 {
            if b[k] != a[k] && !written.contains(&(w.0 + k)) {
                match gc_of(s, w.2, k) {
                    Some(g) => *only.entry(format!("{}+{:#x}", w.2, g & !3)).or_default() += 1,
                    None => {}
                }
            }
        }
    }
    for (f, _) in only.iter().take(12) {
        mism.push(format!("rust wrote {f}, retail did not"));
    }
    if only.len() > 12 {
        mism.push(format!("... {} more rust-only fields", only.len() - 12));
    }
    if std::env::var_os("MELEE_REFEREE_LOG").is_some() {
        let k = match &o.out {
            ROut::Ret { .. } => "ret".to_string(),
            ROut::Boundary { name, gpr, .. } => format!("{name}({:#x})", gpr[1]),
            ROut::NotCheckable(w) => w.clone(),
        };
        eprintln!("livereferee: {} frame {} {k} {}", o.name, o.frame, if mism.is_empty() { "match".into() } else { mism.join(" | ") });
    }
    let e = s.stats.entry(o.name).or_default();
    e.compared += 1;
    if let ROut::Boundary { name, .. } = &o.out {
        *e.boundary.entry(name.to_string()).or_default() += 1;
    }
    if mism.is_empty() {
        e.matched += 1;
    } else {
        e.mismatched += 1;
        if e.mismatches.len() < 40 {
            e.mismatches.push(format!("frame {}: {}", o.frame, mism.join(" | ")));
        }
    }
}

/// Write the report (called at the end of the run).
pub fn report() {
    let Some(s) = st() else { return };
    let mut out = String::new();
    out += &format!("# livereferee {}\n", s.gecko_desc);
    let names: Vec<&'static str> = s.stats.keys().copied().collect();
    for name in names {
        let (own_hit, own_tot) = match s.by_name.get(name).and_then(|&a| func_at(s, a).cloned()) {
            Some(f) => {
                let mut hit = 0;
                let mut tot = 0;
                for a in (f.addr..f.addr + f.size).step_by(4) {
                    let Some(w) = img_word(s, a) else { continue };
                    if conditional(&decode(w, a)) {
                        tot += 2;
                        if let Some(c) = s.cov.get(&a) {
                            hit += (c.0 > 0) as u32 + (c.1 > 0) as u32;
                        }
                    }
                }
                (hit, tot)
            }
            None => (0, 0),
        };
        let e = &s.stats[name];
        out += &format!(
            "S\t{name}\tcalls {}\tcompared {}\tmatch {}\tmismatch {}\tnotcheck {}\town_cov {}/{}\tinsns {}\tpad_stores {}\n",
            e.calls, e.compared, e.matched, e.mismatched, e.notcheck, own_hit, own_tot, e.insns, e.pad_stores
        );
        for (b, n) in &e.boundary {
            out += &format!("B\t{name}\t{b}\t{n}\n");
        }
        for (r, (n, f)) in &e.reasons {
            out += &format!("N\t{name}\t{n}\tfirst frame {f}\t{r}\n");
        }
        for m in &e.mismatches {
            out += &format!("M\t{name}\t{m}\n");
        }
        if !e.data_reads.is_empty() {
            out += &format!("D\t{name}\t.data read as constant: {}\n", e.data_reads.iter().cloned().collect::<Vec<_>>().join(","));
        }
        if !e.gecko_reads.is_empty() {
            let v: Vec<String> = e.gecko_reads.iter().take(16).map(|a| format!("{a:08X}")).collect();
            out += &format!("G\t{name}\tgecko-area loads: {}\n", v.join(","));
        }
    }
    for (a, (t, n)) in &s.cov {
        out += &format!("C\t{a:08X}\t{t}\t{n}\n");
    }
    match std::fs::File::create(&s.out) {
        Ok(mut f) => {
            let _ = f.write_all(out.as_bytes());
        }
        Err(e) => eprintln!("livereferee: report: {e}"),
    }
    let _ = (&s.native_callees, &s.synced);
}

#[allow(dead_code)]
fn unused(_: Op) {}
