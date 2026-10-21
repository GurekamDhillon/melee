//! Smoke test of tu::ftco_09c4 (no float math to check against the C).
#![allow(static_mut_refs)]

use melee_fighter::hooks as h;
use melee_fighter::tu::ftco_09c4 as t;
use melee_fighter::types::*;
use std::vec::Vec;

static mut LOG: std::vec::Vec<String> = std::vec::Vec::new();
fn log(s: String) {
    unsafe { LOG.push(s) }
}

#[test]
fn smoke() {
    let mut f: Box<Fighter> = Box::new(Fighter::zeroed());
    let mut g: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    let dk: &'static mut _ftDonkeyAttributes = Box::leak(Box::new(unsafe { core::mem::zeroed() }));
    dk.x4_motion_state = 340;
    f.x2CC = dk;
    g.user_data = &mut *f as *mut Fighter as *mut _;
    let gp: *mut HSD_GObj = &mut *g;
    unsafe {
        t::ext::FTCO_80099F1C = Some(|_| true);
        t::ext::FTCO_8009A184 = Some(|_, ms, fl, a| log(format!("A184 {ms} {fl} {a}")));
        t::ext::FTCO_8009C5A4 = Some(|v, ms| log(format!("C5A4 {} {ms}", v.is_null())));
        h::FTANIM_SET_ANIM_RATE = Some(|_, r| log(format!("rate {r}")));
    }
    assert!(t::ftCo_8009C4F8(gp));
    unsafe { t::ext::FTCO_80099F1C = Some(|_| false) };
    assert!(!t::ftCo_8009C4F8(gp));
    assert_eq!(unsafe { LOG.join(",") }, format!("A184 346 0 0,rate 0,C5A4 true {}", ftCo_MS_ShoulderedWait));
}
