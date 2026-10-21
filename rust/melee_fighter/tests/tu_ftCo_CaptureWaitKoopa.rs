//! src/tu/ftCo_CaptureWaitKoopa.rs vs the C (gen/cref/tu/ftCo_CaptureWaitKoopa.c: the whole TU, shipping contraction;
//! regenerate with `python rust/melee_fighter/gen/cref/tu/run_tu.py ftCo_CaptureWaitKoopa`).
#![allow(static_mut_refs, non_snake_case)]
mod tu_ftco_harness;
mod tu_ftco_cargo;
use tu_ftco_harness::*;
use tu_ftco_cargo::*;
use melee_fighter::types::*;

macro_rules! v { ($($n:ident = $id:expr),*) => { unsafe { $( hk::$n = Some(|g| { t($id); t(pi(g)) }); )* } } }
const CMS: fn(*mut HSD_GObj, i32, u32, f32, f32, f32, *mut HSD_GObj) = |g, ms, fl, a, bb, c, o| {
    t(9); t(pi(g)); t(ms as u32); t(fl); t(b(a)); t(b(bb)); t(b(c)); t(pi(o))
};
fn kf(i: usize) -> f32 { ((unsafe { SRV[i & 15] } % 41) as i32 - 20) as f32 / 4.0 }
fn pf(f: &mut Fighter) -> u32 { pi(f as *const Fighter) }
use melee_fighter::tu::ftCo_CaptureWaitKoopa as m;
use m::ext::hook as hk;

#[test]
fn ftCo_CaptureWaitKoopa_matches_c() {
    cargo_init();
    v!(ftCo_800DB464 = 1, ftCo_800BC458 = 3, ftCo_800BC4A8 = 4, ftCo_CaptureCut_Enter = 6);
    unsafe {
        hk::Fighter_ChangeMotionState = Some(CMS);
        hk::ftCommon_8007E2F4 = Some(|f, x| { t(2); t(pf(f)); t(x as i32 as u32) });
        hk::ftCo_800DA698 = Some(|g, x| { t(5); t(pi(g)); t(x as u32) });
    }
    let mut c = Check::new(include_str!("data/tu_ftCo_CaptureWaitKoopa.txt"));
    for op in 0..12u32 {
        for k in 0..1000u32 {
            case(op, k);
            cargo_fill();
            let g = go();
            let ret: u32 = match op {
                0 => { m::ftCo_800BCC20(g); 0 }
                1 => { m::ftCo_CaptureWaitKoopa_Anim(g); 0 }
                2 => { m::ftCo_CaptureWaitKoopa_IASA(g); 0 }
                3 => { m::ftCo_CaptureWaitKoopa_Phys(g); 0 }
                4 => { m::ftCo_CaptureWaitKoopa_Coll(g); 0 }
                5 => { m::ftCo_800BCD00(g); 0 }
                6 => { m::ftCo_CaptureWaitKoopaAir_Anim(g); 0 }
                7 => { m::ftCo_CaptureWaitKoopaAir_IASA(g); 0 }
                8 => { m::ftCo_CaptureWaitKoopaAir_Phys(g); 0 }
                9 => { m::ftCo_CaptureWaitKoopaAir_Coll(g); 0 }
                10 => { m::ftCo_CaptureWaitKoopa_Anim(g); f().grab_timer = 0.0; m::ftCo_CaptureWaitKoopaAir_Anim(g); 0 }
                _ => { f().grab_timer = 1.0; m::ftCo_CaptureWaitKoopa_Anim(g); 0 }
            };
            c.line(op, k, ret, state_hash());
        }
    }
    assert_eq!(c.done(), 12000);
}
