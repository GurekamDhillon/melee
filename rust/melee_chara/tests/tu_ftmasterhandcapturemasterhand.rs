//! Smoke test of tu::ftmasterhandcapturemasterhand.
#![allow(static_mut_refs)]
#[path = "common_mh/mod.rs"]
mod common;
use common::*;
use melee_chara::tu::ftmasterhandcapturemasterhand as t;
use melee_fighter::hooks as fh;
use melee_types::*;

#[test]
fn smoke() {
    let mut r = Rig::new();
    let gp = r.gp();
    hook_stubs(1);
    t::ftMh_CaptureMasterHand_Anim(gp);
    t::ftMh_CaptureMasterHand_IASA(gp);
    t::ftMh_CaptureMasterHand_Phys(gp);
    t::ftMh_CaptureMasterHand_Coll(gp);
    assert_eq!(take_log(), "");
    t::ftMh_CaptureMasterHand_80155B80(gp);
    assert_eq!(take_log(), "cms 328 0 0 1 0 true,anim8006EBA4");
    assert_eq!(r.f.invisible(), 1);
    assert_eq!(r.f.x2220_b3(), 1);
    assert_eq!(r.f.x1A6A, 511);
    assert!(r.f.accessory1_cb.is_some());

    // Anim: the mash adds nothing without input; timer above 0 -> nothing happens.
    let mut cd: Box<ftCommonData> = Box::new(unsafe { core::mem::zeroed() });
    cd.x3A8 = 1.0;
    cd.x308 = 0.5;
    unsafe {
        fh::P_FT_COMMON_DATA = &*cd;
        fh::PL_800402D0 = Some(|_, _, _| {});
    }
    r.f.grab_timer = 5.0;
    t::ftMh_CaptureDamageMasterHand_Anim(gp);
    assert_eq!(take_log(), "");
    assert_eq!(r.f.grab_timer, 5.0);
    // Timer expired: back to CaptureWait (329) and the victim (here: itself) goes to Cancel (375).
    r.f.grab_timer = 0.0;
    r.f.victim_gobj = gp;
    r.da.x120 = 9.0;
    t::ftMh_CaptureDamageMasterHand_Anim(gp);
    assert_eq!(take_log(), "cms 329 0 0 1 0 true,anim8006EBA4,cms 375 0 0 1 0 true,anim8006EBA4");
    assert_eq!(unsafe { r.f.mv.mh.unk0.x24 }, 9.0);
}
