//! src/melee/ft/kinds/ftMasterHand/ftmasterhandcapturemasterhand.c. One float compare
//! (`grab_timer <= 0`) and no multiply-add shape, so no fused sites. Smoke-tested by
//! tests/tu_ftmasterhandcapturemasterhand.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use crate::stubs as st;
use melee_types::*;

pub use crate::tu::ftmasterhandcapturedamagemasterhand::ftMh_CaptureDamageMasterHand_80155C94;
pub use crate::tu::ftmasterhandbackdisappear::ftMh_MS_381_8015483C;

pub fn ftMh_CaptureMasterHand_Anim(_gobj: *mut HSD_GObj) { melee_core::ct!("ftMh_CaptureMasterHand_Anim");}

pub fn ftMh_CaptureMasterHand_IASA(_gobj: *mut HSD_GObj) { melee_core::ct!("ftMh_CaptureMasterHand_IASA");}

pub fn ftMh_CaptureMasterHand_Phys(_gobj: *mut HSD_GObj) { melee_core::ct!("ftMh_CaptureMasterHand_Phys");}

pub fn ftMh_CaptureMasterHand_Coll(_gobj: *mut HSD_GObj) { melee_core::ct!("ftMh_CaptureMasterHand_Coll");}

/// Native here (not the round-7 copy): `accessory1_cb` is the ported `ftCo_800DB464`, where the
/// generated `chars::cb` entry is still a panicking placeholder.
pub fn ftMh_CaptureMasterHand_80155B80(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_CaptureMasterHand_80155B80");
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        st::Fighter_ChangeMotionState(gobj, 328, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        (*fp).set_invisible(1);
        (*fp).set_x2220_b3(1);
        (*fp).accessory1_cb = Some(melee_fighter::tu::ftCo_CapturePulled::ftCo_800DB464_cb);
        st::ftCommon_8007E2F4(&mut *fp, 511);
        (*fp).set_x2220_b3(1);
        st::ftAnim_8006EBA4(gobj);
    }
}

pub fn ftMh_CaptureDamageMasterHand_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_CaptureDamageMasterHand_Anim");
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        melee_fighter::ftcommon2::grab_mash(&mut *fp, melee_fighter::hooks::common_data().x3A8);
        if (*fp).grab_timer <= 0.0 {
            ftMh_CaptureDamageMasterHand_80155C94(gobj);
            ftMh_MS_381_8015483C((*fp).victim_gobj);
        }
    }
}
