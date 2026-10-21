//! Smoke test for tu::ftCo_CaptureCut: the release (not constrained: only the grab links are
//! cleared), the push on Enter and the NULL-partner guard.
#![allow(static_mut_refs)]
use melee_fighter::tu::ftCo_CaptureCut::{self as t, ext::hook as h};
use melee_fighter::types::*;
use std::vec::Vec;

static mut LOG: Vec<String> = Vec::new();
fn log(s: String) {
    unsafe { LOG.push(s) }
}

#[test]
fn smoke() {
    let cd: &mut ftCommonData = Box::leak(Box::new(unsafe { core::mem::zeroed() }));
    cd.x370 = 1.25;
    let f: &mut Fighter = Box::leak(Box::new(unsafe { core::mem::zeroed() }));
    let v: &mut Fighter = Box::leak(Box::new(unsafe { core::mem::zeroed() }));
    let g: &mut HSD_GObj = Box::leak(Box::new(unsafe { core::mem::zeroed() }));
    let vg: &mut HSD_GObj = Box::leak(Box::new(unsafe { core::mem::zeroed() }));
    g.user_data = (f as *mut Fighter).cast();
    vg.user_data = (v as *mut Fighter).cast();
    let (gp, vp): (*mut HSD_GObj, *mut HSD_GObj) = (g, vg);
    f.victim_gobj = vp;
    f.x1A5C = vp;
    v.victim_gobj = gp;
    v.x1A5C = gp;
    v.anon_221b.set_x221B_b5(1);
    f.facing_dir = 1.0;
    f.ground_or_air = GA_Ground;
    unsafe {
        melee_fighter::hooks::P_FT_COMMON_DATA = cd;
        h::Fighter_UnkSetFlag_8006CFBC = Some(|_| log("flag".into()));
        h::Fighter_ChangeMotionState = Some(|_, m, _, _, _, _, _| log(format!("cms {m}")));
        h::Mex_Enabled = Some(|_| 0);
    }
    t::ftCo_CaptureCut_Enter(gp);
    let f = unsafe { &*((*gp).user_data as *const Fighter) };
    let v = unsafe { &*((*vp).user_data as *const Fighter) };
    assert_eq!(f.gr_vel, -1.25);
    assert!(f.victim_gobj.is_null() && f.x1A5C.is_null());
    assert!(v.victim_gobj.is_null() && v.x1A5C.is_null());
    assert_eq!(v.anon_221b.x221B_b5(), 0);
    assert_eq!(unsafe { LOG.clone() }, ["flag".to_string(), "flag".into(), format!("cms {}", ftCo_MS_CaptureCut as i32)]);
    // NULL partner: guarded out
    unsafe { LOG.clear() };
    t::ftCo_800DCE34(gp, core::ptr::null_mut());
    assert!(unsafe { LOG.is_empty() });
}
