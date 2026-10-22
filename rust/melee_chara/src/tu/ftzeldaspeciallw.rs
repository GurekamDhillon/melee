//! src/melee/ft/kinds/ftZelda/ftzeldaspeciallw.c: Zelda's down B (Transform, 8013ADB4..8013B5EC).
//! The functions not ported elsewhere are checked bit for bit against the C by a state oracle
//! (tests/tu_ftzeldaspeciallw.rs over gen/turef/tu_ftzeldaspeciallw_ref.c, shipping contraction).
//! Fused sites: none (tools/rust-port/fma_sites.py).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, dead_code)]

use crate::stubs as st;
use melee_types::*;

/// Already ported (rounds 2/4/12 and the r-round callbacks): re-exported under this TU.
pub use crate::chars::r12::ftZelda::ftzeldaspeciallw::{
    ftZd_SpecialAirLw2_IASA, ftZd_SpecialAirLw_IASA, ftZd_SpecialLw2_IASA, ftZd_SpecialLw_IASA,
};
pub use crate::chars::r2::ftZelda::ftzeldaspeciallw::{
    ftZd_SpecialAirLw2_Anim, ftZd_SpecialLw2_Anim, ftZd_SpecialLw2_Phys, ftZd_SpecialLw_Phys,
};
pub use crate::chars::r4::ftZelda::ftzeldaspeciallw::{
    ftZd_SpecialAirLw2_Coll, ftZd_SpecialAirLw_Anim, ftZd_SpecialAirLw_Coll, ftZd_SpecialLw2_Coll,
    ftZd_SpecialLw_8013B1CC, ftZd_SpecialLw_8013B238, ftZd_SpecialLw_8013B400, ftZd_SpecialLw_8013B46C,
    ftZd_SpecialLw_8013B5C4, ftZd_SpecialLw_Anim, ftZd_SpecialLw_Coll,
};
pub use crate::chars::ftZelda::ftzeldaspeciallw::{ftZd_SpecialAirLw2_Phys, ftZd_SpecialAirLw_Phys};

/// Calls into TUs that are not ported yet (or into group 3, via the same fallback as `stubs`).
/// Each is a settable hook; an unset hook without a fallback panics.
pub mod ext {
    use core::ffi::c_void;
    use melee_types::*;
    macro_rules! hook {
        ($hook:ident, $name:ident, ($($a:ident: $t:ty),*) $(-> $r:ty)?) => {
            pub static mut $hook: Option<fn($($t),*) $(-> $r)?> = None;
            #[allow(non_snake_case)]
            pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { $hook } {
                    Some(f) => f($($a),*),
                    None => panic!(concat!("TODO: ", stringify!($name), " is not ported")),
                }
            }
        };
    }
    /// Like `hook!`, but an unset hook runs the native port `$port` (group 3, already ported).
    macro_rules! hook_or {
        ($hook:ident, $name:ident, ($($a:ident: $t:ty),*) $(-> $r:ty)?, $port:path) => {
            pub static mut $hook: Option<fn($($t),*) $(-> $r)?> = None;
            #[allow(non_snake_case)]
            pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { $hook } {
                    Some(f) => f($($a),*),
                    None => $port($($a),*),
                }
            }
        };
    }
    // `efSync_Spawn(gfx_id, gobj, joint)` (ef/efsync.c, variadic; the port passes the one joint).
    hook!(EFSYNC_SPAWN, efSync_Spawn, (gfx_id: i32, gobj: *mut HSD_GObj, joint: *mut c_void) -> *mut c_void);
    // `lb_8000B1CC(jobj, pos0, pos1)` and `lb_800119DC(v, n, a, b, c)` (lb/lb_00B0.c, lb_00F9.c).
    hook!(LB_8000B1CC, lb_8000B1CC, (j: *mut HSD_JObj, pos0: *const Vec3, pos1: *mut Vec3));
    hook!(LB_800119DC, lb_800119DC, (v: *mut Vec3, n: i32, a: f32, b: f32, c: f32));
    // `it_802C3D44(item)` (it/kinds/itzeldadinfire.c).
    hook!(IT_802C3D44, it_802C3D44, (item: *mut HSD_GObj));
    // `ftCommon_8007EFC8(gobj, cb)` (ft/ftcommon.c): group 3's verified port when unset.
    hook_or!(FTCOMMON_8007EFC8, ftCommon_8007EFC8, (g: *mut HSD_GObj, cb: fn(*mut HSD_GObj)),
        super::ftcommon_8007efc8_port);
    /// `ftSk_SpecialLw_80114758` (ft/kinds/ftSeak/ftseakspeciallw.c): not ported yet.
    pub fn ftSk_SpecialLw_80114758(_g: *mut HSD_GObj) {
        panic!("TODO(group4): ftSk_SpecialLw_80114758 (src/melee/ft/kinds/ftSeak/ftseakspeciallw.c) is not ported");
    }
}

/// Group 3's port of `ftCommon_8007EFC8` (melee_fighter), the unset-hook default above.
fn ftcommon_8007efc8_port(g: *mut HSD_GObj, cb: fn(*mut HSD_GObj)) {
    melee_fighter::tu::ftcommon::ftcommon_8007efc8(g, cb)
}

/// `ftZelda_DatAttrs` of a fighter (`fp->dat_attrs`).
#[inline(always)]
unsafe fn da_of(fp: *mut Fighter) -> *mut ftZelda_DatAttrs {
    (*fp).dat_attrs as *mut ftZelda_DatAttrs
}

/// `GET_FIGHTER(gobj)`.
#[inline(always)]
unsafe fn fpp(gobj: *mut HSD_GObj) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}

/// `fp->parts[part].joint`.
#[inline(always)]
unsafe fn joint(fp: *mut Fighter, part: usize) -> *mut core::ffi::c_void {
    (*(*fp).parts.add(part)).joint as *mut core::ffi::c_void
}

/// `Fighter_SetEffectHitlagCallbacks(fp)` (ft/inlines.h).
#[inline(always)]
unsafe fn set_effect_hitlag_cbs(fp: *mut Fighter) {
    (*fp).pre_hitlag_cb = Some(crate::chars::cb::ext__efLib_PauseAll);
    (*fp).post_hitlag_cb = Some(crate::chars::cb::ext__efLib_ResumeAll);
}

/// `ftZd_Init_801393AC` (ft/kinds/ftZelda/ftzelda.c): not ported yet (a callback address only here).
pub unsafe extern "C" fn ftZd_Init_801393AC(_gobj: *mut HSD_GObj) {
    panic!("TODO(group4): ftZd_Init_801393AC (src/melee/ft/kinds/ftZelda/ftzelda.c) is not ported");
}

/// `ftZd_SpecialLw_8013ADB4` (accessory callback: the transform's spark on the top joint).
pub unsafe extern "C" fn ftZd_SpecialLw_8013ADB4(gobj: *mut HSD_GObj) {
    let fp = fpp(gobj);
    if (*fp).x2219_b0() == 0 {
        ext::efSync_Spawn(1276, gobj, joint(fp, 104));
        (*fp).set_x2219_b0(1);
    }
    set_effect_hitlag_cbs(fp);
    (*fp).accessory4_cb = None;
}

/// `ftZd_SpecialLw_8013AE30` (accessory callback: the transform's spark on the hips).
pub unsafe extern "C" fn ftZd_SpecialLw_8013AE30(gobj: *mut HSD_GObj) {
    let fp = fpp(gobj);
    if (*fp).x2219_b0() == 0 {
        ext::efSync_Spawn(1277, gobj, joint(fp, FtPart_HipN as usize));
        (*fp).set_x2219_b0(1);
    }
    set_effect_hitlag_cbs(fp);
    (*fp).accessory4_cb = None;
}

/// `ftZd_SpecialLw_8013AEAC` (accessory callback: Zelda_TransformToSubcharacter).
pub unsafe extern "C" fn ftZd_SpecialLw_8013AEAC(gobj: *mut HSD_GObj) {
    let fp = fpp(gobj);
    (*fp).accessory4_cb = None;
    ext::ftCommon_8007EFC8(gobj, ext::ftSk_SpecialLw_80114758);
}

/// `ftZelda_SpecialLw_StartAction_Helper(gobj)` (static, ftzeldaspeciallw.c).
unsafe fn start_action_helper(gobj: *mut HSD_GObj) {
    let fp = fpp(gobj);
    let attributes = da_of(fp);
    (*fp).cmd_vars[0] = 0;
    (*fp).self_vel.x = (*fp).self_vel.x / (*attributes).x70;
    (*fp).self_vel.y = (*fp).self_vel.y / (*attributes).x74;
    (*fp).gr_vel = (*fp).gr_vel / (*attributes).x70;

    let mut sp20 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    ext::lb_8000B1CC((*(*fp).parts.add(FtPart_TopN as usize)).joint, core::ptr::null(), &mut sp20);
    // MTXDegToRad(60) = (60) * 0.01745329252f.
    ext::lb_800119DC(&mut sp20, 120, 0.4, 0.003, 60.0_f32 * 0.01745329252_f32);

    (*fp).accessory4_cb = Some(ftZd_SpecialLw_8013ADB4);
}

/// `ftZd_SpecialLw_Enter` (AS 355, Zelda_AS_355_Transform_Grounded).
pub fn ftZd_SpecialLw_Enter(gobj: *mut HSD_GObj) {
    unsafe {
        st::Fighter_ChangeMotionState(gobj, 355, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
        start_action_helper(gobj);
    }
}

/// `ftZd_SpecialAirLw_Enter` (AS 357, Zelda_AS_357_Transform_Aerial).
pub fn ftZd_SpecialAirLw_Enter(gobj: *mut HSD_GObj) {
    unsafe {
        st::Fighter_ChangeMotionState(gobj, 357, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
        start_action_helper(gobj);
    }
}

/// `ftZd_SpecialLw_8013B4D8` (AS_ZeldaFinishTransformation): AS 356 grounded, 358 aerial.
pub fn ftZd_SpecialLw_8013B4D8(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let sa = da_of(fp);
        let msid = if (*fp).ground_or_air == 0 { 356 } else { 358 };
        st::Fighter_ChangeMotionState(gobj, msid, 0, (*sa).x80, 1.0, 0.0, core::ptr::null_mut());
        (*fp).accessory4_cb = Some(ftZd_SpecialLw_8013AE30);
    }
}

/// `ftZd_SpecialLw_8013B540`: 1 while the transform's motion is one of the two that may hand off.
pub fn ftZd_SpecialLw_8013B540(gobj: *mut HSD_GObj) -> i32 {
    unsafe {
        let fp = fpp(gobj);
        let action_state_index = (*fp).motion_id;
        if (action_state_index == 344 || action_state_index == 347) && !(*fp).u.zd.x222C.is_null() {
            return 1;
        }
        0
    }
}

/// `ftZd_SpecialLw_8013B574`: consume the cmd_vars[1] pulse while the item is out (motions 345, 348).
pub fn ftZd_SpecialLw_8013B574(gobj: *mut HSD_GObj) -> bool {
    unsafe {
        let fp = fpp(gobj);
        if !(*fp).u.zd.x222C.is_null() {
            match (*fp).motion_id {
                348 | 345 => {
                    if (*fp).cmd_vars[1] == 1 {
                        (*fp).cmd_vars[1] = 0;
                        return true;
                    }
                }
                _ => {}
            }
        }
        false
    }
}

/// `ftZd_SpecialLw_8013B5EC`: destroy the transform's item (if any) and clear the damage hooks.
pub fn ftZd_SpecialLw_8013B5EC(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        if !(*fp).u.zd.x222C.is_null() {
            ext::it_802C3D44((*fp).u.zd.x222C);
            (*fp).u.zd.x222C = core::ptr::null_mut();
        }
        (*fp).death2_cb = None;
        (*fp).take_dmg_cb = None;
    }
}
