//! src/melee/ft/kinds/ftCrazyHand/ftcrazyhand.c: Crazy Hand's init callbacks and file names.
//! No fused sites (only stores of constants / attribute floats). Smoke-tested by
//! tests/tu_ftcrazyhand.rs.
//!
//! Data: the file-name strings are below. `ftCh_Init_MotionStateTable` (45 rows of callbacks
//! into every ftCrazyHand TU plus `ftCamera_UpdateCameraBox`) belongs to the motion-state
//! dispatcher and is TODO until those callbacks are all native. `ftCh_CostumeList` is
//! zero-initialised BSS.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use melee_types::*;

pub const ftCh_Init_DatFilename: &str = "PlCh.dat";
pub const ftCh_Init_DataName: &str = "ftDataCrazyhand";
pub const ftCh_Init_803D4834: &str = "PlChNr.dat";
pub const ftCh_Init_803D4840: &str = "PlyCrazyhand_Share_joint";
pub const ftCh_Init_AnimDatFilename: &str = "PlChAJ.dat";
/// `ftCh_Init_CostumeStrings`: (dat, joint) for the single costume; no matanim joint.
pub const ftCh_Init_CostumeStrings: [[&str; 2]; 1] = [[ftCh_Init_803D4834, ftCh_Init_803D4840]];

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
    /// `ftBossLib_ReportGObjSlotType` (ftbosslib.c): the native `melee_fighter` port when unset
    /// (it needs `Player_GetPlayerSlotType`, which is still a TODO in that crate).
    pub static mut FTBOSSLIB_REPORTGOBJSLOTTYPE: Option<fn(*mut HSD_GObj)> = None;
    #[allow(non_snake_case)]
    pub fn ftBossLib_ReportGObjSlotType(gobj: *mut HSD_GObj) {
        match unsafe { FTBOSSLIB_REPORTGOBJSLOTTYPE } {
            Some(f) => f(gobj),
            None => melee_fighter::tu::ftbosslib::ftBossLib_ReportGObjSlotType(gobj),
        }
    }
}

pub fn ftCh_Init_OnDeath(_gobj: *mut HSD_GObj) { melee_core::ct!("ftCh_Init_OnDeath");}

pub fn ftCh_Init_OnLoad(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_Init_OnLoad");
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        let ftdata = (*fp).ft_data;
        let attr = (*ftdata).ext_attr as *const ftCrazyHand_DatAttrs;
        let items = (*ftdata).x48_items;

        // PUSH_ATTRS(fp, ftCrazyHand_DatAttrs)
        let backup = (*fp).dat_attrs_backup;
        *((*fp).dat_attrs_backup as *mut ftCrazyHand_DatAttrs) = *attr;
        (*fp).dat_attrs = backup;

        ext::ftBossLib_ReportGObjSlotType(gobj);
        ext::it_8026B3F8(*items.add(0), It_Kind_CrazyHand_Laser as i32);
        ext::it_8026B3F8(*items.add(1), It_Kind_CrazyHand_Bullet as i32);
        ext::it_8026B3F8(*items.add(2), It_Kind_CrazyHand_Bomb as i32);
        (*fp).set_no_normal_motion(1);
        (*fp).set_x2229_b6(1);
        (*fp).set_no_kb(1);
        (*fp).set_x222A_b0(1);
        (*fp).set_x222A_b1(1);
        (*fp).set_x2229_b3(1);
        (*fp).cur_pos.x = (*attr).x18;
        (*fp).cur_pos.y = (*attr).x1C;
        (*fp).cur_pos.z = 0.0;
        (*fp).mv.ch.unk0.x28 = 0;
        (*fp).mv.ch.unk0.x2C = 0;
        (*fp).mv.ch.unk0.x30 = 0;
        (*fp).mv.ch.unk0.x34 = 0;
        // the C's int stores above are the grab item slots (grab.x28..x34 share their GC offsets); natively the slots
        // have their own place (melee_types gen/native_unions.py): NULL them as the stores did
        (*fp).mv.ch.grab.x28 = core::ptr::null_mut();
        (*fp).mv.ch.grab.x2C = core::ptr::null_mut();
        (*fp).mv.ch.grab.x30 = core::ptr::null_mut();
        (*fp).mv.ch.grab.x34 = core::ptr::null_mut();
        (*fp).mv.ch.unk0.x38 = -1;
        (*fp).mv.ch.unk0.x3C = -1;
        (*fp).mv.ch.unk0.x40 = -1;
        (*fp).mv.ch.unk0.x1C = 0.0;
        (*fp).mv.ch.unk0.x20 = 0;
        let pos = (*fp).cur_pos;
        (*fp).u.ch.x222C = melee_fighter::tu::ftbosslib::ftBossLib_8015C244(gobj, &pos);
        (*fp).u.ch.x2238 = 1.0;
        (*fp).u.ch.x224C = 0;
        (*fp).u.ch.x2250 = ftMh_MS_Damage2 as i32;
        (*fp).u.ch.x2254 = 0;
        (*fp).cpu.level = 1;
        let level = (*fp).cpu.level;
        let x2238 = (*fp).u.ch.x2238;
        melee_fighter::tu::ftbosslib::ftBossLib_8015BD24(
            level,
            &mut (*fp).u.mh.x223C,
            x2238,
            (*attr).x0,
            (*attr).x8,
            (*attr).x4,
        );
    }
}

/// `COPY_ATTRS(gobj, ftCrazyHand_DatAttrs)`.
pub fn ftCh_Init_LoadSpecialAttrs(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_Init_LoadSpecialAttrs");
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        *((*fp).dat_attrs as *mut ftCrazyHand_DatAttrs) = *((*(*fp).ft_data).ext_attr as *const ftCrazyHand_DatAttrs);
    }
}

/// `ftCh_CostumeList`: zero-initialised BSS array of the C (`UnkCostumeStruct ftCh_CostumeList[1]`), filled when the costumes load.
pub static mut ftCh_CostumeList: [melee_types::UnkCostumeStruct; 1] = unsafe { core::mem::zeroed() };
