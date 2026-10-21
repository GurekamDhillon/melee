//! Smoke test for tu::ftdonkeyms3450 (stateful; no fused sites) through recording hooks.
#![allow(static_mut_refs)]

use melee_chara::tu::ftdonkeyms3450 as t;
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
    a.cargo_hold.x20_TURN_SPEED = 7.5;
    f.x2CC = a;
    g.user_data = &mut *f as *mut Fighter as *mut _;
    let gp: *mut HSD_GObj = &mut *g;
    unsafe {
        t::ext::FTCO_TURN_ENTER = Some(|_, ms, fl, a3, turn, start| log(format!("turn {ms} {fl} {a3} {turn} {start}")));
        t::ext::FTCO_800C97A8 = Some(|_| false);
    }
    assert!(!t::ftDk_MS_345_800E0134(gp));
    assert!(take().is_empty());
    unsafe { t::ext::FTCO_800C97A8 = Some(|_| true) };
    assert!(t::ftDk_MS_345_800E0134(gp));
    assert_eq!(take(), ["turn 345 0 0 7.5 0"]);
}
