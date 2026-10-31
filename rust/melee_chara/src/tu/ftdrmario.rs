//! src/melee/ft/kinds/ftDrMario/ftdrmario.c: Dr. Mario's init callbacks and pill helpers.
//! No fused sites (tools/rust-port/fma_sites.py), no float math. Smoke-tested through recording
//! hooks by tests/tu_ftdrmario.rs.
//!
//! Data: the file-name strings are below; `ftDr_Init_MotionStateTable` (10 rows of
//! Anim/IASA/Phys/Coll pointers into ftMario/ftCommon TUs, from motion 341) belongs to the
//! motion-state dispatcher and is TODO until those callbacks are all native, and
//! `ftDr_CostumeList` is an uninitialised 5-entry BSS array filled at load.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use melee_types::*;

/// Already ported (rounds 5/6): re-exported under this TU.
pub use crate::chars::r5::ftDrMario::ftdrmario::{
    ftDr_Init_OnDeath, ftDr_Init_OnKnockbackEnter, ftDr_Init_OnKnockbackExit,
};
pub use crate::chars::r6::ftDrMario::ftdrmario::{ftDr_Init_OnItemInvisible, ftDr_Init_OnItemVisible};

pub const ftDr_Init_DatFilename: &str = "PlDr.dat";
pub const ftDr_Init_DataName: &str = "ftDataDrmario";
pub const ftDr_Init_AnimDatFilename: &str = "PlDrAJ.dat";
/// `ftDr_Init_DemoMotionFilenames` (result, intro, ending, vi-wait).
pub const ftDr_Init_DemoMotionFilenames: [&str; 4] = [
    "ftDemoResultMotionFileDrmario",
    "ftDemoIntroMotionFileDrmario",
    "ftDemoEndingMotionFileDrmario",
    "ftDemoViWaitMotionFileDrmario",
];
/// `ftDr_Init_CostumeStrings`: (dat, joint, matanim joint) per costume.
pub const ftDr_Init_CostumeStrings: [[&str; 3]; 5] = [
    ["PlDrNr.dat", "PlyDrmario5K_Share_joint", "PlyDrmario5K_Share_matanim_joint"],
    ["PlDrRe.dat", "PlyDrmario5KRe_Share_joint", "PlyDrmario5KRe_Share_matanim_joint"],
    ["PlDrBu.dat", "PlyDrmario5KBu_Share_joint", "PlyDrmario5KBu_Share_matanim_joint"],
    ["PlDrGr.dat", "PlyDrmario5KGr_Share_joint", "PlyDrmario5KGr_Share_matanim_joint"],
    ["PlDrBk.dat", "PlyDrmario5KBk_Share_joint", "PlyDrmario5KBk_Share_matanim_joint"],
];

/// Calls into TUs that are not ported yet. Each is a settable hook; unset it panics.
pub mod ext {
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
    pub static mut FTMR_INIT_ONLOADFORDRMARIO: Option<fn(*mut Fighter) > = None;
            #[allow(non_snake_case)]
            pub fn ftMr_Init_OnLoadForDrMario(fp: *mut Fighter)  {
                match unsafe { FTMR_INIT_ONLOADFORDRMARIO } {
                    Some(f) => f(fp),
                    None => { unsafe { let _ = crate::tu::ftmario::ftMr_Init_OnLoadForDrMario(fp as _); } return; },
                }
            }
    pub static mut FTMR_INIT_LOADSPECIALATTRS: Option<fn(*mut HSD_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn ftMr_Init_LoadSpecialAttrs(gobj: *mut HSD_GObj)  {
                match unsafe { FTMR_INIT_LOADSPECIALATTRS } {
                    Some(f) => f(gobj),
                    None => { unsafe { let _ = crate::tu::ftmario::ftMr_Init_LoadSpecialAttrs(gobj as _); } return; },
                }
            }
    pub static mut IT_8026B3F8: Option<fn(*mut core::ffi::c_void, i32) > = None;
            #[allow(non_snake_case)]
            pub fn it_8026B3F8(article: *mut core::ffi::c_void, kind: i32)  {
                match unsafe { IT_8026B3F8 } {
                    Some(f) => f(article, kind),
                    None => { unsafe { let _ = melee_items::tu::it_26B1::it_8026B3F8(article as _, kind as _); } return; },
                }
            }
    crate::port_hook!(ITDRMARIOPILL_802C0DBC, itDrMarioPill_802C0DBC, crate::adapt::itDrMarioPill_802C0DBC, (item: *mut Item_GObj));
    hook_or!(FTCO_800DEAE8, ftCo_800DEAE8, melee_fighter::tu::ftCo_AppealS::ftCo_800DEAE8,
             (gobj: *mut HSD_GObj, msid0: i32, msid1: i32));
}

#[inline(always)]
unsafe fn fpp(gobj: *mut HSD_GObj) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}

/// `ftDr_Init_80149540` as a Fighter callback entry (`take_dmg_cb` / `death2_cb`).
pub unsafe extern "C" fn cb_ftDr_Init_80149540(g: *mut HSD_GObj) {
    melee_core::ct!("cb_ftDr_Init_80149540");
    ftDr_Init_80149540(g);
}

pub fn ftDr_Init_OnLoad(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftDr_Init_OnLoad");
    unsafe {
        let fp = fpp(gobj);
        let items = (*(*fp).ft_data).x48_items;
        ext::ftMr_Init_OnLoadForDrMario(fp);
        let sa = (*fp).dat_attrs as *mut ftDrMarioAttributes;
        ext::it_8026B3F8(*items.add(1), It_Kind_DrMario_Vitamin as i32);
        ext::it_8026B3F8(*items.add(3), (*sa).x14 as i32);
    }
}

pub fn ftDr_Init_80149540(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftDr_Init_80149540");
    ftDr_Init_801497CC(gobj);
}

pub fn ftDr_Init_OnItemPickup(gobj: *mut HSD_GObj, flag: i32) {
    melee_core::ct!("ftDr_Init_OnItemPickup");
    crate::tu::ftclink::Fighter_OnItemPickup(gobj, flag, 1, 1);
}

pub fn ftDr_Init_OnItemDrop(gobj: *mut HSD_GObj, bool1: i32) {
    melee_core::ct!("ftDr_Init_OnItemDrop");
    crate::tu::ftclink::Fighter_OnItemDrop(gobj, bool1, 1, 1);
}

pub fn ftDr_Init_LoadSpecialAttrs(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftDr_Init_LoadSpecialAttrs");
    ext::ftMr_Init_LoadSpecialAttrs(gobj);
}

/// Drop the pill pointer (destroying the pill item) and clear the damage / death callbacks.
pub fn ftDr_Init_801497CC(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftDr_Init_801497CC");
    unsafe {
        if !gobj.is_null() {
            let fp = fpp(gobj);
            if !fp.is_null() && !(*fp).u.mr.x2240.is_null() {
                ext::itDrMarioPill_802C0DBC((*fp).u.mr.x2240);
                (*fp).u.mr.x2240 = core::ptr::null_mut();
            }
        }
        if !gobj.is_null() {
            let fp = fpp(gobj);
            if !fp.is_null() {
                (*fp).take_dmg_cb = None;
                (*fp).death2_cb = None;
            }
        }
    }
}

/// C `bool`: true unless an Appeal (341 / 342) still holds its pill.
pub fn ftDr_Init_80149844(gobj: *mut HSD_GObj) -> bool {
    melee_core::ct!("ftDr_Init_80149844");
    if gobj.is_null() {
        return true;
    }
    unsafe {
        let fp = fpp(gobj);
        if fp.is_null() {
            return true;
        }
        let tmp = (*fp).motion_id as i32;
        if tmp != 0x155 && tmp != 0x156 {
            return true;
        }
        if (*fp).u.mr.x2240.is_null() {
            return true;
        }
    }
    false
}

/// Forget the pill (it was handed on) and clear the damage / death callbacks.
pub fn ftDr_Init_801498A0(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftDr_Init_801498A0");
    unsafe {
        if !gobj.is_null() {
            let fp = fpp(gobj);
            if !fp.is_null() && !(*fp).u.mr.x2240.is_null() {
                (*fp).u.mr.x2240 = core::ptr::null_mut();
            }
        }
        if !gobj.is_null() {
            let fp = fpp(gobj);
            if !fp.is_null() {
                (*fp).take_dmg_cb = None;
                (*fp).death2_cb = None;
            }
        }
    }
}

/// C `u32`: `cmd_vars[1]`, or 0 without a fighter.
pub fn ftDr_Init_801498EC(gobj: *mut HSD_GObj) -> u32 {
    melee_core::ct!("ftDr_Init_801498EC");
    if !gobj.is_null() {
        unsafe {
            let fp = fpp(gobj);
            if !fp.is_null() {
                return (*fp).cmd_vars[1] as u32;
            }
        }
    }
    0
}

pub fn ftDr_Init_80149910(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftDr_Init_80149910");
    unsafe {
        let fp = fpp(gobj);
        ext::ftCo_800DEAE8(gobj, ftMr_MS_AppealSR as i32, ftMr_MS_AppealSL as i32);
        (*fp).cmd_vars[0] = 1;
        (*fp).cmd_vars[1] = 0;
    }
}

/// `ftDr_CostumeList`: zero-initialised BSS array of the C (`UnkCostumeStruct ftDr_CostumeList[5]`), filled when the costumes load.
pub static mut ftDr_CostumeList: [melee_types::UnkCostumeStruct; 5] = unsafe { core::mem::zeroed() };
