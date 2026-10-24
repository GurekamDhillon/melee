//! src/melee/ft/kinds/ftPopo/ftpopo.c: Ice Climbers' (Popo; Nana shares them) init callbacks. No
//! float arithmetic, so no fused sites (tools/rust-port/fma_sites.py). Smoke-tested by
//! tests/tu_ftpopo.rs.
//!
//! Data: the file-name strings are below; `ftPp_Init_MotionStateTable` (29 rows of
//! Anim/IASA/Phys/Coll pointers into the ftPopo special TUs) belongs to the motion-state dispatcher
//! and is TODO. `ftPp_CostumeList` is zero-initialised BSS.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use crate::stubs as st;
use melee_types::*;

/// Already ported (rounds 5/6): re-exported under this TU.
pub use crate::chars::r5::ftPopo::ftpopo::{ftPp_Init_OnKnockbackEnter, ftPp_Init_OnKnockbackExit};
pub use crate::chars::r6::ftPopo::ftpopo::{ftPp_Init_OnItemInvisible, ftPp_Init_OnItemVisible};
use crate::tu::ftclink::{Fighter_OnItemDrop, Fighter_OnItemPickup};

pub const ftPp_Init_DatFilename: &str = "PlPp.dat";
pub const ftPp_Init_DataName: &str = "ftDataPopo";
pub const ftPp_Init_AnimDatFilename: &str = "PlPpAJ.dat";
/// `ftPp_Init_DemoMotionFilenames` (result, intro, ending, vi-wait).
pub const ftPp_Init_DemoMotionFilenames: [&str; 4] = [
    "ftDemoResultMotionFilePopo",
    "ftDemoIntroMotionFilePopo",
    "ftDemoEndingMotionFilePopo",
    "ftDemoViWaitMotionFilePopo",
];
/// `ftPp_Init_CostumeStrings`: (dat, joint, matanim joint) per costume.
pub const ftPp_Init_CostumeStrings: [[&str; 3]; 4] = [
    ["PlPpNr.dat", "PlyPopo5K_Share_joint", "PlyPopo5K_Share_matanim_joint"],
    ["PlPpGr.dat", "PlyPopo5KGr_Share_joint", "PlyPopo5KGr_Share_matanim_joint"],
    ["PlPpOr.dat", "PlyPopo5KOr_Share_joint", "PlyPopo5KOr_Share_matanim_joint"],
    ["PlPpRe.dat", "PlyPopo5KRe_Share_joint", "PlyPopo5KRe_Share_matanim_joint"],
];

/// Calls into TUs that are not ported yet. Each is a settable hook; unset it panics.
pub mod ext {
    use core::ffi::c_void;
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
    pub static mut IT_8026B3F8: Option<fn(*mut c_void, i32) > = None;
            #[allow(non_snake_case)]
            pub fn it_8026B3F8(article: *mut c_void, kind: i32)  {
                match unsafe { IT_8026B3F8 } {
                    Some(f) => f(article, kind),
                    None => { unsafe { let _ = melee_items::tu::it_26B1::it_8026B3F8(article as _, kind as _); } return; },
                }
            }
    crate::port_hook!(IT_802C17DC, it_802C17DC, crate::adapt::it_802C17DC, (item: *mut Item_GObj));
}

/// Motion flags of the Popo states (`ftPopo/forward.h`), as numeric values of the `Ft_MF_*` ors.
pub mod mf {
    use melee_types::*;
    /// `ftCommon_GroundAirColl_MF`.
    pub const GroundAirColl: u32 = 0x0C4C5080;
    pub const Special: u32 = Ft_MF_SkipModel | Ft_MF_SkipItemVis | Ft_MF_UnkUpdatePhys | Ft_MF_FreezeState;
    pub const SpecialLw: u32 = Special | Ft_MF_KeepColAnimHitStatus;
    pub const SpecialN: u32 = Special | Ft_MF_KeepFastFall | Ft_MF_SkipThrowException;
    pub const SpecialS: u32 = Special | Ft_MF_KeepGfx | Ft_MF_KeepSfx;
    pub const SpecialHi: u32 = SpecialS | Ft_MF_KeepFastFall;
    pub const MS_358: u32 = SpecialLw | Ft_MF_SkipParasol;
    pub const SpecialAirN: u32 = SpecialN | Ft_MF_SkipParasol;
    pub const SpecialAirS: u32 = SpecialS | Ft_MF_SkipParasol;
    pub const SpecialAirHi: u32 = SpecialHi | Ft_MF_SkipParasol;
    pub const SpecialHi_Coll: u32 = GroundAirColl | Ft_MF_KeepGfx | Ft_MF_SkipHit;
    pub const SpecialLw_Coll: u32 = GroundAirColl | Ft_MF_KeepGfx | Ft_MF_KeepSfx;
    pub const SpecialS_Coll: u32 = GroundAirColl | Ft_MF_KeepGfx | Ft_MF_SkipHit | Ft_MF_KeepSfx;
}

type G = *mut HSD_GObj;

#[inline(always)]
unsafe fn fpp(gobj: G) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}

pub fn ftPp_Init_OnItemPickup(gobj: G, flag: i32) {
    Fighter_OnItemPickup(gobj, flag, 1, 1);
}

pub fn ftPp_Init_OnItemDrop(gobj: G, flag: i32) {
    Fighter_OnItemDrop(gobj, flag, 1, 1);
}

/// `PUSH_ATTRS(fp, ftIceClimberAttributes)`.
pub fn ftPp_Init_OnLoadForNana(fp: *mut Fighter) {
    unsafe {
        let backup = (*fp).dat_attrs_backup;
        let src = (*(*fp).ft_data).ext_attr as *const ftIceClimberAttributes;
        *((*fp).dat_attrs_backup as *mut ftIceClimberAttributes) = *src;
        (*fp).dat_attrs = backup;
    }
}

pub fn ftPp_Init_OnLoad(gobj: G) {
    unsafe {
        let fp = fpp(gobj);
        let item_list = (*(*fp).ft_data).x48_items;
        (*fp).set_x2222_b5(1);

        ftPp_Init_OnLoadForNana(fp);

        let da = (*fp).dat_attrs as *const ftIceClimberAttributes;
        (*fp).x40 = (*da).x0;
        ext::it_8026B3F8(*item_list.add(0), It_Kind_IceClimber_Ice as i32);
        ext::it_8026B3F8(*item_list.add(1), It_Kind_IceClimber_Blizzard as i32);
        ext::it_8026B3F8(*item_list.add(2), It_Kind_IceClimber_GumStrings as i32);
    }
}

pub fn ftPp_Init_OnDeath(gobj: G) {
    unsafe {
        let fp = fpp(gobj);
        st::ftParts_80074A4C(gobj, 0, 0);
        st::ftParts_80074A4C(gobj, 1, 0);
        (*fp).u.nn.x2234 = 0;
        (*fp).u.nn.x222C = core::ptr::null_mut();
        (*fp).u.nn.set_x2230_b0(0);
        (*fp).u.nn.x2238 = core::ptr::null_mut();
        (*fp).u.nn.x224C = 0;
        (*fp).u.nn.x2250 = 0.0;
    }
}

pub fn ftPp_Init_8011F060(gobj: G) {
    ftPp_Init_8011F190(gobj);
    crate::tu::ftpopospeciallw::ftPp_SpecialHi_80122898(gobj);
    crate::tu::ftpopospecialhi::ftPp_SpecialS_80121164(gobj);
    crate::tu::ftpopospecials::ftPp_SpecialS_8011F68C(gobj);
}

/// `COPY_ATTRS(gobj, ftIceClimberAttributes)`.
pub fn ftPp_Init_LoadSpecialAttrs(gobj: G) {
    unsafe {
        let fp = fpp(gobj);
        *((*fp).dat_attrs as *mut ftIceClimberAttributes) = *((*(*fp).ft_data).ext_attr as *const ftIceClimberAttributes);
    }
}

pub fn ftPp_Init_8011F16C(gobj: G, item_gobj: *mut Item_GObj) {
    unsafe {
        let fp = fpp(gobj);
        if item_gobj != (*fp).u.nn.x222C {
            return;
        }
        (*fp).u.nn.x222C = core::ptr::null_mut();
        (*fp).death2_cb = None;
        (*fp).take_dmg_cb = None;
    }
}

fn ftPp_Init_8011F190(gobj: G) {
    unsafe {
        let fp = fpp(gobj);
        if (*fp).u.nn.x222C.is_null() {
            return;
        }
        ext::it_802C17DC((*fp).u.nn.x222C);
        ftPp_Init_8011F16C(gobj, (*fp).u.nn.x222C);
    }
}
