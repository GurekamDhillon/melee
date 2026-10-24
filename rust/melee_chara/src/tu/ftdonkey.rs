//! src/melee/ft/kinds/ftDonkey/ftdonkey.c: Donkey Kong's init callbacks, file names and the
//! Hand Slap hitbox placement. Float math: `ftDk_Init_8010DB3C` has `x68 * i - x68 * 1.5f`
//! (one fused site: `fmuladd(x68, i, -(x68 * 1.5))` = `fmsubs`) and `temp_f5 + temp_f3` /
//! `x6C * facing_dir` as separate statements (no fusion). Smoke-tested by
//! tests/tu_ftdonkey.rs.
//!
//! Data: the file-name strings are below. `ftDk_Init_MotionStateTable` (47 rows of callbacks
//! into every ftDonkey TU plus `ftCamera_UpdateCameraBox`) belongs to the motion-state
//! dispatcher and is TODO until those callbacks are all native. `ftDk_CostumeList` is
//! zero-initialised BSS.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use crate::tu::ftclink::{Fighter_OnItemDrop, Fighter_OnItemPickup};
use melee_core::fp::fmsubs;
use melee_types::*;

/// Already ported (rounds 5/6): re-exported under this TU.
pub use crate::chars::r5::ftDonkey::ftdonkey::{
    ftDk_Init_OnDeath, ftDk_Init_OnKnockbackEnter, ftDk_Init_OnKnockbackExit,
};
pub use crate::chars::r6::ftDonkey::ftdonkey::{
    ftDk_Init_8010D774, ftDk_Init_OnItemInvisible, ftDk_Init_OnItemVisible,
};

pub const ftDk_Init_DatFilename: &str = "PlDk.dat";
pub const ftDk_Init_DataName: &str = "ftDataDonkey";
pub const ftDk_Init_803CBE14: &str = "PlDkNr.dat";
pub const ftDk_Init_803CBE20: &str = "PlyDonkey5K_Share_joint";
pub const ftDk_Init_803CBE38: &str = "PlyDonkey5K_Share_matanim_joint";
pub const ftDk_Init_803CBE58: &str = "PlDkBk.dat";
pub const ftDk_Init_803CBE64: &str = "PlyDonkey5KBk_Share_joint";
pub const ftDk_Init_803CBE80: &str = "PlyDonkey5KBk_Share_matanim_joint";
pub const ftDk_Init_803CBEA4: &str = "PlDkRe.dat";
pub const ftDk_Init_803CBEB0: &str = "PlyDonkey5KRe_Share_joint";
pub const ftDk_Init_803CBECC: &str = "PlyDonkey5KRe_Share_matanim_joint";
pub const ftDk_Init_803CBEF0: &str = "PlDkBu.dat";
pub const ftDk_Init_803CBEFC: &str = "PlyDonkey5KBu_Share_joint";
pub const ftDk_Init_803CBF18: &str = "PlyDonkey5KBu_Share_matanim_joint";
pub const ftDk_Init_803CBF3C: &str = "PlDkGr.dat";
pub const ftDk_Init_803CBF48: &str = "PlyDonkey5KGr_Share_joint";
pub const ftDk_Init_803CBF64: &str = "PlyDonkey5KGr_Share_matanim_joint";
pub const ftDk_Init_AnimDatFilename: &str = "PlDkAJ.dat";

/// `ftDk_Init_DemoMotionFilenames` (result, intro, ending, viwait).
pub const ftDk_Init_DemoMotionFilenames: [&str; 4] = [
    "ftDemoResultMotionFileDonkey",
    "ftDemoIntroMotionFileDonkey",
    "ftDemoEndingMotionFileDonkey",
    "ftDemoViWaitMotionFileDonkey",
];

/// `ftDk_Init_CostumeStrings`: (dat, joint, matanim joint) per costume.
pub const ftDk_Init_CostumeStrings: [[&str; 3]; 5] = [
    [ftDk_Init_803CBE14, ftDk_Init_803CBE20, ftDk_Init_803CBE38],
    [ftDk_Init_803CBE58, ftDk_Init_803CBE64, ftDk_Init_803CBE80],
    [ftDk_Init_803CBEA4, ftDk_Init_803CBEB0, ftDk_Init_803CBECC],
    [ftDk_Init_803CBEF0, ftDk_Init_803CBEFC, ftDk_Init_803CBF18],
    [ftDk_Init_803CBF3C, ftDk_Init_803CBF48, ftDk_Init_803CBF64],
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
    // `ftCo_800BFFD0(fp, a, b)` (ft/ft_0C88.c).
    pub static mut FTCO_800BFFD0: Option<fn(*mut Fighter, u32, u32)> = None;
    #[allow(non_snake_case)]
    pub fn ftCo_800BFFD0(fp: *mut Fighter, a: u32, b: u32) {
        match unsafe { FTCO_800BFFD0 } {
            Some(f) => f(fp, a, b),
            None => { unsafe { let _ = melee_fighter::tu::ftcolanim::ftco_800bffd0(&mut *(fp as *mut _), a as _, b as _); } },
        }
    }
    // `mpLib_80056C54(line_id, pos, line_id_out, vec_out, flags_out, normal_out, f1, f2)`
    // (mp/mplib.c); returns the C `bool`.
    pub static mut MPLIB_80056C54: Option<fn(i32, *const Vec3, *mut i32, *mut Vec3, *mut u32, *mut Vec3, f32, f32) -> i32> = None;
    #[allow(non_snake_case)]
    pub fn mpLib_80056C54(line_id: i32, pos: *const Vec3, line_id_out: *mut i32, vec_out: *mut Vec3, flags_out: *mut u32, normal_out: *mut Vec3, f1: f32, f2: f32) -> i32 {
        match unsafe { MPLIB_80056C54 } {
            Some(f) => f(line_id, pos, line_id_out, vec_out, flags_out, normal_out, f1, f2),
            None => { unsafe { melee_world::tu::mplib::mpLib_80056C54(line_id as _, pos as _, line_id_out as _, vec_out as _, flags_out as _, normal_out as _, f1 as _, f2 as _) as _ } },
        }
    }
}

/// `Collide_FloorMask` (mp/forward.h): `Collide_FloorPush | Collide_FloorHug`.
pub const Collide_FloorMask: i32 = 0x8000 | 0x10000;

pub fn ftDk_Init_OnItemPickup(gobj: *mut HSD_GObj, flag: i32) {
    Fighter_OnItemPickup(gobj, flag, 1, 1);
}

pub fn ftDk_Init_OnItemDrop(gobj: *mut HSD_GObj, arg1: i32) {
    Fighter_OnItemDrop(gobj, arg1, 1, 1);
}

pub fn ftDk_Init_UnkMotionStates4(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        let da = (*fp).dat_attrs as *const ftDonkeyAttributes;
        if (*fp).u.dk.x222C == (*da).SpecialN.x2C_MAX_ARM_SWINGS {
            ext::ftCo_800BFFD0(fp, 57, 0);
        }
    }
}

pub fn ftDk_Init_OnLoad(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        let ftdata = (*fp).ft_data;
        let ft_attr = (*ftdata).ext_attr as *mut ftDonkeyAttributes;
        // lbAnim_8001E8F8(tree): the tree's frame count, 0.0 for NULL.
        let frames = |tree: *mut FigaTree| if tree.is_null() { 0.0 } else { (*tree).frames };
        (*ft_attr).x8 = frames(melee_fighter::tu::ftdata::ftData_80085E50(fp, 296));
        (*ft_attr).xC = frames(melee_fighter::tu::ftdata::ftData_80085E50(fp, 297));
        (*ft_attr).x10 = frames(melee_fighter::tu::ftdata::ftData_80085E50(fp, 298));

        // PUSH_ATTRS(fp, ftDonkeyAttributes)
        let backup = (*fp).dat_attrs_backup;
        *((*fp).dat_attrs_backup as *mut ftDonkeyAttributes) = *((*ftdata).ext_attr as *const ftDonkeyAttributes);
        (*fp).dat_attrs = backup;

        (*fp).set_x2222_b0(1);
        (*fp).x2CC = (*fp).dat_attrs as *mut ftDonkeyAttributes;
    }
}

/// `COPY_ATTRS(gobj, ftDonkeyAttributes)`.
pub fn ftDk_Init_LoadSpecialAttrs(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        *((*fp).dat_attrs as *mut ftDonkeyAttributes) = *((*(*fp).ft_data).ext_attr as *const ftDonkeyAttributes);
    }
}

/// `ftCheckThrowB3(fp)` (ft/inlines.h, static inline): test and clear `throw_flags_b3`.
unsafe fn ftCheckThrowB3(fp: *mut Fighter) -> bool {
    if (*fp).anon_2210.anon_0.throw_flags_b3() != 0 {
        (*fp).anon_2210.anon_0.set_throw_flags_b3(0);
        true
    } else {
        false
    }
}

pub fn ftDk_Init_8010DB3C(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        let donkey_attr = (*fp).dat_attrs as *const ftDonkeyAttributes;
        let colldata = &mut (*fp).coll_data as *mut CollData;
        if ftCheckThrowB3(fp) && ((*colldata).env_flags & Collide_FloorMask) != 0 {
            let mut vec_list = [Vec3::zeroed(); 4];
            let mut i = 0;
            while i < 4 {
                let x68 = (*donkey_attr).SpecialLw.x68;
                let temp_f5 = fmsubs(x68, i as f32, x68 * 1.5);
                let temp_f3 = (*donkey_attr).SpecialLw.x6C * (*fp).facing_dir;
                let temp_f6 = temp_f5 + temp_f3;

                if ext::mpLib_80056C54(
                    (*colldata).floor.index,
                    &(*fp).cur_pos,
                    core::ptr::null_mut(),
                    &mut vec_list[i],
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    temp_f6,
                    (*donkey_attr).SpecialLw.x70,
                ) == 0
                {
                    vec_list[i] = (*fp).cur_pos;
                }

                vec_list[i].y += 2.0;
                melee_fighter::ft0df::ftcoll_8007b8a8(&mut (*fp).x914[i], &vec_list[i]);
                i += 1;
            }
        }
    }
}
