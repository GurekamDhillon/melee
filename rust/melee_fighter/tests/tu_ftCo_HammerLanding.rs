//! src/tu/ftCo_HammerLanding.rs vs the C (gen/cref/tu/ftCo_HammerLanding.c: the whole TU,
//! shipping contraction; regenerate with
//! `python rust/melee_fighter/gen/cref/tu/run_tu.py ftCo_HammerLanding`).
#![allow(static_mut_refs, non_snake_case)]
mod tu_ftco_harness;
use tu_ftco_harness::*;
use melee_fighter::tu::ftCo_HammerLanding as m;
use m::ext::hook as hk;

static mut FR: f32 = 0.0;
static mut FL: u32 = 0;
fn isf(p: &mut melee_fighter::types::Fighter) -> u32 { (p as *mut _ == unsafe { F }) as u32 }

#[test]
fn ftCo_HammerLanding_matches_c() {
    init();
    unsafe {
        hk::ftCo_800C54C4 = Some(|p| { t(2); t(isf(p)); FL });
        hk::ftCo_800C548C = Some(|p| { t(3); t(isf(p)); FR });
        hk::ftCo_800C4E94 = Some(|p| { t(4); t(isf(p)) });
        hk::ftCo_800C4F64 = Some(|g| { t(5); t(is_go(g)) });
        hk::ftCo_Landing_Enter = Some(|g, ms, ai, fl, a, bb| { t(6); t(is_go(g)); t(ms as u32); t(ai as u32); t(fl); t(b(a)); t(b(bb)) });
        hk::ftCo_800C4ED8 = Some(|g| { t(7); t(is_go(g)) });
        hk::ft_80084F3C = Some(|g| { t(8); t(is_go(g)) });
        hk::ftCo_HammerWait_Coll = Some(|g| { t(9); t(is_go(g)) });
    }
    let mut c = Check::new(include_str!("data/tu_ftCo_HammerLanding.txt"));
    for op in 0..5u32 {
        for k in 0..2000u32 {
            case(op, k);
            unsafe { FR = rw(); FL = rnd(); }
            unsafe { f().mv.co.hammerlanding.x4 = rg() * 3.0; }
            cd().x6B4 = (rnd() % 64) as i32 - 8;
            let g = go();
            match op {
                0 => m::ftCo_HammerLanding_Enter(g),
                1 => m::ftCo_HammerLanding_Anim(g),
                2 => m::ftCo_HammerLanding_IASA(g),
                3 => m::ftCo_HammerLanding_Phys(g),
                _ => m::ftCo_HammerLanding_Coll(g),
            }
            let mut h = Hs::new();
            h.m(b(unsafe { f().mv.co.hammerlanding.x4 }));
            c.line(op, k, 0, h.0);
        }
    }
    assert_eq!(c.done(), 10000);
}
