//! src/melee/ft/kinds/ftGanon/ftganon.c: Ganondorf's init callbacks (most forward to Captain
//! Falcon's). No float arithmetic, so no fused sites (tools/rust-port/fma_sites.py).
//! Smoke-tested by tests/tu_ftganon.rs.
//!
//! Data: file-name strings below; `ftGn_Init_MotionStateTable` (23 rows: six empty sword-swing
//! rows, then callbacks into the ftCaptain special TUs) belongs to the motion-state dispatcher
//! and is TODO until those callbacks are all native. `ftGn_CostumeList` is zero-initialised BSS.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use melee_types::*;

/// Already ported (rounds 5/6): re-exported under this TU.
pub use crate::chars::r5::ftGanon::ftganon::ftGn_Init_OnDeath;
pub use crate::chars::r6::ftGanon::ftganon::{
    ftGn_Init_OnItemInvisible, ftGn_Init_OnItemVisible, ftGn_Init_OnKnockbackEnter, ftGn_Init_OnKnockbackExit,
};
use crate::tu::ftclink::{Fighter_OnItemDrop, Fighter_OnItemPickup};

pub const ftGn_Init_DatFilename: &str = "PlGn.dat";
pub const ftGn_Init_DataName: &str = "ftDataGanon";
pub const ftGn_Init_AnimDatFilename: &str = "PlGnAJ.dat";
/// `ftGn_Init_DemoMotionFilenames` (result, intro, ending, vi-wait).
pub const ftGn_Init_DemoMotionFilenames: [&str; 4] = [
    "ftDemoResultMotionFileGanon",
    "ftDemoIntroMotionFileGanon",
    "ftDemoEndingMotionFileGanon",
    "ftDemoViWaitMotionFileGanon",
];
/// `ftGn_Init_CostumeStrings`: (dat, joint) per costume; no matanim joint.
pub const ftGn_Init_CostumeStrings: [[&str; 2]; 5] = [
    ["PlGnNr.dat", "PlyGanon5K_Share_joint"],
    ["PlGnRe.dat", "PlyGanon5KRe_Share_joint"],
    ["PlGnBu.dat", "PlyGanon5KBu_Share_joint"],
    ["PlGnGr.dat", "PlyGanon5KGr_Share_joint"],
    ["PlGnLa.dat", "PlyGanon5KLa_Share_joint"],
];

pub fn ftGn_Init_OnItemPickup(gobj: *mut HSD_GObj, flag: i32) {
    melee_core::ct!("ftGn_Init_OnItemPickup");
    Fighter_OnItemPickup(gobj, flag, 1, 1);
}

pub fn ftGn_Init_OnItemDrop(gobj: *mut HSD_GObj, bool1: i32) {
    melee_core::ct!("ftGn_Init_OnItemDrop");
    Fighter_OnItemDrop(gobj, bool1, 1, 1);
}

/// Ganondorf shares Captain Falcon's attribute block layout.
pub fn ftGn_Init_OnLoad(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftGn_Init_OnLoad");
    unsafe {
        crate::tu::ftcaptain::ftCa_Init_OnLoadForGanon((*gobj).user_data as *mut Fighter);
    }
}

pub fn ftGn_Init_LoadSpecialAttrs(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftGn_Init_LoadSpecialAttrs");
    crate::tu::ftcaptain::ftCa_Init_LoadSpecialAttrs(gobj);
}

/// `ftGn_CostumeList`: zero-initialised BSS array of the C (`UnkCostumeStruct ftGn_CostumeList[5]`), filled when the costumes load.
pub static mut ftGn_CostumeList: [melee_types::UnkCostumeStruct; 5] = unsafe { core::mem::zeroed() };
