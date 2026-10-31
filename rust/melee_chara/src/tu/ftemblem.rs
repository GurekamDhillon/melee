//! src/melee/ft/kinds/ftEmblem/ftemblem.c: Roy's init callbacks.
//! No fused sites (tools/rust-port/fma_sites.py), no float math. Smoke-tested through recording
//! hooks by tests/tu_ftemblem.rs.
//!
//! Data: the file-name strings are below; `ftFe_Init_MotionStateTable` (rows of
//! Anim/IASA/Phys/Coll pointers into ftMars TUs, from motion 341) belongs to the motion-state
//! dispatcher and is TODO until those callbacks are all native, and `ftFe_CostumeList` is an
//! uninitialised 5-entry BSS array filled at load.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use melee_types::*;

/// Already ported (rounds 5/6): re-exported under this TU.
pub use crate::chars::r5::ftEmblem::ftemblem::{
    ftFe_Init_OnDeath, ftFe_Init_OnKnockbackEnter, ftFe_Init_OnKnockbackExit,
};
pub use crate::chars::r6::ftEmblem::ftemblem::{ftFe_Init_OnItemInvisible, ftFe_Init_OnItemVisible};

pub const ftFe_Init_DatFilename: &str = "PlFe.dat";
pub const ftFe_Init_DataName: &str = "ftDataEmblem";
pub const ftFe_Init_AnimDatFilename: &str = "PlFeAJ.dat";
/// `ftFe_Init_DemoMotionFilenames` (result, intro, ending, vi-wait).
pub const ftFe_Init_DemoMotionFilenames: [&str; 4] = [
    "ftDemoResultMotionFileEmblem",
    "ftDemoIntroMotionFileEmblem",
    "ftDemoEndingMotionFileEmblem",
    "ftDemoViWaitMotionFileEmblem",
];
/// `ftFe_Init_CostumeStrings`: (dat, joint, matanim joint) per costume.
pub const ftFe_Init_CostumeStrings: [[&str; 3]; 5] = [
    ["PlFeNr.dat", "PlyEmblem5K_Share_joint", "PlyEmblem5K_Share_matanim_joint"],
    ["PlFeRe.dat", "PlyEmblem5KRe_Share_joint", "PlyEmblem5KRe_Share_matanim_joint"],
    ["PlFeBu.dat", "PlyEmblem5KBu_Share_joint", "PlyEmblem5KBu_Share_matanim_joint"],
    ["PlFeGr.dat", "PlyEmblem5KGr_Share_joint", "PlyEmblem5KGr_Share_matanim_joint"],
    ["PlFeYe.dat", "PlyEmblem5KYe_Share_joint", "PlyEmblem5KYe_Share_matanim_joint"],
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
    pub static mut FTMS_INIT_ONLOADFORROY: Option<fn(*mut Fighter) > = None;
            #[allow(non_snake_case)]
            pub fn ftMs_Init_OnLoadForRoy(fp: *mut Fighter)  {
                match unsafe { FTMS_INIT_ONLOADFORROY } {
                    Some(f) => f(fp),
                    None => { unsafe { let _ = crate::tu::ftmars::ftMs_Init_OnLoadForRoy(fp as _); } return; },
                }
            }
    pub static mut FTMS_INIT_LOADSPECIALATTRS: Option<fn(*mut HSD_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn ftMs_Init_LoadSpecialAttrs(gobj: *mut HSD_GObj)  {
                match unsafe { FTMS_INIT_LOADSPECIALATTRS } {
                    Some(f) => f(gobj),
                    None => { unsafe { let _ = crate::tu::ftmars::ftMs_Init_LoadSpecialAttrs(gobj as _); } return; },
                }
            }
}

pub fn ftFe_Init_OnItemPickup(gobj: *mut HSD_GObj, flag: i32) {
    melee_core::ct!("ftFe_Init_OnItemPickup");
    crate::tu::ftclink::Fighter_OnItemPickup(gobj, flag, 0, 1);
}

pub fn ftFe_Init_OnItemDrop(gobj: *mut HSD_GObj, flag: i32) {
    melee_core::ct!("ftFe_Init_OnItemDrop");
    crate::tu::ftclink::Fighter_OnItemDrop(gobj, flag, 0, 1);
}

pub fn ftFe_Init_OnLoad(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftFe_Init_OnLoad");
    unsafe {
        ext::ftMs_Init_OnLoadForRoy((*gobj).user_data as *mut Fighter);
    }
}

pub fn ftFe_Init_LoadSpecialAttrs(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftFe_Init_LoadSpecialAttrs");
    ext::ftMs_Init_LoadSpecialAttrs(gobj);
}

/// `ftFe_CostumeList`: zero-initialised BSS array of the C (`UnkCostumeStruct ftFe_CostumeList[5]`), filled when the costumes load.
pub static mut ftFe_CostumeList: [melee_types::UnkCostumeStruct; 5] = unsafe { core::mem::zeroed() };
