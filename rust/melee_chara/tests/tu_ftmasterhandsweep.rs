//! Smoke test of tu::ftmasterhandsweep (damage drift adds, sweep target, floor-left test).
#![allow(static_mut_refs)]
mod mh_common;
use mh_common::*;
use melee_chara::tu::ftmasterhandsweep as t;

#[test]
fn smoke() {
    let mut r = Rig::new();
    let gp = r.gp();
    t::ftMh_Damage_Coll(gp);
    t::ftMh_SweepLoop_Coll(gp);
    t::ftMh_WaitSweep_Coll(gp);
    r.da.x150 = 0.25;
    r.da.x158 = -0.5;
    r.f.self_vel.y = 1.0;
    r.f.self_vel.z = 1.0;
    t::ftMh_Damage_Phys(gp);
    assert_eq!((r.f.self_vel.y, r.f.self_vel.z), (1.25, 0.5));
    t::ftMh_MS_344_80151828(gp);
    assert_eq!(take(), "cms 346 0 0 1 0,8006EBA4");
    t::ftMh_WaitSweep_Phys(gp);
    take();
    // WaitSweep -> SweepLoop sets the chase target from cur_pos.x - x3C and x38
    r.f.cur_pos.x = 10.0;
    r.da.x3C = 4.0;
    r.da.x38 = 9.0;
    unsafe { FRAMES = 0 };
    t::ftMh_WaitSweep_Anim(gp);
    assert_eq!(take(), "cms 347 0 0 1 0,8006EBA4");
    let xc = unsafe { r.f.mv.mh.unk0.xC };
    assert_eq!((xc.x, xc.y, xc.z), (6.0, 9.0, 0.0));
    // SweepLoop_Anim: left of the floor edge -> SweepWait, else loop again
    unsafe { t::ext::MPFLOORGETLEFT = Some(|_, o| unsafe { (*o).x = 20.0 }) };
    t::ftMh_SweepLoop_Anim(gp);
    assert_eq!(take(), "cms 348 0 0 1 0,8006EBA4");
    unsafe { t::ext::MPFLOORGETLEFT = Some(|_, o| unsafe { (*o).x = -20.0 }) };
    t::ftMh_SweepLoop_Anim(gp);
    assert_eq!(take(), "cms 347 0 0 1 0,8006EBA4");
    unsafe { FRAMES = 1 };
    t::ftMh_SweepLoop_Anim(gp);
    assert_eq!(take(), "");
}
