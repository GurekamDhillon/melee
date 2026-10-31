//! src/melee/ft/kinds/ftPeach/ftpeachfloatattack.c: Peach's aerials while floating.
//! No fused sites (tools/rust-port/fma_sites.py: 0 in every function). Smoke-tested through
//! recording hooks by tests/tu_ftpeachfloatattack.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use crate::stubs as st;
use crate::tu::ftpeachfloat::{ftPe_8011BB6C, ftPe_Float_CheckContinueInput, ftPe_Float_Phys};
use melee_types::*;

/// Already ported (round 5): re-exported under this TU.
pub use crate::chars::r5::ftPeach::ftpeachfloatattack::ftPe_FloatAttackAir_Coll;

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
    fn ftco_800df478_port(fp: *mut Fighter) -> bool {
        melee_fighter::ft0df::ftco_800df478(unsafe { &*fp })
    }
    fn getmsid_port(fp: *mut Fighter) -> i32 {
        melee_fighter::tu::ftCo_AttackAir::ftCo_AttackAir_GetMsidFromCStick(unsafe { &*fp })
    }
    hook_or!(FTCO_800DF478, ftCo_800DF478, ftco_800df478_port, (fp: *mut Fighter) -> bool);
    hook_or!(FTCO_ATTACKAIR_GETMSIDFROMCSTICK, ftCo_AttackAir_GetMsidFromCStick, getmsid_port, (fp: *mut Fighter) -> i32);
    pub static mut IT_8026B30C: Option<fn(*mut Item_GObj) -> i32> = None;
            #[allow(non_snake_case)]
            pub fn it_8026B30C(item: *mut Item_GObj) -> i32 {
                match unsafe { IT_8026B30C } {
                    Some(f) => f(item),
                    None => { return unsafe { melee_items::tu::it_26B1::it_8026B30C(item as _) as _ }; },
                }
            }
    pub static mut FTCO_800CDDA0: Option<fn(*mut HSD_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn ftCo_800CDDA0(gobj: *mut HSD_GObj)  {
                match unsafe { FTCO_800CDDA0 } {
                    Some(f) => f(gobj),
                    None => { unsafe { melee_fighter::tu::ft_0cdd::ftco_800cdda0(gobj as _); } return; },
                }
            }
    pub static mut FTCO_ATTACKAIR_ENTERFROMMSID: Option<fn(*mut HSD_GObj, i32) > = None;
            #[allow(non_snake_case)]
            pub fn ftCo_AttackAir_EnterFromMsid(gobj: *mut HSD_GObj, msid: i32)  {
                match unsafe { FTCO_ATTACKAIR_ENTERFROMMSID } {
                    Some(f) => f(gobj, msid),
                    None => { unsafe { melee_fighter::tu::ftCo_AttackAir::ftCo_AttackAir_EnterFromMsid(gobj as _, msid as _); } return; },
                }
            }
    pub static mut FTCO_80095328: Option<fn(*mut HSD_GObj, *mut bool) -> bool> = None;
            #[allow(non_snake_case)]
            pub fn ftCo_80095328(gobj: *mut HSD_GObj, out: *mut bool) -> bool {
                match unsafe { FTCO_80095328 } {
                    Some(f) => f(gobj, out),
                    None => { return unsafe { melee_fighter::tu::ftCo_ItemThrow::ftCo_80095328(gobj as _, out as _) as _ }; },
                }
            }
    pub static mut FTCO_800CB870: Option<fn(*mut HSD_GObj) -> bool> = None;
            #[allow(non_snake_case)]
            pub fn ftCo_800CB870(gobj: *mut HSD_GObj) -> bool {
                match unsafe { FTCO_800CB870 } {
                    Some(f) => f(gobj),
                    None => { return unsafe { melee_fighter::tu::ftCo_JumpAerial::ftCo_800CB870(gobj as _) as _ }; },
                }
            }
}

#[inline(always)]
unsafe fn fpp(gobj: *mut HSD_GObj) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}

/// `ftCheckThrowB3(fp)` (ft/inlines.h): test and clear `throw_flags_b3`.
unsafe fn ftCheckThrowB3(fp: *mut Fighter) -> bool {
    if (*fp).anon_2210.anon_0.throw_flags_b3() != 0 {
        (*fp).anon_2210.anon_0.set_throw_flags_b3(0);
        true
    } else {
        false
    }
}

pub fn ftPe_8011BE80(gobj: *mut HSD_GObj) -> bool {
    melee_core::ct!("ftPe_8011BE80");
    unsafe {
        let fp = fpp(gobj);
        if ((*fp).input.pressed_buttons & HSD_PAD_A as u32) != 0 || ext::ftCo_800DF478(fp) {
            if (*fp).u.pe.x4 > 0.0 {
                if (ext::ftCo_AttackAir_GetMsidFromCStick(fp) as u32).wrapping_sub(65) <= 1 {
                    if !(*fp).item_gobj.is_null() && ext::it_8026B30C((*fp).item_gobj) == 3 {
                        ext::ftCo_800CDDA0(gobj);
                        return true;
                    }
                }
                ftPe_8011BF34(gobj);
                return true;
            }
        }
        false
    }
}

/// `ftPe_8011BF34` (static).
fn ftPe_8011BF34(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let msid = ext::ftCo_AttackAir_GetMsidFromCStick(fp);
        (*fp).mv.pe.floatattack.x0 = 0;
        ext::ftCo_AttackAir_EnterFromMsid(gobj, msid.wrapping_add(ftCo_MS_CaptureDamageKoopa as i32));
    }
}

pub fn ftPe_FloatAttackAir_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftPe_FloatAttackAir_Anim");
    unsafe {
        let fp = fpp(gobj);
        if (*fp).u.pe.x4 > 0.0 {
            (*fp).u.pe.x4 -= 1.0;
        }
        if ftCheckThrowB3(fp) {
            (*fp).facing_dir = -(*fp).facing_dir;
        }
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            if (*fp).mv.pe.floatattack.x0 != 0 {
                st::ftCo_Fall_Enter(gobj);
            } else {
                ftPe_8011BB6C(gobj, false);
            }
        }
    }
}

pub fn ftPe_FloatAttackAir_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftPe_FloatAttackAir_IASA");
    unsafe {
        let fp = fpp(gobj);
        if !ftPe_Float_CheckContinueInput(&*fp) {
            (*fp).mv.pe.floatattack.x0 = 1;
        }
        if (*fp).allow_interrupt() != 0 && !ext::ftCo_80095328(gobj, core::ptr::null_mut()) {
            if !ftPe_8011BE80(gobj) {
                if ext::ftCo_800CB870(gobj) {
                    return;
                }
            }
        }
    }
}

pub fn ftPe_FloatAttackAir_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftPe_FloatAttackAir_Phys");
    unsafe {
        if (*fpp(gobj)).mv.pe.floatattack.x0 != 0 {
            st::ft_80084DB0(gobj);
        } else {
            ftPe_Float_Phys(gobj);
        }
    }
}
