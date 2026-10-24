//! src/melee/ft/kinds/ftPikachu/ftpikachu.c: Pikachu init callbacks and file names (Pichu
//! reuses `ftPk_Init_OnLoadForPichu` / `ftPk_Init_LoadSpecialAttrs`).
//! No fused sites (tools/rust-port/fma_sites.py); the attribute scaling loop is a plain f32
//! multiply. Smoke-tested by tests/tu_ftpikachu.rs.
//!
//! Data: the file-name strings are below. `ftPk_Init_MotionStateTable` (28 rows of callbacks
//! into every ftPikachu TU plus `ftCamera_UpdateCameraBox`, from motion 341) belongs to the
//! motion-state dispatcher and is TODO until those callbacks are all native;
//! `ftPk_CostumeList` is zero-initialised BSS filled at load.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use crate::tu::ftclink::{Fighter_OnItemDrop, Fighter_OnItemPickup};
use crate::tu::ftfox::{copy_attrs, push_attrs};
use melee_types::*;

/// Already ported (rounds 5/6): re-exported under this TU.
pub use crate::chars::r5::ftPikachu::ftpikachu::{
    ftPk_Init_OnDeath, ftPk_Init_OnKnockbackEnter, ftPk_Init_OnKnockbackExit,
    ftPk_Init_UnkMotionStates1, ftPk_Init_UnkMotionStates2,
};
pub use crate::chars::r6::ftPikachu::ftpikachu::{ftPk_Init_OnItemInvisible, ftPk_Init_OnItemVisible};

pub const ftPk_Init_DatFilename: &str = "PlPk.dat";
pub const ftPk_Init_DataName: &str = "ftDataPikachu";
pub const ftPk_Init_803CE0D4: &str = "PlPkNr.dat";
pub const ftPk_Init_803CE0E0: &str = "PlyPikachu5K_Share_joint";
pub const ftPk_Init_803CE0FC: &str = "PlyPikachu5K_Share_matanim_joint";
pub const ftPk_Init_803CE120: &str = "PlPkRe.dat";
pub const ftPk_Init_803CE12C: &str = "PlyPikachu5KRe_Share_joint";
pub const ftPk_Init_803CE148: &str = "PlyPikachu5KRe_Share_matanim_joint";
pub const ftPk_Init_803CE16C: &str = "PlPkBu.dat";
pub const ftPk_Init_803CE178: &str = "PlyPikachu5KBu_Share_joint";
pub const ftPk_Init_803CE194: &str = "PlyPikachu5KBu_Share_matanim_joint";
pub const ftPk_Init_803CE1B8: &str = "PlPkGr.dat";
pub const ftPk_Init_803CE1C4: &str = "PlyPikachu5KGr_Share_joint";
pub const ftPk_Init_803CE1E0: &str = "PlyPikachu5KGr_Share_matanim_joint";
pub const ftPk_Init_AnimDatFilename: &str = "PlPkAJ.dat";

/// `ftPk_Init_DemoMotionFilenames` (result, intro, ending, vi-wait).
pub const ftPk_Init_DemoMotionFilenames: [&str; 4] = [
    "ftDemoResultMotionFilePikachu",
    "ftDemoIntroMotionFilePikachu",
    "ftDemoEndingMotionFilePikachu",
    "ftDemoViWaitMotionFilePikachu",
];

/// `ftPk_Init_CostumeStrings`: (dat, joint, matanim joint) per costume.
pub const ftPk_Init_CostumeStrings: [[&str; 3]; 4] = [
    [ftPk_Init_803CE0D4, ftPk_Init_803CE0E0, ftPk_Init_803CE0FC],
    [ftPk_Init_803CE120, ftPk_Init_803CE12C, ftPk_Init_803CE148],
    [ftPk_Init_803CE16C, ftPk_Init_803CE178, ftPk_Init_803CE194],
    [ftPk_Init_803CE1B8, ftPk_Init_803CE1C4, ftPk_Init_803CE1E0],
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
    pub static mut IT_8026B3F8: Option<fn(*mut core::ffi::c_void, i32) > = None;
            #[allow(non_snake_case)]
            pub fn it_8026B3F8(article: *mut core::ffi::c_void, kind: i32)  {
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

pub fn ftPk_Init_OnLoadForPichu(fp: *mut Fighter) {
    unsafe { push_attrs::<ftPikachuAttributes>(fp) }
}

pub fn ftPk_Init_OnLoad(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let item_list = (*(*fp).ft_data).x48_items;
        push_attrs::<ftPikachuAttributes>(fp);
        let pika_attr = (*fp).dat_attrs as *const ftPikachuAttributes;
        ext::it_8026B3F8(*item_list.add(0), (*pika_attr).xDC as i32);
        ext::it_8026B3F8(*item_list.add(1), (*pika_attr).specialn_itkind as i32);
        ext::it_8026B3F8(*item_list.add(2), (*pika_attr).specialairn_itkind as i32);
    }
}

pub fn ftPk_Init_OnItemPickup(gobj: *mut HSD_GObj, flag: bool) {
    Fighter_OnItemPickup(gobj, flag as i32, 0, 0);
}

pub fn ftPk_Init_OnItemDrop(gobj: *mut HSD_GObj, bool1: bool) {
    Fighter_OnItemDrop(gobj, bool1 as i32, 0, 0);
}

/// `COPY_ATTRS`, then every float of `height_attributes` scaled by the fighter's `x34_scale.y`.
pub fn ftPk_Init_LoadSpecialAttrs(gobj: *mut HSD_GObj) {
    unsafe {
        copy_attrs::<ftPikachuAttributes>(gobj);
        let fp = fpp(gobj);
        let sa2 = (*fp).dat_attrs as *mut ftPikachuAttributes;
        if (*fp).x34_scale.y != 1.0 {
            let h = &raw mut (*sa2).height_attributes as *mut f32;
            for i in 0..(core::mem::size_of::<ftCollisionBox>() / core::mem::size_of::<f32>()) {
                *h.add(i) *= (*fp).x34_scale.y;
            }
        }
    }
}
