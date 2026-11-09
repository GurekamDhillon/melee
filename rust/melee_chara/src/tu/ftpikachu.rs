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

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use crate::stubs as st;
#[allow(unused_imports)]
use melee_types::{Fighter, HSD_GObj};
use crate::tu::ftclink::{Fighter_OnItemDrop, Fighter_OnItemPickup};
use crate::tu::ftfox::{copy_attrs, push_attrs};
use melee_types::*;

/// Already ported (rounds 5/6): re-exported under this TU.



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

pub fn ftPk_Init_OnLoadForPichu(fp: *mut Fighter) {
    melee_core::ct!("ftPk_Init_OnLoadForPichu");
    unsafe { push_attrs::<ftPikachuAttributes>(fp) }
}

pub fn ftPk_Init_OnLoad(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftPk_Init_OnLoad");
    unsafe {
        let fp = fpp(gobj);
        let item_list = (*(*fp).ft_data).x48_items;
        push_attrs::<ftPikachuAttributes>(fp);
        let pika_attr = (*fp).dat_attrs as *const ftPikachuAttributes;
        ext::it_8026B3F8((*item_list.add(0)).article as *mut ::core::ffi::c_void, (*pika_attr).xDC as i32);
        ext::it_8026B3F8((*item_list.add(1)).article as *mut ::core::ffi::c_void, (*pika_attr).specialn_itkind as i32);
        ext::it_8026B3F8((*item_list.add(2)).article as *mut ::core::ffi::c_void, (*pika_attr).specialairn_itkind as i32);
    }
}

pub fn ftPk_Init_OnItemPickup(gobj: *mut HSD_GObj, flag: bool) {
    melee_core::ct!("ftPk_Init_OnItemPickup");
    Fighter_OnItemPickup(gobj, flag as i32, 0, 0);
}

pub fn ftPk_Init_OnItemDrop(gobj: *mut HSD_GObj, bool1: bool) {
    melee_core::ct!("ftPk_Init_OnItemDrop");
    Fighter_OnItemDrop(gobj, bool1 as i32, 0, 0);
}

/// `COPY_ATTRS`, then every float of `height_attributes` scaled by the fighter's `x34_scale.y`.
pub fn ftPk_Init_LoadSpecialAttrs(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftPk_Init_LoadSpecialAttrs");
    unsafe {
        copy_attrs::<ftPikachuAttributes>(gobj);
        let fp = fpp(gobj);
        let sa2 = (*fp).dat_attrs as *mut ftPikachuAttributes;
        if (*fp).x34_scale.y != 1.0 {
            let h = &raw mut (*sa2).height_attributes as *mut f32;
            for i in 0..(<usize as melee_core::ppc::Ppc>::pdiv(core::mem::size_of::<ftCollisionBox>(), core::mem::size_of::<f32>())) {
                *h.add(i) *= (*fp).x34_scale.y;
            }
        }
    }
}

/// `ftPk_CostumeList`: zero-initialised BSS array of the C (`UnkCostumeStruct ftPk_CostumeList[4]`), filled when the costumes load.
pub static mut ftPk_CostumeList: [melee_types::UnkCostumeStruct; 4] = unsafe { core::mem::zeroed() };

// ---- folded from chars (consolidation step 20/21) ----
/// `ftPk_Init_OnDeath` (src/melee/ft/kinds/ftPikachu/ftpikachu.c).
pub fn ftPk_Init_OnDeath(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftPk_Init_OnDeath");
    unsafe {
        st::ftParts_80074A4C(gobj, 0, 0);
        st::ftParts_80074A4C(gobj, 1, 0);
    }
}

/// `ftPk_Init_UnkMotionStates1` (src/melee/ft/kinds/ftPikachu/ftpikachu.c).
pub fn ftPk_Init_UnkMotionStates1(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftPk_Init_UnkMotionStates1");
    unsafe {
        st::ftParts_80074A4C(gobj, 1, -1);
        st::ftParts_80074B0C(gobj, 1, 0);
    }
}

/// `ftPk_Init_UnkMotionStates2` (src/melee/ft/kinds/ftPikachu/ftpikachu.c).
pub fn ftPk_Init_UnkMotionStates2(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftPk_Init_UnkMotionStates2");
    unsafe {
        st::ftParts_80074A4C(gobj, 1, 0);
        st::ftParts_80074B0C(gobj, 1, 0);
    }
}

/// `ftPk_Init_OnKnockbackEnter` (src/melee/ft/kinds/ftPikachu/ftpikachu.c).
pub fn ftPk_Init_OnKnockbackEnter(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftPk_Init_OnKnockbackEnter");
    unsafe {
        st::Fighter_OnKnockbackEnter(gobj, 1);
    }
}

/// `ftPk_Init_OnKnockbackExit` (src/melee/ft/kinds/ftPikachu/ftpikachu.c).
pub fn ftPk_Init_OnKnockbackExit(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftPk_Init_OnKnockbackExit");
    unsafe {
        st::Fighter_OnKnockbackExit(gobj, 1);
    }
}

/// `ftPk_Init_OnItemInvisible` (src/melee/ft/kinds/ftPikachu/ftpikachu.c).
pub fn ftPk_Init_OnItemInvisible(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftPk_Init_OnItemInvisible");
    unsafe {
        st::Fighter_OnItemInvisible(gobj, 0);
    }
}

/// `ftPk_Init_OnItemVisible` (src/melee/ft/kinds/ftPikachu/ftpikachu.c).
pub fn ftPk_Init_OnItemVisible(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftPk_Init_OnItemVisible");
    unsafe {
        st::Fighter_OnItemVisible(gobj, 0);
    }
}
