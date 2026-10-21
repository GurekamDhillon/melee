//! Smoke test for tu::ftCo_CaptureJump: the jump-out velocity, the anim counter and the IASA
//! short-circuit order.
#![allow(static_mut_refs)]
use melee_fighter::tu::ftCo_CaptureJump::{self as t, ext::hook as h};
use melee_fighter::types::*;
use std::vec::Vec;

static mut LOG: Vec<String> = Vec::new();
static mut HIT: usize = 99;
fn log(s: &str) {
    unsafe { LOG.push(s.into()) }
}
fn chk(i: usize, s: &str) -> bool {
    log(s);
    unsafe { HIT == i }
}

#[test]
fn smoke() {
    let cd: &mut ftCommonData = Box::leak(Box::new(unsafe { core::mem::zeroed() }));
    cd.x374 = 2.5;
    cd.x378 = 3.0;
    cd.x3B8 = 4.0;
    let f: &mut Fighter = Box::leak(Box::new(unsafe { core::mem::zeroed() }));
    let g: &mut HSD_GObj = Box::leak(Box::new(unsafe { core::mem::zeroed() }));
    g.user_data = (f as *mut Fighter).cast();
    f.facing_dir = 1.0;
    unsafe {
        melee_fighter::hooks::P_FT_COMMON_DATA = cd;
        h::ftCommon_8007D5D4 = Some(|_| log("d5d4"));
        h::ftCo_800DC920 = Some(|_, _| log("cut"));
        h::Fighter_ChangeMotionState = Some(|_, _, _, _, _, _, _| log("cms"));
        h::ftAnim_IsFramesRemaining = Some(|_| false);
        h::ftCo_Fall_Enter = Some(|_| log("fall"));
        h::ftCo_SpecialAir_CheckInput = Some(|_| chk(0, "sp"));
        h::ftCo_80095328 = Some(|_, _| chk(1, "it"));
        h::ftCo_800D7100 = Some(|_| chk(2, "d7100"));
        h::ftCo_800C3B10 = Some(|_| chk(3, "c3b10"));
        h::ftCo_80099A58 = Some(|_| chk(4, "esc"));
        h::ftCo_AttackAir_CheckItemThrowInput = Some(|_| chk(5, "aa"));
        h::ftCo_800D705C = Some(|_| chk(6, "d705c"));
        h::ftCo_800CB870 = Some(|_| chk(7, "jump"));
    }
    let gp: *mut HSD_GObj = g;
    t::fn_800DC070(gp);
    let f = unsafe { &mut *((*gp).user_data as *mut Fighter) };
    assert_eq!((f.self_vel.x, f.self_vel.y), (-2.5, 3.0));
    t::ftCo_CaptureJump_Anim(gp);
    assert_eq!(unsafe { f.mv.co.buryjump.x0 }, 1.0);
    assert_eq!(unsafe { LOG.clone() }, ["d5d4", "cut", "cms", "fall"]);
    // grav (aliases buryjump.x0 = 1) below x3B8: no checks
    unsafe { LOG.clear() };
    t::ftCo_CaptureJump_IASA(gp);
    assert!(unsafe { LOG.is_empty() });
    f.mv.ca.specials.grav = 4.0;
    unsafe { HIT = 3 };
    t::ftCo_CaptureJump_IASA(gp);
    assert_eq!(unsafe { LOG.clone() }, ["sp", "it", "d7100", "c3b10"]);
}
