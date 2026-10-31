//! src/melee/ft/kinds/ftNess/ftness.c: Ness's init callbacks.
//! No fused sites (tools/rust-port/fma_sites.py), no float math. Smoke-tested through recording
//! hooks by tests/tu_ftness.rs.
//!
//! Data: the file-name strings are below. `ftNs_Init_MotionStateTable` (41 rows from motion 341:
//! SM/MF/move-id words and Anim/IASA/Phys/Coll pointers into the other ftNess TUs) belongs to the
//! motion-state dispatcher and is TODO until those callbacks are native `extern "C"` entries, and
//! `ftNs_CostumeList` is an uninitialised 4-entry BSS array filled at load.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use melee_types::*;

/// Already ported (rounds 2/5/6): re-exported under this TU.
pub use crate::chars::r5::ftNess::ftness::{
    ftNs_Init_OnDeath, ftNs_Init_OnKnockbackEnter, ftNs_Init_OnKnockbackExit,
};
pub use crate::chars::r6::ftNess::ftness::{ftNs_Init_OnItemInvisible, ftNs_Init_OnItemVisible};
pub use crate::chars::ftNess::ftness::ftNs_Init_GetAbsorbHeal;

pub const ftNs_Init_DatFilename: &str = "PlNs.dat";
pub const ftNs_Init_DataName: &str = "ftDataNess";
pub const ftNs_Init_AnimDatFilename: &str = "PlNsAJ.dat";
/// `ftNs_Init_DemoMotionFilenames` (result, intro, ending, vi-wait).
pub const ftNs_Init_DemoMotionFilenames: [&str; 4] = [
    "ftDemoResultMotionFileNess",
    "ftDemoIntroMotionFileNess",
    "ftDemoEndingMotionFileNess",
    "ftDemoViWaitMotionFileNess",
];
/// `ftNs_Init_CostumeStrings`: (dat, joint, matanim joint) per costume.
pub const ftNs_Init_CostumeStrings: [[&str; 3]; 4] = [
    ["PlNsNr.dat", "PlyNess5K_Share_joint", "PlyNess5K_Share_matanim_joint"],
    ["PlNsYe.dat", "PlyNess5KYe_Share_joint", "PlyNess5KYe_Share_matanim_joint"],
    ["PlNsBu.dat", "PlyNess5KBu_Share_joint", "PlyNess5KBu_Share_matanim_joint"],
    ["PlNsGr.dat", "PlyNess5KGr_Share_joint", "PlyNess5KGr_Share_matanim_joint"],
];

/// Calls into TUs that are not ported yet. Each is a settable hook; unset it panics.
pub mod ext {
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
                    None => { unsafe { melee_items::tu::it_26B1::it_8026B3F8(article as _, kind as _); } return; },
                }
            }
}

#[inline(always)]
unsafe fn fpp(gobj: *mut HSD_GObj) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}

pub use crate::tu::ftfox::{copy_attrs, push_attrs};

pub fn ftNs_Init_OnLoad(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_Init_OnLoad");
    unsafe {
        let fp = fpp(gobj);
        let item_list = (*(*fp).ft_data).x48_items;
        push_attrs::<ftNessAttributes>(fp);
        ext::it_8026B3F8(*item_list.add(0), It_Kind_Ness_PKFire as i32);
        ext::it_8026B3F8(*item_list.add(1), It_Kind_Ness_PKFire_Flame as i32);
        ext::it_8026B3F8(*item_list.add(2), It_Kind_Ness_PKFlush as i32);
        ext::it_8026B3F8(*item_list.add(8), It_Kind_Ness_PKFlush_Explode as i32);
        ext::it_8026B3F8(*item_list.add(3), It_Kind_Ness_PKThunder as i32);
        ext::it_8026B3F8(*item_list.add(4), It_Kind_Ness_PKThunder1 as i32);
        ext::it_8026B3F8(*item_list.add(5), It_Kind_Ness_PKThunder2 as i32);
        ext::it_8026B3F8(*item_list.add(6), It_Kind_Ness_PKThunder3 as i32);
        ext::it_8026B3F8(*item_list.add(7), It_Kind_Ness_PKThunder4 as i32);
        ext::it_8026B3F8(*item_list.add(9), It_Kind_Ness_Bat as i32);
        ext::it_8026B3F8(*item_list.add(10), It_Kind_Ness_Yoyo as i32);
    }
}

pub fn ftNs_Init_OnDamage(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_Init_OnDamage");
    crate::tu::ftnessattackhi4::ftNs_AttackHi4_YoyoItemDespawn(gobj);
    crate::tu::ftnessspecialn::ftNs_SpecialN_ItemPKFlushSetNULL(gobj);
    crate::tu::ftnessspecialhi::ftNs_SpecialHi_TakeDamage(gobj);
    crate::tu::ftnessattacks4::ftNs_AttackS4_ItemNessBatRemove(gobj);
}

pub fn ftNs_Init_OnAbsorb(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_Init_OnAbsorb");
    crate::tu::ftnessspeciallw::ftNs_AbsorbThink_DecideAction(gobj);
}

pub fn ftNs_Init_OnItemPickup(gobj: *mut HSD_GObj, catch_item: i32) {
    melee_core::ct!("ftNs_Init_OnItemPickup");
    crate::tu::ftclink::Fighter_OnItemPickup(gobj, catch_item, 1, 1);
}

pub fn ftNs_Init_OnItemDrop(gobj: *mut HSD_GObj, drop_item: i32) {
    melee_core::ct!("ftNs_Init_OnItemDrop");
    crate::tu::ftclink::Fighter_OnItemDrop(gobj, drop_item, 1, 1);
}

pub fn ftNs_Init_LoadSpecialAttrs(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_Init_LoadSpecialAttrs");
    unsafe { copy_attrs::<ftNessAttributes>(gobj) }
}

/// `ftNs_CostumeList`: zero-initialised BSS array of the C (`UnkCostumeStruct ftNs_CostumeList[4]`), filled when the costumes load.
pub static mut ftNs_CostumeList: [melee_types::UnkCostumeStruct; 4] = unsafe { core::mem::zeroed() };
