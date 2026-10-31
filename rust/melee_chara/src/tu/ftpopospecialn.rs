//! src/melee/ft/kinds/ftPopo/ftpopospecialn.c: Ice Climbers' neutral-B (Ice Shot). One fused site:
//! ftPp_SpecialN_8011F500 (`pos.x = xC * facing_dir + pos.x` is an fmadds;
//! tools/rust-port/fma_sites.py). Smoke-tested by tests/tu_ftpopospecialn.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use crate::stubs as st;
use melee_core::fp::fmadds;
use melee_types::*;

/// Already ported (rounds 2/7): re-exported under this TU.
pub use crate::chars::r2::ftPopo::ftpopospecialn::{
    ftPp_SpecialAirN_Anim, ftPp_SpecialAirN_Phys, ftPp_SpecialN_Anim, ftPp_SpecialN_Phys,
};
pub use crate::chars::r7::ftPopo::ftpopospecialn::ftPp_SpecialN_Enter;

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
    /// Like `hook!`, but an unset hook falls back to an existing native port.
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
    crate::port_hook!(IT_802C17DC, it_802C17DC, crate::adapt::it_802C17DC, (item: *mut HSD_GObj));
    crate::port_hook!(IT_802C16F8, it_802C16F8, crate::adapt::it_802C16F8, (item: *mut HSD_GObj));
    crate::port_hook!(IT_802C1590, it_802C1590, crate::adapt::it_802C1590, (gobj: *mut HSD_GObj, pos: *mut Vec3, kind: i32, facing_dir: f32) -> *mut HSD_GObj);
    pub static mut LB_8000B1CC: Option<fn(*mut HSD_JObj, *const Vec3, *mut Vec3) > = None;
            #[allow(non_snake_case)]
            pub fn lb_8000B1CC(jobj: *mut HSD_JObj, off: *const Vec3, out: *mut Vec3)  {
                match unsafe { LB_8000B1CC } {
                    Some(f) => f(jobj, off, out),
                    None => { unsafe { melee_runtime::tu::lb_00B0::lb_8000B1CC(jobj as _, off as _, out as _); } return; },
                }
            }
    crate::port_hook!(FT_PLAYSFX, ft_PlaySFX, crate::adapt::ft_PlaySFX, (fp: *mut Fighter, sfx: i32, vol: u8, pan: u8));
    hook_or!(FT_800881D8, ft_800881D8, ft_800881d8_port, (fp: *mut Fighter, sfx: i32, vol: u8, pan: u8));
    fn ft_800881d8_port(fp: *mut Fighter, sfx: i32, vol: u8, pan: u8) {
        melee_fighter::ft0881::ft_800881d8(unsafe { &mut *fp }, sfx, vol, pan)
    }
}

type G = *mut HSD_GObj;

#[inline(always)]
unsafe fn fpp(gobj: G) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}

pub fn ftPp_SpecialAirN_Enter(gobj: G) {
    melee_core::ct!("ftPp_SpecialAirN_Enter");
    unsafe {
        let fp = fpp(gobj);
        let icattr = (*fp).dat_attrs as *const ftIceClimberAttributes;

        (*fp).anon_2210.throw_flags = 0;
        (*fp).cmd_vars[0] = 0;
        (*fp).u.nn.x222C = core::ptr::null_mut();

        if (*fp).u.nn.x224C as i32 == 0 {
            (*fp).self_vel.y = (*icattr).x4;
            (*fp).u.nn.x224C = 1;
            (*fp).u.nn.x2250 = 0.0;
        } else {
            (*fp).u.nn.x2250 = -10.0;
        }

        st::Fighter_ChangeMotionState(gobj, 342, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());

        st::ftAnim_8006EBA4(gobj);
        (*fp).accessory4_cb = Some(crate::chars::cb::ftpopospecialn__ftPp_SpecialN_8011F500);
    }
}

pub fn ftPp_SpecialN_IASA(_gobj: G) { melee_core::ct!("ftPp_SpecialN_IASA");}
pub fn ftPp_SpecialAirN_IASA(_gobj: G) { melee_core::ct!("ftPp_SpecialAirN_IASA");}

pub fn ftPp_SpecialN_Coll(gobj: G) {
    melee_core::ct!("ftPp_SpecialN_Coll");
    unsafe {
        if st::ft_80082708(gobj) == 0 {
            let fp1 = fpp(gobj);
            if !(*fp1).u.nn.x222C.is_null() {
                ext::it_802C17DC((*fp1).u.nn.x222C);
                let fp2 = fpp(gobj);
                if (*fp1).u.nn.x222C == (*fp2).u.nn.x222C {
                    (*fp2).u.nn.x222C = core::ptr::null_mut();
                    (*fp2).death2_cb = None;
                    (*fp2).take_dmg_cb = None;
                }
            }
            st::ftCo_Fall_Enter(gobj);
        }
    }
}

pub fn ftPp_SpecialAirN_Coll(gobj: G) {
    melee_core::ct!("ftPp_SpecialAirN_Coll");
    unsafe {
        let fp = fpp(gobj);
        let da = (*fp).dat_attrs as *const ftIceClimberAttributes;
        if st::ft_80081D0C(gobj) != GA_Ground as i32 {
            let fp1 = fpp(gobj);
            if !(*fp1).u.pp.x222C.is_null() {
                ext::it_802C17DC((*fp1).u.pp.x222C);
                let fp2 = fpp(gobj);
                if (*fp1).u.pp.x222C == (*fp2).u.pp.x222C {
                    (*fp2).u.pp.x222C = core::ptr::null_mut();
                    (*fp2).death2_cb = None;
                    (*fp2).take_dmg_cb = None;
                }
            }
            (*fp).u.pp.x224C = 0;
            (*fp).u.pp.x2250 = 0.0;
            st::ftCo_LandingFallSpecial_Enter(gobj, 0, (*da).x8);
        }
    }
}

/// `inlineA0` (static inline): clears the shot when it is still the one `other_fp` holds.
unsafe fn inlineA0(gobj: G, other_fp: *mut Fighter) {
    let fp = fpp(gobj);
    if (*other_fp).u.pp.x222C == (*fp).u.pp.x222C {
        (*fp).u.pp.x222C = core::ptr::null_mut();
        (*fp).death2_cb = None;
        (*fp).take_dmg_cb = None;
    }
}

pub fn ftPp_SpecialN_8011F500(gobj: G) {
    melee_core::ct!("ftPp_SpecialN_8011F500");
    unsafe {
        let fp = fpp(gobj);
        let cmd_var0 = (*fp).cmd_vars[0];

        if cmd_var0 == 0 {
            return;
        }
        if cmd_var0 == 1 {
            let da = (*fp).dat_attrs as *const ftIceClimberAttributes;
            let mut pos: Vec3 = core::mem::zeroed();
            ext::lb_8000B1CC((*(*fp).parts).joint, core::ptr::null(), &mut pos);
            pos.x = fmadds((*da).xC, (*fp).facing_dir, pos.x);
            pos.y += (*da).x10 + (*fp).u.pp.x2250;
            (*fp).u.pp.x222C = ext::it_802C1590(gobj, &mut pos, It_Kind_IceClimber_Ice as i32, (*fp).facing_dir);
            ext::ft_PlaySFX(fp, 130021, 127, 64);
            if !(*fp).u.pp.x222C.is_null() {
                (*fp).death2_cb = Some(crate::chars::cb::ftpopo__ftPp_Init_8011F060);
                (*fp).take_dmg_cb = Some(crate::chars::cb::ftpopo__ftPp_Init_8011F060);
            }
            (*fp).cmd_vars[0] = 0;
        } else if cmd_var0 == 2 {
            if !(*fp).u.pp.x222C.is_null() {
                ext::it_802C16F8((*fp).u.pp.x222C);
                (*fp).cmd_vars[0] = 0;
                if (*fp).kind == Ft_Kind_Popo as _ {
                    ext::ft_800881D8(fp, 130141, 127, 64);
                } else {
                    ext::ft_800881D8(fp, 130090, 127, 64);
                }
                ext::ft_PlaySFX(fp, 130024, 127, 64);
                inlineA0(gobj, fp);
            }
        }
    }
}
