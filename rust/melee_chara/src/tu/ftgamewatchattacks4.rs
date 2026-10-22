//! src/melee/ft/kinds/ftGameWatch/ftgamewatchattacks4.c: Mr. Game & Watch's forward smash and the
//! Torch (fire) item it spawns. No float arithmetic, so no fused sites
//! (tools/rust-port/fma_sites.py). Smoke-tested by tests/tu_ftgamewatchattacks4.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use crate::stubs as st;
use crate::tu::ftgamewatch::{cb_ftGw_Init_OnDamage, ext as gw_ext, ftGw_Init_8014A538};
use melee_types::*;

/// Already ported (rounds 2/4/5 and the item-check helper): re-exported under this TU.
pub use crate::chars::ftGameWatch::ftgamewatchattacks4::ftGw_AttackS4_ItemCheckTorchRemove;
pub use crate::chars::r2::ftGameWatch::ftgamewatchattacks4::{ftGw_AttackS4_Anim, ftGw_AttackS4_Phys};
pub use crate::chars::r4::ftGameWatch::ftgamewatchattacks4::ftGw_AttackS4_Enter;
pub use crate::chars::r5::ftGameWatch::ftgamewatchattacks4::ftGw_AttackS4_IASA;

/// Calls into TUs that are not ported yet. Each is a settable hook; unset it panics.
pub mod ext {
    use melee_types::*;
    pub use crate::tu::ftgamewatch::ext::{lb_8000B1CC, LB_8000B1CC};
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
    // itgamewatchfire.c
    hook!(ITGAMEWATCHFIRE_SPAWN, itGamewatchFire_Spawn,
        (owner: *mut HSD_GObj, pos: *const Vec3, part: i32, facing: f32) -> *mut HSD_GObj);
    hook!(ITGAMEWATCHFIRE_802C6A2C, itGamewatchFire_802C6A2C, (item: *mut HSD_GObj));
    hook!(ITGAMEWATCHFIRE_802C6A78, itGamewatchFire_802C6A78, (item: *mut HSD_GObj));
    hook!(ITGAMEWATCHFIRE_802C6A98, itGamewatchFire_802C6A98, (item: *mut HSD_GObj));
}

#[inline(always)]
unsafe fn fpp(gobj: *mut HSD_GObj) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}

/// `ftGw_ItemTorchSetup` as a Fighter callback entry (accessory4_cb).
pub unsafe extern "C" fn cb_ftGw_ItemTorchSetup(g: *mut HSD_GObj) {
    ftGw_ItemTorchSetup(g);
}
/// `ftGw_AttackS4_ItemTorchEnterHitlag` as a Fighter callback entry (pre_hitlag_cb).
pub unsafe extern "C" fn cb_ftGw_AttackS4_ItemTorchEnterHitlag(g: *mut HSD_GObj) {
    ftGw_AttackS4_ItemTorchEnterHitlag(g);
}
/// `ftGw_AttackS4_ItemTorchExitHitlag` as a Fighter callback entry (post_hitlag_cb).
pub unsafe extern "C" fn cb_ftGw_AttackS4_ItemTorchExitHitlag(g: *mut HSD_GObj) {
    ftGw_AttackS4_ItemTorchExitHitlag(g);
}

/// Spawn the Fire Attack Torch and set up callbacks.
pub fn ftGw_ItemTorchSetup(gobj: *mut HSD_GObj) {
    unsafe {
        let mut sp10 = Vec { x: 0.0, y: 0.0, z: 0.0 };
        let fp = fpp(gobj);
        gw_ext::lb_8000B1CC((*(*fp).parts.add(FtPart_LHandNb as usize)).joint, core::ptr::null(), &mut sp10);
        (*fp).u.gw.x2254_fireGObj = ext::itGamewatchFire_Spawn(gobj, &sp10, FtPart_LHandNb as i32, (*fp).facing_dir);
        if !(*fp).u.gw.x2254_fireGObj.is_null() {
            if (*fp).death2_cb.is_none() {
                (*fp).death2_cb = Some(cb_ftGw_Init_OnDamage);
            }
            if (*fp).take_dmg_cb.is_none() {
                (*fp).take_dmg_cb = Some(cb_ftGw_Init_OnDamage);
            }
        }
        (*fp).pre_hitlag_cb = Some(cb_ftGw_AttackS4_ItemTorchEnterHitlag);
        (*fp).post_hitlag_cb = Some(cb_ftGw_AttackS4_ItemTorchExitHitlag);
        (*fp).accessory4_cb = None;
    }
}

/// Set flags on the Torch (clear the pointer after ending its hitlag).
pub fn ftGw_AttackS4_ItemTorchSetFlag(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        ftGw_AttackS4_ItemTorchExitHitlag(gobj);
        (*fp).u.gw.x2254_fireGObj = core::ptr::null_mut();
    }
}

/// Remove the Torch on damage.
pub fn ftGw_AttackS4_ItemTorchOnDamage(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        if !(*fp).u.gw.x2254_fireGObj.is_null() {
            ext::itGamewatchFire_802C6A2C((*fp).u.gw.x2254_fireGObj);
            ftGw_AttackS4_ItemTorchSetFlag(gobj);
        }
    }
}

/// Apply hitlag to the Torch.
pub fn ftGw_AttackS4_ItemTorchEnterHitlag(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        if !(*fp).u.gw.x2254_fireGObj.is_null() {
            ext::itGamewatchFire_802C6A78((*fp).u.gw.x2254_fireGObj);
        }
    }
}

/// End hitlag for the Torch.
pub fn ftGw_AttackS4_ItemTorchExitHitlag(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        if !(*fp).u.gw.x2254_fireGObj.is_null() {
            ext::itGamewatchFire_802C6A98((*fp).u.gw.x2254_fireGObj);
        }
    }
}

/// Forward Smash Collision callback.
pub fn ftGw_AttackS4_Coll(gobj: *mut HSD_GObj) {
    st::ft_80084104(gobj);
    ftGw_Init_8014A538(gobj);
}
