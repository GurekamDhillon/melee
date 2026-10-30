//! src/melee/ft/kinds/ftLink/ftlink.c: Link's init callbacks, damage callbacks and the Hylian
//! shield push-back. Fused sites (tools/rust-port/fma_sites.py): none (`ftLk_800EB334` multiplies
//! a returned float by a table entry, no add; `ftCo_80092ED8` is group 3's verified port).
//! Smoke-tested by tests/tu_ftlink.rs.
//!
//! Data: the file-name strings are below; `ftLk_Init_MotionStateTable` (21 rows of
//! Anim/IASA/Phys/Coll pointers into the ftLink TUs and ftCommon) belongs to the motion-state
//! dispatcher and is TODO until those callbacks are all native. `ftLk_CostumeList` is
//! zero-initialised BSS.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use crate::stubs as st;
use crate::tu::ftclink::{Fighter_OnItemDrop, Fighter_OnItemPickup};
use crate::tu::ftlinkspecialn::{ftLk_SpecialN_ProcessFv10, ftLk_SpecialN_ProcessFv14};
use crate::tu::ftlinkspecials::ftLk_SpecialS_RemoveBoomerang1;
use melee_fighter::hooks as fh;
use melee_types::*;

/// Already ported (rounds 5/6): re-exported under this TU.
pub use crate::chars::r5::ftLink::ftlink::{ftLk_Init_OnDeath, ftLk_Init_OnKnockbackEnter, ftLk_Init_OnKnockbackExit};
pub use crate::chars::r6::ftLink::ftlink::{ftLk_Init_OnItemInvisible, ftLk_Init_OnItemVisible};

pub const ftLk_Init_DatFilename: &str = "PlLk.dat";
pub const ftLk_Init_DataName: &str = "ftDataLink";
pub const ftLk_Init_AnimDatFilename: &str = "PlLkAJ.dat";
/// `ftLk_Init_DemoMotionFilenames` (result, intro, ending, vi-wait).
pub const ftLk_Init_DemoMotionFilenames: [&str; 4] = [
    "ftDemoResultMotionFileLink",
    "ftDemoIntroMotionFileLink",
    "ftDemoEndingMotionFileLink",
    "ftDemoViWaitMotionFileLink",
];
/// `ftLk_Init_CostumeStrings`: (dat, joint, matanim joint) per costume.
pub const ftLk_Init_CostumeStrings: [[&str; 3]; 5] = [
    ["PlLkNr.dat", "PlyLink5K_Share_joint", "PlyLink5K_Share_matanim_joint"],
    ["PlLkRe.dat", "PlyLink5KRe_Share_joint", "PlyLink5KRe_Share_matanim_joint"],
    ["PlLkBu.dat", "PlyLink5KBu_Share_joint", "PlyLink5KBu_Share_matanim_joint"],
    ["PlLkBk.dat", "PlyLink5KBk_Share_joint", "PlyLink5KBk_Share_matanim_joint"],
    ["PlLkWh.dat", "PlyLink5KWh_Share_joint", "PlyLink5KWh_Share_matanim_joint"],
];

/// Calls into TUs that are not ported yet. Each is a settable hook; unset it panics (except
/// `ftData_80085E50`, which falls back to group 3's port when the hook is unset).
pub mod ext {
    use melee_types::*;
    /// Shared with ftclink (same C functions).
    pub use crate::tu::ftclink::ext::{
        ftParts_800753D4, it_8026B3F8, lbAnim_8001E8F8, FTPARTS_800753D4, IT_8026B3F8, LBANIM_8001E8F8,
    };
    pub static mut FTDATA_80085E50: Option<fn(fp: *mut Fighter, msid: i32) -> *mut FigaTree> = None;
    /// `ftData_80085E50(fp, msid)` (ft/ftdata.c): group 3's port unless a hook is installed.
    #[allow(non_snake_case)]
    pub fn ftData_80085E50(fp: *mut Fighter, msid: i32) -> *mut FigaTree {
        match unsafe { FTDATA_80085E50 } {
            Some(f) => f(fp, msid),
            None => melee_fighter::tu::ftdata::ftData_80085E50(fp, msid),
        }
    }
}

#[inline(always)]
unsafe fn fpp(gobj: *mut HSD_GObj) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}

/// `Fighter_SetDamageCallback(gobj, ftLk_800EAF58)` (ft/inlines.h): take_dmg_cb and death2_cb.
pub fn Fighter_SetDamageCallback_ftLk_800EAF58(gobj: *mut HSD_GObj) {
    melee_core::ct!("Fighter_SetDamageCallback_ftLk_800EAF58");
    unsafe {
        let fp = fpp(gobj);
        (*fp).take_dmg_cb = Some(cb_ftLk_800EAF58);
        (*fp).death2_cb = Some(cb_ftLk_800EAF58);
    }
}

/// `ftLk_800EAF38` as a Fighter callback entry.
pub unsafe extern "C" fn cb_ftLk_800EAF38(g: *mut HSD_GObj) {
    melee_core::ct!("cb_ftLk_800EAF38");
    ftLk_800EAF38(g);
}

/// `ftLk_800EAF58` as a Fighter callback entry.
pub unsafe extern "C" fn cb_ftLk_800EAF58(g: *mut HSD_GObj) {
    melee_core::ct!("cb_ftLk_800EAF58");
    ftLk_800EAF58(g);
}

/// `ftLk_800EB334` as a Fighter callback entry (the shield-hit callback).
pub unsafe extern "C" fn cb_ftLk_800EB334(g: *mut HSD_GObj) {
    melee_core::ct!("cb_ftLk_800EB334");
    ftLk_800EB334(g);
}

pub fn ftLk_Init_BoomerangExists(gobj: *mut HSD_GObj) -> bool {
    melee_core::ct!("ftLk_Init_BoomerangExists");
    unsafe { !(*fpp(gobj)).u.lk.boomerang_gobj.is_null() }
}

/// `PUSH_ATTRS(fp, ftLk_DatAttrs)`: copy ext_attr into the backup, point dat_attrs at it.
pub fn ftLk_Init_OnLoadForCLink(fp: *mut Fighter) {
    melee_core::ct!("ftLk_Init_OnLoadForCLink");
    unsafe {
        let backup = (*fp).dat_attrs_backup;
        let src = (*(*fp).ft_data).ext_attr as *const ftLk_DatAttrs;
        *((*fp).dat_attrs_backup as *mut ftLk_DatAttrs) = *src;
        (*fp).dat_attrs = backup;
    }
}

pub fn ftLk_Init_OnLoad(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLk_Init_OnLoad");
    unsafe {
        let fp = fpp(gobj);
        let ftdata = (*fp).ft_data;
        let ea = (*ftdata).ext_attr as *mut ftLk_DatAttrs;
        let item_list = (*ftdata).x48_items;
        (*ea).attackairlw_hit_anim_frame_end = ext::lbAnim_8001E8F8(ext::ftData_80085E50(fp, 72));
        ftLk_Init_OnLoadForCLink(fp);
        let da = (*fp).dat_attrs as *const ftLk_DatAttrs;
        ext::it_8026B3F8(*item_list.add(0), (*da).x48);
        ext::it_8026B3F8(*item_list.add(1), (*da).x2C);
        ext::it_8026B3F8(*item_list.add(2), (*da).xBC);
        ext::it_8026B3F8(*item_list.add(3), (*da).xC);
        ext::it_8026B3F8(*item_list.add(4), (*da).x10);
        let t = *fh::fighter_804d6540().add((*fp).kind as usize);
        ext::ftParts_800753D4(fp, (*t).x0, *item_list.add(6));
    }
}

/// Damage callback (death3 of the boomerang): drop a flying boomerang.
pub fn ftLk_800EAF38(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLk_800EAF38");
    ftLk_SpecialS_RemoveBoomerang1(gobj);
}

/// Damage callback: drop the boomerang, the bomb/arrow/bow state and the Young Link milk.
pub fn ftLk_800EAF58(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLk_800EAF58");
    ftLk_SpecialS_RemoveBoomerang1(gobj);
    melee_fighter::tu::ftco_0d8e::ftco_800d94d8(gobj);
    ftLk_SpecialN_ProcessFv10(gobj);
    ftLk_SpecialN_ProcessFv14(gobj);
    crate::tu::ftclink::ftCl_Init_80149268(gobj);
}

pub fn ftLk_Init_OnItemPickupExt(gobj: *mut HSD_GObj, flag: i32) {
    melee_core::ct!("ftLk_Init_OnItemPickupExt");
    unsafe {
        let fp = fpp(gobj);
        if st::itIsHeavy((*fp).item_gobj) == 1 {
            st::ftParts_80074A4C(gobj, 1, 1);
        }
    }
    st::ftParts_80074A4C(gobj, 2, 1);
    ftLk_Init_OnItemPickup(gobj, flag);
}

pub fn ftLk_Init_OnItemDropExt(gobj: *mut HSD_GObj, flag: i32) {
    melee_core::ct!("ftLk_Init_OnItemDropExt");
    unsafe {
        let fp = fpp(gobj);
        if st::itIsHeavy((*fp).item_gobj) == 1 {
            st::ftParts_80074A4C(gobj, 1, 0);
        }
    }
    st::ftParts_80074A4C(gobj, 2, 0);
    ftLk_Init_OnItemDrop(gobj, flag);
}

pub fn ftLk_Init_OnItemPickup(gobj: *mut HSD_GObj, flag: i32) {
    melee_core::ct!("ftLk_Init_OnItemPickup");
    Fighter_OnItemPickup(gobj, flag, 1, 1);
}

pub fn ftLk_Init_OnItemDrop(gobj: *mut HSD_GObj, flag: i32) {
    melee_core::ct!("ftLk_Init_OnItemDrop");
    Fighter_OnItemDrop(gobj, flag, 1, 1);
}

/// `COPY_ATTRS(gobj, ftLk_DatAttrs)`, then the model-scale adjustment of the Spin Attack offset.
pub fn ftLk_Init_LoadSpecialAttrs(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLk_Init_LoadSpecialAttrs");
    unsafe {
        let fp = fpp(gobj);
        let sa2 = (*fp).dat_attrs as *mut ftLk_DatAttrs;
        *sa2 = *((*(*fp).ft_data).ext_attr as *const ftLk_DatAttrs);
        if (*fp).x34_scale.y != 1.0 {
            (*sa2).specialhi_pos_y_offset *= (*fp).x34_scale.y;
        }
    }
}

/// Body of `ftLk_800EB334` up to its sound call: the shield-hit push-back velocity.
pub fn shield_push(fp: &mut Fighter) {
    melee_core::ct!("shield_push");
    let ea = unsafe { &*((*fp.ft_data).ext_attr as *const ftLk_DatAttrs) };
    let resultf = melee_fighter::tu::ftCo_Guard::ftCo_80092ED8(fp.x19A4, ea.xD8);
    fp.gr_vel = resultf * fh::common_data().x294;
    let new_ground_vel = if fp.specialn_facing_dir < 0.0 { fp.gr_vel } else { -fp.gr_vel };
    fp.gr_vel = new_ground_vel;
}

pub fn ftLk_800EB334(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLk_800EB334");
    unsafe {
        let fp = &mut *fpp(gobj);
        shield_push(fp);
        fh::play_sfx(fp, 160106, 127, 64);
    }
}
