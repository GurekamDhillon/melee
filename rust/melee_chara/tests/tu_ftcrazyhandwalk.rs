//! Smoke test of tu::ftcrazyhandwalk.
#![allow(static_mut_refs)]
#[path = "common_ch/mod.rs"]
mod common;
use common::*;
use melee_chara::tu::ftcrazyhandwalk as t;
use melee_types::*;

#[test]
fn smoke() {
    let mut r = Rig::new();
    let gp = r.gp();
    hook_stubs(1);
    unsafe {
        t::ext::FTCH_GRABUNK1_8015BC88 = Some(|_| log("bc88".into()));
        melee_chara::stubs::FT_80085134 = Some(|_| log("ft85134".into()));
    }
    // WalkWait_Anim: countdown 2 -> 1, frames remain: nothing.
    unsafe { r.f.mv.ch.unk0.x8 = 2 };
    t::ftCh_WalkWait_Anim(gp);
    assert_eq!(take_log(), "");
    assert_eq!(unsafe { r.f.mv.ch.unk0.x8 }, 1);
    // countdown hits 0 -> WalkShoot start (0x161 = ftMh_MS_WalkShoot 353), then the end-of-anim branch.
    hook_stubs(0);
    t::ftCh_WalkWait_Anim(gp);
    assert_eq!(unsafe { r.f.mv.ch.unk0.x8 }, 0);
    assert_eq!(take_log(), "cms 0x161 0 0 1 0 true,anim8006EBA4,cms 0x160 0 0 1 0 true,anim8006EBA4");
    assert_eq!(r.f.self_vel.x, 0.0);

    // WalkShoot_Anim
    hook_stubs(1);
    t::ftCh_WalkShoot_Anim(gp);
    assert_eq!(take_log(), "");
    hook_stubs(0);
    t::ftCh_WalkShoot_Anim(gp);
    assert_eq!(take_log(), "bc88");
    t::ftCh_WalkLoop_Coll(gp);
    t::ftCh_WalkWait_Coll(gp);
    t::ftCh_WalkShoot_Coll(gp);
    assert_eq!(t::ftCh_Init_80157B58 as usize != 0, true);
}
