//! src/melee/ft/kinds/ftSandbag/ftsandbag.c: Sandbag's init callbacks and WaitReverse.
//! No fused sites (tools/rust-port/fma_sites.py: 0 fmuladd in every function). The two joint
//! limit angles are compile-time constants: `-M_PI / 2` (a C `double` narrowed to the `float`
//! parameter) and `MTXDegToRad(-86)` (`-86 * 0.01745329252f`, folded in `f32`). Smoke-tested
//! through recording hooks by tests/tu_ftsandbag.rs.
//!
//! Data: the file-name strings are below; `ftSb_Init_MotionStateTable` (one row, motion 341)
//! belongs to the motion-state dispatcher and `ftSb_CostumeList` is an uninitialised BSS entry.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use melee_types::*;

/// Already ported (round 2): re-exported under this TU.
pub use crate::chars::r2::ftSandbag::ftsandbag::{ftSb_WaitReverse_Coll, ftSb_WaitReverse_Phys};

pub const ftSb_Init_DatFilename: &str = "PlSb.dat";
pub const ftSb_Init_DataName: &str = "ftDataSandbag";
pub const ftSb_Init_AnimDatFilename: &str = "PlSbAJ.dat";
/// `ftSb_Init_CostumeStrings`: (dat, joint, matanim joint).
pub const ftSb_Init_CostumeStrings: [[&str; 3]; 1] =
    [["PlSbNr.dat", "PlySandbag_Share_joint", "PlySandbag_Share_matanim_joint"]];

/// `ftSandbagAttributes` (local to the C file): `u32 x0_pair[2]`.
#[derive(Clone, Copy)]
#[repr(C)]
pub struct ftSandbagAttributes {
    pub x0_pair: [u32; 2],
}

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
    pub static mut LB_8000C1C0: Option<fn(*mut HSD_JObj, *mut HSD_JObj) > = None;
            #[allow(non_snake_case)]
            pub fn lb_8000C1C0(jobj: *mut HSD_JObj, constraint: *mut HSD_JObj)  {
                match unsafe { LB_8000C1C0 } {
                    Some(f) => f(jobj, constraint),
                    None => { unsafe { let _ = melee_runtime::tu::lb_00B0::lb_8000C1C0(jobj as _, constraint as _); } return; },
                }
            }
    pub static mut LB_8000C228: Option<fn(*mut HSD_JObj, *mut HSD_JObj) > = None;
            #[allow(non_snake_case)]
            pub fn lb_8000C228(jobj: *mut HSD_JObj, constraint: *mut HSD_JObj)  {
                match unsafe { LB_8000C228 } {
                    Some(f) => f(jobj, constraint),
                    None => { unsafe { let _ = melee_runtime::tu::lb_00B0::lb_8000C228(jobj as _, constraint as _); } return; },
                }
            }
    pub static mut LB_8000C420: Option<fn(*mut HSD_JObj, u32, f32) > = None;
            #[allow(non_snake_case)]
            pub fn lb_8000C420(jobj: *mut HSD_JObj, flags: u32, limit: f32)  {
                match unsafe { LB_8000C420 } {
                    Some(f) => f(jobj, flags, limit),
                    None => { unsafe { let _ = melee_runtime::tu::lb_00B0::lb_8000C420(jobj as _, flags as _, limit as _); } return; },
                }
            }
}

#[inline(always)]
unsafe fn fpp(gobj: *mut HSD_GObj) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}

pub fn ftSb_Init_OnDeath(_gobj: *mut HSD_GObj) { melee_core::ct!("ftSb_Init_OnDeath");}

pub fn ftSb_Init_OnLoad(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftSb_Init_OnLoad");
    unsafe {
        let fp = fpp(gobj);
        crate::tu::ftfox::push_attrs::<ftSandbagAttributes>(fp);
        (*fp).set_is_sandbag(1);
        (*fp).set_x2226_b0(1);
        ftSb_Init_8014FA30(fp);
    }
}

pub fn ftSb_Init_8014FA30(fp: *mut Fighter) {
    melee_core::ct!("ftSb_Init_8014FA30");
    unsafe {
        let p = |part: Fighter_Part| (*fp).parts.add(part as usize);
        ext::lb_8000C1C0((*p(FtPart_WaistN)).joint, (*p(FtPart_RLegJ)).joint);
        ext::lb_8000C1C0((*p(FtPart_WaistN)).x4_jobj2, (*p(FtPart_RLegJ)).x4_jobj2);
        ext::lb_8000C1C0((*p(FtPart_WaistN)).joint, (*p(FtPart_LShoulderN)).joint);
        ext::lb_8000C1C0((*p(FtPart_WaistN)).x4_jobj2, (*p(FtPart_LShoulderN)).x4_jobj2);
        ext::lb_8000C228((*p(FtPart_LLegJ)).joint, (*p(FtPart_RShoulderJ)).joint);
        ext::lb_8000C228((*p(FtPart_LLegJ)).x4_jobj2, (*p(FtPart_RShoulderJ)).x4_jobj2);

        let half_pi = (-core::f64::consts::PI / 2.0) as f32;
        ext::lb_8000C420((*p(FtPart_LLegJ)).joint, 1, half_pi);
        ext::lb_8000C420((*p(FtPart_LLegJ)).joint, 2, half_pi);
        ext::lb_8000C420((*p(FtPart_LLegJ)).x4_jobj2, 1, half_pi);
        ext::lb_8000C420((*p(FtPart_LLegJ)).x4_jobj2, 2, half_pi);

        ext::lb_8000C228((*p(FtPart_LLegJA)).joint, (*p(FtPart_WaistN)).joint);
        ext::lb_8000C228((*p(FtPart_LLegJA)).x4_jobj2, (*p(FtPart_WaistN)).x4_jobj2);

        // MTXDegToRad(-86) = -86 * 0.01745329252f
        let angle: f32 = -86.0_f32 * 0.01745329252_f32;
        ext::lb_8000C420((*p(FtPart_LLegJA)).joint, 1, angle);
        ext::lb_8000C420((*p(FtPart_LLegJA)).joint, 2, angle);
        ext::lb_8000C420((*p(FtPart_LLegJA)).x4_jobj2, 1, angle);
        ext::lb_8000C420((*p(FtPart_LLegJA)).x4_jobj2, 2, angle);
    }
}

pub fn ftSb_Init_LoadSpecialAttrs(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftSb_Init_LoadSpecialAttrs");
    unsafe { crate::tu::ftfox::copy_attrs::<ftSandbagAttributes>(gobj) }
}

pub fn ftSb_Init_OnKnockbackEnter(_gobj: *mut HSD_GObj) { melee_core::ct!("ftSb_Init_OnKnockbackEnter");}

pub fn ftSb_Init_OnKnockbackExit(_gobj: *mut HSD_GObj) { melee_core::ct!("ftSb_Init_OnKnockbackExit");}

pub fn ftSb_Init_8014FBA4(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftSb_Init_8014FBA4");
    unsafe {
        let fp = fpp(gobj);
        if (*fp).ground_or_air == GA_Air {
            crate::stubs::ftCommon_8007D7FC(&mut *fp);
        }
        crate::stubs::Fighter_ChangeMotionState(gobj, 0x155, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        melee_fighter::ftcommon2::ftcommon_8007efc0(&mut *fp, melee_fighter::hooks::common_data().x5F0);
    }
}

pub fn ftSb_WaitReverse_Anim(_gobj: *mut HSD_GObj) { melee_core::ct!("ftSb_WaitReverse_Anim");}

pub fn ftSb_WaitReverse_IASA(_gobj: *mut HSD_GObj) { melee_core::ct!("ftSb_WaitReverse_IASA");}

/// `ftSb_CostumeList`: zero-initialised BSS array of the C (`UnkCostumeStruct ftSb_CostumeList[1]`), filled when the costumes load.
pub static mut ftSb_CostumeList: [melee_types::UnkCostumeStruct; 1] = unsafe { core::mem::zeroed() };
