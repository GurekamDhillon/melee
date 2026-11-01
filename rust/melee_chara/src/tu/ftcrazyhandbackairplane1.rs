//! src/melee/ft/kinds/ftCrazyHand/ftcrazyhandbackairplane1.c: Crazy Hand back-airplane (start).
//! No float arithmetic (no fused sites; the velocity clears store 0). Smoke-tested by
//! tests/tu_ftcrazyhandbackairplane1.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
#[allow(unused_imports)]
#[allow(unused_imports)]
use crate::tu::ftcrazyhandbackairplane3::ftCh_Init_80159670;
use crate::tu::ftcrazyhandbackairplane2::ftCh_Init_801594D4;
use melee_types::{Fighter, HSD_GObj};
use crate::stubs as st;
use melee_types::*;

/// Already ported (round 8): re-exported under this TU. The round-4 Init fns belong to the
/// airplane2/3 TUs and are only called from here.




/// Calls into TUs that are not ported yet. Each is a settable hook; unset it panics.
pub mod ext {
    use melee_types::*;
    /// `ftCh_GrabUnk1_8015B670` (ftcrazyhandcapturedamagecrazyhand.c): the round-7 native port
    /// when the hook is unset.
    pub static mut FTCH_GRABUNK1_8015B670: Option<fn(*mut HSD_GObj)> = None;
    #[allow(non_snake_case)]
    pub fn ftCh_GrabUnk1_8015B670(gobj: *mut HSD_GObj) {
        match unsafe { FTCH_GRABUNK1_8015B670 } {
            Some(f) => f(gobj),
            None => crate::chars::r7::ftCrazyHand::ftcrazyhandcapturedamagecrazyhand::ftCh_GrabUnk1_8015B670(gobj),
        }
    }
}

#[inline(always)]
unsafe fn fpp(gobj: *mut HSD_GObj) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}

pub fn ftCh_Init_801592D4(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_Init_801592D4");
    unsafe {
        let fp = fpp(gobj);
        let da = (*(*fp).ft_data).ext_attr as *const ftCrazyHand_DatAttrs;
        (*fp).cmd_vars[1] = 0;
        st::Fighter_ChangeMotionState(gobj, 0x16F, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
        (*fp).set_x2222_b2(1);
        st::ftCommon_8007E2F4(&mut *fp, 511);
        st::ftCommon_8007E2FC(gobj);
        ext::ftCh_GrabUnk1_8015B670((*fp).victim_gobj);
        (*fp).mv.ch.unk0.xC.x = (*da).xCC_pos.y;
        (*fp).mv.ch.unk0.xC.y = (*da).xD4;
        (*fp).mv.ch.unk0.xC.z = 0.0;
    }
}

pub fn ftCh_BackAirplane1_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_BackAirplane1_Anim");
    unsafe {
        let fp = fpp(gobj);
        if (*fp).cmd_vars[1] != 0 {
            melee_fighter::tu::ftbosslib::ftBossLib_8015C5F8(gobj);
            (*fp).cmd_vars[1] = 0;
        }
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            let fp = fpp(gobj);
            if (*fp).u.mh.x2250 == 0x170 {
                ftCh_Init_801594D4(gobj);
            } else {
                ftCh_Init_80159670(gobj);
            }
        }
    }
}

/// `ftCh_UpdateBossMotion(gobj, fp, da)` (ftCrazyHand/inlines.h, static inline).
pub fn ftCh_UpdateBossMotion(gobj: *mut HSD_GObj, fp: *mut Fighter, da: *const ftCrazyHand_DatAttrs) {
    melee_core::ct!("ftCh_UpdateBossMotion");
    unsafe {
        st::ft_80085134(gobj);
        let target = (*fp).mv.ch.unk0.xC;
        melee_fighter::tu::ftbosslib::ftBossLib_8015BE40(
            gobj,
            &target,
            &mut (*fp).mv.ch.unk0.x18,
            (*da).x14,
            (*da).x10,
        );
    }
}

pub fn ftCh_BackAirplane1_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_BackAirplane1_Phys");
    unsafe {
        let fp = fpp(gobj);
        let da = (*(*fp).ft_data).ext_attr as *const ftCrazyHand_DatAttrs;
        ftCh_UpdateBossMotion(gobj, fp, da);
    }
}

pub fn ftCh_BackAirplane1_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_BackAirplane1_Coll");
    unsafe {
        let fp = fpp(gobj);
        if (*fp).mv.ch.unk0.x18 == 0.0 {
            (*fp).self_vel.z = 0.0;
            (*fp).self_vel.y = 0.0;
            (*fp).self_vel.x = 0.0;
        }
    }
}





/// `ftCh_BackAirplane1_IASA` (src/melee/ft/kinds/ftCrazyHand/ftcrazyhandbackairplane1.c).
    pub fn ftCh_BackAirplane1_IASA(gobj: *mut HSD_GObj) {
        unsafe {
            let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
            if (st::Player_GetPlayerSlotType(((*fp).player_id as i32)) == 0 /* Gm_PKind_Human */) {
                st::ftBossLib_8015BD20(gobj);
            }
        }
    }

// ---- folded from chars blocks (consolidation step 20) ----
pub fn ftCh_BackAirplane1_Coll_fp(fp: &mut Fighter) {
    melee_core::ct!("ftCh_BackAirplane1_Coll_fp");
    crate::chars::ftCrazyHand::stop_if_x18_zero(fp)
}
