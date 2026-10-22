//! src/melee/ft/kinds/ftLuigi/ftluigi.c: Luigi's init callbacks. No float arithmetic, so no
//! fused sites (tools/rust-port/fma_sites.py). Smoke-tested by tests/tu_ftluigi.rs.
//!
//! Data: file-name strings below; `ftLg_Init_MotionStateTable` (18 rows of callbacks into the
//! ftLuigi special TUs) and `ftLg_Init_UnkMotionStates0` belong to the motion-state dispatcher
//! and are TODO until those callbacks are all native. `ftLg_CostumeList` is zero-initialised BSS.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use melee_types::*;

/// Already ported (rounds 5/6 and the pure helper): re-exported under this TU.
pub use crate::chars::ftLuigi::ftluigi::ftLg_Init_UnkDemoCallbacks0;
pub use crate::chars::r5::ftLuigi::ftluigi::{ftLg_Init_OnDeath, ftLg_Init_OnKnockbackEnter, ftLg_Init_OnKnockbackExit};
pub use crate::chars::r6::ftLuigi::ftluigi::{ftLg_Init_OnItemInvisible, ftLg_Init_OnItemVisible};
use crate::tu::ftclink::{Fighter_OnItemDrop, Fighter_OnItemPickup};

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
/// `ftLg_Init_803D0AAC`: the Vi demo motion files.
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
    use core::ffi::c_void;
    macro_rules! hook {
        ($hook:ident, $name:ident, ($($a:ident: $t:ty),*) $(-> $r:ty)?) => {
            pub static mut $hook: Option<fn($($t),*) $(-> $r)?> = None;
            #[allow(non_snake_case)]
            pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { $hook } {
                    Some(f) => f($($a),*),
                    None => panic!(concat!("TODO: ", stringify!($name), " is not ported")),
                }
            }
        };
    }
    hook!(IT_8026B3F8, it_8026B3F8, (article: *mut c_void, kind: i32));
}

#[inline(always)]
unsafe fn fpp(gobj: *mut HSD_GObj) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}

pub fn ftLg_Init_OnLoad(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let item_list = (*(*fp).ft_data).x48_items;
        // PUSH_ATTRS(fp, ftLuigiAttributes)
        let backup = (*fp).dat_attrs_backup;
        let src = (*(*fp).ft_data).ext_attr as *const ftLuigiAttributes;
        *((*fp).dat_attrs_backup as *mut ftLuigiAttributes) = *src;
        (*fp).dat_attrs = backup;
        // TARGET_PC: `x222C_cycloneCharge` is read before any write; the block is uncleared.
        (*fp).u.lg.x222C_cycloneCharge = 0;
        ext::it_8026B3F8(*item_list.add(0), It_Kind_Luigi_Fire);
    }
}

pub fn ftLg_Init_OnItemPickup(gobj: *mut HSD_GObj, flag: i32) {
    Fighter_OnItemPickup(gobj, flag, 1, 1);
}

pub fn ftLg_Init_OnItemDrop(gobj: *mut HSD_GObj, flag: i32) {
    Fighter_OnItemDrop(gobj, flag, 1, 1);
}

/// `COPY_ATTRS(gobj, ftLuigiAttributes)`.
pub fn ftLg_Init_LoadSpecialAttrs(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        *((*fp).dat_attrs as *mut ftLuigiAttributes) = *((*(*fp).ft_data).ext_attr as *const ftLuigiAttributes);
    }
}

/// `ftLg_Init_GetMotionFileString`: the Vi demo motion file for demo kind 9 / 10. The C leaves
/// `offset` uninitialised for any other argument; this returns the first entry then.
pub fn ftLg_Init_GetMotionFileString(arg0: u32) -> &'static str {
    let offset = match arg0 {
        9 => 14,
        10 => 15,
        _ => 14,
    };
    ftLg_Init_803D0AAC[offset - 14]
}
