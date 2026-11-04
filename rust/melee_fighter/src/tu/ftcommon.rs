//! src/melee/ft/ftcommon.c: the functions not already ported in the older modules
//! (crate::ftcommon / ftcommon2 / ftcommon3 / ftlib hold the other ~110, verified there):
//! `ftCommon_8007E5AC` (ground-slope Z rotation) and `ftCommon_8007EFC8` (Zelda/Sheik
//! transform: copy the active fighter's state onto the partner). The TARGET_PC disc-free
//! fixture `ftCommon_PcFloorGuardTest` / `ftCommon_PcFixtureFall` is a C self-test of the
//! private `ftcommon3::pc_floor_guard` and is not ported (its Rust counterpart is that
//! module's test). No fused sites.
#![allow(non_snake_case, static_mut_refs)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use crate::ftcommon as fc;
#[allow(unused_imports)]
use crate::ftcommon_c::{G as G_ftcommon_c, fpg as fpg_ftcommon_c};
#[allow(unused_imports)]
use crate::ftcommon::{clamp_ground_vel};
#[allow(unused_imports)]
use crate::ftcommon2 as c2;
#[allow(unused_imports)]
use core::sync::atomic::{AtomicU32, Ordering};
#[allow(unused_imports)]
use crate::ftcommon3::{FT_PARASOL_NONE, G as G_ftcommon3, PARASOL_TABLE_2, PARASOL_TABLE_3_SET, PARASOL_TABLE_4, apply_head_item_cb, bone4, fp, ft_get_parasol_status, holds_sword, ids as ids_ftcommon3, nudge_dx, pc_floor_guard, pc_floor_valid, same_or_adjacent_floor, sgn_c, x594_b0};
#[allow(unused_imports)]
use crate::ftcommon::abs_c;
#[allow(unused_imports)]
use crate::hooks as h;
#[allow(unused_imports)]
use crate::hooks::{common_data, ground_friction_mul};
#[allow(unused_imports)]
use crate::types::{
    Fighter, Ft_Kind_Nana, Ft_Kind_Popo, Ft_Kind_Sandbag, GA_Air, GA_Ground, HSD_GObjEvent, HSD_GObjInteraction,
    MtxPtr, Vec3,
};
#[allow(unused_imports)]
use melee_core::fp::fmadds;
#[allow(unused_imports)]
use crate::ftcommon2::{COLL_X130_LOCKED, PAD_ABXYLR, held_item_scale_raw, ids, mash_dir};
use crate::types::*;

pub type G = *mut HSD_GObj;

pub mod ext {
    use super::G;
    use crate::types::*;
    macro_rules! ext {
        ($($stat:ident $name:ident ($($a:ident: $t:ty),*) $(-> $r:ty)?;)*) => {$(
            pub static mut $stat: Option<fn($($t),*) $(-> $r)?> = None;
            pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { $stat } {
                    Some(f) => f($($a),*),
                    None => melee_core::unported!(concat!("TODO: ", stringify!($name), " not ported")),
                }
            }
        )*};
    }
    ext! {
        PLAYER_GET_ENTITY_AT_INDEX Player_GetEntityAtIndex(slot: i32, i: i32) -> G;
        PLAYER_SWAP_TRANSFORMED_STATES Player_SwapTransformedStates(slot: i32, a: i32, b: i32);
        FIGHTER_UNK_INIT_RESET_80067C98 Fighter_UnkInitReset_80067C98(fp: *mut Fighter);
        PLAYER_SET_HP_BY_INDEX Player_SetHPByIndex(slot: i32, sub: i32, hp: i32);
        FTCOLL_8007B7FC ftColl_8007B7FC(fp: *mut Fighter, arg: i32);
        FTCOLL_8007B760 ftColl_8007B760(g: G, arg: i32);
        FTLIB_SET_SCALE ftLib_SetScale(g: G, s: f32);
        FTCO_800C8348 ftCo_800C8348(g: G, timer: i32, health: i32);
        FTCO_800C8540 ftCo_800C8540(g: G);
        IT_8026B9A8 it_8026B9A8(item: G, owner: G, bone: i32);
        IT_802950D4 it_802950D4(item: G, b: bool);
        FT_800849EC ft_800849EC(src: *mut Fighter, dst: *mut Fighter);
        FT_80081C88 ft_80081C88(g: G, scl_y: f32);
        FTPICKUPITEM_80094818 ftpickupitem_80094818(g: G, b: i32);
        UN_80322314 un_80322314();
    }
    /// Override for `ftPartSetRotZ` (unset = the port in `tu::ftparts`).
    pub static mut FTPART_SET_ROT_Z: Option<fn(&Fighter, i32, f32)> = None;
    pub fn ftPartSetRotZ(f: &Fighter, part: i32, v: f32) { unsafe { crate::tu::ftparts::ftPartSetRotZ(f as _, part as _, v as _); } }
}

#[inline(always)]
fn fpg<'a>(g: G) -> &'a mut Fighter {
    unsafe { &mut *((*g).user_data as *mut Fighter) }
}

/// `ftCommon_8007E5AC`: tilt part 0 to the floor's slope.
pub fn ftcommon_8007e5ac(fp: &mut Fighter) {
    melee_core::ct!("ftcommon_8007e5ac");
    let n = fp.coll_data.floor.normal;
    let tmp = -melee_runtime::lbtrigf::atan2f(n.x, n.y);
    assert!(fp.ground_or_air as i32 == GA_Ground as i32, "fp->ground_or_air == GA_Ground");
    ext::ftPartSetRotZ(fp, 0, tmp);
}

/// `ftCommon_8007EFC8`: hand the match over to the transform partner (Zelda <-> Sheik), then
/// put this fighter to sleep and run `arg1` on the partner.
pub fn ftcommon_8007efc8(gobj: G, arg1: fn(G)) {
    melee_core::ct!("ftcommon_8007efc8");
    let src = fpg(gobj);
    let dst_gobj = ext::Player_GetEntityAtIndex(src.player_id as i32, 1);
    let dst = fpg(dst_gobj);
    ext::Player_SwapTransformedStates(src.player_id as i32, src.is_sub_fighter() as i32, dst.is_sub_fighter() as i32);
    let tmp_bit = src.is_sub_fighter();
    src.set_is_sub_fighter(dst.is_sub_fighter());
    dst.set_is_sub_fighter(tmp_bit);
    ext::Fighter_UnkInitReset_80067C98(dst);
    crate::tu::ftcommon::ftcommon_8007e2fc(fpg(dst_gobj));
    let (src, dst) = (fpg(gobj), fpg(dst_gobj));
    dst.cur_pos = src.cur_pos;
    dst.prev_pos = src.prev_pos;
    dst.pos_delta = src.pos_delta;
    dst.facing_dir = src.facing_dir;
    dst.dmg.x1830_percent = src.dmg.x1830_percent;
    ext::Player_SetHPByIndex(dst.player_id as i32, dst.is_sub_fighter() as i32, dst.dmg.x1830_percent as i32);
    let (src, dst) = (fpg(gobj), fpg(dst_gobj));
    dst.dmg.x18F0 = src.dmg.x18F0;
    dst.self_vel = src.self_vel;
    dst.ground_or_air = src.ground_or_air;
    dst.gr_vel = src.gr_vel;
    dst.xF0_ground_kb_vel = src.xF0_ground_kb_vel;
    dst.xF4_ground_attacker_shield_kb_vel = src.xF4_ground_attacker_shield_kb_vel;
    dst.xF8_playerNudgeVel.x = src.xF8_playerNudgeVel.x;
    dst.xF8_playerNudgeVel.y = src.xF8_playerNudgeVel.y;
    dst.input = src.input;
    dst.active_timer.lstick.x = src.active_timer.lstick.x;
    dst.active_timer.lstick.y = src.active_timer.lstick.y;
    dst.active_timer.trigger = src.active_timer.trigger;
    dst.active_sticky.lstick.x = src.active_sticky.lstick.x;
    dst.active_sticky.lstick.y = src.active_sticky.lstick.y;
    dst.active_sticky.trigger = src.active_sticky.trigger;
    dst.activity_timer.lstick.x = src.activity_timer.lstick.x;
    dst.activity_timer.lstick.y = src.activity_timer.lstick.y;
    dst.activity_timer.trigger = src.activity_timer.trigger;
    dst.x685 = src.x685;
    dst.x686 = src.x686;
    dst.x687 = src.x687;
    dst.x688 = src.x688;
    dst.x689 = src.x689;
    dst.x68A = src.x68A;
    dst.x68B = src.x68B;
    dst.x67C = src.x67C;
    dst.x67D = src.x67D;
    dst.x67E = src.x67E;
    dst.x681 = src.x681;
    dst.x682 = src.x682;
    dst.x67F = src.x67F;
    dst.x680 = src.x680;
    dst.x683 = src.x683;
    dst.x684 = src.x684;
    dst.dmg.x18c4_source_ply = src.dmg.x18c4_source_ply;
    dst.dmg.x18C8 = src.dmg.x18C8;
    dst.set_x221F_b5(src.x221F_b5() as _);
    dst.dmg.x18CC = src.dmg.x18CC;
    dst.dmg.x18D0 = src.dmg.x18D0;
    dst.dmg.x18d4 = src.dmg.x18d4;
    dst.dmg.x18ec_instancehitby = src.dmg.x18ec_instancehitby;
    dst.x21FC_flag = src.x21FC_flag;
    dst.set_x2221_b7(src.x2221_b7() as _);
    dst.set_x2221_b6(src.x2221_b6() as _);
    dst.set_x2221_b4(src.x2221_b4() as _);
    dst.set_x2221_b5(src.x2221_b5() as _);
    dst.x2104 = src.x2104;
    dst.dmg.x1910 = src.dmg.x1910;
    dst.x1968_jumpsUsed = src.x1968_jumpsUsed;
    dst.x1969_walljumpUsed = src.x1969_walljumpUsed;
    dst.shield_health = src.shield_health;
    dst.bury_timer_2 = src.bury_timer_2;
    dst.x2330 = src.x2330;
    dst.x2338 = src.x2338;
    dst.set_x2224_b2(src.x2224_b2() as _);
    if src.x221D_b6() != 0 {
        ext::ftColl_8007B7FC(dst, src.x2004);
        super::ftcolanim::ftco_800c0358(fpg(gobj), fpg(dst_gobj), 0x6B);
        crate::ft0877::ft_800880d8(fpg(gobj));
    } else {
        super::ftcolanim::ftco_800c0200(dst, 9);
    }
    let (src, dst) = (fpg(gobj), fpg(dst_gobj));
    src.x198C = 2;
    if src.x1990 != 0 {
        ext::ftColl_8007B760(dst_gobj, src.x1990);
    }
    let (src, dst) = (fpg(gobj), fpg(dst_gobj));
    dst.set_x2220_b5(src.x2220_b5() as _);
    dst.set_x2220_b6(src.x2220_b6() as _);
    dst.x2008 = src.x2008;
    ext::ftLib_SetScale(dst_gobj, src.x34_scale.y);
    let src = fpg(gobj);
    if src.is_metal() != 0 {
        ext::ftCo_800C8348(dst_gobj, src.metal_timer, src.metal_health);
        ext::ftCo_800C8540(gobj);
    }
    let src = fpg(gobj);
    if src.x2226_b4() != 0 {
        if src.x2226_b7() == 0 {
            super::ft_0c88::ftco_800c88d4(dst_gobj, src.x2030, false);
        }
        super::ft_0c88::ftco_800c8a64(gobj);
    }
    let (src, dst) = (fpg(gobj), fpg(dst_gobj));
    dst.set_x221E_b4(src.x221E_b4() as _);
    if !src.x197C.is_null() {
        let bone = unsafe { (*(*dst.ft_data).x8).x12 } as i32;
        ext::it_8026B9A8(src.x197C, dst_gobj, bone);
        ext::it_802950D4(fpg(gobj).x197C, false);
        let src = fpg(gobj);
        crate::ftcommon3::ftcommon_8007f948(dst_gobj, src.x197C, src.x2014);
        crate::ftcommon3::ftcommon_8007fa00(gobj);
    } else {
        dst.x197C = core::ptr::null_mut();
    }
    let (src, dst) = (fpg(gobj), fpg(dst_gobj));
    if !src.x1980.is_null() {
        let bone = unsafe { (*(*dst.ft_data).x8).x12 } as i32;
        ext::it_8026B9A8(src.x1980, dst_gobj, bone);
        let src = fpg(gobj);
        crate::ftcommon3::ftcommon_8007fe84(dst_gobj, src.x1980, src.x2018, src.x2024 as f32);
        crate::ftcommon2::ftcommon_8007fda0(fpg(gobj));
    } else {
        dst.x1980 = core::ptr::null_mut();
    }
    super::ftchangeparam::ftco_800d105c(dst_gobj);
    super::ftcamera::ftcamera_80076064(fpg(dst_gobj));
    ext::ft_800849EC(fpg(gobj), fpg(dst_gobj));
    ext::ft_80081C88(dst_gobj, fpg(gobj).x34_scale.y);
    let (src, dst) = (fpg(gobj), fpg(dst_gobj));
    if !src.item_gobj.is_null() {
        dst.item_gobj = src.item_gobj;
        dst.set_x221E_b3(src.x221E_b3() as _);
        ext::ftpickupitem_80094818(dst_gobj, 1);
        let bone = unsafe { (*(*fpg(dst_gobj).ft_data).x8).x10 } as i32;
        ext::it_8026B9A8(fpg(gobj).item_gobj, dst_gobj, bone);
    }
    ext::un_80322314();
    super::ftcolanim::ftco_800bfd04(gobj);
    arg1(dst_gobj);
}

// ---- folded from ftcommon2.rs (consolidation steps 21/22) ----
/// `ftCommon_8007D5D4`: become airborne (one jump used, ECB locked 10 frames).
pub fn ftcommon_8007d5d4(fp: &mut Fighter) {
    melee_core::ct!("ftcommon_8007d5d4");
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
    melee_core::ct!("ftcommon_8007d60c");
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

/// `ftCommon_8007D780`.
pub fn ftcommon_8007d780(fp: &mut Fighter) {
    melee_core::ct!("ftcommon_8007d780");
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

/// `ftCommon_8007E2A4`: shield knockback along the floor (the C returns the gobj).
pub fn ftcommon_8007e2a4(fp: &mut Fighter) {
    melee_core::ct!("ftcommon_8007e2a4");
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
    melee_core::ct!("ftcommon_8007e2d0");
    fp.set_x221E_b6(1);
    fp.x1A68 = arg1 as u16;
    fp.grab_cb = grab_cb;
    fp.grabbed_cb = grabbed_cb;
    fp.x2194 = unk_cb;
}

pub fn ftcommon_8007e2f4(fp: &mut Fighter, val: i16) {
    melee_core::ct!("ftcommon_8007e2f4");
    fp.x1A6A = val as u16;
}

/// `ftCommon_8007E2FC`: zero every velocity and acceleration.
pub fn ftcommon_8007e2fc(fp: &mut Fighter) {
    melee_core::ct!("ftcommon_8007e2fc");
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
    melee_core::ct!("ftcommon_8007ebac");
    let (s, u) = ids(fp);
    if h::player_8003544c(s, u) && fp.x221F_b3() == 0 && fp.x2224_b2() == 0 {
        h::lb_rumble_start(fp.x618_player_id, arg1.wrapping_add(2) as i32, arg1 as i32, arg2 as i32);
    }
}

pub fn ftcommon_8007ecd4(fp: &mut Fighter, arg1: i32) {
    melee_core::ct!("ftcommon_8007ecd4");
    let (s, u) = ids(fp);
    if h::player_8003544c(s, u) {
        h::hsd_pad_rumble_remove_id(fp.x618_player_id, arg1.wrapping_add(2));
    }
}

pub fn ftcommon_8007ed2c(fp: &mut Fighter) {
    melee_core::ct!("ftcommon_8007ed2c");
    h::lb_rumble_stop(fp.x618_player_id);
}

/// `ftCommon_8007ED50`: `arg1 * x138 + x13C` is one fmuladd in the shipping build -> fmadds.
pub fn ftcommon_8007ed50(fp: &mut Fighter, arg1: i32) {
    melee_core::ct!("ftcommon_8007ed50");
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
    melee_core::ct!("ftcommon_8007ee0c");
    let cd = common_data();
    let tmp = fmadds(arg1 as f32, cd.xEC, cd.xF0);
    let val2 = tmp as i32;
    if (tmp as i32) < 1 {
        return;
    }
    ftcommon_8007ebac(fp, 10, val2 as u32);
}

pub fn ftcommon_8007efc0(fp: &mut Fighter, val: u32) {
    melee_core::ct!("ftcommon_8007efc0");
    fp.x209A = val as u16;
}

/// `ftCommon_8007F76C`: point the ground and air x velocity along the facing direction.
pub fn ftcommon_8007f76c(fp: &mut Fighter) {
    melee_core::ct!("ftcommon_8007f76c");
    fp.gr_vel = fp.facing_dir * abs_c(fp.gr_vel);
    fp.self_vel.x = fp.facing_dir * abs_c(fp.self_vel.x);
}

/// `ftCommon_8007F7B4`: as 8007F76C, against the facing direction.
pub fn ftcommon_8007f7b4(fp: &mut Fighter) {
    melee_core::ct!("ftcommon_8007f7b4");
    fp.gr_vel = -fp.facing_dir * abs_c(fp.gr_vel);
    fp.self_vel.x = -fp.facing_dir * abs_c(fp.self_vel.x);
}

/// `ftCommon_8007F804`: `x44_mtx` when the z scale is not 1, else NULL.
pub fn ftcommon_8007f804(fp: &mut Fighter) -> MtxPtr {
    melee_core::ct!("ftcommon_8007f804");
    if fp.x34_scale.z != 1.0 {
        return fp.x44_mtx.as_mut_ptr();
    }
    core::ptr::null_mut()
}

/// `ftCommon_8007F8B4`: position plus `xD4_unk_vel`.
pub fn ftcommon_8007f8b4(fp: &Fighter) -> Vec3 {
    melee_core::ct!("ftcommon_8007f8b4");
    Vec3 {
        x: fp.cur_pos.x + fp.xD4_unk_vel.x,
        y: fp.cur_pos.y + fp.xD4_unk_vel.y,
        z: fp.cur_pos.z + fp.xD4_unk_vel.z,
    }
}

/// `my_fminf` (ftcommon.c static inline): `a > b ? b : a`.
#[inline(always)]
pub fn my_fminf(a: f32, b: f32) -> f32 {
    melee_core::ct!("my_fminf");
    if a > b {
        b
    } else {
        a
    }
}

/// `ftCommon_8007FDA0`: scale the held (x1980) item.
pub fn ftcommon_8007fda0(fp: &mut Fighter) {
    melee_core::ct!("ftcommon_8007fda0");
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
    melee_core::ct!("ftcommon_8007ff74");
    fp.x1980 = core::ptr::null_mut();
    ftcommon_8007ecd4(fp, 2);
    let (s, u) = ids(fp);
    h::pl_80040460(s, u);
}

/// `ftCommon_8007FFD8`: mash out of the x1980 hold; `x2018` is an int, so `x2018 -= f` is
/// `(int) ((float) x2018 - f)` (saturating, as gwtool lowers it).
pub fn ftcommon_8007ffd8(fp: &mut Fighter, arg8: f32) -> bool {
    melee_core::ct!("ftcommon_8007ffd8");
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
    melee_core::ct!("ftcommon_80080144");
    (fp.kind == Ft_Kind_Popo || fp.kind == Ft_Kind_Nana) && fp.x619_costume_id >= 2
}

/// `ftCommon_80080174`: rescale the head (x197C) and held (x1980) items.
pub fn ftcommon_80080174(fp: &mut Fighter) {
    melee_core::ct!("ftcommon_80080174");
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
    melee_core::ct!("ftcommon_8008021c");
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
    melee_core::ct!("ftcommon_8008031c");
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
    melee_core::ct!("ftcommon_80080460");
    fp.set_x2225_b6(1);
}

pub fn ftcommon_80080474(fp: &mut Fighter) {
    melee_core::ct!("ftcommon_80080474");
    let v = fp.x2225_b5();
    fp.set_x2225_b6(v);
}

pub fn ftcommon_80080484(fp: &mut Fighter) {
    melee_core::ct!("ftcommon_80080484");
    if fp.smash_attrs.x2138_smashSinceHitbox == -1.0 {
        fp.smash_attrs.x2138_smashSinceHitbox = 0.0;
    }
}

pub fn ftcommon_800804a0(fp: &mut Fighter, arg8: f32) {
    melee_core::ct!("ftcommon_800804a0");
    let mut phi_f31 = arg8;
    let t = ground_friction_mul(fp);
    if t < 1.0 {
        phi_f31 *= t;
    }
    fp.xE8_ground_accel_2 = phi_f31;
}

pub fn ftcommon_800804ec(fp: &Fighter) -> f32 {
    melee_core::ct!("ftcommon_800804ec");
    fp.x40 * fp.x34_scale.x
}

pub fn ftcommon_800804fc(fp: &mut Fighter) {
    melee_core::ct!("ftcommon_800804fc");
    if fp.ground_or_air == GA_Ground {
        fp.dmg.x18c4_source_ply = 6;
        fp.dmg.x18C8 = -1;
    }
}

// ---- folded from ftcommon3.rs (consolidation steps 21/22) ----
/// `ftCommon_8007D6A4`: land (TARGET_PC: fall instead when the floor is gone).
pub fn ftcommon_8007d6a4(f: &mut Fighter) {
    melee_core::ct!("ftcommon_8007d6a4");
    let was_air = f.ground_or_air == GA_Air;
    let valid = pc_floor_valid(f);
    if pc_floor_guard(f, valid, h::ftco_fall_enter) {
        return;
    }
    if x594_b0(f) != 0 {
        f.self_vel.x = f.x6A4_transNOffset.z * f.facing_dir;
    }
    let max = h::script_value(f, 2, f.co_attrs.ground_max_horizontal_velocity);
    clamp_ground_vel(f, max);
    if f.ground_or_air == GA_Air {
        h::script_game_event(4, f.player_id as i32, f.motion_id as i32, f.is_sub_fighter() as i32, 0);
    }
    f.ground_or_air = GA_Ground;
    f.gr_vel = f.self_vel.x;
    f.x1968_jumpsUsed = 0;
    f.x1969_walljumpUsed = 0;
    f.set_x2227_b0(0);
    c2::unlock_ecb(f);
    if was_air {
        h::geno_ground_edge(f, 1);
    }
}

/// `ftCommon_8007D7FC`: 8007D780's air checks, then land.
pub fn ftcommon_8007d7fc(f: &mut Fighter) {
    melee_core::ct!("ftcommon_8007d7fc");
    if f.ground_or_air == GA_Air {
        if h::un_803224dc(f.x8_spawnNum, f.cur_pos.x, f.dmg.x18A4_knockbackMagnitude) {
            f.dmg.x18A4_knockbackMagnitude = 0.0;
        }
        if f.x2227_b0() != 0 && f.x1968_jumpsUsed <= 1 {
            let (s, u) = ids_ftcommon3(f);
            h::pl_8003fc44(s, u);
        }
    }
    ftcommon_8007d6a4(f);
}

pub fn ftcommon_8007d92c(g: G_ftcommon3) {
    melee_core::ct!("ftcommon_8007d92c");
    if fp(g).ground_or_air == GA_Air {
        h::ftco_fall_enter(g);
    } else {
        h::ft_8008a2bc(g);
    }
}

pub fn ftcommon_8007db24(g: G_ftcommon3) {
    melee_core::ct!("ftcommon_8007db24");
    fp(g).set_x2219_b0(0);
    h::eflib_destroy_all(g);
}

pub fn ftcommon_8007db58(g: G_ftcommon3) {
    melee_core::ct!("ftcommon_8007db58");
    let f = fp(g);
    crate::ft0881::ft_80088770(f);
    crate::ft0881::ft_800887cc(f);
    if let Some(cb) = f.take_dmg_cb {
        unsafe { cb(g) };
    }
    if let Some(cb) = fp(g).death1_cb {
        unsafe { cb(g) };
    }
}

/// `ftCommon_8007DD7C`: push apart from grounded fighters on the same / adjacent floor line.
pub fn ftcommon_8007dd7c(g: G_ftcommon3, v: &Vec3) {
    melee_core::ct!("ftcommon_8007dd7c");
    // 20XX TE hand-warmers: fighters do not push each other
    if melee_core::codes::handwarmers_active() {
        return;
    }
    let arg = unsafe { (*g).user_data as *mut Fighter };
    let mut phi_r28 = false;
    let mut cur = h::plink_head(HSD_GOBJ_PLINK_FIGHTER);
    while !cur.is_null() {
        if cur != g && !h::ftlib_80086fd4(cur, g) {
            let c = unsafe { &*((*cur).user_data as *const Fighter) };
            if !(c.x221F_b3() != 0 || c.ground_or_air != GA_Ground || !c.victim_gobj.is_null() || c.is_sub_fighter() != 0) {
                let (a2c4, a_dir, arg_gnd) = unsafe { ((*arg).x2C4, (*arg).facing_dir, (*arg).coll_data.floor.index) };
                let cur_gnd = c.coll_data.floor.index;
                if same_or_adjacent_floor(arg_gnd, cur_gnd) {
                    let sp24 = c2::ftcommon_8007f8b4(c);
                    let vt = c.x2C4;
                    let f0 = nudge_dx(&a2c4, a_dir, v, c.facing_dir, &vt, &sp24);
                    if abs_c(f0) < a2c4.y + vt.y {
                        let cd = h::common_data();
                        let a = unsafe { &mut *arg };
                        if f0 != 0.0 {
                            a.xF8_playerNudgeVel.x += sgn_c(f0 < 0.0, cd.x450);
                        } else {
                            a.xF8_playerNudgeVel.x += sgn_c(phi_r28, cd.x450);
                        }
                        if v.z - sp24.z != 0.0 {
                            a.xF8_playerNudgeVel.y += sgn_c(v.z - sp24.z < 0.0, cd.x454);
                        } else if f0 != 0.0 {
                            a.xF8_playerNudgeVel.y += sgn_c(f0 < 0.0, cd.x454);
                        } else {
                            a.xF8_playerNudgeVel.y += sgn_c(phi_r28, cd.x454);
                        }
                    }
                }
            }
        } else {
            phi_r28 = true;
        }
        cur = unsafe { (*cur).next };
    }
}

/// `ftCommon_8007DFD0`: the sub fighter (Nana) moves off its main fighter, then 8007DD7C.
pub fn ftcommon_8007dfd0(g: G_ftcommon3, arg1: &Vec3) {
    melee_core::ct!("ftcommon_8007dfd0");
    let f = unsafe { (*g).user_data as *mut Fighter };
    let (pid, f2c4, f_dir, f_gnd) = unsafe { ((*f).player_id, (*f).x2C4, (*f).facing_dir, (*f).coll_data.floor.index) };
    let ng = h::player_get_entity(pid as i32);
    let t = unsafe { &*((*ng).user_data as *const Fighter) };
    if t.x221F_b3() == 0 && t.ground_or_air == GA_Ground {
        let t_gnd = t.coll_data.floor.index;
        if same_or_adjacent_floor(f_gnd, t_gnd) {
            let t = unsafe { &*((*ng).user_data as *const Fighter) };
            let sp1c = c2::ftcommon_8007f8b4(t);
            let tv = t.x2C4;
            let f1 = nudge_dx(&f2c4, f_dir, arg1, t.facing_dir, &tv, &sp1c);
            if abs_c(f1) < f2c4.y + tv.y {
                let d = h::common_data().x45C;
                unsafe { (*f).xF8_playerNudgeVel.y -= d };
            }
        }
    }
    ftcommon_8007dd7c(g, arg1);
}

/// `ftCommon_8007E0E4`: compute this frame's player nudge velocity.
pub fn ftcommon_8007e0e4(g: G_ftcommon3) {
    melee_core::ct!("ftcommon_8007e0e4");
    let f = fp(g);
    f.xF8_playerNudgeVel.y = 0.0;
    f.xF8_playerNudgeVel.x = 0.0;
    if f.x2219_b1() == 0 && f.x2219_b5() == 0 && f.ground_or_air == GA_Ground {
        let cd = h::common_data();
        let (phi_f30, phi_f31) = if f.is_sub_fighter() == 0 { (cd.x454, cd.x458) } else { (cd.x45C, cd.x460) };
        let mut sp10 = c2::ftcommon_8007f8b4(f);
        if f.x221D_b5() == 0 {
            if f.is_sub_fighter() == 0 {
                ftcommon_8007dd7c(g, &sp10);
            } else {
                ftcommon_8007dfd0(g, &sp10);
            }
        }
        let f = fp(g);
        if f.xF8_playerNudgeVel.y == 0.0 && sp10.z != 0.0 {
            f.xF8_playerNudgeVel.y = if sp10.z < 0.0 { phi_f30 } else { -phi_f30 };
        }
        let ny = f.xF8_playerNudgeVel.y;
        if (ny > 0.0 && sp10.z < 0.0 && sp10.z + ny >= 0.0) || (ny < 0.0 && sp10.z > 0.0 && sp10.z + ny <= 0.0) {
            f.xF8_playerNudgeVel.y = -sp10.z;
            sp10.z = 0.0;
        }
        if sp10.z + f.xF8_playerNudgeVel.y > phi_f31 {
            f.xF8_playerNudgeVel.y = phi_f31 - sp10.z;
        } else if sp10.z + f.xF8_playerNudgeVel.y < -phi_f31 {
            f.xF8_playerNudgeVel.y = -phi_f31 - sp10.z;
        }
        if f.is_sub_fighter() == 1 {
            f.xF8_playerNudgeVel.x = 0.0;
        }
    }
}

/// `ftCommon_8007E358`: remember bone 4's translation in x1A7C.
pub fn ftcommon_8007e358(g: G_ftcommon3) {
    melee_core::ct!("ftcommon_8007e358");
    let f = fp(g);
    let j = bone4(f);
    h::hsd_jobj_get_translation(j, &mut f.x1A7C);
}

/// `ftCommon_8007E3EC`: move x1A7C toward bone 4's translation by x1A6C (fused) and set it back.
pub fn ftcommon_8007e3ec(g: G_ftcommon3) {
    melee_core::ct!("ftcommon_8007e3ec");
    let f = fp(g);
    let j = bone4(f);
    if h::hsd_jobj_mtx_is_dirty(j) {
        let mut sp10 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        h::hsd_jobj_get_translation(j, &mut sp10);
        sp10.x -= f.x1A7C.x;
        sp10.y -= f.x1A7C.y;
        sp10.z -= f.x1A7C.z;
        f.x1A7C.x = fmadds(sp10.x, f.x1A6C, f.x1A7C.x);
        f.x1A7C.y = fmadds(sp10.y, f.x1A6C, f.x1A7C.y);
        f.x1A7C.z = fmadds(sp10.z, f.x1A6C, f.x1A7C.z);
        h::hsd_jobj_set_translate(j, &f.x1A7C);
    }
}

pub fn ftcommon_8007e690(f: &mut Fighter, arg1: *mut HSD_AnimJoint) {
    melee_core::ct!("ftcommon_8007e690");
    if !f.x20A0_accessory.is_null() {
        h::hsd_jobj_add_anim_all(f.x20A0_accessory, arg1);
        h::hsd_jobj_req_anim_all(f.x20A0_accessory, 0.0);
    }
}

/// `ftCommon_8007E6DC(gobj, item, arg2)`: drop the held item (TARGET_PC m-ex dispatch).
pub fn ftcommon_8007e6dc(g: G_ftcommon3, item: G_ftcommon3, arg2: i32) {
    melee_core::ct!("ftcommon_8007e6dc");
    let f = fp(g);
    if !f.x1978.is_null() && f.x1978 == item {
        f.x1978 = core::ptr::null_mut();
        return;
    }
    if h::ftco_800c5240(g) {
        h::ftco_800c5500(g);
    }
    let f = fp(g);
    h::mex_on_item_drop_ext(f.kind as i32, g, arg2);
    let f = fp(g);
    let (s, u) = ids_ftcommon3(f);
    h::pl_8003ea08(s, u);
    fp(g).item_gobj = core::ptr::null_mut();
}

pub fn ftcommon_8007e79c(g: G_ftcommon3, arg1: i32) {
    melee_core::ct!("ftcommon_8007e79c");
    h::mex_on_item_drop(fp(g).kind as i32, g, arg1);
}

pub fn ftcommon_8007e7e4(g: G_ftcommon3, arg1: i32) {
    melee_core::ct!("ftcommon_8007e7e4");
    h::mex_on_item_pickup2(fp(g).kind as i32, g, arg1);
}

pub fn ftcommon_8007e82c(g: G_ftcommon3) {
    melee_core::ct!("ftcommon_8007e82c");
    fp(g).x1984_heldItemSpec = core::ptr::null_mut();
}

/// `ftCommon_8007E83C(gobj, state, div)`: drive the held parasol's animation speed.
pub fn ftcommon_8007e83c(g: G_ftcommon3, arg1: i32, div: f32) {
    melee_core::ct!("ftcommon_8007e83c");
    assert!(ft_get_parasol_status(g) != FT_PARASOL_NONE, "ftcommon.c:1276");
    let f = fp(g);
    let val = if div == 0.0 {
        f.frame_speed_mul
    } else if h::it_get_kind(f.item_gobj) == It_Kind_Parasol as i32 {
        let r = h::it_8028b08c(f.item_gobj, PARASOL_TABLE_2[arg1 as usize]);
        fp(g).frame_speed_mul * (r as f32 / div)
    } else {
        let r = h::it_802bda40(f.item_gobj, PARASOL_TABLE_4[arg1 as usize]);
        fp(g).frame_speed_mul * (r as f32 / div)
    };
    let item = fp(g).item_gobj;
    if h::it_get_kind(item) == It_Kind_Parasol as i32 {
        assert!((0..7).contains(&arg1), "parasol_table_1[{arg1}] out of range");
        h::it_parasol_table_1(arg1, item, val);
    } else if arg1 >= 0 && arg1 < 7 && PARASOL_TABLE_3_SET[arg1 as usize] {
        h::it_parasol_table_3(arg1, item, val);
    }
}

/// `ftCommon_8007EA90(fp, arg1)`: spawn hit particles (it_802E5F8C) and credit the attacker.
pub fn ftcommon_8007ea90(f: &mut Fighter, arg1: i32) {
    melee_core::ct!("ftcommon_8007ea90");
    if !h::gm_8016b0b4() {
        return;
    }
    let (phi_f31, phi_f30, sp10);
    if f.dmg.kb_applied != 0.0 {
        let kb = f.dmg.kb_applied;
        let mut a = h::ftco_damage_calc_angle(f, kb);
        if f.dmg.facing_dir_1 > 0.0 {
            const PI: f64 = core::f64::consts::PI;
            a = if a as f64 > PI { (PI * 3.0 - a as f64) as f32 } else { (PI - a as f64) as f32 };
        }
        phi_f31 = a;
        phi_f30 = kb;
        sp10 = Vec3 { x: f.dmg.x1854_collpos.x, y: f.dmg.x1854_collpos.y, z: 0.0 };
    } else {
        phi_f31 = (core::f64::consts::PI / 2.0) as f32;
        phi_f30 = h::common_data().x5C0;
        let mut v = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        h::ftlib_800866dc(f.gobj, &mut v);
        sp10 = v;
    }
    let r = h::it_802e5f8c(f.gobj, &sp10, arg1, 1, phi_f31, phi_f30);
    let src = f.dmg.x18c4_source_ply;
    if src != 6 {
        h::player_set_unk98(src, r.wrapping_add(h::player_get_unk98(src)));
    }
}

/// `ftCommon_8007EC30`: 8007EBAC on every fighter.
pub fn ftcommon_8007ec30(arg0: u32, arg1: u32) {
    melee_core::ct!("ftcommon_8007ec30");
    let mut cur = h::plink_head(HSD_GOBJ_PLINK_FIGHTER);
    while !cur.is_null() {
        c2::ftcommon_8007ebac(fp(cur), arg0, arg1);
        cur = unsafe { (*cur).next };
    }
}

pub fn ftcommon_8007eec8(f: &mut Fighter, arg1: i32, arg2: i32) {
    melee_core::ct!("ftcommon_8007eec8");
    if holds_sword(f) {
        let multiplier = (1.0f64 / 256.0) as f32;
        h::it_80284fc4(f.item_gobj, arg1, arg2 as f32 * multiplier);
    }
}

pub fn ftcommon_8007ef5c(f: &mut Fighter, arg1: i32) {
    melee_core::ct!("ftcommon_8007ef5c");
    if holds_sword(f) {
        h::it_80285024(f.item_gobj, arg1);
    }
}

pub fn ftcommon_8007f578(g: G_ftcommon3) {
    melee_core::ct!("ftcommon_8007f578");
    let f = fp(g);
    if f.item_gobj.is_null() {
        return;
    }
    h::mex_on_item_invisible(f.kind as i32, g);
}

/// `ftCommon_8007F5CC(gobj, visible)`; `bool` is int, stored into a `:1`.
pub fn ftcommon_8007f5cc(g: G_ftcommon3, arg1: i32) {
    melee_core::ct!("ftcommon_8007f5cc");
    let f = fp(g);
    let item = f.item_gobj;
    if !item.is_null() && f.x221E_b3() as i32 != arg1 {
        if arg1 == 0 {
            ftcommon_8007f578(g);
        } else {
            h::mex_on_item_visible(f.kind as i32, g);
        }
    }
    fp(g).set_x221E_b3((arg1 & 1) as u8);
}

/// `ftCommon_8007F6A4(fp, jobj)`: scale `jobj` by 1 / model_scaling.
pub fn ftcommon_8007f6a4(f: &mut Fighter, jobj: *mut HSD_JObj) {
    melee_core::ct!("ftcommon_8007f6a4");
    let val = 1.0f32 / unsafe { (*(*f.ft_data).x0).model_scaling };
    let scale = Vec3 { x: val, y: val, z: val };
    h::hsd_jobj_set_scale(jobj, &scale);
}

pub fn ftcommon_8007f824(g: G_ftcommon3) {
    melee_core::ct!("ftcommon_8007f824");
    h::mex_on_knockback_enter(fp(g).kind as i32, g);
}

pub fn ftcommon_8007f86c(g: G_ftcommon3) {
    melee_core::ct!("ftcommon_8007f86c");
    h::mex_on_knockback_exit(fp(g).kind as i32, g);
}

pub fn ftcommon_8007f8e8(g: G_ftcommon3) {
    melee_core::ct!("ftcommon_8007f8e8");
    let f = fp(g);
    if f.x197C.is_null() || f.x1980.is_null() {
        h::mex_on_remove_head_item(f.kind as i32, g);
    }
}

pub fn ftcommon_8007f948(g: G_ftcommon3, arg1: G_ftcommon3, arg2: i32) {
    melee_core::ct!("ftcommon_8007f948");
    let f = fp(g);
    f.x197C = arg1;
    f.x2014 = arg2;
    apply_head_item_cb(g);
}

pub fn ftcommon_8007f9b4(g: G_ftcommon3) {
    melee_core::ct!("ftcommon_8007f9b4");
    fp(g).x197C = core::ptr::null_mut();
    h::ftco_800d105c(g);
    h::ft_80081c88(g, fp(g).x34_scale.y);
}

/// `ftCommon_8007FA00`: place / scale the head item from `co_attrs.x114 / x120 / x12C`.
pub fn ftcommon_8007fa00(g: G_ftcommon3) {
    melee_core::ct!("ftcommon_8007fa00");
    let f = fp(g);
    let (a, b) = (f.co_attrs.x120, f.co_attrs.x114);
    h::it_80294eb0(f.x197C, &a, &b);
    let f = fp(g);
    h::it_80294e78(f.x197C, f.x34_scale.y * f.co_attrs.x12C);
}

/// `ftCommon_8007FA58(gobj, item)`: put `item` on the head (or just refresh the timer).
pub fn ftcommon_8007fa58(g: G_ftcommon3, arg1: G_ftcommon3) {
    melee_core::ct!("ftcommon_8007fa58");
    if !fp(g).x197C.is_null() {
        fp(g).x2014 = h::it_8026b54c(arg1) as i32; // float -> s32 (saturating)
        h::play_sfx(fp(g), 0x117, 0x7F, 0x40);
        c2::ftcommon_8007ebac(fp(g), 0x10, 0);
        h::item_8026a8ec(arg1);
        return;
    }
    let vec = Vec3 { x: 0.0, y: 0.0, z: 0.0 }; // ftCo_803B74A0
    h::it_8026bd0c(arg1);
    h::item_8026abd8(arg1, &vec, 0.0);
    let part = unsafe { (*(*fp(g).ft_data).x8).x12 } as i32;
    h::item_8026ab54(arg1, g, part);
    let t = h::it_8026b54c(arg1) as i32;
    ftcommon_8007f948(g, arg1, t);
    h::ftco_800d105c(g);
    h::ft_80081c88(g, fp(g).x34_scale.y);
    h::play_sfx(fp(g), 0x117, 0x7F, 0x40);
    c2::ftcommon_8007ebac(fp(g), 0x10, 0);
    h::it_8026bcf4(arg1);
    ftcommon_8007fa00(g);
}

/// `ftCommon_8007FC7C(gobj, arg8)`: grow the held (x1980) item, or spawn one.
/// Float -> int conversions saturate (gwtool lowering), int adds wrap (`-fwrapv`).
pub fn ftcommon_8007fc7c(g: G_ftcommon3, arg8: f32) {
    melee_core::ct!("ftcommon_8007fc7c");
    let mut sp20 = (arg8 * h::common_data().x704) as i32;
    if !fp(g).x1980.is_null() {
        let f = fp(g);
        f.x2018 = f.x2018.wrapping_add(sp20);
        if f.x2018 > h::common_data().x700 {
            f.x2018 = h::common_data().x700;
        }
        sp20 = f.x2024;
        f.x2024 = (sp20 as f32 + arg8) as i32;
        c2::ftcommon_8007fda0(f);
    } else {
        let f = fp(g);
        let part = unsafe { (*(*f.ft_data).x8).x12 } as i32;
        let pos = f.cur_pos;
        let item = h::it_8029a748(g, &pos, part, f.facing_dir);
        if !item.is_null() {
            ftcommon_8007fe84(g, item, sp20, arg8);
            c2::ftcommon_8007fda0(fp(g));
        }
    }
    h::play_sfx(fp(g), 0x11F, 0x7F, 0x40);
}

/// `ftCommon_8007FE84(gobj, item, arg2, arg3)`: hold `item` in x1980.
pub fn ftcommon_8007fe84(g: G_ftcommon3, item: G_ftcommon3, arg2: i32, arg3: f32) {
    melee_core::ct!("ftcommon_8007fe84");
    let f = fp(g);
    f.x1980 = item;
    f.x2018 = arg2;
    f.x201C = h::common_data().x6F8;
    f.x2024 = arg3 as i32;
    f.x2021 = 0;
    f.x2020 = 0;
    c2::ftcommon_8007ebac(f, 2, 0);
    apply_head_item_cb(g);
}

/// `ftCommon_8007D9D4`: `atan2f(lstick.y, lstick.x)`.
pub fn ftcommon_8007d9d4(fp: &Fighter) -> f32 {
    melee_core::ct!("ftcommon_8007d9d4");
    melee_runtime::lbtrigf::atan2f(fp.input.lstick[0].y, fp.input.lstick[0].x)
}

// ---- folded from ftcommon_c.rs (consolidation steps 21/22) ----
pub fn ftCommon_CalcGroundAccel_Deaccel(fp: &mut Fighter, friction: f32) { melee_core::ct!("ftCommon_CalcGroundAccel_Deaccel"); fc::calc_ground_accel_deaccel(fp, friction) }

pub fn ftCommon_CalcGroundAccel_DashRun(fp: &mut Fighter, accel: f32, target_vel: f32, friction: f32) { melee_core::ct!("ftCommon_CalcGroundAccel_DashRun"); fc::calc_ground_accel_dash_run(fp, accel, target_vel, friction) }

pub fn ftCommon_CalcGroundAccel_AccelToVel(fp: &mut Fighter, accel: f32, target_vel: f32, friction: f32) { melee_core::ct!("ftCommon_CalcGroundAccel_AccelToVel"); fc::calc_ground_accel_accel_to_vel(fp, accel, target_vel, friction) }

pub fn ftCommon_CalcGroundAccel_AccelToLStickX_fp(fp: &mut Fighter, a: f32, b: f32, c: f32) { melee_core::ct!("ftCommon_CalcGroundAccel_AccelToLStickX_fp"); fc::calc_ground_accel_accel_to_lstick_x(fp, a, b, c) }

pub fn ftCommon_SetSelfMovementFromGroundedMovement_fp(g: G_ftcommon_c) { melee_core::ct!("ftCommon_SetSelfMovementFromGroundedMovement_fp"); unsafe { fc::set_self_movement_from_grounded_movement(fpg_ftcommon_c(g)) } }

pub fn ftCommon_SetSelfMovementFromGroundedMovement_NoFriction(g: G_ftcommon_c) { melee_core::ct!("ftCommon_SetSelfMovementFromGroundedMovement_NoFriction"); unsafe { fc::set_self_movement_from_grounded_movement_no_friction(fpg_ftcommon_c(g)) } }

pub fn ftCommon_ClampGroundVel_fp(fp: &mut Fighter, max: f32) { melee_core::ct!("ftCommon_ClampGroundVel_fp"); fc::clamp_ground_vel(fp, max) }

pub fn ftCommon_ApplyGroundedKnockbackFriction_fp(fp: &mut Fighter, f: f32) { melee_core::ct!("ftCommon_ApplyGroundedKnockbackFriction_fp"); fc::apply_grounded_knockback_friction(fp, f) }

pub fn ftCommon_SetGroundedKnockbackIfLanded(fp: &mut Fighter) { melee_core::ct!("ftCommon_SetGroundedKnockbackIfLanded"); fc::set_grounded_knockback_if_landed(fp) }

pub fn ftCommon_SandbagKnockbackDeaccel_fp(kb: f32, dec: f32) -> f32 { melee_core::ct!("ftCommon_SandbagKnockbackDeaccel_fp"); fc::sandbag_knockback_deaccel(kb, dec) }

pub fn ftCommon_ApplyShieldKnockbackFriction_fp(fp: &mut Fighter, v: f32) { melee_core::ct!("ftCommon_ApplyShieldKnockbackFriction_fp"); fc::apply_shield_knockback_friction(fp, v) }

pub fn ftCommon_CalcSelfAccel_Deaccel_fp(fp: &mut Fighter, friction: f32) { melee_core::ct!("ftCommon_CalcSelfAccel_Deaccel_fp"); fc::calc_self_accel_deaccel(fp, friction) }

pub fn ftCommon_CalcSelfAccel_DeaccelAir_fp(fp: &mut Fighter) { melee_core::ct!("ftCommon_CalcSelfAccel_DeaccelAir_fp"); fc::calc_self_accel_deaccel_air(fp) }

pub fn ftCommon_CalcSelfAccel_DeaccelQuickAir(fp: &mut Fighter) -> bool { melee_core::ct!("ftCommon_CalcSelfAccel_DeaccelQuickAir"); fc::calc_self_accel_deaccel_quick_air(fp) }

pub fn ftCommon_CalcSelfAccel_DeaccelQuick(fp: &mut Fighter, max_vel: f32) -> bool { melee_core::ct!("ftCommon_CalcSelfAccel_DeaccelQuick"); fc::calc_self_accel_deaccel_quick(fp, max_vel) }

pub fn ftCommon_CalcSelfAccel_AccelToVelClamped(fp: &mut Fighter, a: f32, b: f32, c: f32) { melee_core::ct!("ftCommon_CalcSelfAccel_AccelToVelClamped"); fc::calc_self_accel_accel_to_vel_clamped(fp, a, b, c) }

pub fn ftCommon_CalcSelfAccel_AccelToVelClampedFrom(fp: &mut Fighter, vel: f32, a: f32, b: f32, c: f32) { melee_core::ct!("ftCommon_CalcSelfAccel_AccelToVelClampedFrom"); fc::calc_self_accel_accel_to_vel_clamped_from(fp, vel, a, b, c) }

pub fn ftCommon_CalcSelfAccel_Drift_fp(fp: &mut Fighter) { melee_core::ct!("ftCommon_CalcSelfAccel_Drift_fp"); fc::calc_self_accel_drift(fp) }

pub fn ftCommon_CalcSelfAccel_DriftFrom(fp: &mut Fighter, vel: f32) { melee_core::ct!("ftCommon_CalcSelfAccel_DriftFrom"); fc::calc_self_accel_drift_from(fp, vel) }

pub fn ftCommon_CalcSelfAccel_AccelToVel(fp: &mut Fighter, a: f32, b: f32, c: f32) { melee_core::ct!("ftCommon_CalcSelfAccel_AccelToVel"); fc::calc_self_accel_accel_to_vel(fp, a, b, c) }

pub fn ftCommon_CalcSelfAccel_DriftSimple_fp(fp: &mut Fighter, a: f32, b: f32, c: f32) { melee_core::ct!("ftCommon_CalcSelfAccel_DriftSimple_fp"); fc::calc_self_accel_drift_simple(fp, a, b, c) }

pub fn ftCommon_CalcSelfAccel_DriftSimple_NoFriction_fp(fp: &mut Fighter, a: f32, b: f32, c: f32) { melee_core::ct!("ftCommon_CalcSelfAccel_DriftSimple_NoFriction_fp"); fc::calc_self_accel_drift_simple_no_friction(fp, a, b, c) }

pub fn ftCommon_ClampSelfVelX_fp(fp: &mut Fighter, max: f32) { melee_core::ct!("ftCommon_ClampSelfVelX_fp"); fc::clamp_self_vel_x(fp, max) }

pub fn ftCommon_ClampAirDrift(fp: &mut Fighter) { melee_core::ct!("ftCommon_ClampAirDrift"); fc::clamp_air_drift(fp) }

pub fn ftCommon_Fall_fp(fp: &mut Fighter, gravity: f32, terminal_vel: f32) { melee_core::ct!("ftCommon_Fall_fp"); fc::fall(fp, gravity, terminal_vel) }

pub fn ftCommon_FallBasic_fp(fp: &mut Fighter) { melee_core::ct!("ftCommon_FallBasic_fp"); fc::fall_basic(fp) }

pub fn ftCommon_FallFast(fp: &mut Fighter) { melee_core::ct!("ftCommon_FallFast"); fc::fall_fast(fp) }

pub fn ftCommon_ClampFallSpeed(fp: &mut Fighter, v: f32) { melee_core::ct!("ftCommon_ClampFallSpeed"); fc::clamp_fall_speed(fp, v) }

pub fn ftCommon_Ascend(fp: &mut Fighter, accel: f32, max: f32) { melee_core::ct!("ftCommon_Ascend"); fc::ascend(fp, accel, max) }

pub fn ftCommon_CheckFallFast(fp: &mut Fighter) -> bool { melee_core::ct!("ftCommon_CheckFallFast"); fc::check_fall_fast(fp) }

pub fn ftCommon_UpdateFacing_fp(fp: &mut Fighter) { melee_core::ct!("ftCommon_UpdateFacing_fp"); fc::update_facing(fp) }

pub fn ftCommon_8007DA24(fp: &mut Fighter) { melee_core::ct!("ftCommon_8007DA24"); fc::update_facing_outside_deadzone(fp) }

pub fn ftCommon_CalcHitlag_fp(dmg: i32, msid: i32, mul: f32) -> f32 { melee_core::ct!("ftCommon_CalcHitlag_fp"); fc::calc_hitlag(dmg, msid, mul) }

/// `ftCo_GetLStickAngle`: `atan2f(lstick.y, ABS(lstick.x))`.
pub fn ftCo_GetLStickAngle(fp: &Fighter) -> f32 { melee_core::ct!("ftCo_GetLStickAngle"); crate::ftcommon3::ftco_get_lstick_angle(fp) }
