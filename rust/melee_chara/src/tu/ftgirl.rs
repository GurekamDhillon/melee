//! src/melee/ft/kinds/ftZakoGirl/ftgirl.c: the female Wireframe's init callbacks. No float
//! arithmetic, so no fused sites (tools/rust-port/fma_sites.py). State-oracle checked against the C
//! (tests/oracle_ftgirl.rs, gen/oracle).
//!
//! Data: the file-name strings are below; `ftGl_CostumeList` is zero-initialised BSS.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use crate::stubs as st;
#[allow(unused_imports)]
use melee_types::{Fighter, HSD_GObj};
use crate::tu::ftinl;
use melee_types::*;

/// Already ported (rounds 5/6): re-exported under this TU.



pub const ftGl_Init_DatFilename: &str = "PlGl.dat";
pub const ftGl_Init_DataName: &str = "ftDataGirl";
pub const ftGl_Init_803D3580: &str = "PlGlNr.dat";
pub const ftGl_Init_803D358C: &str = "PlyGirl_Share_joint";
pub const ftGl_Init_AnimDatFilename: &str = "PlGlAJ.dat";
/// `ftGl_Init_DemoStrings` (result, intro, ending, vi-wait): only the intro motion file is named.
pub const ftGl_Init_DemoStrings: [Option<&str>; 4] = [None, Some("ftDemoIntroMotionFileGirl"), None, None];
/// `ftGl_Init_CostumeStrings`: (dat, joint, matanim joint) per costume.
pub const ftGl_Init_CostumeStrings: [[Option<&str>; 3]; 1] =
    [[Some(ftGl_Init_803D3580), Some(ftGl_Init_803D358C), None]];

/// Calls into TUs that are not ported yet. Each is a settable hook; unset it panics.
pub mod ext {}

pub fn ftGl_Init_OnLoad(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftGl_Init_OnLoad");
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        ftinl::push_attrs::<i32>(fp);
    }
}

pub fn ftGl_Init_OnItemPickup(gobj: *mut HSD_GObj, flag: i32) {
    melee_core::ct!("ftGl_Init_OnItemPickup");
    ftinl::Fighter_OnItemPickup(gobj, flag, 0, 0);
}

pub fn ftGl_Init_OnItemDrop(gobj: *mut HSD_GObj, flag: i32) {
    melee_core::ct!("ftGl_Init_OnItemDrop");
    ftinl::Fighter_OnItemDrop(gobj, flag, 0, 0);
}

pub fn ftGl_Init_LoadSpecialAttrs(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftGl_Init_LoadSpecialAttrs");
    unsafe { ftinl::copy_attrs::<i32>(gobj) }
}

/// `ftGl_CostumeList`: zero-initialised BSS array of the C (`UnkCostumeStruct ftGl_CostumeList[1]`), filled when the costumes load.
pub static mut ftGl_CostumeList: [melee_types::UnkCostumeStruct; 1] = unsafe { core::mem::zeroed() };

// ---- folded from chars (consolidation step 20/21) ----
/// `ftGl_Init_OnDeath` (src/melee/ft/kinds/ftZakoGirl/ftgirl.c).
pub fn ftGl_Init_OnDeath(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftGl_Init_OnDeath");
    unsafe {
        st::ftParts_80074A4C(gobj, 0, 0);
    }
}

/// `ftGl_Init_OnItemInvisible` (src/melee/ft/kinds/ftZakoGirl/ftgirl.c).
pub fn ftGl_Init_OnItemInvisible(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftGl_Init_OnItemInvisible");
    unsafe {
        st::Fighter_OnItemInvisible(gobj, 0);
    }
}

/// `ftGl_Init_OnItemVisible` (src/melee/ft/kinds/ftZakoGirl/ftgirl.c).
pub fn ftGl_Init_OnItemVisible(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftGl_Init_OnItemVisible");
    unsafe {
        st::Fighter_OnItemVisible(gobj, 0);
    }
}
