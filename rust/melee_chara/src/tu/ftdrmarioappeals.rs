//! src/melee/ft/kinds/ftDrMario/ftdrmarioappeals.c: Dr. Mario's side taunt (the vitamin).
//! No float math (no fused sites). Smoke-tested by tests/tu_ftdrmarioappeals.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use crate::stubs as st;
use crate::tu::ftdrmario::{cb_ftDr_Init_80149540, ftDr_Init_801497CC};
use melee_types::*;

/// Already ported (round 2): re-exported under this TU.
pub use crate::chars::r2::ftDrMario::ftdrmarioappeals::ftDr_AppealS_Phys;

/// Calls into TUs that are not ported yet. Each is a settable hook; unset it panics, except the
/// ones with a verified port in melee_fighter, which the unset hook calls.
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
    pub static mut LB_8000B1CC: Option<fn(*mut HSD_JObj, *mut Vec3, *mut Vec3) > = None;
            #[allow(non_snake_case)]
            pub fn lb_8000B1CC(jobj: *mut HSD_JObj, pos0: *mut Vec3, pos1: *mut Vec3)  {
                match unsafe { LB_8000B1CC } {
                    Some(f) => f(jobj, pos0, pos1),
                    None => { unsafe { melee_runtime::tu::lb_00B0::lb_8000B1CC(jobj as _, pos0 as _, pos1 as _); } return; },
                }
            }
    pub static mut FTMR_SPECIALN_VITAMINRANDOM: Option<fn(*mut HSD_GObj) -> i32> = None;
            #[allow(non_snake_case)]
            pub fn ftMr_SpecialN_VitaminRandom(gobj: *mut HSD_GObj) -> i32 {
                match unsafe { FTMR_SPECIALN_VITAMINRANDOM } {
                    Some(f) => f(gobj),
                    None => { return unsafe { crate::tu::ftmariospecialn::ftMr_SpecialN_VitaminRandom(gobj as _) as _ }; },
                }
            }
    crate::port_hook!(ITDRMARIOPILL_APPEAL_SPAWN, itDrMarioPill_Appeal_Spawn, crate::adapt::itDrMarioPill_Appeal_Spawn,
          (gobj: *mut Item_GObj, pos: *mut Vec3, vitamin_idx: i32, kind: i32, facing_dir: f32) -> *mut Item_GObj);
    pub use super::super::ftdrmario::ext::{itDrMarioPill_802C0DBC, ITDRMARIOPILL_802C0DBC};
    hook_or!(FTCO_APPEALS_IASA, ftCo_AppealS_IASA, melee_fighter::tu::ftCo_AppealS::ftCo_AppealS_IASA, (gobj: *mut HSD_GObj));
    hook_or!(FT_80084280, ft_80084280, melee_fighter::tu::ft_081b::ft_80084280, (gobj: *mut HSD_GObj));
}

pub fn ftDr_AppealS_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftDr_AppealS_Anim");
    unsafe {
        let mut fp = (*gobj).user_data as *mut Fighter;
        let mut sp18 = Vec3::zeroed();
        if (*fp).cmd_vars[0] == 1 && (*fp).u.mr.x2240.is_null() {
            ext::lb_8000B1CC((*(*fp).parts).joint as *mut HSD_JObj, core::ptr::null_mut(), &mut sp18);
            let vitamin_idx = ext::ftMr_SpecialN_VitaminRandom(gobj);
            let tmp = ext::itDrMarioPill_Appeal_Spawn(
                gobj,
                &mut sp18,
                vitamin_idx,
                It_Kind_DrMario_Vitamin as i32,
                (*fp).facing_dir,
            );
            (*fp).u.mr.x2240 = tmp;
            if !tmp.is_null() {
                (*fp).death2_cb = Some(cb_ftDr_Init_80149540);
                (*fp).take_dmg_cb = Some(cb_ftDr_Init_80149540);
            }
        } else if (*fp).cmd_vars[0] == 2 {
            ftDr_Init_801497CC(gobj);
        }
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            if !gobj.is_null() {
                fp = (*gobj).user_data as *mut Fighter;
                if !fp.is_null() && !(*fp).u.mr.x2240.is_null() {
                    ext::itDrMarioPill_802C0DBC((*fp).u.mr.x2240);
                    (*fp).u.mr.x2240 = core::ptr::null_mut();
                }
            }
            if !gobj.is_null() {
                fp = (*gobj).user_data as *mut Fighter;
                if !fp.is_null() {
                    (*fp).take_dmg_cb = None;
                    (*fp).death2_cb = None;
                }
            }
            st::ft_8008A2BC(gobj);
        }
    }
}

pub fn ftDr_AppealS_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftDr_AppealS_IASA");
    ext::ftCo_AppealS_IASA(gobj);
}

pub fn ftDr_AppealS_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftDr_AppealS_Coll");
    ext::ft_80084280(gobj);
}
