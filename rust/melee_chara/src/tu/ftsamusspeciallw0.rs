//! Mechanical native port of the C file named below, generated from the C by `rust/melee_world/gen/mtu/c2rs.py`
//! (libclang AST: every implicit conversion is the C one, `-ffp-contract=on` multiply-adds become `fmadds`).
//! Callees of other TUs are `ext` hooks named like the C (unset: panic "TODO"; `melee_link` wires them to the
//! canonical TU function). `bool` is MSL `int`.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, dead_code, unused_assignments, unused_variables, unused_mut, unused_parens, unused_labels, unused_comparisons, clippy::all)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use crate::chars::ftSamus::gobj_for;
#[allow(unused_imports)]
use crate::stubs as st;
#[allow(unused_imports)]
use melee_types::{Fighter, HSD_GObj};
#[allow(unused_imports)]
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
    fn ftAnim_8006EBA4(a0: *mut HSD_GObj);
    fn ftAnim_IsFramesRemaining(a0: *mut HSD_GObj) -> i32;
    fn ftCo_80099794(a0: *mut HSD_GObj) -> i32;
    fn ftCo_800D638C(a0: *mut HSD_GObj);
    fn ftCo_800D6824(a0: *mut HSD_GObj) -> i32;
    fn ftCo_800D68C0(a0: *mut HSD_GObj) -> i32;
    fn ftCo_Attack100_CheckInput(a0: *mut HSD_GObj) -> i32;
    fn ftCo_Attack1_CheckInput(a0: *mut HSD_GObj) -> i32;
    fn ftCo_AttackHi3_CheckInput(a0: *mut HSD_GObj) -> i32;
    fn ftCo_AttackHi4_CheckInput(a0: *mut HSD_GObj) -> i32;
    fn ftCo_AttackLw3_CheckInput(a0: *mut HSD_GObj) -> i32;
    fn ftCo_AttackLw4_CheckInput(a0: *mut HSD_GObj) -> i32;
    fn ftCo_AttackS3_CheckInput(a0: *mut HSD_GObj) -> i32;
    fn ftCo_AttackS4_CheckInput(a0: *mut HSD_GObj) -> i32;
    fn ftCo_Catch_CheckInput(a0: *mut HSD_GObj) -> i32;
    fn ftCo_Fall_Enter(a0: *mut HSD_GObj);
    fn ftCo_Fall_IASA_Inner(a0: *mut HSD_GObj) -> i32;
    fn ftCo_SpecialS_CheckInput(a0: *mut HSD_GObj) -> i32;
    fn ftColl_8007B868(a0: *mut HSD_GObj) -> i32;
    fn ftCommon_8007D5D4(a0: *mut Fighter);
    fn ftCommon_8007D7FC(a0: *mut Fighter);
    fn ftCommon_8007DB58(a0: *mut HSD_GObj);
    fn ftCommon_8007F804(a0: *mut Fighter) -> *mut [f32; 4];
    fn ftCommon_CalcGroundAccel_AccelToLStickX(a0: *mut Fighter, a1: f32, a2: f32, a3: f32);
    fn ftCommon_CalcSelfAccel_DriftSimple(a0: *mut Fighter, a1: f32, a2: f32, a3: f32);
    fn ftCommon_ClampSelfVelX(a0: *mut Fighter, a1: f32);
    fn ftCommon_FallBasic(a0: *mut Fighter);
    fn ftCommon_SetSelfMovementFromGroundedMovement(a0: *mut HSD_GObj);
    fn ft_80081D0C(a0: *mut HSD_GObj) -> GroundOrAir;
    fn ft_800824A0(a0: *mut HSD_GObj, a1: *mut ftCollisionBox) -> i32;
    fn ft_800827A0(a0: *mut HSD_GObj) -> i32;
    fn ft_80082888(a0: *mut HSD_GObj, a1: *mut ftCollisionBox) -> i32;
    fn ft_80084F3C(a0: *mut HSD_GObj);
    fn ft_8008A2BC(a0: *mut HSD_GObj);
    fn lbColl_80008248(a0: *mut HitCapsule, a1: *mut HurtCapsule, a2: *mut [f32; 4], a3: f32, a4: f32, a5: f32) -> i32;
    }
}



/// `ftSs_Init_80128944`
pub unsafe extern "C" fn ftSs_Init_80128944(mut gobj: *mut HSD_GObj, mut farg1: f32, mut farg2: f32) {
    melee_core::ct!("ftSs_Init_80128944");
    let mut fp: *mut Fighter = HSD_GObjGetUserData(gobj) as *mut Fighter;
    let mut da: *mut _ftSamusAttributes = (*fp).dat_attrs as *mut _ftSamusAttributes;
    let mut float_result: f32 = ftSs_Init_80128AC8(gobj, farg1, farg2);
    let mut __: [u8; 8] = [0; 8];
    if !(ext::ftColl_8007B868(gobj) != 0) {
        's5: {
            match (*fp).x2070.anon_0.x2071_b0_3() as i32 {
                0 | 2 | 3 | 4 => {
                    if (((*fp).x2070.anon_0.x2073 as i32) == 20) || (((*fp).x2070.anon_0.x2071_b5() as i32) == 0) {
                        if ((*fp).x5F4_arr[0 as usize].idx as i32) == 2 {
                            ftSs_Init_80128B1C(gobj, float_result, (*da).x0, 1.0f32);
                        } else {
                            ftSs_Init_80128B1C(gobj, float_result, 0.0f32, 1.0f32);
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

/// `ftSs_Init_80128A1C`
pub unsafe extern "C" fn ftSs_Init_80128A1C(mut gobj: *mut HSD_GObj, mut arg1: *mut c_void, mut farg1: f32) -> i32 {
    melee_core::ct!("ftSs_Init_80128A1C");
    let mut fp: *mut Fighter = HSD_GObjGetUserData(gobj) as *mut Fighter;
    let mut i: i32 = 0;
    {
        i = 0;
        'l6: while i < ((*fp).hurt_capsules_len as i32) {
            if ext::lbColl_80008248(arg1 as *mut HitCapsule, core::ptr::addr_of_mut!((*fp).hurt_capsules[i as usize].capsule) as *mut HurtCapsule, ext::ftCommon_8007F804(fp), farg1, (*fp).x34_scale.y, (*fp).cur_pos.z) != 0 {
                return 1;
            }
            i = i + 1;
        }
    }
    return 0;
}

/// `ftSs_Init_80128AC8`
pub unsafe extern "C" fn ftSs_Init_80128AC8(mut gobj: *mut HSD_GObj, mut farg1: f32, mut farg2: f32) -> f32 {
    melee_core::ct!("ftSs_Init_80128AC8");
    let mut fp: *mut Fighter = HSD_GObjGetUserData(gobj) as *mut Fighter;
    let mut da: *mut _ftSamusAttributes = getFtSpecialAttrs(fp) as *mut _ftSamusAttributes;
    let mut value: f32 = ((*fp).cur_pos.x - farg1) / farg2;
    if value >= 1.0f32 {
        value = 1.0f32;
    }
    if value <= (-1.0f32) {
        value = -1.0f32;
    }
    return fmadds(-(*da).x4, value, 1.5707964f32);
}

/// `ftSamus_80128B1C_inner`
pub unsafe extern "C" fn ftSamus_80128B1C_inner(mut gobj: *mut HSD_GObj, mut angle: f32) {
    melee_core::ct!("ftSamus_80128B1C_inner");
    let mut fp: *mut Fighter = HSD_GObjGetUserData(gobj) as *mut Fighter;
    let mut ftAttr: *mut ftCo_DatAttrs = core::ptr::addr_of_mut!((*fp).co_attrs) as *mut ftCo_DatAttrs;
    let mut samus_attr: *mut _ftSamusAttributes = getFtSpecialAttrs(fp) as *mut _ftSamusAttributes;
    fp = HSD_GObjGetUserData(gobj) as *mut Fighter;
    (*fp).self_vel.x = (*samus_attr).x8 * melee_runtime::gekko::cosf(angle);
    (*fp).self_vel.y = (*samus_attr).x8 * melee_runtime::gekko::sinf(angle);
    ext::ftCommon_ClampSelfVelX(fp, (*ftAttr).air_drift_max * (*samus_attr).x10);
}

/// `ftSs_Init_80128B1C`
pub unsafe extern "C" fn ftSs_Init_80128B1C(mut gobj: *mut HSD_GObj, mut angle: f32, mut arg9: f32, mut argA: f32) {
    melee_core::ct!("ftSs_Init_80128B1C");
    let mut fp: *mut Fighter = null_mut();
    let mut fighter2: *mut Fighter = null_mut();
    let mut __: [u8; 8] = [0; 8];
    fp = { fighter2 = HSD_GObjGetUserData(gobj) as *mut Fighter; fighter2 };
    ext::ftCommon_8007DB58(gobj);
    ftSamus_80128B1C_inner(gobj, angle);
    (*fp).cmd_vars[0 as usize] = 0 as u32;
    (*fp).cmd_vars[1 as usize] = 0 as u32;
    (*fp).mv.ss.unk2.x0 = 0;
    if (*fp).ground_or_air == ((GA_Ground as i32) as u32) {
        ext::ftCommon_8007D5D4(fighter2);
    }
    ext::Fighter_ChangeMotionState(gobj, 342, 0 as u32, arg9, argA, 0.0f32, null_mut());
    ext::ftAnim_8006EBA4(gobj);
}

/// `ftSs_SpecialLw_Anim`
pub unsafe extern "C" fn ftSs_SpecialLw_Anim(mut gobj: *mut HSD_GObj) {
    melee_core::ct!("ftSs_SpecialLw_Anim");
    let mut fp: *mut Fighter = HSD_GObjGetUserData(gobj) as *mut Fighter;
    if ((*fp).cmd_vars[0 as usize] != 0) && (!((*fp).mv.ss.unk2.x0 != 0)) {
        super::ftsamusspeciallw1::ftSs_SpecialLw_8012AEBC(gobj);
        (*fp).mv.ss.unk2.x0 = 1;
    }
    if (!((*fp).cmd_vars[0 as usize] != 0)) && ((*fp).mv.ss.unk2.x0 != 0) {
        super::ftsamusspeciallw1::ftSs_SpecialLw_8012AF38(gobj);
        (*fp).mv.ss.unk2.x0 = 0;
    }
    if !(ext::ftAnim_IsFramesRemaining(gobj) != 0) {
        ext::ft_8008A2BC(gobj);
    }
}

/// `ftSs_SpecialAirLw_Anim`
pub unsafe extern "C" fn ftSs_SpecialAirLw_Anim(mut gobj: *mut HSD_GObj) {
    melee_core::ct!("ftSs_SpecialAirLw_Anim");
    let mut fp: *mut Fighter = HSD_GObjGetUserData(gobj) as *mut Fighter;
    if ((*fp).cmd_vars[0 as usize] != 0) && (!((*fp).mv.ss.unk2.x0 != 0)) {
        super::ftsamusspeciallw1::ftSs_SpecialLw_8012AEBC(gobj);
        (*fp).mv.ss.unk2.x0 = 1;
    }
    if (!((*fp).cmd_vars[0 as usize] != 0)) && ((*fp).mv.ss.unk2.x0 != 0) {
        super::ftsamusspeciallw1::ftSs_SpecialLw_8012AF38(gobj);
        (*fp).mv.ss.unk2.x0 = 0;
    }
    if !(ext::ftAnim_IsFramesRemaining(gobj) != 0) {
        ext::ftCo_Fall_Enter(gobj);
    }
}

/// `ftSs_SpecialLw_IASA`
pub unsafe extern "C" fn ftSs_SpecialLw_IASA(mut gobj: *mut HSD_GObj) {
    melee_core::ct!("ftSs_SpecialLw_IASA");
    let mut __: [u8; 8] = [0; 8];
    let mut fp: *mut Fighter = HSD_GObjGetUserData(gobj) as *mut Fighter;
    let mut da: *mut _ftSamusAttributes = (*fp).dat_attrs as *mut _ftSamusAttributes;
    if ((*fp).cmd_vars[1 as usize] != 0) && ((*fp).input.lstick[0 as usize].y < (*da).x14) {
        (*fp).cmd_vars[1 as usize] = 0 as u32;
        ext::ftCo_800D638C(gobj);
        return;
    }
    {
        if ext::ftCo_SpecialS_CheckInput(gobj) != 0 {
            return;
        }
    }
    {
        if ext::ftCo_Attack100_CheckInput(gobj) != 0 {
            return;
        }
    }
    {
        if ext::ftCo_800D6824(gobj) != 0 {
            return;
        }
    }
    {
        if ext::ftCo_800D68C0(gobj) != 0 {
            return;
        }
    }
    {
        if ext::ftCo_Catch_CheckInput(gobj) != 0 {
            return;
        }
    }
    {
        if ext::ftCo_AttackS4_CheckInput(gobj) != 0 {
            return;
        }
    }
    {
        if ext::ftCo_AttackHi4_CheckInput(gobj) != 0 {
            return;
        }
    }
    {
        if ext::ftCo_AttackLw4_CheckInput(gobj) != 0 {
            return;
        }
    }
    {
        if ext::ftCo_AttackS3_CheckInput(gobj) != 0 {
            return;
        }
    }
    {
        if ext::ftCo_AttackHi3_CheckInput(gobj) != 0 {
            return;
        }
    }
    {
        if ext::ftCo_AttackLw3_CheckInput(gobj) != 0 {
            return;
        }
    }
    {
        if ext::ftCo_Attack1_CheckInput(gobj) != 0 {
            return;
        }
    }
    {
        if ext::ftCo_80099794(gobj) != 0 {
            return;
        }
    }
}

/// `ftSs_SpecialAirLw_IASA`
pub unsafe extern "C" fn ftSs_SpecialAirLw_IASA(mut gobj: *mut HSD_GObj) {
    melee_core::ct!("ftSs_SpecialAirLw_IASA");
    let _ = ext::ftCo_Fall_IASA_Inner(gobj);
}

/// `ftSs_SpecialLw_Phys`
pub unsafe extern "C" fn ftSs_SpecialLw_Phys(mut gobj: *mut HSD_GObj) {
    melee_core::ct!("ftSs_SpecialLw_Phys");
    let mut fp: *mut Fighter = HSD_GObjGetUserData(gobj) as *mut Fighter;
    let mut samus_attr: *mut _ftSamusAttributes = (*fp).dat_attrs as *mut _ftSamusAttributes;
    let mut __: [u8; 4] = [0; 4];
    let mut ftAttr: *mut ftCo_DatAttrs = core::ptr::addr_of_mut!((*fp).co_attrs) as *mut ftCo_DatAttrs;
    if (*fp).cmd_vars[0 as usize] != 0 {
        let mut samus_attr_xC: f32 = (*samus_attr).xC;
        ext::ftCommon_CalcGroundAccel_AccelToLStickX(fp, 0.0f32, (*ftAttr).walk_accel_mul * samus_attr_xC, (*ftAttr).walk_max_vel * samus_attr_xC);
        ext::ftCommon_SetSelfMovementFromGroundedMovement(gobj);
    } else {
        ext::ft_80084F3C(gobj);
    }
}

/// `ftSs_SpecialAirLw_Phys`
pub unsafe extern "C" fn ftSs_SpecialAirLw_Phys(mut gobj: *mut HSD_GObj) {
    melee_core::ct!("ftSs_SpecialAirLw_Phys");
    let mut fp: *mut Fighter = HSD_GObjGetUserData(gobj) as *mut Fighter;
    let mut samus_attr: *mut _ftSamusAttributes = (*fp).dat_attrs as *mut _ftSamusAttributes;
    let mut ftAttr: *mut ftCo_DatAttrs = core::ptr::addr_of_mut!((*fp).co_attrs) as *mut ftCo_DatAttrs;
    let mut __: [u8; 8] = [0; 8];
    ext::ftCommon_FallBasic(fp);
    ext::ftCommon_CalcSelfAccel_DriftSimple(fp, 0.0f32, (*ftAttr).ground_to_air_jump_momentum_multiplier * (*samus_attr).x10, (*ftAttr).jump_h_max_velocity * (*samus_attr).x10);
}

/// `ftSs_SpecialLw_Coll`
pub unsafe extern "C" fn ftSs_SpecialLw_Coll(mut gobj: *mut HSD_GObj) {
    melee_core::ct!("ftSs_SpecialLw_Coll");
    let mut fp: *mut Fighter = HSD_GObjGetUserData(gobj) as *mut Fighter;
    let mut samus_attr: *mut _ftSamusAttributes = (*fp).dat_attrs as *mut _ftSamusAttributes;
    let mut __: [u8; 8] = [0; 8];
    if (*fp).cmd_vars[0 as usize] != 0 {
        if !(ext::ft_80082888(gobj, core::ptr::addr_of_mut!((*samus_attr).height_attributes) as *mut ftCollisionBox) != 0) {
            ftSs_SpecialLw_80129048(gobj);
        }
    } else if !(ext::ft_800827A0(gobj) != 0) {
        ftSs_SpecialLw_80129048(gobj);
    }
}

/// `ftSs_SpecialAirLw_Coll`
pub unsafe extern "C" fn ftSs_SpecialAirLw_Coll(mut gobj: *mut HSD_GObj) {
    melee_core::ct!("ftSs_SpecialAirLw_Coll");
    let mut fp: *mut Fighter = HSD_GObjGetUserData(gobj) as *mut Fighter;
    let mut samus_attr: *mut _ftSamusAttributes = (*fp).dat_attrs as *mut _ftSamusAttributes;
    let mut __: [u8; 8] = [0; 8];
    if (*fp).cmd_vars[0 as usize] != 0 {
        if ext::ft_800824A0(gobj, core::ptr::addr_of_mut!((*samus_attr).height_attributes) as *mut ftCollisionBox) != 0 {
            ftSs_SpecialLw_801290A4(gobj);
        }
    } else if ext::ft_80081D0C(gobj) != 0 {
        ftSs_SpecialLw_801290A4(gobj);
    }
}





/// `ftSs_SpecialLw_80129100`
pub unsafe extern "C" fn ftSs_SpecialLw_80129100(mut gobj: *mut HSD_GObj, mut arg1: *mut i32, mut arg2: *mut i32) -> i32 {
    melee_core::ct!("ftSs_SpecialLw_80129100");
    if !gobj.is_null() {
        let mut fp: *mut Fighter = HSD_GObjGetUserData(gobj) as *mut Fighter;
        let mut samus_attr: *mut _ftSamusAttributes = (*fp).dat_attrs as *mut _ftSamusAttributes;
        let mut __: [u8; 4] = [0; 4];
        if !(!(*fp).u.ss.x222C.is_null()) {
            return (-1_i32);
        }
        *arg1 = (*fp).u.ss.x2230;
        *arg2 = (*samus_attr).x18 as i32;
        return 0;
    }
    return (-1_i32);
}

/// `ftSs_SpecialLw_80129158`
pub unsafe extern "C" fn ftSs_SpecialLw_80129158(mut gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("ftSs_SpecialLw_80129158");
    if !gobj.is_null() {
        let mut fp: *mut Fighter = HSD_GObjGetUserData(gobj) as *mut Fighter;
        let mut motion_state_index: i32 = (*fp).motion_id;
        's7: {
            match motion_state_index {
                343 | 344 | 345 | 346 | 347 | 348 => {
                    if (*fp).x2070.anon_0.x2071_b6() != 0 {
                        return 1;
                    }
                    return 0;
                }
                _ => {}
            }
        }
        return 1;
    }
    return 1;
}

/// `ftSs_SpecialN_801291A8`
pub unsafe extern "C" fn ftSs_SpecialN_801291A8(mut gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("ftSs_SpecialN_801291A8");
    if !gobj.is_null() {
        let mut fp: *mut Fighter = HSD_GObjGetUserData(gobj) as *mut Fighter;
        let mut motion_state_index: i32 = (*fp).motion_id;
        's8: {
            match motion_state_index {
                343 | 344 | 346 | 347 | 348 => {
                    return 0;
                }
                _ => {}
            }
        }
        return 1;
    }
    return 1;
}

/// `HSD_GObjGetUserData`
pub unsafe extern "C" fn HSD_GObjGetUserData(mut gobj: *mut HSD_GObj) -> *mut c_void {
    melee_core::ct!("HSD_GObjGetUserData");
    return (*gobj).user_data;
}

/// `getFtSpecialAttrs`
pub unsafe extern "C" fn getFtSpecialAttrs(mut fp: *mut Fighter) -> *mut c_void {
    melee_core::ct!("getFtSpecialAttrs");
    let mut fighter_attr: *mut c_void = (*fp).dat_attrs;
    return fighter_attr;
}

/// Oracle-checked hand ports (rounds 2-12, tests/chars_dispatch.rs), re-exported under this TU.

// ---- folded from chars (consolidation step 20/21) ----
/// `ftSs_SpecialLw_80129048` (src/melee/ft/kinds/ftSamus/ftsamusspeciallw0.c).
pub fn ftSs_SpecialLw_80129048(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftSs_SpecialLw_80129048");
    let fp = unsafe { &mut *((*gobj).user_data as *mut Fighter) };
    st::ftCommon_8007D5D4(fp);
    st::Fighter_ChangeMotionState(gobj, 342 /* 0x156 */, 0x10u32 /* 0x10 */, fp.cur_anim_frame, fp.frame_speed_mul, 0.0_f32, core::ptr::null_mut());
}

/// `ftSs_SpecialLw_801290A4` (src/melee/ft/kinds/ftSamus/ftsamusspeciallw0.c).
pub fn ftSs_SpecialLw_801290A4(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftSs_SpecialLw_801290A4");
    let fp = unsafe { &mut *((*gobj).user_data as *mut Fighter) };
    st::ftCommon_8007D7FC(fp);
    st::Fighter_ChangeMotionState(gobj, 341 /* 0x155 */, 0x10u32 /* 0x10 */, fp.cur_anim_frame, fp.frame_speed_mul, 0.0_f32, core::ptr::null_mut());
}

// ---- folded from chars blocks (consolidation step 20) ----
pub fn ftSs_SpecialAirLw_Phys_fp(fp: &mut Fighter) {
    melee_core::ct!("ftSs_SpecialAirLw_Phys_fp");
    unsafe { crate::tu::ftsamusspeciallw0::ftSs_SpecialAirLw_Phys(&mut gobj_for(fp)) }
}

/// The gobj may be NULL (then 1).
pub fn ftSs_SpecialLw_80129158_fp(fp: Option<&Fighter>) -> i32 {
    melee_core::ct!("ftSs_SpecialLw_80129158_fp");
    unsafe { crate::tu::ftsamusspeciallw0::ftSs_SpecialLw_80129158(opt_gobj(fp)) }
}

/// The gobj may be NULL (then 1).
pub fn ftSs_SpecialN_801291A8_fp(fp: Option<&Fighter>) -> i32 {
    melee_core::ct!("ftSs_SpecialN_801291A8_fp");
    unsafe { crate::tu::ftsamusspeciallw0::ftSs_SpecialN_801291A8(opt_gobj(fp)) }
}

pub fn opt_gobj(fp: Option<&Fighter>) -> *mut melee_types::HSD_GObj {
    melee_core::ct!("opt_gobj");
    match fp {
        Some(f) => Box::leak(Box::new(gobj_for(f))),
        None => core::ptr::null_mut(),
    }
}
