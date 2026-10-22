//! tu::ftpopospecialn smoke test: Ice Shot Air Enter and ground Coll.
#![allow(static_mut_refs, unused_unsafe)]
mod gw_common;
use gw_common::*;
use melee_chara::tu::ftpopospecialn as t;
use melee_types::*;

#[test]
fn ice_shot() {
    install_common();
    let mut w = World::new();
    let gp = w.gp();
    let da: &'static mut ftIceClimberAttributes = Box::leak(Box::new(unsafe { core::mem::zeroed() }));
    da.x4 = 3.0;
    w.f.dat_attrs = da as *mut _ as *mut _;
    w.f.cmd_vars = [1, 1, 1, 1];
    unsafe { w.f.u.nn.x224C = 0 };
    t::ftPp_SpecialAirN_Enter(gp);
    assert_eq!(take(), "ms 342 0 0,anim");
    assert_eq!(w.f.self_vel.y, 3.0);
    assert_eq!(unsafe { w.f.u.nn.x224C }, 1);
    assert!(w.f.accessory4_cb.is_some());
    // second use in the air: no jump boost, the timer starts at -10.
    t::ftPp_SpecialAirN_Enter(gp);
    let _ = take();
    assert_eq!(unsafe { w.f.u.nn.x2250 }, -10.0);
    assert_eq!(w.f.self_vel.y, 3.0);
}
