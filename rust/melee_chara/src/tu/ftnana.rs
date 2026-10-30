//! src/melee/ft/kinds/ftNana/ftnana.c: Nana's init callbacks. No fused sites
//! (tools/rust-port/fma_sites.py reports 0 for every function). Smoke-tested through recording
//! hooks by tests/tu_ftnana.rs.
//!
//! Data: the file-name strings are below. `ftNn_Init_MotionStateTable` (25 rows of
//! Anim/IASA/Phys/Coll pointers shared with ftPopo, from motion 341) belongs to the
//! motion-state dispatcher and is TODO until those callbacks are all native, and
//! `ftNn_CostumeList` is an uninitialised 4-entry BSS array filled at load.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use crate::stubs as st;
use melee_types::*;

pub use crate::tu::ftfox::{copy_attrs, push_attrs};

pub const ftNn_Init_DatFilename: &str = "PlNn.dat";
pub const ftNn_Init_DataName: &str = "ftDataNana";
pub const ftNn_Init_AnimDatFilename: &str = "PlNnAJ.dat";
/// `ftNn_Init_DemoMotionFilenames` (result, intro, ending, vi-wait; the last one really is Popo's).
pub const ftNn_Init_DemoMotionFilenames: [&str; 4] = [
    "ftDemoResultMotionFileNana",
    "ftDemoIntroMotionFileNana",
    "ftDemoEndingMotionFileNana",
    "ftDemoViWaitMotionFilePopo",
];
/// `ftNn_Init_CostumeStrings`: (dat, joint, matanim joint) per costume.
pub const ftNn_Init_CostumeStrings: [[&str; 3]; 4] = [
    ["PlNnNr.dat", "PlyNana5K_Share_joint", "PlyNana5K_Share_matanim_joint"],
    ["PlNnYe.dat", "PlyNana5KYe_Share_joint", "PlyNana5KYe_Share_matanim_joint"],
    ["PlNnAq.dat", "PlyNana5KAq_Share_joint", "PlyNana5KAq_Share_matanim_joint"],
    ["PlNnWh.dat", "PlyNana5KWh_Share_joint", "PlyNana5KWh_Share_matanim_joint"],
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
    pub static mut FTPP_INIT_ONLOADFORNANA: Option<fn(*mut Fighter) > = None;
            #[allow(non_snake_case)]
            pub fn ftPp_Init_OnLoadForNana(fp: *mut Fighter)  {
                match unsafe { FTPP_INIT_ONLOADFORNANA } {
                    Some(f) => f(fp),
                    None => { unsafe { let _ = crate::tu::ftpopo::ftPp_Init_OnLoadForNana(fp as _); } return; },
                }
            }
}

#[inline(always)]
unsafe fn fpp(gobj: *mut HSD_GObj) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}

pub fn ftNn_Init_OnLoad(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNn_Init_OnLoad");
    unsafe {
        let fp = fpp(gobj);
        (*fp).set_x2222_b4(1);
        ext::ftPp_Init_OnLoadForNana(fp);
        let da = (*fp).dat_attrs as *const ftIceClimberAttributes;
        (*fp).x40 = (*da).xC4;
    }
}

pub fn ftNn_Init_OnDeath(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNn_Init_OnDeath");
    unsafe {
        let fp = fpp(gobj);
        let da = (*fp).dat_attrs as *const ftIceClimberAttributes;
        (*fp).dmg.armor0 = (*da).xC8;
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

pub fn ftNn_Init_80122FAC(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNn_Init_80122FAC");
    crate::tu::ftnanaspecials::ftNn_Init_801238E4(gobj);
}

pub fn ftNn_Init_LoadSpecialAttrs(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNn_Init_LoadSpecialAttrs");
    unsafe { copy_attrs::<ftIceClimberAttributes>(gobj) }
}

pub fn ftNn_Init_8012300C(gobj: *mut HSD_GObj) -> bool {
    melee_core::ct!("ftNn_Init_8012300C");
    unsafe {
        let fp = fpp(gobj);
        if (*fp).x221F_b3() == 0 && (*fp).x2219_b5() == 0 {
            let ret = match (*fp).x2070.anon_0.x2071_b0_3() {
                1 | 3 | 4 | 5 | 6 | 7 | 8 | 10 | 11 | 12 | 13 => true,
                _ => false,
            };
            if !ret {
                crate::tu::ftnanaspecialhi::ftNn_Init_801232A4(gobj);
                return true;
            }
        }
        false
    }
}

pub fn ftNn_Init_8012309C(gobj: *mut HSD_GObj) -> bool {
    melee_core::ct!("ftNn_Init_8012309C");
    unsafe { crate::chars::ftNana::ftnana::ftNn_Init_8012309C(&*fpp(gobj)) }
}
