//! The `static inline` helpers of ft/inlines.h and ft/kinds/ftCommon/inlines.h that the ftYoshi /
//! ftZelda / ftZako TUs use. In C they are compiled into every caller, so their callees are the
//! caller's externals; here each callee is a hook in this module's `ext` (the oracle installs
//! recorders into it together with the TU's own), and the unset default is the native port when one
//! exists. No float arithmetic, so no fused sites.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use melee_types::*;

pub mod ext {
    use crate::tu::hookmac::{hook, hook_or};
    use melee_types::*;
    hook_or!(ITISHEAVY, itIsHeavy, (item: *mut Item_GObj) -> bool, melee_fighter::hooks::it_is_heavy);
    hook_or!(ITGETHOLDKIND, itGetHoldKind, (item: *mut Item_GObj) -> i32, melee_fighter::tu::ftwaitanim::ext::itGetHoldKind);
    hook_or!(FTANIM_80070FB4, ftAnim_80070FB4, (gobj: *mut HSD_GObj, a: i32, b: i32), melee_fighter::ftanim::ftanim_80070fb4);
    hook!(FTANIM_80070C48, ftAnim_80070C48, (gobj: *mut HSD_GObj, flag: i32));
    hook!(FTANIM_80070CC4, ftAnim_80070CC4, (gobj: *mut HSD_GObj, flag: i32));
    hook!(FTANIM_800704F0, ftAnim_800704F0, (gobj: *mut HSD_GObj, tobj_idx: i32, frame: f32));
    hook_or!(FTCOMMON_8007D5D4, ftCommon_8007D5D4, (fp: *mut Fighter),
        |fp: *mut Fighter| unsafe { melee_fighter::ftcommon2::ftcommon_8007d5d4(&mut *fp) });
    hook_or!(FTCOMMON_8007D7FC, ftCommon_8007D7FC, (fp: *mut Fighter),
        |fp: *mut Fighter| unsafe { melee_fighter::ftcommon3::ftcommon_8007d7fc(&mut *fp) });
    hook!(HSD_JOBJSETMTXDIRTYSUB, HSD_JObjSetMtxDirtySub, (jobj: *mut HSD_JObj));
    // The ft/ callees shared by the ftYoshi / ftZelda TUs; each TU's `ext` re-exports the ones it calls.
    hook!(FTANIM_8006EBA4, ftAnim_8006EBA4, (gobj: *mut HSD_GObj));
    hook!(FTANIM_ISFRAMESREMAINING, ftAnim_IsFramesRemaining, (gobj: *mut HSD_GObj) -> bool);
    hook!(FT_8008A2BC, ft_8008A2BC, (gobj: *mut HSD_GObj));
    hook!(FTCO_FALL_ENTER, ftCo_Fall_Enter, (gobj: *mut HSD_GObj));
    hook_or!(FT_80084F3C, ft_80084F3C, (gobj: *mut HSD_GObj), melee_fighter::ft084e::ft_80084f3c);
    hook_or!(FT_80084EEC, ft_80084EEC, (gobj: *mut HSD_GObj), melee_fighter::ft084e::ft_80084eec);
    hook_or!(FT_80085134, ft_80085134, (gobj: *mut HSD_GObj), melee_fighter::ft084e::ft_80085134);
    hook_or!(FT_80082708, ft_80082708, (gobj: *mut HSD_GObj) -> i32,
        |g: *mut HSD_GObj| melee_fighter::tu::ft_081b::ft_80082708(g) as i32);
    hook_or!(FT_80081D0C, ft_80081D0C, (gobj: *mut HSD_GObj) -> i32,
        |g: *mut HSD_GObj| melee_fighter::tu::ft_081b::ft_80081d0c(g) as i32);
    hook_or!(FT_800827A0, ft_800827A0, (gobj: *mut HSD_GObj) -> bool, melee_fighter::tu::ft_081b::ft_800827a0);
    hook!(FT_8008403C, ft_8008403C, (gobj: *mut HSD_GObj, cb: HSD_GObjEvent));
    hook!(FT_80082C74, ft_80082C74, (gobj: *mut HSD_GObj, cb: HSD_GObjEvent));
    hook!(FT_80083A48, ft_80083A48, (gobj: *mut HSD_GObj, cb: HSD_GObjEvent));
    hook_or!(FT_CHECKGROUNDANDLEDGE, ft_CheckGroundAndLedge, (gobj: *mut HSD_GObj, dir: i32) -> bool,
        melee_fighter::tu::ft_081b::ft_check_ground_and_ledge);
    hook_or!(FTCLIFFCOMMON_80081298, ftCliffCommon_80081298, (gobj: *mut HSD_GObj) -> bool,
        melee_fighter::tu::ftcliffcommon::ftcliffcommon_80081298);
    hook_or!(FTCLIFFCOMMON_80081370, ftCliffCommon_80081370, (gobj: *mut HSD_GObj),
        melee_fighter::tu::ftcliffcommon::ftcliffcommon_80081370);
    hook_or!(FTCOMMON_CLAMPAIRDRIFT, ftCommon_ClampAirDrift, (fp: *mut Fighter),
        |fp: *mut Fighter| unsafe { melee_fighter::ftcommon::clamp_air_drift(&mut *fp) });
    hook_or!(FTCOMMON_CALCGROUNDACCEL_DEACCEL, ftCommon_CalcGroundAccel_Deaccel, (fp: *mut Fighter, friction: f32),
        |fp: *mut Fighter, f: f32| unsafe { melee_fighter::ftcommon::calc_ground_accel_deaccel(&mut *fp, f) });
    hook_or!(FTCOMMON_8007D60C, ftCommon_8007D60C, (fp: *mut Fighter),
        |fp: *mut Fighter| unsafe { melee_fighter::ftcommon2::ftcommon_8007d60c(&mut *fp) });
    hook_or!(FTCOMMON_CALCSELFACCEL_DEACCELAIR, ftCommon_CalcSelfAccel_DeaccelAir, (fp: *mut Fighter),
        |fp: *mut Fighter| unsafe { melee_fighter::ftcommon::calc_self_accel_deaccel_air(&mut *fp) });
    hook_or!(FTCOMMON_CLAMPSELFVELX, ftCommon_ClampSelfVelX, (fp: *mut Fighter, max: f32),
        |fp: *mut Fighter, m: f32| unsafe { melee_fighter::ftcommon::clamp_self_vel_x(&mut *fp, m) });
    hook_or!(FTCOMMON_FALLBASIC, ftCommon_FallBasic, (fp: *mut Fighter),
        |fp: *mut Fighter| unsafe { melee_fighter::ftcommon::fall_basic(&mut *fp) });
    hook_or!(FTCOMMON_FALL, ftCommon_Fall, (fp: *mut Fighter, gravity: f32, terminal_vel: f32),
        |fp: *mut Fighter, g: f32, t: f32| unsafe { melee_fighter::ftcommon::fall(&mut *fp, g, t) });
    hook_or!(FTCOMMON_SETSELFMOVEMENTFROMGROUNDEDMOVEMENT, ftCommon_SetSelfMovementFromGroundedMovement, (gobj: *mut HSD_GObj),
        |g: *mut HSD_GObj| unsafe { melee_fighter::ftcommon::set_self_movement_from_grounded_movement(&mut *((*g).user_data as *mut Fighter)) });
    hook!(FIGHTER_CHANGEMOTIONSTATE, Fighter_ChangeMotionState,
        (gobj: *mut HSD_GObj, msid: i32, flags: u32, anim_start: f32, anim_speed: f32, anim_blend: f32, arg3: *mut HSD_GObj));
}

#[inline(always)]
unsafe fn fpp(gobj: *mut HSD_GObj) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}

/// `Fighter_OnItemPickup(gobj, catchItemFlag, bool2, bool3)`: used for all fighters except Kirby and Purin.
pub fn Fighter_OnItemPickup(gobj: *mut HSD_GObj, catch_item: i32, bool2: i32, bool3: i32) {
    unsafe {
        let fp = fpp(gobj);
        if !ext::itIsHeavy((*fp).item_gobj) {
            match ext::itGetHoldKind((*fp).item_gobj) {
                1 => ext::ftAnim_80070FB4(gobj, bool2, 1),
                2 => ext::ftAnim_80070FB4(gobj, bool2, 0),
                3 => ext::ftAnim_80070FB4(gobj, bool2, 2),
                4 => ext::ftAnim_80070FB4(gobj, bool2, 3),
                _ => {}
            }
            if catch_item != 0 {
                ext::ftAnim_80070C48(gobj, bool3);
            }
        }
    }
}

/// `Fighter_OnItemInvisible(gobj, flag)`.
pub fn Fighter_OnItemInvisible(gobj: *mut HSD_GObj, flag: i32) {
    unsafe {
        let fp = fpp(gobj);
        if !ext::itIsHeavy((*fp).item_gobj) {
            ext::ftAnim_80070CC4(gobj, flag);
        }
    }
}

/// `Fighter_OnItemVisible(gobj, flag)`.
pub fn Fighter_OnItemVisible(gobj: *mut HSD_GObj, flag: i32) {
    unsafe {
        let fp = fpp(gobj);
        if !ext::itIsHeavy((*fp).item_gobj) {
            ext::ftAnim_80070C48(gobj, flag);
        }
    }
}

/// `Fighter_OnItemDrop(gobj, dropItemFlag, bool2, bool3)`.
pub fn Fighter_OnItemDrop(gobj: *mut HSD_GObj, drop_item: i32, bool2: i32, bool3: i32) {
    ext::ftAnim_80070FB4(gobj, bool2, -1);
    if drop_item != 0 {
        ext::ftAnim_80070CC4(gobj, bool3);
    }
}

/// `Fighter_OnKnockbackEnter(gobj, arg1)`.
pub fn Fighter_OnKnockbackEnter(gobj: *mut HSD_GObj, arg1: i32) {
    ext::ftAnim_800704F0(gobj, arg1, 3.0);
    ext::ftAnim_800704F0(gobj, 0, 3.0);
}

/// `Fighter_OnKnockbackExit(gobj, arg1)`.
pub fn Fighter_OnKnockbackExit(gobj: *mut HSD_GObj, arg1: i32) {
    ext::ftAnim_800704F0(gobj, arg1, 0.0);
    ext::ftAnim_800704F0(gobj, 0, 0.0);
}

/// `Fighter_SetEffectHitlagCallbacks(fp)`: efLib_PauseAll / efLib_ResumeAll as the hitlag callbacks
/// (the chars::cb entries panic `TODO` when the game calls them, until efLib is ported).
pub fn Fighter_SetEffectHitlagCallbacks(fp: *mut Fighter) {
    unsafe {
        (*fp).pre_hitlag_cb = Some(crate::chars::cb::ext__efLib_PauseAll);
        (*fp).post_hitlag_cb = Some(crate::chars::cb::ext__efLib_ResumeAll);
    }
}

/// `ftCommon_GroundToAirStateChange(gobj, fp, msid, flags)`.
pub fn ftCommon_GroundToAirStateChange(gobj: *mut HSD_GObj, fp: *mut Fighter, msid: i32, flags: u32) {
    ext::ftCommon_8007D5D4(fp);
    ext::Fighter_ChangeMotionState(gobj, msid, flags, unsafe { (*fp).cur_anim_frame }, 1.0, 0.0, core::ptr::null_mut());
}

/// `ftCommon_AirToGroundStateChange(gobj, fp, msid, flags)`.
pub fn ftCommon_AirToGroundStateChange(gobj: *mut HSD_GObj, fp: *mut Fighter, msid: i32, flags: u32) {
    ext::ftCommon_8007D7FC(fp);
    ext::Fighter_ChangeMotionState(gobj, msid, flags, unsafe { (*fp).cur_anim_frame }, 1.0, 0.0, core::ptr::null_mut());
}

/// `getStickDirX(fp)`: -1 for a stick left of centre, else +1.
pub fn getStickDirX(fp: *mut Fighter) -> i32 {
    if unsafe { (*fp).input.lstick[0].x } < 0.0 {
        -1
    } else {
        1
    }
}

/// `ftGetFacingDirInt(fp)`.
pub fn ftGetFacingDirInt(fp: *mut Fighter) -> i32 {
    if unsafe { (*fp).facing_dir } < 0.0 {
        -1
    } else {
        1
    }
}

/// `PUSH_ATTRS(fp, T)` (ft/inlines.h): copy `ft_data->ext_attr` into the backup block and point
/// `dat_attrs` at it.
pub unsafe fn push_attrs<T: Copy>(fp: *mut Fighter) {
    let backup = (*fp).dat_attrs_backup;
    let src = (*(*fp).ft_data).ext_attr as *const T;
    *((*fp).dat_attrs_backup as *mut T) = *src;
    (*fp).dat_attrs = backup;
}

/// `COPY_ATTRS(gobj, T)` (ft/inlines.h): copy `ft_data->ext_attr` over the live `dat_attrs` block.
pub unsafe fn copy_attrs<T: Copy>(gobj: *mut HSD_GObj) {
    let fp = fpp(gobj);
    let sa2 = (*fp).dat_attrs as *mut T;
    let ext_attr = (*(*fp).ft_data).ext_attr as *const T;
    *sa2 = *ext_attr;
}

/// `HSD_JObjMtxIsDirty(jobj)` (jobj.h inline).
pub unsafe fn HSD_JObjMtxIsDirty(jobj: *mut HSD_JObj) -> bool {
    let fl = (*jobj).flags;
    fl & JOBJ_USER_DEF_MTX as u32 == 0 && fl & JOBJ_MTX_DIRTY as u32 != 0
}

/// The `HSD_JObjSetMtxDirty(jobj)` macro (jobj.h).
pub unsafe fn HSD_JObjSetMtxDirty(jobj: *mut HSD_JObj) {
    if !jobj.is_null() && !HSD_JObjMtxIsDirty(jobj) {
        ext::HSD_JObjSetMtxDirtySub(jobj);
    }
}

/// `HSD_JObjSetScale(jobj, scale)` (jobj.h inline; the asserts are debug-only).
pub unsafe fn HSD_JObjSetScale(jobj: *mut HSD_JObj, scale: *const Vec3) {
    (*jobj).scale = *scale;
    if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
        HSD_JObjSetMtxDirty(jobj);
    }
}

/// `HSD_JObjGetScale(jobj, scale)` (jobj.h inline).
pub unsafe fn HSD_JObjGetScale(jobj: *mut HSD_JObj, scale: *mut Vec3) {
    *scale = (*jobj).scale;
}
