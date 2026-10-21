//! Smoke test for tu::ftdonkeyheavywalk (stateful; no fused sites) through recording hooks.
#![allow(static_mut_refs)]

use melee_chara::tu::{ftdonkeyheavyturn as turn, ftdonkeyheavywait0 as w0, ftdonkeyheavywalk as t};
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
    a.x8 = 1.0;
    a.xC = 2.0;
    a.x10 = 3.0;
    a.x14 = 4.0;
    a.x18 = 5.0;
    a.x1C = 6.0;
    f.x2CC = a;
    g.user_data = &mut *f as *mut Fighter as *mut _;
    let gp: *mut HSD_GObj = &mut *g;
    unsafe {
        t::ext::FTWALKCOMMON_800DFC70 = Some(|_| false);
        t::ext::FTWALKCOMMON_800DFCA4 =
            Some(|_, ms, fl, s, a, b, c, d, e, f, m| log(format!("walk {ms} {fl} {s} {a} {b} {c} {d} {e} {f} {m}")));
        t::ext::FTWALKCOMMON_800DFDDC = Some(|_| log("anim".into()));
        t::ext::FTWALKCOMMON_800DFEC8 = Some(|_, _cb| log("iasa".into()));
        t::ext::FTWALKCOMMON_800E0060 = Some(|_| log("phys".into()));
        turn::ext::FTCO_80094EA4 = Some(|_| false);
        turn::ext::FTDK_MS_347_800E0378 = Some(|_| false);
        w0::ext::FT_8008A1FC = Some(|_| false);
    }
    // 800DFA98: not walking -> false, nothing started.
    assert!(!t::ftDk_MS_341_800DFA98(gp));
    assert!(take().is_empty());
    unsafe { t::ext::FTWALKCOMMON_800DFC70 = Some(|_| true) };
    assert!(t::ftDk_MS_341_800DFA98(gp));
    assert_eq!(take(), ["walk 342 0 0 1 2 3 4 5 6 1"]);
    t::ftDk_HeavyWalk_Anim(gp);
    t::ftDk_HeavyWalk_Phys(gp);
    assert_eq!(take(), ["anim", "phys"]);
    // IASA reaches the walk-common IASA only when every earlier check says no.
    t::ftDk_HeavyWalk_IASA(gp);
    assert_eq!(take(), ["iasa"]);
    unsafe { turn::ext::FTCO_80094EA4 = Some(|_| true) };
    t::ftDk_HeavyWalk_IASA(gp);
    assert!(take().is_empty());
    unsafe {
        turn::ext::FTCO_80094EA4 = Some(|_| false);
        turn::ext::FTDK_MS_347_800E0378 = Some(|_| true);
    }
    t::ftDk_HeavyWalk_IASA(gp);
    assert!(take().is_empty());
}
