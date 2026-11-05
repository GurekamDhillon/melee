//! src/melee/ft/kinds/ftCrazyHand/ftcrazyhandfingerbeam.c: `ftCh_Init_80158B3C`, the finger-beam
//! windup state. No float math.
#![allow(non_snake_case, unused_unsafe)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use melee_types::{Fighter, HSD_GObj};
#[allow(unused_imports)]
use crate::stubs as st;
use melee_types::*;

/// Calls into other TUs (settable hooks; unset they run the port).
pub mod ext {
    use melee_types::*;
    crate::port_hook!(FIGHTER_CHANGEMOTIONSTATE, Fighter_ChangeMotionState, crate::adapt::Fighter_ChangeMotionState, (gobj: *mut HSD_GObj, msid: i32, flags: u32, anim_start: f32, anim_speed: f32, anim_blend: f32, arg3: *mut HSD_GObj));
    crate::port_hook!(FTANIM_SETANIMRATE, ftAnim_SetAnimRate, crate::adapt::ftAnim_SetAnimRate, (gobj: *mut HSD_GObj, rate: f32));
}

/// `ftCh_Init_80158B3C`.
pub fn ftCh_Init_80158B3C(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_Init_80158B3C");
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        let da = (*(*fp).ft_data).ext_attr as *const ftCrazyHand_DatAttrs;
        ext::Fighter_ChangeMotionState(gobj, 0x16A, 0, (*fp).cur_anim_frame, 1.0, 0.0, core::ptr::null_mut());
        ext::ftAnim_SetAnimRate(gobj, (*da).xCC_pos.x);
        (*fp).mv.ch.unk0.x8 = melee_core::ppc::fctiwz_f32((*da).xC4_pos.y); // C float -> s32 truncation
    }
}

/// `ftCh_Init_80158F34` (src/melee/ft/kinds/ftCrazyHand/ftcrazyhandfingerbeam.c).
    pub fn ftCh_Init_80158F34(gobj: *mut HSD_GObj) {
        unsafe {
            let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
            st::Fighter_ChangeMotionState(gobj, 370 /* 0x172 */, 0u32, 0.0_f32, 1.0_f32, 0.0_f32, core::ptr::null_mut());
            st::ftAnim_8006EBA4(gobj);
            (*fp).mv.ch.unk0.xC.x = (*((*(*fp).ft_data).ext_attr as *const melee_types::ftCrazyHand_DatAttrs)).x18;
            (*fp).mv.ch.unk0.xC.y = (*((*(*fp).ft_data).ext_attr as *const melee_types::ftCrazyHand_DatAttrs)).x1C;
            (*fp).mv.ch.unk0.xC.z = f32::from_bits(0x00000000) /* 0.0f */;
        }
    }

// ---- folded from chars blocks (consolidation step 20) ----
/// `ftCh_FingerBeamEnd_Phys` (src/melee/ft/kinds/ftCrazyHand/ftcrazyhandfingerbeam.c).
pub fn ftCh_FingerBeamEnd_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_FingerBeamEnd_Phys");
    st::ft_80085134(gobj);
}

// ---- folded from chars blocks (consolidation step 20) ----
/// `ftCh_Init_80159098` (src/melee/ft/kinds/ftCrazyHand/ftcrazyhandfingerbeam.c).
pub fn ftCh_Init_80159098(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_Init_80159098");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        st::Fighter_ChangeMotionState(gobj, 364 /* 0x16C */, 0u32, 0.0_f32, 1.0_f32, 0.0_f32, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
        (*fp).mv.ch.unk0.x24 = (*((*(*fp).ft_data).ext_attr as *const melee_types::ftCrazyHand_DatAttrs)).xD8;
        (*fp).cmd_vars[0] = 1_u32;
        (*fp).self_vel.x = f32::from_bits(0x00000000) /* 0 */;
        (*fp).self_vel.y = f32::from_bits(0x00000000) /* 0 */;
        (*fp).self_vel.z = f32::from_bits(0x00000000) /* 0 */;
        (*fp).mv.ch.unk0.xC.x = (*((*(*fp).ft_data).ext_attr as *const melee_types::ftCrazyHand_DatAttrs)).x18;
        (*fp).mv.ch.unk0.xC.y = (*((*(*fp).ft_data).ext_attr as *const melee_types::ftCrazyHand_DatAttrs)).x1C;
        (*fp).mv.ch.unk0.xC.z = f32::from_bits(0x00000000) /* 0 */;
    }
}

/// `fn_80159288` (src/melee/ft/kinds/ftCrazyHand/ftcrazyhandfingerbeam.c).
pub fn fn_80159288(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_80159288");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        (*fp).self_vel.z = f32::from_bits(0x00000000) /* 0 */;
        (*fp).self_vel.y = f32::from_bits(0x00000000) /* 0 */;
        (*fp).self_vel.x = f32::from_bits(0x00000000) /* 0 */;
        (*fp).mv.ch.unk0.x20 = (1_i32 /* true */);
        (*fp).set_x221E_b6(0);
        crate::chars::r9::ftCrazyHand::ftcrazyhandfingergun2::ftCh_GrabUnk1_8015ABD0(gobj);
    }
}

// ---- folded from chars blocks (consolidation step 20) ----
pub fn ftCh_FingerGun1_Coll_fp(fp: &mut Fighter) {
    melee_core::ct!("ftCh_FingerGun1_Coll");
    crate::chars::ftCrazyHand::stop_if_x18_zero(fp)
}

/// `ftCh_FingerGun1_Coll` (src/melee/ft/kinds/ftCrazyHand/ftcrazyhandfingerbeam.c): the C signature.
pub fn ftCh_FingerGun1_Coll(gobj: *mut HSD_GObj) {
    ftCh_FingerGun1_Coll_fp(unsafe { &mut *((*gobj).user_data as *mut Fighter) })
}

// ---- folded from chars blocks (consolidation step 20) ----
/// `ftCh_FingerBeamEnd_Anim` (src/melee/ft/kinds/ftCrazyHand/ftcrazyhandfingerbeam.c).
pub fn ftCh_FingerBeamEnd_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_FingerBeamEnd_Anim");
    unsafe {
        if (st::ftAnim_IsFramesRemaining(gobj) == 0) {
            let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
            (*fp).self_vel.z = f32::from_bits(0x00000000) /* 0 */;
            (*fp).self_vel.x = f32::from_bits(0x00000000) /* 0 */;
            (*fp).self_vel.y = f32::from_bits(0x00000000) /* 0 */;
            crate::chars::r9::ftCrazyHand::ftcrazyhandfingergun2::ftCh_GrabUnk1_8015AC50(gobj);
        }
    }
}

// ---- folded from chars blocks (consolidation step 20) ----
/// `ftCh_FingerBeamLoop_IASA` (src/melee/ft/kinds/ftCrazyHand/ftcrazyhandfingerbeam.c).
pub fn ftCh_FingerBeamLoop_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_FingerBeamLoop_IASA");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        if (st::Player_GetPlayerSlotType(((*fp).player_id as i32)) == 0 /* Gm_PKind_Human */) {
            st::ftBossLib_8015BD20(gobj);
        }
    }
}

/// `ftCh_FingerBeamEnd_IASA` (src/melee/ft/kinds/ftCrazyHand/ftcrazyhandfingerbeam.c).
pub fn ftCh_FingerBeamEnd_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_FingerBeamEnd_IASA");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        if (st::Player_GetPlayerSlotType(((*fp).player_id as i32)) == 0 /* Gm_PKind_Human */) {
            st::ftBossLib_8015BD20(gobj);
        }
    }
}

/// `ftCh_BackPunch_IASA` (src/melee/ft/kinds/ftCrazyHand/ftcrazyhandfingerbeam.c).
pub fn ftCh_BackPunch_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_BackPunch_IASA");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        if (st::Player_GetPlayerSlotType(((*fp).player_id as i32)) == 0 /* Gm_PKind_Human */) {
            st::ftBossLib_8015BD20(gobj);
        }
    }
}

/// `ftCh_FingerGun1_IASA` (src/melee/ft/kinds/ftCrazyHand/ftcrazyhandfingerbeam.c).
pub fn ftCh_FingerGun1_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_FingerGun1_IASA");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        if (st::Player_GetPlayerSlotType(((*fp).player_id as i32)) == 0 /* Gm_PKind_Human */) {
            st::ftBossLib_8015BD20(gobj);
        }
    }
}

/// `fn_80159288` as the `grab_cb` (`HSD_GObjEvent`) value.
pub unsafe extern "C" fn cb_fn_80159288(gobj: *mut HSD_GObj) {
    fn_80159288(gobj)
}

/// `ftCh_FingerBeamLoop_Anim` (src/melee/ft/kinds/ftCrazyHand/ftcrazyhandfingerbeam.c).
pub fn ftCh_FingerBeamLoop_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_FingerBeamLoop_Anim");
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        let timer = (*fp).mv.ch.unk0.x8.wrapping_sub(1);
        let is_zero = timer == 0;
        (*fp).mv.ch.unk0.x8 = timer;
        if is_zero {
            ext::ftAnim_SetAnimRate(gobj, 1.0);
            ftCh_Init_80158DFC(gobj);
        }
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            ext::Fighter_ChangeMotionState(gobj, 0x16A, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
            st::ftAnim_8006EBA4(gobj);
        }
    }
}

/// `ftCh_FingerBeamLoop_Phys` (src/melee/ft/kinds/ftCrazyHand/ftcrazyhandfingerbeam.c).
/// `my_lbVector_Len` holds the two fused f32 sites (fma_sites.py): the shared Master Hand copy.
pub fn ftCh_FingerBeamLoop_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_FingerBeamLoop_Phys");
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        let da = (*(*fp).ft_data).ext_attr as *const ftCrazyHand_DatAttrs;
        let mut pos: Vec3 = core::mem::zeroed();
        let mut vel: Vec3 = core::mem::zeroed();
        st::ft_80085134(gobj);
        crate::adapt::ftBossLib_8015C208(gobj, &mut pos);
        pos.x += (*da).xBC_pos.y;
        pos.y += (*da).xC4_pos.x;
        pos.z = 0.0;
        melee_runtime::tu::lbvector::lbVector_Diff(&mut pos as *mut _ as *mut _, &mut (*fp).cur_pos as *mut _ as *mut _, &mut vel as *mut _ as *mut _);
        let len = crate::tu::ftmasterhandbackdisappear::my_lbVector_Len(&vel);
        if len < (*da).x14 {
            (*fp).self_vel.x = vel.x;
            (*fp).self_vel.y = vel.y;
        } else {
            melee_runtime::tu::lbvector::lbVector_Normalize(&mut vel as *mut _ as *mut _);
            let speed = len * (*da).x10;
            vel.x *= speed;
            vel.y *= speed;
            vel.z *= speed;
            (*fp).self_vel.x = vel.x;
            (*fp).self_vel.y = vel.y;
        }
    }
}

/// `ftCh_FingerBeamLoop_Coll` (src/melee/ft/kinds/ftCrazyHand/ftcrazyhandfingerbeam.c).
pub fn ftCh_FingerBeamLoop_Coll(_gobj: *mut HSD_GObj) { melee_core::ct!("ftCh_FingerBeamLoop_Coll"); }

/// `ftCh_Init_80158DFC` (src/melee/ft/kinds/ftCrazyHand/ftcrazyhandfingerbeam.c).
pub fn ftCh_Init_80158DFC(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_Init_80158DFC");
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        ext::Fighter_ChangeMotionState(gobj, 0x16B, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
        melee_fighter::ftcommon2::ftcommon_8007e2d0(
            &mut *fp,
            0x100,
            Some(cb_fn_80159288),
            None,
            Some(crate::tu::ftcrazyhandcapturecrazyhand::cb_fn_8015B548),
        );
        (*fp).mv.ch.unk0.x20 = 0;
    }
}

/// `ftCh_FingerBeamEnd_Coll` (src/melee/ft/kinds/ftCrazyHand/ftcrazyhandfingerbeam.c).
pub fn ftCh_FingerBeamEnd_Coll(_gobj: *mut HSD_GObj) { melee_core::ct!("ftCh_FingerBeamEnd_Coll"); }

/// `ftCh_BackPunch_Anim` (src/melee/ft/kinds/ftCrazyHand/ftcrazyhandfingerbeam.c).
pub fn ftCh_BackPunch_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_BackPunch_Anim");
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        crate::tu::ftcrazyhandfingerbeam0::ext::ftCh_GrabUnk1_8015BC88(gobj);
    }
}

/// `ftCh_BackPunch_Phys` (src/melee/ft/kinds/ftCrazyHand/ftcrazyhandfingerbeam.c).
pub fn ftCh_BackPunch_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_BackPunch_Phys");
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        let da = (*(*fp).ft_data).ext_attr as *const ftCrazyHand_DatAttrs;
        crate::tu::ftcrazyhandfingergun2::ftCh_UpdateBossMotion(gobj, fp, da);
    }
}

/// `ftCh_BackPunch_Coll` (src/melee/ft/kinds/ftCrazyHand/ftcrazyhandfingerbeam.c).
pub fn ftCh_BackPunch_Coll(_gobj: *mut HSD_GObj) { melee_core::ct!("ftCh_BackPunch_Coll"); }

/// `ftCh_FingerGun1_Anim` (src/melee/ft/kinds/ftCrazyHand/ftcrazyhandfingerbeam.c). The x24
/// pre-decrement is a float countdown.
pub fn ftCh_FingerGun1_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_FingerGun1_Anim");
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        (*fp).mv.ch.unk0.x24 -= 1.0;
        if (*fp).mv.ch.unk0.x24 <= 0.0 && (*fp).cmd_vars[0] != 0 {
            crate::tu::ftcrazyhandcapturewaitcrazyhand::ftCh_GrabUnk1_8015B800((*fp).victim_gobj as *mut HSD_GObj);
            (*fp).mv.ch.unk0.x20 = 0;
            (*fp).cmd_vars[0] = 0;
        }
    }
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        crate::tu::ftcrazyhandfingerbeam0::ext::ftCh_GrabUnk1_8015BC88(gobj);
    }
}

/// `ftCh_FingerGun1_Phys` (src/melee/ft/kinds/ftCrazyHand/ftcrazyhandfingerbeam.c).
pub fn ftCh_FingerGun1_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_FingerGun1_Phys");
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        let da = (*(*fp).ft_data).ext_attr as *const ftCrazyHand_DatAttrs;
        crate::tu::ftcrazyhandfingergun2::ftCh_UpdateBossMotion(gobj, fp, da);
    }
}
