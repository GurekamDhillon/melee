
//! Hand port, round 3: src/melee/ft/ftlib.c, the fighter accessor library other code calls
#![allow(non_upper_case_globals)]

//! with a fighter `HSD_GObj*`. Functions keep the C signature shape: `g` is the raw gobj
//! (`G = *mut HSD_GObj`), `gobj->user_data` is the `Fighter`. Callers must pass a live fighter
//! gobj, as in C (the C dereferences without checks too).
//!
//! The fighter list (`HSD_GObjPLinkHead[HSD_GOBJ_PLINK_FIGHTER]`) comes from
//! `hooks::plink_head`. Calls out of the ported set go through `crate::hooks`.
//! Fused sites (`tools/rust-port/fma_sites.py src/melee/ft/ftlib.c`): ftLib_FindNearestOpponent and
//! ftLib_FindNearestOpponentInDir, one f32 fmuladd each (`dx*dx + dy*dy` -> `fmadds(dx, dx, dy*dy)`).
//! TARGET_PC paths are the ones ported (80086A8C returns true; Slippi Whispy fix in 800864A8;
//! Geno template kinds in GetKind; m-ex crowd sfx in 8008746C; UpdateLogicOffCamera).
//!
//! Not ported from ftlib.c: ftLib_ApplyMetalBox (metal box: ftCo/it/camera), ftLib_LoadFighterCostume /
//! 80087574 / 80087610 (archive preloading), ftLib_GetGroundSlopeAngle (atan2f; depends on group 1's MSL).
//! Checked against the C (cut verbatim, shipping contraction) by tests/ftlib_ref.rs.

use crate::ftcommon2 as c2;

use crate::hooks as h;

use crate::types::*;

use melee_core::fp::fmadds;

pub type G = *mut HSD_GObj;

#[inline(always)]
pub(crate) fn fp<'a>(g: G) -> &'a mut Fighter {
    unsafe { &mut *((*g).user_data as *mut Fighter) }
}

/// Iterate the fighter p-link list like the C `for (cur = head; cur; cur = cur->next)`.
/// `next` is read before calling `f` (the C reads it after; no ported body unlinks).
pub(crate) fn each(mut f: impl FnMut(G) -> bool) -> G {
    let mut cur = h::plink_head(HSD_GOBJ_PLINK_FIGHTER);
    while !cur.is_null() {
        if f(cur) {
            return cur;
        }
        cur = unsafe { (*cur).next };
    }
    core::ptr::null_mut()
}

pub fn is_master_hand_present() -> bool {
    !each(|c| get_kind(c) == Ft_Kind_MasterH).is_null()
}

pub fn is_crazy_hand_present() -> bool {
    !each(|c| get_kind(c) == Ft_Kind_CrezyH).is_null()
}

#[inline]
pub(crate) fn skip_opponent(fp_own: Option<&Fighter>, cur: &Fighter) -> bool {
    cur.x221F_b3() != 0 || (h::gm_8016b168() && fp_own.is_some_and(|f| cur.team == f.team))
}

pub fn set_scale(g: G, val: f32) {
    fp(g).x34_scale.y = val;
    h::fighter_update_model_scale(g);
    c2::ftcommon_80080174(fp(g));
}

pub fn get_item(g: G) -> G { fp(g).item_gobj }

/// `ftLib_UpdateLogicOffCamera` (TARGET_PC).
pub fn update_logic_off_camera(g: G) {
    let f = fp(g);
    if f.x221F_b3() != 0 {
        return;
    }
    if f.x2229_b3() == 0 && f.x2220_b7() == 0 && !h::camera_80030a50().is_null() {
        let b = !h::camera_logic_to_screen(f.x890_cameraBox, &mut f.x2188);
        f.set_x221F_b0(b as _);
    } else {
        f.set_x221F_b0(0);
    }
}

pub fn get_motion_id(g: G) -> i32 { fp(g).motion_id as i32 }

#[inline]
pub(crate) fn rumble_helper(g: G, arg1: i32, arg2: i32, val: i32) {
    let f = fp(g);
    if !h::player_8003544c(f.player_id as i32, f.is_sub_fighter() as i32) {
        return;
    }
    if f.x221F_b3() == 0 && f.x2224_b2() == 0 {
        h::lb_rumble_start(f.x618_player_id, val, arg1, arg2);
    }
}

/// `ftLib_GetKind` (TARGET_PC): a Geno define on a template skeleton answers as its base kind.
pub fn get_kind(g: G) -> FighterKind {
    let k = fp(g).kind;
    if k >= Ft_Kind_Mex0 && h::geno_define_template_skeleton(k as i32) != 0 {
        return h::geno_define_base_kind(k as i32) as FighterKind;
    }
    k
}

#[inline]
pub(crate) fn msid_in(g: G, lo: i32, hi: i32) -> bool {
    let m = get_motion_id(g);
    m >= lo && m <= hi
}

pub use crate::tu::ftlib::ftlib_count_fighters;
pub use crate::tu::ftlib::ftlib_find_lowest_percent_opponent;
pub use crate::tu::ftlib::ftlib_find_nearest_opponent;
pub use crate::tu::ftlib::ftlib_find_nearest_opponent_in_dir;
pub use crate::tu::ftlib::ftlib_get_opponents_dir;
pub use crate::tu::ftlib::ftlib_get_facing_dir;
pub use crate::tu::ftlib::ftlib_get_ground_air;
pub use crate::tu::ftlib::ftlib_get_l_stick;
pub use crate::tu::ftlib::ftlib_get_hip_j_obj;
pub use crate::tu::ftlib::ftlib_get_part_j_obj;
pub use crate::tu::ftlib::ftlib_get_pos;
pub use crate::tu::ftlib::ftlib_set_pos;
pub use crate::tu::ftlib::ftlib_get_prev_pos;
pub use crate::tu::ftlib::ftlib_get_camera_bone_pos;
pub use crate::tu::ftlib::ftlib_release_item;
pub use crate::tu::ftlib::ftlib_clear_held_special_item;
pub use crate::tu::ftlib::ftlib_is_holding_item;
pub use crate::tu::ftlib::ftlib_get_held_special_item;
pub use crate::tu::ftlib::ftlib_is_input_disabled;
pub use crate::tu::ftlib::ftlib_disable_input;
pub use crate::tu::ftlib::ftlib_disable_all_input;
pub use crate::tu::ftlib::ftlib_enable_input;
pub use crate::tu::ftlib::ftlib_enable_all_input;
pub use crate::tu::ftlib::ftlib_is_item_visible;
pub use crate::tu::ftlib::ftlib_is_fighter;
pub use crate::tu::ftlib::ftlib_get_coll_data;
pub use crate::tu::ftlib::ftlib_get_ecb_center;
pub use crate::tu::ftlib::ftlib_get_model_scale;
pub use crate::tu::ftlib::ftlib_800869f8;
pub use crate::tu::ftlib::ftlib_get_scale;
pub use crate::tu::ftlib::ftlib_is_shielding;
pub use crate::tu::ftlib::ftlib_80086a4c;
pub use crate::tu::ftlib::ftlib_get_screen_pos;
pub use crate::tu::ftlib::ftlib_update_screen_visibility;
pub use crate::tu::ftlib::ftlib_is_offscreen;
pub use crate::tu::ftlib::ftlib_get_camera_subject;
pub use crate::tu::ftlib::ftlib_80086b80;
pub use crate::tu::ftlib::ftlib_get_camera_subject_bone_pos;
pub use crate::tu::ftlib::ftlib_is_camera_subject_in_bounds;
pub use crate::tu::ftlib::ftlib_get_player_index;
pub use crate::tu::ftlib::ftlib_get_pos_delta;
pub use crate::tu::ftlib::ftlib_start_rumble;
pub use crate::tu::ftlib::ftlib_start_rumble_all;
pub use crate::tu::ftlib::ftlib_start_item_rumble;
pub use crate::tu::ftlib::ftlib_start_item_rumble_all;
pub use crate::tu::ftlib::ftlib_stop_item_rumble;
pub use crate::tu::ftlib::ftlib_get_team;
pub use crate::tu::ftlib::ftlib_is_in_hitstun;
pub use crate::tu::ftlib::ftlib_can_be_magnified;
pub use crate::tu::ftlib::ftlib_80086f4c;
pub use crate::tu::ftlib::ftlib_get_name_tag_height;
pub use crate::tu::ftlib::ftlib_is_sword_swing;
pub use crate::tu::ftlib::ftlib_is_same_player;
pub use crate::tu::ftlib::ftlib_is_sleeping;
pub use crate::tu::ftlib::ftlib_inc_fighter_ref_count;
pub use crate::tu::ftlib::ftlib_dec_fighter_ref_count;
pub use crate::tu::ftlib::ftlib_get_shake_offset;
pub use crate::tu::ftlib::ftlib_get_sub_color;
pub use crate::tu::ftlib::ftlib_set_percent;
pub use crate::tu::ftlib::ftlib_get_percent;
pub use crate::tu::ftlib::ftlib_80087140;
pub use crate::tu::ftlib::ftlib_80087284;
pub use crate::tu::ftlib::ftlib_get_shadow;
pub use crate::tu::ftlib::ftlib_casts_shadow;
pub use crate::tu::ftlib::ftlib_get_last_attacker_slot;
pub use crate::tu::ftlib::ftlib_is_last_attacker_sub_fighter;
pub use crate::tu::ftlib::ftlib_is_sleeping_8008731_c;
pub use crate::tu::ftlib::ftlib_is_dead;
pub use crate::tu::ftlib::ftlib_is_dead_up;
pub use crate::tu::ftlib::ftlib_is_dead_up_star;
pub use crate::tu::ftlib::ftlib_is_dead_up_fall;
pub use crate::tu::ftlib::ftlib_is_rebirth;
pub use crate::tu::ftlib::ftlib_is_entry;
pub use crate::tu::ftlib::ftlib_find_by_spawn_num;
pub use crate::tu::ftlib::ftlib_get_crowd_knockback;
pub use crate::tu::ftlib::ftlib_get_spawn_num;
pub use crate::tu::ftlib::ftlib_8008746c;
pub use crate::tu::ftlib::ftlib_is_sub_fighter;
pub use crate::tu::ftlib::ftlib_800874cc;
pub use crate::tu::ftlib::ftlib_is_frames_remaining;
pub use crate::tu::ftlib::ftlib_is_charging_smash;
pub use crate::tu::ftlib::ftlib_get_last_hit_source_type;
pub use crate::tu::ftlib::ftlib_get_last_hit_source_kind;
pub use crate::tu::ftlib::ftlib_get_game_watch_color;
pub use crate::tu::ftlib::ftlib_get_game_watch_outline_color;
pub use crate::tu::ftlib::ftlib_is_being_nudged;
