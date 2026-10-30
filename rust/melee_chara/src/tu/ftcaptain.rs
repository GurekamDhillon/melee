//! src/melee/ft/kinds/ftCaptain/ftcaptain.c: Captain Falcon's init callbacks (shared with
//! Ganondorf through `ftCa_Init_OnLoadForGanon`). No float arithmetic, so no fused sites.
//! Smoke-tested by tests/tu_ftcaptain.rs.
//!
//! Data: file-name strings below; `ftCa_Init_MotionStateTable` (23 rows of callbacks into the
//! ftCaptain special TUs and ftCommon item swings) belongs to the motion-state dispatcher and is
//! TODO until those callbacks are all native. `ftCa_CostumeList` is zero-initialised BSS.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use melee_types::*;

/// Already ported (rounds 5/6): re-exported under this TU.
pub use crate::chars::r5::ftCaptain::ftcaptain::ftCa_Init_OnDeath;
pub use crate::chars::r6::ftCaptain::ftcaptain::{
    ftCa_Init_800E28C8, ftCa_Init_OnItemInvisible, ftCa_Init_OnItemVisible,
};
use crate::tu::ftclink::{Fighter_OnItemDrop, Fighter_OnItemPickup};

pub const ftCa_Init_DatFilename: &str = "PlCa.dat";
pub const ftCa_Init_DataName: &str = "ftDataCaptain";
pub const ftCa_Init_AnimDatFilename: &str = "PlCaAJ.dat";
/// `re_dat`: trailing dot intentional (lbFileGetFullName appends "usd" / "dat").
pub const re_dat: &str = "PlCaRe.";
/// `ftCa_Init_DemoMotionFilenames` (result, intro, ending, vi-wait).
pub const ftCa_Init_DemoMotionFilenames: [&str; 4] = [
    "ftDemoResultMotionFileCaptain",
    "ftDemoIntroMotionFileCaptain",
    "ftDemoEndingMotionFileCaptain",
    "ftDemoViWaitMotionFileCaptain",
];
/// `ftCa_Init_CostumeStrings`: (dat, joint) per costume; no matanim joint.
pub const ftCa_Init_CostumeStrings: [[&str; 2]; 6] = [
    ["PlCaNr.dat", "PlyCaptain5K_Share_joint"],
    ["PlCaGy.dat", "PlyCaptain5KGy_Share_joint"],
    [re_dat, "PlyCaptain5KRe_Share_joint"],
    ["PlCaWh.dat", "PlyCaptain5KWh_Share_joint"],
    ["PlCaGr.dat", "PlyCaptain5KGr_Share_joint"],
    ["PlCaBu.dat", "PlyCaptain5KBu_Share_joint"],
];

#[inline(always)]
unsafe fn fpp(gobj: *mut HSD_GObj) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}

pub fn ftCa_Init_OnItemPickup(gobj: *mut HSD_GObj, arg1: i32) {
    melee_core::ct!("ftCa_Init_OnItemPickup");
    Fighter_OnItemPickup(gobj, arg1, 1, 1);
}

/// Used for both OnItemRelease and OnUnknownItemRelated.
pub fn ftCa_Init_OnItemDrop(gobj: *mut HSD_GObj, bool1: i32) {
    melee_core::ct!("ftCa_Init_OnItemDrop");
    Fighter_OnItemDrop(gobj, bool1, 1, 1);
}

/// `PUSH_ATTRS(fp, ftCaptain_DatAttrs)`: copy ext_attr into the backup, point dat_attrs at it.
pub fn ftCa_Init_OnLoadForGanon(fp: *mut Fighter) {
    melee_core::ct!("ftCa_Init_OnLoadForGanon");
    unsafe {
        let backup = (*fp).dat_attrs_backup;
        let src = (*(*fp).ft_data).ext_attr as *const ftCaptain_DatAttrs;
        *((*fp).dat_attrs_backup as *mut ftCaptain_DatAttrs) = *src;
        (*fp).dat_attrs = backup;
    }
}

pub fn ftCa_Init_OnLoad(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCa_Init_OnLoad");
    unsafe {
        let fp = fpp(gobj);
        (*fp).set_can_walljump(1);
        ftCa_Init_OnLoadForGanon(fp);
    }
}

/// `COPY_ATTRS(gobj, ftCaptain_DatAttrs)`.
pub fn ftCa_Init_LoadSpecialAttrs(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCa_Init_LoadSpecialAttrs");
    unsafe {
        let fp = fpp(gobj);
        *((*fp).dat_attrs as *mut ftCaptain_DatAttrs) = *((*(*fp).ft_data).ext_attr as *const ftCaptain_DatAttrs);
    }
}
