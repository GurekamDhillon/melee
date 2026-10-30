//! src/melee/ft/kinds/ftMasterHand/ftmasterhandthrownmasterhand.c. No float math, no fused
//! sites. Smoke-tested by tests/tu_ftmasterhandthrownmasterhand.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports)]

use crate::stubs as st;
use melee_types::*;

/// Calls into TUs outside this file. Each is a settable hook; unset it falls back to the
/// native port in `melee_fighter`.
pub mod ext {
    use melee_types::*;
    macro_rules! hook_or {
        ($hook:ident, $name:ident, $port:path, ($($a:ident: $t:ty),*) $(-> $r:ty)?) => {
            pub static mut $hook: Option<fn($($t),*) $(-> $r)?> = None;
            #[allow(non_snake_case)]
            pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { $hook } {
                    Some(f) => f($($a),*),
                    None => $port($($a),*),
                }
            }
        };
    }
    hook_or!(FTCO_CAPTURECUT_ENTER, ftCo_CaptureCut_Enter, melee_fighter::tu::ftCo_CaptureCut::ftCo_CaptureCut_Enter, (gobj: *mut HSD_GObj));
}

pub fn ftMh_CaptureWaitMasterHand_80155D1C(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_CaptureWaitMasterHand_80155D1C");
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        (*fp).set_invisible(0);
        (*fp).facing_dir = -(*fp).facing_dir;
        ext::ftCo_CaptureCut_Enter(gobj);
    }
}

pub fn ftMh_CaptureWaitMasterHand_Anim(_gobj: *mut HSD_GObj) { melee_core::ct!("ftMh_CaptureWaitMasterHand_Anim");}

pub fn ftMh_CaptureWaitMasterHand_IASA(_gobj: *mut HSD_GObj) { melee_core::ct!("ftMh_CaptureWaitMasterHand_IASA");}

pub fn ftMh_CaptureWaitMasterHand_Phys(_gobj: *mut HSD_GObj) { melee_core::ct!("ftMh_CaptureWaitMasterHand_Phys");}

pub fn ftMh_CaptureWaitMasterHand_Coll(_gobj: *mut HSD_GObj) { melee_core::ct!("ftMh_CaptureWaitMasterHand_Coll");}

pub fn ftMh_CaptureWaitMasterHand_80155D6C(gobj: *mut HSD_GObj, _arg1: i32) {
    melee_core::ct!("ftMh_CaptureWaitMasterHand_80155D6C");
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        let victim_fp = (*(*fp).victim_gobj).user_data as *mut Fighter;

        (*fp).facing_dir = (*victim_fp).facing_dir;
        (*fp).mv.mh.unk4.x0 = 0;
        st::Fighter_ChangeMotionState(gobj, ftCo_MS_ThrownMasterHand as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        (*fp).set_invisible(0);
        (*fp).accessory1_cb = Some(melee_fighter::tu::ftCo_Thrown::ftCo_800DE508_cb);
        st::ftCommon_8007E2F4(&mut *fp, 511);
        st::ftAnim_8006EBA4(gobj);
    }
}

pub fn ftMh_ThrownMasterHand_Anim(_gobj: *mut HSD_GObj) { melee_core::ct!("ftMh_ThrownMasterHand_Anim");}

pub fn ftMh_ThrownMasterHand_IASA(_gobj: *mut HSD_GObj) { melee_core::ct!("ftMh_ThrownMasterHand_IASA");}

pub fn ftMh_ThrownMasterHand_Phys(_gobj: *mut HSD_GObj) { melee_core::ct!("ftMh_ThrownMasterHand_Phys");}

pub fn ftMh_ThrownMasterHand_Coll(_gobj: *mut HSD_GObj) { melee_core::ct!("ftMh_ThrownMasterHand_Coll");}
