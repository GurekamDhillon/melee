//! Calls out of group 3 into code it does not own: other groups (sfx, items...) and the PC
//! port's add-ons (`FT_SCRIPT_VALUE` / `ScriptGame_ModValue`, Geno). Each is a settable
//! function pointer whose default is the vanilla behaviour, so a vanilla match needs none set.
//!
//! Single-threaded game state, like the C globals they replace: set them before the match.
#![allow(static_mut_refs)]

use crate::types::{ftCommonData, CmSubject, Fighter, Fighter_804D6520_t, Fighter_ShakeTable_t, FighterKind, HSD_GObj, HSD_JObj, Item_GObj, S32Vec2, Vec2, Vec3};

/// `ScriptGame_ModValue(player_id, field, base)` behind `FT_SCRIPT_VALUE` (ft/forward.h).
pub static mut SCRIPT_MOD_VALUE: Option<fn(player_id: i32, field: i32, base: f32) -> f32> = None;
/// `Geno_FallLimit(fp, terminal)` (ftCommon_Fall, TARGET_PC).
pub static mut GENO_FALL_LIMIT: Option<fn(fp: &Fighter, terminal: f32) -> f32> = None;
/// Override for `ft_PlaySFX(fp, id, volume, pan)`; unset = `crate::ft0877::play_sfx` (round 5).
pub static mut PLAY_SFX: Option<fn(fp: &mut Fighter, id: i32, vol: i32, pan: i32)> = None;
/// `ft_GetGroundFrictionMultiplier(fp)` (ft/ft_0877.c, reads the stage's floor material: group 2).
pub static mut GROUND_FRICTION_MUL: Option<fn(fp: &Fighter) -> f32> = None;

/// `p_ftCommonData` (fighter.h): PlCo's common fighter constants, loaded from the DAT.
/// TODO(move-to-core): owned by the fighter loader once ftdata is ported.
pub static mut P_FT_COMMON_DATA: *const ftCommonData = core::ptr::null();

/// `Fighter_804D6520` (fighter.h, PlCo `pData[13]`): bunny-hood movement multipliers.
/// TODO(move-to-core): owned by the fighter loader (Fighter_LoadCommonData) once ftdata is ported.
pub static mut FIGHTER_804D6520: *const Fighter_804D6520_t = core::ptr::null();

/// `*Fighter_804D6520`. Panics if it has not been installed.
#[inline]
pub fn fighter_804d6520() -> &'static Fighter_804D6520_t {
    let p = unsafe { FIGHTER_804D6520 };
    assert!(!p.is_null(), "Fighter_804D6520 not loaded");
    unsafe { &*p }
}

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
    match unsafe { PLAY_SFX } {
        Some(f) => f(fp, id, vol, pan),
        // C parameters are u8: callers' values are truncated, as the C call converts them.
        None => crate::ft0877::play_sfx(fp, id, vol as u8, pan as u8),
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
/// `pl_8003FC44(slot, is_sub)` override; unset = `crate::plbonus` port (round 4).
pub static mut PL_8003FC44: Option<fn(slot: i32, sub: i32)> = None;
/// `pl_800402D0(slot, is_sub, mashed)` override; unset = `crate::plbonus` port (round 4).
pub static mut PL_800402D0: Option<fn(slot: i32, sub: i32, r: i32)> = None;
/// `pl_80040460(slot, is_sub)` override; unset = `crate::plbonus` port (round 4).
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
pub fn pl_8003fc44(slot: i32, sub: i32) { match unsafe { PL_8003FC44 } { Some(f) => f(slot, sub), None => crate::plbonus::pl_8003fc44(slot, sub) } }
pub fn pl_800402d0(slot: i32, sub: i32, r: i32) { match unsafe { PL_800402D0 } { Some(f) => f(slot, sub, r), None => crate::plbonus::pl_800402d0(slot, sub, r) } }
pub fn pl_80040460(slot: i32, sub: i32) { match unsafe { PL_80040460 } { Some(f) => f(slot, sub), None => crate::plbonus::pl_80040460(slot, sub) } }
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
    GM_8016C5C0 gm_8016c5c0(slot: i32) -> i32 = "TODO(group6 gm) gm_8016C5C0";
    IFSTATUS_802F6D10 ifstatus_802f6d10(slot: i32) = "TODO(group6 if) ifStatus_802F6D10";
    GM_8016B168 gm_8016b168() -> bool = "TODO(group6 gm) gm_8016B168";
    IFSTATUS_802F69C0 ifstatus_802f69c0(a: i32, b: i32) = "TODO(group6 if) ifStatus_802F69C0";
    IFSTATUS_802F6AF8 ifstatus_802f6af8(slot: i32) = "TODO(group6 if) ifStatus_802F6AF8";
    GM_GET_FRAME_COUNT gm_get_frame_count() -> u32 = "TODO(group6 gm) gm_GetFrameCount";
    IFSTATUS_802F6C04 ifstatus_802f6c04(slot: i32) = "TODO(group6 if) ifStatus_802F6C04";
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

// ---- port round 4 (pl/pl_040D.c, pl/plbonuslib.c tail) -------------------------------------

/// `pl_804D6470` (player.c .bss): plbonus.c's constants table, set by `pl_80038914`'s loader.
/// TODO(group3 plbonus): owned by the plbonus.c port.
pub static mut PL_804D6470: *const crate::types::pl_804D6470_t = core::ptr::null();
/// `*pl_804D6470`. Panics if the table has not been installed.
#[inline]
pub fn pl_804d6470() -> &'static crate::types::pl_804D6470_t {
    let p = unsafe { PL_804D6470 };
    assert!(!p.is_null(), "pl_804D6470 not loaded");
    unsafe { &*p }
}

// round 11: plbonus.c's primitives are ported (`crate::plbonus3`); the hooks stay settable.
ported_hooks! {
    PL_80038788 pl_80038788(slot: i32, kind: i32, n: i32) = crate::plbonus3::pl_80038788;
    PL_80038824 pl_80038824(slot: i32, kind: i32) = crate::plbonus3::pl_80038824;
    PL_80038898 pl_80038898(slot: i32, kind: i32) = crate::plbonus3::pl_80038898;
    PL_80039418 pl_80039418(slot: i32, kind: i32) -> u32 = crate::plbonus3::pl_80039418;
    PL_800386D8 pl_800386d8(s: &crate::types::plActionStats, n: i32) -> u32 = crate::plbonus3::pl_800386d8;
}
needed_hooks! {
    /// `gmDecisionGetType(kind)` (gm_16F1.c, a walk over the static `lbl_803D5A4C` table).
    GM_DECISION_GET_TYPE gm_decision_get_type(kind: i32) -> i32 = "TODO(group6 gm) gmDecisionGetType";
    GM_8016F2F8 gm_8016f2f8(kind: i32, slot: u8) -> i32 = "TODO(group6 gm) gm_8016F2F8";
    GM_8016B6E8 gm_8016b6e8(slot: i32, v: i32) = "TODO(group6 gm) gm_8016B6E8";
}

// ---- port round 5 (ft/ft_0877.c) ---------------------------------------------------------
// ft_0877 hooks that player.c / plbonus call now default to `crate::ft0877`.
ported_hooks! {
    FT_80087BEC ft_80087bec(g: G, v: i32) = crate::ft0877::ft_80087bec;
    FT_80087BC0 ft_80087bc0(g: G, v: i32) = crate::ft0877::ft_80087bc0;
    FT_80087AC0 ft_80087ac0(g: G, v: i32) = crate::ft0877::ft_80087ac0;
    FT_80087BAC ft_80087bac(g: G, v: i32) = crate::ft0877::ft_80087bac;
    FT_80087A8C ft_80087a8c(g: G) -> f32 = crate::ft0877::ft_80087a8c;
    FT_80087A98 ft_80087a98(g: G) -> i32 = crate::ft0877::ft_80087a98;
    FT_80087AA4 ft_80087aa4(g: G) -> bool = crate::ft0877::ft_80087aa4;
    FT_80087AB4 ft_80087ab4(g: G) -> u8 = crate::ft0877::ft_80087ab4;
}

/// TARGET_PC `Geno_DefineVoicePlayOwner(kind, sfx, owner)` (pc/geno). Unset (vanilla, no Geno
/// voice defines): 0, i.e. the normal sfx path.
pub static mut GENO_DEFINE_VOICE_PLAY_OWNER: Option<fn(kind: i32, sfx: i32, owner: *mut Fighter) -> i32> = None;
pub fn geno_define_voice_play_owner(kind: i32, sfx: i32, owner: *mut Fighter) -> i32 {
    match unsafe { GENO_DEFINE_VOICE_PLAY_OWNER } { Some(f) => f(kind, sfx, owner), None => 0 }
}

needed_hooks! {
    FTCO_800C06B4 ftco_800c06b4(fp: &mut Fighter) -> i32 = "TODO(group3 ftCo) ftCo_800C06B4";
    FTCO_800A101C ftco_800a101c(fp: &mut Fighter, kind: i32, level: i32, x14: i32) = "TODO(group3 ftCo) ftCo_800A101C";
    LBAUDIOAX_800233EC lbaudioax_800233ec(id: i32) -> i32 = "TODO(group1 lb) lbAudioAx_800233EC";
    LBAUDIOAX_80023130 lbaudioax_80023130(sfx: i32) -> i32 = "TODO(group1 lb) lbAudioAx_80023130";
    LBAUDIOAX_800230C8 lbaudioax_800230c8(ssm: i32, off: &mut i32, out: &mut i32) -> i32 = "TODO(group1 lb) lbAudioAx_800230C8";
    LBAUDIOAX_80023220 lbaudioax_80023220(ssm: i32) -> i32 = "TODO(group1 lb) lbAudioAx_80023220";
    LBAUDIOAX_80024DC4 lbaudioax_80024dc4(sfx: i32) = "TODO(group1 lb) lbAudioAx_80024DC4";
    LBAUDIOAX_80024FDC lbaudioax_80024fdc() = "TODO(group1 lb) lbAudioAx_80024FDC";
    LBAUDIOAX_80024FF4 lbaudioax_80024ff4() = "TODO(group1 lb) lbAudioAx_80024FF4";
    LBAUDIOAX_8002500C lbaudioax_8002500c(n: i32) = "TODO(group1 lb) lbAudioAx_8002500C";
    LBAUDIOAX_80025038 lbaudioax_80025038(n: i32) = "TODO(group1 lb) lbAudioAx_80025038";
    LBAUDIOAX_800237A8 lbaudioax_800237a8(sfx: i32, vol: i32, pan: i32) -> i32 = "TODO(group1 lb) lbAudioAx_800237A8";
    LBAUDIOAX_80024B94 lbaudioax_80024b94(h: i32, v: i32) -> i32 = "TODO(group1 lb) lbAudioAx_80024B94";
}

// ---- port round 6 (ft/ft_0881.c voice channels) ------------------------------------------
needed_hooks! {
    LBAUDIOAX_80023870 lbaudioax_80023870(sfx: i32, vol: i32, pan: i32, chan: i32) -> i32 = "TODO(group1 lb) lbAudioAx_80023870";
    LBAUDIOAX_80026510 lbaudioax_80026510(g: G) -> bool = "TODO(group1 lb) lbAudioAx_80026510";
    LBAUDIOAX_800265C4 lbaudioax_800265c4(g: G, sfx: i32) -> bool = "TODO(group1 lb) lbAudioAx_800265C4";
}

// ---- port round 7 (ft/ft_0892.c, pl/pltrick.c attack stats) ------------------------------
ported_hooks! {
    PL_80037C60 pl_80037c60(g: G, prev: crate::types::Struct2070) = crate::pltrick::pl_80037c60;
    FT_800898B4 ft_800898b4(g: G) -> *mut crate::types::ft_800898B4_t = crate::ft0892::ft_800898b4;
    PL_8003FE40 pl_8003fe40(slot: i32, sub: i32) = crate::plbonus::pl_8003fe40;
}
needed_hooks! {
    FTFX_APPEALS_CHECK_IF_USED ftfx_appeals_check_if_used(fp: &mut Fighter) -> bool = "TODO(group4 ftFx) ftFx_AppealS_CheckIfUsed";
    /// `ftData_80085FD4(fp, i)->x8`.
    FTDATA_80085FD4_X8 ftdata_80085fd4_x8(fp: &mut Fighter, i: i32) -> usize = "TODO(group3 ftdata) ftData_80085FD4";
}

// ---- port round 8 (ft/ftcommon.c: landing, nudge, accessory, item/parasol callbacks) -------
ported_hooks! {
    FTCOMMON_8007E6DC ftcommon_8007e6dc(g: G, o: G, i: i32) = crate::ftcommon3::ftcommon_8007e6dc;
    FTCOMMON_8007E82C ftcommon_8007e82c(g: G) = crate::ftcommon3::ftcommon_8007e82c;
    FTCOMMON_8007D92C ftcommon_8007d92c(g: G) = crate::ftcommon3::ftcommon_8007d92c;
    FT_GET_PARASOL_STATUS ft_get_parasol_status(g: G) -> i32 = crate::ftcommon3::ft_get_parasol_status;
    FTLIB_80086FD4 ftlib_80086fd4(a: G, b: G) -> bool = crate::ftlib::ftlib_80086fd4;
    FTLIB_800866DC ftlib_800866dc(g: G, v: &mut Vec3) = crate::ftlib::ftlib_800866dc;
    PLAYER_GET_ENTITY player_get_entity(slot: i32) -> G = crate::player::get_entity;
    PLAYER_GET_UNK98 player_get_unk98(slot: i32) -> i32 = crate::player::get_unk98;
    PLAYER_SET_UNK98 player_set_unk98(slot: i32, v: i32) = crate::player::set_unk98;
}

/// TARGET_PC `Script_GameEvent(what, a, b, c, d)` (gw_script.c, Lua events; output only, it is
/// dispatched after the frame). Unset: nothing.
pub static mut SCRIPT_GAME_EVENT: Option<fn(what: i32, a: i32, b: i32, c: i32, d: i32)> = None;
pub fn script_game_event(what: i32, a: i32, b: i32, c: i32, d: i32) {
    if let Some(f) = unsafe { SCRIPT_GAME_EVENT } { f(what, a, b, c, d) }
}
/// `OSReport` of ftCommon_PcFloorGuard's "pc fighter floor lost" line (log only). Unset: nothing.
pub static mut PC_FLOOR_LOST_REPORT: Option<fn(fp: &Fighter)> = None;
pub fn pc_floor_lost_report(fp: &Fighter) {
    if let Some(f) = unsafe { PC_FLOOR_LOST_REPORT } { f(fp) }
}

type I = *mut Item_GObj;
needed_hooks! {
    /// `mpLib_8004D164()->line_count`, or None when it returns NULL.
    MPLIB_8004D164_LINE_COUNT mplib_8004d164_line_count() -> Option<i32> = "TODO(group2 mp) mpLib_8004D164";
    MP_LINE_GET_NEXT mp_line_get_next(line: i32) -> i32 = "TODO(group2 mp) mpLineGetNext";
    MP_LINE_GET_PREV mp_line_get_prev(line: i32) -> i32 = "TODO(group2 mp) mpLineGetPrev";
    FT_80084A18 ft_80084a18(g: G) -> bool = "TODO(group3 ft_081B) ft_80084A18";
    FTCO_FALL_ENTER ftco_fall_enter(g: G) = "TODO(group3 ftCo) ftCo_Fall_Enter";
    FT_8008A2BC ft_8008a2bc(g: G) = "TODO(group3 ft_08A1) ft_8008A2BC";
    EFLIB_DESTROY_ALL eflib_destroy_all(g: G) = "TODO(group1 ef) efLib_DestroyAll";
    HSD_JOBJ_SET_TRANSLATE hsd_jobj_set_translate(j: *mut HSD_JObj, v: &Vec3) = "TODO(group1 HSD) HSD_JObjSetTranslate";
    HSD_JOBJ_SET_SCALE hsd_jobj_set_scale(j: *mut HSD_JObj, v: &Vec3) = "TODO(group1 HSD) HSD_JObjSetScale";
    HSD_JOBJ_LOAD_JOINT hsd_jobj_load_joint(j: *mut crate::types::HSD_Joint) -> *mut HSD_JObj = "TODO(group1 HSD) HSD_JObjLoadJoint";
    /// `HSD_JObjAddAnimAll(j, an, NULL, NULL)`.
    HSD_JOBJ_ADD_ANIM_ALL hsd_jobj_add_anim_all(j: *mut HSD_JObj, an: *mut crate::types::HSD_AnimJoint) = "TODO(group1 HSD) HSD_JObjAddAnimAll";
    HSD_JOBJ_REQ_ANIM_ALL hsd_jobj_req_anim_all(j: *mut HSD_JObj, f: f32) = "TODO(group1 HSD) HSD_JObjReqAnimAll";
    FTCO_800C5240 ftco_800c5240(g: G) -> bool = "TODO(group3 ftCo) ftCo_800C5240";
    FTCO_800C5500 ftco_800c5500(g: G) = "TODO(group3 ftCo) ftCo_800C5500";
    /// TARGET_PC `Mex_OnItemDropExtDispatch(kind, gobj, arg, ftData_OnItemDropExt[kind])`
    /// (m-ex Arch_FighterFunc slot 16 + the ftdata.c table).
    MEX_ON_ITEM_DROP_EXT mex_on_item_drop_ext(kind: i32, g: G, a: i32) = "TODO(pc mex + group3 ftdata) Mex_OnItemDropExtDispatch";
    MEX_ON_ITEM_DROP mex_on_item_drop(kind: i32, g: G, a: i32) = "TODO(pc mex + group3 ftdata) Mex_OnItemDropDispatch";
    MEX_ON_ITEM_PICKUP2 mex_on_item_pickup2(kind: i32, g: G, a: i32) = "TODO(pc mex + group3 ftdata) Mex_OnItemPickup2Dispatch";
    MEX_ON_ITEM_INVISIBLE mex_on_item_invisible(kind: i32, g: G) = "TODO(pc mex + group3 ftdata) Mex_OnItemInvisibleDispatch";
    MEX_ON_ITEM_VISIBLE mex_on_item_visible(kind: i32, g: G) = "TODO(pc mex + group3 ftdata) Mex_OnItemVisibleDispatch";
    MEX_ON_KNOCKBACK_ENTER mex_on_knockback_enter(kind: i32, g: G) = "TODO(pc mex + group3 ftdata) Mex_OnKnockbackEnterDispatch";
    MEX_ON_KNOCKBACK_EXIT mex_on_knockback_exit(kind: i32, g: G) = "TODO(pc mex + group3 ftdata) Mex_OnKnockbackExitDispatch";
    MEX_ON_REMOVE_HEAD_ITEM mex_on_remove_head_item(kind: i32, g: G) = "TODO(pc mex + group3 ftdata) Mex_OnRemoveHeadItemDispatch";
    MEX_ON_APPLY_HEAD_ITEM mex_on_apply_head_item(kind: i32, g: G) = "TODO(pc mex + group3 ftdata) Mex_OnApplyHeadItemDispatch";
    IT_8028B08C it_8028b08c(i: I, state: i32) -> i32 = "TODO(group5 it) it_8028B08C";
    IT_802BDA40 it_802bda40(i: I, v: i32) -> i32 = "TODO(group5 it) it_802BDA40";
    /// `parasol_table_1[idx](item, v)`: it_8028B718 / B780 / B7E8 / B850 / B648 / B6B0 / B618.
    IT_PARASOL_TABLE_1 it_parasol_table_1(idx: i32, i: I, v: f32) = "TODO(group5 it) it_8028B618..it_8028B850";
    /// `parasol_table_3[idx](item, v)` for idx 4 (it_802BDD40) and 6 (it_802BDDB4).
    IT_PARASOL_TABLE_3 it_parasol_table_3(idx: i32, i: I, v: f32) = "TODO(group5 it) it_802BDD40 / it_802BDDB4";
    GM_8016B0B4 gm_8016b0b4() -> bool = "TODO(group6 gm) gm_8016B0B4";
    FTCO_DAMAGE_CALC_ANGLE ftco_damage_calc_angle(fp: &mut Fighter, kb: f32) -> f32 = "TODO(group3 ftCo) ftCo_Damage_CalcAngle";
    IT_802E5F8C it_802e5f8c(g: G, v: &Vec3, a: i32, b: i32, ang: f32, kb: f32) -> i32 = "TODO(group5 it) it_802E5F8C";
    FTCO_800D105C ftco_800d105c(g: G) = "TODO(group3 ftCo) ftCo_800D105C";
    FT_80081C88 ft_80081c88(g: G, s: f32) = "TODO(group3 ft_081B) ft_80081C88";
    IT_80294EB0 it_80294eb0(i: I, a: &Vec3, b: &Vec3) = "TODO(group5 it) it_80294EB0";
    ITEM_8026A8EC item_8026a8ec(i: I) = "TODO(group5 it) Item_8026A8EC";
    ITEM_8026ABD8 item_8026abd8(i: I, v: &Vec3, f: f32) = "TODO(group5 it) Item_8026ABD8";
    ITEM_8026AB54 item_8026ab54(i: I, g: G, part: i32) = "TODO(group5 it) Item_8026AB54";
    IT_8029A748 it_8029a748(g: G, pos: &Vec3, part: i32, dir: f32) -> I = "TODO(group5 it) it_8029A748";
}

// ---- port round 9: item calls whose group-5 port now exists (melee_items::itlib) ----------
// `gobj->user_data` is the Item (GET_ITEM). Each stays overridable (the ref tests trace them).
mod it5 {
    use crate::types::{Item, Item_GObj};
    use melee_items::itlib as il;
    #[inline(always)]
    fn ip<'a>(g: *mut Item_GObj) -> &'a mut Item {
        unsafe { &mut *((*g).user_data as *mut Item) }
    }
    pub fn it_is_heavy(g: *mut Item_GObj) -> bool { unsafe { il::it_is_heavy(ip(g)) } }
    pub fn it_get_kind(g: *mut Item_GObj) -> i32 { il::it_get_kind(ip(g)) }
    pub fn it_8026b6c8(g: *mut Item_GObj) -> i32 { il::it_8026b6c8(ip(g)) as i32 }
    pub fn it_get_motion_id(g: *mut Item_GObj) -> i32 { il::it_get_motion_id(ip(g)) }
    pub fn it_8026b54c(g: *mut Item_GObj) -> f32 { unsafe { il::it_8026b54c(ip(g)) } }
    pub fn it_8026bd0c(g: *mut Item_GObj) { il::it_8026bd0c(ip(g)) }
    pub fn it_8026bcf4(g: *mut Item_GObj) { il::it_8026bcf4(ip(g)) }
}
ported_hooks! {
    IT_IS_HEAVY it_is_heavy(g: G) -> bool = it5::it_is_heavy;
    IT_GET_KIND it_get_kind(g: G) -> i32 = it5::it_get_kind;
    IT_8026B6C8 it_8026b6c8(g: I) -> i32 = it5::it_8026b6c8;
    IT_GET_MOTION_ID it_get_motion_id(i: I) -> i32 = it5::it_get_motion_id;
    IT_8026B54C it_8026b54c(i: I) -> f32 = it5::it_8026b54c;
    IT_8026BD0C it_8026bd0c(i: I) = it5::it_8026bd0c;
    IT_8026BCF4 it_8026bcf4(i: I) = it5::it_8026bcf4;
}

// ---- group 3 round 10: stubs whose port now exists elsewhere -------------------------------
// it_80284FC4 / it_80285024: group 5's itsword.c ports (melee_items::itkinds5, verified there).
// HSD_JObjGetTranslation / HSD_JObjMtxIsDirty: melee_types::inline (the jobj.h inlines).
mod r10 {
    use crate::types::{HSD_JObj, Item, Item_GObj, Vec3};
    #[inline(always)]
    fn ip<'a>(g: *mut Item_GObj) -> &'a mut Item {
        unsafe { &mut *((*g).user_data as *mut Item) }
    }
    pub fn it_80284fc4(g: *mut Item_GObj, a: i32, f: f32) { melee_items::itkinds5::it_80284fc4(ip(g), a, f) }
    pub fn it_80285024(g: *mut Item_GObj, a: i32) { melee_items::itkinds5::it_80285024(ip(g), a) }
    pub fn jobj_get_translation(j: *mut HSD_JObj, out: &mut Vec3) {
        melee_types::inline::HSD_JObjGetTranslation(unsafe { &*j }, out)
    }
    pub fn jobj_mtx_is_dirty(j: *mut HSD_JObj) -> bool { melee_types::inline::HSD_JObjMtxIsDirty(unsafe { &*j }) }
}
ported_hooks! {
    IT_80284FC4 it_80284fc4(i: I, a: i32, f: f32) = r10::it_80284fc4;
    IT_80285024 it_80285024(i: I, a: i32) = r10::it_80285024;
    HSD_JOBJ_GET_TRANSLATION hsd_jobj_get_translation(j: *mut HSD_JObj, out: &mut Vec3) = r10::jobj_get_translation;
    HSD_JOBJ_MTX_IS_DIRTY hsd_jobj_mtx_is_dirty(j: *mut HSD_JObj) -> bool = r10::jobj_mtx_is_dirty;
}

// ---- group 3 round 10: calls out of plbonuslib.c pl_8003DF44..pl_8003ED0C -----------------
mod it10 {
    use crate::types::{HSD_GObj, Item, Item_GObj};
    use melee_items::itlib as il;
    #[inline(always)]
    fn ip<'a>(g: *mut Item_GObj) -> &'a mut Item {
        unsafe { &mut *((*g).user_data as *mut Item) }
    }
    pub fn it_8026b7e8(g: *mut Item_GObj) -> i32 { il::it_8026b7e8(ip(g)) }
    pub fn it_8026b774(g: *mut Item_GObj, a: u8) -> bool { il::it_8026b774(ip(g), a) }
    pub fn it_8026b7bc(g: *mut Item_GObj) -> i32 { il::it_8026b7bc(ip(g)) }
    pub fn it_8026b960(g: *mut Item_GObj) -> f32 { il::it_8026b960(ip(g)) }
    pub fn it_8026b7cc(g: *mut Item_GObj) -> i32 { il::it_8026b7cc(ip(g)) }
    /// `itGetOwner` (it_26B1.c): `GET_ITEM(gobj)->owner`.
    pub fn it_get_owner(g: *mut Item_GObj) -> *mut HSD_GObj { ip(g).owner }
}
ported_hooks! {
    IT_8026B7E8 it_8026b7e8(i: I) -> i32 = it10::it_8026b7e8;
    IT_8026B774 it_8026b774(i: I, a: u8) -> bool = it10::it_8026b774;
    IT_8026B7BC it_8026b7bc(i: I) -> i32 = it10::it_8026b7bc;
    IT_8026B960 it_8026b960(i: I) -> f32 = it10::it_8026b960;
    IT_8026B7CC it_8026b7cc(i: I) -> i32 = it10::it_8026b7cc;
    IT_GET_OWNER it_get_owner(i: I) -> G = it10::it_get_owner;
    FTLIB_80086960 ftlib_80086960(g: G) -> bool = crate::ftlib::ftlib_80086960;
    FTLIB_80086BE0 ftlib_80086be0(g: G) -> u8 = crate::ftlib::ftlib_80086be0;
    FTLIB_800874BC ftlib_800874bc(g: G) -> i32 = crate::ftlib::ftlib_800874bc;
    FT_800898A8 ft_800898a8(g: G) -> *mut crate::types::Struct2074 = crate::ft0892::ft_800898a8;
}
needed_hooks! {
    /// `it_8026C1D4()`: the L-Gun's `timer` special attribute (`it_804D6D24[It_Kind_L_Gun]`).
    IT_8026C1D4 it_8026c1d4() -> u32 = "TODO(group5 it) it_8026C1D4";
}
ported_hooks! {
    PL_8003DFF4 pl_8003dff4(slot: i32, sub: i32, id: i32) = crate::plbonus2::pl_8003dff4;
    PL_8003EA08 pl_8003ea08(slot: i32, sub: i32) = crate::plbonus2::pl_8003ea08;
}
