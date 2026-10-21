//! src/tu/ftCo_ShieldBreakFall.rs vs the C (gen/cref/tu/ftCo_ShieldBreakFall.c: the whole TU,
//! shipping contraction; regenerate with
//! `python rust/melee_fighter/gen/cref/tu/run_tu.py ftCo_ShieldBreakFall`).
#![allow(static_mut_refs, non_snake_case)]
mod tu_ftco_harness;
mod tu_ftco_s209;
use tu_ftco_harness::*;
use tu_ftco_s209::*;
use melee_fighter::tu::ftCo_ShieldBreakFall as m;
use m::ext::hook as hk;
use melee_fighter::types::*;

#[test]
fn ftCo_ShieldBreakFall_matches_c() {
    s209_init();
    unsafe {
        hk::Fighter_ChangeMotionState = Some(|g, ms, fl, a, bb, c, o| {
            t(1); t(pi(g)); t(ms as u32); t(fl); t(b(a)); t(b(bb)); t(b(c)); t(pi(o))
        });
        hk::ftCommon_ClampAirDrift = Some(|f| { t(2); t(pi(f as *const Fighter)) });
        hk::ftCommon_8007EBAC = Some(|f, a, bb| { t(3); t(pi(f as *const Fighter)); t(a); t(bb) });
        hk::ft_80084EEC = Some(|g| { t(4); t(pi(g)) });
        hk::ft_80082C74 = Some(|g, cb| {
            t(5); t(pi(g));
            t((cb as usize == melee_fighter::tu::ftCo_ShieldBreakDown::ftCo_80098E3C as usize) as u32)
        });
    }
    let mut c = Check::new(include_str!("data/tu_ftCo_ShieldBreakFall.txt"));
    for op in 0..5u32 {
        for k in 0..1000u32 {
            case(op, k);
            s209_fill();
            let g = go();
            match op {
                0 => m::ftCo_80098D90(g),
                1 => m::ftCo_ShieldBreakFall_Anim(g),
                2 => m::ftCo_ShieldBreakFall_IASA(g),
                3 => m::ftCo_ShieldBreakFall_Phys(g),
                _ => m::ftCo_ShieldBreakFall_Coll(g),
            }
            c.line(op, k, 0, state_hash());
        }
    }
    assert_eq!(c.done(), 5000);
}
