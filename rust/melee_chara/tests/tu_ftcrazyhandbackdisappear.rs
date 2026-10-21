//! Smoke test of tu::ftcrazyhandbackdisappear.
#![allow(static_mut_refs)]
#[path = "common_ch/mod.rs"]
mod common;
use common::*;
use melee_chara::tu::ftcrazyhandbackairplane2::ext;
use melee_chara::tu::ftcrazyhandbackdisappear as t;
use melee_types::*;

#[test]
fn smoke() {
    let mut r = Rig::new();
    let gp = r.gp();
    hook_stubs(1);
    unsafe { ext::FTCH_GRABUNK1_8015BC88 = Some(|_| log("grabBC88".into())) };
    r.ext.xF0 = 3;
    r.ext.xEC = 4.5;
    r.f.self_vel = Vec3 { x: 1.0, y: 2.0, z: 3.0 };
    // Needs a second fighter for ftBossLib_8015C208 -> ftlib; keep to the pieces that do not.
    t::ftCh_BackDisappear_Anim(gp);
    assert_eq!(take_log(), "");
    hook_stubs(0);
    r.f.self_vel.x = 5.0;
    t::ftCh_BackDisappear_Anim(gp);
    assert_eq!(take_log(), "grabBC88");
    assert_eq!(r.f.self_vel.x, 0.0);

    // Phys: counts down; at <= 0 it stops x velocity.
    unsafe { r.f.mv.ch.unk0.x0 = 1.0 };
    r.f.self_vel.x = 5.0;
    t::ftCh_BackDisappear_Phys(gp);
    assert_eq!(unsafe { r.f.mv.ch.unk0.x0 }, 0.0);
    assert_eq!(r.f.self_vel.x, 0.0);
    t::ftCh_BackDisappear_Coll(gp);
}
