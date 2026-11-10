//! ftLib_UpdateLogicOffCamera (src/tu/ftlib.rs) vs the C (gen/cref/tu/ftlib_logic.c: ftlib.c whole;
//! regenerate with `python rust/melee_fighter/gen/cref/tu/run_tu.py ftlib_logic`).
#![allow(static_mut_refs, non_snake_case)]
mod tu_ftco_harness;
use melee_fighter::hooks as h;
use melee_fighter::tu::ftlib as m;
use melee_fighter::types::*;
use tu_ftco_harness::*;

static mut CB: *mut CmSubject = core::ptr::null_mut();
fn ep<T>(p: *const T) -> u32 {
    unsafe { if p.is_null() { 0 } else if p as usize == CB as usize { 1 } else if p as usize == GO as usize { 2 } else { 3 } }
}
#[test]
fn ft_lib_update_logic_off_camera_matches_c() {
    init();
    unsafe {
        CB = Box::leak(Box::new(core::mem::zeroed()));
        h::CAMERA_80030A50 = Some(|| { t(1); if sr(0) { go2() } else { core::ptr::null_mut() } });
        h::CAMERA_LOGIC_TO_SCREEN = Some(|s, pt| { t(2); t(ep(s)); pt.x = rnd() as i32; pt.y = rnd() as i32; sr(1) });
    }
    let mut c = Check::new(include_str!("data/tu_ftlib_logic.txt"));
    for k in 0..2000u32 {
        case(0, k);
        unsafe {
            let f = f();
            f.x890_cameraBox = CB;
            f.set_is_sleeping((rnd() & 1) as _); f.set_x2229_b3((rnd() & 1) as _); f.set_x2220_b7((rnd() & 1) as _); f.set_x221F_b0((rnd() & 1) as _);
            f.x2188.x = rnd() as i32; f.x2188.y = rnd() as i32;
            m::ftLib_UpdateLogicOffCamera(go());
            let mut hh: u32 = 0x811C_9DC5;
            for v in [f.x221F_b0() as u32, f.x2188.x as u32, f.x2188.y as u32] { hh = hh.wrapping_mul(0x0100_0193) ^ v; }
            c.line(0, k, 0, hh);
        }
    }
    assert_eq!(c.done(), 2000);
}
