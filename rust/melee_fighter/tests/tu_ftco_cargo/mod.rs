//! Rust half of gen/cref/tu/ftco_cargo_fill.h (shared by the Capture*/Cargo* TU tests).
#![allow(dead_code, static_mut_refs)]
use crate::tu_ftco_harness::*;
use melee_fighter::types::*;

pub static mut DA: *mut _ftDonkeyAttributes = core::ptr::null_mut();
pub static mut JO: *mut HSD_JObj = core::ptr::null_mut();
pub fn da() -> &'static mut _ftDonkeyAttributes { unsafe { &mut *DA } }
pub fn jo() -> &'static mut HSD_JObj { unsafe { &mut *JO } }

pub fn pi<T>(p: *const T) -> u32 {
    let p = p as usize;
    if p == 0 { 0 }
    else if p == go() as usize { 1 }
    else if p == go2() as usize { 2 }
    else if p == f() as *const Fighter as usize { 3 }
    else if p == f2() as *const Fighter as usize { 4 }
    else { 9 }
}

pub fn cargo_init() {
    init();
    unsafe {
        DA = Box::leak(Box::new(_ftDonkeyAttributes::zeroed()));
        JO = Box::leak(Box::new(HSD_JObj::zeroed()));
    }
}

pub fn cargo_fill() {
    unsafe {
        *DA = _ftDonkeyAttributes::zeroed();
        *JO = HSD_JObj::zeroed();
        (*go()).hsd_obj = JO as *mut core::ffi::c_void;
    }
    let fr = f();
    fr.victim_gobj = go2();
    fr.x2CC = unsafe { DA };
    da().x4_motion_state = (rnd() % 400) as i32;
    da().cargo_hold.x24_JUMP_STARTUP_LAG = rw();
    da().cargo_hold.x28_LANDING_LAG = rw();
    fr.ground_or_air = rnd() & 1;
    fr.motion_id = (rnd() % 400) as _;
    fr.facing_dir = rg();
    fr.facing_dir1 = rg();
    fr.cur_anim_frame = rw();
    fr.grab_timer = rw();
    fr.input.lstick[0].x = rg();
    fr.input.lstick[0].y = rg();
    fr.input.pressed_buttons = (rnd() & 0x3FF) as _;
    fr.active_timer.lstick.x = (rnd() & 0xFF) as u8;
    fr.active_timer.lstick.y = (rnd() & 0xFF) as u8;
    fr.mv.ca.specialhi.vel.x = rw();
    fr.mv.ca.specialhi.vel.y = rw();
    fr.mv.co.capturekirby.scale.x = rw();
    fr.mv.co.capturekirby.scale.y = rw();
    fr.mv.co.capturekirby.scale.z = rw();
    fr.set_x221C_b6((rnd() & 1) as u16);
    f2().facing_dir = rg();
    let c = cd();
    c.tap_jump_threshold = rg();
    c.tap_jump_window = (rnd() % 300) as i32;
    c.x464 = rg();
    c.x468 = rw();
    c.dash_smash_stick_threshold = rg();
    c.dash_smash_window = (rnd() % 300) as i32;
    c.x44 = rw();
    c.x98 = rg();
    c.attackhi3_stick_threshold_y = rg();
    c.xB0 = rg();
    jo().flags = rnd();
}

pub fn state_hash() -> u32 {
    let fr = f();
    let mut h = Hs::new();
    unsafe {
        h.m(b(fr.facing_dir));
        h.m(b(fr.grab_timer));
        h.m(fr.active_timer.lstick.x as u32);
        h.m(fr.active_timer.lstick.y as u32);
        h.m(b(fr.mv.ca.specialhi.vel.x));
        h.m(b(fr.mv.ca.specialhi.vel.y));
        h.m(fr.mv.co.cargokneebend.x0 as u32);
        h.m(fr.mv.co.cargokneebend.x4 as u32);
        h.m(fr.mv.co.capturekirby.x18 as u32);
    }
    h.m(b(jo().scale.x));
    h.m(b(jo().scale.y));
    h.m(b(jo().scale.z));
    h.m(fr.accessory1_cb.is_some() as u32);
    h.m(fr.x2226_b2() as u32);
    h.m(fr.invisible() as u32);
    h.m(fr.x221B.x221B_b5() as u32);
    h.m(fr.x221B.x221B_b7() as u32);
    h.m(pi(fr.x1A5C));
    h.m(pi(fr.victim_gobj));
    h.m(fr.ground_or_air);
    h.0
}
