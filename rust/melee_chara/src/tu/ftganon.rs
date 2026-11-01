//! src/melee/ft/kinds/ftGanon/ftganon.c: Ganondorf's init callbacks (most forward to Captain
//! Falcon's). No float arithmetic, so no fused sites (tools/rust-port/fma_sites.py).
//! Smoke-tested by tests/tu_ftganon.rs.
//!
//! Data: file-name strings below; `ftGn_Init_MotionStateTable` (23 rows: six empty sword-swing
//! rows, then callbacks into the ftCaptain special TUs) belongs to the motion-state dispatcher
//! and is TODO until those callbacks are all native. `ftGn_CostumeList` is zero-initialised BSS.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use crate::stubs as st;
#[allow(unused_imports)]
use melee_types::{Fighter, HSD_GObj};
use melee_types::*;

/// Already ported (rounds 5/6): re-exported under this TU.


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

// ---- folded from chars (consolidation step 20/21) ----
/// `ftGn_Init_OnDeath` (src/melee/ft/kinds/ftGanon/ftganon.c).
pub fn ftGn_Init_OnDeath(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftGn_Init_OnDeath");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        st::ftParts_80074A4C(gobj, 0, 0);
        st::ftParts_80074A4C(gobj, 1, -1);
        (*fp).u.gn.during_specials = (0_u32 /* false */);
        (*fp).u.gn.during_specials_start = (0_u32 /* false */);
    }
}

/// `ftGn_Init_OnItemInvisible` (src/melee/ft/kinds/ftGanon/ftganon.c).
pub fn ftGn_Init_OnItemInvisible(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftGn_Init_OnItemInvisible");
    unsafe {
        st::Fighter_OnItemInvisible(gobj, 1 /* true */);
    }
}

/// `ftGn_Init_OnItemVisible` (src/melee/ft/kinds/ftGanon/ftganon.c).
pub fn ftGn_Init_OnItemVisible(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftGn_Init_OnItemVisible");
    unsafe {
        st::Fighter_OnItemVisible(gobj, 1 /* true */);
    }
}

/// `ftGn_Init_OnKnockbackEnter` (src/melee/ft/kinds/ftGanon/ftganon.c).
pub fn ftGn_Init_OnKnockbackEnter(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftGn_Init_OnKnockbackEnter");
    unsafe {
        st::ftAnim_ApplyPartAnim(gobj, 3, 3, 0.0_f32);
        st::ftAnim_ApplyPartAnim(gobj, 4, 3, 0.0_f32);
    }
}

/// `ftGn_Init_OnKnockbackExit` (src/melee/ft/kinds/ftGanon/ftganon.c).
pub fn ftGn_Init_OnKnockbackExit(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftGn_Init_OnKnockbackExit");
    unsafe {
        st::ftAnim_ApplyPartAnim(gobj, 3, 2, 0.0_f32);
        st::ftAnim_ApplyPartAnim(gobj, 4, 2, 0.0_f32);
    }
}
