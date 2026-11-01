//! src/melee/ft/kinds/ftFox/ftfox.c: Fox's init callbacks (also used by Falco).
//! No fused sites (tools/rust-port/fma_sites.py), no float math. Smoke-tested through recording
//! hooks by tests/tu_ftfox.rs.
//!
//! Data: the file-name strings are below; `ftFx_Init_MotionStateTable` (rows of
//! Anim/IASA/Phys/Coll pointers into the other ftFox TUs, from motion 341) belongs to the
//! motion-state dispatcher and is TODO until those callbacks are all native, and
//! `ftFx_CostumeList` is an uninitialised 4-entry BSS array filled at load.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use crate::stubs as st;
#[allow(unused_imports)]
use melee_types::{Fighter, HSD_GObj};
use melee_types::*;

/// Already ported (rounds 5/6): re-exported under this TU.



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
    pub static mut FTFX_SPECIALN_REMOVEBLASTER: Option<fn(*mut HSD_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn ftFx_SpecialN_RemoveBlaster(gobj: *mut HSD_GObj)  {
                match unsafe { FTFX_SPECIALN_REMOVEBLASTER } {
                    Some(f) => f(gobj),
                    None => { unsafe { crate::tu::ftfoxspecialn::ftFx_SpecialN_RemoveBlaster(gobj as _); } return; },
                }
            }
}

#[inline(always)]
unsafe fn fpp(gobj: *mut HSD_GObj) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}

/// `PUSH_ATTRS(fp, T)` (ft/inlines.h): copy the loaded `ext_attr` block into the fighter's backup
/// buffer and make that the live `dat_attrs`.
pub unsafe fn push_attrs<T: Copy>(fp: *mut Fighter) {
    melee_core::ct!("push_attrs");
    let backup = (*fp).dat_attrs_backup;
    let src = (*(*fp).ft_data).ext_attr as *const T;
    *((*fp).dat_attrs_backup as *mut T) = *src;
    (*fp).dat_attrs = backup;
}

/// `COPY_ATTRS(gobj, T)` (ft/inlines.h): overwrite the live `dat_attrs` with `ext_attr`.
pub unsafe fn copy_attrs<T: Copy>(gobj: *mut HSD_GObj) {
    melee_core::ct!("copy_attrs");
    let fp = fpp(gobj);
    let dst = (*fp).dat_attrs as *mut T;
    let src = (*(*fp).ft_data).ext_attr as *const T;
    *dst = *src;
}

/// C `bool`: whether the blaster is out.
pub fn ftFx_Init_800E5534(gobj: *mut HSD_GObj) -> bool {
    melee_core::ct!("ftFx_Init_800E5534");
    unsafe { !(*fpp(gobj)).u.fx.x222C_blasterGObj.is_null() }
}

pub fn ftFx_Init_800E5588(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftFx_Init_800E5588");
    ext::ftFx_SpecialN_RemoveBlaster(gobj);
}

pub fn ftFx_Init_OnItemPickup(gobj: *mut HSD_GObj, flag: i32) {
    melee_core::ct!("ftFx_Init_OnItemPickup");
    crate::tu::ftclink::Fighter_OnItemPickup(gobj, flag, 1, 1);
}

pub fn ftFx_Init_OnItemDrop(gobj: *mut HSD_GObj, flag: i32) {
    melee_core::ct!("ftFx_Init_OnItemDrop");
    crate::tu::ftclink::Fighter_OnItemDrop(gobj, flag, 1, 1);
}

pub fn ftFx_Init_OnLoadForFalco(fp: *mut Fighter) {
    melee_core::ct!("ftFx_Init_OnLoadForFalco");
    unsafe { push_attrs::<ftFox_DatAttrs>(fp) }
}

pub fn ftFx_Init_OnLoad(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftFx_Init_OnLoad");
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
    melee_core::ct!("ftFx_Init_LoadSpecialAttrs");
    unsafe { copy_attrs::<ftFox_DatAttrs>(gobj) }
}

/// `ftFx_CostumeList`: zero-initialised BSS array of the C (`UnkCostumeStruct ftFx_CostumeList[4]`), filled when the costumes load.
pub static mut ftFx_CostumeList: [melee_types::UnkCostumeStruct; 4] = unsafe { core::mem::zeroed() };

// ---- folded from chars (consolidation step 20/21) ----
/// `ftFx_Init_OnDeath` (src/melee/ft/kinds/ftFox/ftfox.c).
pub fn ftFx_Init_OnDeath(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftFx_Init_OnDeath");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        (*fp).u.fx.x222C_blasterGObj = core::mem::zeroed(); /* 0 */
        st::ftParts_80074A4C(gobj, 0, 0);
    }
}

/// `ftFx_Init_OnItemInvisible` (src/melee/ft/kinds/ftFox/ftfox.c).
pub fn ftFx_Init_OnItemInvisible(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftFx_Init_OnItemInvisible");
    unsafe {
        st::Fighter_OnItemInvisible(gobj, 1);
    }
}

/// `ftFx_Init_OnItemVisible` (src/melee/ft/kinds/ftFox/ftfox.c).
pub fn ftFx_Init_OnItemVisible(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftFx_Init_OnItemVisible");
    unsafe {
        st::Fighter_OnItemVisible(gobj, 1);
    }
}

/// `ftFx_Init_OnKnockbackEnter` (src/melee/ft/kinds/ftFox/ftfox.c).
pub fn ftFx_Init_OnKnockbackEnter(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftFx_Init_OnKnockbackEnter");
    unsafe {
        st::Fighter_OnKnockbackEnter(gobj, 1);
        st::ftAnim_ApplyPartAnim(gobj, 3, 3, 0.0_f32);
        st::ftAnim_ApplyPartAnim(gobj, 4, 3, 0.0_f32);
    }
}

/// `ftFx_Init_OnKnockbackExit` (src/melee/ft/kinds/ftFox/ftfox.c).
pub fn ftFx_Init_OnKnockbackExit(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftFx_Init_OnKnockbackExit");
    unsafe {
        st::Fighter_OnKnockbackExit(gobj, 1);
        st::ftAnim_ApplyPartAnim(gobj, 3, 2, 0.0_f32);
        st::ftAnim_ApplyPartAnim(gobj, 4, 2, 0.0_f32);
    }
}
