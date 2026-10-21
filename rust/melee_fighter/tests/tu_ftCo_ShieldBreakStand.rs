//! src/tu/ftCo_ShieldBreakStand.rs vs the C (gen/cref/tu/ftCo_ShieldBreakStand.c: the whole TU,
//! shipping contraction; regenerate with
//! `python rust/melee_fighter/gen/cref/tu/run_tu.py ftCo_ShieldBreakStand`).
#![allow(static_mut_refs, non_snake_case)]
mod tu_ftco_harness;
mod tu_ftco_s209;
use tu_ftco_harness::*;
use tu_ftco_s209::*;
use melee_fighter::tu::ftCo_ShieldBreakStand as m;
use m::ext::hook as hk;

macro_rules! vs { ($($n:ident = $id:expr),*) => { unsafe { $( hk::$n = Some(|g| { t($id); t(pi(g)) }); )* } } }

#[test]
fn ftCo_ShieldBreakStand_matches_c() {
    s209_init();
    unsafe {
        hk::Fighter_ChangeMotionState = Some(|g, ms, fl, a, bb, c, o| {
            t(1); t(pi(g)); t(ms as u32); t(fl); t(b(a)); t(b(bb)); t(b(c)); t(pi(o))
        });
        hk::ftAnim_IsFramesRemaining = Some(|g| { t(2); t(pi(g)); sr(2) });
    }
    vs!(ftCo_80099010 = 3, ft_80084F3C = 4, ft_80083F88 = 5);
    let mut c = Check::new(include_str!("data/tu_ftCo_ShieldBreakStand.txt"));
    for op in 0..5u32 {
        for k in 0..1000u32 {
            case(op, k);
            s209_fill();
            let g = go();
            match op {
                0 => m::ftCo_80098F3C(g),
                1 => m::ftCo_ShieldBreakStand_Anim(g),
                2 => m::ftCo_ShieldBreakStand_IASA(g),
                3 => m::ftCo_ShieldBreakStand_Phys(g),
                _ => m::ftCo_ShieldBreakStand_Coll(g),
            }
            c.line(op, k, 0, state_hash());
        }
    }
    assert_eq!(c.done(), 5000);
}
