//! src/melee/ft/kinds/ftZelda/ftzelda.c: Zelda's init callbacks. No float arithmetic, so no fused
//! sites (tools/rust-port/fma_sites.py). State-oracle checked against the C (tests/oracle_ftzelda.rs,
//! gen/oracle).
//!
//! Data: the file-name strings are below; `ftZd_Init_MotionStateTable` (19 rows of Anim/IASA/Phys/Coll
//! pointers into the ftZelda special TUs, most unported) belongs to the motion-state dispatcher and is
//! TODO until those callbacks are all native. `ftZd_CostumeList` is zero-initialised BSS.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use crate::stubs as st;
#[allow(unused_imports)]
use melee_types::{Fighter, HSD_GObj};
use crate::tu::ftinl;
use melee_types::*;

/// Already ported (rounds 5/6): re-exported under this TU.



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
        match unsafe { IT_8026B3F8 } { Some(f) => f(article, kind), None => { unsafe { melee_items::tu::it_26B1::it_8026B3F8(article as _, kind as _); } return; } }
    }
    pub static mut FTZD_SPECIALLW_8013B5EC: Option<fn(*mut HSD_GObj)> = None;
    #[allow(non_snake_case)]
    pub fn ftZd_SpecialLw_8013B5EC(gobj: *mut HSD_GObj) {
        match unsafe { FTZD_SPECIALLW_8013B5EC } { Some(f) => f(gobj), None => { unsafe { crate::tu::ftzeldaspeciallw::ftZd_SpecialLw_8013B5EC(gobj as _); } return; } }
    }
    hook_or!(FTPARTS_80074A4C, ftParts_80074A4C, (gobj: *mut HSD_GObj, model_idx: i32, val: i32), melee_fighter::ft0df::ftparts_80074a4c);
    hook_or!(FTPARTS_80074A74, ftParts_80074A74, (gobj: *mut HSD_GObj, model_idx: i32) -> i32, melee_fighter::ft0df::ftparts_80074a74);
}

pub fn ftZd_Init_OnLoad(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftZd_Init_OnLoad");
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        let item_list = (*(*fp).ft_data).x48_items;
        ftinl::push_attrs::<ftZelda_DatAttrs>(fp);
        ext::it_8026B3F8(*item_list.add(0), It_Kind_Zelda_DinFire as i32);
        ext::it_8026B3F8(*item_list.add(1), It_Kind_Zelda_DinFire_Explode as i32);
    }
}

pub fn ftZd_Init_801393AC(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftZd_Init_801393AC");
    ext::ftZd_SpecialLw_8013B5EC(gobj);
}

pub fn ftZd_Init_OnItemPickup(gobj: *mut HSD_GObj, flag: i32) {
    melee_core::ct!("ftZd_Init_OnItemPickup");
    ftinl::Fighter_OnItemPickup(gobj, flag, 1, 1);
}

pub fn ftZd_Init_OnItemDrop(gobj: *mut HSD_GObj, flag: i32) {
    melee_core::ct!("ftZd_Init_OnItemDrop");
    ftinl::Fighter_OnItemDrop(gobj, flag, 1, 1);
}

pub fn ftZd_Init_LoadSpecialAttrs(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftZd_Init_LoadSpecialAttrs");
    unsafe { ftinl::copy_attrs::<ftZelda_DatAttrs>(gobj) }
}

/// `Zelda_RemoveDress`.
pub fn ftZd_Init_801395C8(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftZd_Init_801395C8");
    let result = ext::ftParts_80074A74(gobj, 1);
    if result == -1 {
        ext::ftParts_80074A4C(gobj, 1, 0);
    } else {
        ext::ftParts_80074A4C(gobj, 1, -1);
    }
}

/// `ftZd_CostumeList`: zero-initialised BSS array of the C (`UnkCostumeStruct ftZd_CostumeList[5]`), filled when the costumes load.
pub static mut ftZd_CostumeList: [melee_types::UnkCostumeStruct; 5] = unsafe { core::mem::zeroed() };

// ---- folded from chars (consolidation step 20/21) ----
/// `ftZd_Init_OnDeath` (src/melee/ft/kinds/ftZelda/ftzelda.c).
pub fn ftZd_Init_OnDeath(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftZd_Init_OnDeath");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        st::ftParts_80074A4C(gobj, 0, 0);
        st::ftParts_80074A4C(gobj, 1, 0);
        (*fp).u.zd.x222C = core::mem::zeroed(); /* 0 */
    }
}

/// `ftZd_Init_OnKnockbackEnter` (src/melee/ft/kinds/ftZelda/ftzelda.c).
pub fn ftZd_Init_OnKnockbackEnter(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftZd_Init_OnKnockbackEnter");
    unsafe {
        st::Fighter_OnKnockbackEnter(gobj, 1);
    }
}

/// `ftZd_Init_OnKnockbackExit` (src/melee/ft/kinds/ftZelda/ftzelda.c).
pub fn ftZd_Init_OnKnockbackExit(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftZd_Init_OnKnockbackExit");
    unsafe {
        st::Fighter_OnKnockbackExit(gobj, 1);
    }
}

/// `ftZd_Init_OnItemInvisible` (src/melee/ft/kinds/ftZelda/ftzelda.c).
pub fn ftZd_Init_OnItemInvisible(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftZd_Init_OnItemInvisible");
    unsafe {
        st::Fighter_OnItemInvisible(gobj, 1);
    }
}

/// `ftZd_Init_OnItemVisible` (src/melee/ft/kinds/ftZelda/ftzelda.c).
pub fn ftZd_Init_OnItemVisible(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftZd_Init_OnItemVisible");
    unsafe {
        st::Fighter_OnItemVisible(gobj, 1);
    }
}
