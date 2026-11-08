//! src/melee/ft/ftlib.c: the functions left out of `crate::ftlib` (round 3), which holds the
//! other 93 (oracle-verified by tests/ftlib_ref.rs) and is re-exported here.
//! ftLib_GetGroundSlopeAngle has no fused site (fma_sites.py); it is a plain atan2f of the floor normal.
#![allow(non_snake_case)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use crate::ftcommon2 as c2;
#[allow(unused_imports)]
use melee_core::fp::fmadds;
#[allow(unused_imports)]
use crate::ftlib::{G, each, fp as fp_ftlib, get_item, get_kind, get_motion_id, msid_in, rumble_helper, skip_opponent, update_logic_off_camera};
pub use crate::ftlib::*;

use crate::hooks as h;
use crate::types::*;

#[inline(always)]
fn fp<'a>(g: G) -> &'a mut Fighter {
    unsafe { &mut *((*g).user_data as *mut Fighter) }
}

/// Callees in TUs not ported yet (settable; unset panics). TODO rewire.
pub mod ext {
    use super::G;
    use crate::types::*;
    macro_rules! stubs {
        ($($stat:ident $name:ident ($($a:ident: $t:ty),*) $(-> $r:ty)?;)*) => {$(
            pub static mut $stat: Option<fn($($t),*) $(-> $r)?> = None;
            pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { $stat } {
                    Some(f) => f($($a),*),
                    None => melee_core::unported!(concat!("TODO stub: ", stringify!($name))),
                }
            }
        )*};
    }
    stubs! {
        FTCO_800C8348 ftCo_800C8348(g: G, a: i32, b: i32);
        IT_8026B560 it_8026B560(it: G) -> f32;
        IT_8026B574 it_8026B574(it: G) -> f32;
        CAMERA_REQUEST_QUAKE Camera_RequestQuake(k: CmQuakeKind, pos: &Vec3);
        FTDATA_8008572C ftData_8008572C(kind: i32);
        EFASYNC_LOAD_SYNC efAsync_LoadSync(id: i32);
        FTDATA_80085820 ftData_80085820(kind: i32, costume: u8);
        FTDATA_80085A14 ftData_80085A14(kind: i32);
        FTDATA_800857E0 ftData_800857E0(kind: i32);
        // `ftData_UnkBytePerCharacter[kind]` (ftdata.c table).
        FTDATA_UNK_BYTE ftData_UnkBytePerCharacter(kind: i32) -> i32;
        // `CostumeListsForeachCharacter[kind].numCostumes`.
        COSTUME_COUNT CostumeListsForeachCharacter_numCostumes(kind: i32) -> u8;
    }
}

/// `ftLib_ApplyMetalBox`: fighter picks up a metal box.
pub fn ftlib_apply_metal_box(g: G, item: G) {
    melee_core::ct!("ftlib_apply_metal_box");
    assert!(h::it_get_kind(item) == It_Kind_MetalB as i32, "ftlib.c:1117");
    ext::ftCo_800C8348(g, melee_core::ppc::fctiwz_f32(ext::it_8026B560(item)), melee_core::ppc::fctiwz_f32(ext::it_8026B574(item)));
    h::ftco_800d105c(g);
    h::ft_80081c88(g, fp(g).x34_scale.y);
    crate::ftcommon2::ftcommon_8007ebac(fp(g), 0xC, 0);
    let pos = fp(g).cur_pos;
    ext::Camera_RequestQuake(QuakeKind_Small, &pos);
    h::play_sfx(fp(g), 0x121, 0x7F, 0x40);
}

/// `ftLib_LoadFighterCostume`: load one costume's archives for `ft_kind` (C param is s8).
pub fn ftlib_load_fighter_costume(ft_kind: i8, arg1: u8) {
    melee_core::ct!("ftlib_load_fighter_costume");
    let k = ft_kind as i32;
    ext::ftData_8008572C(k);
    ext::efAsync_LoadSync(ext::ftData_UnkBytePerCharacter(k));
    ext::ftData_80085820(k, arg1);
    ext::ftData_80085A14(k);
    ext::ftData_800857E0(k);
}

/// `ftLib_LoadFighterAllCostumes`: load every costume's archives for `ft_kind`.
pub fn ftlib_load_fighter_all_costumes(ft_kind: i8) {
    melee_core::ct!("ftlib_load_fighter_all_costumes");
    let k = ft_kind as i32;
    ext::ftData_8008572C(k);
    ext::efAsync_LoadSync(ext::ftData_UnkBytePerCharacter(k));
    let mut i: u8 = 0;
    while i < ext::CostumeListsForeachCharacter_numCostumes(k) {
        ext::ftData_80085820(k, i);
        i = i.wrapping_add(1);
    }
    ext::ftData_80085A14(k);
    ext::ftData_800857E0(k);
}

/// `ftLib_LoadKirbyHats` (TARGET_PC path): Kirby hat archives for every valid selectable kind.
pub fn ftlib_load_kirby_hats(arg0: u8) {
    melee_core::ct!("ftlib_load_kirby_hats");
    for i in 0..SELKIND_COUNT as i32 {
        h::ftkb_specialn_800eed50(crate::player::player_800325c8(i as _, 0) as i32, arg0 as i32);
        if i == CKind_Zelda as i32 || i == CKind_Seak as i32 {
            h::ftkb_specialn_800eed50(crate::player::player_800325c8(i as _, 1) as i32, arg0 as i32);
        }
    }
}

/// `ftLib_GetGroundSlopeAngle`: floor slope angle; asserts the fighter is grounded.
pub fn ftlib_get_ground_slope_angle(g: G) -> f32 {
    melee_core::ct!("ftlib_get_ground_slope_angle");
    let f = fp(g);
    assert!(f.ground_or_air == GA_Ground, "ftlib.c:1517");
    let n = f.coll_data.floor.normal;
    melee_runtime::lbtrigf::atan2f(-n.x, n.y)
}

/// `ftLib_UpdateLogicOffCamera` (TARGET_PC, ftlib.c): the off-screen flag `ftLib_UpdateScreenVisibility` writes from
/// the draw callback, computed from logic (the projection of the camera box through the game camera).
pub fn ftLib_UpdateLogicOffCamera(gobj: G) {
    melee_core::ct!("ftLib_UpdateLogicOffCamera");
    let f = fp(gobj);
    if f.x221F_b3() != 0 {
        return; // ftDrawCommon_80080E18 skips the test then too
    }
    if f.x2229_b3() == 0 && f.x2220_b7() == 0 && !h::camera_80030a50().is_null() {
        let cam = f.x890_cameraBox;
        let on = h::camera_logic_to_screen(cam, &mut f.x2188);
        f.set_x221F_b0(!on as u8);
    } else {
        f.set_x221F_b0(0);
    }
}

// ---- folded from ftlib.rs (consolidation steps 21/22) ----
/// `ftLib_CountFighters`: number of fighter gobjs.
pub fn ftlib_count_fighters() -> i32 {
    melee_core::ct!("ftlib_count_fighters");
    let mut n = 0;
    each(|_| {
        n += 1;
        false
    });
    n
}

/// `ftLib_FindLowestPercentOpponent`: the opponent with the lowest percent.
pub fn ftlib_find_lowest_percent_opponent(g: G) -> G {
    melee_core::ct!("ftlib_find_lowest_percent_opponent");
    let own = if g.is_null() { None } else { Some(&*fp_ftlib(g)) };
    let mut min_percent: i32 = 999;
    let mut result: G = core::ptr::null_mut();
    each(|cur| {
        if !ftlib_is_same_player(g, cur) {
            let cf = fp_ftlib(cur);
            if cf.x221F_b3() != 0 {
                return false;
            }
            if h::gm_8016b168() && own.is_some_and(|f| cf.team == f.team) {
                return false;
            }
            if cf.dmg.x1830_percent < min_percent as f32 {
                min_percent = melee_core::ppc::fctiwz_f32(cf.dmg.x1830_percent);
                result = cur;
            }
        }
        false
    });
    result
}

/// `ftLib_FindNearestOpponent`: the closest opponent to `pos` (camera target bone).
pub fn ftlib_find_nearest_opponent(pos: &Vec3, g: G) -> G {
    melee_core::ct!("ftlib_find_nearest_opponent");
    let own = if g.is_null() { None } else { Some(&*fp_ftlib(g)) };
    let mut min_dist = f32::MAX;
    let mut result: G = core::ptr::null_mut();
    each(|cur| {
        if ftlib_is_same_player(g, cur) || skip_opponent(own, fp_ftlib(cur)) {
            return false;
        }
        let mut v = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        ftlib_get_camera_bone_pos(cur, &mut v);
        let dx = pos.x - v.x;
        let dy = pos.y - v.y;
        let dist = fmadds(dx, dx, dy * dy);
        if dist < min_dist {
            min_dist = dist;
            result = cur;
        }
        false
    });
    result
}

/// `ftLib_FindNearestOpponentInDir`: the closest opponent on the `facing_dir` side of `v`.
pub fn ftlib_find_nearest_opponent_in_dir(v: &Vec3, g: G, facing_dir: f32) -> G {
    melee_core::ct!("ftlib_find_nearest_opponent_in_dir");
    let own = if g.is_null() { None } else { Some(&*fp_ftlib(g)) };
    let mut min_diff = f32::MAX;
    let mut result: G = core::ptr::null_mut();
    each(|cur| {
        if ftlib_is_same_player(g, cur) || skip_opponent(own, fp_ftlib(cur)) {
            return false;
        }
        let mut p = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        ftlib_get_camera_bone_pos(cur, &mut p);
        if (facing_dir == -1.0 && p.x > v.x) || (facing_dir == 1.0 && p.x < v.x) {
            return false;
        }
        let dx = v.x - p.x;
        let dy = v.y - p.y;
        let diff = fmadds(dx, dx, dy * dy);
        if diff < min_diff {
            min_diff = diff;
            result = cur;
        }
        false
    });
    result
}

/// `ftLib_GetOpponentsDir`: which side (-1 / +1) most opponents are on; ties draw `HSD_Randi(2)`.
pub fn ftlib_get_opponents_dir(v: &Vec3, g: G) -> f32 {
    melee_core::ct!("ftlib_get_opponents_dir");
    let own = if g.is_null() { None } else { Some(&*fp_ftlib(g)) };
    let mut result: i32 = 0;
    each(|cur| {
        if ftlib_is_same_player(g, cur) {
            return false;
        }
        let cf = fp_ftlib(cur);
        if skip_opponent(own, cf) {
            return false;
        }
        // TARGET_PC: Slippi WhispyBlowDirFix.
        if h::slippi_codes() == 2 && cf.motion_id <= 0xB {
            return false;
        }
        let mut p = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        ftlib_get_camera_bone_pos(cur, &mut p);
        result += if p.x - v.x < 0.0 { -1 } else { 1 };
        false
    });
    if result == 0 {
        result = if h::hsd_randi(2) != 0 { 1 } else { -1 };
    }
    if result < 0 { -1.0 } else { 1.0 }
}

pub fn ftlib_get_facing_dir(g: G) -> f32 { unsafe { melee_items::tu::ftlib::ftLib_GetFacingDir(g as _) as _ } }

pub fn ftlib_get_ground_air(g: G) -> i32 { unsafe { melee_items::tu::ftlib::ftLib_GetGroundAir(g as _) as _ } }

pub fn ftlib_get_l_stick(g: G, x: &mut f32, y: &mut f32) { unsafe { melee_items::tu::ftlib::ftLib_GetLStick(g as _, x as _, y as _); } }

/// `ftLib_GetHipJObj`: the joint of bone `ftParts_GetBoneIndex(fp_ftlib, 4)`.
pub fn ftlib_get_hip_j_obj(g: G) -> *mut HSD_JObj {
    melee_core::ct!("ftlib_get_hip_j_obj");
    let f = fp_ftlib(g);
    let i = h::ftparts_get_bone_index(f, 4);
    unsafe { (*f.parts.offset(i as isize)).joint }
}

pub fn ftlib_get_part_j_obj(g: G, part: i32) -> *mut HSD_JObj { unsafe { melee_items::tu::ftlib::ftLib_GetPartJObj(g as _, part as _) as _ } }

pub fn ftlib_get_pos(g: G, pos: &mut Vec3) { melee_core::ct!("ftlib_get_pos"); unsafe { melee_world::ftlib::ftLib_GetPos(g, pos) } }

pub fn ftlib_set_pos(g: G, pos: &Vec3) { melee_core::ct!("ftlib_set_pos"); fp_ftlib(g).cur_pos = *pos; }

pub fn ftlib_get_prev_pos(g: G, pos: &mut Vec3) { melee_core::ct!("ftlib_get_prev_pos"); unsafe { melee_world::ftlib::ftLib_GetPrevPos(g, pos) } }

/// `ftLib_GetCameraBonePos`: the camera target bone's world position (`lb_8000B1CC`).
pub fn ftlib_get_camera_bone_pos(g: G, v: &mut Vec3) {
    melee_core::ct!("ftlib_get_camera_bone_pos");
    let f = fp_ftlib(g);
    let i = unsafe { (*(*f.ft_data).x0).camera_zoom_target_bone };
    let j = ftlib_get_part_j_obj(g, i);
    let off = f.co_attrs.x170;
    h::lb_8000b1cc(j, &off, v);
}

pub fn ftlib_release_item(g: G, other: G) {
    melee_core::ct!("ftlib_release_item");
    let f = fp_ftlib(g);
    if !f.item_gobj.is_null() || !f.x1978.is_null() {
        h::ftcommon_8007e6dc(g, other, 1);
    }
}

pub fn ftlib_clear_held_special_item(g: G) {
    melee_core::ct!("ftlib_clear_held_special_item");
    if !fp_ftlib(g).x1984_heldItemSpec.is_null() {
        h::ftcommon_8007e82c(g);
    }
}

pub fn ftlib_is_holding_item(g: G, arg1: G) -> bool { unsafe { melee_items::tu::ftlib::ftLib_IsHoldingItem(g as _, arg1 as _) as _ } }

pub fn ftlib_get_held_special_item(g: G) -> G { unsafe { melee_items::tu::ftlib::ftLib_GetHeldSpecialItem(g as _) as _ } }

pub fn ftlib_is_input_disabled(g: G) -> bool { melee_core::ct!("ftlib_is_input_disabled"); fp_ftlib(g).x221D_b4() != 0 }

pub fn ftlib_disable_input(g: G) {
    melee_core::ct!("ftlib_disable_input");
    h::fighter_reset_input_data(g);
    fp_ftlib(g).set_x221D_b4(1);
}

pub fn ftlib_disable_all_input() {
    melee_core::ct!("ftlib_disable_all_input");
    each(|c| {
        h::ftlib_disable_input(c);
        false
    });
}

pub fn ftlib_enable_input(g: G) { melee_core::ct!("ftlib_enable_input"); fp_ftlib(g).set_x221D_b4(0); }

pub fn ftlib_enable_all_input() {
    melee_core::ct!("ftlib_enable_all_input");
    each(|c| {
        h::ftlib_enable_input(c);
        false
    });
}

pub fn ftlib_is_item_visible(g: G, arg1: G) -> bool {
    melee_core::ct!("ftlib_is_item_visible");
    let f = fp_ftlib(g);
    if f.item_gobj != arg1 && f.x197C != arg1 && f.x1980 != arg1 {
        return true;
    }
    !(f.invisible() != 0
        || f.x221E_b5() != 0
        || (f.x221E_b3() == 0 && f.item_gobj == arg1)
        || (f.x221E_b4() == 0 && (f.x197C == arg1 || f.x1980 == arg1)))
}

/// `ftLib_IsFighter`: is `arg` a fighter gobj.
pub fn ftlib_is_fighter(arg: G) -> bool { melee_core::ct!("ftlib_is_fighter"); unsafe { melee_world::ftlib::ftLib_IsFighter(arg) } }

pub fn ftlib_get_coll_data(g: G) -> *mut CollData { melee_core::ct!("ftlib_get_coll_data"); unsafe { melee_world::ftlib::ftLib_GetCollData(g) } }

pub fn ftlib_get_ecb_center(g: G, v: &mut Vec3) {
    melee_core::ct!("ftlib_get_ecb_center");
    let f = fp_ftlib(g);
    let tmp = 0.5f32 * (f.coll_data.ecb.top.y + f.coll_data.ecb.bottom.y);
    v.x = f.cur_pos.x + 0.0;
    v.y = f.cur_pos.y + tmp;
    v.z = f.cur_pos.z + 0.0;
}

pub fn ftlib_get_model_scale(g: G) -> f32 { unsafe { melee_items::tu::ftlib::ftLib_GetModelScale(g as _) as _ } }

pub fn ftlib_800869f8(g: G) -> f32 { melee_core::ct!("ftlib_800869f8"); fp_ftlib(g).x34_scale.x * fp_ftlib(g).co_attrs.model_scaling }

pub fn ftlib_get_scale(g: G) -> f32 { unsafe { melee_items::tu::ftlib::ftLib_GetScale(g as _) as _ } }

pub fn ftlib_is_shielding(g: G) -> bool {
    melee_core::ct!("ftlib_is_shielding");
    matches!(fp_ftlib(g).motion_id as i32, ftCo_MS_GuardOn | ftCo_MS_Guard | ftCo_MS_GuardSetOff)
}

pub fn ftlib_80086a4c(g: G, val: f32) { melee_core::ct!("ftlib_80086a4c"); unsafe { melee_world::ftlib::ftLib_80086A4C(g, val) } }

pub fn ftlib_get_screen_pos(g: G, x: &mut S32Vec2) -> bool {
    melee_core::ct!("ftlib_get_screen_pos");
    let f = fp_ftlib(g);
    if f.x2229_b3() == 0 && f.x2220_b7() == 0 {
        *x = f.x2188;
    }
    false
}

/// `ftLib_UpdateScreenVisibility` (TARGET_PC): always true; the flag write moved to `update_logic_off_camera`.
pub fn ftlib_update_screen_visibility(_g: G) -> bool { melee_core::ct!("ftlib_update_screen_visibility"); true }

pub fn ftlib_is_offscreen(g: G) -> bool { melee_core::ct!("ftlib_is_offscreen"); unsafe { melee_world::ftlib::ftLib_IsOffscreen(g) } }

pub fn ftlib_get_camera_subject(g: G) -> *mut CmSubject { melee_core::ct!("ftlib_get_camera_subject"); unsafe { melee_world::ftlib::ftLib_GetCameraSubject(g) } }

pub fn ftlib_80086b80(g: G) -> f32 { melee_core::ct!("ftlib_80086b80"); unsafe { melee_world::ftlib::ftLib_80086B80(g) } }

pub fn ftlib_get_camera_subject_bone_pos(g: G, v: &mut Vec3) { melee_core::ct!("ftlib_get_camera_subject_bone_pos"); unsafe { melee_world::ftlib::ftLib_GetCameraSubjectBonePos(g, v) } }

pub fn ftlib_is_camera_subject_in_bounds(g: G) -> bool {
    melee_core::ct!("ftlib_is_camera_subject_in_bounds");
    let cam = ftlib_get_camera_subject(g);
    h::camera_80031154(unsafe { &mut (*cam).pos })
}

pub fn ftlib_get_player_index(g: G) -> u8 { melee_core::ct!("ftlib_get_player_index"); unsafe { melee_world::ftlib::ftLib_GetPlayerIndex(g) } }

pub fn ftlib_get_pos_delta(g: G, v: &mut Vec3) { melee_core::ct!("ftlib_get_pos_delta"); unsafe { melee_world::ftlib::ftLib_GetPosDelta(g, v) } }

pub fn ftlib_start_rumble(g: G, a: i32, b: i32) { melee_core::ct!("ftlib_start_rumble"); rumble_helper(g, a, b, 0); }

pub fn ftlib_start_rumble_all(a: i32, b: i32) {
    melee_core::ct!("ftlib_start_rumble_all");
    each(|c| {
        ftlib_start_rumble(c, a, b);
        false
    });
}

pub fn ftlib_start_item_rumble(g: G, a: i32, b: i32) { melee_core::ct!("ftlib_start_item_rumble"); rumble_helper(g, a, b, 1); }

pub fn ftlib_start_item_rumble_all(a: i32, b: i32) {
    melee_core::ct!("ftlib_start_item_rumble_all");
    each(|c| {
        ftlib_start_item_rumble(c, a, b);
        false
    });
}

pub fn ftlib_stop_item_rumble(g: G) {
    melee_core::ct!("ftlib_stop_item_rumble");
    let f = fp_ftlib(g);
    if h::player_8003544c(f.player_id as i32, f.is_sub_fighter() as i32) {
        h::hsd_pad_rumble_remove_id(f.x618_player_id, 1);
    }
}

pub fn ftlib_get_team(g: G) -> i32 { unsafe { melee_items::tu::ftlib::ftLib_GetTeam(g as _) as _ } }

pub fn ftlib_is_in_hitstun(g: G) -> bool { melee_core::ct!("ftlib_is_in_hitstun"); unsafe { melee_world::ftlib::ftLib_IsInHitstun(g) } }

pub fn ftlib_can_be_magnified(g: G) -> bool {
    melee_core::ct!("ftlib_can_be_magnified");
    let f = fp_ftlib(g);
    !(f.invisible() != 0
        || f.x221E_b2() != 0
        || h::player_get_more_flags_bit4(f.player_id as i32) != 0
        || f.is_sandbag() != 0
        || f.x2229_b3() != 0
        || f.x2220_b7() != 0)
}

pub fn ftlib_80086f4c(g: G) -> bool {
    melee_core::ct!("ftlib_80086f4c");
    let f = fp_ftlib(g);
    f.x221F_b2() == 0 && f.x221F_b1() == 0 && f.x209A == 1
}

pub fn ftlib_get_name_tag_height(g: G) -> f32 {
    melee_core::ct!("ftlib_get_name_tag_height");
    let f = fp_ftlib(g);
    if f.invisible() != 0 {
        return unsafe { (*(*f.ft_data).x0).name_tag_height };
    }
    f.co_attrs.name_tag_height
}

pub fn ftlib_is_sword_swing(g: G) -> bool {
    melee_core::ct!("ftlib_is_sword_swing");
    matches!(get_motion_id(g),
        ftCo_MS_SwordSwing1 | ftCo_MS_SwordSwing3 | ftCo_MS_SwordSwing4 | ftCo_MS_SwordSwingDash)
}

/// `ftLib_IsSamePlayer`: same gobj or same player slot.
pub fn ftlib_is_same_player(g0: G, g1: G) -> bool {
    melee_core::ct!("ftlib_is_same_player");
    if !g0.is_null() && !g1.is_null() {
        if g0 == g1 {
            return true;
        }
        if fp_ftlib(g0).player_id == fp_ftlib(g1).player_id {
            return true;
        }
    }
    false
}

pub fn ftlib_is_sleeping(g: G) -> bool { melee_core::ct!("ftlib_is_sleeping"); unsafe { melee_world::ftlib::ftLib_IsSleeping(g) } }

pub fn ftlib_inc_fighter_ref_count(a: i32) { melee_core::ct!("ftlib_inc_fighter_ref_count"); h::ftdata_80085560(a, 1); }

pub fn ftlib_dec_fighter_ref_count(a: i32) { melee_core::ct!("ftlib_dec_fighter_ref_count"); h::ftdata_80085560(a, -1); }

pub fn ftlib_get_shake_offset(g: G, v: &mut Vec3) -> bool {
    melee_core::ct!("ftlib_get_shake_offset");
    let f = fp_ftlib(g);
    if f.dmg.x18B8 != 0.0 || f.dmg.x18BC != 0.0 {
        v.x = f.dmg.x18B8;
        v.y = f.dmg.x18BC;
        v.z = 0.0;
        return true;
    }
    false
}

/// `ftLib_GetSubColor`: `p_ftCommonData->x6D8[controller_index]` (a flexible array in the C).
pub fn ftlib_get_sub_color(g: G, val: &mut i32) -> bool {
    melee_core::ct!("ftlib_get_sub_color");
    let i = fp_ftlib(g).x61A_controller_index;
    if i != 0 {
        *val = unsafe { *h::common_data().x6D8.as_ptr().add(i as usize) };
        return true;
    }
    false
}

/// `ftLib_SetPercent`: `dmg.x1830_percent = x` (int -> float).
pub fn ftlib_set_percent(g: G, x: i32) { melee_core::ct!("ftlib_set_percent"); fp_ftlib(g).dmg.x1830_percent = x as f32; }

/// `ftLib_GetPercent`: `(s32) dmg.x1830_percent` (PPC fctiwz, melee_core::ppc).
pub fn ftlib_get_percent(g: G) -> i32 { melee_core::ct!("ftlib_get_percent"); melee_core::ppc::fctiwz_f32(fp_ftlib(g).dmg.x1830_percent) }

pub fn ftlib_80087140(g: G) {
    melee_core::ct!("ftlib_80087140");
    if fp_ftlib(g).x221F_b3() == 0 {
        h::ftco_800d4f24(g, 1);
        c2::ftcommon_8007ed2c(fp_ftlib(g));
        h::fighter_unk_process_death(g);
        h::ftcommon_8007d92c(g);
    }
}

pub fn ftlib_80087284(g: G) -> bool { melee_core::ct!("ftlib_80087284"); get_motion_id(g) >= ftCo_MS_LightThrowF4 }

pub fn ftlib_get_shadow(g: G) -> *mut LbShadow { melee_core::ct!("ftlib_get_shadow"); unsafe { melee_world::ftlib::ftLib_GetShadow(g) } }

pub fn ftlib_casts_shadow(g: G) -> bool {
    melee_core::ct!("ftlib_casts_shadow");
    let f = fp_ftlib(g);
    f.x221E_b1() == 0 && f.x2226_b4() == 0 && f.is_sub_fighter() == 0
}

pub fn ftlib_get_last_attacker_slot(g: G) -> i32 { melee_core::ct!("ftlib_get_last_attacker_slot"); fp_ftlib(g).dmg.x18c4_source_ply }

pub fn ftlib_is_last_attacker_sub_fighter(g: G) -> i32 { melee_core::ct!("ftlib_is_last_attacker_sub_fighter"); fp_ftlib(g).x221F_b5() as i32 }

pub fn ftlib_is_sleeping_8008731_c(g: G) -> i32 { melee_core::ct!("ftlib_is_sleeping_8008731_c"); unsafe { melee_world::ftlib::ftLib_IsSleeping_8008731C(g) } }

pub fn ftlib_is_dead(g: G) -> bool { melee_core::ct!("ftlib_is_dead"); unsafe { melee_world::ftlib::ftLib_IsDead(g) } }

pub fn ftlib_is_dead_up(g: G) -> bool { melee_core::ct!("ftlib_is_dead_up"); msid_in(g, ftCo_MS_DeadUp, ftCo_MS_DeadUpFallHitCameraIce) }

pub fn ftlib_is_dead_up_star(g: G) -> bool { melee_core::ct!("ftlib_is_dead_up_star"); msid_in(g, ftCo_MS_DeadUpStar, ftCo_MS_DeadUpStarIce) }

pub fn ftlib_is_dead_up_fall(g: G) -> bool { melee_core::ct!("ftlib_is_dead_up_fall"); msid_in(g, ftCo_MS_DeadUpFall, ftCo_MS_DeadUpFallHitCameraIce) }

pub fn ftlib_is_rebirth(g: G) -> bool { melee_core::ct!("ftlib_is_rebirth"); msid_in(g, ftCo_MS_Rebirth, ftCo_MS_RebirthWait) }

pub fn ftlib_is_entry(g: G) -> bool { melee_core::ct!("ftlib_is_entry"); unsafe { melee_world::ftlib::ftLib_IsEntry(g) } }

/// `ftLib_FindBySpawnNum`: the fighter gobj with spawn number `i`.
pub fn ftlib_find_by_spawn_num(i: u32) -> G { melee_core::ct!("ftlib_find_by_spawn_num"); each(|c| fp_ftlib(c).x8_spawnNum as u32 == i) }

pub fn ftlib_get_crowd_knockback(g: G) -> f32 { melee_core::ct!("ftlib_get_crowd_knockback"); fp_ftlib(g).dmg.x18A4_knockbackMagnitude }

pub fn ftlib_get_spawn_num(g: G) -> u32 { melee_core::ct!("ftlib_get_spawn_num"); fp_ftlib(g).x8_spawnNum as u32 }

/// `ftLib_8008746C` (TARGET_PC): the crowd cheer sfx id; m-ex relative ids (5000..9999) are
/// rebased onto the port kind's sound bank.
pub fn ftlib_8008746c(g: G) -> i32 {
    melee_core::ct!("ftlib_8008746c");
    let f = fp_ftlib(g);
    if c2::ftcommon_80080144(f) {
        return 0x1FBD1;
    }
    let id = unsafe { (*(*f.ft_data).x4C_sfx).x34 };
    if (5000..10000).contains(&id) {
        let ssm = h::mex_ssm_for_port_kind(f.kind as i32);
        if ssm >= 0 {
            return id - 5000 + ssm * 10000;
        }
    }
    id
}

pub fn ftlib_is_sub_fighter(g: G) -> i32 { melee_core::ct!("ftlib_is_sub_fighter"); unsafe { melee_world::ftlib::ftLib_IsSubFighter(g) } }

pub fn ftlib_800874cc(g: G, arg1: *mut core::ffi::c_void, arg2: i32) {
    melee_core::ct!("ftlib_800874cc");
    let f = fp_ftlib(g);
    h::pl_8003e4a4(f.player_id as i32, f.is_sub_fighter() as i32, arg1, arg2);
}

pub fn ftlib_is_frames_remaining(g: G) { melee_core::ct!("ftlib_is_frames_remaining"); h::ftanim_is_frames_remaining(g); }

pub fn ftlib_is_charging_smash(g: G) -> bool { melee_core::ct!("ftlib_is_charging_smash"); fp_ftlib(g).smash_attrs.state as i32 == 2 }

pub fn ftlib_get_last_hit_source_type(g: G) -> i32 { melee_core::ct!("ftlib_get_last_hit_source_type"); fp_ftlib(g).dmg.x18CC }

pub fn ftlib_get_last_hit_source_kind(g: G) -> i32 { melee_core::ct!("ftlib_get_last_hit_source_kind"); fp_ftlib(g).dmg.x18D0 }

pub fn ftlib_get_game_watch_color(g: G, dst: *mut core::ffi::c_void) {
    melee_core::ct!("ftlib_get_game_watch_color");
    if get_kind(g) == Ft_Kind_GameWatch { h::ftgw_init_8014a7f4(g, dst) } else { h::ftkb_init_800eeb00(g, dst) }
}

pub fn ftlib_get_game_watch_outline_color(g: G, dst: *mut core::ffi::c_void) {
    melee_core::ct!("ftlib_get_game_watch_outline_color");
    if get_kind(g) == Ft_Kind_GameWatch { h::ftgw_init_8014a814(g, dst) } else { h::ftkb_init_800eeb1c(g, dst) }
}

pub fn ftlib_is_being_nudged(g: G) -> bool { melee_core::ct!("ftlib_is_being_nudged"); fp_ftlib(g).xF8_playerNudgeVel.x != 0.0 }

/// `ftLib_IsFramesRemaining` (was ftLib_800876B4): `void` in ftlib.c, it only calls ftAnim_IsFramesRemaining; its one caller
/// (gm_1798.c, declared `s32`) reads the return register, which still holds that result in the C build.
pub unsafe fn ftLib_IsFramesRemaining(gobj: *mut melee_types::HSD_GObj) -> i32 {
    melee_core::ct!("ftLib_IsFramesRemaining");
    crate::tu::ftanim::ftAnim_IsFramesRemaining(gobj as _) as i32
}
