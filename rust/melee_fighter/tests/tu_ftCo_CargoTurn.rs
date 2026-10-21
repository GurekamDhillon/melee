//! src/tu/ftCo_CargoTurn.rs vs the C (gen/cref/tu/ftCo_CargoTurn.c: the whole TU, shipping
//! contraction; regenerate with `python rust/melee_fighter/gen/cref/tu/run_tu.py ftCo_CargoTurn`).
#![allow(static_mut_refs, non_snake_case)]
mod tu_ftco_harness;
use tu_ftco_harness::*;
use melee_fighter::tu::ftCo_CargoTurn as m;
use m::ext::hook as hk;
use melee_fighter::types::*;

#[test]
fn ftCo_CargoTurn_matches_c() {
    init();
    let dk = Box::leak(Box::new(ftDonkeyAttributes::zeroed()));
    unsafe {
        hk::ftCo_800C97A8 = Some(|g| { t(1); t(is_go(g)); sr(1) });
        hk::ftCo_Turn_Enter = Some(|g, ms, fl, a, bb, c| { t(2); t(is_go(g)); t(ms as u32); t(fl); t(b(a)); t(b(bb)); t(b(c)) });
        hk::ftCo_8009C5A4 = Some(|g, ms| { t(3); t((g == go2()) as u32); t(ms as u32) });
        hk::ftAnim_IsFramesRemaining = Some(|g| { t(4); t(is_go(g)); sr(3) });
        hk::ftCo_8009B518 = Some(|g| { t(5); t(is_go(g)) });
        hk::ftCo_8009BF3C = Some(|g| { t(6); t(is_go(g)); sr(5) });
        hk::ftCo_8009BB1C = Some(|g| { t(7); t(is_go(g)); sr(6) });
        hk::ft_80084F3C = Some(|g| { t(8); t(is_go(g)) });
        hk::ftCo_CargoWait_Coll = Some(|g| { t(9); t(is_go(g)) });
    }
    let mut c = Check::new(include_str!("data/tu_ftCo_CargoTurn.txt"));
    for op in 0..6u32 {
        for k in 0..2000u32 {
            case(op, k);
            *dk = ftDonkeyAttributes::zeroed();
            dk.x4_motion_state = (rnd() % 400) as i32;
            dk.cargo_hold.x20_TURN_SPEED = rg();
            let fr = f();
            fr.x2CC = dk as *mut _;
            fr.victim_gobj = go2();
            unsafe {
                fr.mv.co.cargoturn.x0 = (rnd() & 1) as i32;
                fr.mv.co.walk.fast_anim_frame = rg() * 2.0;
            }
            fr.facing_dir = rg();
            f2().facing_dir = rg();
            let g = go();
            let ret = match op {
                0 => m::ftCo_8009B818(g) as u32,
                1 => { m::ftCo_8009B860(g); 0 }
                2 => { m::ftCo_CargoTurn_Anim(g); 0 }
                3 => { m::ftCo_CargoTurn_IASA(g); 0 }
                4 => { m::ftCo_CargoTurn_Phys(g); 0 }
                _ => { m::ftCo_CargoTurn_Coll(g); 0 }
            };
            let mut h = Hs::new();
            unsafe {
                h.m(b(f().facing_dir)); h.m(b(f2().facing_dir));
                h.m(f().mv.co.cargoturn.x0 as u32); h.m(b(f().mv.co.walk.fast_anim_frame));
            }
            c.line(op, k, ret, h.0);
        }
    }
    assert_eq!(c.done(), 12000);
}
