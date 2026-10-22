//! tu::ftpurinspeciallw smoke test: Rest Enter/Anim/Coll motion-state choice by facing direction.
#![allow(static_mut_refs, unused_unsafe)]
mod gw_common;
use gw_common::*;
use melee_chara::tu::ftpurinspeciallw as t;
use melee_types::*;

#[test]
fn rest() {
    install_common();
    let mut w = World::new();
    let gp = w.gp();
    w.f.facing_dir = -1.0;
    w.f.cmd_vars[0] = 5;
    t::ftPr_SpecialLw_Enter(gp);
    assert_eq!(take(), format!("ms {} 0 0,anim", ftPr_MS_SpecialLwL));
    assert_eq!(w.f.cmd_vars[0], 0);
    assert!(w.f.accessory4_cb.is_some());
    w.f.facing_dir = 1.0;
    t::ftPr_SpecialAirLw_Enter(gp);
    assert_eq!(take(), format!("ms {} 0 0,anim", ftPr_MS_SpecialAirLwR));

    unsafe { FRAMES_REMAINING = 0 };
    t::ftPr_SpecialLw_Anim(gp);
    assert_eq!(take(), "wait");
    t::ftPr_SpecialAirLw_Anim(gp);
    assert_eq!(take(), "fall");
    unsafe { FRAMES_REMAINING = 1 };
    t::ftPr_SpecialLw_Anim(gp);
    assert_eq!(take(), "");

    // ft_800827A0 stand-in returns 0 (left the ground): switch to the air state.
    t::ftPr_SpecialLw_Coll(gp);
    assert!(take().contains(&format!("ms {}", ftPr_MS_SpecialAirLwR)));
    // ft_80081D0C returns 1 (== GA_Ground? only a non-ground result lands).
    t::ftPr_SpecialAirLw_Coll(gp);
    let _ = take();
}
