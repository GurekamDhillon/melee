//! Smoke test for tu::ftCo_CaptureWait: the mash/timer drain, the anim-rate cycle, the jump-out
//! input and the non-Yoshi entry.
#![allow(static_mut_refs)]
use melee_fighter::tu::ftCo_CaptureWait::{self as t, ext::hook as h};
use melee_fighter::types::*;
use std::vec::Vec;

static mut LOG: Vec<String> = Vec::new();
static mut MASH: bool = false;
fn log(s: String) {
    unsafe { LOG.push(s) }
}

#[test]
fn smoke() {
    let cd: &mut ftCommonData = Box::leak(Box::new(unsafe { core::mem::zeroed() }));
    cd.grab_timer_decrement = 1.0;
    cd.x3A8 = 0.5;
    cd.x3AC = 5.0;
    cd.x3B0 = 4.0;
    cd.shouldered_anim_rate = 3.0;
    cd.tap_jump_threshold = 0.6;
    let f: &mut Fighter = Box::leak(Box::new(unsafe { core::mem::zeroed() }));
    let v: &mut Fighter = Box::leak(Box::new(unsafe { core::mem::zeroed() }));
    let g: &mut HSD_GObj = Box::leak(Box::new(unsafe { core::mem::zeroed() }));
    let vg: &mut HSD_GObj = Box::leak(Box::new(unsafe { core::mem::zeroed() }));
    g.user_data = (f as *mut Fighter).cast();
    vg.user_data = (v as *mut Fighter).cast();
    let gp: *mut HSD_GObj = g;
    f.victim_gobj = vg;
    f.grab_timer = 3.0;
    v.kind = Ft_Kind_Mario;
    unsafe {
        melee_fighter::hooks::P_FT_COMMON_DATA = cd;
        h::ftCommon_GrabMash = Some(|_, _| MASH);
        h::ftAnim_SetAnimRate = Some(|_, r| log(format!("rate {r}")));
        h::ftCo_800DA698 = Some(|_, _| log("catchcut".into()));
        h::Fighter_ChangeMotionState = Some(|_, m, _, _, _, _, _| log(format!("cms {m}")));
        h::ftCommon_8007E2F4 = Some(|_, _| log("e2f4".into()));
        h::ftCommon_8007EBAC = Some(|_, a, _| log(format!("ebac {a}")));
    }
    let f = unsafe { &mut *((*gp).user_data as *mut Fighter) };
    // mash -> anim rate goes up for x3B0 frames
    unsafe { MASH = true };
    t::ftCo_CaptureWaitHi_Anim(gp);
    assert_eq!(f.grab_timer, 2.0);
    assert_eq!(unsafe { f.mv.co.capturewait.x4 }, 4.0);
    unsafe { MASH = false };
    t::ftCo_CaptureWaitHi_Anim(gp);
    assert_eq!(unsafe { f.mv.co.capturewait.x4 }, 3.0);
    assert_eq!(unsafe { f.mv.co.capturewait.x0 }, 2.0);
    assert_eq!(unsafe { LOG.clone() }, ["rate 3"]);
    // X pressed early buffers the jump-out
    f.input.pressed_buttons = 1 << 10;
    t::ftCo_CaptureWaitHi_IASA(gp);
    assert_eq!(unsafe { f.mv.co.capturewait.xC }, 1);
    f.input.lstick[0].y = 0.7;
    assert!(t::fn_800DC044(gp));
    // non-Yoshi entry
    unsafe { LOG.clear() };
    f.motion_id = ftCo_MS_CapturePulledHi as i32;
    t::fn_800DB6C8(gp);
    assert_eq!(unsafe { LOG.clone() }, [format!("cms {}", ftCo_MS_CaptureWaitHi as i32), "e2f4".into(), "ebac 3".into()]);
    assert_eq!(f.invisible(), 0);
}
