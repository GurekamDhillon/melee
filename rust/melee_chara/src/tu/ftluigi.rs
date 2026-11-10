//! src/melee/ft/kinds/ftLuigi/ftluigi.c: Luigi's init callbacks. No float arithmetic, so no
//! fused sites (tools/rust-port/fma_sites.py). Smoke-tested by tests/tu_ftluigi.rs.
//!
//! Data: the file-name strings are below; `ftLg_Init_MotionStateTable` (18 rows of
//! Anim/IASA/Phys/Coll pointers into the ftLuigi special TUs and ftCommon) belongs to the
//! motion-state dispatcher and is TODO until those callbacks are all native.
//! `ftLg_CostumeList` is zero-initialised BSS.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use crate::stubs as st;
#[allow(unused_imports)]
use melee_types::{Fighter, HSD_GObj};
use crate::tu::ftclink::{Fighter_OnItemDrop, Fighter_OnItemPickup};
use melee_types::*;

/// Already ported (rounds 5/6 and the hand-written `chars::ftLuigi`): re-exported under this TU.




pub const ftLg_Init_DatFilename: &str = "PlLg.dat";
pub const ftLg_Init_DataName: &str = "ftDataLuigi";
pub const ftLg_Init_AnimDatFilename: &str = "PlLgAJ.dat";
/// `ftLg_Init_DemoMotionFilenames` (result, intro, ending, vi-wait).
pub const ftLg_Init_DemoMotionFilenames: [&str; 4] = [
    "ftDemoResultMotionFileLuigi",
    "ftDemoIntroMotionFileLuigi",
    "ftDemoEndingMotionFileLuigi",
    "ftDemoViWaitMotionFileLuigi",
];
/// `ftLg_Init_803D0AAC`: the two Vi demo motion file names (kinds 14 and 15).
pub const ftLg_Init_803D0AAC: [&str; 2] = ["ftDemoVi0102MotionFileLuigi", "ftDemoVi1101MotionFileLuigi"];
/// `ftLg_Init_CostumeStrings`: (dat, joint, matanim joint) per costume.
pub const ftLg_Init_CostumeStrings: [[&str; 3]; 4] = [
    ["PlLgNr.dat", "PlyLuigi5K_Share_joint", "PlyLuigi5K_Share_matanim_joint"],
    ["PlLgWh.dat", "PlyLuigi5KWh_Share_joint", "PlyLuigi5KWh_Share_matanim_joint"],
    ["PlLgAq.dat", "PlyLuigi5KAq_Share_joint", "PlyLuigi5KAq_Share_matanim_joint"],
    ["PlLgPi.dat", "PlyLuigi5KPi_Share_joint", "PlyLuigi5KPi_Share_matanim_joint"],
];

/// Calls into TUs that are not ported yet. Each is a settable hook; unset it panics.
pub mod ext {
    /// `it_8026B3F8(article, kind)` (it/it_26B1.c): shared with ftlink / ftclink.
    pub use crate::tu::ftclink::ext::{it_8026B3F8, IT_8026B3F8};
}

#[inline(always)]
unsafe fn fpp(gobj: *mut HSD_GObj) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}

pub fn ftLg_Init_OnLoad(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLg_Init_OnLoad");
    unsafe {
        let fp = fpp(gobj);
        let item_list = (*(*fp).ft_data).x48_items;
        // PUSH_ATTRS(fp, ftLuigiAttributes)
        let backup = (*fp).dat_attrs_backup;
        let src = (*(*fp).ft_data).ext_attr as *const ftLuigiAttributes;
        *((*fp).dat_attrs_backup as *mut ftLuigiAttributes) = *src;
        (*fp).dat_attrs = backup;
        // TARGET_PC: read-before-write of x222C_cycloneCharge (HSD_ObjAlloc does not clear it).
        (*fp).u.lg.x222C_cycloneCharge = 0;
        ext::it_8026B3F8((*item_list.add(0)).article as *mut ::core::ffi::c_void, It_Kind_Luigi_Fire as i32);
    }
}

pub fn ftLg_Init_OnItemPickup(gobj: *mut HSD_GObj, flag: i32) {
    melee_core::ct!("ftLg_Init_OnItemPickup");
    Fighter_OnItemPickup(gobj, flag, 1, 1);
}

pub fn ftLg_Init_OnItemDrop(gobj: *mut HSD_GObj, flag: i32) {
    melee_core::ct!("ftLg_Init_OnItemDrop");
    Fighter_OnItemDrop(gobj, flag, 1, 1);
}

/// `COPY_ATTRS(gobj, ftLuigiAttributes)`.
pub fn ftLg_Init_LoadSpecialAttrs(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLg_Init_LoadSpecialAttrs");
    unsafe {
        let fp = fpp(gobj);
        *((*fp).dat_attrs as *mut ftLuigiAttributes) = *((*(*fp).ft_data).ext_attr as *const ftLuigiAttributes);
    }
}

/// `ftLg_Init_GetMotionFileString(arg0)`: the Vi demo file for kind 9 / 10. The C leaves `offset`
/// uninitialised for any other value (undefined behaviour), so those return `None`.
pub fn ftLg_Init_GetMotionFileString(arg0: i32) -> Option<&'static str> {
    melee_core::ct!("ftLg_Init_GetMotionFileString");
    let offset = match arg0 {
        9 => 14,
        10 => 15,
        _ => return None,
    };
    Some(ftLg_Init_803D0AAC[offset - 14])
}

/// `ftLg_CostumeList`: zero-initialised BSS array of the C (`UnkCostumeStruct ftLg_CostumeList[4]`), filled when the costumes load.
pub static mut ftLg_CostumeList: [melee_types::UnkCostumeStruct; 4] = unsafe { core::mem::zeroed() };

// ---- folded from chars (consolidation step 20/21) ----
pub fn ftLg_Init_UnkDemoCallbacks0(arg0: i32, arg1: &mut i32, arg2: &mut i32) {
    melee_core::ct!("ftLg_Init_UnkDemoCallbacks0");
    if arg0 != 10 {
        if arg0 < 10 && arg0 >= 9 {
            *arg2 = 14;
            *arg1 = 14;
        }
    } else {
        *arg2 = 15;
        *arg1 = 15;
    }
}

/// `ftLg_Init_OnDeath` (src/melee/ft/kinds/ftLuigi/ftluigi.c).
pub fn ftLg_Init_OnDeath(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLg_Init_OnDeath");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        st::ftParts_80074A4C(gobj, 0, 0 /* false */);
        (*fp).u.lg.x2234 = 0_u32;
    }
}

/// `ftLg_Init_OnKnockbackEnter` (src/melee/ft/kinds/ftLuigi/ftluigi.c).
pub fn ftLg_Init_OnKnockbackEnter(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLg_Init_OnKnockbackEnter");
    unsafe {
        st::Fighter_OnKnockbackEnter(gobj, 1);
    }
}

/// `ftLg_Init_OnKnockbackExit` (src/melee/ft/kinds/ftLuigi/ftluigi.c).
pub fn ftLg_Init_OnKnockbackExit(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLg_Init_OnKnockbackExit");
    unsafe {
        st::Fighter_OnKnockbackExit(gobj, 1);
    }
}

/// `ftLg_Init_OnItemInvisible` (src/melee/ft/kinds/ftLuigi/ftluigi.c).
pub fn ftLg_Init_OnItemInvisible(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLg_Init_OnItemInvisible");
    unsafe {
        st::Fighter_OnItemInvisible(gobj, 1);
    }
}

/// `ftLg_Init_OnItemVisible` (src/melee/ft/kinds/ftLuigi/ftluigi.c).
pub fn ftLg_Init_OnItemVisible(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLg_Init_OnItemVisible");
    unsafe {
        st::Fighter_OnItemVisible(gobj, 1);
    }
}
