//! Mechanical native port of the C file named below, generated from the C by `rust/melee_world/gen/mtu/c2rs.py`
//! (libclang AST: every implicit conversion is the C one, `-ffp-contract=on` multiply-adds become `fmadds`).
//! Callees of other TUs are `ext` hooks named like the C (unset: panic "TODO"; `melee_link` wires them to the
//! canonical TU function). `bool` is MSL `int`.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, dead_code, unused_assignments, unused_variables, unused_mut, unused_parens, unused_labels, unused_comparisons, clippy::all)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use crate::stubs as st;
#[allow(unused_imports)]
use melee_types::{Fighter, HSD_GObj};
use core::ffi::{c_char, c_void};
use core::ptr::null_mut;

use melee_core::fp::fmadds;
use melee_types::*;

pub mod ext {
    #![allow(non_upper_case_globals, non_snake_case, unused_imports, dead_code)]
    use super::*;
    macro_rules! ext_fns {
        ($(fn $name:ident($($a:ident: $t:ty),*) $(-> $r:ty)?;)*) => {
            pub mod hook {
                use super::*;
                $( pub static mut $name: Option<fn($($t),*) $(-> $r)?> = None; )*
            }
            $( pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { hook::$name } {
                    Some(f) => f($($a),*),
                    None => panic!(concat!("TODO: ", stringify!($name), " is not ported")),
                }
            } )*
        };
    }
    ext_fns! {
    fn Fighter_ChangeMotionState(a0: *mut HSD_GObj, a1: i32, a2: u32, a3: f32, a4: f32, a5: f32, a6: *mut HSD_GObj);
    fn efLib_DestroyAll(a0: *mut HSD_GObj);
    fn efLib_PauseAll(a0: *mut HSD_GObj);
    fn efLib_ResumeAll(a0: *mut HSD_GObj);
    fn efSync_Spawn(a0: i32, a1: *mut HSD_GObj, a2: &[*const c_void]) -> *mut c_void;
    fn ftAnim_8006EBA4(a0: *mut HSD_GObj);
    fn ftAnim_IsFramesRemaining(a0: *mut HSD_GObj) -> i32;
    fn ftCliffCommon_80081298(a0: *mut HSD_GObj) -> i32;
    fn ftCliffCommon_80081370(a0: *mut HSD_GObj);
    fn ftCo_80096900(a0: *mut HSD_GObj, a1: i32, a2: i32, a3: i32, a4: f32, a5: f32);
    fn ftCo_Fall_Enter(a0: *mut HSD_GObj);
    fn ftCo_LandingFallSpecial_Enter(a0: *mut HSD_GObj, a1: i32, a2: f32);
    fn ftCommon_8007D60C(a0: *mut Fighter);
    fn ftCommon_8007D7FC(a0: *mut Fighter);
    fn ftCommon_CalcSelfAccel_Drift(a0: *mut Fighter);
    fn ftCommon_CalcSelfAccel_DriftSimple(a0: *mut Fighter, a1: f32, a2: f32, a3: f32);
    fn ftCommon_ClampSelfVelX(a0: *mut Fighter, a1: f32);
    fn ftCommon_UpdateFacing(a0: *mut Fighter);
    fn ftPartSetRotY(a0: *mut Fighter, a1: i32, a2: f32);
    fn ft_80081D0C(a0: *mut HSD_GObj) -> GroundOrAir;
    fn ft_80084104(a0: *mut HSD_GObj);
    fn ft_80084DB0(a0: *mut HSD_GObj);
    fn ft_80084F3C(a0: *mut HSD_GObj);
    fn ft_800851C0(a0: *mut HSD_GObj);
    fn ft_CheckGroundAndLedge(a0: *mut HSD_GObj, a1: i32) -> i32;
    }
}



/// `ftSs_SpecialHi_Enter`
pub unsafe extern "C" fn ftSs_SpecialHi_Enter(mut gobj: *mut HSD_GObj) {
    melee_core::ct!("ftSs_SpecialHi_Enter");
    let mut fp: *mut Fighter = HSD_GObjGetUserData(gobj) as *mut Fighter;
    let mut __: [u8; 8] = [0; 8];
    ext::Fighter_ChangeMotionState(gobj, 353, 0 as u32, 0.0f32, 1.0f32, 0.0f32, null_mut());
    ftSamus_updateDamageDeathCBs(gobj);
    Fighter_SetEffectHitlagCallbacks(fp);
    ext::ftCommon_8007D7FC(fp);
    Fighter_ClearCmdVars(fp);
    (*fp).mv.ss.unk5.x0 = 0;
    ext::ftAnim_8006EBA4(gobj);
    let _ = ext::efSync_Spawn(1154, gobj, &[(*(*fp).parts.offset((FtPart_YRotN as i32) as isize)).joint as *const c_void]);
    (*fp).u.ss.x2244 = 1 as u32;
}

/// `ftSs_SpecialAirHi_Enter`
pub unsafe extern "C" fn ftSs_SpecialAirHi_Enter(mut gobj: *mut HSD_GObj) {
    melee_core::ct!("ftSs_SpecialAirHi_Enter");
    let mut fp: *mut Fighter = HSD_GObjGetUserData(gobj) as *mut Fighter;
    let mut samus_attr: *mut _ftSamusAttributes = (*fp).dat_attrs as *mut _ftSamusAttributes;
    ext::Fighter_ChangeMotionState(gobj, 354, 0 as u32, 0.0f32, 1.0f32, 0.0f32, null_mut());
    ftSamus_updateDamageDeathCBs(gobj);
    Fighter_SetEffectHitlagCallbacks(fp);
    ext::ftCommon_8007D60C(fp);
    Fighter_ClearCmdVars(fp);
    (*fp).mv.ss.unk5.x0 = 0;
    (*fp).self_vel.y = (*samus_attr).x44;
    ext::ftCommon_ClampSelfVelX(fp, (*samus_attr).x40);
    ext::ftAnim_8006EBA4(gobj);
    let _ = ext::efSync_Spawn(1154, gobj, &[(*(*fp).parts.offset((FtPart_YRotN as i32) as isize)).joint as *const c_void]);
    (*fp).u.ss.x2244 = 1 as u32;
}



/// `ftSs_SpecialHi_Anim`
pub unsafe extern "C" fn ftSs_SpecialHi_Anim(mut gobj: *mut HSD_GObj) {
    melee_core::ct!("ftSs_SpecialHi_Anim");
    let mut fp: *mut Fighter = HSD_GObjGetUserData(gobj) as *mut Fighter;
    let mut samus_attr: *mut _ftSamusAttributes = null_mut();
    let mut samus_attr2: *mut _ftSamusAttributes = null_mut();
    samus_attr = { samus_attr2 = (*fp).dat_attrs as *mut _ftSamusAttributes; samus_attr2 };
    if !(ext::ftAnim_IsFramesRemaining(gobj) != 0) {
        let mut fighter2: *mut Fighter = fp;
        ftSamus_DestroyAllUnsetx2444(gobj);
        ext::ftCommon_8007D60C(fighter2);
        if (*samus_attr).x50 == 0.0f32 {
            ext::ftCo_Fall_Enter(gobj);
            return;
        }
        ext::ftCo_80096900(gobj, 1, 1, 0, (*samus_attr).x48, (*samus_attr).x50);
    }
}

/// `ftSs_SpecialAirHi_Anim`
pub unsafe extern "C" fn ftSs_SpecialAirHi_Anim(mut gobj: *mut HSD_GObj) {
    melee_core::ct!("ftSs_SpecialAirHi_Anim");
    let mut fp: *mut Fighter = HSD_GObjGetUserData(gobj) as *mut Fighter;
    let mut samus_attr: *mut _ftSamusAttributes = null_mut();
    let mut samus_attr2: *mut _ftSamusAttributes = null_mut();
    samus_attr = { samus_attr2 = (*fp).dat_attrs as *mut _ftSamusAttributes; samus_attr2 };
    if !(ext::ftAnim_IsFramesRemaining(gobj) != 0) {
        let mut fighter2: *mut Fighter = fp;
        ftSamus_DestroyAllUnsetx2444(gobj);
        ext::ftCommon_8007D60C(fighter2);
        if (*samus_attr).x50 == 0.0f32 {
            ext::ftCo_Fall_Enter(gobj);
            return;
        }
        ext::ftCo_80096900(gobj, 1, 1, 0, (*samus_attr).x48, (*samus_attr).x50);
    }
}

/// `ftSs_SpecialHi_IASA`
pub unsafe extern "C" fn ftSs_SpecialHi_IASA(mut gobj: *mut HSD_GObj) {
    melee_core::ct!("ftSs_SpecialHi_IASA");
    let mut mag: f32 = 0.0;
    let mut lstick_x: f32 = 0.0;
    let mut fp: *mut Fighter = HSD_GObjGetUserData(gobj) as *mut Fighter;
    let mut samus_attr: *mut _ftSamusAttributes = (*fp).dat_attrs as *mut _ftSamusAttributes;
    let mut __: [u8; 4] = [0; 4];
    if (!((*fp).cmd_vars[1 as usize] != 0)) && (!((*fp).mv.ss.unk5.x0 != 0)) {
        if { lstick_x = (*fp).input.lstick[0 as usize].x; lstick_x } < 0.0f32 {
            mag = -lstick_x;
        } else {
            mag = lstick_x;
        }
        if mag > (*samus_attr).x4C {
            if (((*fp).facing_dir == 1.0f32) && (lstick_x < 0.0f32)) || (((*fp).facing_dir == (-1.0f32)) && (lstick_x > 0.0f32)) {
                (*fp).cmd_vars[1 as usize] = 1 as u32;
                (*fp).mv.ss.unk5.x0 = 1;
                ext::ftCommon_UpdateFacing(fp);
                ext::ftPartSetRotY(fp, 0, ((3.141592653589793f64 / (2 as f64)) * ((*fp).facing_dir as f64)) as f32);
            }
        }
    }
}

/// `ftSs_SpecialAirHi_IASA`
pub unsafe extern "C" fn ftSs_SpecialAirHi_IASA(mut gobj: *mut HSD_GObj) {
    melee_core::ct!("ftSs_SpecialAirHi_IASA");
    let mut mag: f32 = 0.0;
    let mut lstick_x: f32 = 0.0;
    let mut fp: *mut Fighter = HSD_GObjGetUserData(gobj) as *mut Fighter;
    let mut samus_attr: *mut _ftSamusAttributes = (*fp).dat_attrs as *mut _ftSamusAttributes;
    let mut __: [u8; 8] = [0; 8];
    if (!((*fp).cmd_vars[1 as usize] != 0)) && (!((*fp).mv.ss.unk5.x0 != 0)) {
        if { lstick_x = (*fp).input.lstick[0 as usize].x; lstick_x } < 0.0f32 {
            mag = -lstick_x;
        } else {
            mag = lstick_x;
        }
        if mag > (*samus_attr).x4C {
            if (((*fp).facing_dir == 1.0f32) && (lstick_x < 0.0f32)) || (((*fp).facing_dir == (-1.0f32)) && (lstick_x > 0.0f32)) {
                (*fp).cmd_vars[1 as usize] = 1 as u32;
                (*fp).mv.ss.unk5.x0 = 1;
                ext::ftCommon_UpdateFacing(fp);
                ext::ftPartSetRotY(fp, 0, ((3.141592653589793f64 / (2 as f64)) * ((*fp).facing_dir as f64)) as f32);
            }
        }
    }
}

/// `ftSs_SpecialHi_Phys`
pub unsafe extern "C" fn ftSs_SpecialHi_Phys(mut gobj: *mut HSD_GObj) {
    melee_core::ct!("ftSs_SpecialHi_Phys");
    let mut fp: *mut Fighter = getFighter(gobj);
    let mut samus_attr: *mut _ftSamusAttributes = getFtSpecialAttrs(fp) as *mut _ftSamusAttributes;
    if (*fp).cmd_vars[0 as usize] != 0 {
        ext::ftCommon_8007D60C(fp);
        (*fp).cmd_vars[0 as usize] = 0 as u32;
        (*fp).self_vel.x = (*samus_attr).x38 * (*fp).facing_dir;
    }
    if (*fp).ground_or_air == (1 as u32) {
        ext::ft_800851C0(gobj);
        ext::ftCommon_CalcSelfAccel_DriftSimple(fp, 0.0f32, (*samus_attr).x3C, (*samus_attr).x40);
        ext::ftCommon_CalcSelfAccel_Drift(fp);
        return;
    }
    ext::ft_80084F3C(gobj);
}

/// `ftSs_SpecialAirHi_Phys`
pub unsafe extern "C" fn ftSs_SpecialAirHi_Phys(mut gobj: *mut HSD_GObj) {
    melee_core::ct!("ftSs_SpecialAirHi_Phys");
    let mut fp: *mut Fighter = HSD_GObjGetUserData(gobj) as *mut Fighter;
    let mut samus_attr: *mut _ftSamusAttributes = (*fp).dat_attrs as *mut _ftSamusAttributes;
    let mut __: [u8; 8] = [0; 8];
    ext::ft_80084DB0(gobj);
    ext::ftCommon_CalcSelfAccel_DriftSimple(fp, 0.0f32, (*samus_attr).x3C, (*samus_attr).x40);
}

/// `ftSs_SpecialHi_Coll`
pub unsafe extern "C" fn ftSs_SpecialHi_Coll(mut gobj: *mut HSD_GObj) {
    melee_core::ct!("ftSs_SpecialHi_Coll");
    let mut fp: *mut Fighter = getFighter(gobj);
    let mut samus_attr: *mut _ftSamusAttributes = (*fp).dat_attrs as *mut _ftSamusAttributes;
    let mut __: [u8; 8] = [0; 8];
    if (*fp).ground_or_air == ((GA_Air as i32) as u32) {
        let mut direction: i32 = 0;
        if (*fp).self_vel.y >= 0.0f32 {
            let _ = ext::ft_80081D0C(gobj);
            return;
        }
        if (*fp).facing_dir == 1.0f32 {
            direction = 1;
        } else {
            direction = (-1_i32);
        }
        if ext::ft_CheckGroundAndLedge(gobj, direction) != 0 {
            ftSamus_DestroyAllUnsetx2444(gobj);
            ext::ftCo_LandingFallSpecial_Enter(gobj, 0, (*samus_attr).x50);
            return;
        }
        if ext::ftCliffCommon_80081298(gobj) != 0 {
            ftSamus_DestroyAllUnsetx2444(gobj);
            ext::ftCliffCommon_80081370(gobj);
        }
    } else {
        ext::ft_80084104(gobj);
    }
}

/// `ftSs_SpecialAirHi_Coll`
pub unsafe extern "C" fn ftSs_SpecialAirHi_Coll(mut gobj: *mut HSD_GObj) {
    melee_core::ct!("ftSs_SpecialAirHi_Coll");
    let mut fp: *mut Fighter = HSD_GObjGetUserData(gobj) as *mut Fighter;
    let mut samus_attr: *mut _ftSamusAttributes = (*fp).dat_attrs as *mut _ftSamusAttributes;
    let mut __: [u8; 4] = [0; 4];
    if (*fp).ground_or_air == ((GA_Air as i32) as u32) {
        let mut direction: i32 = 0;
        if (*fp).self_vel.y >= 0.0f32 {
            let _ = ext::ft_80081D0C(gobj);
            return;
        }
        if (*fp).facing_dir == 1.0f32 {
            direction = 1;
        } else {
            direction = (-1_i32);
        }
        if ext::ft_CheckGroundAndLedge(gobj, direction) != 0 {
            ftSamus_DestroyAllUnsetx2444(gobj);
            ext::ftCo_LandingFallSpecial_Enter(gobj, 0, (*samus_attr).x50);
            return;
        }
        if ext::ftCliffCommon_80081298(gobj) != 0 {
            ftSamus_DestroyAllUnsetx2444(gobj);
            ext::ftCliffCommon_80081370(gobj);
        }
    } else {
        ext::ft_80084104(gobj);
    }
}

/// `HSD_GObjGetUserData`
pub unsafe extern "C" fn HSD_GObjGetUserData(mut gobj: *mut HSD_GObj) -> *mut c_void {
    melee_core::ct!("HSD_GObjGetUserData");
    return (*gobj).user_data;
}

/// `ftSamus_updateDamageDeathCBs`
pub unsafe extern "C" fn ftSamus_updateDamageDeathCBs(mut gobj: *mut HSD_GObj) {
    melee_core::ct!("ftSamus_updateDamageDeathCBs");
    let mut fp: *mut Fighter = HSD_GObjGetUserData(gobj) as *mut Fighter;
    (*fp).take_dmg_cb = Some(super::ftsamus::ftSs_Init_80128428);
    (*fp).death2_cb = Some(super::ftsamus::ftSs_Init_80128428);
}

/// `Fighter_SetEffectHitlagCallbacks`
pub unsafe extern "C" fn Fighter_SetEffectHitlagCallbacks(mut fp: *mut Fighter) {
    melee_core::ct!("Fighter_SetEffectHitlagCallbacks");
    (*fp).pre_hitlag_cb = Some(melee_world::tu::efn::eflib::efLib_PauseAll);
    (*fp).post_hitlag_cb = Some(melee_world::tu::efn::eflib::efLib_ResumeAll);
}

/// `Fighter_ClearCmdVars`
pub unsafe extern "C" fn Fighter_ClearCmdVars(mut fp: *mut Fighter) {
    melee_core::ct!("Fighter_ClearCmdVars");
    (*fp).cmd_vars[3 as usize] = 0 as u32;
    (*fp).cmd_vars[2 as usize] = 0 as u32;
    (*fp).cmd_vars[1 as usize] = 0 as u32;
    (*fp).cmd_vars[0 as usize] = 0 as u32;
}

/// `getFighter`
pub unsafe extern "C" fn getFighter(mut gobj: *mut HSD_GObj) -> *mut Fighter {
    melee_core::ct!("getFighter");
    return (*gobj).user_data as *mut Fighter;
}

/// `getFtSpecialAttrs`
pub unsafe extern "C" fn getFtSpecialAttrs(mut fp: *mut Fighter) -> *mut c_void {
    melee_core::ct!("getFtSpecialAttrs");
    let mut fighter_attr: *mut c_void = (*fp).dat_attrs;
    return fighter_attr;
}

/// Oracle-checked hand ports (rounds 2-12, tests/chars_dispatch.rs), re-exported under this TU.

// ---- folded from chars (consolidation step 20/21) ----
/// `ftSamus_DestroyAllUnsetx2444` (static) (src/melee/ft/kinds/ftSamus/ftsamusspecialhi.c).
pub fn ftSamus_DestroyAllUnsetx2444(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftSamus_DestroyAllUnsetx2444");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        st::efLib_DestroyAll(gobj);
        (*fp).u.ss.x2244 = 0_u32;
    }
}
