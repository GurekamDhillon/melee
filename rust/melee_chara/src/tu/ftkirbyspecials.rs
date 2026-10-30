//! src/melee/ft/kinds/ftKirby/ftkirbyspecials.c: Kirby's Hammer Flip (side B) and the Hat helpers
//! shared with the swallow code. No fused sites (tools/rust-port/fma_sites.py). Smoke-tested by
//! tests/tu_ftkirbyspecials.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use crate::stubs as st;
use melee_fighter::hooks as fh;
use melee_types::*;

/// Already ported (rounds 2, 4 and the root module): re-exported under this TU.
pub use crate::chars::r2::ftKirby::ftkirbyspecials::{
    ftKb_SpecialAirS_Anim, ftKb_SpecialAirS_Phys, ftKb_SpecialS_Anim, ftKb_SpecialS_Phys,
};
pub use crate::chars::r4::ftKirby::ftkirbyspecials::{ftKb_SpecialAirS_Enter, ftKb_SpecialS_Enter};

/// Calls into TUs that are not ported yet. Each is a settable hook; unset it panics.
pub mod ext {
    use core::ffi::c_void;
    use melee_types::{HSD_GObj, Item_GObj, Vec3};
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
    pub use crate::tu::ftcaptainspecialn::ext::{efSync_Spawn, EFSYNC_SPAWN};
    pub static mut LB_8000B1CC: Option<fn(*mut c_void, *const Vec3, *mut Vec3) > = None;
            #[allow(non_snake_case)]
            pub fn lb_8000B1CC(joint: *mut c_void, off: *const Vec3, out: *mut Vec3)  {
                match unsafe { LB_8000B1CC } {
                    Some(f) => f(joint, off, out),
                    None => { unsafe { let _ = melee_runtime::tu::lb_00B0::lb_8000B1CC(joint as _, off as _, out as _); } return; },
                }
            }
    pub static mut IT_802ADC34: Option<fn(*mut Item_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn it_802ADC34(item: *mut Item_GObj)  {
                match unsafe { IT_802ADC34 } {
                    Some(f) => f(item),
                    None => { unsafe { let _ = melee_items::tu::itkirbyhammer::it_802ADC34(item as _); } return; },
                }
            }
    pub static mut IT_802ADC54: Option<fn(*mut HSD_GObj, *const Vec3, i32, i32, f32) -> *mut Item_GObj> = None;
            #[allow(non_snake_case)]
            pub fn it_802ADC54(gobj: *mut HSD_GObj, pos: *const Vec3, part: i32, ground_or_air: i32, dir: f32) -> *mut Item_GObj {
                match unsafe { IT_802ADC54 } {
                    Some(f) => f(gobj, pos, part, ground_or_air, dir),
                    None => { return unsafe { melee_items::tu::itkirbyhammer::it_802ADC54(gobj as _, pos as _, part as _, ground_or_air as _, dir as _) as _ }; },
                }
            }
    crate::port_hook!(OSREPORT, OSReport, crate::adapt::OSReport, (msg: &str));
    pub static mut ASSERT: Option<fn(&str, u32, &str) > = None;
            #[allow(non_snake_case)]
            pub fn __assert(file: &str, line: u32, cond: &str)  {
                match unsafe { ASSERT } {
                    Some(f) => f(file, line, cond),
                    None => { unsafe { let _ = melee_runtime::tu::lbext::__assert(file as _, line as _, cond as _); } return; },
                }
            }
}

#[inline(always)]
unsafe fn fpp(gobj: *mut HSD_GObj) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}

/// `Fighter_SetEffectHitlagCallbacks` (ft/inlines.h).
#[inline(always)]
unsafe fn set_effect_hitlag_callbacks(fp: *mut Fighter) {
    (*fp).pre_hitlag_cb = Some(crate::chars::cb::ext__efLib_PauseAll);
    (*fp).post_hitlag_cb = Some(crate::chars::cb::ext__efLib_ResumeAll);
}

/// `ftKb_SpecialN_800F5800`: current position (the root module's Fighter-only half).
pub fn ftKb_SpecialN_800F5800(gobj: *mut HSD_GObj, vec: &mut Vec3) {
    melee_core::ct!("ftKb_SpecialN_800F5800");
    unsafe { crate::chars::ftKirby::ftkirbyspecials::ftKb_SpecialN_800F5800(&*fpp(gobj), vec) }
}

/// `ftKb_SpecialAirLw_800F539C`: clear the hat item pointer.
pub fn ftKb_SpecialAirLw_800F539C(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_SpecialAirLw_800F539C");
    unsafe {
        (*fpp(gobj)).u.kb.hat.x0 = core::ptr::null_mut();
    }
}

/// `fn_800F53AC_SpawnEffect` (static inline).
pub fn fn_800F53AC_SpawnEffect(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_800F53AC_SpawnEffect");
    unsafe {
        let fp = fpp(gobj);
        let ef_id = if (*fp).ground_or_air == GA_Air { 0x497 } else { 0x496 };
        let hip = (*(*fp).parts.add(FtPart_HipN as usize)).joint as *mut core::ffi::c_void;
        let facing = &mut (*fp).facing_dir as *mut f32 as *mut core::ffi::c_void;
        ext::efSync_Spawn(ef_id, gobj, &[hip, facing]);
        (*fp).set_x2219_b0(1);
        set_effect_hitlag_callbacks(fp);
    }
}

/// `fn_800F53AC_CleanupItem` (static inline).
pub fn fn_800F53AC_CleanupItem(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_800F53AC_CleanupItem");
    unsafe {
        let fp = fpp(gobj);
        let item = (*fp).u.kb.hat.x0;
        if !item.is_null() {
            ext::it_802ADC34(item);
            (*fp).u.kb.hat.x0 = core::ptr::null_mut();
        }
    }
}

/// `fn_800F53AC` (accessory4 callback: hat item spawn / cleanup).
pub fn fn_800F53AC(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_800F53AC");
    unsafe {
        let fp = fpp(gobj);
        let mut pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        if (*fp).cmd_vars[0] == 1 {
            ext::lb_8000B1CC((*(*fp).parts.add(FtPart_R3rdNa as usize)).joint as *mut _, core::ptr::null(), &mut pos);
            let dir = (*fp).facing_dir;
            (*fp).u.kb.hat.x0 = ext::it_802ADC54(gobj, &pos, FtPart_R3rdNa as i32, (*fp).ground_or_air as i32, dir);
            (*fp).x1984_heldItemSpec = (*fp).u.kb.hat.x0;
            (*fp).cmd_vars[0] = 0;
            if !(*fp).u.kb.hat.x0.is_null() {
                let ours = crate::chars::cb::ftkirby__ftKb_Init_800EE74C as unsafe extern "C" fn(*mut HSD_GObj);
                if let Some(cur) = (*fp).death2_cb {
                    if cur as usize != ours as usize {
                        ext::OSReport("Other Dead_Proc Existence");
                        ext::__assert("ftkirbyspecials.c", 0x66, "0");
                    }
                }
                (*fp).death2_cb = Some(ours);
                (*fp).take_dmg_cb = Some(crate::chars::cb::ftkirby__ftKb_Init_800EE7B8);
                fn_800F53AC_SpawnEffect(gobj);
            }
        }
        if (*fp).cmd_vars[0] == 2 {
            fn_800F53AC_CleanupItem(gobj);
            (*fp).cmd_vars[0] = 0;
        }
    }
}

/// `fn_800F53AC` as the `accessory4_cb` entry.
pub unsafe extern "C" fn cb_fn_800F53AC(g: *mut HSD_GObj) {
    melee_core::ct!("cb_fn_800F53AC");
    fn_800F53AC(g);
}

pub fn ftKb_SpecialAirLw_800F5524(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_SpecialAirLw_800F5524");
    unsafe {
        let fp = fpp(gobj);
        if !(*fp).u.kb.hat.x0.is_null() {
            ext::it_802ADC34((*fp).u.kb.hat.x0);
            (*fp).u.kb.hat.x0 = core::ptr::null_mut();
        }
    }
}

pub fn ftKb_SpecialS_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_SpecialS_Coll");
    unsafe {
        if st::ft_80082708(gobj) != 0 {
            return;
        }
        let fp = fpp(gobj);
        if !(*fp).u.kb.hat.x0.is_null() {
            ext::it_802ADC34((*fp).u.kb.hat.x0);
            (*fp).u.kb.hat.x0 = core::ptr::null_mut();
        }
        st::ftCo_Fall_Enter(gobj);
    }
}

pub fn ftKb_SpecialAirS_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_SpecialAirS_Coll");
    unsafe {
        let fp = fpp(gobj);
        let da = (*fp).dat_attrs as *const ftKb_DatAttrs;
        if st::ft_80081D0C(gobj) != 0 {
            let fp2 = fpp(gobj);
            if !(*fp2).u.kb.hat.x0.is_null() {
                ext::it_802ADC34((*fp2).u.kb.hat.x0);
                (*fp2).u.kb.hat.x0 = core::ptr::null_mut();
            }
            (*fp).u.kb.x64 = 0;
            st::ftCo_LandingFallSpecial_Enter(gobj, 0, (*da).specials_landing_lag);
        }
    }
}

/// Fills the victim collision box from the Kirby attributes (`ftKb_SpecialN_800F5820`).
pub fn ftKb_SpecialN_800F5820(gobj: *mut HSD_GObj, victim_coll_box: *mut ftCollisionBox, victim_scale_x: f32) {
    melee_core::ct!("ftKb_SpecialN_800F5820");
    unsafe {
        let fp = fpp(gobj);
        let da = (*fp).dat_attrs as *const ftKb_DatAttrs;
        (*victim_coll_box).top = (*da).specialn_unk1 * victim_scale_x;
        (*victim_coll_box).bottom = (*da).specialn_swallow_star_y_release * victim_scale_x;
        (*victim_coll_box).left.x = (*da).specialn_unk2 * victim_scale_x;
        (*victim_coll_box).left.y = (*da).specialn_unk3 * victim_scale_x;
        (*victim_coll_box).right.x = (*da).specialn_unk4 * victim_scale_x;
        (*victim_coll_box).right.y = (*da).specialn_unk5 * victim_scale_x;
    }
}

/// `ftKb_SpecialN_800F5874`: the two radians of the Kirby common-data ext attrs.
pub fn ftKb_SpecialN_800F5874(arg0: *mut Vec2) {
    melee_core::ct!("ftKb_SpecialN_800F5874");
    unsafe {
        let cd = (*crate::tu::ftkirbyspecialn::ftKb_KirbyFtData()).ext_attr as *const ftCommonData;
        (*arg0).x = (*cd).x9C_radians;
        (*arg0).y = (*cd).xA0_radians;
    }
}

/// `ftKb_SpecialN_800F5898`: Kirby's hat-less item joint (`ft_data->x48_items[4]`).
pub fn ftKb_SpecialN_800F5898(gobj: *mut HSD_GObj) -> *mut HSD_Joint {
    melee_core::ct!("ftKb_SpecialN_800F5898");
    unsafe {
        let fp = fpp(gobj);
        let ca = (*fp).ft_data;
        *(*ca).x48_items.add(4) as *mut HSD_Joint
    }
}

#[allow(unused_imports)]
use fh as _fh;
