//! Smoke test for tu::ftdonkeyheavywait1 (stateful; no fused sites) through recording hooks.
#![allow(static_mut_refs)]

use melee_chara::stubs as st;
use melee_chara::tu::{ftdonkeyheavyturn as turn, ftdonkeyheavywait1 as t};
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
    a.cargo_hold.x28_LANDING_LAG = 2.0;
    f.x2CC = a;
    g.user_data = &mut *f as *mut Fighter as *mut _;
    let gp: *mut HSD_GObj = &mut *g;
    unsafe {
        st::FIGHTER_CHANGEMOTIONSTATE = Some(|_, ms, fl, _, sp, _, _| log(format!("ms {ms} {fl:x} {sp}")));
        st::FTANIM_SETANIMRATE = Some(|_, r| log(format!("rate {r}")));
        turn::ext::FTCO_80094EA4 = Some(|_| true);
        t::ext::FTCO_KNEEBEND_CHECK_SHORTHOP = Some(|_| log("shorthop".into()));
        t::ext::FTCO_LANDING_ENTER = Some(|_, ms, ai, fl, s, sp| log(format!("land {ms} {ai} {fl} {s} {sp}")));
    }
    t::ftDk_HeavyWait1_IASA(gp);
    assert!(take().is_empty());
    unsafe { turn::ext::FTCO_80094EA4 = Some(|_| false) };
    t::ftDk_HeavyWait1_IASA(gp);
    assert_eq!(take(), ["shorthop"]);
    t::ftDk_MS_346_800E05E4(gp);
    assert_eq!(take(), ["land 349 true 0 0 1", "rate 0"]);
    unsafe { assert_eq!(f.mv.dk.unk8.x4, 2.0) };
    // Landing countdown: 2 -> 1 -> 0 (no wait yet), then the wait transition fires and it goes to -1.
    t::ftDk_HeavyLanding_Anim(gp);
    t::ftDk_HeavyLanding_Anim(gp);
    assert!(take().is_empty());
    unsafe { assert_eq!(f.mv.dk.unk8.x4, 0.0) };
    t::ftDk_HeavyLanding_Anim(gp);
    assert_eq!(take(), ["ms 341 0 1"]);
    unsafe { assert_eq!(f.mv.dk.unk8.x4, -1.0) };
}
