//! src/tu/ftCo_RunBrake.rs vs the C (gen/cref/tu/ftCo_RunBrake.c: the whole TU, shipping
//! contraction; regenerate with `python rust/melee_fighter/gen/cref/tu/run_tu.py ftCo_RunBrake`).
#![allow(static_mut_refs, non_snake_case)]
mod tu_ftco_harness;
mod tu_ftco_s209;
use tu_ftco_harness::*;
use tu_ftco_s209::*;
use melee_fighter::tu::ftCo_RunBrake as m;
use m::ext::hook as hk;
use melee_fighter::types::*;

macro_rules! vs { ($($n:ident = $id:expr),*) => { unsafe { $( hk::$n = Some(|g| { t($id); t(pi(g)) }); )* } } }
macro_rules! bs { ($($n:ident = $id:expr),*) => { unsafe { $( hk::$n = Some(|g| { t($id); t(pi(g)); sr($id) }); )* } } }

#[test]
fn ftCo_RunBrake_matches_c() {
    s209_init();
    unsafe {
        hk::Fighter_ChangeMotionState = Some(|g, ms, fl, a, bb, c, o| {
            t(1); t(pi(g)); t(ms as u32); t(fl); t(b(a)); t(b(bb)); t(b(c)); t(pi(o))
        });
        hk::ft_PlaySFX = Some(|f, id, v, p| { t(2); t(pi(f as *const Fighter)); t(id as u32); t(v as u32); t(p as u32) });
        hk::ftAnim_SetAnimRate = Some(|g, r| { t(3); t(pi(g)); t(b(r)) });
        hk::ftCommon_CalcGroundAccel_Deaccel = Some(|f, v| { t(9); t(pi(f as *const Fighter)); t(b(v)) });
    }
    bs!(ftAnim_IsFramesRemaining = 4, fn_800CAF78 = 6, fn_800C9CEC = 7, ftCo_800D5FB0 = 8);
    vs!(ft_8008A2BC = 5, ftCommon_SetSelfMovementFromGroundedMovement = 10, ft_80084280 = 11);
    let mut c = Check::new(include_str!("data/tu_ftCo_RunBrake.txt"));
    for op in 0..6u32 {
        for k in 0..1000u32 {
            case(op, k);
            s209_fill();
            let g = go();
            let ret: u32 = match op {
                0 => m::ftCo_RunBrake_CheckInput(g) as u32,
                1 => { m::ftCo_RunBrake_Enter(g); 0 }
                2 => { m::ftCo_RunBrake_Anim(g); 0 }
                3 => { m::ftCo_RunBrake_IASA(g); 0 }
                4 => { m::ftCo_RunBrake_Phys(g); 0 }
                _ => { m::ftCo_RunBrake_Coll(g); 0 }
            };
            c.line(op, k, ret, state_hash());
        }
    }
    assert_eq!(c.done(), 6000);
}
