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
stub!(/// TODO(group3): `ftCommon_8007D7FC(fp)` (ft/ftcommon.c; needs un_803224DC, mpLib).
    FTCOMMON_8007D7FC, ftCommon_8007D7FC, (fp: &mut Fighter));
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
