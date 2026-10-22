//! Smoke test of tu::ftmasterhandthrownmasterhand (hooks record the calls).
#![allow(static_mut_refs)]
mod mh_common;
use mh_common::*;
use melee_chara::tu::ftmasterhandthrownmasterhand as t;
use melee_types::*;

#[test]
fn smoke() {
    let mut r = Rig::new();
    let gp = r.gp();
    unsafe { t::ext::FTCO_CAPTURECUT_ENTER = Some(|_| log("capturecut")) };
    // 80155D1C: visible again, facing flipped, CaptureCut.
    r.f.set_invisible(1);
    r.f.facing_dir = 1.0;
    t::ftMh_CaptureWaitMasterHand_80155D1C(gp);
    assert_eq!((take(), r.f.invisible(), r.f.facing_dir), ("capturecut".to_string(), 0, -1.0));
    // 80155D6C: copies the victim's facing, enters ThrownMasterHand (330).
    let mut vf: Box<Fighter> = Box::new(Fighter::zeroed());
    vf.facing_dir = -1.0;
    let mut vg: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    vg.user_data = &mut *vf as *mut Fighter as *mut _;
    r.f.victim_gobj = &mut *vg;
    r.f.facing_dir = 1.0;
    unsafe { r.f.mv.mh.unk4.x0 = 7 };
    t::ftMh_CaptureWaitMasterHand_80155D6C(gp, 330);
    assert_eq!(r.f.facing_dir, -1.0);
    assert_eq!(unsafe { r.f.mv.mh.unk4.x0 }, 0);
    assert!(r.f.accessory1_cb.is_some());
    assert!(take().starts_with("cms 330 0 0 1 0"));
    for f in [
        t::ftMh_CaptureWaitMasterHand_Anim, t::ftMh_CaptureWaitMasterHand_IASA,
        t::ftMh_CaptureWaitMasterHand_Phys, t::ftMh_CaptureWaitMasterHand_Coll,
        t::ftMh_ThrownMasterHand_Anim, t::ftMh_ThrownMasterHand_IASA,
        t::ftMh_ThrownMasterHand_Phys, t::ftMh_ThrownMasterHand_Coll,
    ] {
        f(gp);
    }
}
