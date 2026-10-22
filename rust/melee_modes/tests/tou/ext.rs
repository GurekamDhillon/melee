//! The callee side of the oracle. Every stubbed callee (the Rust port's `mod ext` through
//! `hook::HOOK`, the C stubs through `orc_cb`) lands in `ext_call`, which logs the call
//! (name, fixed arguments with pointers normalised) and produces the return value / out-effects
//! from a per-run seeded generator. Both implementations therefore see exactly the same callee.
use std::cell::Cell;
use std::collections::HashMap;
use std::ffi::{c_char, CStr};
use std::sync::OnceLock;

use super::gen_tables::CALLEES;
use super::space::{name_hash, type_index_opt, Fnv, Space, RAW};

pub struct Rng(pub u64);
impl Rng {
    pub fn new(seed: u64) -> Self {
        Rng(seed ^ 0x9E3779B97F4A7C15)
    }
    pub fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E3779B97F4A7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
        z ^ (z >> 31)
    }
    /// uniform in [0, n)
    pub fn below(&mut self, n: u64) -> u64 {
        if n == 0 { 0 } else { self.next() % n }
    }
    /// inclusive range
    pub fn range(&mut self, lo: i64, hi: i64) -> i64 {
        lo + self.below((hi - lo + 1) as u64) as i64
    }
    pub fn chance(&mut self, num: u64, den: u64) -> bool {
        self.below(den) < num
    }
    pub fn f32r(&mut self, lo: f32, hi: f32) -> f32 {
        let u = (self.next() >> 40) as f32 / (1u64 << 24) as f32;
        lo + (hi - lo) * u
    }
}

/// Persistent world objects the stubs hand out (set by the world builder, per space).
#[derive(Default, Clone, Copy)]
pub struct Persist {
    pub rules: usize,
    pub prefs: usize,
    pub preload: usize,
    pub exit_buf: usize,
    pub names: [usize; 8],
    pub plink: usize,
    /// GetNameText may return NULL (only the callers that test for it are run that way)
    pub names_null: bool,
}

pub struct Run {
    pub sp: *mut Space,
    pub persist: Persist,
    pub rng: Rng,
    pub log: Fnv,
    pub trace: Option<Vec<String>>,
    pub ncalls: u32,
}

thread_local! {
    static CUR: Cell<*mut Run> = const { Cell::new(std::ptr::null_mut()) };
}

pub fn with_run<R>(run: &mut Run, f: impl FnOnce() -> R) -> R {
    CUR.with(|c| c.set(run as *mut Run));
    let r = f();
    CUR.with(|c| c.set(std::ptr::null_mut()));
    r
}

pub struct CalleeInfo {
    pub arity: usize,
    pub ret: &'static str,
    pub pointee: &'static str,
    pub args: &'static [&'static str],
}

pub fn callee_info(name: &str) -> Option<&'static CalleeInfo> {
    static MAP: OnceLock<HashMap<&'static str, CalleeInfo>> = OnceLock::new();
    let m = MAP.get_or_init(|| {
        CALLEES
            .iter()
            .map(|c| (c.0, CalleeInfo { arity: c.1, ret: c.2, pointee: c.3, args: c.4 }))
            .collect()
    });
    m.get(name)
}

unsafe fn cstr(p: u64) -> Option<String> {
    if p == 0 {
        return None;
    }
    Some(CStr::from_ptr(p as usize as *const c_char).to_string_lossy().into_owned())
}

/// Pad-button words the title/tournament scenes test (dolphin/pad.h)
const PAD_BITS: [u32; 17] = [
    1, 2, 4, 8, 0x10, 0x20, 0x40, 0x100, 0x200, 0x400, 0x800, 0x1000, 0x10000, 0x20000, 0x40000, 0x80000, 0x600,
];

fn pad_word(rng: &mut Rng) -> u64 {
    let mut w = 0u32;
    if rng.chance(1, 3) {
        return 0;
    }
    for _ in 0..rng.range(1, 3) {
        w |= PAD_BITS[rng.below(PAD_BITS.len() as u64) as usize];
    }
    w as u64
}

/// Entry point for every callee call.
pub fn ext_call(name: &str, args_in: &[u64]) -> u64 {
    let run: &mut Run = CUR.with(|c| unsafe { c.get().as_mut() }).expect("ext call outside a run");
    let sp: &mut Space = unsafe { &mut *run.sp };
    let info = callee_info(name).unwrap_or_else(|| panic!("callee {name} has no C prototype"));
    let args: Vec<u64> = (0..info.arity).map(|i| args_in.get(i).copied().unwrap_or(0)).collect();
    if name == "__assert" {
        let s = |i: usize| unsafe { cstr(args_in.get(i).copied().unwrap_or(0)) }.unwrap_or_default();
        eprintln!("ASSERT {} line {} : {}", s(0), args_in.get(1).copied().unwrap_or(0), s(2));
        if let Some(tr) = run.trace.as_ref() {
            for l in tr.iter().rev().take(12).rev() {
                eprintln!("  call {l}");
            }
        }
    }
    // pure accessor the port reads inline (gm_1A3F.c: `return state->info.exit_data`): not a logged callee
    if name == "gm_GetGameModeStateExitData" {
        let off = std::mem::offset_of!(melee_types::GameModeState, info) + std::mem::offset_of!(melee_types::GameSceneInfo, exit_data);
        return unsafe { *((args[0] as usize + off) as *const u64) };
    }
    if name == "gm_GetGameModeStateEnterData" {
        let off = std::mem::offset_of!(melee_types::GameModeState, info) + std::mem::offset_of!(melee_types::GameSceneInfo, enter_data);
        return unsafe { *((args[0] as usize + off) as *const u64) };
    }
    run.ncalls += 1;
    // ---- log
    run.log.str(name);
    let mut line = String::new();
    for (i, &a) in args.iter().enumerate() {
        let cl = info.args.get(i).copied().unwrap_or("u64");
        let v = if cl == "p" { arg_ptr_log(sp, name, i, a, run.trace.is_some(), &mut line) } else { a };
        run.log.u64(v);
        if run.trace.is_some() {
            line += &format!(" {v:#x}");
        }
    }
    if let Some(t) = run.trace.as_mut() {
        t.push(format!("{name}({})", line.trim()));
        if std::env::var("TOU_LIVE").is_ok() {
            eprintln!("LIVE {}", t.last().unwrap());
        }
    }
    let rng = &mut run.rng;
    let p = run.persist;
    let r: u64 = unsafe { behave(sp, rng, &p, name, info, &args) };
    run.log.u64(if info.ret == "p" { sp.norm(r) } else { r });
    r
}

/// Pointer argument for the log: arena / global / function pointers are tagged by `norm`; the
/// few callees that take a pointer to a value (colour, string) log the pointee instead; any
/// other foreign pointer (stack, rodata) is logged as a constant.
fn arg_ptr_log(sp: &Space, name: &str, i: usize, a: u64, _trace: bool, line: &mut String) -> u64 {
    let n = sp.norm(a);
    if n != a || a == 0 {
        return n;
    }
    // a foreign pointer: log the pointee for the callees where it carries data
    let data: Option<Vec<u8>> = unsafe {
        match (name, i) {
            ("DrawRectangle", 4) => Some(std::slice::from_raw_parts(a as usize as *const u8, 4).to_vec()),
            ("HSD_SisLib_803A6B98", 3) | ("HSD_SisLib_803A70A0", 2) | ("printf", 0) | ("OSReport", 0) | ("HSD_SisLib_803A62A0", 1)
            | ("HSD_SisLib_803A62A0", 2) | ("lbArchive_80016DBC", 0) | ("lbArchive_LoadSymbols", 0) | ("lbArchive_LoadArchive", 0)
            | ("__assert", 0) | ("__assert", 2) => cstr(a).map(|s| s.into_bytes()),
            ("HSD_CObjSetInterest", 1) | ("HSD_CObjSetEyePosition", 1) => Some(std::slice::from_raw_parts(a as usize as *const u8, 12).to_vec()),
            _ => None,
        }
    };
    match data {
        Some(d) => {
            let mut f = Fnv::new();
            f.bytes(&d);
            let _ = line;
            0xD000_0000_0000_0000 | (f.0 & 0x0FFF_FFFF_FFFF_FFFF)
        }
        None => 0xEEEE_EEEE_EEEE_EEEE,
    }
}

unsafe fn alloc_ret(sp: &mut Space, info: &CalleeInfo) -> u64 {
    if info.pointee == "HSD_JObj" {
        // a sibling chain: the callers walk `->next` up to 13 times (e.g. HSD_JObjGetChild of a model)
        let root = sp.alloc_ty("HSD_JObj", 1) as *mut melee_types::HSD_JObj;
        let mut prev: *mut melee_types::HSD_JObj = std::ptr::null_mut();
        for k in 0..14 {
            let n = sp.alloc_ty("HSD_JObj", 1) as *mut melee_types::HSD_JObj;
            let d = sp.alloc_ty("HSD_DObj", 1) as *mut melee_types::HSD_DObj;
            (*d).mobj = sp.alloc_ty("HSD_MObj", 1) as *mut melee_types::HSD_MObj;
            (*n).u.dobj = d;
            if k == 0 {
                (*root).child = n;
            } else {
                (*prev).next = n;
            }
            prev = n;
        }
        return root as u64;
    }
    let t = type_index_opt(info.pointee);
    match t {
        Some(t) => sp.alloc(super::gen_tables::TYPES[t].1, t, 1) as u64,
        None => sp.alloc(0x400, RAW, 1) as u64,
    }
}

unsafe fn behave(sp: &mut Space, rng: &mut Rng, p: &Persist, name: &str, info: &CalleeInfo, a: &[u64]) -> u64 {
    let base = sp.base as usize;
    let ptr = |off: usize| (base + off) as u64;
    match name {
        // ---- persistent objects
        "gmMainLib_GetGameRules" => return ptr(p.rules),
        "gmMainLib_GetGamePrefs" => return ptr(p.prefs),
        "lbDvd_GetPreloadCacheScene" => return ptr(p.preload),
        "gm_GetCurrentSceneExitData" => return ptr(p.exit_buf),
        "GetNameText" => {
            let id = a[0] as i32 as i64;
            if p.names_null && id.rem_euclid(7) == 3 {
                return 0;
            }
            return ptr(p.names[id.rem_euclid(8) as usize]);
        }
        // ---- out effects
        "lb_80011E24" => {
            // (jobj, HSD_JObj** result, int, int): *result = a fresh JObj
            let j = sp.alloc_ty("HSD_JObj", 1);
            if a[1] != 0 {
                *(a[1] as usize as *mut u64) = j as u64;
            }
            return 0;
        }
        "gm_801692E8" => {
            // (secs, datetime* out)
            let d = a[1] as usize as *mut u8;
            let v: [u8; 8] = [0x07, 0xE8, rng.below(12) as u8 + 1, rng.below(28) as u8 + 1, rng.below(24) as u8, rng.below(60) as u8, rng.below(6) as u8, 0];
            std::ptr::copy_nonoverlapping(v.as_ptr(), d, 7);
            return 0;
        }
        "fn_80169000" => {
            let h = a[1] as usize as *mut u8;
            for i in 0..4 {
                *h.add(i) = rng.below(10) as u8;
            }
            return 0;
        }
        "HSD_GObjFree" => {
            // unlink the head of its p_link list
            let g = a[0] as usize as *mut u8;
            // HSD_GObj: classifier u16 @0, p_link u8 @2, next @8
            let plink = *g.add(2) as usize;
            let next = *(g.add(8) as *const u64);
            let head = p.plink as *mut u64;
            *head.add(plink) = next;
            return 0;
        }
        "HSD_GObjObject_80390A70" => {
            // (gobj, kind, obj): attach the HSD object
            let g = a[0] as usize as *mut melee_types::HSD_GObj;
            (*g).obj_kind = a[1] as u8;
            (*g).hsd_obj = a[2] as usize as *mut core::ffi::c_void;
            return 0;
        }
        "GObj_Create" => {
            // (classifier, p_link, priority) -> a fresh gobj with those fields set
            let g = sp.alloc_ty("HSD_GObj", 1) as *mut melee_types::HSD_GObj;
            (*g).classifier = a[0] as u16;
            (*g).p_link = a[1] as u8;
            (*g).p_priority = a[2] as u8;
            return g as u64;
        }
        "HSD_Randi" => {
            let m = a[0] as i32;
            return if m <= 0 { 0 } else { rng.below(m as u64) };
        }
        "HSD_Rand" => return rng.next() & 0xFFFF_FFFF,
        "gm_80164430" => return 1,
        "mnSelStageRandom" => return rng.below(30),
        "gm_GetNumCostumesForCKind" => return 1 + rng.below(6),
        "gmMainLib_8015ED30" => return rng.below(3),
        "gmMainLib_8015ECB0" => return rng.below(100),
        "gm_GetCurrentGameMode" => return [0u64, 24, 6, 1][rng.below(4) as usize],
        "gm_GetCurrentSceneIndex" => return [2u64, 0, 9, 36][rng.below(4) as usize],
        "lbTime_GetTimeInSeconds" => return rng.below(100000),
        "gm_GetButtonsTriggered" | "gm_GetButtonsPressed" | "gm_801A36C0" => return pad_word(rng),
        "lbAudioAx_80026E84" | "lbAudioAx_80026EBC" => return rng.next(),
        "lbAudioAx_8002305C" => return rng.below(64),
        "fn_80160400" => return rng.below(100),
        "gm_80162800" => return rng.below(100),
        "mn_8022F298" => return (rng.f32r(0.0, 600.0)).to_bits() as u64,
        _ => {}
    }
    match info.ret {
        "v" => 0,
        "p" => alloc_ret(sp, info),
        "f" => (rng.f32r(-10.0, 10.0)).to_bits() as u64,
        "d" => (rng.f32r(-10.0, 10.0) as f64).to_bits(),
        k => {
            let v = rng.next() & 3;
            if k.starts_with('i') {
                v as i64 as u64
            } else {
                v
            }
        }
    }
}

/// The Rust port's hook (`mod ext` stubs).
pub unsafe fn rust_hook(name: &'static str, args: &[u64]) -> u64 {
    ext_call(name, args)
}

#[cfg(feature = "c_oracle")]
#[no_mangle]
pub extern "C" fn orc_cb(name: *const c_char, args: *const u64, n: i32) -> u64 {
    let nm = unsafe { CStr::from_ptr(name) }.to_str().unwrap();
    let a = unsafe { std::slice::from_raw_parts(args, n.max(0) as usize) };
    ext_call(nm, a)
}

pub fn _unused(_: u64) -> u64 {
    name_hash("")
}

/// Callback argument: the recipe passes an index into `CB_NAMES`; the Rust side maps it to the port's
/// function (the C side gets the address of the same-named C function, see run.rs).
pub fn cb_event(k: u64) -> melee_types::HSD_GObjEvent {
    use melee_modes::tu::{gmtou_0, gmtou_1};
    match k {
        1 => Some(gmtou_0::fn_801910E0),
        2 => Some(gmtou_1::fn_80196DBC),
        _ => None,
    }
}
