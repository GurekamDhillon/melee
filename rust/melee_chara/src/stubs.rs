//! Calls out of the character code into group 3 (ft/ core: fighter.c, ftanim.c, ftcoll.c,
//! ft_0877/ft_0881, ftCommon states) that group 3 has not ported yet.
//!
//! Each is a settable function pointer. Unset, it panics with `TODO(group3): <C name>`, so a
//! missing port is loud, never a silent wrong value. When group 3 ports one, its default here
//! becomes a call to that port (as `ftCommon_8007D5D4` / `ftCommon_8007D60C` already are).
//! Return-valued C functions returning `bool` / `GroundOrAir` return `i32` here (MSL `bool` is int).
//!
//! Single-threaded game state, like the C globals: install before the match.
#![allow(non_snake_case, non_upper_case_globals, static_mut_refs)]

use melee_types::{Fighter, HSD_GObj};

pub type G = *mut HSD_GObj;

#[inline]
fn need<T>(h: Option<T>, what: &str) -> T {
    match h {
        Some(f) => f,
        None => panic!("TODO(group3): {what} is not ported; install melee_chara::stubs hook"),
    }
}

macro_rules! stub {
    ($(#[$m:meta])* $hook:ident, $name:ident, ($($a:ident: $t:ty),*) $(-> $r:ty)?) => {
        $(#[$m])*
        pub static mut $hook: Option<fn($($t),*) $(-> $r)?> = None;
        $(#[$m])*
        #[inline]
        pub fn $name($($a: $t),*) $(-> $r)? {
            need(unsafe { $hook }, stringify!($name))($($a),*)
        }
    };
}

/// Like `stub!`, but an unset hook falls back to group 3's verified port `$port` instead of
/// panicking. The hook stays so the character oracle can install recording stubs.
macro_rules! stub_or {
    ($(#[$m:meta])* $hook:ident, $name:ident, $port:path, ($($a:ident: $t:ty),*) $(-> $r:ty)?) => {
        $(#[$m])*
        pub static mut $hook: Option<fn($($t),*) $(-> $r)?> = None;
        $(#[$m])*
        #[inline]
        pub fn $name($($a: $t),*) $(-> $r)? {
            match unsafe { $hook } {
                Some(f) => f($($a),*),
                None => $port($($a),*),
            }
        }
    };
}

stub!(/// TODO(group3): `ft_80084F3C(gobj)` (ft/ft_0877.c, ground/air physics step).
    FT_80084F3C, ft_80084F3C, (g: G));
stub!(/// TODO(group3): `ft_80084EEC(gobj)` (ft/ft_0877.c).
    FT_80084EEC, ft_80084EEC, (g: G));
stub!(/// TODO(group3): `ft_80085134(gobj)` (ft/ft_0877.c).
    FT_80085134, ft_80085134, (g: G));
stub!(/// TODO(group3): `ft_80084DB0(gobj)` (ft/ft_0877.c).
    FT_80084DB0, ft_80084DB0, (g: G));
stub!(/// TODO(group3): `ft_80084FA8(gobj)` (ft/ft_0877.c).
    FT_80084FA8, ft_80084FA8, (g: G));
stub!(/// TODO(group3): `ft_80083F88(gobj)` (ft/ft_0877.c, collision).
    FT_80083F88, ft_80083F88, (g: G));
stub!(/// TODO(group3): `ftAnim_IsFramesRemaining(gobj)` (ft/ftanim.c).
    FTANIM_ISFRAMESREMAINING, ftAnim_IsFramesRemaining, (g: G) -> i32);
stub!(/// TODO(group3): `ftAnim_8006EBA4(gobj)` (ft/ftanim.c).
    FTANIM_8006EBA4, ftAnim_8006EBA4, (g: G));
stub!(/// TODO(group3): `Fighter_ChangeMotionState(gobj, msid, flags, anim_start, anim_speed, anim_blend, arg3)` (ft/fighter.c).
    FIGHTER_CHANGEMOTIONSTATE, Fighter_ChangeMotionState,
    (g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, arg3: G));
stub!(/// TODO(group3): `ft_8008A2BC(gobj)` (ftCommon Wait enter).
    FT_8008A2BC, ft_8008A2BC, (g: G));
stub!(/// TODO(group3): `ftCo_Fall_Enter(gobj)` (ftCommon/ftCo_Fall.c).
    FTCO_FALL_ENTER, ftCo_Fall_Enter, (g: G));
stub!(/// TODO(group3): `ft_80081D0C(gobj)` (ft/ftcoll.c, air collision; returns GroundOrAir/bool).
    FT_80081D0C, ft_80081D0C, (g: G) -> i32);
stub!(/// TODO(group3): `ft_80082708(gobj)` (ft/ftcoll.c, ground collision).
    FT_80082708, ft_80082708, (g: G) -> i32);
stub!(/// TODO(group3): `ft_800827A0(gobj)` (ft/ftcoll.c).
    FT_800827A0, ft_800827A0, (g: G) -> i32);
stub_or!(/// `ftCommon_8007D7FC(fp)`: group 3's verified port (night 3) when the hook is unset.
    FTCOMMON_8007D7FC, ftCommon_8007D7FC, melee_fighter::ftcommon3::ftcommon_8007d7fc, (fp: &mut Fighter));
stub!(/// TODO(group3): `ftCo_80096900(gobj, a, b, allow_interrupt, mobility, landing_lag)` (FallSpecial enter).
    FTCO_80096900, ftCo_80096900, (g: G, a: i32, b: i32, allow_interrupt: i32, mobility: f32, landing_lag: f32));

/// `ftCommon_8007D5D4(fp)`: group 3's port.
#[inline]
pub fn ftCommon_8007D5D4(fp: &mut Fighter) {
    melee_fighter::ftcommon2::ftcommon_8007d5d4(fp)
}

/// `ftCommon_8007D60C(fp)`: group 3's port.
#[inline]
pub fn ftCommon_8007D60C(fp: &mut Fighter) {
    melee_fighter::ftcommon2::ftcommon_8007d60c(fp)
}

/// `ftCommon_GroundToAirStateChange` (ftCommon/inlines.h, static inline).
#[inline]
pub fn ftCommon_GroundToAirStateChange(g: G, fp: &mut Fighter, msid: i32, flags: u32) {
    ftCommon_8007D5D4(fp);
    Fighter_ChangeMotionState(g, msid, flags, fp.cur_anim_frame, 1.0, 0.0, core::ptr::null_mut());
}

/// `ftCommon_AirToGroundStateChange` (ftCommon/inlines.h, static inline).
#[inline]
pub fn ftCommon_AirToGroundStateChange(g: G, fp: &mut Fighter, msid: i32, flags: u32) {
    ftCommon_8007D7FC(fp);
    Fighter_ChangeMotionState(g, msid, flags, fp.cur_anim_frame, 1.0, 0.0, core::ptr::null_mut());
}

// ---- Round 5 (gen/port_r5.py): more group-3 calls the character code makes. ----
// `HSD_GObjEvent` arguments are `Option<unsafe extern "C" fn(*mut HSD_GObj)>` (chars::cb entries).
use melee_types::HSD_GObjEvent;
stub!(/// TODO(group3): `ft_80082C74(gobj, cb)` (ft/ftcoll.c, ground collision with edge callback).
    FT_80082C74, ft_80082C74, (g: G, cb: HSD_GObjEvent));
stub!(/// TODO(group3): `ft_8008403C(gobj, cb)` (ft/ft_0877.c, air collision with landing callback).
    FT_8008403C, ft_8008403C, (g: G, cb: HSD_GObjEvent));
stub!(/// TODO(group3): `ft_800841B8(gobj, cb)` (ft/ft_0877.c).
    FT_800841B8, ft_800841B8, (g: G, cb: HSD_GObjEvent));
stub!(/// TODO(group3): `ft_80084104(gobj)` (ft/ft_0877.c).
    FT_80084104, ft_80084104, (g: G));
stub!(/// TODO(group3): `ftParts_80074A4C(gobj, model_idx, val)` (ft/ftparts.c).
    FTPARTS_80074A4C, ftParts_80074A4C, (g: G, model_idx: i32, val: i32));
stub!(/// TODO(group3): `ftParts_80074B0C(gobj, model_idx, val)` (ft/ftparts.c).
    FTPARTS_80074B0C, ftParts_80074B0C, (g: G, model_idx: i32, val: i32));
stub!(/// TODO(group3): `ft_CheckGroundAndLedge(gobj, direction)` (ft/ftcoll.c; returns bool).
    FT_CHECKGROUNDANDLEDGE, ft_CheckGroundAndLedge, (g: G, direction: i32) -> i32);
stub_or!(/// `ftCommon_8007DB24(gobj)`: group 3's verified port (night 3) when the hook is unset.
    FTCOMMON_8007DB24, ftCommon_8007DB24, melee_fighter::ftcommon3::ftcommon_8007db24, (g: G));
stub_or!(/// `ftCommon_8007D92C(gobj)`: group 3's verified port (night 3) when the hook is unset.
    FTCOMMON_8007D92C, ftCommon_8007D92C, melee_fighter::ftcommon3::ftcommon_8007d92c, (g: G));
stub!(/// TODO(group3): `ftCo_Wait_IASA(gobj)` (ftCommon/ftCo_Wait.c).
    FTCO_WAIT_IASA, ftCo_Wait_IASA, (g: G));
stub_or!(/// `ftCommon_8007EBAC(fp, u32, u32)`: group 3's verified port (night 3) when the hook is unset.
    FTCOMMON_8007EBAC, ftCommon_8007EBAC, melee_fighter::ftcommon2::ftcommon_8007ebac, (fp: &mut Fighter, a: u32, b: u32));
stub!(/// TODO(group3): `ftAnim_SetAnimRate(gobj, rate)` (ft/ftanim.c).
    FTANIM_SETANIMRATE, ftAnim_SetAnimRate, (g: G, rate: f32));
stub!(/// TODO(group3): `ftAnim_800704F0(gobj, tobj_idx, frame)` (ft/ftanim.c).
    FTANIM_800704F0, ftAnim_800704F0, (g: G, tobj_idx: i32, frame: f32));
stub!(/// TODO(group3): `ftCo_LandingFallSpecial_Enter(gobj, allow_interrupt, landing_lag)`.
    FTCO_LANDINGFALLSPECIAL_ENTER, ftCo_LandingFallSpecial_Enter, (g: G, allow_interrupt: i32, landing_lag: f32));
stub_or!(/// `Fighter_UnkSetFlag_8006CFBC(gobj)` (ft/fighter.c): group 3's verified port (night 3) when the hook is unset.
    FIGHTER_UNKSETFLAG_8006CFBC, Fighter_UnkSetFlag_8006CFBC, melee_fighter::fighter::fighter_unk_set_flag_8006cfbc, (g: G));
stub!(/// TODO(group3): `ftCo_800957F4(gobj, msid)` (ftCommon/ftCo_ItemThrow.c).
    FTCO_800957F4, ftCo_800957F4, (g: G, msid: i32));

/// `Fighter_OnKnockbackEnter` (ft/inlines.h, static inline).
#[inline]
pub fn Fighter_OnKnockbackEnter(g: G, arg1: i32) {
    ftAnim_800704F0(g, arg1, 3.0);
    ftAnim_800704F0(g, 0, 3.0);
}

/// `Fighter_OnKnockbackExit` (ft/inlines.h, static inline).
#[inline]
pub fn Fighter_OnKnockbackExit(g: G, arg1: i32) {
    ftAnim_800704F0(g, arg1, 0.0);
    ftAnim_800704F0(g, 0, 0.0);
}

/// `Fighter_ClearCmdVars` (ft/inlines.h, static inline).
#[inline]
pub fn Fighter_ClearCmdVars(fp: &mut Fighter) {
    fp.cmd_vars[3] = 0;
    fp.cmd_vars[2] = 0;
    fp.cmd_vars[1] = 0;
    fp.cmd_vars[0] = 0;
}

/// `ftCommon_8007E2F4(fp, val)`: group 3's port (`fp->x1A6A = val`).
#[inline]
pub fn ftCommon_8007E2F4(fp: &mut Fighter, val: i16) {
    melee_fighter::ftcommon2::ftcommon_8007e2f4(fp, val)
}

/// `ftCommon_8007E2FC(gobj)`: group 3's port (zero every velocity / acceleration).
#[inline]
pub fn ftCommon_8007E2FC(g: G) {
    melee_fighter::ftcommon2::ftcommon_8007e2fc(unsafe { &mut *((*g).user_data as *mut Fighter) })
}

/// `ftCommon_UpdateFacing(fp)`: group 3's port.
#[inline]
pub fn ftCommon_UpdateFacing(fp: &mut Fighter) {
    melee_fighter::ftcommon::update_facing(fp)
}

// ---- Round 6 (gen/port_r6.py): more calls out of the character code. ----

stub!(/// TODO(group3): `ftColl_8007AEF8(gobj)` (ft/ftcoll.c).
    FTCOLL_8007AEF8, ftColl_8007AEF8, (g: G));
stub!(/// TODO(group3): `ftCo_AirCatchHit_Coll(gobj)` (ft/ftcoll.c / ftCommon AirCatch).
    FTCO_AIRCATCHHIT_COLL, ftCo_AirCatchHit_Coll, (g: G));
stub!(/// TODO(group3): `ftAnim_ApplyPartAnim(gobj, a, b, frame)` (ft/ftanim.c).
    FTANIM_APPLYPARTANIM, ftAnim_ApplyPartAnim, (g: G, a: i32, b: i32, frame: f32));
stub!(/// TODO(group3): `ftAnim_80070C48(gobj, flag)` (ft/ftanim.c; reached through Fighter_OnItemVisible).
    FTANIM_80070C48, ftAnim_80070C48, (g: G, flag: i32));
stub!(/// TODO(group3): `ftAnim_80070CC4(gobj, flag)` (ft/ftanim.c; reached through Fighter_OnItemInvisible).
    FTANIM_80070CC4, ftAnim_80070CC4, (g: G, flag: i32));
stub!(/// TODO(ef): `efLib_DestroyAll(gobj)` (ef/eflib.c; no group owns ef/ yet).
    EFLIB_DESTROYALL, efLib_DestroyAll, (g: G));
stub!(/// TODO(group5): `itIsHeavy(item_gobj)` (it/it_26B1.c; returns bool).
    ITISHEAVY, itIsHeavy, (item: *mut melee_types::Item_GObj) -> i32);

/// `ftCommon_Fall(fp, gravity, terminal_vel)`: group 3's port.
#[inline]
pub fn ftCommon_Fall(fp: &mut Fighter, gravity: f32, terminal_vel: f32) {
    melee_fighter::ftcommon::fall(fp, gravity, terminal_vel)
}

/// `ftCommon_ClampAirDrift(fp)`: group 3's port.
#[inline]
pub fn ftCommon_ClampAirDrift(fp: &mut Fighter) {
    melee_fighter::ftcommon::clamp_air_drift(fp)
}

/// `Fighter_OnItemVisible(gobj, flag)` (ft/inlines.h, static inline).
#[inline]
pub fn Fighter_OnItemVisible(g: G, flag: i32) {
    let item = unsafe { (*((*g).user_data as *mut Fighter)).item_gobj };
    if itIsHeavy(item) == 0 {
        ftAnim_80070C48(g, flag);
    }
}

/// `Fighter_OnItemInvisible(gobj, flag)` (ft/inlines.h, static inline).
#[inline]
pub fn Fighter_OnItemInvisible(g: G, flag: i32) {
    let item = unsafe { (*((*g).user_data as *mut Fighter)).item_gobj };
    if itIsHeavy(item) == 0 {
        ftAnim_80070CC4(g, flag);
    }
}

// round 8 (gen/port_r8.py)
stub!(/// TODO(group3): `Player_GetPlayerSlotType(slot)` (pl/player.c; returns Gm_PKind, as i32).
    PLAYER_GETPLAYERSLOTTYPE, Player_GetPlayerSlotType, (slot: i32) -> i32);

/// `ftBossLib_8015BD20(gobj)` (ft/ftbosslib.c): the C body is `return;`. Ported as is.
#[inline]
pub fn ftBossLib_8015BD20(_g: G) {}

stub!(/// TODO(group3): `ftCliffCommon_80081298(gobj)` (ft/ftcliffcommon.c, ledge grab check; returns bool).
    FTCLIFFCOMMON_80081298, ftCliffCommon_80081298, (g: G) -> i32);
