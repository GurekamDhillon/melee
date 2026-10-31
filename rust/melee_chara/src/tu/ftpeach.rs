//! src/melee/ft/kinds/ftPeach/ftpeach.c: Peach's init callbacks (load, death, item, knockback).
//! No fused sites (tools/rust-port/fma_sites.py), no float math. Smoke-tested through recording
//! hooks by tests/tu_ftpeach.rs.
//!
//! Data: the file-name strings are below; `ftPe_Init_MotionStateTable` (30 rows of
//! Anim/IASA/Phys/Coll pointers into ftPeach/ftCommon TUs, from motion 341) belongs to the
//! motion-state dispatcher and is TODO until those callbacks are all native (ftpeachspecialn.c
//! and ftpeachspecials.c are not ported yet), and `ftPe_CostumeList` is an uninitialised 5-entry
//! BSS array filled at load.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use crate::stubs as st;
use melee_types::*;

/// Already ported (rounds 5/6): re-exported under this TU.
pub use crate::chars::r5::ftPeach::ftpeach::{ftPe_Init_OnKnockbackEnter, ftPe_Init_OnKnockbackExit};
pub use crate::chars::r6::ftPeach::ftpeach::{ftPe_Init_OnItemInvisible, ftPe_Init_OnItemVisible};

pub const ftPe_Init_DatFilename: &str = "PlPe.dat";
pub const ftPe_Init_DataName: &str = "ftDataPeach";
pub const ftPe_Init_AnimDatFilename: &str = "PlPeAJ.dat";
/// `ftPe_Init_DemoMotionFilenames` (result, intro, ending, vi-wait).
pub const ftPe_Init_DemoMotionFilenames: [&str; 4] = [
    "ftDemoResultMotionFilePeach",
    "ftDemoIntroMotionFilePeach",
    "ftDemoEndingMotionFilePeach",
    "ftDemoViWaitMotionFilePeach",
];
/// `ftPe_Init_CostumeStrings`: (dat, joint, matanim joint) per costume.
pub const ftPe_Init_CostumeStrings: [[&str; 3]; 5] = [
    ["PlPeNr.dat", "PlyPeach5K_Share_joint", "PlyPeach5K_Share_matanim_joint"],
    ["PlPeYe.dat", "PlyPeach5KYe_Share_joint", "PlyPeach5KYe_Share_matanim_joint"],
    ["PlPeWh.dat", "PlyPeach5KWh_Share_joint", "PlyPeach5KWh_Share_matanim_joint"],
    ["PlPeBu.dat", "PlyPeach5KBu_Share_joint", "PlyPeach5KBu_Share_matanim_joint"],
    ["PlPeGr.dat", "PlyPeach5KGr_Share_joint", "PlyPeach5KGr_Share_matanim_joint"],
];

/// Calls into TUs that are not ported yet. Each is a settable hook; unset it panics.
pub mod ext {
    use core::ffi::c_void;
    use melee_types::*;
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
    macro_rules! hook_or {
        ($hook:ident, $name:ident, $port:path, ($($a:ident: $t:ty),*) $(-> $r:ty)?) => {
            pub static mut $hook: Option<fn($($t),*) $(-> $r)?> = None;
            #[allow(non_snake_case)]
            pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { $hook } {
                    Some(f) => f($($a),*),
                    None => $port($($a),*),
                }
            }
        };
    }
    crate::port_hook!(LBANIM_8001E8F8, lbAnim_8001E8F8, crate::adapt::lbAnim_8001E8F8, (tree: *mut FigaTree) -> f32);
    pub static mut FTDATA_80085E50: Option<fn(*mut Fighter, i32) -> *mut FigaTree> = None;
            #[allow(non_snake_case)]
            pub fn ftData_80085E50(fp: *mut Fighter, msid: i32) -> *mut FigaTree {
                match unsafe { FTDATA_80085E50 } {
                    Some(f) => f(fp, msid),
                    None => { return unsafe { melee_fighter::tu::ftdata::ftData_80085E50(fp as _, msid as _) as _ }; },
                }
            }
    pub static mut IT_8026B3F8: Option<fn(*mut c_void, i32) > = None;
            #[allow(non_snake_case)]
            pub fn it_8026B3F8(article: *mut c_void, kind: i32)  {
                match unsafe { IT_8026B3F8 } {
                    Some(f) => f(article, kind),
                    None => { unsafe { melee_items::tu::it_26B1::it_8026B3F8(article as _, kind as _); } return; },
                }
            }
    /// `ftPe_SpecialN_OnDeath2(gobj)` (ftpeachspecialn.c, not ported yet).
    pub static mut FTPE_SPECIALN_ONDEATH2: Option<fn(*mut HSD_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn ftPe_SpecialN_OnDeath2(gobj: *mut HSD_GObj)  {
                match unsafe { FTPE_SPECIALN_ONDEATH2 } {
                    Some(f) => f(gobj),
                    None => { unsafe { crate::tu::ftpeachspecialn::ftPe_SpecialN_OnDeath2(gobj as _); } return; },
                }
            }
    hook_or!(FTPARTS_80074A74, ftParts_80074A74, melee_fighter::ft0df::ftparts_80074a74, (gobj: *mut HSD_GObj, model_idx: i32) -> i32);
}

#[inline(always)]
unsafe fn fpp(gobj: *mut HSD_GObj) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}

pub fn ftPe_Init_OnDeath(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftPe_Init_OnDeath");
    unsafe {
        let fp = fpp(gobj);
        (*fp).u.pe.has_float = 1;
        (*fp).u.pe.attacks4_motion_id = ftCo_MS_None as _;
        (*fp).u.pe.specialairn_used = 0;
        (*fp).u.pe.parasol_gobj_1 = core::ptr::null_mut();
        (*fp).u.pe.parasol_gobj_0 = core::ptr::null_mut();
        (*fp).u.pe.toad_gobj = core::ptr::null_mut();
        (*fp).u.pe.veg_gobj = core::ptr::null_mut();
        st::ftParts_80074A4C(gobj, 0, 0);
        st::ftParts_80074A4C(gobj, 2, 0);
        st::ftParts_80074A4C(gobj, 3, -1);
        st::ftParts_80074A4C(gobj, 4, 0);

        match (*fp).x619_costume_id {
            1 => {
                st::ftParts_80074A4C(gobj, 1, -1);
                st::ftParts_80074A4C(gobj, 5, 0);
                st::ftParts_80074A4C(gobj, 6, -1);
            }
            _ => {
                st::ftParts_80074A4C(gobj, 1, 0);
                st::ftParts_80074A4C(gobj, 5, -1);
                st::ftParts_80074A4C(gobj, 6, 0);
            }
        }
    }
}

pub fn ftPe_Init_OnLoad(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftPe_Init_OnLoad");
    unsafe {
        let fp = fpp(gobj);
        let ext_attrs = (*(*fp).ft_data).ext_attr as *mut ftPe_DatAttrs;
        let items = (*(*fp).ft_data).x48_items;
        (*ext_attrs).floatfallf_anim_start = ext::lbAnim_8001E8F8(ext::ftData_80085E50(fp, 18));
        (*ext_attrs).floatfallb_anim_start = ext::lbAnim_8001E8F8(ext::ftData_80085E50(fp, 19));
        // PUSH_ATTRS(fp, ftPe_DatAttrs): copy ext_attr into the backup, point dat_attrs at it.
        let backup = (*fp).dat_attrs_backup;
        *((*fp).dat_attrs_backup as *mut ftPe_DatAttrs) = *ext_attrs;
        (*fp).dat_attrs = backup;
        ext::it_8026B3F8(*items.add(0), It_Kind_Peach_Explode);
        ext::it_8026B3F8(*items.add(1), It_Kind_Peach_Turnip);
        ext::it_8026B3F8(*items.add(2), It_Kind_Peach_Parasol);
        ext::it_8026B3F8(*items.add(3), It_Kind_Peach_Toad);
        ext::it_8026B3F8(*items.add(4), It_Kind_Peach_ToadSpore);
    }
}

/// `ftPe_Init_OnDeath2` as a Fighter callback entry (`death2_cb` / `death3_cb` / `take_dmg_cb`).
pub unsafe extern "C" fn cb_ftPe_Init_OnDeath2(g: *mut HSD_GObj) {
    melee_core::ct!("cb_ftPe_Init_OnDeath2");
    ftPe_Init_OnDeath2(g);
}

pub fn ftPe_Init_OnDeath2(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftPe_Init_OnDeath2");
    crate::tu::ftpeachspecialhi::ftPe_8011D598(gobj);
    ext::ftPe_SpecialN_OnDeath2(gobj);
    crate::tu::ftpeachspeciallw::ftPe_SpecialLw_8011CFA0(gobj);
}

pub fn ftPe_Init_OnItemPickup(gobj: *mut HSD_GObj, flag: i32) {
    melee_core::ct!("ftPe_Init_OnItemPickup");
    crate::tu::ftclink::Fighter_OnItemPickup(gobj, flag, 1, 1);
}

pub fn ftPe_Init_OnItemDrop(gobj: *mut HSD_GObj, flag: i32) {
    melee_core::ct!("ftPe_Init_OnItemDrop");
    crate::tu::ftclink::Fighter_OnItemDrop(gobj, flag, 1, 1);
}

/// `COPY_ATTRS(gobj, ftPe_DatAttrs)`.
pub fn ftPe_Init_LoadSpecialAttrs(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftPe_Init_LoadSpecialAttrs");
    unsafe {
        let fp = fpp(gobj);
        *((*fp).dat_attrs as *mut ftPe_DatAttrs) = *((*(*fp).ft_data).ext_attr as *const ftPe_DatAttrs);
    }
}

/// Toggle the visibility of model part 2 (parasol-or-hand part): hidden (`-1`) becomes shown (`0`).
pub fn ftPe_Init_8011B93C(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftPe_Init_8011B93C");
    let result = ext::ftParts_80074A74(gobj, 2);
    if result == -1 {
        st::ftParts_80074A4C(gobj, 2, 0);
    } else {
        st::ftParts_80074A4C(gobj, 2, -1);
    }
}

/// `ftPe_CostumeList`: zero-initialised BSS array of the C (`UnkCostumeStruct ftPe_CostumeList[5]`), filled when the costumes load.
pub static mut ftPe_CostumeList: [melee_types::UnkCostumeStruct; 5] = unsafe { core::mem::zeroed() };
