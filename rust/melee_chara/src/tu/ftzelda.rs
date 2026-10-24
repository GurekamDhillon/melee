//! src/melee/ft/kinds/ftZelda/ftzelda.c: Zelda's init callbacks. No float arithmetic, so no fused
//! sites (tools/rust-port/fma_sites.py). State-oracle checked against the C (tests/oracle_ftzelda.rs,
//! gen/oracle).
//!
//! Data: the file-name strings are below; `ftZd_Init_MotionStateTable` (19 rows of Anim/IASA/Phys/Coll
//! pointers into the ftZelda special TUs, most unported) belongs to the motion-state dispatcher and is
//! TODO until those callbacks are all native. `ftZd_CostumeList` is zero-initialised BSS.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use crate::tu::ftinl;
use melee_types::*;

/// Already ported (rounds 5/6): re-exported under this TU.
pub use crate::chars::r5::ftZelda::ftzelda::{ftZd_Init_OnDeath, ftZd_Init_OnKnockbackEnter, ftZd_Init_OnKnockbackExit};
pub use crate::chars::r6::ftZelda::ftzelda::{ftZd_Init_OnItemInvisible, ftZd_Init_OnItemVisible};

pub const ftZd_Init_DatFilename: &str = "PlZd.dat";
pub const ftZd_Init_DataName: &str = "ftDataZelda";
pub const ftZd_Init_803CFCB0: &str = "PlZdNr.dat";
pub const ftZd_Init_803CFCBC: &str = "PlyZelda5K_Share_joint";
pub const ftZd_Init_803CFCD4: &str = "PlyZelda5K_Share_matanim_joint";
pub const ftZd_Init_803CFCF4: &str = "PlZdRe.dat";
pub const ftZd_Init_803CFD00: &str = "PlyZelda5KRe_Share_joint";
pub const ftZd_Init_803CFD1C: &str = "PlyZelda5KRe_Share_matanim_joint";
pub const ftZd_Init_803CFD40: &str = "PlZdBu.dat";
pub const ftZd_Init_803CFD4C: &str = "PlyZelda5KBu_Share_joint";
pub const ftZd_Init_803CFD68: &str = "PlyZelda5KBu_Share_matanim_joint";
pub const ftZd_Init_803CFD8C: &str = "PlZdGr.dat";
pub const ftZd_Init_803CFD98: &str = "PlyZelda5KGr_Share_joint";
pub const ftZd_Init_803CFDB4: &str = "PlyZelda5KGr_Share_matanim_joint";
pub const ftZd_Init_803CFDD8: &str = "PlZdWh.dat";
pub const ftZd_Init_803CFDE4: &str = "PlyZelda5KWh_Share_joint";
pub const ftZd_Init_803CFE00: &str = "PlyZelda5KWh_Share_matanim_joint";
pub const ftZd_Init_AnimDatFilename: &str = "PlZdAJ.dat";
/// `ftZd_Init_DemoMotionFilenames` (result, intro, ending, vi-wait).
pub const ftZd_Init_DemoMotionFilenames: [&str; 4] = [
    "ftDemoResultMotionFileZelda",
    "ftDemoIntroMotionFileZelda",
    "ftDemoEndingMotionFileZelda",
    "ftDemoViWaitMotionFileZelda",
];
/// `ftZd_Init_CostumeStrings`: (dat, joint, matanim joint) per costume.
pub const ftZd_Init_CostumeStrings: [[&str; 3]; 5] = [
    [ftZd_Init_803CFCB0, ftZd_Init_803CFCBC, ftZd_Init_803CFCD4],
    [ftZd_Init_803CFCF4, ftZd_Init_803CFD00, ftZd_Init_803CFD1C],
    [ftZd_Init_803CFD40, ftZd_Init_803CFD4C, ftZd_Init_803CFD68],
    [ftZd_Init_803CFD8C, ftZd_Init_803CFD98, ftZd_Init_803CFDB4],
    [ftZd_Init_803CFDD8, ftZd_Init_803CFDE4, ftZd_Init_803CFE00],
];

/// Calls into TUs that are not ported yet. Each is a settable hook; unset it panics.
pub mod ext {
    use crate::tu::hookmac::{hook, hook_or};
    use core::ffi::c_void;
    use melee_types::*;
    pub static mut IT_8026B3F8: Option<fn(*mut c_void, i32)> = None;
    #[allow(non_snake_case)]
    pub fn it_8026B3F8(article: *mut c_void, kind: i32) {
        match unsafe { IT_8026B3F8 } { Some(f) => f(article, kind), None => { unsafe { let _ = melee_items::tu::it_26B1::it_8026B3F8(article as _, kind as _); } return; } }
    }
    pub static mut FTZD_SPECIALLW_8013B5EC: Option<fn(*mut HSD_GObj)> = None;
    #[allow(non_snake_case)]
    pub fn ftZd_SpecialLw_8013B5EC(gobj: *mut HSD_GObj) {
        match unsafe { FTZD_SPECIALLW_8013B5EC } { Some(f) => f(gobj), None => { unsafe { let _ = crate::tu::ftzeldaspeciallw::ftZd_SpecialLw_8013B5EC(gobj as _); } return; } }
    }
    hook_or!(FTPARTS_80074A4C, ftParts_80074A4C, (gobj: *mut HSD_GObj, model_idx: i32, val: i32), melee_fighter::ft0df::ftparts_80074a4c);
    hook_or!(FTPARTS_80074A74, ftParts_80074A74, (gobj: *mut HSD_GObj, model_idx: i32) -> i32, melee_fighter::ft0df::ftparts_80074a74);
}

pub fn ftZd_Init_OnLoad(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        let item_list = (*(*fp).ft_data).x48_items;
        ftinl::push_attrs::<ftZelda_DatAttrs>(fp);
        ext::it_8026B3F8(*item_list.add(0), It_Kind_Zelda_DinFire as i32);
        ext::it_8026B3F8(*item_list.add(1), It_Kind_Zelda_DinFire_Explode as i32);
    }
}

pub fn ftZd_Init_801393AC(gobj: *mut HSD_GObj) {
    ext::ftZd_SpecialLw_8013B5EC(gobj);
}

pub fn ftZd_Init_OnItemPickup(gobj: *mut HSD_GObj, flag: i32) {
    ftinl::Fighter_OnItemPickup(gobj, flag, 1, 1);
}

pub fn ftZd_Init_OnItemDrop(gobj: *mut HSD_GObj, flag: i32) {
    ftinl::Fighter_OnItemDrop(gobj, flag, 1, 1);
}

pub fn ftZd_Init_LoadSpecialAttrs(gobj: *mut HSD_GObj) {
    unsafe { ftinl::copy_attrs::<ftZelda_DatAttrs>(gobj) }
}

/// `Zelda_RemoveDress`.
pub fn ftZd_Init_801395C8(gobj: *mut HSD_GObj) {
    let result = ext::ftParts_80074A74(gobj, 1);
    if result == -1 {
        ext::ftParts_80074A4C(gobj, 1, 0);
    } else {
        ext::ftParts_80074A4C(gobj, 1, -1);
    }
}
