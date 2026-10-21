//! Calls out of group 3 into code it does not own: other groups (sfx, items...) and the PC
//! port's add-ons (`FT_SCRIPT_VALUE` / `ScriptGame_ModValue`, Geno). Each is a settable
//! function pointer whose default is the vanilla behaviour, so a vanilla match needs none set.
//!
//! Single-threaded game state, like the C globals they replace: set them before the match.
#![allow(static_mut_refs)]

use crate::types::{ftCommonData, CmSubject, Fighter, Fighter_ShakeTable_t, FighterKind, HSD_GObj, HSD_JObj, Item_GObj, S32Vec2, Vec2, Vec3};

/// `ScriptGame_ModValue(player_id, field, base)` behind `FT_SCRIPT_VALUE` (ft/forward.h).
pub static mut SCRIPT_MOD_VALUE: Option<fn(player_id: i32, field: i32, base: f32) -> f32> = None;
/// `Geno_FallLimit(fp, terminal)` (ftCommon_Fall, TARGET_PC).
pub static mut GENO_FALL_LIMIT: Option<fn(fp: &Fighter, terminal: f32) -> f32> = None;
/// `ft_PlaySFX(fp, id, volume, pan)` (ft/ft_0877.c; sound is group 1).
pub static mut PLAY_SFX: Option<fn(fp: &mut Fighter, id: i32, vol: i32, pan: i32)> = None;
/// `ft_GetGroundFrictionMultiplier(fp)` (ft/ft_0877.c, reads the stage's floor material: group 2).
pub static mut GROUND_FRICTION_MUL: Option<fn(fp: &Fighter) -> f32> = None;

/// `p_ftCommonData` (fighter.h): PlCo's common fighter constants, loaded from the DAT.
/// TODO(move-to-core): owned by the fighter loader once ftdata is ported.
pub static mut P_FT_COMMON_DATA: *const ftCommonData = core::ptr::null();

/// `FT_SCRIPT_VALUE(fp, field, base)`.
#[inline]
pub fn script_value(fp: &Fighter, field: i32, base: f32) -> f32 {
    match unsafe { SCRIPT_MOD_VALUE } {
        Some(f) => f(fp.player_id as i32, field, base),
        None => base,
    }
}

#[inline]
pub fn geno_fall_limit(fp: &Fighter, terminal: f32) -> f32 {
    match unsafe { GENO_FALL_LIMIT } {
        Some(f) => f(fp, terminal),
        None => terminal,
    }
}

#[inline]
pub fn play_sfx(fp: &mut Fighter, id: i32, vol: i32, pan: i32) {
    if let Some(f) = unsafe { PLAY_SFX } {
        f(fp, id, vol, pan)
    }
}

#[inline]
pub fn ground_friction_mul(fp: &Fighter) -> f32 {
    match unsafe { GROUND_FRICTION_MUL } {
        Some(f) => f(fp),
        None => 1.0,
    }
}

/// `*p_ftCommonData`. Panics if the common data has not been installed.
#[inline]
pub fn common_data() -> &'static ftCommonData {
    let p = unsafe { P_FT_COMMON_DATA };
    assert!(!p.is_null(), "p_ftCommonData not loaded");
    unsafe { &*p }
}

// ---- round 2 (ftcommon.c field logic) ----------------------------------------------------
// Calls whose owner is not ported yet. A hook with no vanilla-neutral default panics when it is
// reached unset ("TODO(group): ..."), so a missing port is loud, never a silent wrong value.
// Rumble (lb/HSD pad) is output only, never game state, so its default is a no-op.

/// `Geno_GroundEdge(fp, landing)` (pc/geno, TARGET_PC). Vanilla: returns at once (no Geno fighter).
pub static mut GENO_GROUND_EDGE: Option<fn(fp: &mut Fighter, landing: i32)> = None;
/// TODO(group3 pl): `pl_8003FC44(slot, is_sub)` (pl/plbonuslib.c), not ported.
pub static mut PL_8003FC44: Option<fn(slot: i32, sub: i32)> = None;
/// TODO(group3 pl): `pl_800402D0(slot, is_sub, mashed)` (pl/plbonuslib.c), not ported.
pub static mut PL_800402D0: Option<fn(slot: i32, sub: i32, r: i32)> = None;
/// TODO(group3 pl): `pl_80040460(slot, is_sub)` (pl/plbonuslib.c), not ported.
pub static mut PL_80040460: Option<fn(slot: i32, sub: i32)> = None;
/// `Player_8003544C(slot, is_sub)` override; unset = `crate::player::player_8003544c`.
pub static mut PLAYER_8003544C: Option<fn(slot: i32, sub: i32) -> bool> = None;
/// `Player_UpdateJoystickCountByIndex` override; unset = `crate::player` port.
pub static mut PLAYER_UPDATE_JOYSTICK_COUNT: Option<fn(slot: i32, sub: i32)> = None;
/// TODO(group1 lb): `lb_80014574(port, id, a, b)` (rumble start). Default no-op.
pub static mut LB_RUMBLE_START: Option<fn(port: u8, id: i32, a: i32, b: i32)> = None;
/// TODO(group1 HSD pad): `HSD_PadRumbleRemoveId(port, id)`. Default no-op.
pub static mut HSD_PAD_RUMBLE_REMOVE_ID: Option<fn(port: u8, id: i32)> = None;
/// TODO(group1 lb): `lb_800145C0(port)` (rumble stop). Default no-op.
pub static mut LB_RUMBLE_STOP: Option<fn(port: u8)> = None;
/// TODO(group6 un): `un_803224DC(spawn, pos_x, kb_mag)`, not ported.
pub static mut UN_803224DC: Option<fn(spawn: i32, x: f32, kb: f32) -> bool> = None;
/// TODO(group5 it): `it_80294E78(item, scale)`.
pub static mut IT_80294E78: Option<fn(item: *mut Item_GObj, f: f32)> = None;
/// TODO(group5 it): `it_8029A89C(item, scale)`.
pub static mut IT_8029A89C: Option<fn(item: *mut Item_GObj, f: f32)> = None;
/// TODO(group5 it): `it_8029A8F4(item, &scale)`.
pub static mut IT_8029A8F4: Option<fn(item: *mut Item_GObj, v: &Vec3)> = None;
/// TODO(group3 ftCommon states): `ftCo_80090690(fp, &shift)` (non-NULL -> Some(shift)).
pub static mut FTCO_80090690: Option<fn(fp: &mut Fighter) -> Option<Vec2>> = None;
/// TODO(group3 ftCommon states): `ftCo_800DEEE8(fp, &shift)` (non-NULL -> Some(shift)).
pub static mut FTCO_800DEEE8: Option<fn(fp: &mut Fighter) -> Option<Vec2>> = None;
/// `Fighter_GrabMashShake` (fighter.h, PlCo data). TODO(move-to-core): fighter loader.
pub static mut FIGHTER_GRAB_MASH_SHAKE: *const Fighter_ShakeTable_t = core::ptr::null();

#[inline]
fn need<T>(h: Option<T>, what: &str) -> T {
    match h {
        Some(f) => f,
        None => panic!("{what} is not ported; install its hook (melee_fighter::hooks)"),
    }
}

pub fn geno_ground_edge(fp: &mut Fighter, landing: i32) {
    if let Some(f) = unsafe { GENO_GROUND_EDGE } {
        f(fp, landing)
    }
}
pub fn pl_8003fc44(slot: i32, sub: i32) { need(unsafe { PL_8003FC44 }, "TODO(group3 pl) pl_8003FC44")(slot, sub) }
pub fn pl_800402d0(slot: i32, sub: i32, r: i32) { need(unsafe { PL_800402D0 }, "TODO(group3 pl) pl_800402D0")(slot, sub, r) }
pub fn pl_80040460(slot: i32, sub: i32) { need(unsafe { PL_80040460 }, "TODO(group3 pl) pl_80040460")(slot, sub) }
/// Unset: the port in `crate::player` (port round 2).
pub fn player_8003544c(slot: i32, sub: i32) -> bool {
    match unsafe { PLAYER_8003544C } {
        Some(f) => f(slot, sub),
        None => crate::player::player_8003544c(slot, sub),
    }
}
/// Unset: the port in `crate::player` (port round 2).
pub fn player_update_joystick_count(slot: i32, sub: i32) {
    match unsafe { PLAYER_UPDATE_JOYSTICK_COUNT } {
        Some(f) => f(slot, sub),
        None => crate::player::update_joystick_count_by_index(slot, sub),
    }
}
pub fn lb_rumble_start(port: u8, id: i32, a: i32, b: i32) {
    if let Some(f) = unsafe { LB_RUMBLE_START } {
        f(port, id, a, b)
    }
}
pub fn hsd_pad_rumble_remove_id(port: u8, id: i32) {
    if let Some(f) = unsafe { HSD_PAD_RUMBLE_REMOVE_ID } {
        f(port, id)
    }
}
pub fn lb_rumble_stop(port: u8) {
    if let Some(f) = unsafe { LB_RUMBLE_STOP } {
        f(port)
    }
}
pub fn un_803224dc(spawn: i32, x: f32, kb: f32) -> bool { need(unsafe { UN_803224DC }, "TODO(group6 un) un_803224DC")(spawn, x, kb) }
pub fn it_80294e78(item: *mut Item_GObj, f: f32) { need(unsafe { IT_80294E78 }, "TODO(group5 it) it_80294E78")(item, f) }
pub fn it_8029a89c(item: *mut Item_GObj, f: f32) { need(unsafe { IT_8029A89C }, "TODO(group5 it) it_8029A89C")(item, f) }
pub fn it_8029a8f4(item: *mut Item_GObj, v: &Vec3) { need(unsafe { IT_8029A8F4 }, "TODO(group5 it) it_8029A8F4")(item, v) }
pub fn ftco_80090690(fp: &mut Fighter) -> Option<Vec2> { need(unsafe { FTCO_80090690 }, "TODO(group3 ftCo) ftCo_80090690")(fp) }
pub fn ftco_800deee8(fp: &mut Fighter) -> Option<Vec2> { need(unsafe { FTCO_800DEEE8 }, "TODO(group3 ftCo) ftCo_800DEEE8")(fp) }
/// `*Fighter_GrabMashShake`. Panics if not installed.
pub fn grab_mash_shake() -> &'static Fighter_ShakeTable_t {
    let p = unsafe { FIGHTER_GRAB_MASH_SHAKE };
    assert!(!p.is_null(), "Fighter_GrabMashShake not loaded");
    unsafe { &*p }
}

// ---- port round 2 (pl/player.c) ----------------------------------------------------------
// `G` is a fighter entity (`HSD_GObj*`). Each hook panics when reached unset.

/// `ScriptGame_ModValue(slot, field, base)` with a player slot (player.c's attack/defense ratio).
#[inline]
pub fn script_mod_value(slot: i32, field: i32, base: f32) -> f32 {
    match unsafe { SCRIPT_MOD_VALUE } {
        Some(f) => f(slot, field, base),
        None => base,
    }
}

macro_rules! needed_hooks {
    ($($(#[$m:meta])* $stat:ident $func:ident ($($a:ident: $t:ty),*) $(-> $r:ty)? = $what:literal;)*) => {$(
        $(#[$m])*
        pub static mut $stat: Option<fn($($t),*) $(-> $r)?> = None;
        #[doc = concat!("Calls `", stringify!($stat), "`; ", $what, ".")]
        pub fn $func($($a: $t),*) $(-> $r)? { need(unsafe { $stat }, $what)($($a),*) }
    )*};
}
type G = *mut HSD_GObj;
needed_hooks! {
    FTKB_SPECIALN_800EED50 ftkb_specialn_800eed50(a: i32, b: i32) = "TODO(group4 ftKb) ftKb_SpecialN_800EED50";
    FTCO_800D4F24 ftco_800d4f24(g: G, i: i32) = "TODO(group3 ftCo) ftCo_800D4F24";
    HSD_GOBJ_FREE hsd_gobj_free(g: G) = "TODO(group1 HSD) HSD_GObjFree";
    FTCO_800D4FF4 ftco_800d4ff4(g: G) = "TODO(group3 ftCo) ftCo_800D4FF4";
    IFSTATUS_802F6E1C ifstatus_802f6e1c(slot: i32) = "TODO(group6 if) ifStatus_802F6E1C";
    FTDEMO_SET_FACING_DIRECTION ftdemo_set_facing_direction(g: G, d: f32) = "TODO(group3 ftdemo) ftDemo_SetFacingDirection";
    FT_80087BEC ft_80087bec(g: G, v: i32) = "TODO(group3 ftlib) ft_80087BEC";
    FT_80087BC0 ft_80087bc0(g: G, v: i32) = "TODO(group3 ftlib) ft_80087BC0";
    GM_8016C5C0 gm_8016c5c0(slot: i32) -> i32 = "TODO(group6 gm) gm_8016C5C0";
    IFSTATUS_802F6D10 ifstatus_802f6d10(slot: i32) = "TODO(group6 if) ifStatus_802F6D10";
    GM_8016B168 gm_8016b168() -> bool = "TODO(group6 gm) gm_8016B168";
    IFSTATUS_802F69C0 ifstatus_802f69c0(a: i32, b: i32) = "TODO(group6 if) ifStatus_802F69C0";
    IFSTATUS_802F6AF8 ifstatus_802f6af8(slot: i32) = "TODO(group6 if) ifStatus_802F6AF8";
    GM_GET_FRAME_COUNT gm_get_frame_count() -> u32 = "TODO(group6 gm) gm_GetFrameCount";
    IFSTATUS_802F6C04 ifstatus_802f6c04(slot: i32) = "TODO(group6 if) ifStatus_802F6C04";
    FT_80087AC0 ft_80087ac0(g: G, v: i32) = "TODO(group3 ftlib) ft_80087AC0";
    FT_80087BAC ft_80087bac(g: G, v: i32) = "TODO(group3 ftlib) ft_80087BAC";
}

// ---- port round 3 (ft/ftlib.c) -----------------------------------------------------------
// ftLib_* hooks that player.c calls now default to the ports in `crate::ftlib`; setting one
// overrides it (the player.c test does, with trace stubs).

macro_rules! ported_hooks {
    ($($stat:ident $func:ident ($($a:ident: $t:ty),*) $(-> $r:ty)? = $port:path;)*) => {$(
        #[doc = concat!("Override for `", stringify!($port), "`; unset = the port.")]
        pub static mut $stat: Option<fn($($t),*) $(-> $r)?> = None;
        pub fn $func($($a: $t),*) $(-> $r)? {
            match unsafe { $stat } { Some(f) => f($($a),*), None => $port($($a),*) }
        }
    )*};
}
ported_hooks! {
    FTLIB_800867E8 ftlib_800867e8(g: G) = crate::ftlib::ftlib_800867e8;
    FTLIB_8008688C ftlib_8008688c(g: G) = crate::ftlib::ftlib_8008688c;
    FTLIB_8008701C ftlib_8008701c(g: G) -> bool = crate::ftlib::ftlib_8008701c;
    FTLIB_800873CC ftlib_800873cc(g: G) -> bool = crate::ftlib::ftlib_800873cc;
    FTLIB_80087140 ftlib_80087140(g: G) = crate::ftlib::ftlib_80087140;
    FTLIB_80086664 ftlib_80086664(g: G, v: &Vec3) = crate::ftlib::ftlib_80086664;
    FTLIB_800869F8 ftlib_800869f8(g: G) -> f32 = crate::ftlib::ftlib_800869f8;
    FTLIB_SET_SCALE ftlib_set_scale(g: G, s: f32) = crate::ftlib::set_scale;
    FTLIB_800870F0 ftlib_800870f0(g: G, v: i32) = crate::ftlib::ftlib_800870f0;
    FTLIB_80086F4C ftlib_80086f4c(g: G) -> bool = crate::ftlib::ftlib_80086f4c;
    FTLIB_80086F80 ftlib_80086f80(g: G) -> f32 = crate::ftlib::ftlib_80086f80;
    FTLIB_GET_KIND ftlib_get_kind(g: G) -> FighterKind = crate::ftlib::get_kind;
    FTLIB_80087300 ftlib_80087300(g: G) -> i32 = crate::ftlib::ftlib_80087300;
    FTLIB_80086A4C ftlib_80086a4c(g: G, v: f32) = crate::ftlib::ftlib_80086a4c;
    FTLIB_80086BB4 ftlib_80086bb4(g: G) -> bool = crate::ftlib::ftlib_80086bb4;
    FTLIB_80086B90 ftlib_80086b90(g: G, v: &mut Vec3) = crate::ftlib::ftlib_80086b90;
    PLAYER_GET_MORE_FLAGS_BIT4 player_get_more_flags_bit4(slot: i32) -> u8 = crate::player::get_more_flags_bit4;
}

/// TARGET_PC `Geno_DefineTemplateSkeleton(kind)`. Unset (vanilla, no Geno defines): 0.
pub static mut GENO_DEFINE_TEMPLATE_SKELETON: Option<fn(kind: i32) -> i32> = None;
pub fn geno_define_template_skeleton(kind: i32) -> i32 {
    match unsafe { GENO_DEFINE_TEMPLATE_SKELETON } { Some(f) => f(kind), None => 0 }
}

needed_hooks! {
    /// `HSD_GObjPLinkHead[link]`.
    PLINK_HEAD plink_head(link: i32) -> G = "TODO(group1 HSD) HSD_GObjPLinkHead";
    HSD_RANDI hsd_randi(n: i32) -> i32 = "TODO(group1 HSD) HSD_Randi";
    LB_8000B1CC lb_8000b1cc(j: *mut HSD_JObj, off: &Vec3, out: &mut Vec3) = "TODO(group1 lb) lb_8000B1CC";
    /// TARGET_PC `Slippi_Codes()` (pc/platform).
    SLIPPI_CODES slippi_codes() -> i32 = "TODO(pc) Slippi_Codes";
    FTPARTS_GET_BONE_INDEX ftparts_get_bone_index(fp: &Fighter, i: i32) -> i32 = "TODO(group3 ftparts) ftParts_GetBoneIndex";
    FIGHTER_UPDATE_MODEL_SCALE fighter_update_model_scale(g: G) = "TODO(group3 fighter) Fighter_UpdateModelScale";
    FTCOMMON_8007E6DC ftcommon_8007e6dc(g: G, o: G, i: i32) = "TODO(group3 ftcommon) ftCommon_8007E6DC";
    FTCOMMON_8007E82C ftcommon_8007e82c(g: G) = "TODO(group3 ftcommon) ftCommon_8007E82C";
    FTCOMMON_8007D92C ftcommon_8007d92c(g: G) = "TODO(group3 ftcommon) ftCommon_8007D92C";
    FIGHTER_RESET_INPUT_DATA fighter_reset_input_data(g: G) = "TODO(group3 fighter) Fighter_ResetInputData_80068854";
    FIGHTER_UNK_PROCESS_DEATH fighter_unk_process_death(g: G) = "TODO(group3 fighter) Fighter_UnkProcessDeath_80068354";
    CAMERA_80030A50 camera_80030a50() -> G = "TODO(group2 cm) Camera_80030A50";
    CAMERA_LOGIC_TO_SCREEN camera_logic_to_screen(s: *mut CmSubject, out: &mut S32Vec2) -> bool = "TODO(group2 cm) Camera_LogicToScreen";
    CAMERA_80031154 camera_80031154(v: &mut Vec3) -> bool = "TODO(group2 cm) Camera_80031154";
    FTDATA_80085560 ftdata_80085560(a: i32, b: i32) = "TODO(group3 ftdata) ftData_80085560";
    GENO_DEFINE_BASE_KIND geno_define_base_kind(kind: i32) -> i32 = "TODO(pc geno) Geno_DefineBaseKind";
    MEX_SSM_FOR_PORT_KIND mex_ssm_for_port_kind(kind: i32) -> i32 = "TODO(pc mex) Mex_SsmForPortKind";
    PL_8003E4A4 pl_8003e4a4(slot: i32, sub: i32, p: *mut core::ffi::c_void, a: i32) = "TODO(group3 pl) pl_8003E4A4";
    FTANIM_IS_FRAMES_REMAINING ftanim_is_frames_remaining(g: G) -> bool = "TODO(group3 ftanim) ftAnim_IsFramesRemaining";
    FTGW_INIT_8014A7F4 ftgw_init_8014a7f4(g: G, d: *mut core::ffi::c_void) = "TODO(group4 ftGw) ftGw_Init_8014A7F4";
    FTGW_INIT_8014A814 ftgw_init_8014a814(g: G, d: *mut core::ffi::c_void) = "TODO(group4 ftGw) ftGw_Init_8014A814";
    FTKB_INIT_800EEB00 ftkb_init_800eeb00(g: G, d: *mut core::ffi::c_void) = "TODO(group4 ftKb) ftKb_Init_800EEB00";
    FTKB_INIT_800EEB1C ftkb_init_800eeb1c(g: G, d: *mut core::ffi::c_void) = "TODO(group4 ftKb) ftKb_Init_800EEB1C";
}
