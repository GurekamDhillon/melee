//! An m-ex fighter's `ftFunction` PPC code run in the match: the native form of the C build's
//! `pc/platform/gw_mex_ftfunction_runtime.c` interpreter wiring (gw_mex_interp_*, the MoveLogic trampolines).
//!
//! The blob runs through mex_import's interpreter (`ppc::Machine`, PowerPC-exact float mode: fused multiply-add,
//! Gekko's fctiw/fctiwz) over `remap::GuestSpace`, which gives the native little-endian `Fighter` / `HSD_GObj` guest
//! addresses and rewrites every field access through the GameCube offset tables, so the mod writes the very objects
//! the native match simulates. Guest memory:
//!
//! * the relocated blob (code and its own data) at `CODE_BASE` (+ per fighter), within rel24 reach of the game's text
//!   so a `bl` to a vanilla function decodes to that function;
//! * the fighter's PlXx.dat data section as HSD_ArchiveParse leaves it (big-endian, relocated) at `DAT_BASE` (+ per
//!   fighter); the guest's `fp->ft_data` reads that ftData (its attributes and article tables are the mod's own
//!   layouts, which only the guest reads);
//! * `fp->dat_attrs_backup`, the 0x424-byte buffer of the game heap, as raw guest bytes: the mod's onLoad copies its
//!   private attributes there and points `fp->dat_attrs` at it, as m-ex does;
//! * r13 / r2 small data: the DOL's .sdata / .sdata2 around the two bases (the float constants the code loads).
//!
//! Rollback: every byte the guest can write lives in the game heap, in a native game object, or in this module's
//! statics and the arena-allocated regions they own, all of which the snapshot covers. The machine itself is made per
//! call. Nothing here reads the clock or iterates a hash map.
//!
//! Out-of-blob calls go through [`call_out`]: memset/memcpy, MEX_IndexFighterItem, and the vanilla functions of
//! `mex_bridge` (generated: arguments from r3.. / f1.., pointers translated between guest handles and native objects,
//! small records the guest keeps on its stack marshalled in and out). Any other target ends that callback and is
//! counted by name (`MELEE_MEX_REPORT=1` prints the counters at exit of each fighter's first faults).
#![allow(static_mut_refs)]

use core::ffi::c_void;
use mex_import::mexfn::{Blob, Owner};
use mex_import::ppc::{Bridge, Cpu, Fault, Machine, Mem, Step};
use mex_import::remap::GuestSpace;
use melee_types::{Fighter, HSD_GObj, MotionState};
use std::collections::BTreeMap;

type G = *mut HSD_GObj;

pub const SLOT_NAMES: [&str; 46] = [
    "onLoad", "OnRespawn", "onDestroy", "MoveLogic", "SpecialN", "SpecialNAir", "SpecialS", "SpecialSAir", "SpecialHi",
    "SpecialHiAir", "SpecialLw", "SpecialLwAir", "onAbsorb", "OnItemPickup", "onMakeItemInvisible", "onMakeItemVisible",
    "OnItemRelease", "OnItemPickup2", "onUnknownItemRelated", "onApplyHeadItem", "onRemoveHeadItem", "EyeTextureDamaged",
    "EyeTextureNormal", "onFrame", "onActionStateChange", "onReapplyAttr", "onModelRender", "onShadowRender",
    "onUnknownMultijump", "onActionStateChangeWhileEyeTextureIsChanged", "onTwoEntryTable", "onFloat", "onDoubleJump",
    "onZair", "onLanding", "onFSmash", "onUSmash", "onDSmash", "onGetExtResultAnim", "onIndexExtResultAnim",
    "MoveLogicDemo", "onIntroL", "onIntroR", "onTaunt", "onCatch", "GetTrailData",
];
const SLOT_MOVE_LOGIC: u32 = 3;

/// (m-ex ftFunction slot, the engine's one-argument dispatch event): gw_mex_ftfunction_runtime.c's gw_mex_hook rows.
const SLOT_EVENTS: [(u32, usize); 20] = {
    use melee_fighter::platform as p;
    [
        (0, p::MEX_EVENT_ON_LOAD),
        (1, p::MEX_EVENT_ON_DEATH),
        (2, p::MEX_EVENT_ON_DESTROY),
        (23, p::MEX_EVENT_ON_FRAME),
        (12, p::MEX_EVENT_ON_ABSORB),
        (4, p::MEX_EVENT_SPECIAL_N),
        (5, p::MEX_EVENT_SPECIAL_N_AIR),
        (6, p::MEX_EVENT_SPECIAL_S),
        (7, p::MEX_EVENT_SPECIAL_S_AIR),
        (8, p::MEX_EVENT_SPECIAL_HI),
        (9, p::MEX_EVENT_SPECIAL_HI_AIR),
        (10, p::MEX_EVENT_SPECIAL_LW),
        (11, p::MEX_EVENT_SPECIAL_LW_AIR),
        (32, p::MEX_EVENT_ON_DOUBLE_JUMP),
        (34, p::MEX_EVENT_ON_LANDING),
        (35, p::MEX_EVENT_ON_FSMASH),
        (36, p::MEX_EVENT_ON_USMASH),
        (37, p::MEX_EVENT_ON_DSMASH),
        (43, p::MEX_EVENT_ON_TAUNT),
        (44, p::MEX_EVENT_ON_CATCH),
    ]
};

const CODE_BASE: u32 = 0x8180_0000;
const CODE_END: u32 = 0x8200_0000;
const DAT_BASE: u32 = 0x8800_0000;
const STACK_BASE: u32 = 0x8170_0000;
const STACK_SIZE: u32 = 0x2_0000;
/// fighter.c: HSD_ObjAllocInit(&fighter_dat_attrs_alloc_data, 0x424, 4).
const ATTRS_SIZE: u32 = 0x424;
const ATTRS_BASE: u32 = 0x8600_0000; // + player-free index * 0x1000
const SCRATCH_BASE: u32 = 0x8700_0000; // marshalled records of one call
/// NTSC 1.02 `_SDA_BASE_` (r13) and `_SDA2_BASE_` (r2).
const R13: u32 = 0x804D_B6A0;
const R2: u32 = 0x804D_F9E0;
const BUDGET: u64 = 4_000_000;
/// Fighter GC offsets the runtime itself reads (fighter.h): motion_id, the MotionState table base index (x18),
/// ft_data, dat_attrs_backup.
const FT_MOTION_ID: u32 = 0x10;
const FT_X18: u32 = 0x18;
const FT_FT_DATA: u32 = 0x10C;
const FT_DAT_ATTRS_BACKUP: u32 = 0x2D8;

const MEMSET: u32 = 0x8000_3100;
const MEMCPY: u32 = 0x8000_31F4;
/// gw_mex_ftfunction_runtime.c GW_MEX_GUEST_INDEX_ITEM (MEX_IndexFighterItem).
const MEX_INDEX_FIGHTER_ITEM: u32 = 0x803D_7058;
/// GW_MEX_GUEST_GET_FT_ITEM_ID (MEX_GetFtItemID).
const MEX_GET_FT_ITEM_ID: u32 = 0x803D_7088;
const EF_SYNC_SPAWN: u32 = 0x8005_FDDC;
const HSD_MEM_ALLOC: u32 = 0x8037_F1E4;
/// GW_MEX_GUEST_GET_DATA (MEX_GetData).
const MEX_GET_DATA: u32 = 0x803D_7094;
const HSD_FREE: u32 = 0x8037_F1B0;
/// GW_MEX_GUEST_CALLOC (m-ex calloc(size)).
const MEX_CALLOC: u32 = 0x803D_706C;
/// Guest addresses of the game-heap blocks the guest allocates (HSD_MemAlloc).
const HEAP_GUEST: u32 = 0x8A00_0000;

#[derive(Default, Debug, Clone)]
pub struct SlotStats {
    pub calls: u64,
    pub returned: u64,
    pub insns: u64,
    pub faults: BTreeMap<String, u64>,
}

struct Fighter_ {
    file: String,
    blob: Blob,
    code_base: u32,
    ft_data: u32,
    /// slot -> stats; MoveLogic callbacks count under 1000 + which.
    stats: BTreeMap<u32, SlotStats>,
    /// The native MotionState table built from the blob's MoveLogic table (kept alive here).
    move_table: Vec<MotionState>,
    /// Guest callbacks of each MoveLogic row: [anim, input, phys, coll].
    move_guest: Vec<[u32; 4]>,
}

struct State {
    space: GuestSpace,
    kinds: BTreeMap<i32, Fighter_>,
    code_next: u32,
    dat_next: u32,
    /// Out-of-blob calls by target: (bound, unresolved).
    bound: BTreeMap<u32, u64>,
    unresolved: BTreeMap<u32, u64>,
    /// MEX_IndexFighterItem: (fighter kind, item id) -> ItemDesc guest address.
    articles: BTreeMap<(u32, u32), u32>,
    /// Pool trampolines: slot -> (kind, guest address).
    pool: Vec<(i32, u32)>,
    /// dat_attrs_backup buffers given a guest address: native pointer -> guest base.
    attrs: Vec<(usize, u32)>,
    small_data: bool,
    depth: u32,
    heap_next: u32,
    /// Native retail callback -> its vanilla address (fn_guest).
    retail: BTreeMap<usize, u32>,
}

static mut ST: Option<State> = None;

/// The debug switches, read once (they are looked up on every call out of the blob).
static REPORT: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
static CALLTRACE: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
fn flag(c: &std::sync::OnceLock<bool>, name: &str) -> bool {
    *c.get_or_init(|| std::env::var_os(name).is_some())
}

fn st() -> &'static mut State {
    unsafe {
        ST.get_or_insert_with(|| {
            let mut space = GuestSpace::new();
            space.add_bytes(STACK_BASE, vec![0; STACK_SIZE as usize]);
            space.fn_native = Some(fn_native);
            space.fn_guest = Some(fn_guest);
            if std::env::var_os("MELEE_MEX_TRACE").is_some() {
                space.trace = Some(Vec::new());
            }
            State {
                space,
                kinds: BTreeMap::new(),
                code_next: CODE_BASE,
                dat_next: DAT_BASE,
                bound: BTreeMap::new(),
                unresolved: BTreeMap::new(),
                articles: BTreeMap::new(),
                pool: Vec::new(),
                attrs: Vec::new(),
                small_data: false,
                depth: 0,
                heap_next: HEAP_GUEST,
                retail: BTreeMap::new(),
            }
        })
    }
}

include!("mex_guest_pool.rs");

unsafe extern "C" fn pool_tramp<const I: usize>(g: G) {
    let Some(&(kind, addr)) = st().pool.get(I) else { return };
    run_at(kind, addr, 2000, g);
}

fn in_code(a: u32) -> bool {
    unsafe { ST.as_ref() }.is_some_and(|s| s.kinds.values().any(|k| a >= k.code_base && a < k.code_base + k.blob.code.len() as u32))
}

fn kind_of_code(a: u32) -> Option<i32> {
    st().kinds.iter().find(|(_, k)| a >= k.code_base && a < k.code_base + k.blob.code.len() as u32).map(|(&n, _)| n)
}

/// A guest code address as a native callback: its pool trampoline (made on first use).
fn fn_native(a: u32) -> Option<usize> {
    let Some(kind) = kind_of_code(a) else {
        // a retail callback the guest installs (fp->cb = ftCo_..., the vanilla address in its code): the port's
        return vanilla_gobj_fn(a).map(|f| f as usize);
    };
    let s = st();
    let i = match s.pool.iter().position(|&(k, x)| k == kind && x == a) {
        Some(i) => i,
        None => {
            if s.pool.len() == POOL.len() {
                eprintln!("mexguest: trampoline pool full ({}): guest {a:08X} left unwired", POOL.len());
                return None;
            }
            s.pool.push((kind, a));
            s.pool.len() - 1
        }
    };
    Some(POOL[i] as usize)
}

fn fn_guest(p: usize) -> Option<u32> {
    let s = unsafe { ST.as_mut()? };
    if let Some(a) = s.pool.iter().enumerate().find(|(i, _)| POOL[*i] as usize == p).map(|(_, &(_, a))| a) {
        return Some(a);
    }
    // a native retail callback in a field the guest reads (to compare or to call): its vanilla address
    if s.retail.is_empty() {
        for n in crate::mex_bridge::GOBJ_FN_NAMES {
            if let (Some(f), Some(a)) = (crate::mex_bridge::gobj_fn(n), mex_import::layout::vanilla_addr(n)) {
                s.retail.insert(f as usize, a);
            }
        }
    }
    s.retail.get(&p).copied()
}

/// The native function a vanilla guest address names, for a one-argument gobj callback (MoveLogic rows that keep a
/// retail function, fn-pointer fields the guest copies from vanilla code).
fn vanilla_gobj_fn(a: u32) -> Option<unsafe extern "C" fn(G)> {
    let name = mex_import::layout::vanilla_name(a)?;
    crate::mex_bridge::gobj_fn(name)
}

/// `gw_Mex_FtFunctionInstall(kind, archive data, size)`: relocate the fighter file's `ftFunction` and register its
/// overrides with the engine's m-ex dispatch tables. `k` is the m-ex internal id, `file` / `symbol` its PlXx.dat and
/// ftData public.
pub fn install(kind: i32, file: &str, symbol: &str) {
    if std::env::var_os("MELEE_MEX_GUEST_OFF").is_some() {
        eprintln!("mexguest: MELEE_MEX_GUEST_OFF: kind {kind} runs its clone base");
        return;
    }
    let s = st();
    if s.kinds.contains_key(&kind) {
        return;
    }
    let disc = &mut crate::host::Host::get().disc;
    let dat = match disc.read(file) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("mexguest: kind {kind}: {file}: {e}");
            return;
        }
    };
    if !s.small_data {
        s.small_data = true;
        // .sdata / .sbss around r13, .sdata2 / .sbss2 around r2 (zero where the DOL has no bytes: the bss)
        for (lo, hi) in [(R13 - 0x8000, R13 + 0x8000), (R2 - 0x8000, R2 + 0x8000)] {
            let mut v = vec![0u8; (hi - lo) as usize];
            let mut a = lo;
            while a < hi {
                if let Ok(b) = disc.dol_bytes(a, 0x100) {
                    v[(a - lo) as usize..(a - lo) as usize + 0x100].copy_from_slice(&b);
                }
                a += 0x100;
            }
            s.space.add_bytes(lo, v);
        }
    }
    let ar = match mex_import::hsd::Archive::parse(&dat) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("mexguest: kind {kind}: {file}: {e}");
            return;
        }
    };
    let Some(off) = ar.public("ftFunction") else {
        eprintln!("mexguest: kind {kind}: {file} has no ftFunction");
        return;
    };
    let code_base = s.code_next;
    let blob = match mex_import::mexfn::parse_at_base(&ar, off, Owner::Fighter, format!("{file}:ftFunction"), code_base) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("mexguest: kind {kind}: {file}: ftFunction: {e}");
            return;
        }
    };
    if code_base + blob.code.len() as u32 > CODE_END {
        eprintln!("mexguest: kind {kind}: no code space left for {file}");
        return;
    }
    s.code_next = (code_base + blob.code.len() as u32 + 0xFFF) & !0xFFF;
    s.space.add_bytes(code_base, blob.code.clone());
    let dat_base = s.dat_next;
    let mut data = ar.data.clone();
    for &o in &ar.relocs {
        let o = o as usize;
        if o + 4 <= data.len() {
            let w = u32::from_be_bytes(data[o..o + 4].try_into().unwrap());
            data[o..o + 4].copy_from_slice(&w.wrapping_add(dat_base).to_be_bytes());
        }
    }
    s.dat_next = (dat_base + data.len() as u32 + 0xFFFF) & !0xFFFF;
    s.space.add_bytes(dat_base, data);
    let Some(fd) = ar.public(symbol) else {
        eprintln!("mexguest: kind {kind}: {file} has no {symbol}");
        return;
    };
    let mut f = Fighter_ { file: file.into(), blob, code_base, ft_data: dat_base + fd, stats: BTreeMap::new(), move_table: Vec::new(), move_guest: Vec::new() };
    if std::env::var_os("MELEE_MEX_CALLS").is_some() {
        // every vanilla function the blob links to (its relocations with an absolute 0x8xxxxxxx target)
        let mut t: Vec<u32> = f.blob.relocs.iter().filter(|r| r.flag == 0x0A && r.target & 0xF000_0000 == 0x8000_0000).map(|r| r.target).collect();
        t.sort();
        t.dedup();
        for a in t {
            let n = mex_import::layout::vanilla_name(a).unwrap_or("?");
            let b = mex_import::layout::vanilla_name(a).is_some_and(|n| unsafe { crate::mex_bridge::bridged(n) });
            eprintln!("mexcall: {kind} {a:08X} {n}{}", if b { "" } else { " UNBOUND" });
        }
    }
    let mut wired = Vec::new();
    for (slot, off) in f.blob.overrides.clone() {
        if let Some(&(_, ev)) = SLOT_EVENTS.iter().find(|e| e.0 == slot) {
            melee_fighter::platform::Mex_SetHook(ev, kind as usize, Some(EVENT_TRAMPS[ev]));
            wired.push(SLOT_NAMES[slot as usize]);
        } else if slot == SLOT_MOVE_LOGIC {
            build_move_table(&mut f, code_base + off);
            wired.push("MoveLogic");
        }
    }
    eprintln!(
        "mexguest: kind {kind}: {file} ftFunction {} bytes at {code_base:08X}, {} overrides; running {}",
        f.blob.code.len(),
        f.blob.overrides.len(),
        wired.join(" ")
    );
    s.kinds.insert(kind, f);
}

/// The MoveLogic table (a MotionState[] in the blob, gw_mex_movelogic_wire): a native copy whose anim/input/phys/coll
/// callbacks are trampolines into the row's guest code (or the retail function a row keeps), cam_cb the retail one.
fn build_move_table(f: &mut Fighter_, table: u32) {
    let rel = table - f.code_base;
    let end = f.blob.symbols.iter().find(|s| s.0 == rel && s.1 > rel).map(|s| s.1).unwrap_or_else(|| {
        f.blob.overrides.iter().filter(|o| o.0 & 0x8000_0000 == 0 && o.1 > rel).map(|o| o.1).min().unwrap_or(f.blob.code.len() as u32)
    });
    let n = ((end - rel) / 0x20) as usize;
    let rd = |o: u32| u32::from_be_bytes(f.blob.code[o as usize..o as usize + 4].try_into().unwrap());
    let s = st();
    for i in 0..n {
        let r = rel + i as u32 * 0x20;
        let mut m = MotionState::zeroed();
        m.anim_id = rd(r) as _;
        m.x4_flags = rd(r + 4) as _;
        // the anon_8 word (move_id << 24 and the flag bits) as the retail tables spell it (retail_ms.rs: `_x0 = (id << 24 | ...).to_be()`, GC byte order)
        m.anon_8._x0 = rd(r + 8).to_be();
        let mut g = [0u32; 4];
        let mut cbs: [melee_types::HSD_GObjEvent; 5] = [None; 5];
        for (w, cb) in cbs.iter_mut().enumerate() {
            let a = rd(r + 0xC + 4 * w as u32);
            if w < 4 {
                g[w] = a;
            }
            *cb = if a == 0 {
                None
            } else if a >= f.code_base && a < f.code_base + f.blob.code.len() as u32 && w < 4 {
                Some(MOVE_TRAMPS[w])
            } else {
                vanilla_gobj_fn(a)
            };
        }
        m.anim_cb = cbs[0];
        m.input_cb = cbs[1];
        m.phys_cb = cbs[2];
        m.coll_cb = cbs[3];
        m.cam_cb = cbs[4];
        if std::env::var_os("MELEE_MEX_CALLS").is_some() {
            eprintln!("mexmove: row {i} anim {:#x} flags {:#x} word8 {:#010x} cbs {:08X?}", rd(r), rd(r + 4), rd(r + 8), (0..5).map(|w| rd(r + 0xC + 4 * w)).collect::<Vec<_>>());
        }
        f.move_table.push(m);
        f.move_guest.push(g);
    }
}

/// The installed MoveLogic table of `kind` (None: the retail / clone-base one).
pub fn move_logic_table(kind: i32) -> Option<*mut MotionState> {
    let f = unsafe { ST.as_mut()? }.kinds.get_mut(&kind)?;
    (!f.move_table.is_empty()).then(|| f.move_table.as_mut_ptr())
}

const fn event_tramps() -> [unsafe extern "C" fn(G); melee_fighter::platform::MEX_EVENT_COUNT] {
    let mut t: [unsafe extern "C" fn(G); melee_fighter::platform::MEX_EVENT_COUNT] = [ev_tramp::<0>; melee_fighter::platform::MEX_EVENT_COUNT];
    macro_rules! set { ($($e:literal)*) => { $( t[$e] = ev_tramp::<$e>; )* } }
    set!(0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27 28 29 30 31 32 33 34 35 36 37 38 39);
    t
}
static EVENT_TRAMPS: [unsafe extern "C" fn(G); melee_fighter::platform::MEX_EVENT_COUNT] = event_tramps();
static MOVE_TRAMPS: [unsafe extern "C" fn(G); 4] = [move_tramp::<0>, move_tramp::<1>, move_tramp::<2>, move_tramp::<3>];

unsafe fn kind_of(g: G) -> Option<i32> {
    if g.is_null() || (*g).user_data.is_null() {
        return None;
    }
    Some((*((*g).user_data as *mut Fighter)).kind as i32)
}

unsafe extern "C" fn ev_tramp<const E: usize>(g: G) {
    let Some(kind) = kind_of(g) else { return };
    let Some(&(slot, _)) = SLOT_EVENTS.iter().find(|e| e.1 == E) else { return };
    let Some(f) = st().kinds.get(&kind) else { return };
    let Some(&(_, off)) = f.blob.overrides.iter().find(|o| o.0 == slot) else { return };
    let a = f.code_base + off;
    let faulted = run_at_f(kind, a, slot, g).1;
    // an onLoad that could not finish leaves the fighter half set up: the clone base's onLoad runs after it (its
    // attributes and article tables), as the native match did before the guest code ran
    if faulted && E == melee_fighter::platform::MEX_EVENT_ON_LOAD {
        if let Some(v) = melee_fighter::tu::ftdata::ftData_OnLoad.get(kind as usize).copied().flatten() {
            eprintln!("mexguest: kind {kind}: onLoad faulted - the clone base's onLoad runs");
            v(g);
        }
    }
}

unsafe extern "C" fn move_tramp<const W: usize>(g: G) {
    let Some(kind) = kind_of(g) else { return };
    let fp = (*g).user_data as *mut u8;
    let s = st();
    let (Some(_), Some(_)) = (bind_fighter(s, g), Some(())) else { return };
    let h = s.space.handle_of(fp).unwrap_or(0);
    let mid = s.space.ld32(h + FT_MOTION_ID).unwrap_or(0);
    let x18 = s.space.ld32(h + FT_X18).unwrap_or(0);
    let Some(f) = s.kinds.get(&kind) else { return };
    let idx = mid.wrapping_sub(x18) as usize;
    let Some(row) = f.move_guest.get(idx) else { return };
    let a = row[W];
    if a != 0 {
        run_at(kind, a, 1000 + W as u32, g);
    }
}

/// Register the gobj and its fighter (and the fighter's dat_attrs_backup buffer) in the guest space; the gobj's
/// guest handle.
unsafe fn bind_fighter(s: &mut State, g: G) -> Option<u32> {
    let fp = (*g).user_data as *mut Fighter;
    if fp.is_null() {
        return None;
    }
    let gh = match s.space.handle_of(g as *mut u8) {
        Some(h) => h,
        None => s.space.add_native(None, "HSD_GObj", g as *mut u8).ok()?,
    };
    if s.space.handle_of(fp as *mut u8).is_none() {
        let fh = s.space.add_native(None, "Fighter", fp as *mut u8).ok()?;
        if let Some(f) = s.kinds.get(&((*fp).kind as i32)) {
            s.space.ptr_override.insert((fh, FT_FT_DATA), f.ft_data);
        }
        let ab = (*fp).dat_attrs_backup as usize;
        if ab != 0 && !s.attrs.iter().any(|a| a.0 == ab) {
            let base = ATTRS_BASE + s.attrs.len() as u32 * 0x1000;
            s.space.add_raw(base, ab as *mut u8, ATTRS_SIZE);
            s.attrs.push((ab, base));
        }
        let _ = FT_DAT_ATTRS_BACKUP;
    }
    Some(gh)
}

/// Vanilla pointer globals of the small-data areas the guest reads (`lwz r9, p_ftCommonData`): their guest handles
/// written into the r13 / r2 bytes before a run (the DOL's image holds 0 there, the bss).
unsafe fn sync_globals(s: &mut State) {
    let rows: [(u32, usize, &'static str); 1] = [(0x804D_6554, melee_runtime::shared::p_ftCommonData as usize, "ftCommonData")];
    for (a, p, rec) in rows {
        if let Ok(h) = s.space.handle(p, Some(rec)) {
            let _ = s.space.st32(a, h);
        }
    }
}

/// Run guest code at `a` with r3 = the gobj (gw_ppc_call). Re-entrant: a nested call (a bridged function that calls
/// back into guest code) runs on the stack below the outer one.
unsafe fn run_at(kind: i32, a: u32, slot: u32, g: G) -> u32 {
    run_at_f(kind, a, slot, g).0
}

/// `run_at`, also saying whether the run faulted.
unsafe fn run_at_f(kind: i32, a: u32, slot: u32, g: G) -> (u32, bool) {
    let s = st();
    let Some(gh) = bind_fighter(s, g) else { return (0, true) };
    sync_globals(s);
    let mut cpu = Cpu::default();
    cpu.gpr[1] = STACK_BASE + STACK_SIZE - 0x100 - s.depth * 0x2000;
    cpu.gpr[2] = R2;
    cpu.gpr[13] = R13;
    cpu.gpr[3] = gh;
    cpu.lr = 0;
    cpu.pc = a;
    let (r, faulted) = run_cpu(kind, slot, cpu);
    (r.gpr[3], faulted)
}

unsafe fn run_cpu(kind: i32, slot: u32, cpu: Cpu) -> (Cpu, bool) {
    let s = st();
    if s.depth >= 8 {
        return (cpu, true);
    }
    s.depth += 1;
    let mut m = Machine::new_gekko(cpu);
    let mut n = 0u64;
    let mut fault = None;
    let mut br = Br;
    loop {
        if n >= BUDGET {
            fault = Some(Fault::Budget);
            break;
        }
        n += 1;
        match m.step(&mut st().space, &mut br) {
            Ok(Step::Continue) => {}
            Ok(Step::Return) => break,
            Err(f) => {
                fault = Some(f);
                break;
            }
        }
    }
    let s = st();
    s.depth -= 1;
    if let Some(t) = s.space.trace.as_mut() {
        for r in t.drain(..) {
            if r.store {
                eprintln!("mextrace: slot {slot} store {}+{:#x} ({} bytes)", r.rec, r.gc_off, r.size);
            }
        }
    }
    let fk = fault.map(|f| match f {
        Fault::Unresolved(t) => format!("unresolved {t:08X} {}", mex_import::layout::vanilla_name(t).unwrap_or("?")),
        Fault::Mem(a) => format!("mem {a:08X} at {}", where_(kind, m.cpu.pc.wrapping_sub(4))),
        other => format!("{other:?} at {}", where_(kind, m.cpu.pc.wrapping_sub(4))),
    });
    let mut dump = String::new();
    if fk.is_some() && flag(&REPORT, "MELEE_MEX_REPORT") {
        let pc = m.cpu.pc.wrapping_sub(4);
        let c = &m.cpu;
        dump += &format!("mexguest:   r3..r12 {:08X?} r28..r31 {:08X?} lr {:08X}\n", &c.gpr[3..13], &c.gpr[28..32], c.lr);
        for a in (pc.wrapping_sub(0x30)..=pc).step_by(4) {
            if let Ok(w) = s.space.ld32(a) {
                dump += &format!("mexguest:   {a:08X} {w:08X} {:?}\n", mex_import::ppc::ir::decode(w, a));
            }
        }
    }
    let faulted = fk.is_some();
    if let Some(f) = s.kinds.get_mut(&kind) {
        let stt = f.stats.entry(slot).or_default();
        stt.calls += 1;
        stt.insns += n;
        match fk {
            None => stt.returned += 1,
            Some(k) => {
                if *stt.faults.entry(k.clone()).or_default() == 0 && flag(&REPORT, "MELEE_MEX_REPORT") {
                    eprintln!("mexguest: kind {kind} slot {slot}: {k}");
                    eprint!("{dump}");
                }
                *stt.faults.get_mut(&k).unwrap() += 1;
            }
        }
    }
    (m.cpu, faulted)
}

fn where_(kind: i32, pc: u32) -> String {
    let Some(f) = st().kinds.get(&kind) else { return format!("{pc:08X}") };
    let off = pc.wrapping_sub(f.code_base);
    match f.blob.symbols.iter().rev().find(|s| s.0 <= off) {
        Some(s) if off < f.blob.code.len() as u32 => format!("{}+{:#x}", s.2, off - s.0),
        _ => format!("{pc:08X}"),
    }
}

struct Br;

impl Bridge<GuestSpace> for Br {
    fn in_blob(&self, a: u32) -> bool {
        in_code(a)
    }
    fn call(&mut self, cpu: &mut Cpu, mem: &mut GuestSpace, target: u32) -> Result<(), Fault> {
        call_out(cpu, mem, target)
    }
}

fn call_out(cpu: &mut Cpu, mem: &mut GuestSpace, target: u32) -> Result<(), Fault> {
    if flag(&CALLTRACE, "MELEE_MEX_CALLTRACE") {
        let f = |i: usize| f64::from_bits(cpu.fpr[i]);
        eprintln!(
            "mexcall: {target:08X} {} r3..r6 {:08X?} f1..f3 {:?} {:?} {:?}",
            mex_import::layout::vanilla_name(target).unwrap_or("?"),
            &cpu.gpr[3..7],
            f(1),
            f(2),
            f(3)
        );
    }
    let (d, a, n) = (cpu.gpr[3], cpu.gpr[4], cpu.gpr[5]);
    let ok = match target {
        MEMCPY => {
            if d <= a {
                for k in 0..n {
                    let b = mem.ld8(a.wrapping_add(k))?;
                    mem.st8(d.wrapping_add(k), b)?;
                }
            } else {
                for k in (0..n).rev() {
                    let b = mem.ld8(a.wrapping_add(k))?;
                    mem.st8(d.wrapping_add(k), b)?;
                }
            }
            true
        }
        MEMSET => {
            for k in 0..n {
                mem.st8(d.wrapping_add(k), a as u8)?;
            }
            true
        }
        MEX_INDEX_FIGHTER_ITEM => {
            // (fighter kind, ItemDesc, item id): gw_mex_shim_index_item. Recorded; the native match has no m-ex item
            // table to write it into.
            st().articles.insert((d, n), a);
            cpu.gpr[3] = 0;
            true
        }
        HSD_MEM_ALLOC | MEX_CALLOC => {
            // a block of the game heap (so a rollback restores it) as plain guest bytes at a fresh guest address
            let p = unsafe { melee_runtime::tu::memory::HSD_MemAlloc(d as i32) } as *mut u8;
            if !p.is_null() && target == MEX_CALLOC {
                // gw_mex_shim_calloc: m-ex's calloc(size), zeroed
                unsafe { core::ptr::write_bytes(p, 0, d as usize) };
            }
            if p.is_null() {
                cpu.gpr[3] = 0;
            } else {
                let s = st();
                let base = s.heap_next;
                s.heap_next = (base + d.max(4) + 0x1F) & !0x1F;
                unsafe { mem.add_raw(base, p, d.max(4)) };
                cpu.gpr[3] = base;
            }
            true
        }
        HSD_FREE => {
            if let Some(p) = mem.native(d) {
                unsafe { melee_runtime::tu::memory::HSD_Free(p as *mut c_void) };
                mem.regions.retain(|r| !(r.base == d && matches!(r.backing, mex_import::remap::Backing::Raw { .. })));
            }
            true
        }
        MEX_GET_DATA => {
            // gw_mex_shim_get_data: MxDt.dat tables (the image is mapped as guest bytes at its own address) and the
            // game's costume table (the native rows, by handle)
            let Some((base, size, root)) = crate::mex_host::mexdata_image() else { return Err(Fault::Unresolved(target)) };
            if !mem.regions.iter().any(|r| r.base == base) {
                unsafe { mem.add_raw(base, base as usize as *mut u8, size) };
            }
            let field = |mem: &mut GuestSpace, arch: u32, f: u32| -> u32 {
                let a = mem.ld32(root + arch).unwrap_or(0);
                if a == 0 { 0 } else { mem.ld32(a + f).unwrap_or(0) }
            };
            cpu.gpr[3] = match d {
                8 => {
                    let p = unsafe { core::ptr::addr_of_mut!(melee_fighter::tu::ftdata::CostumeListsForeachCharacter) } as usize;
                    mem.handle(p, Some("UnkCostumeList")).unwrap_or(0)
                }
                10 => field(mem, 0x28, 0x00),
                11 => field(mem, 0x28, 0x10),
                12 => field(mem, 0x08, 0x00),
                13 => field(mem, 0x08, 0x04),
                14 => field(mem, 0x08, 0x08),
                15 => field(mem, 0x08, 0x0C),
                16 => root,
                _ => return Err(Fault::Unresolved(0xD000_0000 | d)),
            };
            true
        }
        MEX_GET_FT_ITEM_ID => {
            // gw_mex_shim_get_ft_item_id: the global item kind of the fighter's article `a` (MxDt.dat item_lookup)
            let kind = mem.native(d).filter(|&p| p != 0).map(|g| unsafe { kind_of(g as G) }).flatten();
            match kind.and_then(|k| crate::mex_host::ft_item_global(k, a)) {
                Some(v) => {
                    cpu.gpr[3] = v;
                    true
                }
                None => false,
            }
        }
        EF_SYNC_SPAWN => {
            // efSync_Spawn(id, gobj, ...): the variadic words as the native call takes them (pointers to objects the
            // guest knows by handle, records it keeps on its stack copied word-swapped in and out, plain values)
            let g = mem.native(a).unwrap_or(0);
            let mut words = [0usize; 6];
            let mut tmp: Vec<(u32, Box<[u32; 16]>)> = Vec::new();
            for (k, w) in words.iter_mut().enumerate() {
                let h = cpu.gpr[5 + k];
                *w = if h == 0 {
                    0
                } else if let Some(p) = mem.interior(h).or_else(|| mem.regions.iter().find(|r| r.base == h && matches!(r.backing, mex_import::remap::Backing::Native { .. })).and_then(|_| mem.native(h))) {
                    p
                } else if mem.native(h).is_some() {
                    let mut b = Box::new([0u32; 16]);
                    for (j, x) in b.iter_mut().enumerate() {
                        *x = mem.ld32(h + 4 * j as u32).unwrap_or(0);
                    }
                    let p = b.as_mut_ptr() as usize;
                    tmp.push((h, b));
                    p
                } else {
                    h as usize
                };
            }
            unsafe { melee_chara::adapt::efSync_Spawn_usize(d as i32, g as G, &words) };
            for (h, b) in tmp {
                for (j, x) in b.iter().enumerate() {
                    let _ = mem.st32(h + 4 * j as u32, *x);
                }
            }
            true
        }
        _ => match mex_import::layout::vanilla_name(target) {
            // an m-ex custom article (global item kind >= CustomItemStart 237, SpawnItem.kind at GC +0x8): the native
            // item system has no m-ex item table (its ItemDesc and itFunction states are the guest's), so the spawn
            // spawns nothing: the call returns NULL (what Item_80268B18 returns when the item cannot be made) and the
            // guest goes on; counted under 0xC0000000 | kind
            Some(n) if matches!(n, "Item_80268B18" | "Item_80268B5C" | "Item_80268B9C" | "Item_8026862C") && mem.ld32(d.wrapping_add(8)).is_ok_and(|k| k >= 237) => {
                *st().unresolved.entry(0xC000_0000 | mem.ld32(d + 8).unwrap_or(0)).or_default() += 1;
                cpu.gpr[3] = 0;
                return Ok(());
            }
            // an item function on the NULL a refused custom spawn returned: nothing to do (the native one would fault)
            Some(n) if d == 0 && (n.starts_with("Item_") || n.starts_with("it_") || n.starts_with("it")) => {
                cpu.gpr[3] = 0;
                return Ok(());
            }
            Some(name) => {
                let mut c = Call { cpu, mem, fix: Vec::new(), bad: false, scratch: SCRATCH_BASE + 0x10000 * st().depth };
                let hit = unsafe { crate::mex_bridge::call(name, &mut c) };
                c.finish();
                if c.bad {
                    let s = st();
                    *s.unresolved.entry(target).or_default() += 1;
                    return Err(Fault::Mem(0));
                }
                hit
            }
            None => false,
        },
    };
    let s = st();
    if ok {
        *s.bound.entry(target).or_default() += 1;
        Ok(())
    } else {
        *s.unresolved.entry(target).or_default() += 1;
        Err(Fault::Unresolved(target))
    }
}

/// One bridged call's arguments and result (mex_bridge's generated arms use it).
pub struct Call<'a> {
    cpu: &'a mut Cpu,
    mem: &'a mut GuestSpace,
    /// Marshalled records: (guest address, native buffer, record).
    fix: Vec<(u32, Box<[u64]>, &'static str, u32)>,
    pub bad: bool,
    scratch: u32,
}

impl Call<'_> {
    pub fn int(&self, i: usize) -> u32 {
        self.cpu.gpr[3 + i]
    }
    pub fn flt(&self, i: usize) -> f64 {
        f64::from_bits(self.cpu.fpr[1 + i])
    }
    /// Pointer argument `i` (an r3.. word) to a `rec` (None: untyped): a registered object's native pointer, a record
    /// the guest keeps in plain memory marshalled into a native copy (written back after the call), or the host byte
    /// of plain memory for an untyped pointer.
    pub fn ptr(&mut self, i: usize, rec: Option<&'static str>) -> *mut c_void {
        let h = self.cpu.gpr[3 + i];
        if h == 0 {
            return core::ptr::null_mut();
        }
        // inside a registered native object: the field / element that starts there (fp->coll_data, &fp->cur_pos)
        if let Some(nat) = self.mem.interior(h) {
            return nat as *mut c_void;
        }
        if let Some(nat) = self.mem.native(h) {
            let registered = self.mem.regions.iter().any(|r| r.base == h && matches!(r.backing, mex_import::remap::Backing::Native { .. }));
            if registered || rec.is_none() {
                return nat as *mut c_void;
            }
        }
        if let Some(rec) = rec {
            if let (Some(ns), Some(gs)) = (self.mem.layouts.native_size(rec), self.mem.layouts.gc_size(rec)) {
                let mut buf = vec![0u64; ns.div_ceil(8)].into_boxed_slice();
                let base = self.scratch;
                self.scratch += (gs + 0x1F) & !0xF;
                let p = buf.as_mut_ptr() as *mut u8;
                unsafe {
                    if self.mem.add_native(Some(base), rec, p).is_ok() {
                        for k in 0..gs {
                            if let Ok(b) = self.mem.ld8(h + k) {
                                let _ = self.mem.st8(base + k, b);
                            }
                        }
                        self.fix.push((h, buf, rec, base));
                        return p as *mut c_void;
                    }
                }
            }
        }
        self.bad = true;
        core::ptr::null_mut()
    }
    /// A gobj callback argument: guest code as its pool trampoline, a vanilla address as the port's function.
    pub fn func(&mut self, i: usize) -> Option<unsafe extern "C" fn(G)> {
        let a = self.cpu.gpr[3 + i];
        if a == 0 {
            return None;
        }
        if let Some(p) = fn_native(a) {
            return Some(unsafe { core::mem::transmute::<usize, unsafe extern "C" fn(G)>(p) });
        }
        let f = vanilla_gobj_fn(a);
        if f.is_none() {
            self.bad = true;
        }
        f
    }
    pub fn ret_int(&mut self, v: u32) {
        self.cpu.gpr[3] = v;
    }
    pub fn ret_flt(&mut self, v: f64) {
        self.cpu.fpr[1] = v.to_bits();
    }
    pub fn ret_ptr(&mut self, p: *mut c_void, rec: Option<&'static str>) {
        match self.mem.handle(p as usize, rec) {
            Ok(h) => self.cpu.gpr[3] = h,
            Err(_) => {
                self.cpu.gpr[3] = 0;
                self.bad = true;
            }
        }
    }
    fn finish(&mut self) {
        for (h, mut buf, rec, base) in std::mem::take(&mut self.fix) {
            let gs = self.mem.layouts.gc_size(rec).unwrap_or(0);
            for k in 0..gs {
                if let Ok(b) = self.mem.ld8(base + k) {
                    let _ = self.mem.st8(h + k, b);
                }
            }
            self.mem.forget_native(buf.as_mut_ptr() as *mut u8);
        }
    }
}

/// Is a guest code callback behind this native function pointer (a pool trampoline)?
pub fn is_guest_fn(p: usize) -> bool {
    fn_guest(p).is_some()
}

/// The per-fighter counters (`MELEE_MEX_REPORT`, the slpcheck summary).
pub fn report() -> String {
    let Some(s) = (unsafe { ST.as_ref() }) else { return String::new() };
    let mut o = String::new();
    for (kind, f) in &s.kinds {
        o += &format!("mexguest: kind {kind} {} ({} bytes)\n", f.file, f.blob.code.len());
        for (slot, st) in &f.stats {
            let name = if *slot >= 2000 {
                "fn-pointer".to_string()
            } else if *slot >= 1000 {
                format!("MoveLogic {}", ["anim", "input", "phys", "coll"][(*slot - 1000) as usize])
            } else {
                SLOT_NAMES.get(*slot as usize).copied().unwrap_or("?").to_string()
            };
            o += &format!("mexguest:   {name}: {} calls, {} returned, {} insns", st.calls, st.returned, st.insns);
            for (k, v) in &st.faults {
                o += &format!("; {k} x{v}");
            }
            o += "\n";
        }
    }
    for (t, n) in &s.unresolved {
        o += &format!("mexguest: unresolved {t:08X} {} x{n}\n", mex_import::layout::vanilla_name(*t).unwrap_or("?"));
    }
    for (t, n) in &s.bound {
        o += &format!("mexguest: bound {t:08X} {} x{n}\n", mex_import::layout::vanilla_name(*t).unwrap_or("?"));
    }
    o
}
