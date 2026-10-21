//! Smoke test for tu::ftCo_CaptureMewtwo: entry order, grab links and the motion ids.
#![allow(static_mut_refs)]
use melee_fighter::tu::ftCo_CaptureMewtwo::{self as t, ext::hook as h};
use melee_fighter::types::*;
use std::vec::Vec;

static mut LOG: Vec<String> = Vec::new();
fn log(s: String) {
    unsafe { LOG.push(s) }
}

#[test]
fn enter_sequence() {
    let f: &mut Fighter = Box::leak(Box::new(unsafe { core::mem::zeroed() }));
    let v: &mut Fighter = Box::leak(Box::new(unsafe { core::mem::zeroed() }));
    let g: &mut HSD_GObj = Box::leak(Box::new(unsafe { core::mem::zeroed() }));
    let vg: &mut HSD_GObj = Box::leak(Box::new(unsafe { core::mem::zeroed() }));
    g.user_data = (f as *mut Fighter).cast();
    vg.user_data = (v as *mut Fighter).cast();
    v.facing_dir = -1.0;
    f.facing_dir = 1.0;
    f.anon_221b.set_x221B_b5(1);
    unsafe {
        h::ftCo_ReleaseItemAndVictim = Some(|_| log("release".into()));
        h::ftCo_800DB368 = Some(|_, _| log("db368".into()));
        h::Fighter_ChangeMotionState = Some(|_, m, fl, _, _, _, o| log(format!("cms {m} {fl} {}", !o.is_null())));
        h::ftCommon_8007D5D4 = Some(|_| log("d5d4".into()));
        h::ftCommon_8007E2F4 = Some(|_, v| log(format!("e2f4 {v:#x}")));
        h::ftAnim_8006EBA4 = Some(|_| log("eba4".into()));
        h::ftCommon_8007E2FC = Some(|_| log("e2fc".into()));
        h::ftCo_800BD0E8 = Some(|_, m| log(format!("thrown {m}")));
    }
    let (gp, vp): (*mut HSD_GObj, *mut HSD_GObj) = (g, vg);
    t::ftCo_800BD000(gp, vp);
    let f = unsafe { &*((*gp).user_data as *const Fighter) };
    assert_eq!(f.victim_gobj, vp);
    assert_eq!(f.x1A5C, vp);
    assert_eq!(f.anon_221b.x221B_b5(), 0);
    assert_eq!(f.facing_dir, -1.0);
    let want = vec![
        "release".to_string(),
        "db368".into(),
        format!("cms {} 0 true", ftCo_MS_CaptureMewtwoAir as i32),
        "d5d4".into(),
        "e2f4 0x1ff".into(),
        "eba4".into(),
        "e2fc".into(),
        format!("thrown {}", ftCo_MS_ThrownMewtwoAir as i32),
    ];
    assert_eq!(unsafe { LOG.clone() }, want);
}
