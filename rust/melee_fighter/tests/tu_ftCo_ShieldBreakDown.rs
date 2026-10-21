//! src/tu/ftCo_ShieldBreakDown.rs vs the C (gen/cref/tu/ftCo_ShieldBreakDown.c: the whole TU,
//! shipping contraction; regenerate with
//! `python rust/melee_fighter/gen/cref/tu/run_tu.py ftCo_ShieldBreakDown`).
#![allow(static_mut_refs, non_snake_case)]
mod tu_ftco_harness;
mod tu_ftco_s209;
use tu_ftco_harness::*;
use tu_ftco_s209::*;
use melee_fighter::tu::ftCo_ShieldBreakDown as m;
use m::ext::hook as hk;
use melee_fighter::types::*;

macro_rules! vs { ($($n:ident = $id:expr),*) => { unsafe { $( hk::$n = Some(|g| { t($id); t(pi(g)) }); )* } } }
macro_rules! bs { ($($n:ident = $id:expr),*) => { unsafe { $( hk::$n = Some(|g| { t($id); t(pi(g)); sr($id) }); )* } } }

#[test]
fn ftCo_ShieldBreakDown_matches_c() {
    s209_init();
    unsafe {
        hk::ftCommon_8007D7FC = Some(|f| { t(1); t(pi(f as *const Fighter)) });
        hk::Fighter_ChangeMotionState = Some(|g, ms, fl, a, bb, c, o| {
            t(3); t(pi(g)); t(ms as u32); t(fl); t(b(a)); t(b(bb)); t(b(c)); t(pi(o))
        });
    }
    bs!(ftCo_80097570 = 2, ftAnim_IsFramesRemaining = 5);
    vs!(ftCo_800978D4 = 4, ftCo_80098F3C = 6, ft_80084F3C = 7, ft_80083F88 = 8);
    let mut c = Check::new(include_str!("data/tu_ftCo_ShieldBreakDown.txt"));
    for op in 0..5u32 {
        for k in 0..1000u32 {
            case(op, k);
            s209_fill();
            let g = go();
            match op {
                0 => m::ftCo_80098E3C(g),
                1 => m::ftCo_ShieldBreakDown_Anim(g),
                2 => m::ftCo_ShieldBreakDown_IASA(g),
                3 => m::ftCo_ShieldBreakDown_Phys(g),
                _ => m::ftCo_ShieldBreakDown_Coll(g),
            }
            c.line(op, k, 0, state_hash());
        }
    }
    assert_eq!(c.done(), 5000);
}
