//! Rust half of gen/cref/tu/ftco_misc8_fill.h: same fill order, same state hash.
#![allow(dead_code, static_mut_refs)]
use super::tu_ftco_harness::*;
use melee_fighter::types::*;

pub static mut FD: *mut ftData = core::ptr::null_mut();
pub static mut SFX: *mut FtSFX = core::ptr::null_mut();
pub static mut JO: *mut HSD_JObj = core::ptr::null_mut();
pub static mut PARTS: *mut FighterBone = core::ptr::null_mut();

pub fn m8_init() {
    init();
    unsafe {
        FD = Box::leak(Box::new(ftData::zeroed()));
        SFX = Box::leak(Box::new(FtSFX::zeroed()));
        JO = Box::leak(Box::new(HSD_JObj::zeroed()));
        PARTS = Box::leak(Box::new([FighterBone::zeroed(), FighterBone::zeroed()])).as_mut_ptr();
    }
}

pub fn pi<T>(p: *const T) -> u32 {
    let p = p as usize;
    unsafe {
        if p == 0 { 0 }
        else if p == GO as usize { 1 }
        else if p == GO2 as usize { 2 }
        else if p == F as usize { 3 }
        else if p == &(*F).x60C as *const _ as usize { 4 }
        else if p == JO as usize { 5 }
        else { 9 }
    }
}
pub fn pf(fr: &Fighter) -> u32 { pi(fr as *const Fighter) }

fn bits(v: f32) -> u32 { v.to_bits() }

pub fn m8_fill() {
    unsafe {
        *FD = ftData::zeroed();
        *SFX = FtSFX::zeroed();
        *JO = HSD_JObj::zeroed();
        *PARTS = FighterBone::zeroed();
        *PARTS.add(1) = FighterBone::zeroed();
        let fr = f();
        fr.ft_data = FD;
        (*FD).x4C_sfx = SFX;
        fr.parts = PARTS;
        (*PARTS).joint = JO;
        (*SFX).x18 = rnd() as i32;
        (*SFX).x24 = rnd() as i32;
    }
    let d = cd();
    d.x478 = rw(); d.x47C = rw(); d.x464 = rg(); d.x468 = (rnd() % 300) as f32; d.x46C = rw();
    d.x470 = rw(); d.x254 = rg(); d.x3D8 = rg(); d.x3DC = rw(); d.x760 = (rnd() % 3) as i32;
    d.x764 = rnd() as i32; d.x250 = (rnd() % 10) as f32; d.tap_jump_threshold = rg();
    d.passive_wall_vel_y_base = rg(); d.x58_someLStickXThreshold = rg(); d.x430 = rw();
    d.x410 = rnd() as i32; d.x440 = rg(); d.run_accel_taper_gain = rg();
    d.run_dash_turn_friction_multiplier = rg();
    let fr = f();
    fr.coll_data.env_flags = rnd() as i32; fr.set_is_sandbag((rnd() & 1) as u8); fr.coll_data.floor.index = (rnd() % 8) as i32;
    fr.facing_dir = rg(); fr.cur_pos.x = rw(); fr.cur_pos.y = rw(); fr.cur_pos.z = rg();
    fr.self_vel.x = rg(); fr.self_vel.y = rw(); fr.self_vel.z = rg(); fr.gr_vel = rw();
    fr.input.lstick[0].x = rg(); fr.input.lstick[0].y = rg();
    fr.active_timer.lstick.x = (rnd() % 256) as u8; fr.active_timer.lstick.y = (rnd() % 256) as u8;
    fr.input.held_buttons[0] = rnd(); fr.x618_player_id = (rnd() % 4) as u8; fr.player_id = (rnd() % 6) as _;
    fr.ground_or_air = (rnd() & 1) as _; fr.dmg.x191C = rw(); fr.dmg.facing_dir = rg();
    fr.co_attrs.clank_animation_length = rw(); fr.co_attrs.passiveceil_vel_x = rg();
    fr.co_attrs.passivewall_vel_x = rg(); fr.co_attrs.wall_jump_horizontal_velocity = rg();
    fr.co_attrs.wall_jump_vertical_velocity = rw(); fr.co_attrs.gravity = rg();
    fr.co_attrs.terminal_velocity = rw(); fr.co_attrs.aerial_friction = rg();
    fr.co_attrs.run_animation_scaling = rw(); fr.co_attrs.dash_accel_mul = rg();
    fr.co_attrs.dash_accel_base = rg();
    if fr.co_attrs.dash_accel_base.is_nan() { fr.co_attrs.dash_accel_base = 0.5; } /* NaN+NaN picks an operand by codegen order */
    fr.co_attrs.dash_max_velocity = rw(); fr.co_attrs.ground_friction = rg();
    fr.coll_data.ecb.top.y = rw(); fr.coll_data.ecb.left.x = rg(); fr.coll_data.ecb.left.y = rw();
    fr.coll_data.ecb.right.x = rg(); fr.coll_data.ecb.right.y = rw();
    fr.x68C_transNPos.y = rw(); fr.x68C_transNPos.z = rg();
    fr.x67E = (rnd() % 20) as u8; fr.x1969_walljumpUsed = (rnd() & 1) as u8; fr.cmd_vars[0] = rnd() & 1;
    fr.motion_id = if rnd() & 1 != 0 { ftCo_MS_PassiveWall as _ } else { (rnd() % 400) as _ };
    unsafe {
        fr.anon_2210.anon_0.set_throw_flags_b0((rnd() & 1) as u8);
        fr.anon_2210.anon_0.set_throw_flags_b3((rnd() & 1) as u8);
    }
    fr.cur_anim_frame = rw(); fr.frame_speed_mul = rg();
    let mv = &mut fr.mv as *mut _ as *mut u32;
    unsafe {
        for i in 0..2 {
            *mv.add(i) = if rnd() & 1 != 0 { bits(rg()) } else { rnd() % 3 };
        }
        *mv.add(2) = rnd() & 1;
        *mv.add(3) = rnd() % 4;
    }
}

pub fn state_hash() -> u32 {
    let fr = f();
    let mut h = Hs::new();
    h.m(bits(fr.self_vel.x)); h.m(bits(fr.self_vel.y)); h.m(bits(fr.self_vel.z)); h.m(bits(fr.gr_vel));
    h.m(bits(fr.cur_pos.x)); h.m(bits(fr.cur_pos.y)); h.m(bits(fr.cur_pos.z)); h.m(bits(fr.facing_dir));
    h.m(fr.active_timer.lstick.x as u32); h.m(fr.active_timer.lstick.y as u32); h.m(fr.cmd_vars[0]);
    unsafe {
        h.m(fr.anon_2210.anon_0.throw_flags_b0() as u32);
        h.m(fr.anon_2210.anon_0.throw_flags_b3() as u32);
    }
    let mv = &fr.mv as *const _ as *const u32;
    for i in 0..4 { h.m(unsafe { *mv.add(i) }); }
    h.0
}
