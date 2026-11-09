//! tu::gm_1A36 vs the C (state oracle): gen/turef/tu_gm_1A36_ref.c is the game's gm_1A36.c compiled natively
//! (shipping contraction); the expected lines are tests/data/tu_gm_1A36.txt (regenerate with
//! `python rust/melee_modes/gen/turef/tu_shard537_gm.py tu_gm_1A36`). Each line is the callee log, the return
//! value and the whole `controller_map` after the call.
#![allow(static_mut_refs)]
#[path = "../../melee_core/test_support/fnsym.rs"]
mod fnsym;

mod tu537;
use melee_modes::tu::gm_1A36 as t;
use melee_types::*;
use tu537::Rng;

const NCASES: usize = 300;
const FUNC_NAMES: [&str; 10] = [
    "gm_GetButtonsPressed",
    "gm_GetButtonsTriggered",
    "gm_801A36C0",
    "gm_801A36E0",
    "gm_801A3714",
    "gm_801A3820",
    "fn_801A396C",
    "gm_EvaluateAllControllerInputs",
    "gm_801A3E88",
    "gm_801A3EF4",
];
const BITS: [u32; 13] = [0, 1, 2, 3, 5, 6, 8, 9, 12, 16, 17, 18, 19];

static mut LOG: String = String::new();
static mut PADS: [HSD_PadStatus; 4] = unsafe { core::mem::zeroed() };
static mut MODES: [GameMode; 4] = unsafe { core::mem::zeroed() };

unsafe extern "C" fn init_a() {
    LOG.push_str("ia;");
}
unsafe extern "C" fn init_b() {
    LOG.push_str("ib;");
}

fn rbtn(r: &mut Rng) -> u64 {
    let mut v = 0u64;
    for k in 0..BITS.len() {
        let x = r.rnd();
        if x % 3 == 0 {
            v |= 1u64 << BITS[k];
        }
    }
    let x = r.rnd();
    if x % 4 == 0 {
        let s = r.rnd();
        v |= 1u64 << (s % 40);
    }
    v
}

#[test]
fn matches_c() {
    let want = tu537::data("tu_gm_1A36");
    unsafe {
        t::ext::HSD_PADCOPYSTATUS = Some(|| PADS.as_mut_ptr());
        t::ext::GM_GETALLGAMEMODES = Some(|| MODES.as_mut_ptr());
    }
    let mut r = Rng::new();
    let mut lines = want.lines();
    for f in 0..FUNC_NAMES.len() {
        for i in 0..NCASES {
            unsafe {
                // setup_case(), in the C driver's order.
                for k in 0..5 {
                    let c = &mut t::controller_map.x0[k];
                    c.button = rbtn(&mut r);
                    c.trigger = if r.rnd() % 2 != 0 { rbtn(&mut r) } else { 0 };
                    c.repeat = rbtn(&mut r);
                    c.release = if r.rnd() % 2 != 0 { rbtn(&mut r) } else { 0 };
                    c.repeat2 = rbtn(&mut r);
                    let x = r.rnd();
                    c.repeat_timer = if x % 4 == 0 { 0 } else { ((x >> 8) as i32) % 7 - 2 };
                    let x = r.rnd();
                    c.x2C = (x % 0x72) as i32;
                }
                t::controller_map.xF0 = Some(t::fn_801A396C);
                t::controller_map.xF4 = (r.rnd() % 0x30) as u16;
                t::controller_map.xF6 = (r.rnd() % 10) as u8;
                t::controller_map.xF8 = (r.rnd() % 0x50) as u16;
                t::controller_map.xFA = (r.rnd() % 10) as u8;
                t::controller_map.xFC = (r.rnd() % 0x70) as u16;
                t::controller_map.xFE = (r.rnd() % 10) as u8;
                for k in 0..4 {
                    PADS[k].button = rbtn(&mut r) as u32;
                    PADS[k].trigger = rbtn(&mut r) as u32;
                    PADS[k].repeat = rbtn(&mut r) as u32;
                    PADS[k].release = rbtn(&mut r) as u32;
                }
                MODES = core::mem::zeroed();
                MODES[0].kind = 1;
                MODES[0].on_init = Some(init_a);
                MODES[1].kind = 2;
                MODES[2].kind = 3;
                MODES[2].on_init = Some(init_b);
                MODES[3].kind = GM_COUNT as u8;
                MODES[3].on_init = Some(init_a);

                LOG.clear();
                let idx = (r.rnd() % 5) as i32;
                let a1 = rbtn(&mut r);
                let mut a2 = rbtn(&mut r);
                a2 |= 1u64 << (32 + r.rnd() % 8);
                let mut ret = 0u64;
                match f {
                    0 => ret = t::gm_GetButtonsPressed(idx as u8),
                    1 => ret = t::gm_GetButtonsTriggered(idx as u8),
                    2 => ret = t::gm_801A36C0(idx as u8),
                    3 => t::gm_801A36E0(idx, (r.rnd() % 100) as i32 - 20),
                    4 => t::gm_801A3714(idx, a1, a2),
                    5 => t::gm_801A3820(idx, a1, a2),
                    6 => t::fn_801A396C(idx),
                    7 => t::gm_EvaluateAllControllerInputs(),
                    8 => t::gm_801A3E88(),
                    _ => t::gm_801A3EF4(),
                }
                let mut got = format!("{}.{} {}|log={}|ret={:x}", f, i, FUNC_NAMES[f], LOG, ret);
                for k in 0..5 {
                    let c = &t::controller_map.x0[k];
                    got += &format!(
                        "|{:x},{:x},{:x},{:x},{:x},{},{}",
                        c.button, c.trigger, c.repeat, c.release, c.repeat2, c.repeat_timer, c.x2C
                    );
                }
                let m = &t::controller_map;
                let fl = match m.xF0 {
                    None => 0,
                    Some(p) if fnsym::same(p as usize, t::fn_801A396C as usize) => 1,
                    Some(_) => 2,
                };
                got += &format!("|xF0={},{},{},{},{},{},{}", fl, m.xF4, m.xF6, m.xF8, m.xFA, m.xFC, m.xFE);
                let line = lines.next().expect("fewer lines than cases");
                assert_eq!(got, line, "function {} case {}", FUNC_NAMES[f], i);
            }
        }
    }
    assert!(lines.next().is_none(), "more lines than cases");
}
