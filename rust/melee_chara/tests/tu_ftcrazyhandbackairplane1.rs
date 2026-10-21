//! Smoke test of tu::ftcrazyhandbackairplane1 (no float math to check against the C).
#![allow(static_mut_refs)]
#[path = "common_ch/mod.rs"]
mod common;
use common::*;
use melee_chara::tu::ftcrazyhandbackairplane1 as t;
use melee_types::*;
use melee_chara::tu::ftcrazyhandbackairplane1::ext;

#[test]
fn smoke() {
    unsafe { body() }
}
#[allow(unused_unsafe)]
unsafe fn body() {
    let mut r = Rig::new();
    let gp = r.gp();
    hook_stubs(1);
    unsafe {
        ext::FTCH_GRABUNK1_8015B670 = Some(|_| log("grabunk".into()));
    }
    r.f.ft_data = &mut *r.fd;
    r.ext.xCC_pos.y = 2.5;
    r.ext.xD4 = -7.0;
    r.f.mv.ch.unk0.xC.z = 9.0;
    t::ftCh_Init_801592D4(gp);
    assert_eq!(take_log(), "cms 0x16f 0 0 1 0 true,anim8006EBA4,grabunk");
    unsafe {
        assert_eq!(r.f.mv.ch.unk0.xC.x, 2.5);
        assert_eq!(r.f.mv.ch.unk0.xC.y, -7.0);
        assert_eq!(r.f.mv.ch.unk0.xC.z, 0.0);
    }
    assert_eq!(r.f.x2222_b2(), 1);
    assert_eq!(r.f.x1A6A, 511);

    // Anim: frames remaining -> only the cmd_vars sfx branch (needs HSD_Randi; skip it with 0).
    t::ftCh_BackAirplane1_Anim(gp);
    assert_eq!(take_log(), "");
    // Anim: done, x2250 selects the next state.
    hook_stubs(0);
    r.f.u.mh.x2250 = 0x170;
    t::ftCh_BackAirplane1_Anim(gp);
    assert_eq!(take_log(), "cms 0x170 0 0 1 0 true,anim8006EBA4");
    r.f.u.mh.x2250 = 0x171;
    t::ftCh_BackAirplane1_Anim(gp);
    assert_eq!(take_log(), "cms 0x171 0 0 1 0 true,anim8006EBA4");

    // Phys: ft_80085134, then the native ftBossLib_8015BE40 (target == cur_pos -> distance 0 < x14).
    r.ext.x14 = 1.0;
    r.ext.x10 = 0.5;
    r.f.cur_pos = Vec3 { x: 1.0, y: 2.0, z: 3.0 };
    r.f.mv.ch.unk0.xC = r.f.cur_pos;
    r.f.mv.ch.unk0.x18 = 55.0;
    t::ftCh_BackAirplane1_Phys(gp);
    assert_eq!(take_log(), "ft85134");
    assert_eq!(r.f.mv.ch.unk0.x18, 0.0);

    // Coll clears velocity only when x18 == 0.
    r.f.self_vel = Vec3 { x: 1.0, y: 2.0, z: 3.0 };
    r.f.mv.ch.unk0.x18 = 1.0;
    t::ftCh_BackAirplane1_Coll(gp);
    assert_eq!(r.f.self_vel.y, 2.0);
    r.f.mv.ch.unk0.x18 = 0.0;
    t::ftCh_BackAirplane1_Coll(gp);
    assert_eq!((r.f.self_vel.x, r.f.self_vel.y, r.f.self_vel.z), (0.0, 0.0, 0.0));
}
