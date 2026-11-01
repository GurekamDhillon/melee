//! src/melee/ft/kinds/ftZakoBoy/ftboy.c: the male Wireframe's init callbacks. No float arithmetic,
//! so no fused sites (tools/rust-port/fma_sites.py). State-oracle checked against the C
//! (tests/oracle_ftboy.rs, gen/oracle).
//!
//! Data: the file-name strings are below; `ftBo_CostumeList` is zero-initialised BSS.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use crate::stubs as st;
#[allow(unused_imports)]
use melee_types::{Fighter, HSD_GObj};
use crate::tu::ftinl;
use melee_types::*;

/// Already ported (rounds 5/6): re-exported under this TU.



pub const ftBo_Init_DatFilename: &str = "PlBo.dat";
pub const ftBo_Init_DataName: &str = "ftDataBoy";
pub const ftBo_Init_803D3500: &str = "PlBoNr.dat";
pub const ftBo_Init_803D350C: &str = "PlyBoy_Share_joint";
pub const ftBo_Init_AnimDatFilename: &str = "PlBoAJ.dat";
/// `ftBo_Init_DemoStrings` (result, intro, ending, vi-wait): only the intro motion file is named.
pub const ftBo_Init_DemoStrings: [Option<&str>; 4] = [None, Some("ftDemoIntroMotionFileBoy"), None, None];
/// `ftBo_Init_CostumeStrings`: (dat, joint, matanim joint) per costume.
pub const ftBo_Init_CostumeStrings: [[Option<&str>; 3]; 1] =
    [[Some(ftBo_Init_803D3500), Some(ftBo_Init_803D350C), None]];

/// Calls into TUs that are not ported yet. Each is a settable hook; unset it panics.
pub mod ext {}

pub fn ftBo_Init_OnItemPickup(gobj: *mut HSD_GObj, flag: i32) {
    melee_core::ct!("ftBo_Init_OnItemPickup");
    ftinl::Fighter_OnItemPickup(gobj, flag, 0, 0);
}

pub fn ftBo_Init_OnItemDrop(gobj: *mut HSD_GObj, bool1: i32) {
    melee_core::ct!("ftBo_Init_OnItemDrop");
    ftinl::Fighter_OnItemDrop(gobj, bool1, 0, 0);
}

pub fn ftBo_Init_OnLoad(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftBo_Init_OnLoad");
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        ftinl::push_attrs::<ftZakoboyAttributes>(fp);
    }
}

pub fn ftBo_Init_LoadSpecialAttrs(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftBo_Init_LoadSpecialAttrs");
    unsafe { ftinl::copy_attrs::<ftZakoboyAttributes>(gobj) }
}

/// `ftBo_CostumeList`: zero-initialised BSS array of the C (`UnkCostumeStruct ftBo_CostumeList[1]`), filled when the costumes load.
pub static mut ftBo_CostumeList: [melee_types::UnkCostumeStruct; 1] = unsafe { core::mem::zeroed() };

// ---- folded from chars (consolidation step 20/21) ----
/// `ftBo_Init_OnDeath` (src/melee/ft/kinds/ftZakoBoy/ftboy.c).
pub fn ftBo_Init_OnDeath(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftBo_Init_OnDeath");
    unsafe {
        st::ftParts_80074A4C(gobj, 0, 0);
    }
}

/// `ftBo_Init_OnItemInvisible` (src/melee/ft/kinds/ftZakoBoy/ftboy.c).
pub fn ftBo_Init_OnItemInvisible(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftBo_Init_OnItemInvisible");
    unsafe {
        st::Fighter_OnItemInvisible(gobj, 0);
    }
}

/// `ftBo_Init_OnItemVisible` (src/melee/ft/kinds/ftZakoBoy/ftboy.c).
pub fn ftBo_Init_OnItemVisible(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftBo_Init_OnItemVisible");
    unsafe {
        st::Fighter_OnItemVisible(gobj, 0);
    }
}
