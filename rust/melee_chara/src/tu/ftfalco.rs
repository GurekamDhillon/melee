//! src/melee/ft/kinds/ftFalco/ftfalco.c: Falco's init callbacks (most delegate to ftFox's).
//! No fused sites (tools/rust-port/fma_sites.py), no float math. Smoke-tested through recording
//! hooks by tests/tu_ftfalco.rs.
//!
//! Data: the file-name strings are below; `ftFc_Init_MotionStateTable` (rows of
//! Anim/IASA/Phys/Coll pointers into the ftFox TUs, from motion 341) belongs to the
//! motion-state dispatcher and is TODO until those callbacks are all native, and
//! `ftFc_CostumeList` is an uninitialised 4-entry BSS array filled at load.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use crate::stubs as st;
#[allow(unused_imports)]
use melee_types::{Fighter, HSD_GObj};
use melee_types::*;

/// Already ported (rounds 5/6): re-exported under this TU.



pub const ftFc_Init_DatFilename: &str = "PlFc.dat";
pub const ftFc_Init_DataName: &str = "ftDataFalco";
pub const ftFc_Init_AnimDatFilename: &str = "PlFcAJ.dat";
/// `ftFc_Init_DemoMotionFilenames` (result, intro, ending, vi-wait).
pub const ftFc_Init_DemoMotionFilenames: [&str; 4] = [
    "ftDemoResultMotionFileFalco",
    "ftDemoIntroMotionFileFalco",
    "ftDemoEndingMotionFileFalco",
    "ftDemoViWaitMotionFileFalco",
];
/// `ftFc_Init_CostumeStrings`: (dat, joint, matanim joint) per costume.
pub const ftFc_Init_CostumeStrings: [[&str; 3]; 4] = [
    ["PlFcNr.dat", "PlyFalco5K_Share_joint", "PlyFalco5K_Share_matanim_joint"],
    ["PlFcRe.dat", "PlyFalco5KRe_Share_joint", "PlyFalco5KRe_Share_matanim_joint"],
    ["PlFcBu.dat", "PlyFalco5KBu_Share_joint", "PlyFalco5KBu_Share_matanim_joint"],
    ["PlFcGr.dat", "PlyFalco5KGr_Share_joint", "PlyFalco5KGr_Share_matanim_joint"],
];

/// Calls into TUs that are not ported yet: ftFox's (same C functions, same hooks).
pub mod ext {
    pub use crate::tu::ftfox::ext::{it_8026B3F8, IT_8026B3F8};
}

pub fn ftFc_Init_OnItemPickup(gobj: *mut HSD_GObj, flag: i32) {
    melee_core::ct!("ftFc_Init_OnItemPickup");
    crate::tu::ftclink::Fighter_OnItemPickup(gobj, flag, 1, 1);
}

pub fn ftFc_Init_OnItemDrop(gobj: *mut HSD_GObj, flag: i32) {
    melee_core::ct!("ftFc_Init_OnItemDrop");
    crate::tu::ftclink::Fighter_OnItemDrop(gobj, flag, 1, 1);
}

pub fn ftFc_Init_OnLoad(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftFc_Init_OnLoad");
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        let items = (*(*fp).ft_data).x48_items;
        (*fp).set_can_walljump(1);
        crate::tu::ftfox::ftFx_Init_OnLoadForFalco(fp);
        // `s32* sa2 = fp->dat_attrs`: the two item kinds at +0x1C / +0x20 of the Fox block.
        let sa2 = (*fp).dat_attrs as *const i32;
        ext::it_8026B3F8(*items.add(0), *sa2.add(7));
        ext::it_8026B3F8(*items.add(1), *sa2.add(8));
        ext::it_8026B3F8(*items.add(3), It_Kind_Falco_Phantasm as i32);
    }
}

pub fn ftFc_Init_LoadSpecialAttrs(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftFc_Init_LoadSpecialAttrs");
    crate::tu::ftfox::ftFx_Init_LoadSpecialAttrs(gobj);
}

/// `ftFc_CostumeList`: zero-initialised BSS array of the C (`UnkCostumeStruct ftFc_CostumeList[4]`), filled when the costumes load.
pub static mut ftFc_CostumeList: [melee_types::UnkCostumeStruct; 4] = unsafe { core::mem::zeroed() };

// ---- folded from chars (consolidation step 20/21) ----
/// `ftFc_Init_OnDeath` (src/melee/ft/kinds/ftFalco/ftfalco.c).
pub fn ftFc_Init_OnDeath(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftFc_Init_OnDeath");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        (*fp).u.fx.x222C_blasterGObj = core::mem::zeroed(); /* 0 */
        st::ftParts_80074A4C(gobj, 0, 0);
    }
}

/// `ftFc_Init_OnItemInvisible` (src/melee/ft/kinds/ftFalco/ftfalco.c).
pub fn ftFc_Init_OnItemInvisible(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftFc_Init_OnItemInvisible");
    unsafe {
        st::Fighter_OnItemInvisible(gobj, 1);
    }
}

/// `ftFc_Init_OnItemVisible` (src/melee/ft/kinds/ftFalco/ftfalco.c).
pub fn ftFc_Init_OnItemVisible(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftFc_Init_OnItemVisible");
    unsafe {
        st::Fighter_OnItemVisible(gobj, 1);
    }
}

/// `ftFc_Init_OnKnockbackEnter` (src/melee/ft/kinds/ftFalco/ftfalco.c).
pub fn ftFc_Init_OnKnockbackEnter(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftFc_Init_OnKnockbackEnter");
    unsafe {
        st::Fighter_OnKnockbackEnter(gobj, 1);
        st::ftAnim_ApplyPartAnim(gobj, 3, 3, 0.0_f32);
        st::ftAnim_ApplyPartAnim(gobj, 4, 3, 0.0_f32);
    }
}

/// `ftFc_Init_OnKnockbackExit` (src/melee/ft/kinds/ftFalco/ftfalco.c).
pub fn ftFc_Init_OnKnockbackExit(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftFc_Init_OnKnockbackExit");
    unsafe {
        st::Fighter_OnKnockbackExit(gobj, 1);
        st::ftAnim_ApplyPartAnim(gobj, 3, 2, 0.0_f32);
        st::ftAnim_ApplyPartAnim(gobj, 4, 2, 0.0_f32);
    }
}
