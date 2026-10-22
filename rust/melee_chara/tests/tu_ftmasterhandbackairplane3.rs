//! Smoke test of tu::ftmasterhandbackairplane3 (BackPunch countdown checked exactly).
#![allow(static_mut_refs)]
#[path = "common_mh/mod.rs"]
mod common;
use common::*;
use melee_chara::tu::ftmasterhandbackairplane3 as t;

#[test]
fn smoke() {
    let mut r = Rig::new();
    let gp = r.gp();
    hook_stubs(1);
    unsafe { t::ext::FTMH_MS_389_80151018 = Some(|_| log("389".into())) };
    // Anim waits while frames remain, then hands off.
    t::ftMh_BackAirplane3_Anim(gp);
    t::ftMh_BackPunch_Anim(gp);
    assert_eq!(take_log(), "");
    hook_stubs(0);
    r.f.self_vel.x = 4.0;
    t::ftMh_BackAirplane3_Anim(gp);
    assert_eq!(take_log(), "389");
    t::ftMh_BackPunch_Anim(gp);
    assert_eq!(take_log(), "389");
    assert_eq!(r.f.self_vel.x, 0.0);
    // Phys: --x0 > 0 chases (ftBossLib_8015BF74, needs a fighter list: skip), else stops x.
    unsafe { r.f.mv.mh.unk0.x0 = 1.0 };
    r.f.self_vel.x = 7.0;
    t::ftMh_BackPunch_Phys(gp);
    assert_eq!(unsafe { r.f.mv.mh.unk0.x0 }, 0.0);
    assert_eq!(r.f.self_vel.x, 0.0);
    unsafe { r.f.mv.mh.unk0.x0 = -3.5 };
    r.f.self_vel.x = 7.0;
    t::ftMh_BackPunch_Phys(gp);
    assert_eq!(unsafe { r.f.mv.mh.unk0.x0 }, -4.5);
    assert_eq!(r.f.self_vel.x, 0.0);
    t::ftMh_BackAirplane3_Coll(gp);
    let _: fn(*mut melee_types::HSD_GObj) = t::ftMh_MS_369_80153B90;
}
