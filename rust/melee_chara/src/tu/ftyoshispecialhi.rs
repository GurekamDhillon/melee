//! src/melee/ft/kinds/ftYoshi/ftyoshispecialhi.c: Yoshi's Egg Throw (up B). Fused sites
//! (tools/rust-port/fma_sites.py): the two `(x4 * x100 + xFC)` of `ftYs_SpecialS_8012DF8C` and the
//! `x4 * x110 + x10C` argument of `fn_8012E110`, written with `melee_core::fp::fmadds`. sinf/cosf are
//! hooks whose default is the MSL port (`melee_runtime::gekko`). State-oracle checked against the C
//! (tests/oracle_ftyoshispecialhi.rs, gen/oracle).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use crate::tu::ftinl;
use melee_core::fp::fmadds;
use melee_types::*;

/// Already ported (rounds 2/3/4/6/7/11): re-exported under this TU.
pub use crate::chars::r11::ftYoshi::ftyoshispecialhi::{ftYs_SpecialAirHi_Anim, ftYs_SpecialHi_Anim};
pub use crate::chars::r2::ftYoshi::ftyoshispecialhi::{ftYs_SpecialAirHi_Phys, ftYs_SpecialHi_Phys};
pub use crate::chars::r3::ftYoshi::ftyoshispecialhi::setAccessory4Callback;
pub use crate::chars::r4::ftYoshi::ftyoshispecialhi::{fn_8012E44C, setDamageCallbacks};
pub use crate::chars::r6::ftYoshi::ftyoshispecialhi::fn_8012E3B4;
pub use crate::chars::r7::ftYoshi::ftyoshispecialhi::{ftYs_SpecialAirHi_Enter, ftYs_SpecialHi_Enter};

/// Calls into TUs that are not ported yet. Each is a settable hook; unset it panics.
pub mod ext {
    use crate::tu::hookmac::{hook, hook_or};
    use melee_types::*;
    hook_or!(COSF, cosf, (x: f32) -> f32, melee_runtime::gekko::cosf);
    hook_or!(SINF, sinf, (x: f32) -> f32, melee_runtime::gekko::sinf);
    hook!(IT_802F2CE0, it_802F2CE0, (item: *mut Item_GObj, clear_destroy_type: i32));
    hook!(LB_8000B1CC, lb_8000B1CC, (jobj: *mut HSD_JObj, pos0: *mut Vec3, pos1: *mut Vec3));
    hook!(IT_802B2A10, it_802B2A10, (gobj: *mut HSD_GObj, pos: *mut Vec3, arg: i32, dir: f32) -> *mut Item_GObj);
    hook!(IT_802B28C8, it_802B28C8, (item: *mut Item_GObj, pos: *mut Vec3, offset: *mut Vec3, facing_dir: f32, unused: f32));
    hook!(IT_802B2890, it_802B2890, (item: *mut Item_GObj));
    pub use crate::tu::ftinl::ext::{ft_80082C74, ft_80083A48, ft_8008403C};
}

#[inline(always)]
unsafe fn fpp(gobj: *mut HSD_GObj) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}

/// `fn_8012E110` as a Fighter callback entry (accessory4_cb): the shared `chars::cb` entry, which calls it.
pub use crate::chars::cb::ftyoshispecialhi__fn_8012E110 as cb_fn_8012E110;
/// `fn_8012E3B4` as a callback entry.
pub unsafe extern "C" fn cb_fn_8012E3B4(g: *mut HSD_GObj) {
    fn_8012E3B4(g);
}
/// `fn_8012E44C` as a callback entry.
pub unsafe extern "C" fn cb_fn_8012E44C(g: *mut HSD_GObj) {
    fn_8012E44C(g);
}

pub fn ftYs_SpecialS_8012DF18(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        if !(*fp).target_item_gobj.is_null() {
            let msid = (*fp).motion_id as i32;
            let mut clear_destroy_type = false;
            if msid == ftYs_MS_SpecialN1_1 as i32
                || msid == ftYs_MS_SpecialN2_1 as i32
                || msid == ftYs_MS_SpecialAirN1_2 as i32
                || msid == ftYs_MS_SpecialAirN2_1 as i32
            {
                clear_destroy_type = true;
            }
            ext::it_802F2CE0((*fp).target_item_gobj, clear_destroy_type as i32);
            (*fp).x1A64 = core::ptr::null_mut();
            (*fp).target_item_gobj = core::ptr::null_mut();
        }
    }
}

/// The egg's throw velocity from the stick: `arg1` receives the vector.
pub fn ftYs_SpecialS_8012DF8C(gobj: *mut HSD_GObj, arg1: *mut Vec3) {
    unsafe {
        let fp = fpp(gobj);
        let da = &*((*fp).dat_attrs as *const ftYs_DatAttrs);
        let mut mag: f32;
        {
            let sx = (*fp).input.lstick[0].x;
            let ax = if sx < 0.0 { -sx } else { sx };
            let temp = ax / da.xEC;
            mag = temp;
            if mag > 1.0 {
                mag = 1.0;
            }
            mag *= da.xF0;
            if mag < da.xF4 {
                mag = 0.0;
            }
            mag *= ftinl::getStickDirX(fp) as f32;
        }
        {
            let angle: f32;
            if (*fp).facing_dir == 1.0 {
                angle = da.specialhi_base_angle - mag;
            } else {
                // M_PI is a double: the subtractions happen in double, one rounding to float
                angle = (3.14159265358979323846_f64 - da.specialhi_base_angle as f64 - mag as f64) as f32;
            }
            let x4 = (*fp).mv.ys.specialhi.x4 as f32;
            (*arg1).x = fmadds(x4, da.x100, da.xFC) * ext::cosf(angle);
            (*arg1).y = fmadds(x4, da.x100, da.xFC) * ext::sinf(angle);
            (*arg1).z = 0.0;
        }
    }
}

pub fn fn_8012E110(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let da = &*((*fp).dat_attrs as *const ftYs_DatAttrs);
        if (*fp).anon_2210.anon_0.throw_flags_b0() != 0 {
            (*fp).anon_2210.anon_0.set_throw_flags_b0(0);
            let mut sp30 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
            ext::lb_8000B1CC((*(*fp).parts.add(31)).joint, core::ptr::null_mut(), &mut sp30);
            let item = ext::it_802B2A10(gobj, &mut sp30, 0x1F, (*fp).facing_dir);
            (*fp).u.ys.x2238 = item;
            (*fp).x1984_heldItemSpec = item;
            setDamageCallbacks(gobj);
        }
        if (*fp).cmd_vars[0] != 0 && !(*fp).u.ys.x2238.is_null() {
            let mut sp24 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
            let mut sp18 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
            (*fp).cmd_vars[0] = 0;
            (*fp).mv.ys.specialhi.x0 = 1;
            sp24.x = da.x104 * (*fp).facing_dir;
            sp24.y = da.x108;
            sp24.z = 0.0;
            ftYs_SpecialS_8012DF8C(gobj, &mut sp18);
            {
                let x4 = (*fp).mv.ys.specialhi.x4 as f32;
                ext::it_802B28C8((*fp).u.ys.x2238, &mut sp18, &mut sp24, fmadds(x4, da.x110, da.x10C), x4);
            }
            (*fp).u.ys.x2238 = core::ptr::null_mut();
            (*fp).take_dmg_cb = None;
            (*fp).death2_cb = None;
        }
    }
}

pub fn ftYs_SpecialS_8012E270(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        if !(*fp).u.ys.x2238.is_null() && (*fp).mv.ys.specials.x0 == 0 {
            ext::it_802B2890((*fp).u.ys.x2238);
        }
        (*fp).u.ys.x2238 = core::ptr::null_mut();
        (*fp).take_dmg_cb = None;
        (*fp).death2_cb = None;
    }
}

pub fn ftYs_SpecialHi_Coll(gobj: *mut HSD_GObj) {
    ext::ft_8008403C(gobj, Some(cb_fn_8012E3B4));
}

pub fn ftYs_SpecialAirHi_Coll(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        if (*fp).mv.ys.specialhi.x0 != 0 {
            ext::ft_80083A48(gobj, Some(cb_fn_8012E44C));
        } else {
            ext::ft_80082C74(gobj, Some(cb_fn_8012E44C));
        }
    }
}
