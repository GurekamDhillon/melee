//! src/tu/ftCo_HammerKneeBend.rs vs the C (gen/cref/tu/ftCo_HammerKneeBend.c: the whole TU,
//! shipping contraction; regenerate with
//! `python rust/melee_fighter/gen/cref/tu/run_tu.py ftCo_HammerKneeBend`).
#![allow(static_mut_refs, non_snake_case)]
mod tu_ftco_harness;
use tu_ftco_harness::*;
use melee_fighter::tu::ftCo_HammerKneeBend as m;
use m::ext::hook as hk;

static mut FR: f32 = 0.0;
static mut FL: u32 = 0;
fn isf(p: &mut melee_fighter::types::Fighter) -> u32 { (p as *mut _ == unsafe { F }) as u32 }

#[test]
fn ftCo_HammerKneeBend_matches_c() {
    init();
    unsafe {
        hk::Fighter_ChangeMotionState = Some(|g, ms, fl, a, bb, c, o| { t(1); t(is_go(g)); t(ms as u32); t(fl); t(b(a)); t(b(bb)); t(b(c)); t(o.is_null() as u32) });
        hk::ftCo_800C54C4 = Some(|p| { t(2); t(isf(p)); FL });
        hk::ftCo_800C548C = Some(|p| { t(3); t(isf(p)); FR });
        hk::ftCo_800C4E94 = Some(|p| { t(4); t(isf(p)) });
        hk::ftCo_800C4F64 = Some(|g| { t(5); t(is_go(g)) });
        hk::ftCo_800C5A98 = Some(|g| { t(6); t(is_go(g)) });
        hk::ftCo_KneeBend_Check_ShortHop = Some(|g| { t(7); t(is_go(g)) });
        hk::ft_80084F3C = Some(|g| { t(8); t(is_go(g)) });
        hk::ftCo_HammerWait_Coll = Some(|g| { t(9); t(is_go(g)) });
    }
    let mut c = Check::new(include_str!("data/tu_ftCo_HammerKneeBend.txt"));
    for op in 0..5u32 {
        for k in 0..2000u32 {
            case(op, k);
            unsafe { FR = rw(); FL = rnd(); }
            let fr = f();
            unsafe {
                fr.mv.co.hammerkneebend.x0 = rnd() as i32; fr.mv.co.hammerkneebend.x4 = rnd() as i32;
                fr.mv.co.hammerkneebend.x8 = rg() * 3.0;
            }
            fr.co_attrs.jump_startup_time = rw();
            let g = go();
            match op {
                0 => m::ftCo_800C5B88(g, rnd() as i32),
                1 => m::ftCo_HammerKneeBend_Anim(g),
                2 => m::ftCo_HammerKneeBend_IASA(g),
                3 => m::ftCo_HammerKneeBend_Phys(g),
                _ => m::ftCo_HammerKneeBend_Coll(g),
            }
            let mut h = Hs::new();
            unsafe {
                let v = &f().mv.co.hammerkneebend;
                h.m(v.x0 as u32); h.m(v.x4 as u32); h.m(b(v.x8));
            }
            c.line(op, k, 0, h.0);
        }
    }
    assert_eq!(c.done(), 10000);
}
