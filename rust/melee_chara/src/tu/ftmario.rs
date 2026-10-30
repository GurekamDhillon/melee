//! src/melee/ft/kinds/ftMario/ftmario.c: Mario / Dr. Mario init callbacks. No float arithmetic,
//! so no fused sites (tools/rust-port/fma_sites.py). Smoke-tested by tests/tu_ftmario.rs.
//!
//! Data: file-name strings below; `ftMr_Init_MotionStateTable` (10 rows of callbacks into the
//! ftMario special TUs) and `ftMr_Init_UnkMotionStates0` belong to the motion-state dispatcher
//! and are TODO until those callbacks are all native. `ftMr_CostumeList` is zero-initialised BSS.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use melee_types::*;

/// Already ported (rounds 5/6 and the pure helper): re-exported under this TU.
pub use crate::chars::ftMario::ftmario::ftMr_Init_UnkDemoCallbacks0;
pub use crate::chars::r5::ftMario::ftmario::{ftMr_Init_OnDeath, ftMr_Init_OnKnockbackEnter, ftMr_Init_OnKnockbackExit};
pub use crate::chars::r6::ftMario::ftmario::{ftMr_Init_OnItemInvisible, ftMr_Init_OnItemVisible};
use crate::tu::ftclink::{Fighter_OnItemDrop, Fighter_OnItemPickup};

pub const ftMr_Init_DatFilename: &str = "PlMr.dat";
pub const ftMr_Init_DataName: &str = "ftDataMario";
pub const ftMr_Init_AnimDatFilename: &str = "PlMrAJ.dat";
/// `ftMr_Init_DemoMotionFilenames` (result, intro, ending, vi-wait).
pub const ftMr_Init_DemoMotionFilenames: [&str; 4] = [
    "ftDemoResultMotionFileMario",
    "ftDemoIntroMotionFileMario",
    "ftDemoEndingMotionFileMario",
    "ftDemoViWaitMotionFileMario",
];
pub const ftMr_Init_DemoViMotionFilenames: [&str; 2] = ["ftDemoVi0102MotionFileMario", "ftDemoVi1101MotionFileMario"];
/// `ftMr_Init_CostumeStrings`: (dat, joint, matanim joint) per costume.
pub const ftMr_Init_CostumeStrings: [[&str; 3]; 5] = [
    ["PlMrNr.dat", "PlyMario5K_Share_joint", "PlyMario5K_Share_matanim_joint"],
    ["PlMrYe.dat", "PlyMario5KYe_Share_joint", "PlyMario5KYe_Share_matanim_joint"],
    ["PlMrBk.dat", "PlyMario5KBk_Share_joint", "PlyMario5KBk_Share_matanim_joint"],
    ["PlMrBu.dat", "PlyMario5KBu_Share_joint", "PlyMario5KBu_Share_matanim_joint"],
    ["PlMrGr.dat", "PlyMario5KGr_Share_joint", "PlyMario5KGr_Share_matanim_joint"],
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
    pub static mut IT_8026B3F8: Option<fn(*mut c_void, i32) > = None;
            #[allow(non_snake_case)]
            pub fn it_8026B3F8(article: *mut c_void, kind: i32)  {
                match unsafe { IT_8026B3F8 } {
                    Some(f) => f(article, kind),
                    None => { unsafe { let _ = melee_items::tu::it_26B1::it_8026B3F8(article as _, kind as _); } return; },
                }
            }
}

#[inline(always)]
unsafe fn fpp(gobj: *mut HSD_GObj) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}

/// `PUSH_ATTRS(fp, ftMario_DatAttrs)`: copy ext_attr into the backup, point dat_attrs at it.
pub fn ftMr_Init_OnLoadForDrMario(fp: *mut Fighter) {
    melee_core::ct!("ftMr_Init_OnLoadForDrMario");
    unsafe {
        let backup = (*fp).dat_attrs_backup;
        let src = (*(*fp).ft_data).ext_attr as *const ftMario_DatAttrs;
        *((*fp).dat_attrs_backup as *mut ftMario_DatAttrs) = *src;
        (*fp).dat_attrs = backup;
    }
}

pub fn ftMr_Init_OnLoad(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMr_Init_OnLoad");
    unsafe {
        let fp = fpp(gobj);
        let items = (*(*fp).ft_data).x48_items;
        (*fp).set_can_walljump(1);
        ftMr_Init_OnLoadForDrMario(fp);
        let sa = (*fp).dat_attrs as *const ftMario_DatAttrs;
        ext::it_8026B3F8(*items.add(0), It_Kind_Mario_Fire);
        ext::it_8026B3F8(*items.add(2), (*sa).specials.cape_kind);
    }
}

pub fn ftMr_Init_OnTakeDamage(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMr_Init_OnTakeDamage");
    crate::tu::ftmariospecials::ftMr_SpecialS_RemoveCape(gobj);
}

pub fn ftMr_Init_OnItemPickup(gobj: *mut HSD_GObj, flag: i32) {
    melee_core::ct!("ftMr_Init_OnItemPickup");
    Fighter_OnItemPickup(gobj, flag, 1, 1);
}

pub fn ftMr_Init_OnItemDrop(gobj: *mut HSD_GObj, flag: i32) {
    melee_core::ct!("ftMr_Init_OnItemDrop");
    Fighter_OnItemDrop(gobj, flag, 1, 1);
}

/// `COPY_ATTRS(gobj, ftMario_DatAttrs)`.
pub fn ftMr_Init_LoadSpecialAttrs(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMr_Init_LoadSpecialAttrs");
    unsafe {
        let fp = fpp(gobj);
        *((*fp).dat_attrs as *mut ftMario_DatAttrs) = *((*(*fp).ft_data).ext_attr as *const ftMario_DatAttrs);
    }
}

/// `ftMr_Init_GetMotionFileString`: the Vi demo motion file for demo kind 9 / 10. The C leaves
/// `offset` uninitialised for any other argument; this returns the first entry then.
pub fn ftMr_Init_GetMotionFileString(arg0: u32) -> &'static str {
    melee_core::ct!("ftMr_Init_GetMotionFileString");
    let offset = match arg0 {
        9 => 14,
        10 => 15,
        _ => 14,
    };
    ftMr_Init_DemoViMotionFilenames[offset - 14]
}
