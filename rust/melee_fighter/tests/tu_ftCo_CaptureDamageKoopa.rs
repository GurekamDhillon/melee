//! Smoke test for tu::ftCo_CaptureDamageKoopa: the entry and the Anim branches.
#![allow(static_mut_refs)]
use melee_fighter::tu::ftCo_CaptureDamageKoopa::{self as t, ext::hook as h};
use melee_fighter::types::*;
use std::vec::Vec;

static mut LOG: Vec<String> = Vec::new();
static mut FRAMES: bool = true;
fn log(s: String) {
    unsafe { LOG.push(s) }
}

#[test]
fn smoke() {
    let f: &mut Fighter = Box::leak(Box::new(unsafe { core::mem::zeroed() }));
    let g: &mut HSD_GObj = Box::leak(Box::new(unsafe { core::mem::zeroed() }));
    g.user_data = (f as *mut Fighter).cast();
    f.facing_dir = 1.0;
    unsafe {
        h::Fighter_ChangeMotionState = Some(|_, m, fl, _, _, _, _| log(format!("cms {m} {fl}")));
        h::ftCommon_8007E2F4 = Some(|_, _| log("e2f4".into()));
        h::ftAnim_8006EBA4 = Some(|_| log("eba4".into()));
        h::ftAnim_IsFramesRemaining = Some(|_| FRAMES);
        h::ftCo_800BCC20 = Some(|_| log("wait".into()));
        h::ftCo_800BC458 = Some(|_| log("mash".into()));
        h::ftCo_800BC4A8 = Some(|_| log("wobble".into()));
        h::ftCo_800DA698 = Some(|_, _| log("catchcut".into()));
        h::ftCo_CaptureCut_Enter = Some(|_| log("cut".into()));
    }
    let gp: *mut HSD_GObj = g;
    t::ftCo_800BC9C8(gp);
    let f = unsafe { &mut *((*gp).user_data as *mut Fighter) };
    assert!(f.accessory1_cb.is_some());
    f.grab_timer = 1.0;
    t::ftCo_CaptureDamageKoopa_Anim(gp);
    f.grab_timer = 0.0;
    t::ftCo_CaptureDamageKoopa_Anim(gp);
    assert_eq!(f.facing_dir, -1.0);
    unsafe { FRAMES = false };
    t::ftCo_CaptureDamageKoopa_Anim(gp);
    let want = vec![
        format!("cms {} 0", ftCo_MS_CaptureDamageKoopa as i32),
        "e2f4".into(),
        "eba4".into(),
        "mash".into(),
        "wobble".into(),
        "mash".into(),
        "catchcut".into(),
        "cut".into(),
        "wait".into(),
    ];
    assert_eq!(unsafe { LOG.clone() }, want);
}
