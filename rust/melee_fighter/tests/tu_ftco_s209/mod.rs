//! Rust half of gen/cref/tu/ftco_s209_fill.h (RunBrake/RunDirect/ShieldBreak*/Shouldered/SpecialAir
//! TU tests): same fill order, same hash. Every callee out of a TU is a recording hook; a
//! bool-returning one returns `sr(id)`.
#![allow(dead_code, static_mut_refs)]
use crate::tu_ftco_harness::*;
use melee_fighter::types::*;

pub static mut JO: *mut [HSD_JObj; 4] = core::ptr::null_mut();
pub static mut BONES: *mut [FighterBone; 4] = core::ptr::null_mut();
pub static mut FD: *mut ftData = core::ptr::null_mut();
pub static mut FD8: *mut ftData_x8 = core::ptr::null_mut();

pub fn pi<T>(p: *const T) -> u32 {
    let p = p as usize;
    let fr = f();
    let fr2 = f2();
    if p == 0 { return 0; }
    if p == go() as usize { return 1; }
    if p == go2() as usize { return 2; }
    if p == fr as *const Fighter as usize { return 3; }
    if p == fr2 as *const Fighter as usize { return 4; }
    for i in 0..4 {
        if p == unsafe { &(*JO)[i] } as *const HSD_JObj as usize { return 10 + i as u32; }
    }
    if p == &fr.xDF4[1] as *const HitCapsule as usize { return 20; }
    if p == &fr2.xDF4[1] as *const HitCapsule as usize { return 21; }
    if p == &fr.dmg.facing_dir_1 as *const f32 as usize { return 22; }
    if p == &fr2.dmg.facing_dir_1 as *const f32 as usize { return 23; }
    if p == &fr.x60C as *const *mut core::ffi::c_void as usize { return 24; }
    9
}

pub fn s209_init() {
    init();
    unsafe {
        JO = Box::leak(Box::new([HSD_JObj::zeroed(); 4]));
        BONES = Box::leak(Box::new([FighterBone::zeroed(); 4]));
        FD = Box::leak(Box::new(ftData::zeroed()));
        FD8 = Box::leak(Box::new(ftData_x8::zeroed()));
    }
}

fn hit_fill(h: &mut HitCapsule) {
    h.unk_count = rnd() & 7;
    h.damage = rw();
    h.kb_angle = (rnd() % 361) as i32;
    h.element = rnd() % 16;
}

fn mv_word() -> u32 {
    match rnd() % 4 {
        0 => 0,
        1 => rnd() & 3,
        _ => b(rw()),
    }
}

pub fn mv_words(fr: &mut Fighter) -> *mut u32 {
    &mut fr.mv as *mut Fighter_MotionVars as *mut u32
}

fn mv_fill(fr: &mut Fighter) {
    let w = mv_words(fr);
    unsafe {
        *w = mv_word();
        *w.add(1) = mv_word();
    }
}

pub fn s209_fill() {
    unsafe {
        *JO = [HSD_JObj::zeroed(); 4];
        *BONES = [FighterBone::zeroed(); 4];
        *FD = ftData::zeroed();
        *FD8 = ftData_x8::zeroed();
        for i in 0..4 {
            (*BONES)[i].joint = &mut (*JO)[i];
        }
        (*FD).x8 = FD8;
        (*FD8).x11 = (rnd() % 4) as u8;
    }
    let fr = f();
    unsafe {
        fr.ft_data = FD;
        fr.parts = BONES as *mut FighterBone;
        f2().parts = BONES as *mut FighterBone;
    }
    fr.victim_gobj = go2();
    f2().victim_gobj = go();
    fr.kind = (rnd() % 33) as _;
    fr.ground_or_air = rnd() & 1;
    fr.motion_id = match rnd() % 4 {
        0 => ftCo_MS_ShieldBreakDownU as _,
        1 => ftCo_MS_ShoulderedWait as _,
        _ => (rnd() % 400) as _,
    };
    fr.facing_dir = match rnd() % 4 {
        0 => 1.0,
        1 => -1.0,
        _ => rg(),
    };
    fr.gr_vel = rw();
    fr.grab_timer = rg();
    fr.input.lstick[0].x = rg();
    fr.input.lstick[0].y = rg();
    fr.input.pressed_buttons = (rnd() & 0x3FF) as _;
    fr.active_duration.lstick.x = (rnd() & 0xFF) as u8;
    fr.cmd_vars[0] = rnd() & 1;
    fr.cmd_vars[1] = rnd() & 1;
    mv_fill(fr);
    fr.co_attrs.ground_friction = rw();
    fr.co_attrs.max_run_brake_frames = rw();
    fr.co_attrs.shield_break_initial_velocity = rw();
    fr.x197C = if rnd() & 1 != 0 { go2() } else { core::ptr::null_mut() };
    fr.set_x2222_b6((rnd() & 1) as u8);
    fr.set_x2228_b7((rnd() & 1) as u8);
    fr.dmg.x1830_percent = rw();
    fr.self_vel.x = rw();
    fr.self_vel.y = rw();
    hit_fill(&mut fr.xDF4[1]);
    let fr2 = f2();
    fr2.facing_dir = rg();
    fr2.ground_or_air = rnd() & 1;
    mv_fill(fr2);
    hit_fill(&mut fr2.xDF4[1]);
    let c = cd();
    c.x58_someLStickXThreshold = rg();
    c.x42C = rw();
    c.run_dash_turn_friction_multiplier = rw();
    c.x410 = (rnd() % 100) as i32;
    c.x4A0 = rw();
    c.x4A4 = rw();
    c.x4A8 = rw();
    c.x4AC = rw();
    c.x3B0 = rw();
    c.shouldered_anim_rate = rw();
    c.x218 = rg();
    c.x21C = rg();
    c.x220 = rg();
    c.x224 = (rnd() % 300) as i32;
}

fn hm_dmg(h: &mut Hs, fr: &Fighter) {
    h.m(b(fr.dmg.kb_applied));
    h.m(fr.dmg.x1848_kb_angle as u32);
    h.m(b(fr.dmg.facing_dir_1));
    h.m(fr.dmg.x184c_damaged_hurtbox as u32);
    h.m(b(fr.dmg.x1854_collpos.x));
    h.m(b(fr.dmg.x1854_collpos.y));
    h.m(b(fr.dmg.x1854_collpos.z));
    h.m(fr.dmg.x1860_element);
}

pub fn state_hash() -> u32 {
    let fr = f();
    let w = mv_words(fr);
    let mut h = Hs::new();
    h.m(fr.cmd_vars[0]);
    h.m(fr.cmd_vars[1]);
    unsafe {
        h.m(*w);
        h.m(*w.add(1));
    }
    h.m(b(fr.facing_dir));
    h.m(b(fr.self_vel.x));
    h.m(b(fr.self_vel.y));
    h.m(fr.x2222_b3() as u32);
    h.m(fr.x2227_b5() as u32);
    h.m(fr.accessory1_cb.is_some() as u32);
    hm_dmg(&mut h, fr);
    hm_dmg(&mut h, f2());
    h.0
}
