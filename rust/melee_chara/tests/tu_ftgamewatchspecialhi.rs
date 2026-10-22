//! tu::ftgamewatchspecialhi smoke test: Fire Rescue item lifecycle, the landing hand-off in
//! Anim, and the aerial Coll branches (the fused spawn height and the aim math are compared
//! with the C in tests/tu_ftgamewatch.rs). One #[test]: hooks are globals.
#![allow(static_mut_refs, unused_unsafe)]
mod gw_common;
use gw_common::*;
use melee_chara::stubs as st;
use melee_chara::tu::ftgamewatchspecialhi as t;
use melee_types::*;

#[test]
fn fire_rescue() {
    install_common();
    unsafe {
        t::ext::IT_802C8038 = Some(|_, _, part, kind, facing, scale| {
            log(format!("spawn part {part} kind {kind} facing {facing} scale {scale}"));
            fake(0x500)
        });
        t::ext::IT_802C8158 = Some(|i| log(format!("destroy {:x}", i as usize)));
        t::ext::IT_802C81C8 = Some(|i| log(format!("enter_hitlag {:x}", i as usize)));
        t::ext::IT_802C81E8 = Some(|i| log(format!("exit_hitlag {:x}", i as usize)));
        melee_chara::tu::ftgamewatch::ext::FTCOMMON_GETMODELSCALE = Some(|_| 2.0);
        st::FTCO_80096900 = Some(|_, a, b, c, m, lag| log(format!("fallspecial {a} {b} {c} {m} {lag}")));
        st::FTCO_LANDINGFALLSPECIAL_ENTER = Some(|_, ai, lag| log(format!("landing {ai} {lag}")));
        st::FT_CHECKGROUNDANDLEDGE = Some(|_, dir| {
            log(format!("ledge {dir}"));
            0
        });
        st::FTCLIFFCOMMON_80081298 = Some(|_| 0);
    }
    let mut w = World::new();
    let gp = w.gp();
    w.f.facing_dir = 1.0;
    w.f.motion_id = ftGw_MS_SpecialAirHi as FtMotionId;
    w.da.x60_GAMEWATCH_RESCUE_LANDING = 12.0;

    // Setup: the item is spawned with the motion-relative kind and scale 2.5.
    t::ftGw_SpecialHi_ItemRescueSetup(gp);
    assert_eq!(take(), format!("spawn part {FtPart_TopN} kind 1 facing 1 scale 2.5"));
    assert_eq!(unsafe { w.f.u.gw.x226C_rescueGObj }, fake(0x500));
    assert!(w.f.death2_cb.is_some() && w.f.take_dmg_cb.is_some() && w.f.pre_hitlag_cb.is_some());
    // already out: a second Setup does nothing.
    t::ftGw_SpecialHi_ItemRescueSetup(gp);
    assert_eq!(take(), "");

    t::ftGw_SpecialHi_ItemRescueEnterHitlag(gp);
    t::ftGw_SpecialHi_ItemRescueExitHitlag(gp);
    assert_eq!(take(), "enter_hitlag 500,exit_hitlag 500");

    // Which motions keep the item.
    for (ms, remove) in [(ftGw_MS_SpecialHi, false), (ftGw_MS_SpecialAirHi, false), (ftGw_MS_SpecialLw, true)] {
        w.f.motion_id = ms as FtMotionId;
        assert_eq!(t::ftGw_SpecialHi_ItemCheckRescueRemove(gp), remove);
    }

    // Anim: nothing while frames remain; landing lag 0 -> Fall; else FallSpecial with the lag.
    unsafe { FRAMES_REMAINING = 1 };
    t::ftGw_SpecialAirHi_Anim(gp);
    assert_eq!(take(), "");
    unsafe { FRAMES_REMAINING = 0 };
    t::ftGw_SpecialAirHi_Anim(gp);
    w.da.x60_GAMEWATCH_RESCUE_LANDING = 0.0;
    t::ftGw_SpecialHi_Anim(gp);
    assert_eq!(take(), "fallspecial 1 0 1 1 12,fall");

    // Coll: before frame 4 nothing; rising + landing -> remove item + landing; falling -> ledge
    // check with the facing direction.
    w.da.x60_GAMEWATCH_RESCUE_LANDING = 9.0;
    w.f.cur_anim_frame = 4.0;
    t::ftGw_SpecialAirHi_Coll(gp);
    assert_eq!(take(), "");
    w.f.cur_anim_frame = 5.0;
    w.f.self_vel.y = 1.0;
    t::ftGw_SpecialAirHi_Coll(gp);
    assert_eq!(take(), "destroy 500,7D7FC,landing 0 9");
    assert!(unsafe { w.f.u.gw.x226C_rescueGObj }.is_null());
    assert!(w.f.death2_cb.is_none() && w.f.take_dmg_cb.is_none());
    w.f.self_vel.y = -1.0;
    w.f.facing_dir = -1.0;
    t::ftGw_SpecialHi_Coll(gp);
    assert_eq!(take(), "ledge -1");
}
