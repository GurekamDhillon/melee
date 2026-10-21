//! src/tu/ftCo_CatchCut.rs vs the C (gen/cref/tu/ftCo_CatchCut.c: the whole TU, shipping
//! contraction; regenerate with `python rust/melee_fighter/gen/cref/tu/run_tu.py ftCo_CatchCut`).
#![allow(static_mut_refs, non_snake_case)]
mod tu_ftco_harness;
use tu_ftco_harness::*;
use melee_fighter::tu::ftCo_CatchCut as m;
use m::ext::hook as hk;
use melee_fighter::types::*;

#[test]
fn ftCo_CatchCut_matches_c() {
    init();
    unsafe {
        hk::Fighter_ChangeMotionState = Some(|g, ms, fl, a, bb, c, o| {
            t(1); t(is_go(g)); t(ms as u32); t(fl); t(b(a)); t(b(bb)); t(b(c)); t(o.is_null() as u32)
        });
        hk::ftCo_CaptureCut_Enter = Some(|g| { t(2); t((g == go2()) as u32) });
        hk::ftAnim_IsFramesRemaining = Some(|g| { t(3); t(is_go(g)); sr(1) });
        hk::ftCommon_8007D92C = Some(|g| { t(4); t(is_go(g)) });
        hk::ftCommon_CalcGroundAccel_Deaccel = Some(|fr, x| { t(5); t((fr as *mut Fighter == F) as u32); t(b(x)) });
        hk::ftCommon_SetSelfMovementFromGroundedMovement = Some(|g| { t(6); t(is_go(g)) });
        hk::ft_80084DB0 = Some(|g| { t(7); t(is_go(g)) });
        hk::ft_80084104 = Some(|g| { t(8); t(is_go(g)) });
        hk::ft_80082B1C = Some(|g| { t(9); t(is_go(g)) });
        hk::ft_8008370C = Some(|g, cb| {
            t(10); t(is_go(g));
            t((cb.map(|p| p as usize) == Some(m::ft_80082B1C_cb as usize)) as u32);
            unsafe { (cb.unwrap())(g) }
        });
    }
    let mut c = Check::new(include_str!("data/tu_ftCo_CatchCut.txt"));
    for op in 0..5u32 {
        for k in 0..2000u32 {
            case(op, k);
            let fr = f();
            fr.ground_or_air = rnd() & 1;
            fr.facing_dir = rg();
            fr.victim_gobj = go2();
            let d = cd();
            d.x370 = rw(); d.x374 = rw(); d.x378 = rw(); d.x64 = rg();
            fr.co_attrs.ground_friction = rg();
            fr.self_vel.x = rg(); fr.self_vel.y = rg(); fr.self_vel.z = rg(); fr.gr_vel = rg();
            let g = go();
            match op {
                0 => m::ftCo_800DA698(g, rnd() & 1 != 0),
                1 => m::ftCo_CatchCut_Anim(g),
                2 => m::ftCo_CatchCut_IASA(g),
                3 => m::ftCo_CatchCut_Phys(g),
                _ => m::ftCo_CatchCut_Coll(g),
            }
            let fr = f();
            let mut h = Hs::new();
            h.m(b(fr.self_vel.x)); h.m(b(fr.self_vel.y)); h.m(b(fr.self_vel.z)); h.m(b(fr.gr_vel));
            c.line(op, k, 0, h.0);
        }
    }
    assert_eq!(c.done(), 10000);
}
