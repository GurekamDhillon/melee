//! src/melee/ft/kinds/ftKirby/ftkirbycaptureyoshi.c: Kirby caught by Yoshi's egg-lay / tongue
//! (ftCo_CaptureKirbyYoshi). No fused sites (tools/rust-port/fma_sites.py). Stateful only:
//! smoke-tested by tests/tu_ftkirbycaptureyoshi.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use crate::stubs as st;
use melee_types::*;

/// Calls into TUs that are not ported yet. Each is a settable hook; unset it panics.
pub mod ext {
    use melee_types::HSD_GObj;
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
    crate::port_hook!(FTCOLL_8007B62C, ftColl_8007B62C, crate::adapt::ftColl_8007B62C, (gobj: *mut HSD_GObj, v: i32));

    /// `ftCo_ReleaseItemAndVictim(gobj)`: group 3's port when the hook is unset.
    pub static mut FTCO_RELEASEITEMANDVICTIM: Option<fn(*mut HSD_GObj)> = None;
    #[allow(non_snake_case)]
    pub fn ftCo_ReleaseItemAndVictim(gobj: *mut HSD_GObj) {
        match unsafe { FTCO_RELEASEITEMANDVICTIM } {
            Some(f) => f(gobj),
            None => melee_fighter::tu::ftco_800c7590::ftco_release_item_and_victim(gobj),
        }
    }

    /// `ftCo_800DB368(attacker_fp, fp)`: group 3's port when the hook is unset.
    pub static mut FTCO_800DB368: Option<fn(*mut melee_types::Fighter, *mut melee_types::Fighter)> = None;
    #[allow(non_snake_case)]
    pub fn ftCo_800DB368(attacker: *mut melee_types::Fighter, fp: *mut melee_types::Fighter) {
        match unsafe { FTCO_800DB368 } {
            Some(f) => f(attacker, fp),
            None => unsafe { melee_fighter::tu::ftCo_CapturePulled::ftCo_800DB368(&mut *attacker, &mut *fp) },
        }
    }
}

#[inline(always)]
unsafe fn fpp(gobj: *mut HSD_GObj) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}

/// `ftCo_800DB464` as the `HSD_GObjEvent` the C stores in `accessory1_cb`.
pub unsafe extern "C" fn cb_ftCo_800DB464(g: *mut HSD_GObj) {
    melee_core::ct!("cb_ftCo_800DB464");
    melee_fighter::tu::ftCo_CapturePulled::ftCo_800DB464(g);
}

/// `fn_8010A930(gobj, attacker_gobj)`: enter the Yoshi-capture state (motion 0x14B).
pub fn fn_8010A930(gobj: *mut HSD_GObj, attacker_gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_8010A930");
    unsafe {
        let fp = fpp(gobj);
        let attacker_fp = fpp(attacker_gobj);
        ext::ftCo_ReleaseItemAndVictim(gobj);
        (*fp).x1A5C = attacker_gobj;
        (*fp).victim_gobj = attacker_gobj;
        (*fp).x221B.set_x221B_b5(0);
        (*fp).x221B.set_x221B_b7(0);
        (*fp).facing_dir = -(*attacker_fp).facing_dir;
        ext::ftCo_800DB368(attacker_fp, fp);
        st::Fighter_ChangeMotionState(gobj, 0x14B, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        (*fp).accessory1_cb = Some(cb_ftCo_800DB464);
        st::ftCommon_8007D5D4(&mut *fp);
        melee_fighter::ftcommon2::ftcommon_8007e2f4(&mut *fp, 0x1FF);
        st::ftAnim_8006EBA4(gobj);
        melee_fighter::ftcommon2::ftcommon_8007e2fc(&mut *fp);
    }
}

pub fn ftCo_CaptureKirbyYoshi_Anim(_gobj: *mut HSD_GObj) { melee_core::ct!("ftCo_CaptureKirbyYoshi_Anim");}
pub fn ftCo_CaptureKirbyYoshi_IASA(_gobj: *mut HSD_GObj) { melee_core::ct!("ftCo_CaptureKirbyYoshi_IASA");}
pub fn ftCo_CaptureKirbyYoshi_Phys(_gobj: *mut HSD_GObj) { melee_core::ct!("ftCo_CaptureKirbyYoshi_Phys");}
pub fn ftCo_CaptureKirbyYoshi_Coll(_gobj: *mut HSD_GObj) { melee_core::ct!("ftCo_CaptureKirbyYoshi_Coll");}

/// `ftKb_SpecialNYs_8010AA2C`: hide Kirby while he is inside Yoshi's egg, then ftColl_8007B62C(gobj, 2).
pub fn ftKb_SpecialNYs_8010AA2C(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_SpecialNYs_8010AA2C");
    unsafe {
        let fp = fpp(gobj);
        (*fp).set_invisible(1);
        ext::ftColl_8007B62C(gobj, 2);
    }
}
