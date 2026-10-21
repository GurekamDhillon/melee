//! src/melee/ft/ftcommon.c: the functions not already ported in the older modules
//! (crate::ftcommon / ftcommon2 / ftcommon3 / ftlib hold the other ~110, verified there):
//! `ftCommon_8007E5AC` (ground-slope Z rotation) and `ftCommon_8007EFC8` (Zelda/Sheik
//! transform: copy the active fighter's state onto the partner). The TARGET_PC disc-free
//! fixture `ftCommon_PcFloorGuardTest` / `ftCommon_PcFixtureFall` is a C self-test of the
//! private `ftcommon3::pc_floor_guard` and is not ported (its Rust counterpart is that
//! module's test). No fused sites.
#![allow(non_snake_case, static_mut_refs)]

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
                    None => panic!(concat!("TODO: ", stringify!($name), " not ported")),
                }
            }
        )*};
    }
    ext! {
        PLAYER_GET_ENTITY_AT_INDEX Player_GetEntityAtIndex(slot: i32, i: i32) -> G;
        PLAYER_SWAP_TRANSFORMED_STATES Player_SwapTransformedStates(slot: i32, a: i32, b: i32);
        FIGHTER_UNK_INIT_RESET_80067C98 Fighter_UnkInitReset_80067C98(fp: *mut Fighter);
        PLAYER_SET_HP_BY_INDEX Player_SetHPByIndex(slot: i32, sub: i32, hp: f32);
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
    pub fn ftPartSetRotZ(f: &Fighter, part: i32, v: f32) {
        match unsafe { FTPART_SET_ROT_Z } {
            Some(h) => h(f, part, v),
            None => crate::tu::ftparts::ftPartSetRotZ(f, part, v),
        }
    }
}

#[inline(always)]
fn fpg<'a>(g: G) -> &'a mut Fighter {
    unsafe { &mut *((*g).user_data as *mut Fighter) }
}

/// `ftCommon_8007E5AC`: tilt part 0 to the floor's slope.
pub fn ftcommon_8007e5ac(fp: &mut Fighter) {
    let n = fp.coll_data.floor.normal;
    let tmp = -melee_runtime::lbtrigf::atan2f(n.x, n.y);
    assert!(fp.ground_or_air as i32 == GA_Ground as i32, "fp->ground_or_air == GA_Ground");
    ext::ftPartSetRotZ(fp, 0, tmp);
}

/// `ftCommon_8007EFC8`: hand the match over to the transform partner (Zelda <-> Sheik), then
/// put this fighter to sleep and run `arg1` on the partner.
pub fn ftcommon_8007efc8(gobj: G, arg1: fn(G)) {
    let src = fpg(gobj);
    let dst_gobj = ext::Player_GetEntityAtIndex(src.player_id as i32, 1);
    let dst = fpg(dst_gobj);
    ext::Player_SwapTransformedStates(src.player_id as i32, src.is_sub_fighter() as i32, dst.is_sub_fighter() as i32);
    let tmp_bit = src.is_sub_fighter();
    src.set_is_sub_fighter(dst.is_sub_fighter());
    dst.set_is_sub_fighter(tmp_bit);
    ext::Fighter_UnkInitReset_80067C98(dst);
    crate::ftcommon2::ftcommon_8007e2fc(fpg(dst_gobj));
    let (src, dst) = (fpg(gobj), fpg(dst_gobj));
    dst.cur_pos = src.cur_pos;
    dst.prev_pos = src.prev_pos;
    dst.pos_delta = src.pos_delta;
    dst.facing_dir = src.facing_dir;
    dst.dmg.x1830_percent = src.dmg.x1830_percent;
    ext::Player_SetHPByIndex(dst.player_id as i32, dst.is_sub_fighter() as i32, dst.dmg.x1830_percent);
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
