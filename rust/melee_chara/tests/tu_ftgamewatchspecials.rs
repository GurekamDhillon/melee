//! tu::ftgamewatchspecials smoke test: Judgement Enter (roll -> motion), Judgement item hitlag /
//! removal, the Phys cmd_vars transitions and the state changes. The roll and the Setup are
//! compared with the C in tests/tu_ftgamewatch.rs. One #[test]: hooks are globals.
#![allow(static_mut_refs, unused_unsafe)]
mod gw_common;
use gw_common::*;
use melee_chara::tu::ftgamewatchspecials as t;
use melee_types::*;

#[test]
fn judgement() {
    install_common();
    unsafe {
        t::ext::HSD_RANDI = Some(|max| max - 1);
        t::ext::IT_802C7A84 = Some(|i| log(format!("destroy {:x}", i as usize)));
        t::ext::IT_802C7AD0 = Some(|i| log(format!("enter_hitlag {:x}", i as usize)));
        t::ext::IT_802C7AF0 = Some(|i| log(format!("exit_hitlag {:x}", i as usize)));
        melee_chara::tu::ftgamewatch::ext::FTPARTS_GETBONEINDEX = Some(|_, p| p);
    }
    let mut w = World::new();
    let gp = w.gp();
    w.da.x34_GAMEWATCH_JUDGE_ROLL = [1; 9];
    w.da.x20_GAMEWATCH_JUDGE_MOMENTUM_PRESERVE = 2.0;

    // Roll: the highest remaining number (Randi stand-in = max-1), never the last two.
    unsafe {
        w.f.u.gw.x222C_judgeVar1 = 1;
        w.f.u.gw.x2230_judgeVar2 = 0;
    }
    assert_eq!(t::ftGw_SpecialS_GetRandomInt(gp), 8);
    assert_eq!(unsafe { (w.f.u.gw.x222C_judgeVar1, w.f.u.gw.x2230_judgeVar2) }, (8, 1));
    assert_eq!(t::ftGw_SpecialS_GetRandomInt(gp), 7);
    assert_eq!(unsafe { (w.f.u.gw.x222C_judgeVar1, w.f.u.gw.x2230_judgeVar2) }, (7, 8));

    // Enter rolls again (excluding the last two: 7, 8 -> 6; then 6, 7 -> 8).
    // Enter: the motion is SpecialS1 + roll; the grounded one zeroes y velocity, the aerial one
    // divides x velocity by the preserve factor.
    w.f.self_vel = Vec { x: 6.0, y: 3.0, z: 0.0 };
    t::ftGw_SpecialS_Enter(gp);
    let l = take();
    assert!(l.starts_with(&format!("ms {} 0 0,anim", ftGw_MS_SpecialS1 as i32 + 6)), "{l}");
    assert_eq!(w.f.self_vel.y, 0.0);
    assert!(w.f.accessory4_cb.is_some());
    t::ftGw_SpecialAirS_Enter(gp);
    let l = take();
    assert!(l.starts_with(&format!("ms {} 0 0,anim", ftGw_MS_SpecialAirS1 as i32 + 8)), "{l}");
    assert_eq!(w.f.self_vel.x, 3.0);

    // Phys (aerial): first frame of cmd_vars[0] == 1 launches once, then zeroes y velocity.
    w.da.x28_GAMEWATCH_JUDGE_VEL_Y = 4.5;
    w.f.cmd_vars[0] = 1;
    unsafe { w.f.u.gw.x2234 = 0 };
    t::ftGw_SpecialS_Phys(gp);
    assert_eq!(w.f.cmd_vars[0], 2);

    // Item hitlag / removal.
    unsafe { w.f.u.gw.x2264_judgementGObj = fake(0x600) };
    t::ftGw_SpecialS_ItemJudgementEnterHitlag(gp);
    t::ftGw_SpecialS_ItemJudgementExitHitlag(gp);
    t::ftGw_SpecialS_ItemJudgementRemove(gp);
    assert_eq!(take(), "enter_hitlag 600,exit_hitlag 600,destroy 600,exit_hitlag 600");
    assert!(unsafe { w.f.u.gw.x2264_judgementGObj }.is_null());
    assert!(w.f.death2_cb.is_none() && w.f.take_dmg_cb.is_none());

    // Coll: leaving the ground / landing switches to the other state set and resets x2234.
    t::ftGw_SpecialS_Coll(gp);
    let l = take();
    assert!(l.contains(&format!("ms {} ", ftGw_MS_SpecialAirS1 as i32 + 8)), "{l}");
    unsafe { w.f.u.gw.x2234 = 1 };
    t::ftGw_SpecialAirS_Coll(gp);
    let l = take();
    assert!(l.contains(&format!("ms {} ", ftGw_MS_SpecialS1 as i32 + 8)), "{l}");
    assert_eq!(unsafe { w.f.u.gw.x2234 }, 0);
}
