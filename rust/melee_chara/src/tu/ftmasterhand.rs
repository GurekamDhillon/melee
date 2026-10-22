//! src/melee/ft/kinds/ftMasterHand/ftmasterhand.c: Master Hand's init callbacks and file names.
//! No fused sites (only stores of constants / attribute values). Smoke-tested by
//! tests/tu_ftmasterhand.rs.
//!
//! Data: the file-name strings are below. `ftMh_Init_MotionStateTable` (49 rows of callbacks
//! into every ftMasterHand TU plus `ftCamera_UpdateCameraBox`) belongs to the motion-state
//! dispatcher and is TODO until those callbacks are all native. `ftMh_CostumeList` is
//! zero-initialised BSS.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use melee_types::*;

/// Calls into TUs that are not ported yet (same hooks as ftcrazyhand's).
pub mod ext {
    pub use crate::tu::ftcrazyhand::ext::{
        ftBossLib_ReportGObjSlotType, it_8026B3F8, FTBOSSLIB_REPORTGOBJSLOTTYPE, IT_8026B3F8,
    };
}

pub const ftMh_Init_DatFilename: &str = "PlMh.dat";
pub const ftMh_Init_DataName: &str = "ftDataMasterhand";
pub const ftMh_Init_803D4090: &str = "PlMhNr.dat";
pub const ftMh_Init_803D409C: &str = "PlyMasterhand_Share_joint";
pub const ftMh_Init_AnimDatFilename: &str = "PlMhAJ.dat";
/// `ftMh_Init_CostumeStrings`: (dat, joint) for the single costume; the matanim joint is NULL.
pub const ftMh_Init_CostumeStrings: [[Option<&str>; 3]; 1] = [[Some(ftMh_Init_803D4090), Some(ftMh_Init_803D409C), None]];

pub fn ftMh_Init_OnDeath(_gobj: *mut HSD_GObj) {}

pub fn ftMh_Init_OnLoad(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        let ftdata = (*fp).ft_data;
        let attr = (*ftdata).ext_attr as *const ftMasterHand_SpecialAttrs;
        let items = (*ftdata).x48_items;

        // PUSH_ATTRS(fp, ftMasterHand_SpecialAttrs)
        let backup = (*fp).dat_attrs_backup;
        *((*fp).dat_attrs_backup as *mut ftMasterHand_SpecialAttrs) = *attr;
        (*fp).dat_attrs = backup;

        ext::ftBossLib_ReportGObjSlotType(gobj);
        ext::it_8026B3F8(*items.add(0), It_Kind_MasterHand_Laser as i32);
        ext::it_8026B3F8(*items.add(1), It_Kind_MasterHand_Bullet as i32);
        (*fp).set_no_normal_motion(1);
        (*fp).set_x2229_b6(1);
        (*fp).set_no_kb(1);
        (*fp).set_x222A_b0(1);
        (*fp).set_x222A_b1(1);
        (*fp).set_x2229_b3(1);
        (*fp).cur_pos.x = (*attr).x30_pos2.x;
        (*fp).cur_pos.y = (*attr).x30_pos2.y;
        (*fp).cur_pos.z = 0.0;
        (*fp).mv.mh.unk0.x34 = 0;
        (*fp).mv.mh.unk0.x38 = 0;
        (*fp).mv.mh.unk0.x3C = 0;
        (*fp).mv.mh.unk0.x40 = 0;
        (*fp).mv.mh.unk0.x28 = -1;
        (*fp).mv.mh.unk0.x2C = -1;
        (*fp).mv.mh.unk0.x30 = -1;
        (*fp).mv.mh.unk0.x1C = 0.0;
        (*fp).mv.mh.unk0.x20 = 0;
        let pos = (*fp).cur_pos;
        (*fp).u.mh.x222C = melee_fighter::tu::ftbosslib::ftBossLib_8015C244(gobj, &pos);
        (*fp).u.mh.x2238 = 1.0;
        (*fp).u.mh.x224C = 0;
        (*fp).u.mh.x2250 = ftMh_MS_SweepLoop as i32;
        (*fp).u.mh.x2254 = 0;
        (*fp).cpu.level = 1;
        let level = (*fp).cpu.level;
        let x2238 = (*fp).u.mh.x2238;
        melee_fighter::tu::ftbosslib::ftBossLib_8015BD24(
            level,
            &mut (*fp).u.mh.x223C,
            x2238,
            (*attr).x18,
            (*attr).x20,
            (*attr).x1C,
        );
    }
}

/// `COPY_ATTRS(gobj, ftMasterHand_SpecialAttrs)`.
pub fn ftMh_Init_LoadSpecialAttrs(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        *((*fp).dat_attrs as *mut ftMasterHand_SpecialAttrs) =
            *((*(*fp).ft_data).ext_attr as *const ftMasterHand_SpecialAttrs);
    }
}
