//! Smoke test for tu::ftdonkeyheavywait0 (stateful; no fused sites) through recording hooks.
#![allow(static_mut_refs)]

use melee_chara::stubs as st;
use melee_chara::tu::{ftdonkeyheavyturn as turn, ftdonkeyheavywait0 as t, ftdonkeyheavywalk as walk, ftdonkeyms3450 as ms};
use melee_types::*;

static mut LOG: std::vec::Vec<String> = std::vec::Vec::new();
fn log(s: String) {
    unsafe { LOG.push(s) }
}
fn take() -> std::vec::Vec<String> {
    unsafe { core::mem::take(&mut LOG) }
}

#[test]
fn smoke() {
    let mut f: Box<Fighter> = Box::new(Fighter::zeroed());
    let mut g: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    let a: &'static mut ftDonkeyAttributes = Box::leak(Box::new(ftDonkeyAttributes::zeroed()));
    a.motion_state = 341;
    f.x2CC = a;
    g.user_data = &mut *f as *mut Fighter as *mut _;
    let gp: *mut HSD_GObj = &mut *g;
    unsafe {
        st::FIGHTER_CHANGEMOTIONSTATE = Some(|_, ms, fl, _, sp, _, _| log(format!("ms {ms} {fl:x} {sp}")));
        st::FTCOMMON_8007D7FC = Some(|_| log("7D7FC".into()));
        t::ext::FT_8008A1FC = Some(|_| false);
        turn::ext::FTCO_80094EA4 = Some(|_| false);
        turn::ext::FTDK_MS_347_800E0378 = Some(|_| false);
        ms::ext::FTCO_800C97A8 = Some(|_| false);
        walk::ext::FTWALKCOMMON_800DFC70 = Some(|_| false);
    }
    // 800DF980: air leaves the air state first.
    f.ground_or_air = GA_Air;
    t::ftDk_MS_341_800DF980(gp);
    assert_eq!(take(), ["7D7FC", "ms 341 0 1"]);
    f.ground_or_air = GA_Ground;
    t::ftDk_MS_341_800DF980(gp);
    assert_eq!(take(), ["ms 341 0 1"]);
    // 800DF938 follows the stick check.
    assert!(!t::ftDk_MS_341_800DF938(gp));
    assert!(take().is_empty());
    unsafe { t::ext::FT_8008A1FC = Some(|_| true) };
    assert!(t::ftDk_MS_341_800DF938(gp));
    assert_eq!(take(), ["ms 341 0 1"]);
    // IASA: nothing fires -> walk check (false) -> no state change.
    t::ftDk_HeavyWait0_IASA(gp);
    assert!(take().is_empty());
    // Walk check true -> walk motion state (341 + 1 via the hooked 800DFCA4).
    unsafe {
        walk::ext::FTWALKCOMMON_800DFC70 = Some(|_| true);
        walk::ext::FTWALKCOMMON_800DFCA4 = Some(|_, ms, _, _, _, _, _, _, _, _, _| log(format!("walk {ms}")));
    }
    t::ftDk_HeavyWait0_IASA(gp);
    assert_eq!(take(), ["walk 342"]);
    // An earlier handler (turn) short-circuits.
    unsafe {
        ms::ext::FTCO_800C97A8 = Some(|_| true);
        ms::ext::FTCO_TURN_ENTER = Some(|_, ms, _, _, _, _| log(format!("turn {ms}")));
    }
    t::ftDk_HeavyWait0_IASA(gp);
    assert_eq!(take(), ["turn 345"]);
}
