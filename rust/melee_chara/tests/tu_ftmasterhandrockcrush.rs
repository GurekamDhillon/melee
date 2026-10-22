//! Smoke test of tu::ftmasterhandrockcrush (countdown in x8, state hand-offs).
#![allow(static_mut_refs)]
mod mh_common;
use mh_common::*;
use melee_chara::tu::ftmasterhandrockcrush as t;

#[test]
fn smoke() {
    let mut r = Rig::new();
    let gp = r.gp();
    t::ftMh_Drill_Coll(gp);
    t::ftMh_RockCrushUp_Coll(gp);
    t::ftMh_RockCrushWait_Coll(gp);
    // Up: frames remain -> nothing; done -> wait state and countdown loaded from xA0
    t::ftMh_RockCrushUp_Anim(gp);
    assert_eq!(take(), "");
    unsafe { FRAMES = 0 };
    r.da.xA0 = 3;
    t::ftMh_RockCrushUp_Anim(gp);
    assert_eq!(take(), "cms 356 0 0 1 0,8006EBA4");
    assert_eq!(unsafe { r.f.mv.mh.unk0.x8 }, 3);
    // Wait: 3 -> 2 -> 1 -> 0 (then the down state); every frame-end also restarts the wait anim
    t::ftMh_RockCrushWait_Anim(gp);
    t::ftMh_RockCrushWait_Anim(gp);
    assert_eq!(unsafe { r.f.mv.mh.unk0.x8 }, 1);
    take();
    r.f.self_vel.x = 4.0;
    t::ftMh_RockCrushWait_Anim(gp);
    assert_eq!(take(), "cms 357 0 0 1 0,8006EBA4,cms 356 0 0 1 0,8006EBA4");
    assert_eq!(r.f.self_vel.x, 0.0);
    unsafe { FRAMES = 1 };
    r.f.mv.mh.unk0.x8 = 5;
    t::ftMh_RockCrushWait_Anim(gp);
    assert_eq!(take(), "");
    assert_eq!(unsafe { r.f.mv.mh.unk0.x8 }, 4);
    t::ftMh_RockCrushUp_Phys(gp);
    assert_eq!(take(), "85134");
}
