//! tu::ftgamewatchspecialn smoke test: Enter / Anim loop / IASA loop / state changes of the Chef
//! (the sausage pick and the IASA decisions are compared with the C in tests/tu_ftgamewatch.rs).
//! One #[test]: hooks are globals.
#![allow(static_mut_refs, unused_unsafe)]
mod gw_common;
use gw_common::*;
use melee_chara::tu::ftgamewatchspecialn as t;
use melee_types::*;

const FLAGS: u32 = (Ft_MF_KeepColAnimHitStatus
    | Ft_MF_SkipHit
    | Ft_MF_SkipModel
    | Ft_MF_SkipMatAnim
    | Ft_MF_SkipColAnim
    | Ft_MF_UpdateCmd
    | Ft_MF_SkipItemVis
    | Ft_MF_Unk19
    | Ft_MF_SkipModelPartVis
    | Ft_MF_SkipModelFlags
    | Ft_MF_Unk27) as u32;

#[test]
fn chef() {
    install_common();
    let mut w = World::new();
    let gp = w.gp();
    w.da.x1C_GAMEWATCH_CHEF_MAX = 4.0;
    w.da.x18_GAMEWATCH_CHEF_LOOPFRAME = 20.0;

    // Enter: velocity, motion, flags cleared, sausage callback installed.
    w.f.self_vel.y = 5.0;
    w.f.cmd_vars = [1, 1, 1, 0];
    t::ftGw_SpecialN_Enter(gp);
    assert_eq!(take(), format!("ms {} 0 0,anim", ftGw_MS_SpecialN));
    assert_eq!((w.f.self_vel.y, w.f.cmd_vars), (0.0, [0, 0, 0, 0]));
    assert!(w.f.accessory4_cb.is_some());
    assert_eq!(unsafe { (w.f.mv.gw.SpecialN.isChefLoopDisable, w.f.mv.gw.SpecialN.maxSausage) }, (0, 0));
    t::ftGw_SpecialAirN_Enter(gp);
    assert_eq!(take(), format!("ms {} 0 0,anim", ftGw_MS_SpecialAirN));

    // Anim: a loop request below the sausage cap restarts the loop one frame early.
    unsafe { FRAMES_REMAINING = 1 };
    w.f.cmd_vars[2] = 1;
    t::ftGw_SpecialN_Anim(gp);
    assert_eq!(take(), format!("ms {} {:x} 19,anim", ftGw_MS_SpecialN, FLAGS));
    assert_eq!(w.f.cmd_vars[2], 0);
    // disabled loop: the request is consumed but nothing restarts; finished animation -> wait.
    unsafe { w.f.mv.gw.SpecialN.isChefLoopDisable = 1 };
    w.f.cmd_vars[2] = 1;
    unsafe { FRAMES_REMAINING = 0 };
    t::ftGw_SpecialN_Anim(gp);
    assert_eq!(take(), "wait");
    w.f.cmd_vars[2] = 1;
    t::ftGw_SpecialAirN_Anim(gp);
    assert_eq!(take(), "fall");
    // at the cap: no restart.
    unsafe {
        w.f.mv.gw.SpecialN.isChefLoopDisable = 0;
        w.f.mv.gw.SpecialN.maxSausage = 4;
    }
    w.f.cmd_vars[2] = 1;
    unsafe { FRAMES_REMAINING = 1 };
    t::ftGw_SpecialAirN_Anim(gp);
    assert_eq!(take(), "");

    // IASA: releasing B disables the loop; a fresh B press with cmd_vars[1] restarts it.
    unsafe { w.f.mv.gw.SpecialN.maxSausage = 1 };
    w.f.input.held_buttons[0] = 0;
    w.f.input.pressed_buttons = HSD_PAD_B as u32;
    w.f.cmd_vars[1] = 1;
    t::ftGw_SpecialAirN_IASA(gp);
    assert_eq!(take(), format!("ms {} {:x} 19,anim", ftGw_MS_SpecialAirN, FLAGS));
    assert_eq!(unsafe { w.f.mv.gw.SpecialN.isChefLoopDisable }, 0, "Loop re-enables the loop flag");
    w.f.cmd_vars[1] = 0;
    t::ftGw_SpecialN_IASA(gp);
    assert_eq!(take(), "");
    assert_eq!(unsafe { w.f.mv.gw.SpecialN.isChefLoopDisable }, 1);

    // Collision hand-offs: the sausage callback is re-armed.
    w.f.accessory4_cb = None;
    t::ftGw_SpecialN_Coll(gp); // ft_800827A0 stand-in returns 0: leaves the ground
    let l = take();
    assert!(l.contains(&format!("ms {} {:x}", ftGw_MS_SpecialAirN, FLAGS)), "{l}");
    assert!(w.f.accessory4_cb.is_some());
    w.f.accessory4_cb = None;
    t::ftGw_SpecialAirN_Coll(gp); // ft_80081D0C stand-in returns 1: lands
    let l = take();
    assert!(l.contains(&format!("ms {} {:x}", ftGw_MS_SpecialN, FLAGS)), "{l}");
    assert!(w.f.accessory4_cb.is_some());
}
