//! src/melee/ft/kinds/ftMewtwo/ftmewtwo.c: Mewtwo's init callbacks. No fused sites
//! (tools/rust-port/fma_sites.py): the only float math is the scale multiply in
//! `ftMt_Init_LoadSpecialAttrs`. Smoke-tested through recording hooks by tests/tu_ftmewtwo.rs.
//!
//! Data: the file-name strings are below. `ftMt_Init_MotionStateTable` (20 rows of
//! Anim/IASA/Phys/Coll pointers into the other ftMewtwo TUs, from motion 341) belongs to the
//! motion-state dispatcher and is TODO until those callbacks are all native, and
//! `ftMt_CostumeList` is an uninitialised 4-entry BSS array filled at load.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use melee_types::*;

/// Already ported (rounds 5/12): re-exported under this TU.
pub use crate::chars::r12::ftMewtwo::ftmewtwo::{ftMt_Init_OnItemInvisible, ftMt_Init_OnItemVisible};
pub use crate::chars::r5::ftMewtwo::ftmewtwo::{
    ftMt_Init_OnDeath, ftMt_Init_OnKnockbackEnter, ftMt_Init_OnKnockbackExit,
};

pub const ftMt_Init_DatFilename: &str = "PlMt.dat";
pub const ftMt_Init_DataName: &str = "ftDataMewtwo";
pub const ftMt_Init_AnimDatFilename: &str = "PlMtAJ.dat";
/// `ftMt_Init_DemoMotionFilenames` (result, intro, ending, vi-wait).
pub const ftMt_Init_DemoMotionFilenames: [&str; 4] = [
    "ftDemoResultMotionFileMewtwo",
    "ftDemoIntroMotionFileMewtwo",
    "ftDemoEndingMotionFileMewtwo",
    "ftDemoViWaitMotionFileMewtwo",
];
/// `ftMt_Init_CostumeStrings`: (dat, joint, matanim joint) per costume.
pub const ftMt_Init_CostumeStrings: [[&str; 3]; 4] = [
    ["PlMtNr.dat", "PlyMewtwo5K_Share_joint", "PlyMewtwo5K_Share_matanim_joint"],
    ["PlMtRe.dat", "PlyMewtwo5KRe_Share_joint", "PlyMewtwo5KRe_Share_matanim_joint"],
    ["PlMtBu.dat", "PlyMewtwo5KBu_Share_joint", "PlyMewtwo5KBu_Share_matanim_joint"],
    ["PlMtGr.dat", "PlyMewtwo5KGr_Share_joint", "PlyMewtwo5KGr_Share_matanim_joint"],
];

/// Calls into TUs that are not ported yet. Each is a settable hook; unset it panics, except the
/// ones with a verified port in melee_fighter, which the unset hook calls.
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
    pub static mut IT_8026B3F8: Option<fn(*mut core::ffi::c_void, i32) > = None;
            #[allow(non_snake_case)]
            pub fn it_8026B3F8(article: *mut core::ffi::c_void, kind: i32)  {
                match unsafe { IT_8026B3F8 } {
                    Some(f) => f(article, kind),
                    None => { unsafe { let _ = melee_items::tu::it_26B1::it_8026B3F8(article as _, kind as _); } return; },
                }
            }
    fn ftco_800bffd0_port(fp: *mut Fighter, a: i32, b: i32) -> bool {
        melee_fighter::tu::ftcolanim::ftco_800bffd0(unsafe { &mut *fp }, a, b)
    }
    hook_or!(FTCO_800BFFD0, ftCo_800BFFD0, ftco_800bffd0_port, (fp: *mut Fighter, a: i32, b: i32) -> bool);
}

#[inline(always)]
unsafe fn fpp(gobj: *mut HSD_GObj) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}

pub use crate::tu::ftfox::{copy_attrs, push_attrs};

pub fn ftMt_Init_OnLoad(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMt_Init_OnLoad");
    unsafe {
        let fp = fpp(gobj);
        let item_list = (*(*fp).ft_data).x48_items;

        push_attrs::<ftMewtwoAttributes>(fp);

        (*(*fp).parts.add(FtPart_TransN as usize)).anon_8.anon_0.set_flags_b4(1);
        (*fp).set_x2221_b2(1);
        ext::it_8026B3F8(*item_list.add(0), It_Kind_Mewtwo_Disable as i32);
        ext::it_8026B3F8(*item_list.add(1), It_Kind_Mewtwo_ShadowBall as i32);
    }
}

pub fn ftMt_Init_OnItemPickup(_gobj: *mut HSD_GObj, _flag: i32) { melee_core::ct!("ftMt_Init_OnItemPickup");}

/// Callback target of `death2_cb` / `death3_cb`.
pub fn ftMt_Init_OnDeath2(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMt_Init_OnDeath2");
    crate::tu::ftmewtwospeciallw::ftMt_SpecialLw_RemoveDisable(gobj);
    crate::tu::ftmewtwospecialn::ftMt_SpecialN_OnTakeDamage(gobj);
}

/// Callback target of `take_dmg_cb`.
pub fn ftMt_Init_OnTakeDamage(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMt_Init_OnTakeDamage");
    crate::tu::ftmewtwospeciallw::ftMt_SpecialLw_RemoveDisable(gobj);
    crate::tu::ftmewtwospecialn::ftMt_SpecialN_OnDeath(gobj);
}

pub fn ftMt_Init_OnItemDrop(_gobj: *mut HSD_GObj, _flag: i32) { melee_core::ct!("ftMt_Init_OnItemDrop");}

pub fn ftMt_Init_UnkMotionStates4(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMt_Init_UnkMotionStates4");
    unsafe {
        let fp = fpp(gobj);
        let da = (*fp).dat_attrs as *const ftMewtwoAttributes;
        let attr_value = (*da).x0_MEWTWO_SHADOWBALL_CHARGE_CYCLES;
        if ((*fp).u.mt.x2234_shadowBallCharge as f32) == attr_value {
            ext::ftCo_800BFFD0(fp, 92, 0);
        }
    }
}

pub fn ftMt_Init_LoadSpecialAttrs(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMt_Init_LoadSpecialAttrs");
    unsafe {
        let fp = fpp(gobj);
        copy_attrs::<ftMewtwoAttributes>(gobj);
        let sa2 = (*fp).dat_attrs as *mut ftMewtwoAttributes;
        if (*fp).x34_scale.y != 1.0 {
            (*sa2).x80_MEWTWO_DISABLE_OFFSET_X *= (*fp).x34_scale.y;
            (*sa2).x84_MEWTWO_DISABLE_OFFSET_Y *= (*fp).x34_scale.y;
        }
    }
}

/// `ftMt_CostumeList`: zero-initialised BSS array of the C (`UnkCostumeStruct ftMt_CostumeList[4]`), filled when the costumes load.
pub static mut ftMt_CostumeList: [melee_types::UnkCostumeStruct; 4] = unsafe { core::mem::zeroed() };
