//! src/melee/ft/kinds/ftZakoBoy/ftboy.c: the male Wireframe's init callbacks. No float arithmetic,
//! so no fused sites (tools/rust-port/fma_sites.py). State-oracle checked against the C
//! (tests/oracle_ftboy.rs, gen/oracle).
//!
//! Data: the file-name strings are below; `ftBo_CostumeList` is zero-initialised BSS.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use crate::tu::ftinl;
use melee_types::*;

/// Already ported (rounds 5/6): re-exported under this TU.
pub use crate::chars::r5::ftZakoBoy::ftboy::ftBo_Init_OnDeath;
pub use crate::chars::r6::ftZakoBoy::ftboy::{ftBo_Init_OnItemInvisible, ftBo_Init_OnItemVisible};

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
    ftinl::Fighter_OnItemPickup(gobj, flag, 0, 0);
}

pub fn ftBo_Init_OnItemDrop(gobj: *mut HSD_GObj, bool1: i32) {
    ftinl::Fighter_OnItemDrop(gobj, bool1, 0, 0);
}

pub fn ftBo_Init_OnLoad(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        ftinl::push_attrs::<ftZakoboyAttributes>(fp);
    }
}

pub fn ftBo_Init_LoadSpecialAttrs(gobj: *mut HSD_GObj) {
    unsafe { ftinl::copy_attrs::<ftZakoboyAttributes>(gobj) }
}
