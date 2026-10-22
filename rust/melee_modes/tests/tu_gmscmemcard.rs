//! tu::gmscmemcard vs the C (state oracle): gen/turef/tu_gmscmemcard_ref.c is the game's gmscmemcard.c compiled
//! natively (shipping contraction) with the notice window, the card library, the pad and the TARGET_PC port
//! hooks stubbed and logged; the expected lines are tests/data/tu_gmscmemcard.txt (regenerate with
//! `python rust/melee_modes/gen/turef/tu_shard537_gm.py tu_gmscmemcard`). Each line is the callee log, the return
//! value, `enter_data`, the two debug statics and the exit data written by OnExit.
#![allow(static_mut_refs)]

mod tu537;
use core::ffi::c_void;
use melee_modes::tu::gmscmemcard as t;
use tu537::Rng;

const FUNC_NAMES: [&str; 6] = [
    "gm_801AEE6C",
    "gm_801AF0D4",
    "gm_801AF250",
    "OnFrame",
    "gm_Scene_MemCard_OnEnter",
    "gm_Scene_MemCard_OnExit",
];
const NCASES: [usize; 6] = [300, 500, 500, 3000, 300, 100];
const PADBITS: [u32; 10] = [0, 1, 2, 3, 16, 17, 18, 19, 8, 12];

static mut LOG: String = String::new();
static mut PAD: u32 = 0;
static mut STATUS: u32 = 0;
static mut PROBE: i32 = 0;
static mut CBBC: i32 = 0;
static mut CC4C: i32 = 0;
static mut C8BC: i32 = 0;
static mut B8C8: bool = false;
static mut US: bool = false;
static mut SKIP: i32 = 0;
static mut MEXEN: i32 = 0;
static mut SETTINGS: i32 = 0;
static mut DBLVL: i32 = 0;
static mut PADBTN: u32 = 0;
static mut PADTRG: u32 = 0;

fn bits(f: f32) -> String {
    format!("{:08x}", f.to_bits())
}
fn log(s: &str) {
    unsafe { LOG.push_str(s) }
}

fn install_hooks() {
    use t::ext::*;
    unsafe {
        GM_801AEDC8 = Some(|| {
            log("pad;");
            unsafe { PAD }
        });
        GM_801AE848 = Some(|a| log(&format!("848({});", a)));
        GM_801ADE1C = Some(|a, b, x, y| {
            log(&format!("DE1C({},{},{},{});", a, b, bits(x), bits(y)));
            true
        });
        GM_801AE44C = Some(|a, v| log(&format!("E44C({},{});", a, bits(v))));
        GM_801AE544 = Some(|a, v| log(&format!("E544({},{});", a, bits(v))));
        GM_801AE050 = Some(|a, b, c, x, y| {
            log(&format!("E050({},{},{},{},{});", a, b, c, bits(x), bits(y)));
            true
        });
        GM_801AE640 = Some(|a, b| log(&format!("E640({},{});", a, b as i32)));
        GM_801AE74C = Some(|a, b| log(&format!("E74C({},{});", a, b as i32)));
        GM_801ADDD8 = Some(|| log("DDD8;"));
        LBCARDNEW_PROBEEX = Some(|chan| {
            log(&format!("probe({});", chan));
            unsafe { PROBE }
        });
        LBCARDNEW_ALLOCWORKAREA = Some(|| log("alloc;"));
        LBCARDGAME_LOADARCHIVE = Some(|a| log(&format!("loadarch({});", a)));
        LBCARDGAME_SETCARDSTATUS = Some(|s| log(&format!("setstatus({});", s)));
        LBCARDGAME_SAVECHANGES = Some(|| log("save;"));
        LB_8001C87C = Some(|| {
            log("c87C;");
            unsafe { STATUS }
        });
        LB_8001CBBC = Some(|| {
            log("cbbc;");
            unsafe { CBBC }
        });
        LB_8001CC4C = Some(|| {
            log("cc4c;");
            unsafe { CC4C }
        });
        LB_8001C8BC = Some(|| {
            log("c8bc;");
            unsafe { C8BC }
        });
        LB_8001B8C8 = Some(|chan| {
            log(&format!("b8c8({});", chan));
            unsafe { B8C8 }
        });
        LBLANG_ISSAVEDLANGUAGEUS = Some(|| unsafe { US });
        GMMAINLIB_8015FA34 = Some(|v| log(&format!("fa34({});", v)));
        GM_801A4B60 = Some(|| log("4B60;"));
        LBAUDIOAX_80024030 = Some(|id| log(&format!("sfx({});", id)));
        DBLEVEL = Some(|| unsafe { DBLVL });
        HSD_PADCOPYSTATUS0_BUTTON = Some(|| unsafe { PADBTN });
        HSD_PADCOPYSTATUS0_TRIGGER = Some(|| unsafe { PADTRG });
        SCENEREPORT_MEMCARD = Some(|d, o| log(&format!("rep({},{});", d, o)));
        SCENEREPORT_CURSOR = Some(|w, a, b| log(&format!("cursor({},{},{});", w, a, b)));
        SCENELAUNCH_SKIPMEMCARD = Some(|| {
            log("skip;");
            unsafe { SKIP }
        });
        MEX_ENABLED = Some(|n| {
            log(&format!("mex({});", n));
            unsafe { MEXEN }
        });
        SETTINGS_INT = Some(|k, d| {
            log(&format!("set({},{});", k, d));
            unsafe { SETTINGS }
        });
    }
}

#[test]
fn matches_c() {
    let want = tu537::data("tu_gmscmemcard");
    install_hooks();
    let mut r = Rng::new();
    let mut lines = want.lines();
    for f in 0..FUNC_NAMES.len() {
        for i in 0..NCASES[f] {
            unsafe {
                // setup_case(), in the C driver's order.
                PAD = 0;
                for k in 0..10 {
                    if r.rnd() % 3 == 0 {
                        PAD |= 1u32 << PADBITS[k];
                    }
                }
                t::enter_data.decision = (r.rnd() % 22) as i32;
                t::enter_data.unk0.unk0 = (r.rnd() % 3) as i32;
                t::enter_data.unk0.chan = (r.rnd() % 2) as u8;
                t::enter_data.unk0.mode_id = r.rnd() as u8;
                t::enter_data.unk8.unk0 = (r.rnd() % 2) as i32;
                t::enter_data.unk8.mode_id = r.rnd() as u8;
                t::enter_data.unk10 = (r.rnd() % 2) as i32;
                t::enter_data.unk18 = (r.rnd() % 2) as i32;
                t::enter_data.unk1C = (r.rnd() % 2) as u8;
                t::gm_804D6870 = (r.rnd() % 2) as u8;
                t::gm_804D6872 = (6 + r.rnd() % 19) as u16;
                PROBE = (r.rnd() % 2) as i32;
                STATUS = r.rnd() % 17;
                CBBC = [0, 2, 0xF, 0xD, 1, 3, 5][(r.rnd() % 7) as usize];
                CC4C = (r.rnd() % 2) as i32;
                C8BC = (r.rnd() % 2) as i32;
                B8C8 = r.rnd() % 2 != 0;
                US = r.rnd() % 2 != 0;
                DBLVL = (r.rnd() % 5) as i32;
                SKIP = (r.rnd() % 4 == 0) as i32;
                MEXEN = (r.rnd() % 2) as i32;
                SETTINGS = (r.rnd() % 2) as i32;
                PADBTN = 0;
                PADTRG = 0;
                if r.rnd() % 2 != 0 {
                    PADBTN |= t::HSD_PAD_L;
                }
                if r.rnd() % 2 != 0 {
                    PADBTN |= t::HSD_PAD_R;
                }
                if r.rnd() % 2 != 0 {
                    PADBTN |= t::HSD_PAD_A;
                }
                if r.rnd() % 2 != 0 {
                    PADBTN |= t::HSD_PAD_B;
                }
                if r.rnd() % 2 != 0 {
                    PADTRG |= t::HSD_PAD_L;
                }
                if r.rnd() % 2 != 0 {
                    PADTRG |= t::HSD_PAD_R;
                }

                let a = r.rnd();
                let b = r.rnd();
                let c = r.rnd();
                let rr = r.rnd();
                let use_in = rr % 4 != 0;
                let mut inp = t::enterData::default();
                inp.unk0.unk0 = (a % 3) as i32;
                inp.unk0.chan = (b % 2) as u8;
                inp.unk0.mode_id = c as u8;
                inp.decision = 99;
                let mut out = t::exitData { unk0: 77, mode_id: 66 };
                LOG.clear();
                let mut ret = 0;
                match f {
                    0 => t::gm_801AEE6C((a % 4) as i32, (b % 25) as i32, (c % 6) as i32 + 2),
                    1 => ret = t::gm_801AF0D4() as i32,
                    2 => t::gm_801AF250(),
                    3 => t::gm_Scene_MemCard_OnFrame(),
                    4 => t::gm_Scene_MemCard_OnEnter(if use_in { &mut inp as *mut _ as *mut c_void } else { core::ptr::null_mut() }),
                    _ => t::gm_Scene_MemCard_OnExit(if use_in { &mut out as *mut _ as *mut c_void } else { core::ptr::null_mut() }),
                }
                let e = &t::enter_data;
                let got = format!(
                    "{}.{} {}|log={}|ret={}|ed={},{},{},{},{},{},{},{},{}|g={},{}|ex={},{}",
                    f,
                    i,
                    FUNC_NAMES[f],
                    LOG,
                    ret,
                    e.unk0.unk0,
                    e.unk0.chan,
                    e.unk0.mode_id,
                    e.unk8.unk0,
                    e.unk8.mode_id,
                    e.unk10,
                    e.decision,
                    e.unk18,
                    e.unk1C,
                    t::gm_804D6870,
                    t::gm_804D6872,
                    out.unk0,
                    out.mode_id
                );
                let line = lines.next().expect("fewer lines than cases");
                assert_eq!(got, line, "function {} case {}", FUNC_NAMES[f], i);
            }
        }
    }
    assert!(lines.next().is_none(), "more lines than cases");
}
