//! Hand port, round 2: the field logic of src/melee/ft/ftcommon.c (lines 246-256, 541-620,
//! 672-686, 777-827, 993-1046, 1302-1309, 1325-1355, 1379-1382, 1581-1584, 1596-1666,
//! 1808-2024): jumps/ECB, grab mash, velocity setters, rumble triggers, held-item scale, stick
//! activity. Same rules as `ftcommon` (C operation order, `ABS` = `abs_c`, gobj -> fighter).
//! Fused sites follow the shipping build (`tools/rust-port/fma_sites.py src/melee/ft/ftcommon.c`):
//! 8007ED50, 8007EE0C, 8007FDA0, 80080174 have one f32 fmuladd each -> `fmadds`.
//! Calls out of the ported set go through `crate::hooks`.
//! Checked against the C (cut verbatim, built with the shipping contraction) by
//! tests/ftcommon2_ref.rs.

use crate::ftcommon::abs_c;
use crate::hooks as h;
use crate::hooks::{common_data, ground_friction_mul};
use crate::types::{
    Fighter, Ft_Kind_Nana, Ft_Kind_Popo, Ft_Kind_Sandbag, GA_Air, GA_Ground, HSD_GObjEvent, HSD_GObjInteraction,
    MtxPtr, Vec3,
};
use melee_core::fp::fmadds;

/// `CollData_X130_Locked` (mp/forward.h).
const COLL_X130_LOCKED: u32 = 1 << 4;
/// `HSD_PAD_AB | HSD_PAD_XY | HSD_PAD_LR` (baselib/controller.h).
const PAD_ABXYLR: u32 = (1 << 8) | (1 << 9) | (1 << 10) | (1 << 11) | (1 << 31);

#[inline(always)]
fn ids(fp: &Fighter) -> (i32, i32) {
    (fp.player_id as i32, fp.is_sub_fighter() as i32)
}

/// `ftCommon_SandbagGetKnockbackDeaccelX`: `((float*) fp->dat_attrs)[0]`.
pub fn sandbag_get_knockback_deaccel_x(fp: &Fighter) -> f32 {
    assert!(fp.kind == Ft_Kind_Sandbag, "ftcommon.c:299");
    unsafe { *(fp.dat_attrs as *const f32) }
}

/// `ftCommon_SandbagGetKnockbackDeaccelY`: `((float*) fp->dat_attrs)[1]`.
pub fn sandbag_get_knockback_deaccel_y(fp: &Fighter) -> f32 {
    assert!(fp.kind == Ft_Kind_Sandbag, "ftcommon.c:308");
    unsafe { *(fp.dat_attrs as *const f32).add(1) }
}

pub fn unlock_ecb(fp: &mut Fighter) {
    fp.ecb_lock = 0;
    fp.coll_data.x130_flags &= !COLL_X130_LOCKED;
}

/// `ftCommon_8007D5D4`: become airborne (one jump used, ECB locked 10 frames).
pub fn ftcommon_8007d5d4(fp: &mut Fighter) {
    let was_ground = fp.ground_or_air == GA_Ground;
    fp.ground_or_air = GA_Air;
    fp.gr_vel = 0.0;
    fp.x98_atk_shield_kb.z = 0.0;
    fp.cur_pos.z = 0.0;
    fp.x74_self_accel.y = 0.0;
    fp.x1968_jumpsUsed = 1;
    fp.ecb_lock = 10;
    fp.coll_data.x130_flags |= COLL_X130_LOCKED;
    if was_ground {
        h::geno_ground_edge(fp, 0);
    }
}

/// `ftCommon_8007D60C`: become airborne with every jump used (ECB locked 5 frames).
pub fn ftcommon_8007d60c(fp: &mut Fighter) {
    let was_ground = fp.ground_or_air == GA_Ground;
    if fp.x2227_b0() != 0 && fp.x1968_jumpsUsed <= 1 {
        let (s, u) = ids(fp);
        h::pl_8003fc44(s, u);
    }
    fp.ground_or_air = GA_Air;
    fp.gr_vel = 0.0;
    fp.x74_self_accel.y = 0.0;
    fp.x1968_jumpsUsed = fp.co_attrs.max_jumps as u8;
    fp.ecb_lock = 5;
    fp.coll_data.x130_flags |= COLL_X130_LOCKED;
    if was_ground {
        h::geno_ground_edge(fp, 0);
    }
}

pub fn use_all_jumps(fp: &mut Fighter) {
    fp.x1968_jumpsUsed = fp.co_attrs.max_jumps as u8;
}

/// `ftCommon_8007D780`.
pub fn ftcommon_8007d780(fp: &mut Fighter) {
    if fp.ground_or_air != GA_Air {
        return;
    }
    if h::un_803224dc(fp.x8_spawnNum, fp.cur_pos.x, fp.dmg.x18A4_knockbackMagnitude) {
        fp.dmg.x18A4_knockbackMagnitude = 0.0;
    }
    if fp.x2227_b0() != 0 && fp.x1968_jumpsUsed <= 1 {
        let (s, u) = ids(fp);
        h::pl_8003fc44(s, u);
    }
}

/// `ftCommon_InitGrab(fp, bool arg1, timer)`; `bool` is MSL `int`, stored into a `:1`.
pub fn init_grab(fp: &mut Fighter, arg1: i32, timer: f32) {
    fp.grab_timer = timer;
    fp.x1A51 = 0;
    fp.x1A50 = 0;
    fp.set_x2224_b6(arg1 as u8);
    if fp.x2224_b6() != 0 {
        fp.x1A52 = 0;
        fp.x1A53 = h::grab_mash_shake().x4 as u8;
    }
}

/// Stick-direction latch shared by GrabMash and 8007FFD8.
#[inline(always)]
fn mash_dir(x: f32, y: f32, dx: &mut i8, dy: &mut i8) {
    let t = common_data().x308;
    if x < -t {
        *dx = -1;
    }
    if x > t {
        *dx = 1;
    }
    if y < -t {
        *dy = -1;
    }
    if y > t {
        *dy = 1;
    }
}

pub fn grab_mash(fp: &mut Fighter, arg1: f32) -> bool {
    let mut result = false;
    if fp.input.pressed_buttons & PAD_ABXYLR != 0 {
        fp.grab_timer -= arg1;
        result = true;
    }
    let (r5, r6) = (fp.x1A50, fp.x1A51);
    let (mut dx, mut dy) = (r5, r6);
    mash_dir(fp.input.lstick[0].x, fp.input.lstick[0].y, &mut dx, &mut dy);
    fp.x1A50 = dx;
    fp.x1A51 = dy;
    if r5 != fp.x1A50 || r6 != fp.x1A51 {
        fp.grab_timer -= arg1;
        result = true;
    }
    if result && fp.x2224_b6() != 0 {
        fp.set_x2224_b5(1);
        fp.x1A52 = fp.x1A52.wrapping_add(1);
        if fp.x1A52 >= fp.x1A53 {
            fp.x1A52 = 0;
        }
    } else {
        fp.set_x2224_b5(0);
    }
    let (s, u) = ids(fp);
    h::pl_800402d0(s, u, result as i32);
    result
}

/// `ftCommon_8007E2A4`: shield knockback along the floor (the C returns the gobj).
pub fn ftcommon_8007e2a4(fp: &mut Fighter) {
    fp.x98_atk_shield_kb.x = fp.coll_data.floor.normal.y * fp.xF4_ground_attacker_shield_kb_vel;
    fp.x98_atk_shield_kb.y = -fp.coll_data.floor.normal.x * fp.xF4_ground_attacker_shield_kb_vel;
}

/// `ftCommon_8007E2D0`.
pub fn ftcommon_8007e2d0(
    fp: &mut Fighter,
    arg1: i16,
    grab_cb: HSD_GObjEvent,
    unk_cb: HSD_GObjEvent,
    grabbed_cb: HSD_GObjInteraction,
) {
    fp.set_x221E_b6(1);
    fp.x1A68 = arg1 as u16;
    fp.grab_cb = grab_cb;
    fp.grabbed_cb = grabbed_cb;
    fp.x2194 = unk_cb;
}

pub fn ftcommon_8007e2f4(fp: &mut Fighter, val: i16) {
    fp.x1A6A = val as u16;
}

/// `ftCommon_8007E2FC`: zero every velocity and acceleration.
pub fn ftcommon_8007e2fc(fp: &mut Fighter) {
    let z = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    fp.xE4_ground_accel_1 = 0.0;
    fp.xE8_ground_accel_2 = 0.0;
    fp.x74_self_accel = z;
    fp.gr_vel = 0.0;
    fp.self_vel = z;
    fp.xF0_ground_kb_vel = 0.0;
    fp.x8c_kb_vel = z;
    fp.xF4_ground_attacker_shield_kb_vel = 0.0;
    fp.x98_atk_shield_kb = z;
    fp.xD4_unk_vel = z;
}

/// `ftCommon_8007EBAC`: start rumble `arg1` on this fighter's port.
pub fn ftcommon_8007ebac(fp: &mut Fighter, arg1: u32, arg2: u32) {
    let (s, u) = ids(fp);
    if h::player_8003544c(s, u) && fp.x221F_b3() == 0 && fp.x2224_b2() == 0 {
        h::lb_rumble_start(fp.x618_player_id, arg1.wrapping_add(2) as i32, arg1 as i32, arg2 as i32);
    }
}

pub fn ftcommon_8007ecd4(fp: &mut Fighter, arg1: i32) {
    let (s, u) = ids(fp);
    if h::player_8003544c(s, u) {
        h::hsd_pad_rumble_remove_id(fp.x618_player_id, arg1.wrapping_add(2));
    }
}

pub fn ftcommon_8007ed2c(fp: &mut Fighter) {
    h::lb_rumble_stop(fp.x618_player_id);
}

/// `ftCommon_8007ED50`: `arg1 * x138 + x13C` is one fmuladd in the shipping build -> fmadds.
pub fn ftcommon_8007ed50(fp: &mut Fighter, arg1: i32) {
    let cd = common_data();
    let tmp = fmadds(arg1 as f32, cd.x138, cd.x13C);
    let val2 = tmp as i32;
    if (tmp as i32) < 1 {
        return;
    }
    ftcommon_8007ebac(fp, 5, val2 as u32);
}

/// `ftCommon_8007EE0C`: `arg1 * xEC + xF0` fused (fmadds), as 8007ED50.
pub fn ftcommon_8007ee0c(fp: &mut Fighter, arg1: i32) {
    let cd = common_data();
    let tmp = fmadds(arg1 as f32, cd.xEC, cd.xF0);
    let val2 = tmp as i32;
    if (tmp as i32) < 1 {
        return;
    }
    ftcommon_8007ebac(fp, 10, val2 as u32);
}

pub fn ftcommon_8007efc0(fp: &mut Fighter, val: u32) {
    fp.x209A = val as u16;
}

pub fn get_model_scale(fp: &Fighter) -> f32 {
    fp.x34_scale.y * fp.co_attrs.model_scaling
}

/// `ftCommon_8007F76C`: point the ground and air x velocity along the facing direction.
pub fn ftcommon_8007f76c(fp: &mut Fighter) {
    fp.gr_vel = fp.facing_dir * abs_c(fp.gr_vel);
    fp.self_vel.x = fp.facing_dir * abs_c(fp.self_vel.x);
}

/// `ftCommon_8007F7B4`: as 8007F76C, against the facing direction.
pub fn ftcommon_8007f7b4(fp: &mut Fighter) {
    fp.gr_vel = -fp.facing_dir * abs_c(fp.gr_vel);
    fp.self_vel.x = -fp.facing_dir * abs_c(fp.self_vel.x);
}

/// `ftCommon_8007F804`: `x44_mtx` when the z scale is not 1, else NULL.
pub fn ftcommon_8007f804(fp: &mut Fighter) -> MtxPtr {
    if fp.x34_scale.z != 1.0 {
        return fp.x44_mtx.as_mut_ptr();
    }
    core::ptr::null_mut()
}

/// `ftCommon_8007F8B4`: position plus `xD4_unk_vel`.
pub fn ftcommon_8007f8b4(fp: &Fighter) -> Vec3 {
    Vec3 {
        x: fp.cur_pos.x + fp.xD4_unk_vel.x,
        y: fp.cur_pos.y + fp.xD4_unk_vel.y,
        z: fp.cur_pos.z + fp.xD4_unk_vel.z,
    }
}

/// `my_fminf` (ftcommon.c static inline): `a > b ? b : a`.
#[inline(always)]
fn my_fminf(a: f32, b: f32) -> f32 {
    if a > b {
        b
    } else {
        a
    }
}

/// `x710 * x2024 + x708`: one fmuladd in the shipping build (8007FDA0, 80080174) -> fmadds.
#[inline(always)]
fn held_item_scale_raw(fp: &Fighter) -> f32 {
    let cd = common_data();
    fmadds(cd.x710, fp.x2024 as f32, cd.x708)
}

/// `ftCommon_8007FDA0`: scale the held (x1980) item.
pub fn ftcommon_8007fda0(fp: &mut Fighter) {
    let phi_f31 = my_fminf(held_item_scale_raw(fp), common_data().x70C);
    let temp_f1 = 1.0f32 / phi_f31;
    let mut sp20 = fp.co_attrs.x130;
    sp20.x *= temp_f1;
    sp20.y *= temp_f1;
    sp20.z *= temp_f1;
    h::it_8029a8f4(fp.x1980, &sp20);
    // `temp_r30[1].x`: the float after the Vec3 at co_attrs+0x130, i.e. co_attrs.x13C
    h::it_8029a89c(fp.x1980, phi_f31 * fp.co_attrs.x13C * fp.x34_scale.y);
}

/// `ftCommon_8007FF74`.
pub fn ftcommon_8007ff74(fp: &mut Fighter) {
    fp.x1980 = core::ptr::null_mut();
    ftcommon_8007ecd4(fp, 2);
    let (s, u) = ids(fp);
    h::pl_80040460(s, u);
}

/// `ftCommon_8007FFD8`: mash out of the x1980 hold; `x2018` is an int, so `x2018 -= f` is
/// `(int) ((float) x2018 - f)` (saturating, as gwtool lowers it).
pub fn ftcommon_8007ffd8(fp: &mut Fighter, arg8: f32) -> bool {
    let mut r = false;
    if fp.input.pressed_buttons & PAD_ABXYLR != 0 {
        fp.x2018 = (fp.x2018 as f32 - arg8) as i32;
        r = true;
    }
    let (b0, b1) = (fp.x2020, fp.x2021);
    let (mut dx, mut dy) = (b0, b1);
    mash_dir(fp.input.lstick[0].x, fp.input.lstick[0].y, &mut dx, &mut dy);
    fp.x2020 = dx;
    fp.x2021 = dy;
    if b0 != fp.x2020 || b1 != fp.x2021 {
        fp.x2018 = (fp.x2018 as f32 - arg8) as i32;
        r = true;
    }
    let (s, u) = ids(fp);
    h::pl_800402d0(s, u, r as i32);
    r
}

/// `ftCommon_80080144`: Ice Climbers in costume 2 or higher.
pub fn ftcommon_80080144(fp: &Fighter) -> bool {
    (fp.kind == Ft_Kind_Popo || fp.kind == Ft_Kind_Nana) && fp.x619_costume_id >= 2
}

/// `ftCommon_80080174`: rescale the head (x197C) and held (x1980) items.
pub fn ftcommon_80080174(fp: &mut Fighter) {
    if !fp.x197C.is_null() {
        h::it_80294e78(fp.x197C, fp.x34_scale.y * fp.co_attrs.x12C);
    }
    if !fp.x1980.is_null() {
        let phi_f2 = my_fminf(held_item_scale_raw(fp), common_data().x70C);
        h::it_8029a89c(fp.x1980, phi_f2 * fp.co_attrs.x13C * fp.x34_scale.y);
    }
}

/// `ftCommon_8008021C`: sum the hit-shake offsets into dmg.x18B8/x18BC.
pub fn ftcommon_8008021c(fp: &mut Fighter) {
    fp.dmg.x18BC = 0.0;
    fp.dmg.x18B8 = 0.0;
    if let Some(s) = h::ftco_80090690(fp) {
        fp.dmg.x18B8 += s.x;
        fp.dmg.x18BC += s.y;
    }
    if fp.x2224_b5() != 0 {
        let s = unsafe { *h::grab_mash_shake().x0.add(fp.x1A52 as usize) };
        fp.dmg.x18B8 += s.x;
        fp.dmg.x18BC += s.y;
    }
    if let Some(s) = h::ftco_800deee8(fp) {
        fp.dmg.x18B8 += s.x;
        fp.dmg.x18BC += s.y;
    }
}

/// `ftCommon_8008031C`: count stick/trigger activity for the player stats.
pub fn ftcommon_8008031c(fp: &mut Fighter) {
    let cd = common_data();
    let t = fp.activity_timer;
    if (abs_c(fp.input.lstick[0].x) >= cd.x7B8 && (t.lstick.x as f32) < cd.x7C0)
        || (abs_c(fp.input.lstick[0].y) >= cd.x7B8 && (t.lstick.y as f32) < cd.x7C0)
    {
        let (s, u) = ids(fp);
        h::player_update_joystick_count(s, u);
        fp.activity_timer.lstick.y = 0xFE;
        fp.activity_timer.lstick.x = 0xFE;
    }
    if abs_c(fp.input.triggers[0]) >= cd.x7BC && (fp.activity_timer.trigger as f32) < cd.x7C0 {
        let (s, u) = ids(fp);
        h::player_update_joystick_count(s, u);
        fp.activity_timer.trigger = 0xFE;
    }
}

pub fn ftcommon_80080460(fp: &mut Fighter) {
    fp.set_x2225_b6(1);
}

pub fn ftcommon_80080474(fp: &mut Fighter) {
    let v = fp.x2225_b5();
    fp.set_x2225_b6(v);
}

pub fn ftcommon_80080484(fp: &mut Fighter) {
    if fp.smash_attrs.x2138_smashSinceHitbox == -1.0 {
        fp.smash_attrs.x2138_smashSinceHitbox = 0.0;
    }
}

pub fn ftcommon_800804a0(fp: &mut Fighter, arg8: f32) {
    let mut phi_f31 = arg8;
    let t = ground_friction_mul(fp);
    if t < 1.0 {
        phi_f31 *= t;
    }
    fp.xE8_ground_accel_2 = phi_f31;
}

pub fn ftcommon_800804ec(fp: &Fighter) -> f32 {
    fp.x40 * fp.x34_scale.x
}

pub fn ftcommon_800804fc(fp: &mut Fighter) {
    if fp.ground_or_air == GA_Ground {
        fp.dmg.x18c4_source_ply = 6;
        fp.dmg.x18C8 = -1;
    }
}
