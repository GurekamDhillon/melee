//! src/melee/ft/kinds/ftLink/ftlinkspeciallw.c: Link's Bomb (down B). No float arithmetic, so
//! no fused sites (tools/rust-port/fma_sites.py). Smoke-tested by tests/tu_ftlinkspeciallw.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use crate::stubs as st;
use melee_fighter::hooks as fh;
use melee_types::*;

/// Already ported (rounds 2 and 3): re-exported under this TU.
pub use crate::chars::r2::ftLink::ftlinkspeciallw::{ftLk_SpecialAirLw_Phys, ftLk_SpecialLw_Phys};
pub use crate::chars::r3::ftLink::ftlinkspeciallw::{ftLk_SpecialAirLw_Coll, ftLk_SpecialLw_Coll};

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
    // `lb_8000B1CC(jobj, offset, out)` (lb/lb_00B0.c): the world position of a joint.
    crate::port_hook!(LB_8000B1CC, lb_8000B1CC, crate::adapt::lb_8000B1CC, (jobj: *mut core::ffi::c_void, off: Option<&Vec3>, out: &mut Vec3));
    // `it_8029DD58(owner, pos, x10, part, facing_dir)` (it/kinds/itlinkbomb.c): spawn the bomb.
    crate::port_hook!(IT_8029DD58, it_8029DD58, crate::adapt::it_8029DD58, (owner: *mut HSD_GObj, pos: &Vec3, x10: i32, part: i32, facing_dir: f32) -> *mut Item_GObj);
}

#[inline(always)]
unsafe fn fpp(gobj: *mut HSD_GObj) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}

/// `ftCheckThrowB0` (ft/inlines.h): test and clear throw_flags_b0.
#[inline(always)]
pub(crate) fn ftCheckThrowB0(fp: &mut Fighter) -> bool {
    melee_core::ct!("ftCheckThrowB0");
    unsafe {
        if fp.anon_2210.anon_0.throw_flags_b0() != 0 {
            fp.anon_2210.anon_0.set_throw_flags_b0(0);
            true
        } else {
            false
        }
    }
}

/// `updateBomb` (static): a held bomb (or its sibling kind) is re-thrown through ftCo_800957F4.
fn updateBomb(gobj: *mut HSD_GObj, msid: i32) -> bool {
    unsafe {
        let fp = fpp(gobj);
        if !(*fp).item_gobj.is_null() {
            let bomb_kind = (fh::it_get_kind((*fp).item_gobj) as u32).wrapping_sub(It_Kind_Link_Bomb as u32);
            if bomb_kind <= 1 {
                st::ftCo_800957F4(gobj, msid);
            }
            return true;
        }
        false
    }
}

/// `spawnBomb` (static): the script's throw_flags_b0 pulse creates the bomb in the hand.
pub fn spawnBomb(gobj: *mut HSD_GObj) {
    melee_core::ct!("spawnBomb");
    unsafe {
        let fp = fpp(gobj);
        let da = (*fp).dat_attrs as *const ftLk_DatAttrs;
        if ftCheckThrowB0(&mut *fp) {
            let mut pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
            let idx = fh::ftparts_get_bone_index(&*fp, FtPart_LThumbNb as i32);
            ext::lb_8000B1CC((*(*fp).parts.offset(idx as isize)).joint as *mut _, None, &mut pos);
            let part = (*da).x48;
            let facing_dir = (*fp).facing_dir;
            let x10 = (*(*(*fp).ft_data).x8).x10 as i32;
            let igobj = ext::it_8029DD58(gobj, &pos, x10, part, facing_dir);
            (*fp).item_gobj = igobj;
            st::ftParts_80074A4C(gobj, 2, 1);
            if !igobj.is_null() {
                melee_fighter::tu::ftpickupitem::ftpickupitem_80094818(gobj, true);
            }
        }
    }
}

/// `spawnBomb` as a Fighter callback entry.
pub unsafe extern "C" fn cb_spawnBomb(g: *mut HSD_GObj) {
    melee_core::ct!("cb_spawnBomb");
    spawnBomb(g);
}

/// `doEnter` (static).
fn doEnter(gobj: *mut HSD_GObj, msid0: i32, msid1: i32) {
    unsafe {
        let fp = fpp(gobj);
        if !updateBomb(gobj, msid0) {
            (*fp).anon_2210.throw_flags = 0;
            st::Fighter_ChangeMotionState(gobj, msid1, Ft_MF_None as u32, 0.0, 1.0, 0.0, core::ptr::null_mut());
            st::ftAnim_8006EBA4(gobj);
            (*fp).accessory4_cb = Some(cb_spawnBomb);
        }
    }
}

pub fn ftLk_SpecialLw_Enter(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLk_SpecialLw_Enter");
    doEnter(gobj, 108, ftLk_MS_SpecialLw as i32);
}

pub fn ftLk_SpecialAirLw_Enter(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLk_SpecialAirLw_Enter");
    doEnter(gobj, 112, ftLk_MS_SpecialAirLw as i32);
}

/// `doAnim` (static): run `cb` when the animation ends.
fn doAnim(gobj: *mut HSD_GObj, cb: fn(*mut HSD_GObj)) {
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        cb(gobj);
    }
}

pub fn ftLk_SpecialLw_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLk_SpecialLw_Anim");
    doAnim(gobj, st::ft_8008A2BC);
}

pub fn ftLk_SpecialAirLw_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLk_SpecialAirLw_Anim");
    doAnim(gobj, st::ftCo_Fall_Enter);
}
