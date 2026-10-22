//! tu::gm_1A3F vs the C (state oracle): gen/turef/tu_gm_1A3F_ref.c is the game's gm_1A3F.c compiled natively
//! (shipping contraction) with every callee stubbed and logged, a scripted frame loop (`gm_801A4D34`) and
//! `gm_801A4510`'s endless loop left through a jump after three mode loads; the expected lines are
//! tests/data/tu_gm_1A3F.txt (regenerate with `python rust/melee_modes/gen/turef/tu_shard537_gm.py tu_gm_1A3F`).
//! Each line is the callee log, the return value, the whole `state_machine`, the preload cache and the
//! gmMainLib flags. The Rust side leaves `gm_801A4510` with a panic caught by the test.
#![allow(static_mut_refs)]

mod tu537;
use core::ffi::c_void;
use melee_modes::tu::gm_1A3F as t;
use melee_types::*;
use tu537::Rng;

const FUNC_NAMES: [&str; 21] = [
    "preloadState",
    "firstState",
    "nextState",
    "findState",
    "findMode",
    "gm_801A4014",
    "EnterData",
    "ExitData",
    "SetGameModeStateId",
    "SetNextGameModeStateId",
    "GetPreviousSceneIndex",
    "GetCurrentSceneIndex",
    "SetNewGameModePending",
    "SetPendingGameMode",
    "ChangeGameModeAfterCurrentScene",
    "GetCurrentGameMode",
    "GetPreviousGameMode",
    "SetGameModeOverride",
    "Is1PMode",
    "runGameMode",
    "gm_801A4510",
];
const NCASES: [usize; 21] = [60, 60, 100, 100, 80, 120, 20, 20, 30, 30, 20, 20, 20, 30, 30, 20, 20, 30, 60, 250, 150];
const NMODES: usize = 7;
const GM_FRONTEND_K: u8 = 46;
const KPOOL: [u8; 6] = [3, 7, 12, 21, 28, 33];
const SKPOOL: [u8; 6] = [GS_STAFFROLL as u8, GS_RESULTS as u8, GS_CSS as u8, 10, 20, 30];

struct Escape;

static mut LOG: String = String::new();
static mut MODES: [GameMode; NMODES + 1] = unsafe { core::mem::zeroed() };
static mut STATES: [[GameModeState; 6]; NMODES] = unsafe { core::mem::zeroed() };
static mut DAT: [i32; 4] = [0, 1, 2, 3];
static mut SCENES: [GameScene; 4] = unsafe { core::mem::zeroed() };
static mut STANDIN: GameScene = unsafe { core::mem::zeroed() };
static mut PRELOAD: PreloadedGameModeState = unsafe { core::mem::zeroed() };
static mut SCRIPT: [i32; 16] = [0; 16];
static mut SI: i32 = 0;
static mut OVRV: [i32; 4] = [0; 4];
static mut OI: usize = 0;
static mut HEAP2: i32 = 0;
static mut HEAP3: i32 = 0;
static mut POL1: i32 = 0;
static mut POL2: i32 = 0;
static mut STAND: bool = false;
static mut TOYKNOWN: i32 = 0;
static mut PROBE: bool = false;
static mut BARRIER: i32 = 0;
static mut DVD: i32 = 0;
static mut CNT11: i32 = 0;
static mut BUSY: i32 = 0;
static mut DTV: u32 = 0;
static mut PROG: u32 = 0;
static mut ROUTE: bool = false;
static mut LOADS: i32 = 0;
static mut MAXLOADS: i32 = 0;
static mut ESCAPE: bool = false;
static mut RESETTING: bool = false;
static mut SKIP_INTRO: bool = false;
static mut LAUNCH_BTN: u16 = 0;

fn log(s: &str) {
    unsafe { LOG.push_str(s) }
}

unsafe extern "C" fn se_a(s: *mut GameModeState) {
    log(&format!("seA({});", (*s).id));
}
unsafe extern "C" fn se_b(s: *mut GameModeState) {
    log(&format!("seB({});", (*s).id));
}
unsafe extern "C" fn sx_a(s: *mut GameModeState) {
    log(&format!("sxA({});", (*s).id));
}
unsafe extern "C" fn sx_b(s: *mut GameModeState) {
    log(&format!("sxB({});", (*s).id));
}
unsafe extern "C" fn ld_a() {
    log("ldA;");
}
unsafe extern "C" fn ld_b() {
    log("ldB;");
}
unsafe extern "C" fn ud_a() {
    log("udA;");
}
unsafe extern "C" fn ud_b() {
    log("udB;");
}
unsafe extern "C" fn in_a() {
    log("inA;");
}
unsafe extern "C" fn in_b() {
    log("inB;");
}
unsafe fn dname(d: *mut c_void) -> &'static str {
    if d.is_null() {
        "null"
    } else if d == &raw mut DAT[0] as *mut c_void {
        "d0"
    } else if d == &raw mut DAT[1] as *mut c_void {
        "d1"
    } else if d == &raw mut DAT[2] as *mut c_void {
        "d2"
    } else {
        "d3"
    }
}
unsafe extern "C" fn scn_enter(d: *mut c_void) {
    log(&format!("scE({});", dname(d)));
}
unsafe extern "C" fn scn_exit(d: *mut c_void) {
    log(&format!("scX({});", dname(d)));
}
unsafe extern "C" fn stand_enter(d: *mut c_void) {
    log(&format!("stE({});", dname(d)));
}
unsafe extern "C" fn fr0() {}
unsafe extern "C" fn fr1() {}
unsafe extern "C" fn fr2() {}
unsafe extern "C" fn fr3() {}
unsafe extern "C" fn frs() {}
unsafe extern "C" fn wrapped() {}
unsafe extern "C" fn ovr() -> u8 {
    let v = OVRV[OI % 4];
    OI += 1;
    log(&format!("ovr={};", v));
    v as u8
}
fn fname(f: Option<unsafe extern "C" fn()>) -> &'static str {
    let a = match f {
        Some(f) => f as usize,
        None => return "?",
    };
    if a == fr0 as usize {
        "f0"
    } else if a == fr1 as usize {
        "f1"
    } else if a == fr2 as usize {
        "f2"
    } else if a == fr3 as usize {
        "f3"
    } else if a == frs as usize {
        "fS"
    } else if a == wrapped as usize {
        "wrap"
    } else {
        "?"
    }
}

unsafe fn script_next() -> i32 {
    if SI >= 16 {
        SI += 1;
        return if SI % 2 == 1 { 1 } else { 7 };
    }
    let v = SCRIPT[SI as usize];
    SI += 1;
    v
}

fn install_hooks() {
    use t::ext::*;
    unsafe {
        LBDVD_80018CF4 = Some(|v| log(&format!("8CF4({});", v)));
        LBDVD_GETPRELOADCACHESCENE = Some(|| unsafe {
            log("getcache;");
            &raw mut PRELOAD
        });
        LBDVD_80018254 = Some(|| log("8254;"));
        LBDVD_80018F58 = Some(|v| unsafe {
            log(&format!("8F58({});", v as i32));
            if ESCAPE {
                let l = LOADS;
                LOADS += 1;
                if l >= MAXLOADS {
                    std::panic::panic_any(Escape);
                }
            }
        });
        LBHEAP_80015BB8 = Some(|i| unsafe {
            log(&format!("heap({});", i));
            if i == 2 {
                HEAP2
            } else {
                HEAP3
            }
        });
        LBCARDNEW_FORGETMEMORY = Some(|| log("forget;"));
        LBCARDGAME_RESET = Some(|| log("cardreset;"));
        LBSNAP_8001E27C = Some(|| log("snapE27C;"));
        TOY_803127D4 = Some(|| log("toy27D4;"));
        TYDISPLAY_8031C8B8 = Some(|| log("tyC8B8;"));
        HSD_SISLIB_803A6048 = Some(|n| log(&format!("6048({:x});", n)));
        LB_8001CDB4 = Some(|| log("CDB4;"));
        LBCARDNEW_COMPLETEALLTASKS = Some(|r| {
            log(&format!("all({});", r));
            0
        });
        LBCARDNEW_COMPLETENEXTTASK = Some(|| unsafe {
            let r = if CNT11 > 0 { 11 } else { 0 };
            CNT11 -= 1;
            log(&format!("next={};", r));
            r
        });
        LBMTHP_8001F800 = Some(|| log("mthp;"));
        LBAUDIOAX_80027DBC = Some(|| log("audio;"));
        HSD_PADRESET = Some(|| log("padreset;"));
        HSD_DEVCOMISBUSY = Some(|i| unsafe {
            let r = BUSY > 0;
            BUSY -= 1;
            log(&format!("busy({})={};", i, r as i32));
            r
        });
        HSD_VISETBLACK = Some(|b| log(&format!("black({});", b as i32)));
        DVDCHECKDISK = Some(|| {
            log("dvd;");
            unsafe { DVD }
        });
        OSRESETSYSTEM = Some(|a, b, c| log(&format!("osreset({},{},{});", a, b, c)));
        VIGETDTVSTATUS = Some(|| {
            log("dtv;");
            unsafe { DTV }
        });
        OSGETPROGRESSIVEMODE = Some(|| {
            log("prog;");
            unsafe { PROG }
        });
        GM_FINDGAMESCENEHANDLER = Some(|k| unsafe {
            log(&format!("find({});", k));
            &raw mut SCENES[(k % 4) as usize]
        });
        GM_801A4BD4 = Some(|| log("4BD4;"));
        GM_801A4B88 = Some(|i| unsafe { log(&format!("4B88({});", (*i).scene_kind)) });
        GMMAINLIB_8015FBA4 = Some(|| log("FBA4;"));
        GM_GETALLGAMEMODES = Some(|| unsafe { MODES.as_mut_ptr() });
        GM_801A3EF4 = Some(|| log("3EF4;"));
        SCENEREPORT_STATE = Some(|p, m, id, k| log(&format!("rep({},{},{},{});", p, m, id, k)));
        UI_SCENEPOLICY = Some(|k| unsafe {
            let r = if k % 2 != 0 { POL1 } else { POL2 };
            log(&format!("policy({})={};", k, r));
            r
        });
        GMFRONTEND_ATLASSTANDIN = Some(|k| unsafe {
            log(&format!("standin({});", k));
            if STAND {
                &raw mut STANDIN
            } else {
                core::ptr::null_mut()
            }
        });
        UI_SCENEPOLICYMASK = Some(|k, known| log(&format!("mask({},{});", k, known)));
        FAD_TOYWINDOWKNOWN = Some(|k| {
            log(&format!("known({});", k));
            unsafe { TOYKNOWN }
        });
        FIGHTERCONTROLS_MATCHCLEAR = Some(|| log("fcclear;"));
        UI_TOYPROBE = Some(|| {
            log("probe;");
            unsafe { PROBE as i32 }
        });
        FAD_OVERLAYWRAP = Some(|kind, inner| {
            log(&format!("wrap({},{});", kind, fname(inner)));
            Some(wrapped)
        });
        SCRIPT_ONEPBARRIERPENDING = Some(|| {
            log("barrier;");
            unsafe { BARRIER }
        });
        GM_ONEPINTERSTAGE = Some(|| log("interstage;"));
        GMFRONTEND_ROUTE = Some(|from, to| unsafe {
            let r = if ROUTE { GM_FRONTEND_K } else { to };
            log(&format!("route({},{})={};", from, to, r));
            r
        });
        GMFRONTEND_REPORTEDMODE = Some(|| {
            log("reported;");
            17
        });
        GM_801A4D34 = Some(|cb, info| unsafe {
            log(&format!("4D34({},{});", fname(cb), (*info).scene_kind));
            let op = script_next();
            let arg = script_next();
            match op {
                1 => t::gm_ChangeGameModeAfterCurrentScene(MODES[arg as usize % NMODES].kind as i32),
                2 => {
                    t::gm_SetPendingGameMode(MODES[arg as usize % NMODES].kind);
                    t::gm_SetNewGameModePending();
                }
                3 => t::gm_SetPendingGameMode(MODES[arg as usize % NMODES].kind),
                4 => t::gm_SetNextGameModeStateId((arg % 3) as u8),
                5 => t::gm_SetGameModeStateId((arg % 4) as u8),
                6 => RESETTING = true,
                7 => t::gm_SetGameModeOverride(Some(ovr)),
                8 => t::gm_SetGameModeOverride(None),
                _ => {}
            }
        });
        GMMAINLIB_8046B0F0_RESETTING = Some(|| unsafe { RESETTING });
        SET_GMMAINLIB_8046B0F0_RESETTING = Some(|v| unsafe { RESETTING = v });
        SET_GMMAINLIB_8046B0F0_SKIP_INTRO = Some(|v| unsafe { SKIP_INTRO = v });
        DB_GAMELAUNCHBUTTONSTATE = Some(|| unsafe { LAUNCH_BTN });
    }
}

fn pick<T: Copy>(r: &mut Rng, a: T, b: T, c: T) -> T {
    let x = r.rnd();
    if x % 3 == 0 {
        a
    } else if x % 3 == 1 {
        b
    } else {
        c
    }
}

unsafe fn build_world(r: &mut Rng) {
    MODES = core::mem::zeroed();
    STATES = core::mem::zeroed();
    DAT = [0, 1, 2, 3];
    for k in 0..4usize {
        SCENES[k] = core::mem::zeroed();
        SCENES[k].kind = k as u8;
        SCENES[k].on_frame = Some(match k {
            0 => fr0 as unsafe extern "C" fn(),
            1 => fr1,
            2 => fr2,
            _ => fr3,
        });
        SCENES[k].on_enter = if k == 3 { None } else { Some(scn_enter) };
        SCENES[k].on_exit = if k == 2 { None } else { Some(scn_exit) };
    }
    STANDIN = core::mem::zeroed();
    STANDIN.kind = 99;
    STANDIN.on_frame = Some(frs);
    STANDIN.on_enter = Some(stand_enter);
    STANDIN.on_exit = None;
    let a = (r.rnd() % 6) as usize;
    let b = (a + 1 + (r.rnd() % 5) as usize) % 6;
    let mut c = b;
    while c == a || c == b {
        c = (r.rnd() % 6) as usize;
    }
    MODES[0].kind = GM_BOOT as u8;
    MODES[1].kind = GM_PROGRESSIVE_SCAN as u8;
    MODES[2].kind = KPOOL[a];
    MODES[3].kind = KPOOL[b];
    MODES[4].kind = KPOOL[c];
    MODES[5].kind = GM_FRONTEND_K;
    MODES[6].kind = 0;
    MODES[NMODES].kind = GM_COUNT as u8;
    for i in 0..NMODES {
        MODES[i].preloaded = (r.rnd() % 2) as u8;
        MODES[i].on_load = pick(r, None, Some(ld_a as unsafe extern "C" fn()), Some(ld_b));
        MODES[i].on_unload = pick(r, None, Some(ud_a as unsafe extern "C" fn()), Some(ud_b));
        MODES[i].on_init = pick(r, None, Some(in_a as unsafe extern "C" fn()), Some(in_b));
        MODES[i].states = STATES[i].as_mut_ptr();
        let n = 1 + (r.rnd() % 4) as usize;
        let mut id: i32 = 0;
        for k in 0..n {
            let s = &mut STATES[i][k];
            let x = r.rnd();
            id = if k == 0 { (x % 2) as i32 } else { id + 1 + (x % 2) as i32 };
            if k == n - 1 && id < 4 {
                id = 4;
            }
            s.id = id as u8;
            s.preload = (r.rnd() % 5) as u8;
            s.flags = r.rnd() as u16;
            s.on_enter = pick(r, None, Some(se_a as unsafe extern "C" fn(*mut GameModeState)), Some(se_b));
            s.on_exit = pick(r, None, Some(sx_a as unsafe extern "C" fn(*mut GameModeState)), Some(sx_b));
            s.info.scene_kind = SKPOOL[(r.rnd() % 6) as usize];
            let x = r.rnd();
            s.info.enter_data = if x % 2 != 0 { &raw mut DAT[((x >> 4) % 4) as usize] as *mut c_void } else { core::ptr::null_mut() };
            let x = r.rnd();
            s.info.exit_data = if x % 2 != 0 { &raw mut DAT[((x >> 4) % 4) as usize] as *mut c_void } else { core::ptr::null_mut() };
        }
        STATES[i][n].id = 0xFF;
    }
}

unsafe fn mode_index(m: *mut GameMode) -> i32 {
    if m.is_null() {
        -1
    } else {
        m.offset_from(MODES.as_mut_ptr()) as i32
    }
}
unsafe fn state_index(s: *mut GameModeState) -> i32 {
    if s.is_null() {
        return -1;
    }
    for i in 0..NMODES {
        for k in 0..6 {
            if s == &raw mut STATES[i][k] {
                return (i * 10 + k) as i32;
            }
        }
    }
    -2
}

#[test]
fn matches_c() {
    let want = tu537::data("tu_gm_1A3F");
    install_hooks();
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        if info.payload().downcast_ref::<Escape>().is_none() {
            default_hook(info)
        }
    }));
    let mut r = Rng::new();
    let mut lines = want.lines();
    for f in 0..FUNC_NAMES.len() {
        for i in 0..NCASES[f] {
            unsafe {
                build_world(&mut r);
                RESETTING = false;
                SKIP_INTRO = false;
                SI = 0;
                OI = 0;
                let mut k = 0;
                while k < 16 {
                    let x = r.rnd();
                    SCRIPT[k] = match x % 10 {
                        0 | 1 | 2 => 0,
                        3 => 1,
                        4 => 2,
                        5 => 3,
                        6 => 4,
                        7 => 5,
                        8 => 6,
                        _ => {
                            if (x >> 8) % 2 != 0 {
                                7
                            } else {
                                8
                            }
                        }
                    };
                    SCRIPT[k + 1] = (r.rnd() % 64) as i32;
                    k += 2;
                }
                for k in 0..4 {
                    let x = r.rnd();
                    OVRV[k] = if x % 3 == 0 { MODES[2 + ((x >> 4) % 3) as usize].kind as i32 } else { GM_COUNT as i32 };
                }
                HEAP2 = (r.rnd() % 2) as i32;
                HEAP3 = (r.rnd() % 2) as i32;
                POL1 = (r.rnd() % 3) as i32;
                POL2 = (r.rnd() % 3) as i32;
                STAND = r.rnd() % 2 != 0;
                TOYKNOWN = (r.rnd() % 2) as i32;
                PROBE = r.rnd() % 3 == 0;
                BARRIER = (r.rnd() % 2) as i32;
                DVD = (r.rnd() % 2) as i32;
                CNT11 = (r.rnd() % 3) as i32;
                BUSY = (r.rnd() % 3) as i32;
                DTV = r.rnd() % 2;
                PROG = r.rnd() % 2;
                ROUTE = r.rnd() % 3 == 0;
                LAUNCH_BTN = if r.rnd() % 2 != 0 { t::HSD_PAD_B as u16 } else { 0 };
                PRELOAD.is_heap_persistent[0] = (r.rnd() % 2) as i32;
                PRELOAD.is_heap_persistent[1] = (r.rnd() % 2) as i32;
                t::state_machine = core::mem::zeroed();
                t::state_machine.routing.curr_mode = (r.rnd() % 47) as u8;
                t::state_machine.routing.pending_mode = (r.rnd() % 47) as u8;
                t::state_machine.routing.prev_mode = (r.rnd() % 47) as u8;
                t::state_machine.routing.curr_state_id = (r.rnd() % 5) as u8;
                t::state_machine.routing.prev_state_id = (r.rnd() % 5) as u8;
                let x = r.rnd();
                t::state_machine.routing.next_state_id = if x % 4 == 0 { ((x >> 4) % 3) as u8 } else { 0 };
                t::state_machine.get_override = if r.rnd() % 3 == 0 { Some(ovr) } else { None };
                LOG.clear();
                ESCAPE = false;
                LOADS = 0;
                MAXLOADS = 3;
                let mi = (r.rnd() % NMODES as u32) as usize;
                let mut si = (r.rnd() % 6) as usize;
                let a = r.rnd();
                let mut nst = 0usize;
                while STATES[mi][nst].id != 0xFF {
                    nst += 1;
                }
                si %= nst;
                let m: *mut GameMode = &raw mut MODES[mi];
                let mut ret: i32 = 0;
                match f {
                    0 => t::preloadState(&raw mut STATES[mi][si]),
                    1 => ret = t::firstState(STATES[mi].as_mut_ptr(), a as u8) as i32,
                    2 => ret = t::nextState(STATES[mi].as_mut_ptr()) as i32,
                    3 => ret = state_index(t::findState(STATES[mi].as_mut_ptr())),
                    4 => ret = mode_index(t::findMode((a % 48) as u8)),
                    5 => t::gm_801A4014(m),
                    6 => {
                        let d = t::gm_GetGameModeStateEnterData(&raw mut STATES[mi][si]) as *mut i32;
                        ret = if d.is_null() { -1 } else { *d };
                    }
                    7 => {
                        let d = t::gm_GetGameModeStateExitData(&raw mut STATES[mi][si]) as *mut i32;
                        ret = if d.is_null() { -1 } else { *d };
                    }
                    8 => t::gm_SetGameModeStateId(a as u8),
                    9 => t::gm_SetNextGameModeStateId(a as u8),
                    10 => ret = t::gm_GetPreviousSceneIndex() as i32,
                    11 => ret = t::gm_GetCurrentSceneIndex() as i32,
                    12 => t::gm_SetNewGameModePending(),
                    13 => t::gm_SetPendingGameMode(a as u8),
                    14 => t::gm_ChangeGameModeAfterCurrentScene((a % 64) as i32),
                    15 => ret = t::gm_GetCurrentGameMode() as i32,
                    16 => ret = t::gm_GetPreviousGameMode() as i32,
                    17 => t::gm_SetGameModeOverride(if a % 2 != 0 { Some(ovr) } else { None }),
                    18 => ret = t::gm_Is1PMode((a % 50) as u8) as i32,
                    19 => ret = t::runGameMode((*m).kind) as i32,
                    _ => {
                        ESCAPE = true;
                        let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe { t::gm_801A4510() }));
                        assert!(res.is_err() && res.unwrap_err().downcast_ref::<Escape>().is_some(), "gm_801A4510 returned or failed");
                        ESCAPE = false;
                    }
                }
                let sm = &t::state_machine;
                let ov = match sm.get_override {
                    None => 0,
                    Some(p) if p as usize == ovr as usize => 1,
                    Some(_) => 2,
                };
                let got = format!(
                    "{}.{} {}|log={}|ret={}|sm={},{},{},{},{},{},{},{},{},{},{},{},{},{}|pre={},{}|si={},{}|rs={},{},{},{}",
                    f,
                    i,
                    FUNC_NAMES[f],
                    LOG,
                    ret,
                    sm.routing.curr_mode,
                    sm.routing.pending_mode,
                    sm.routing.prev_mode,
                    sm.routing.curr_state_id,
                    sm.routing.prev_state_id,
                    sm.routing.next_state_id,
                    sm.backup_routing.curr_mode,
                    sm.backup_routing.pending_mode,
                    sm.backup_routing.prev_mode,
                    sm.backup_routing.curr_state_id,
                    sm.backup_routing.prev_state_id,
                    sm.backup_routing.next_state_id,
                    sm.pending_mode_change,
                    ov,
                    PRELOAD.is_heap_persistent[0],
                    PRELOAD.is_heap_persistent[1],
                    SI,
                    OI,
                    RESETTING as i32,
                    SKIP_INTRO as i32,
                    LOADS,
                    CNT11
                );
                let line = lines.next().expect("fewer lines than cases");
                assert_eq!(got, line, "function {} case {}", FUNC_NAMES[f], i);
            }
        }
    }
    assert!(lines.next().is_none(), "more lines than cases");
}
