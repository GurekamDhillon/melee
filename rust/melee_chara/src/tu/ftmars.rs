//! src/melee/ft/kinds/ftMars/ftmars.c: Marth / Roy init callbacks, file names and the two
//! neutral-B effect spawners. No float arithmetic, so no fused sites
//! (tools/rust-port/fma_sites.py: 0 fmuladd). Smoke-tested by tests/tu_ftmars.rs.
//!
//! Data: the file-name strings are below. `ftMs_Init_MotionStateTable` (29 rows of callbacks into
//! the ftMars special TUs plus `ftCamera_UpdateCameraBox`) belongs to the motion-state
//! dispatcher and is TODO until those callbacks are all native. `ftMs_CostumeList` is
//! zero-initialised BSS.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use crate::stubs as st;
#[allow(unused_imports)]
use melee_types::{Fighter, HSD_GObj};
use melee_fighter::hooks as fh;
use melee_types::*;

/// Already ported (rounds 4/5/6): re-exported under this TU.



use crate::tu::ftclink::{Fighter_OnItemDrop, Fighter_OnItemPickup};

/// Calls into TUs that are not ported yet (same hook as ftcaptainspecialn's).
pub mod ext {
    pub use crate::tu::ftcaptainspecialn::ext::{efSync_Spawn, EFSYNC_SPAWN};
}

pub const ftMs_Init_DatFilename: &str = "PlMs.dat";
pub const ftMs_Init_DataName: &str = "ftDataMars";
pub const ftMs_Init_803CF838: &str = "PlMsNr.dat";
pub const ftMs_Init_803CF844: &str = "PlyMars5K_Share_joint";
pub const ftMs_Init_803CF85C: &str = "PlyMars5K_Share_matanim_joint";
pub const ftMs_Init_803CF87C: &str = "PlMsRe.dat";
pub const ftMs_Init_803CF888: &str = "PlyMars5KRe_Share_joint";
pub const ftMs_Init_803CF8A0: &str = "PlyMars5KRe_Share_matanim_joint";
pub const ftMs_Init_803CF8C0: &str = "PlMsGr.dat";
pub const ftMs_Init_803CF8CC: &str = "PlyMars5KGr_Share_joint";
pub const ftMs_Init_803CF8E4: &str = "PlyMars5KGr_Share_matanim_joint";
pub const ftMs_Init_803CF904: &str = "PlMsBk.dat";
pub const ftMs_Init_803CF910: &str = "PlyMars5KBk_Share_joint";
pub const ftMs_Init_803CF928: &str = "PlyMars5KBk_Share_matanim_joint";
pub const ftMs_Init_803CF948: &str = "PlMsWh.dat";
pub const ftMs_Init_803CF954: &str = "PlyMars5KWh_Share_joint";
pub const ftMs_Init_803CF96C: &str = "PlyMars5KWh_Share_matanim_joint";
pub const ftMs_Init_AnimDatFilename: &str = "PlMsAJ.dat";
/// `ftMs_Init_DemoMotionFilenames` (result, intro, ending, vi-wait).
pub const ftMs_Init_DemoMotionFilenames: [&str; 4] = [
    "ftDemoResultMotionFileMars",
    "ftDemoIntroMotionFileMars",
    "ftDemoEndingMotionFileMars",
    "ftDemoViWaitMotionFileMars",
];
/// `ftMs_Init_CostumeStrings`: (dat, joint, matanim joint) per costume.
pub const ftMs_Init_CostumeStrings: [[&str; 3]; 5] = [
    [ftMs_Init_803CF838, ftMs_Init_803CF844, ftMs_Init_803CF85C],
    [ftMs_Init_803CF87C, ftMs_Init_803CF888, ftMs_Init_803CF8A0],
    [ftMs_Init_803CF8C0, ftMs_Init_803CF8CC, ftMs_Init_803CF8E4],
    [ftMs_Init_803CF904, ftMs_Init_803CF910, ftMs_Init_803CF928],
    [ftMs_Init_803CF948, ftMs_Init_803CF954, ftMs_Init_803CF96C],
];

#[inline(always)]
unsafe fn fpp(gobj: *mut HSD_GObj) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}

pub fn ftMs_Init_OnItemPickup(gobj: *mut HSD_GObj, flag: i32) {
    melee_core::ct!("ftMs_Init_OnItemPickup");
    Fighter_OnItemPickup(gobj, flag, 0, 1);
}

pub fn ftMs_Init_OnItemDrop(gobj: *mut HSD_GObj, flag: i32) {
    melee_core::ct!("ftMs_Init_OnItemDrop");
    Fighter_OnItemDrop(gobj, flag, 0, 1);
}

/// `PUSH_ATTRS(fp, MarsAttributes)`: copy ext_attr into the backup, point dat_attrs at it.
pub fn ftMs_Init_OnLoadForRoy(fp: *mut Fighter) {
    melee_core::ct!("ftMs_Init_OnLoadForRoy");
    unsafe {
        let backup = (*fp).dat_attrs_backup;
        let src = (*(*fp).ft_data).ext_attr as *const MarsAttributes;
        *((*fp).dat_attrs_backup as *mut MarsAttributes) = *src;
        (*fp).dat_attrs = backup;
    }
}

pub fn ftMs_Init_OnLoad(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMs_Init_OnLoad");
    unsafe {
        let fp = fpp(gobj);
        let backup = (*fp).dat_attrs_backup;
        let src = (*(*fp).ft_data).ext_attr as *const MarsAttributes;
        *((*fp).dat_attrs_backup as *mut MarsAttributes) = *src;
        (*fp).dat_attrs = backup;
    }
}

/// `COPY_ATTRS(gobj, MarsAttributes)`.
pub fn ftMs_Init_LoadSpecialAttrs(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMs_Init_LoadSpecialAttrs");
    unsafe {
        let fp = fpp(gobj);
        *((*fp).dat_attrs as *mut MarsAttributes) = *((*(*fp).ft_data).ext_attr as *const MarsAttributes);
    }
}

/// Shared body of the two neutral-B effect spawners: effect `ids` = (Marth, Roy).
unsafe fn spawn_n_effect(gobj: *mut HSD_GObj, id_mars: i32, id_emblem: i32) {
    let fp = fpp(gobj);
    if (*fp).x2219_b0() == 0 {
        let joint = (*(*fp).parts).joint as *mut core::ffi::c_void;
        match fh::ftlib_get_kind(gobj) as i32 {
            18 => {
                ext::efSync_Spawn(id_mars, gobj, &[joint]);
            }
            26 => {
                ext::efSync_Spawn(id_emblem, gobj, &[joint]);
            }
            _ => {}
        }
        (*fp).set_x2219_b0(1);
    }
    (*fp).pre_hitlag_cb = Some(crate::chars::cb::ext__efLib_PauseAll);
    (*fp).post_hitlag_cb = Some(crate::chars::cb::ext__efLib_ResumeAll);
    (*fp).accessory4_cb = None;
}

/// Accessory-4 callback of the neutral-B ground end.
pub fn ftMs_SpecialN_801365A8(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMs_SpecialN_801365A8");
    unsafe { spawn_n_effect(gobj, 1266, 1297) }
}

/// Accessory-4 callback of the neutral-B air end.
pub fn ftMs_SpecialN_8013666C(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMs_SpecialN_8013666C");
    unsafe { spawn_n_effect(gobj, 1267, 1298) }
}

pub unsafe extern "C" fn cb_ftMs_SpecialN_801365A8(g: *mut HSD_GObj) {
    melee_core::ct!("cb_ftMs_SpecialN_801365A8");
    ftMs_SpecialN_801365A8(g);
}
pub unsafe extern "C" fn cb_ftMs_SpecialN_8013666C(g: *mut HSD_GObj) {
    melee_core::ct!("cb_ftMs_SpecialN_8013666C");
    ftMs_SpecialN_8013666C(g);
}

/// `ftMs_CostumeList`: zero-initialised BSS array of the C (`UnkCostumeStruct ftMs_CostumeList[5]`), filled when the costumes load.
pub static mut ftMs_CostumeList: [melee_types::UnkCostumeStruct; 5] = unsafe { core::mem::zeroed() };

// ---- folded from chars (consolidation step 20/21) ----
/// `ftMs_SpecialN_80136730` (src/melee/ft/kinds/ftMars/ftmars.c).
pub fn ftMs_SpecialN_80136730(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMs_SpecialN_80136730");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        (*fp).cmd_vars[0] = 0_u32;
        (*fp).mv.ms.specialn.cur_frame = 0_i32;
    }
}

/// `ftMs_Init_OnDeath` (src/melee/ft/kinds/ftMars/ftmars.c).
pub fn ftMs_Init_OnDeath(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMs_Init_OnDeath");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        st::ftParts_80074A4C(gobj, 0, 0);
        st::ftParts_80074A4C(gobj, 1, 0);
        (*fp).u.ms.x222C = 0_u32;
    }
}

/// `ftMs_Init_OnKnockbackEnter` (src/melee/ft/kinds/ftMars/ftmars.c).
pub fn ftMs_Init_OnKnockbackEnter(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMs_Init_OnKnockbackEnter");
    unsafe {
        st::Fighter_OnKnockbackEnter(gobj, 1);
    }
}

/// `ftMs_Init_OnKnockbackExit` (src/melee/ft/kinds/ftMars/ftmars.c).
pub fn ftMs_Init_OnKnockbackExit(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMs_Init_OnKnockbackExit");
    unsafe {
        st::Fighter_OnKnockbackExit(gobj, 1);
    }
}

/// `ftMs_Init_OnItemInvisible` (src/melee/ft/kinds/ftMars/ftmars.c).
pub fn ftMs_Init_OnItemInvisible(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMs_Init_OnItemInvisible");
    unsafe {
        st::Fighter_OnItemInvisible(gobj, 0);
    }
}

/// `ftMs_Init_OnItemVisible` (src/melee/ft/kinds/ftMars/ftmars.c).
pub fn ftMs_Init_OnItemVisible(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMs_Init_OnItemVisible");
    unsafe {
        st::Fighter_OnItemVisible(gobj, 0);
    }
}
