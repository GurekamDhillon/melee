//! Hand port, round 3: src/melee/ft/ftlib.c, the fighter accessor library other code calls
#![allow(non_upper_case_globals)]
//! with a fighter `HSD_GObj*`. Functions keep the C signature shape: `g` is the raw gobj
//! (`G = *mut HSD_GObj`), `gobj->user_data` is the `Fighter`. Callers must pass a live fighter
//! gobj, as in C (the C dereferences without checks too).
//!
//! The fighter list (`HSD_GObjPLinkHead[HSD_GOBJ_PLINK_FIGHTER]`) comes from
//! `hooks::plink_head`. Calls out of the ported set go through `crate::hooks`.
//! Fused sites (`tools/rust-port/fma_sites.py src/melee/ft/ftlib.c`): ftLib_8008627C and
//! ftLib_80086368, one f32 fmuladd each (`dx*dx + dy*dy` -> `fmadds(dx, dx, dy*dy)`).
//! TARGET_PC paths are the ones ported (80086A8C returns true; Slippi Whispy fix in 800864A8;
//! Geno template kinds in GetKind; m-ex crowd sfx in 8008746C; UpdateLogicOffCamera).
//!
//! Not ported from ftlib.c: ftLib_800871A8 (metal box: ftCo/it/camera), ftLib_80087508 /
//! 80087574 / 80087610 (archive preloading), ftLib_8008777C (atan2f; depends on group 1's MSL).
//! Checked against the C (cut verbatim, shipping contraction) by tests/ftlib_ref.rs.

use crate::ftcommon2 as c2;
use crate::hooks as h;
use crate::types::*;
use melee_core::fp::fmadds;

pub type G = *mut HSD_GObj;

#[inline(always)]
fn fp<'a>(g: G) -> &'a mut Fighter {
    unsafe { &mut *((*g).user_data as *mut Fighter) }
}

/// Iterate the fighter p-link list like the C `for (cur = head; cur; cur = cur->next)`.
/// `next` is read before calling `f` (the C reads it after; no ported body unlinks).
fn each(mut f: impl FnMut(G) -> bool) -> G {
    let mut cur = h::plink_head(HSD_GOBJ_PLINK_FIGHTER);
    while !cur.is_null() {
        if f(cur) {
            return cur;
        }
        cur = unsafe { (*cur).next };
    }
    core::ptr::null_mut()
}

/// `ftLib_800860C4`: number of fighter gobjs.
pub fn ftlib_800860c4() -> i32 {
    let mut n = 0;
    each(|_| {
        n += 1;
        false
    });
    n
}

pub fn is_master_hand_present() -> bool {
    !each(|c| get_kind(c) == Ft_Kind_MasterH).is_null()
}

pub fn is_crazy_hand_present() -> bool {
    !each(|c| get_kind(c) == Ft_Kind_CrezyH).is_null()
}

#[inline]
fn skip_opponent(fp_own: Option<&Fighter>, cur: &Fighter) -> bool {
    cur.x221F_b3() != 0 || (h::gm_8016b168() && fp_own.is_some_and(|f| cur.team == f.team))
}

/// `ftLib_80086198`: the opponent with the lowest percent.
pub fn ftlib_80086198(g: G) -> G {
    let own = if g.is_null() { None } else { Some(&*fp(g)) };
    let mut min_percent: i32 = 999;
    let mut result: G = core::ptr::null_mut();
    each(|cur| {
        if !ftlib_80086fd4(g, cur) {
            let cf = fp(cur);
            if cf.x221F_b3() != 0 {
                return false;
            }
            if h::gm_8016b168() && own.is_some_and(|f| cf.team == f.team) {
                return false;
            }
            if cf.dmg.x1830_percent < min_percent as f32 {
                min_percent = cf.dmg.x1830_percent as i32;
                result = cur;
            }
        }
        false
    });
    result
}

/// `ftLib_8008627C`: the closest opponent to `pos` (camera target bone).
pub fn ftlib_8008627c(pos: &Vec3, g: G) -> G {
    let own = if g.is_null() { None } else { Some(&*fp(g)) };
    let mut min_dist = f32::MAX;
    let mut result: G = core::ptr::null_mut();
    each(|cur| {
        if ftlib_80086fd4(g, cur) || skip_opponent(own, fp(cur)) {
            return false;
        }
        let mut v = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        ftlib_800866dc(cur, &mut v);
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

/// `ftLib_80086368`: the closest opponent on the `facing_dir` side of `v`.
pub fn ftlib_80086368(v: &Vec3, g: G, facing_dir: f32) -> G {
    let own = if g.is_null() { None } else { Some(&*fp(g)) };
    let mut min_diff = f32::MAX;
    let mut result: G = core::ptr::null_mut();
    each(|cur| {
        if ftlib_80086fd4(g, cur) || skip_opponent(own, fp(cur)) {
            return false;
        }
        let mut p = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        ftlib_800866dc(cur, &mut p);
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

/// `ftLib_800864A8`: which side (-1 / +1) most opponents are on; ties draw `HSD_Randi(2)`.
pub fn ftlib_800864a8(v: &Vec3, g: G) -> f32 {
    let own = if g.is_null() { None } else { Some(&*fp(g)) };
    let mut result: i32 = 0;
    each(|cur| {
        if ftlib_80086fd4(g, cur) {
            return false;
        }
        let cf = fp(cur);
        if skip_opponent(own, cf) {
            return false;
        }
        // TARGET_PC: Slippi WhispyBlowDirFix.
        if h::slippi_codes() == 2 && cf.motion_id <= 0xB {
            return false;
        }
        let mut p = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        ftlib_800866dc(cur, &mut p);
        result += if p.x - v.x < 0.0 { -1 } else { 1 };
        false
    });
    if result == 0 {
        result = if h::hsd_randi(2) != 0 { 1 } else { -1 };
    }
    if result < 0 { -1.0 } else { 1.0 }
}

pub fn ftlib_800865c0(g: G) -> f32 { fp(g).facing_dir }
pub fn ftlib_800865cc(g: G) -> i32 { fp(g).ground_or_air as i32 }
pub fn ftlib_800865d8(g: G, x: &mut f32, y: &mut f32) {
    let f = fp(g);
    *x = f.input.lstick[0].x;
    *y = f.input.lstick[0].y;
}
/// `ftLib_800865F0`: the joint of bone `ftParts_GetBoneIndex(fp, 4)`.
pub fn ftlib_800865f0(g: G) -> *mut HSD_JObj {
    let f = fp(g);
    let i = h::ftparts_get_bone_index(f, 4);
    unsafe { (*f.parts.offset(i as isize)).joint }
}
pub fn ftlib_80086630(g: G, part: i32) -> *mut HSD_JObj {
    unsafe { (*fp(g).parts.offset(part as isize)).joint }
}
pub fn ftlib_80086644(g: G, pos: &mut Vec3) { unsafe { melee_world::ftlib::ftLib_80086644(g, pos) } }
pub fn ftlib_80086664(g: G, pos: &Vec3) { fp(g).cur_pos = *pos; }
pub fn ftlib_80086684(g: G, pos: &mut Vec3) { unsafe { melee_world::ftlib::ftLib_80086684(g, pos) } }

pub fn set_scale(g: G, val: f32) {
    fp(g).x34_scale.y = val;
    h::fighter_update_model_scale(g);
    c2::ftcommon_80080174(fp(g));
}

/// `ftLib_800866DC`: the camera target bone's world position (`lb_8000B1CC`).
pub fn ftlib_800866dc(g: G, v: &mut Vec3) {
    let f = fp(g);
    let i = unsafe { (*(*f.ft_data).x0).camera_zoom_target_bone };
    let j = ftlib_80086630(g, i);
    let off = f.co_attrs.x170;
    h::lb_8000b1cc(j, &off, v);
}

pub fn ftlib_80086724(g: G, other: G) {
    let f = fp(g);
    if !f.item_gobj.is_null() || !f.x1978.is_null() {
        h::ftcommon_8007e6dc(g, other, 1);
    }
}
pub fn ftlib_80086764(g: G) {
    if !fp(g).x1984_heldItemSpec.is_null() {
        h::ftcommon_8007e82c(g);
    }
}
pub fn get_item(g: G) -> G { fp(g).item_gobj }
pub fn ftlib_800867a0(g: G, arg1: G) -> bool { get_item(g) == arg1 || fp(g).x1978 == arg1 }
pub fn ftlib_800867cc(g: G) -> G { fp(g).x1984_heldItemSpec }
pub fn ftlib_800867d8(g: G) -> bool { fp(g).x221D_b4() != 0 }
pub fn ftlib_800867e8(g: G) {
    h::fighter_reset_input_data(g);
    fp(g).set_x221D_b4(1);
}
pub fn ftlib_80086824() {
    each(|c| {
        h::ftlib_800867e8(c);
        false
    });
}
pub fn ftlib_8008688c(g: G) { fp(g).set_x221D_b4(0); }
pub fn ftlib_800868a4() {
    each(|c| {
        h::ftlib_8008688c(c);
        false
    });
}

pub fn ftlib_800868d4(g: G, arg1: G) -> bool {
    let f = fp(g);
    if f.item_gobj != arg1 && f.x197C != arg1 && f.x1980 != arg1 {
        return true;
    }
    !(f.invisible() != 0
        || f.x221E_b5() != 0
        || (f.x221E_b3() == 0 && f.item_gobj == arg1)
        || (f.x221E_b4() == 0 && (f.x197C == arg1 || f.x1980 == arg1)))
}

/// `ftLib_80086960`: is `arg` a fighter gobj.
pub fn ftlib_80086960(arg: G) -> bool { unsafe { melee_world::ftlib::ftLib_80086960(arg) } }
pub fn ftlib_80086984(g: G) -> *mut CollData { unsafe { melee_world::ftlib::ftLib_80086984(g) } }
pub fn ftlib_80086990(g: G, v: &mut Vec3) {
    let f = fp(g);
    let tmp = 0.5f32 * (f.coll_data.ecb.top.y + f.coll_data.ecb.bottom.y);
    v.x = f.cur_pos.x + 0.0;
    v.y = f.cur_pos.y + tmp;
    v.z = f.cur_pos.z + 0.0;
}
pub fn ftlib_800869d4(g: G) -> f32 { c2::get_model_scale(fp(g)) }
pub fn ftlib_800869f8(g: G) -> f32 { fp(g).x34_scale.x * fp(g).co_attrs.model_scaling }
pub fn ftlib_80086a0c(g: G) -> f32 { fp(g).x34_scale.y }
pub fn ftlib_80086a18(g: G) -> bool {
    matches!(fp(g).motion_id as i32, ftCo_MS_GuardOn | ftCo_MS_Guard | ftCo_MS_GuardSetOff)
}
pub fn ftlib_80086a4c(g: G, val: f32) { unsafe { melee_world::ftlib::ftLib_80086A4C(g, val) } }
pub fn ftlib_80086a58(g: G, x: &mut S32Vec2) -> bool {
    let f = fp(g);
    if f.x2229_b3() == 0 && f.x2220_b7() == 0 {
        *x = f.x2188;
    }
    false
}
/// `ftLib_80086A8C` (TARGET_PC): always true; the flag write moved to `update_logic_off_camera`.
pub fn ftlib_80086a8c(_g: G) -> bool { true }
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
pub fn ftlib_80086b64(g: G) -> bool { unsafe { melee_world::ftlib::ftLib_80086B64(g) } }
pub fn ftlib_80086b74(g: G) -> *mut CmSubject { unsafe { melee_world::ftlib::ftLib_80086B74(g) } }
pub fn ftlib_80086b80(g: G) -> f32 { unsafe { melee_world::ftlib::ftLib_80086B80(g) } }
pub fn ftlib_80086b90(g: G, v: &mut Vec3) { unsafe { melee_world::ftlib::ftLib_80086B90(g, v) } }
pub fn ftlib_80086bb4(g: G) -> bool {
    let cam = ftlib_80086b74(g);
    h::camera_80031154(unsafe { &mut (*cam).pos })
}
pub fn ftlib_80086be0(g: G) -> u8 { unsafe { melee_world::ftlib::ftLib_80086BE0(g) } }
pub fn ftlib_80086bec(g: G, v: &mut Vec3) { unsafe { melee_world::ftlib::ftLib_80086BEC(g, v) } }
pub fn get_motion_id(g: G) -> i32 { fp(g).motion_id as i32 }

#[inline]
fn rumble_helper(g: G, arg1: i32, arg2: i32, val: i32) {
    let f = fp(g);
    if !h::player_8003544c(f.player_id as i32, f.is_sub_fighter() as i32) {
        return;
    }
    if f.x221F_b3() == 0 && f.x2224_b2() == 0 {
        h::lb_rumble_start(f.x618_player_id, val, arg1, arg2);
    }
}
pub fn ftlib_80086c18(g: G, a: i32, b: i32) { rumble_helper(g, a, b, 0); }
pub fn ftlib_80086c9c(a: i32, b: i32) {
    each(|c| {
        ftlib_80086c18(c, a, b);
        false
    });
}
pub fn ftlib_80086d40(g: G, a: i32, b: i32) { rumble_helper(g, a, b, 1); }
pub fn ftlib_80086dc4(a: i32, b: i32) {
    each(|c| {
        ftlib_80086d40(c, a, b);
        false
    });
}
pub fn ftlib_80086e68(g: G) {
    let f = fp(g);
    if h::player_8003544c(f.player_id as i32, f.is_sub_fighter() as i32) {
        h::hsd_pad_rumble_remove_id(f.x618_player_id, 1);
    }
}
pub fn ftlib_80086eb4(g: G) -> i32 { fp(g).team as i32 }
pub fn ftlib_80086ec0(g: G) -> bool { unsafe { melee_world::ftlib::ftLib_80086EC0(g) } }
pub fn ftlib_80086ed0(g: G) -> bool {
    let f = fp(g);
    !(f.invisible() != 0
        || f.x221E_b2() != 0
        || h::player_get_more_flags_bit4(f.player_id as i32) != 0
        || f.is_sandbag() != 0
        || f.x2229_b3() != 0
        || f.x2220_b7() != 0)
}
pub fn ftlib_80086f4c(g: G) -> bool {
    let f = fp(g);
    f.x221F_b2() == 0 && f.x221F_b1() == 0 && f.x209A == 1
}
pub fn ftlib_80086f80(g: G) -> f32 {
    let f = fp(g);
    if f.invisible() != 0 {
        return unsafe { (*(*f.ft_data).x0).name_tag_height };
    }
    f.co_attrs.name_tag_height
}
pub fn ftlib_80086fa8(g: G) -> bool {
    matches!(get_motion_id(g),
        ftCo_MS_SwordSwing1 | ftCo_MS_SwordSwing3 | ftCo_MS_SwordSwing4 | ftCo_MS_SwordSwingDash)
}
/// `ftLib_80086FD4`: same gobj or same player slot.
pub fn ftlib_80086fd4(g0: G, g1: G) -> bool {
    if !g0.is_null() && !g1.is_null() {
        if g0 == g1 {
            return true;
        }
        if fp(g0).player_id == fp(g1).player_id {
            return true;
        }
    }
    false
}
pub fn ftlib_8008701c(g: G) -> bool { unsafe { melee_world::ftlib::ftLib_8008701C(g) } }
pub fn ftlib_8008702c(a: i32) { h::ftdata_80085560(a, 1); }
pub fn ftlib_80087050(a: i32) { h::ftdata_80085560(a, -1); }
pub fn ftlib_80087074(g: G, v: &mut Vec3) -> bool {
    let f = fp(g);
    if f.dmg.x18B8 != 0.0 || f.dmg.x18BC != 0.0 {
        v.x = f.dmg.x18B8;
        v.y = f.dmg.x18BC;
        v.z = 0.0;
        return true;
    }
    false
}
/// `ftLib_800870BC`: `p_ftCommonData->x6D8[controller_index]` (a flexible array in the C).
pub fn ftlib_800870bc(g: G, val: &mut i32) -> bool {
    let i = fp(g).x61A_controller_index;
    if i != 0 {
        *val = unsafe { *h::common_data().x6D8.as_ptr().add(i as usize) };
        return true;
    }
    false
}
/// `ftLib_800870F0`: `dmg.x1830_percent = x` (int -> float).
pub fn ftlib_800870f0(g: G, x: i32) { fp(g).dmg.x1830_percent = x as f32; }
/// `ftLib_80087120`: `(s32) dmg.x1830_percent` (PPC fctiwz saturates; NaN differs, see ftcommon).
pub fn ftlib_80087120(g: G) -> i32 { fp(g).dmg.x1830_percent as i32 }
pub fn ftlib_80087140(g: G) {
    if fp(g).x221F_b3() == 0 {
        h::ftco_800d4f24(g, 1);
        c2::ftcommon_8007ed2c(fp(g));
        h::fighter_unk_process_death(g);
        h::ftcommon_8007d92c(g);
    }
}
pub fn ftlib_80087284(g: G) -> bool { get_motion_id(g) >= ftCo_MS_LightThrowF4 }
/// `ftLib_GetKind` (TARGET_PC): a Geno define on a template skeleton answers as its base kind.
pub fn get_kind(g: G) -> FighterKind {
    let k = fp(g).kind;
    if k >= Ft_Kind_Mex0 && h::geno_define_template_skeleton(k as i32) != 0 {
        return h::geno_define_base_kind(k as i32) as FighterKind;
    }
    k
}
pub fn ftlib_800872b0(g: G) -> *mut LbShadow { unsafe { melee_world::ftlib::ftLib_800872B0(g) } }
pub fn ftlib_800872bc(g: G) -> bool {
    let f = fp(g);
    f.x221E_b1() == 0 && f.x2226_b4() == 0 && f.is_sub_fighter() == 0
}
pub fn ftlib_80087300(g: G) -> i32 { fp(g).dmg.x18c4_source_ply }
pub fn ftlib_8008730c(g: G) -> i32 { fp(g).x221F_b5() as i32 }
pub fn ftlib_8008731c(g: G) -> i32 { unsafe { melee_world::ftlib::ftLib_8008731C(g) } }
#[inline]
fn msid_in(g: G, lo: i32, hi: i32) -> bool {
    let m = get_motion_id(g);
    m >= lo && m <= hi
}
pub fn ftlib_8008732c(g: G) -> bool { unsafe { melee_world::ftlib::ftLib_8008732C(g) } }
pub fn ftlib_80087354(g: G) -> bool { msid_in(g, ftCo_MS_DeadUp, ftCo_MS_DeadUpFallHitCameraIce) }
pub fn ftlib_8008737c(g: G) -> bool { msid_in(g, ftCo_MS_DeadUpStar, ftCo_MS_DeadUpStarIce) }
pub fn ftlib_800873a4(g: G) -> bool { msid_in(g, ftCo_MS_DeadUpFall, ftCo_MS_DeadUpFallHitCameraIce) }
pub fn ftlib_800873cc(g: G) -> bool { msid_in(g, ftCo_MS_Rebirth, ftCo_MS_RebirthWait) }
pub fn ftlib_800873f4(g: G) -> bool { unsafe { melee_world::ftlib::ftLib_800873F4(g) } }
/// `ftLib_8008741C`: the fighter gobj with spawn number `i`.
pub fn ftlib_8008741c(i: u32) -> G { each(|c| fp(c).x8_spawnNum as u32 == i) }
pub fn ftlib_80087454(g: G) -> f32 { fp(g).dmg.x18A4_knockbackMagnitude }
pub fn ftlib_80087460(g: G) -> u32 { fp(g).x8_spawnNum as u32 }
/// `ftLib_8008746C` (TARGET_PC): the crowd cheer sfx id; m-ex relative ids (5000..9999) are
/// rebased onto the port kind's sound bank.
pub fn ftlib_8008746c(g: G) -> i32 {
    let f = fp(g);
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
pub fn ftlib_800874bc(g: G) -> i32 { unsafe { melee_world::ftlib::ftLib_800874BC(g) } }
pub fn ftlib_800874cc(g: G, arg1: *mut core::ffi::c_void, arg2: i32) {
    let f = fp(g);
    h::pl_8003e4a4(f.player_id as i32, f.is_sub_fighter() as i32, arg1, arg2);
}
pub fn ftlib_800876b4(g: G) { h::ftanim_is_frames_remaining(g); }
pub fn ftlib_800876d4(g: G) -> bool { fp(g).smash_attrs.state as i32 == 2 }
pub fn ftlib_800876f4(g: G) -> i32 { fp(g).dmg.x18CC }
pub fn ftlib_80087700(g: G) -> i32 { fp(g).dmg.x18D0 }
pub fn ftlib_8008770c(g: G, dst: *mut core::ffi::c_void) {
    if get_kind(g) == Ft_Kind_GameWatch { h::ftgw_init_8014a7f4(g, dst) } else { h::ftkb_init_800eeb00(g, dst) }
}
pub fn ftlib_80087744(g: G, dst: *mut core::ffi::c_void) {
    if get_kind(g) == Ft_Kind_GameWatch { h::ftgw_init_8014a814(g, dst) } else { h::ftkb_init_800eeb1c(g, dst) }
}
pub fn ftlib_800877d4(g: G) -> bool { fp(g).xF8_playerNudgeVel.x != 0.0 }
