//! src/tu/ftCo_ThrownMewtwo.rs vs the C (gen/cref/tu/ftCo_ThrownMewtwo.c: the whole TU, shipping
//! contraction; regenerate with `python rust/melee_fighter/gen/cref/tu/run_tu.py ftCo_ThrownMewtwo`).
#![allow(static_mut_refs, non_snake_case)]
mod tu_ftco_harness;
use tu_ftco_harness::*;
use melee_fighter::tu::ftCo_ThrownMewtwo as m;
use m::ext::hook as hk;
use melee_fighter::types::*;

#[test]
fn ftCo_ThrownMewtwo_matches_c() {
    init();
    unsafe {
        hk::Fighter_ChangeMotionState = Some(|g, ms, fl, a, bb, c, o| {
            t(1); t(is_go(g)); t(ms as u32); t(fl); t(b(a)); t(b(bb)); t(b(c)); t((o == go2()) as u32)
        });
        hk::ftCo_800DE508 = Some(|_| t(2));
        hk::ftCommon_8007E2F4 = Some(|fr, v| { t(3); t((fr as *const Fighter == f() as *const Fighter) as u32); t(v as i32 as u32) });
        hk::ftAnim_8006EBA4 = Some(|g| { t(4); t(is_go(g)) });
    }
    let mut c = Check::new(include_str!("data/tu_ftCo_ThrownMewtwo.txt"));
    for op in 0..2u32 {
        for k in 0..2000u32 {
            case(op, k);
            let fr = f();
            fr.victim_gobj = go2();
            f2().facing_dir = rg();
            fr.facing_dir = rg();
            fr.set_x2222_b6((rnd() & 1) as _);
            fr.cur_anim_frame = rw();
            unsafe { fr.mv.co.capturekoopa.x0 = (rnd() & 1) as i32 };
            let g = go();
            match op {
                0 => m::ftCo_800BD0E8(g, (rnd() % 400) as i32),
                _ => {
                    m::ftCo_ThrownMewtwo_Anim(g); m::ftCo_ThrownMewtwo_IASA(g); m::ftCo_ThrownMewtwo_Phys(g); m::ftCo_ThrownMewtwo_Coll(g);
                    m::ftCo_ThrownMewtwoAir_Anim(g); m::ftCo_ThrownMewtwoAir_IASA(g); m::ftCo_ThrownMewtwoAir_Phys(g); m::ftCo_ThrownMewtwoAir_Coll(g);
                }
            }
            let fr = f();
            let mut h = Hs::new();
            h.m(b(fr.facing_dir));
            unsafe { h.m(fr.mv.co.capturekoopa.x0 as u32) };
            h.m(fr.accessory1_cb.is_some() as u32);
            c.line(op, k, 0, h.0);
        }
    }
    assert_eq!(c.done(), 4000);
}
