//! src/melee/ft/kinds/ftPichu/ftpichu.c: Pichu init callbacks and file names. Reuses Pikachu's
//! attribute loading. No fused sites (tools/rust-port/fma_sites.py). Smoke-tested by
//! tests/tu_ftpichu.rs.
//!
//! Data: the file-name strings are below. `ftPc_CostumeList` is zero-initialised BSS filled at
//! load. (Pichu has no `MotionStateTable` of its own: it shares Pikachu's.)
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use crate::stubs as st;
use crate::tu::ftclink::{Fighter_OnItemDrop, Fighter_OnItemPickup};
use melee_types::*;

/// Already ported (rounds 5/6): re-exported under this TU.
pub use crate::chars::r5::ftPichu::ftpichu::{ftPc_Init_OnKnockbackEnter, ftPc_Init_OnKnockbackExit};
pub use crate::chars::r6::ftPichu::ftpichu::{ftPc_Init_OnItemInvisible, ftPc_Init_OnItemVisible};

pub const ftPc_Init_DatFilename: &str = "PlPc.dat";
pub const ftPc_Init_DataName: &str = "ftDataPichu";
pub const ftPc_Init_803D2200: &str = "PlPcNr.dat";
pub const ftPc_Init_803D220C: &str = "PlyPichu5K_Share_joint";
pub const ftPc_Init_803D2224: &str = "PlyPichu5K_Share_matanim_joint";
pub const ftPc_Init_803D2244: &str = "PlPcRe.dat";
pub const ftPc_Init_803D2250: &str = "PlyPichu5KRe_Share_joint";
pub const ftPc_Init_803D226C: &str = "PlyPichu5KRe_Share_matanim_joint";
pub const ftPc_Init_803D2290: &str = "PlPcBu.dat";
pub const ftPc_Init_803D229C: &str = "PlyPichu5KBu_Share_joint";
pub const ftPc_Init_803D22B8: &str = "PlyPichu5KBu_Share_matanim_joint";
pub const ftPc_Init_803D22DC: &str = "PlPcGr.dat";
pub const ftPc_Init_803D22E8: &str = "PlyPichu5KGr_Share_joint";
pub const ftPc_Init_803D2304: &str = "PlyPichu5KGr_Share_matanim_joint";
pub const ftPc_Init_AnimDatFilename: &str = "PlPcAJ.dat";

/// `ftPc_Init_DemoMotionFilenames` (result, intro, ending, vi-wait).
pub const ftPc_Init_DemoMotionFilenames: [&str; 4] = [
    "ftDemoResultMotionFilePichu",
    "ftDemoIntroMotionFilePichu",
    "ftDemoEndingMotionFilePichu",
    "ftDemoViWaitMotionFilePichu",
];

/// `ftPc_Init_CostumeStrings`: (dat, joint, matanim joint) per costume.
pub const ftPc_Init_CostumeStrings: [[&str; 3]; 4] = [
    [ftPc_Init_803D2200, ftPc_Init_803D220C, ftPc_Init_803D2224],
    [ftPc_Init_803D2244, ftPc_Init_803D2250, ftPc_Init_803D226C],
    [ftPc_Init_803D2290, ftPc_Init_803D229C, ftPc_Init_803D22B8],
    [ftPc_Init_803D22DC, ftPc_Init_803D22E8, ftPc_Init_803D2304],
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
                    None => melee_core::unported!(concat!("TODO: ", stringify!($name), " is not ported")),
                }
            }
        };
    }
    pub static mut IT_8026B3F8: Option<fn(*mut core::ffi::c_void, i32) > = None;
            #[allow(non_snake_case)]
            pub fn it_8026B3F8(article: *mut core::ffi::c_void, kind: i32)  {
                match unsafe { IT_8026B3F8 } {
                    Some(f) => f(article, kind),
                    None => { unsafe { melee_items::tu::it_26B1::it_8026B3F8(article as _, kind as _); } return; },
                }
            }
}

#[inline(always)]
unsafe fn fpp(gobj: *mut HSD_GObj) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}

pub fn ftPc_Init_OnLoad(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftPc_Init_OnLoad");
    unsafe {
        let fp = fpp(gobj);
        let items = (*(*fp).ft_data).x48_items;
        (*fp).set_can_walljump(1);
        crate::tu::ftpikachu::ftPk_Init_OnLoadForPichu(fp);
        let attrs = (*fp).dat_attrs as *const ftPichuAttributes;
        ext::it_8026B3F8(*items.add(0), (*attrs).xDC as i32);
        ext::it_8026B3F8(*items.add(1), (*attrs).x14 as i32);
        ext::it_8026B3F8(*items.add(2), (*attrs).x18 as i32);
    }
}

pub fn ftPc_Init_OnDeath(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftPc_Init_OnDeath");
    unsafe {
        let fp = fpp(gobj);
        st::ftParts_80074A4C(gobj, 0, 0);
        match (*fp).x619_costume_id {
            0 => {
                st::ftParts_80074A4C(gobj, 1, -1);
                st::ftParts_80074A4C(gobj, 2, -1);
                st::ftParts_80074A4C(gobj, 3, -1);
            }
            1 => {
                st::ftParts_80074A4C(gobj, 1, 0);
                st::ftParts_80074A4C(gobj, 2, -1);
                st::ftParts_80074A4C(gobj, 3, -1);
            }
            2 => {
                st::ftParts_80074A4C(gobj, 1, -1);
                st::ftParts_80074A4C(gobj, 2, 0);
                st::ftParts_80074A4C(gobj, 3, -1);
            }
            3 => {
                st::ftParts_80074A4C(gobj, 1, -1);
                st::ftParts_80074A4C(gobj, 2, -1);
                st::ftParts_80074A4C(gobj, 3, 0);
            }
            _ => {}
        }
    }
}

pub fn ftPc_Init_OnItemPickup(gobj: *mut HSD_GObj, flag: bool) {
    melee_core::ct!("ftPc_Init_OnItemPickup");
    Fighter_OnItemPickup(gobj, flag as i32, 0, 0);
}

pub fn ftPc_Init_OnItemDrop(gobj: *mut HSD_GObj, bool1: bool) {
    melee_core::ct!("ftPc_Init_OnItemDrop");
    Fighter_OnItemDrop(gobj, bool1 as i32, 0, 0);
}

pub fn ftPc_Init_LoadSpecialAttrs(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftPc_Init_LoadSpecialAttrs");
    crate::tu::ftpikachu::ftPk_Init_LoadSpecialAttrs(gobj);
}

/// `ftPc_CostumeList`: zero-initialised BSS array of the C (`UnkCostumeStruct ftPc_CostumeList[4]`), filled when the costumes load.
pub static mut ftPc_CostumeList: [melee_types::UnkCostumeStruct; 4] = unsafe { core::mem::zeroed() };
