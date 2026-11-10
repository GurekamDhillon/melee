//! src/melee/ft/kinds/ftKoopa/ftkoopa.c: Bowser's init callbacks (shared with Giga Bowser
//! through `ftKp_Init_OnLoadForGKoopa`) and the SpecialS attribute getters.
//! No float arithmetic, so no fused sites (tools/rust-port/fma_sites.py). Smoke-tested by
//! tests/tu_ftkoopa.rs.
//!
//! Data: the file-name strings are below; `ftKp_Init_MotionStateTable` (the `ftKp_MS_SelfCount`
//! rows of Anim/IASA/Phys/Coll pointers into the ftKoopa TUs) belongs to the motion-state
//! dispatcher and is TODO until the SpecialS callbacks are all native. `ftKp_CostumeList` is
//! zero-initialised BSS.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use crate::chars::da;
#[allow(unused_imports)]
use melee_types::{ftKoopaAttributes, Fighter};
#[allow(unused_imports)]
use melee_types::{HSD_GObj};
use crate::stubs as st;
use crate::tu::ftclink::{Fighter_OnItemDrop, Fighter_OnItemPickup};
use melee_types::*;

/// Already ported (rounds 5/6 and hand-written): re-exported under this TU. The SpecialS getters
/// take the `Fighter` the C reads from the gobj (pure helpers).




pub const ftKp_Init_DatFilename: &str = "PlKp.dat";
pub const ftKp_Init_DataName: &str = "ftDataKoopa";
pub const ftKp_Init_AnimDatFilename: &str = "PlKpAJ.dat";
/// `ftKp_Init_DemoMotionFilenames` (result, intro, ending, vi-wait).
pub const ftKp_Init_DemoMotionFilenames: [&str; 4] = [
    "ftDemoResultMotionFileKoopa",
    "ftDemoIntroMotionFileKoopa",
    "ftDemoEndingMotionFileKoopa",
    "ftDemoViWaitMotionFileKoopa",
];
/// `ftKp_Init_CostumeStrings`: (dat, joint, matanim joint) per costume.
pub const ftKp_Init_CostumeStrings: [[&str; 3]; 4] = [
    ["PlKpNr.dat", "PlyKoopa5K_Share_joint", "PlyKoopa5K_Share_matanim_joint"],
    ["PlKpRe.dat", "PlyKoopa5KRe_Share_joint", "PlyKoopa5KRe_Share_matanim_joint"],
    ["PlKpBu.dat", "PlyKoopa5KBu_Share_joint", "PlyKoopa5KBu_Share_matanim_joint"],
    ["PlKpBk.dat", "PlyKoopa5KBk_Share_joint", "PlyKoopa5KBk_Share_matanim_joint"],
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
                    None => melee_core::unported!(concat!("TODO: ", stringify!($name), " is not ported")),
                }
            }
        };
    }
    pub static mut IT_8026B3F8: Option<fn(*mut c_void, i32) > = None;
            #[allow(non_snake_case)]
            pub fn it_8026B3F8(article: *mut c_void, kind: i32)  {
                match unsafe { IT_8026B3F8 } {
                    Some(f) => f(article, kind),
                    None => { unsafe { melee_items::tu::it_26B1::it_8026B3F8(article as _, kind as _); } return; },
                }
            }
    pub static mut FTCO_800DE2A8: Option<fn(*mut HSD_GObj, *mut HSD_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn ftCo_800DE2A8(gobj: *mut HSD_GObj, vic: *mut HSD_GObj)  {
                match unsafe { FTCO_800DE2A8 } {
                    Some(f) => f(gobj, vic),
                    None => { unsafe { melee_fighter::tu::ftCo_Throw::ftCo_800DE2A8(gobj as _, vic as _); } return; },
                }
            }
    pub static mut FTCO_800DE7C0: Option<fn(*mut HSD_GObj, i32, i32)> = None;
    #[allow(non_snake_case)]
    pub fn ftCo_800DE7C0(gobj: *mut HSD_GObj, a: i32, b: i32) {
        match unsafe { FTCO_800DE7C0 } {
            Some(f) => f(gobj, a, b),
            None => { unsafe { melee_fighter::tu::ftCo_Thrown::ftCo_800DE7C0(gobj as _, a as _, (b != 0)); } },
        }
    }
}

#[inline(always)]
unsafe fn fpp(gobj: *mut HSD_GObj) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}

pub fn ftKp_Init_OnDeath(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKp_Init_OnDeath");
    unsafe {
        let fp = fpp(gobj);
        let koopa_attr = (*fp).dat_attrs as *const ftKoopaAttributes;
        st::ftParts_80074A4C(gobj, 0, 0);
        (*fp).dmg.armor0 = (*koopa_attr).x0;
        (*fp).u.kp.x222C = (*koopa_attr).x10;
        (*fp).u.kp.x2230 = (*koopa_attr).x18;
    }
}

pub fn ftKp_Init_UnkMotionStates3(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKp_Init_UnkMotionStates3");
    crate::tu::ftkoopaspecialn::ftKp_SpecialLw_80134D78(gobj);
}

/// `PUSH_ATTRS(fp, ftKoopaAttributes)`: copy ext_attr into the backup, point dat_attrs at it.
pub fn ftKp_Init_OnLoadForGKoopa(fp: *mut Fighter) {
    melee_core::ct!("ftKp_Init_OnLoadForGKoopa");
    unsafe {
        let backup = (*fp).dat_attrs_backup;
        let src = (*(*fp).ft_data).ext_attr as *const ftKoopaAttributes;
        *((*fp).dat_attrs_backup as *mut ftKoopaAttributes) = *src;
        (*fp).dat_attrs = backup;
    }
}

pub fn ftKp_Init_OnLoad(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKp_Init_OnLoad");
    unsafe {
        let fp = fpp(gobj);
        let items = (*(*fp).ft_data).x48_items;
        ftKp_Init_OnLoadForGKoopa(fp);
        ext::it_8026B3F8((*items.add(0)).article as *mut ::core::ffi::c_void, It_Kind_Koopa_Flame as i32);
        (*fp).set_x2226_b1(1);
    }
}

/// `ftKp_Init_80132B38`: the empty damage callback.
pub fn ftKp_Init_80132B38(_gobj: *mut HSD_GObj) { melee_core::ct!("ftKp_Init_80132B38");}

pub fn ftKp_Init_OnItemPickup(gobj: *mut HSD_GObj, flag: i32) {
    melee_core::ct!("ftKp_Init_OnItemPickup");
    Fighter_OnItemPickup(gobj, flag, 1, 1);
}

pub fn ftKp_Init_OnItemDrop(gobj: *mut HSD_GObj, bool1: i32) {
    melee_core::ct!("ftKp_Init_OnItemDrop");
    Fighter_OnItemDrop(gobj, bool1, 1, 1);
}

/// `COPY_ATTRS(gobj, ftKoopaAttributes)`.
pub fn ftKp_Init_LoadSpecialAttrs(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKp_Init_LoadSpecialAttrs");
    unsafe {
        let fp = fpp(gobj);
        *((*fp).dat_attrs as *mut ftKoopaAttributes) = *((*(*fp).ft_data).ext_attr as *const ftKoopaAttributes);
    }
}

/// `ftCheckThrowB4` (ft/inlines.h): test and clear the script's throw flag.
fn ftCheckThrowB4(fp: &mut Fighter) -> bool {
    unsafe {
        if fp.x2210.x0.throw_flags_b4() != 0 {
            fp.x2210.x0.set_throw_flags_b4(0);
            true
        } else {
            false
        }
    }
}

pub fn ftKp_SpecialS_80132E30(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKp_SpecialS_80132E30");
    unsafe {
        let fp = fpp(gobj);
        if ftCheckThrowB4(&mut *fp) {
            (*fp).facing_dir = -(*fp).facing_dir;
            (*fp).mv.kp.specials.xC = 1; // C `unk1.xC`: same word as specials.xC on the GC (unk1 has native-size pointers)
        }
        if (*fp).cmd_vars[0] == 0 {
            return;
        }
        if (*fp).victim_gobj.is_null() {
            return;
        }
        let victim_gobj = (*fp).victim_gobj;
        st::ftCommon_8007E2F4(&mut *fp, 0);
        ext::ftCo_800DE2A8(gobj, victim_gobj);
        ext::ftCo_800DE7C0(victim_gobj, 0, 0);
        (*fp).cmd_vars[0] = 0;
    }
}

/// `ftKp_CostumeList`: zero-initialised BSS array of the C (`UnkCostumeStruct ftKp_CostumeList[4]`), filled when the costumes load.
pub static mut ftKp_CostumeList: [melee_types::UnkCostumeStruct; 4] = unsafe { core::mem::zeroed() };

// ---- folded from chars (consolidation step 20/21) ----
pub fn ftKp_SpecialS_80132DC0(gobj: *mut HSD_GObj) -> f32 {
    melee_core::ct!("ftKp_SpecialS_80132DC0");
    let fp = unsafe { &*((*gobj).user_data as *const Fighter) };
    da::<ftKoopaAttributes>(fp).x4C
}

pub fn ftKp_SpecialS_80132DD0(gobj: *mut HSD_GObj) -> f32 {
    melee_core::ct!("ftKp_SpecialS_80132DD0");
    let fp = unsafe { &*((*gobj).user_data as *const Fighter) };
    da::<ftKoopaAttributes>(fp).x48
}

pub fn ftKp_SpecialS_80132DE0(gobj: *mut HSD_GObj) -> f32 {
    melee_core::ct!("ftKp_SpecialS_80132DE0");
    let fp = unsafe { &*((*gobj).user_data as *const Fighter) };
    da::<ftKoopaAttributes>(fp).x44
}

pub fn ftKp_SpecialS_80132DF0(gobj: *mut HSD_GObj) -> f32 {
    melee_core::ct!("ftKp_SpecialS_80132DF0");
    let fp = unsafe { &*((*gobj).user_data as *const Fighter) };
    da::<ftKoopaAttributes>(fp).x40
}

pub fn ftKp_SpecialS_80132E00(gobj: *mut HSD_GObj) -> f32 {
    melee_core::ct!("ftKp_SpecialS_80132E00");
    let fp = unsafe { &*((*gobj).user_data as *const Fighter) };
    da::<ftKoopaAttributes>(fp).x3C
}

pub fn ftKp_SpecialS_80132E10(gobj: *mut HSD_GObj) -> f32 {
    melee_core::ct!("ftKp_SpecialS_80132E10");
    let fp = unsafe { &*((*gobj).user_data as *const Fighter) };
    da::<ftKoopaAttributes>(fp).x34
}

pub fn ftKp_SpecialS_80132E20(gobj: *mut HSD_GObj) -> f32 {
    melee_core::ct!("ftKp_SpecialS_80132E20");
    let fp = unsafe { &*((*gobj).user_data as *const Fighter) };
    da::<ftKoopaAttributes>(fp).x38
}

/// `ftKp_Init_OnKnockbackEnter` (src/melee/ft/kinds/ftKoopa/ftkoopa.c).
pub fn ftKp_Init_OnKnockbackEnter(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKp_Init_OnKnockbackEnter");
    unsafe {
        st::Fighter_OnKnockbackEnter(gobj, 1 /* true */);
    }
}

/// `ftKp_Init_OnKnockbackExit` (src/melee/ft/kinds/ftKoopa/ftkoopa.c).
pub fn ftKp_Init_OnKnockbackExit(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKp_Init_OnKnockbackExit");
    unsafe {
        st::Fighter_OnKnockbackExit(gobj, 1 /* true */);
    }
}

/// `ftKp_Init_OnItemInvisible` (src/melee/ft/kinds/ftKoopa/ftkoopa.c).
pub fn ftKp_Init_OnItemInvisible(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKp_Init_OnItemInvisible");
    unsafe {
        st::Fighter_OnItemInvisible(gobj, 1 /* true */);
    }
}

/// `ftKp_Init_OnItemVisible` (src/melee/ft/kinds/ftKoopa/ftkoopa.c).
pub fn ftKp_Init_OnItemVisible(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKp_Init_OnItemVisible");
    unsafe {
        st::Fighter_OnItemVisible(gobj, 1 /* true */);
    }
}
