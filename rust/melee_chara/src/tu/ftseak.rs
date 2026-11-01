//! src/melee/ft/kinds/ftSeak/ftseak.c: Sheik's init callbacks. No fused sites
//! (tools/rust-port/fma_sites.py), no float math. Not yet tested: no C oracle and no smoke test (unverified).
//!
//! Data: the file-name strings are below; `ftSk_Init_MotionStateTable` (24 rows from motion 341,
//! pointers into the other ftSeak TUs) belongs to the motion-state dispatcher and is TODO until
//! those callbacks are all native, and `ftSk_CostumeList` is an uninitialised 5-entry BSS array.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use crate::stubs as st;
#[allow(unused_imports)]
use melee_types::{Fighter, HSD_GObj};
use melee_types::*;

/// Already ported (rounds 5/6): re-exported under this TU.



pub const ftSk_Init_DatFilename: &str = "PlSk.dat";
pub const ftSk_Init_DataName: &str = "ftDataSeak";
pub const ftSk_Init_AnimDatFilename: &str = "PlSkAJ.dat";
/// `ftSk_Init_DemoMotionFilenames` (result, intro, ending, vi-wait).
pub const ftSk_Init_DemoMotionFilenames: [&str; 4] = [
    "ftDemoResultMotionFileSeak",
    "ftDemoIntroMotionFileSeak",
    "ftDemoEndingMotionFileSeak",
    "ftDemoViWaitMotionFileSeak",
];

/// `ftSk_Init_CostumeStrings`: (dat, joint, matanim joint) per costume.
pub const ftSk_Init_CostumeStrings: [[&str; 3]; 5] = [
    ["PlSkNr.dat", "PlySeak5K_Share_joint", "PlySeak5K_Share_matanim_joint"],
    ["PlSkRe.dat", "PlySeak5KRe_Share_joint", "PlySeak5KRe_Share_matanim_joint"],
    ["PlSkBu.dat", "PlySeak5KBu_Share_joint", "PlySeak5KBu_Share_matanim_joint"],
    ["PlSkGr.dat", "PlySeak5KGr_Share_joint", "PlySeak5KGr_Share_matanim_joint"],
    ["PlSkWh.dat", "PlySeak5KWh_Share_joint", "PlySeak5KWh_Share_matanim_joint"],
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
    pub static mut FTCO_800BFFD0: Option<fn(*mut Fighter, i32, i32)> = None;
    #[allow(non_snake_case)]
    pub fn ftCo_800BFFD0(fp: *mut Fighter, a: i32, b: i32) {
        match unsafe { FTCO_800BFFD0 } {
            Some(f) => f(fp, a, b),
            None => { unsafe { melee_fighter::tu::ftcolanim::ftco_800bffd0(&mut *(fp as *mut _), a as _, b as _); } },
        }
    }
}

#[inline(always)]
unsafe fn fpp(gobj: *mut HSD_GObj) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}

/// `ftSk_Init_80110198` as a callback entry (the take-damage / death2 callback).
pub unsafe extern "C" fn cb_ftSk_Init_80110198(g: *mut HSD_GObj) {
    melee_core::ct!("cb_ftSk_Init_80110198");
    ftSk_Init_80110198(g);
}

pub fn ftSk_Init_OnLoad(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftSk_Init_OnLoad");
    unsafe {
        let fp = fpp(gobj);
        let item_list = (*(*fp).ft_data).x48_items;

        (*fp).set_can_walljump(1);

        crate::tu::ftfox::push_attrs::<ftSeakAttributes>(fp);

        ext::it_8026B3F8(*item_list.add(0), It_Kind_Seak_NeedleThrow as i32);
        ext::it_8026B3F8(*item_list.add(1), It_Kind_Seak_NeedleHeld as i32);
        ext::it_8026B3F8(*item_list.add(2), It_Kind_Seak_Vanish as i32);
        ext::it_8026B3F8(*item_list.add(3), It_Kind_Seak_Chain as i32);
    }
}

pub fn ftSk_Init_80110198(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftSk_Init_80110198");
    crate::tu::ftseakspecialn::ftSk_SpecialN_80111FBC(gobj);
    crate::tu::ftseakspecials::ftSk_SpecialS_CheckAndDestroyChain(gobj);
}

pub fn ftSk_Init_UnkMotionStates4(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftSk_Init_UnkMotionStates4");
    unsafe {
        let fp = fpp(gobj);
        if (*fp).u.sk.x0 == 6 {
            ext::ftCo_800BFFD0(fp, 86, 0);
        }
    }
}

pub fn ftSk_Init_OnItemPickup(gobj: *mut HSD_GObj, flag: i32) {
    melee_core::ct!("ftSk_Init_OnItemPickup");
    crate::tu::ftclink::Fighter_OnItemPickup(gobj, flag, 1, 1);
}

pub fn ftSk_Init_OnItemDrop(gobj: *mut HSD_GObj, flag: i32) {
    melee_core::ct!("ftSk_Init_OnItemDrop");
    crate::tu::ftclink::Fighter_OnItemDrop(gobj, flag, 1, 1);
}

pub fn ftSk_Init_LoadSpecialAttrs(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftSk_Init_LoadSpecialAttrs");
    unsafe { crate::tu::ftfox::copy_attrs::<ftSeakAttributes>(gobj) }
}

/// `ftSk_CostumeList`: zero-initialised BSS array of the C (`UnkCostumeStruct ftSk_CostumeList[5]`), filled when the costumes load.
pub static mut ftSk_CostumeList: [melee_types::UnkCostumeStruct; 5] = unsafe { core::mem::zeroed() };

// ---- folded from chars (consolidation step 20/21) ----
/// `ftSk_Init_OnDeath` (src/melee/ft/kinds/ftSeak/ftseak.c).
pub fn ftSk_Init_OnDeath(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftSk_Init_OnDeath");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        (*fp).u.sk.x0 = 0_i32;
        (*fp).u.sk.x4 = core::mem::zeroed(); /* 0 */
        (*fp).u.sk.x8 = core::mem::zeroed(); /* 0 */
        st::ftParts_80074A4C(gobj, 0, 0);
        st::ftParts_80074A4C(gobj, 1, -1);
    }
}

/// `ftSk_Init_OnKnockbackEnter` (src/melee/ft/kinds/ftSeak/ftseak.c).
pub fn ftSk_Init_OnKnockbackEnter(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftSk_Init_OnKnockbackEnter");
    unsafe {
        st::Fighter_OnKnockbackEnter(gobj, 1);
    }
}

/// `ftSk_Init_OnKnockbackExit` (src/melee/ft/kinds/ftSeak/ftseak.c).
pub fn ftSk_Init_OnKnockbackExit(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftSk_Init_OnKnockbackExit");
    unsafe {
        st::Fighter_OnKnockbackExit(gobj, 1);
    }
}

/// `ftSk_Init_OnItemInvisible` (src/melee/ft/kinds/ftSeak/ftseak.c).
pub fn ftSk_Init_OnItemInvisible(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftSk_Init_OnItemInvisible");
    unsafe {
        st::Fighter_OnItemInvisible(gobj, 1);
    }
}

/// `ftSk_Init_OnItemVisible` (src/melee/ft/kinds/ftSeak/ftseak.c).
pub fn ftSk_Init_OnItemVisible(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftSk_Init_OnItemVisible");
    unsafe {
        st::Fighter_OnItemVisible(gobj, 1);
    }
}
