//! src/tu/ftCo_MissFoot.rs vs the C (gen/cref/tu/ftCo_MissFoot.c: the whole TU, shipping
//! contraction; regenerate with `python rust/melee_fighter/gen/cref/tu/run_tu.py ftCo_MissFoot`).
#![allow(static_mut_refs, non_snake_case)]
mod tu_ftco_harness;
use tu_ftco_harness::*;
use melee_fighter::tu::ftCo_MissFoot as m;
use m::ext::hook as hk;
use melee_fighter::types::*;

fn isf(fr: &mut Fighter) -> u32 { (fr as *mut Fighter == f() as *mut Fighter) as u32 }

#[test]
fn ftCo_MissFoot_matches_c() {
    init();
    unsafe {
        hk::Fighter_ChangeMotionState = Some(|g, ms, fl, a, bb, c, o| {
            t(1); t(is_go(g)); t(ms as u32); t(fl); t(b(a)); t(b(bb)); t(b(c)); t(o.is_null() as u32)
        });
        hk::ftCommon_ClampAirDrift = Some(|fr| { t(2); t(isf(fr)) });
        hk::ftCommon_8007D5D4 = Some(|fr| { t(3); t(isf(fr)) });
        hk::ftAnim_IsFramesRemaining = Some(|g| { t(4); t(is_go(g)); sr(4) });
        hk::ftCo_80090780 = Some(|g| { t(5); t(is_go(g)) });
        hk::ft_80084DB0 = Some(|g| { t(6); t(is_go(g)) });
        hk::ft_80082F28 = Some(|g| { t(7); t(is_go(g)) });
    }
    let mut c = Check::new(include_str!("data/tu_ftCo_MissFoot.txt"));
    for op in 0..5u32 {
        for k in 0..1000u32 {
            case(op, k);
            let fr = f();
            fr.x8c_kb_vel.x = rw(); fr.x8c_kb_vel.y = rw();
            fr.ground_or_air = (rnd() & 1) as _;
            let g = go();
            match op {
                0 => m::ftCo_8009F39C(g),
                1 => m::ftCo_MissFoot_Anim(g),
                2 => m::ftCo_MissFoot_IASA(g),
                3 => m::ftCo_MissFoot_Phys(g),
                _ => m::ftCo_MissFoot_Coll(g),
            }
            let fr = f();
            let mut h = Hs::new();
            h.m(b(fr.x8c_kb_vel.x)); h.m(b(fr.x8c_kb_vel.y)); h.m(fr.ground_or_air as u32);
            c.line(op, k, 0, h.0);
        }
    }
    assert_eq!(c.done(), 5000);
}
