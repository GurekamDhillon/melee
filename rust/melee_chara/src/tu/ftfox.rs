//! src/melee/ft/kinds/ftFox/ftfox.c: Fox's init callbacks (also used by Falco).
//! No fused sites (tools/rust-port/fma_sites.py), no float math. Smoke-tested through recording
//! hooks by tests/tu_ftfox.rs.
//!
//! Data: the file-name strings are below; `ftFx_Init_MotionStateTable` (rows of
//! Anim/IASA/Phys/Coll pointers into the other ftFox TUs, from motion 341) belongs to the
//! motion-state dispatcher and is TODO until those callbacks are all native, and
//! `ftFx_CostumeList` is an uninitialised 4-entry BSS array filled at load.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use melee_types::*;

/// Already ported (rounds 5/6): re-exported under this TU.
pub use crate::chars::r5::ftFox::ftfox::ftFx_Init_OnDeath;
pub use crate::chars::r6::ftFox::ftfox::{
    ftFx_Init_OnItemInvisible, ftFx_Init_OnItemVisible, ftFx_Init_OnKnockbackEnter,
    ftFx_Init_OnKnockbackExit,
};

pub const ftFx_Init_DatFilename: &str = "PlFx.dat";
pub const ftFx_Init_DataName: &str = "ftDataFox";
pub const ftFx_Init_AnimDatFilename: &str = "PlFxAJ.dat";
/// `ftFx_Init_DemoMotionFilenames` (result, intro, ending, vi-wait).
pub const ftFx_Init_DemoMotionFilenames: [&str; 4] = [
    "ftDemoResultMotionFileFox",
    "ftDemoIntroMotionFileFox",
    "ftDemoEndingMotionFileFox",
    "ftDemoViWaitMotionFileFox",
];
/// `ftFx_Init_CostumeStrings`: (dat, joint, matanim joint) per costume.
pub const ftFx_Init_CostumeStrings: [[&str; 3]; 4] = [
    ["PlFxNr.dat", "PlyFox5K_Share_joint", "PlyFox5K_Share_matanim_joint"],
    ["PlFxOr.dat", "PlyFox5KOr_Share_joint", "PlyFox5KOr_Share_matanim_joint"],
    ["PlFxLa.dat", "PlyFox5KLa_Share_joint", "PlyFox5KLa_Share_matanim_joint"],
    ["PlFxGr.dat", "PlyFox5KGr_Share_joint", "PlyFox5KGr_Share_matanim_joint"],
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
    hook!(IT_8026B3F8, it_8026B3F8, (article: *mut core::ffi::c_void, kind: i32));
    hook!(FTFX_SPECIALN_REMOVEBLASTER, ftFx_SpecialN_RemoveBlaster, (gobj: *mut HSD_GObj));
}

#[inline(always)]
unsafe fn fpp(gobj: *mut HSD_GObj) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}

/// `PUSH_ATTRS(fp, T)` (ft/inlines.h): copy the loaded `ext_attr` block into the fighter's backup
/// buffer and make that the live `dat_attrs`.
pub unsafe fn push_attrs<T: Copy>(fp: *mut Fighter) {
    let backup = (*fp).dat_attrs_backup;
    let src = (*(*fp).ft_data).ext_attr as *const T;
    *((*fp).dat_attrs_backup as *mut T) = *src;
    (*fp).dat_attrs = backup;
}

/// `COPY_ATTRS(gobj, T)` (ft/inlines.h): overwrite the live `dat_attrs` with `ext_attr`.
pub unsafe fn copy_attrs<T: Copy>(gobj: *mut HSD_GObj) {
    let fp = fpp(gobj);
    let dst = (*fp).dat_attrs as *mut T;
    let src = (*(*fp).ft_data).ext_attr as *const T;
    *dst = *src;
}

/// C `bool`: whether the blaster is out.
pub fn ftFx_Init_800E5534(gobj: *mut HSD_GObj) -> bool {
    unsafe { !(*fpp(gobj)).u.fx.x222C_blasterGObj.is_null() }
}

pub fn ftFx_Init_800E5588(gobj: *mut HSD_GObj) {
    ext::ftFx_SpecialN_RemoveBlaster(gobj);
}

pub fn ftFx_Init_OnItemPickup(gobj: *mut HSD_GObj, flag: i32) {
    crate::tu::ftclink::Fighter_OnItemPickup(gobj, flag, 1, 1);
}

pub fn ftFx_Init_OnItemDrop(gobj: *mut HSD_GObj, flag: i32) {
    crate::tu::ftclink::Fighter_OnItemDrop(gobj, flag, 1, 1);
}

pub fn ftFx_Init_OnLoadForFalco(fp: *mut Fighter) {
    unsafe { push_attrs::<ftFox_DatAttrs>(fp) }
}

pub fn ftFx_Init_OnLoad(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let item_list = (*(*fp).ft_data).x48_items;
        (*fp).set_can_walljump(1);
        push_attrs::<ftFox_DatAttrs>(fp);
        let fox_attr = (*fp).dat_attrs as *mut ftFox_DatAttrs;
        ext::it_8026B3F8(*item_list.add(0), (*fox_attr).x1C_FOX_BLASTER_SHOT_ITKIND as i32);
        ext::it_8026B3F8(*item_list.add(1), (*fox_attr).x20_FOX_BLASTER_GUN_ITKIND as i32);
        ext::it_8026B3F8(*item_list.add(2), It_Kind_Fox_Illusion as i32);
    }
}

pub fn ftFx_Init_LoadSpecialAttrs(gobj: *mut HSD_GObj) {
    unsafe { copy_attrs::<ftFox_DatAttrs>(gobj) }
}
