//! src/melee/ft/kinds/ftLuigi/ftluigispecialn.c: Fireball (neutral B). No float arithmetic, so no
//! fused sites (tools/rust-port/fma_sites.py). Smoke-tested through recording hooks by
//! tests/tu_ftluigispecialn.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use melee_fighter::hooks as fh;
use melee_types::*;

/// Already ported (rounds 2, 4, 7): re-exported under this TU.
pub use crate::chars::r2::ftLuigi::ftluigispecialn::{
    ftLg_SpecialAirN_Anim, ftLg_SpecialAirN_Phys, ftLg_SpecialN_Anim, ftLg_SpecialN_Phys,
};
pub use crate::chars::r4::ftLuigi::ftluigispecialn::{ftLg_SpecialAirN_Coll, ftLg_SpecialN_Coll};
pub use crate::chars::r7::ftLuigi::ftluigispecialn::{ftLg_SpecialAirN_Enter, ftLg_SpecialN_Enter};

/// Calls into TUs that are not ported yet. Each is a settable hook; unset it panics, except the
/// ones with a native port behind them (the port runs when the hook is unset).
pub mod ext {
    use core::ffi::c_void;
    use melee_types::*;
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
    // `lb_8000B1CC(joint, offset, out)` (lb/lb_00B0.c); the C passes a NULL offset here.
    pub static mut LB_8000B1CC: Option<fn(*mut HSD_JObj, *const Vec3, *mut Vec3) > = None;
            #[allow(non_snake_case)]
            pub fn lb_8000B1CC(joint: *mut HSD_JObj, off: *const Vec3, out: *mut Vec3)  {
                match unsafe { LB_8000B1CC } {
                    Some(f) => f(joint, off, out),
                    None => { unsafe { melee_runtime::tu::lb_00B0::lb_8000B1CC(joint as _, off as _, out as _); } return; },
                }
            }
    pub static mut IT_802C01AC: Option<fn(*mut HSD_GObj, *mut Vec3, i32, f32) > = None;
            #[allow(non_snake_case)]
            pub fn it_802C01AC(gobj: *mut HSD_GObj, pos: *mut Vec3, kind: i32, facing: f32)  {
                match unsafe { IT_802C01AC } {
                    Some(f) => f(gobj, pos, kind, facing),
                    None => { unsafe { melee_items::tu::itluigifireball::it_802C01AC(gobj as _, pos as _, kind as _, facing as _); } return; },
                }
            }
    // `efSync_Spawn(gfx_id, gobj, ...)` (ef/efsync.c, variadic): the trailing arguments as a slice.
    crate::port_hook!(EFSYNC_SPAWN, efSync_Spawn, crate::adapt::efSync_Spawn, (gfx_id: i32, gobj: *mut HSD_GObj, args: &[*mut c_void]) -> *mut c_void);
    hook_or!(FTCO_FALL_IASA_INNER, ftCo_Fall_IASA_Inner, ftco_fall_iasa_inner_port, (gobj: *mut HSD_GObj));
    fn ftco_fall_iasa_inner_port(gobj: *mut HSD_GObj) {
        melee_fighter::tu::ftCo_Fall::ftCo_Fall_IASA_Inner(gobj);
    }
}

#[inline(always)]
unsafe fn fpp(gobj: *mut HSD_GObj) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}

/// `ftLg_SpecialN_FireSpawn` as a Fighter callback entry (`accessory4_cb`).
pub unsafe extern "C" fn cb_ftLg_SpecialN_FireSpawn(g: *mut HSD_GObj) {
    melee_core::ct!("cb_ftLg_SpecialN_FireSpawn");
    ftLg_SpecialN_FireSpawn(g);
}

/// Luigi's grounded Fireball IASA callback.
pub fn ftLg_SpecialN_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLg_SpecialN_IASA");
    unsafe {
        if (*fpp(gobj)).cmd_vars[0] != 0 {
            crate::stubs::ftCo_Wait_IASA(gobj);
        }
    }
}

/// Luigi's aerial Fireball IASA callback.
pub fn ftLg_SpecialAirN_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLg_SpecialAirN_IASA");
    unsafe {
        if (*fpp(gobj)).cmd_vars[0] != 0 {
            ext::ftCo_Fall_IASA_Inner(gobj);
        }
    }
}

/// Luigi's Fireball Spawn callback.
pub fn ftLg_SpecialN_FireSpawn(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLg_SpecialN_FireSpawn");
    unsafe {
        let fp = fpp(gobj);
        let flag = if (*fp).anon_2210.anon_0.throw_flags_b0() != 0 {
            (*fp).anon_2210.anon_0.set_throw_flags_b0(0);
            true
        } else {
            false
        };
        if flag {
            let mut sp10 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
            let bone = fh::ftparts_get_bone_index(&*fp, FtPart_L1stNb as i32);
            ext::lb_8000B1CC((*(*fp).parts.offset(bone as isize)).joint, core::ptr::null(), &mut sp10);
            ext::it_802C01AC(gobj, &mut sp10, It_Kind_Luigi_Fire, (*fp).facing_dir);
            let bone = fh::ftparts_get_bone_index(&*fp, FtPart_L1stNb as i32);
            let joint = (*(*fp).parts.offset(bone as isize)).joint;
            ext::efSync_Spawn(
                1287,
                gobj,
                &[joint as *mut core::ffi::c_void, &mut (*fp).facing_dir as *mut f32 as *mut core::ffi::c_void],
            );
        }
    }
}
