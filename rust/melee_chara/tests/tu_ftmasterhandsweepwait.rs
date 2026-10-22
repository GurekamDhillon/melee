//! Smoke test of tu::ftmasterhandsweepwait.
#![allow(static_mut_refs)]
mod mh_common;
use mh_common::*;
use melee_chara::tu::ftmasterhandsweepwait as t;

#[test]
fn smoke() {
    let mut r = Rig::new();
    let gp = r.gp();
    t::ftMh_SweepWait_Coll(gp);
    t::ftMh_SweepWait_Anim(gp);
    assert_eq!(take(), "");
    unsafe { FRAMES = 0 };
    t::ftMh_SweepWait_Anim(gp);
    assert_eq!(take(), "389");
    t::ftMh_SweepWait_Phys(gp);
    t::ftMh_MS_348_80151BB8(gp);
    assert_eq!(take(), "85134,cms 349 0 0 1 0,8006EBA4");
}
