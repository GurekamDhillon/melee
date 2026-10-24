//! src/melee/ft/kinds/ftPeach/ftpeachfloat.c: Peach's float (start / continue checks, the Float
//! motion state). No fused sites (tools/rust-port/fma_sites.py: 0 in every function). Smoke-tested
//! through recording hooks by tests/tu_ftpeachfloat.rs.
//!
//! TARGET_PC build: `ftPe_8011BA54` / `ftPe_8011BAD8` route non-Peach kinds through
//! `Mex_OnFloatDispatch` (m-ex OnFloat patches), reproduced here.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use crate::stubs as st;
use melee_fighter::hooks as fh;
use melee_types::*;

/// `HSD_PAD_XY` (controller.h).
pub const HSD_PAD_XY: u32 = (HSD_PAD_X | HSD_PAD_Y) as u32;

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
    /// `Mex_OnFloatDispatch(kind, gobj)` (pc/mex, TARGET_PC): the m-ex onFloat callback.
    hook!(MEX_ONFLOATDISPATCH, Mex_OnFloatDispatch, (kind: i32, gobj: *mut HSD_GObj) -> i32);
    /// `efAsync_Spawn(gobj, queue_head, spawn_kind, gfx_id, jobj, ...)` (ef/efasync.c).
    hook!(EFASYNC_SPAWN, efAsync_Spawn, (gobj: *mut HSD_GObj, q: *mut core::ffi::c_void, kind: u32, gfx: u32, jobj: *mut HSD_JObj));
    /// `ftCo_SpecialAir_CheckInput(gobj)` (ftCommon/ftCo_SpecialAir.c).
    pub static mut FTCO_SPECIALAIR_CHECKINPUT: Option<fn(*mut HSD_GObj) -> bool> = None;
            #[allow(non_snake_case)]
            pub fn ftCo_SpecialAir_CheckInput(gobj: *mut HSD_GObj) -> bool {
                match unsafe { FTCO_SPECIALAIR_CHECKINPUT } {
                    Some(f) => f(gobj),
                    None => { return unsafe { melee_fighter::tu::ftCo_SpecialAir::ftCo_SpecialAir_CheckInput(gobj as _) as _ }; },
                }
            }
    /// `ft_800831CC(gobj, line_cb, coll_cb)` (ft/ft_081B.c): air collision with a landing-line filter.
    crate::port_hook!(FT_800831CC, ft_800831CC, crate::adapt::ft_800831CC, (gobj: *mut HSD_GObj, line_cb: Option<unsafe extern "C" fn(*mut HSD_GObj, i32) -> bool>, cb: HSD_GObjEvent));
    hook_or!(FTCO_80096CC8, ftCo_80096CC8, melee_fighter::tu::ftCo_FallSpecial::ftCo_80096CC8, (gobj: *mut HSD_GObj, line_id: i32) -> bool);
    /// `ft_80082B1C(gobj)` (ft/ft_081B.c), used as the landing callback.
    pub static mut FT_80082B1C: Option<fn(*mut HSD_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn ft_80082B1C(gobj: *mut HSD_GObj)  {
                match unsafe { FT_80082B1C } {
                    Some(f) => f(gobj),
                    None => { unsafe { let _ = melee_fighter::tu::ft_081b::ft_80082b1c(gobj as _); } return; },
                }
            }
    hook_or!(FTCO_JUMPAERIAL_PHYS_CB, ftCo_JumpAerial_Phys_Cb, melee_fighter::tu::ftCo_JumpAerial::ftCo_JumpAerial_Phys_Cb, (gobj: *mut HSD_GObj));
}

/// `ftCo_80096CC8` as the `bool (*)(Fighter_GObj*, int)` callback entry.
pub unsafe extern "C" fn cb_ftCo_80096CC8(g: *mut HSD_GObj, line_id: i32) -> bool {
    ext::ftCo_80096CC8(g, line_id)
}
/// `ft_80082B1C` as an `HSD_GObjEvent` entry.
pub unsafe extern "C" fn cb_ft_80082B1C(g: *mut HSD_GObj) {
    ext::ft_80082B1C(g);
}

#[inline(always)]
unsafe fn fpp(gobj: *mut HSD_GObj) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}

/// `ftPe_Float_CheckContinueInput`: stick up past the tap-jump threshold, or X/Y held.
pub fn ftPe_Float_CheckContinueInput(fp: &Fighter) -> bool {
    fp.input.lstick[0].y >= fh::common_data().tap_jump_threshold || (fp.input.held_buttons[0] & HSD_PAD_XY) != 0
}

/// `checkStartFloatInput` (static): stick down past `x88` and X/Y held.
fn checkStartFloatInput(gobj: *mut HSD_GObj) -> bool {
    unsafe {
        let fp = &*fpp(gobj);
        fp.input.lstick[0].y <= -fh::common_data().x88 && (fp.input.held_buttons[0] & HSD_PAD_XY) != 0
    }
}

pub fn ftPe_8011BA54(gobj: *mut HSD_GObj) -> bool {
    unsafe {
        let fp = fpp(gobj);
        let float_input = checkStartFloatInput(gobj);
        // TARGET_PC: m-ex "Fighter OnFloat/onFloat - Down+XY.asm".
        if (*fp).kind != Ft_Kind_Peach && float_input {
            return ext::Mex_OnFloatDispatch((*fp).kind as i32, gobj) != 0;
        }
        if (*fp).kind == Ft_Kind_Peach && (*fp).u.pe.has_float != 0 && float_input {
            ftPe_8011BB6C(gobj, true);
            return true;
        }
        false
    }
}

pub fn ftPe_8011BAD8(gobj: *mut HSD_GObj) -> bool {
    unsafe {
        let fp = fpp(gobj);
        // TARGET_PC: m-ex "Fighter OnFloat/onFloat - JumpPeak.asm".
        if (*fp).kind != Ft_Kind_Peach && (*fp).self_vel.y <= 0.0 && ftPe_Float_CheckContinueInput(&*fp) {
            return ext::Mex_OnFloatDispatch((*fp).kind as i32, gobj) != 0;
        }
        if (*fp).kind == Ft_Kind_Peach && (*fp).self_vel.y <= 0.0 && (*fp).u.pe.has_float != 0 {
            if ftPe_Float_CheckContinueInput(&*fp) {
                ftPe_8011BB6C(gobj, true);
                return true;
            }
        }
        false
    }
}

/// `spawnParticle` (static).
fn spawnParticle(gobj: *mut HSD_GObj, joint: *mut HSD_JObj) {
    unsafe {
        let fp = fpp(gobj);
        ext::efAsync_Spawn(gobj, &raw mut (*fp).x60C as *mut core::ffi::c_void, 0, 1236, joint);
    }
}

pub fn ftPe_8011BB6C(gobj: *mut HSD_GObj, arg1: bool) {
    unsafe {
        let fp = fpp(gobj);
        let da = (*fp).dat_attrs as *const ftPe_DatAttrs;
        st::Fighter_ChangeMotionState(gobj, ftPe_MS_Float as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        (*fp).u.pe.has_float = 0;
        if arg1 {
            (*fp).u.pe.x4 = (*da).xC;
        }
        (*fp).self_vel.y = 0.0;
        (*fp).set_x2219_b0(1);
        let bone = melee_fighter::ftdmgfall::ftparts_get_bone_index(&*fp, FtPart_TransN as i32);
        let joint = (*(*fp).parts.add(bone as usize)).joint;
        spawnParticle(gobj, joint);
    }
}

pub fn ftPe_Float_Anim(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        if (*fp).u.pe.x4 > 0.0 {
            (*fp).u.pe.x4 -= 1.0;
        }
        if (*fp).u.pe.x4 <= 0.0 {
            crate::tu::ftpeachfloatfall::ftPe_UpdateFloatDir(gobj);
        }
    }
}

pub fn ftPe_Float_IASA(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        if !ext::ftCo_SpecialAir_CheckInput(gobj) && !crate::tu::ftpeachfloatattack::ftPe_8011BE80(gobj) {
            let float_input = (*fp).input.lstick[0].y >= fh::common_data().tap_jump_threshold
                || ((*fp).input.held_buttons[0] & HSD_PAD_XY) != 0;
            if !float_input {
                crate::tu::ftpeachfloatfall::ftPe_UpdateFloatDir(gobj);
            }
        }
    }
}

pub fn ftPe_Float_Phys(gobj: *mut HSD_GObj) {
    unsafe { melee_fighter::ftcommon::calc_self_accel_drift(&mut *fpp(gobj)) }
}

pub fn ftPe_Float_Coll(gobj: *mut HSD_GObj) {
    ext::ft_800831CC(gobj, Some(cb_ftCo_80096CC8), Some(cb_ft_80082B1C));
}
