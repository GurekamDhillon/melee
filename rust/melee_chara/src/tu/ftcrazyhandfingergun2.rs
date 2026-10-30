//! src/melee/ft/kinds/ftCrazyHand/ftcrazyhandfingergun2.c: Crazy Hand's second finger-gun state.
//! Also holds `ftCh_UpdateBossMotion` (static inline of ftCrazyHand/inlines.h). No fused sites.
//! Smoke-tested by tests/tu_ftcrazyhandfingergun2.rs.
#![allow(non_snake_case, unused_unsafe, static_mut_refs)]

use crate::stubs as st;
use melee_types::*;

/// Already ported (rounds 8/9): re-exported under this TU.
pub use crate::chars::r8::ftCrazyHand::ftcrazyhandfingergun2::ftCh_FingerGun2_IASA;
pub use crate::chars::r9::ftCrazyHand::ftcrazyhandfingergun2::{ftCh_GrabUnk1_8015ABD0, ftCh_GrabUnk1_8015AC50};
use crate::chars::r9::ftCrazyHand::ftcrazyhandfingerbeam::ftCh_Init_80158F34;

pub mod ext {
    use melee_types::*;
    pub static mut FTCH_INIT_801592D4: Option<fn(*mut HSD_GObj)> = None;
    #[allow(non_snake_case)]
    pub fn ftCh_Init_801592D4(gobj: *mut HSD_GObj) {
        match unsafe { FTCH_INIT_801592D4 } {
            Some(f) => f(gobj),
            None => { unsafe { let _ = crate::tu::ftcrazyhandbackairplane1::ftCh_Init_801592D4(gobj as _); } },
        }
    }
}

/// `ftCh_UpdateBossMotion` (ftCrazyHand/inlines.h, static inline).
pub unsafe fn ftCh_UpdateBossMotion(gobj: *mut HSD_GObj, fp: *mut Fighter, da: *const ftCrazyHand_DatAttrs) {
    melee_core::ct!("ftCh_UpdateBossMotion");
    unsafe {
        st::ft_80085134(gobj);
        let xc = (*fp).mv.ch.unk0.xC;
        melee_fighter::tu::ftbosslib::ftBossLib_8015BE40(gobj, &xc, &mut (*fp).mv.ch.unk0.x18, (*da).x14, (*da).x10);
    }
}

fn ftCh_FingerGun2_Anim_inline(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        if (*fp).mv.ch.unk0.x20 == 1 {
            ext::ftCh_Init_801592D4(gobj);
        } else {
            ftCh_Init_80158F34(gobj);
        }
    }
}

pub fn ftCh_FingerGun2_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_FingerGun2_Anim");
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        ftCh_FingerGun2_Anim_inline(gobj);
    }
}

pub fn ftCh_FingerGun2_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_FingerGun2_Phys");
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        let da = (*(*fp).ft_data).ext_attr as *const ftCrazyHand_DatAttrs;
        ftCh_UpdateBossMotion(gobj, fp, da);
    }
}

pub fn ftCh_FingerGun2_Coll(_gobj: *mut HSD_GObj) { melee_core::ct!("ftCh_FingerGun2_Coll");}
