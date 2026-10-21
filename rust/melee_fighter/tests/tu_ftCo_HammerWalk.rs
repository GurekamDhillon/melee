//! src/tu/ftCo_HammerWalk.rs vs the C (gen/cref/tu/ftCo_HammerWalk.c: the whole TU, shipping
//! contraction; regenerate with `python rust/melee_fighter/gen/cref/tu/run_tu.py ftCo_HammerWalk`).
#![allow(static_mut_refs, non_snake_case)]
mod tu_ftco_harness;
use tu_ftco_harness::*;
use melee_fighter::tu::ftCo_HammerWalk as m;
use m::ext::hook as hk;
use melee_fighter::types::*;

fn isf(p: &Fighter) -> u32 { core::ptr::eq(p, f()) as u32 }
static mut XC: f32 = 0.0;

#[test]
fn ftCo_HammerWalk_matches_c() {
    init();
    unsafe {
        hk::ftWalkCommon_800DFC70 = Some(|g| { t(1); t(is_go(g)); sr(1) });
        hk::ftWalkCommon_800E0060 = Some(|g| { t(2); t(is_go(g)) });
        hk::ftCo_CalcYScaledKnockback = Some(|a, s, c| { t(3); t(b(a)); t(b(s)); t(b(c)); rw() });
        hk::Fighter_804D6524_xC = Some(|| XC);
        hk::Fighter_ChangeMotionState = Some(|g, ms, fl, a, bb, c, o| {
            t(4); t(is_go(g)); t(ms as u32); t(fl); t(b(a)); t(b(bb)); t(b(c)); t(o.is_null() as u32)
        });
        hk::ftAnim_8006EBA4 = Some(|g| { t(5); t(is_go(g)) });
        hk::ftCo_800C54C4 = Some(|fr| { t(6); t(isf(fr)); rnd() });
        hk::ftCo_800C548C = Some(|fr| { t(7); t(isf(fr)); rw() });
        hk::ftCo_800C4E94 = Some(|fr| { t(8); t(isf(fr)) });
        hk::ftCo_800C4F64 = Some(|g| { t(9); t(is_go(g)) });
        hk::ftCo_800C5A50 = Some(|_| { t(10); sr(2) });
        hk::ftCo_800C60C8 = Some(|_| { t(11); sr(3) });
        hk::ftCo_800C4E4C = Some(|_| { t(12); sr(4) });
        hk::ftCo_HammerWait_Coll = Some(|g| { t(13); t(is_go(g)) });
    }
    let mut c = Check::new(include_str!("data/tu_ftCo_HammerWalk.txt"));
    for op in 0..7u32 {
        for k in 0..2000u32 {
            case(op, k);
            f().x34_scale.y = if rnd() & 1 != 0 { 1.0 } else { rg() };
            f().gr_vel = rw();
            unsafe { XC = rg() };
            let g = go();
            let ret = match op {
                0 => m::ftCo_800C55CC(g) as u32,
                1 => { m::ftCo_800C5614(g); 0 }
                2 => { m::ftCo_HammerWalk_Anim(g); 0 }
                3 => { m::ftCo_HammerWalk_IASA(g); 0 }
                4 => { m::ftCo_HammerWalk_Phys(g); 0 }
                5 => { m::ftCo_HammerWalk_Coll(g); 0 }
                _ => { m::ftCo_800C5614(g); m::ftCo_800C5614(g); 0 }
            };
            let fr = f();
            let mut h = Hs::new();
            h.m(fr.motion_id as u32);
            unsafe { h.m(b(fr.mv.co.walk.accel_mul)); h.m(b(fr.mv.co.walk.x0)); }
            c.line(op, k, ret, h.0);
        }
    }
    assert_eq!(c.done(), 14000);
}
