//! tu::ftpopo smoke test: Ice Climbers' death reset and attribute copies.
#![allow(static_mut_refs, unused_unsafe)]
mod gw_common;
use gw_common::*;
use melee_chara::stubs as st;
use melee_chara::tu::ftpopo as t;
use melee_types::*;

#[test]
fn init() {
    install_common();
    unsafe { st::FTPARTS_80074A4C = Some(|_, a, _| log(format!("parts {a}"))) };
    let mut w = World::new();
    let gp = w.gp();
    unsafe {
        w.f.u.nn.x2234 = 7;
        w.f.u.nn.x224C = 9;
        w.f.u.nn.x2250 = 2.0;
    }
    t::ftPp_Init_OnDeath(gp);
    assert_eq!(take(), "parts 0,parts 1");
    assert_eq!(unsafe { (w.f.u.nn.x2234, w.f.u.nn.x224C, w.f.u.nn.x2250) }, (0, 0, 0.0));

    // 8011F16C clears the callbacks only for the tracked item.
    let item = fake(0x1000) as *mut Item_GObj;
    unsafe { w.f.u.nn.x222C = item as *mut _ };
    t::ftPp_Init_8011F16C(gp, fake(0x2000) as *mut Item_GObj);
    assert!(!unsafe { w.f.u.nn.x222C }.is_null());
    t::ftPp_Init_8011F16C(gp, item);
    assert!(unsafe { w.f.u.nn.x222C }.is_null());
    assert_eq!(t::ftPp_Init_CostumeStrings[3][0], "PlPpRe.dat");
}
