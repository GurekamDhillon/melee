//! src/melee/ft/kinds/ftDonkey/ftdonkeyspeciallw.c: Hand Slap (down B).
//! No fused sites (tools/rust-port/fma_sites.py: 0 fmuladd in every function). Smoke-tested
//! through recording hooks by tests/tu_ftdonkeyspeciallw.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports)]

use crate::stubs as st;
use melee_types::*;

/// Already ported (rounds 2, 3 and the hand-written module): re-exported under this TU.
pub use crate::chars::ftDonkey::ftdonkeyspeciallw::ftDk_SpecialLwLoop_IASA;
pub use crate::chars::r2::ftDonkey::ftdonkeyspeciallw::{
    doTransition, ftDk_SpecialLwEnd0_Anim, ftDk_SpecialLwEnd0_Coll, ftDk_SpecialLwEnd0_Phys,
    ftDk_SpecialLwEnd1_Anim, ftDk_SpecialLwEnd1_Coll, ftDk_SpecialLwEnd1_Phys, ftDk_SpecialLwLoop_Coll,
    ftDk_SpecialLwLoop_Phys, ftDk_SpecialLwStart_Coll, ftDk_SpecialLwStart_Phys,
};
pub use crate::chars::r3::ftDonkey::ftdonkeyspeciallw::ftDk_SpecialLw_Enter;

/// Calls into TUs that are not ported yet (settable hooks; unset they panic, except
/// `ft_80089824` which falls back to the verified group 3 port).
pub mod ext {
    use melee_types::*;
    type G = *mut HSD_GObj;
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
    pub static mut EFSYNC_SPAWN: Option<fn(i32, G, *mut HSD_JObj)> = None;
    #[allow(non_snake_case)]
    pub fn efSync_Spawn(id: i32, g: G, jobj: *mut HSD_JObj) {
        match unsafe { EFSYNC_SPAWN } {
            Some(f) => f(id, g, jobj),
            None => { unsafe { let _ = melee_world::tu::efsync::efSync_Spawn(id as _, g, &mut *(jobj as *mut _)); } },
        }
    }
    crate::port_hook!(EFASYNC_SPAWN, efAsync_Spawn, crate::adapt::efAsync_Spawn_plain, (g: G, q: *mut core::ffi::c_void, kind: u32, gfx: u32, jobj: *mut HSD_JObj));
    crate::port_hook!(FT_800892A0, ft_800892A0, crate::adapt::ft_800892A0, (g: G));
    // TODO: ftdonkey.c (`ftDk_Init_8010DB3C`) is not ported.
    pub static mut FTDK_INIT_8010DB3C: Option<fn(G) > = None;
            #[allow(non_snake_case)]
            pub fn ftDk_Init_8010DB3C(g: G)  {
                match unsafe { FTDK_INIT_8010DB3C } {
                    Some(f) => f(g),
                    None => { unsafe { let _ = crate::tu::ftdonkey::ftDk_Init_8010DB3C(g as _); } return; },
                }
            }

    pub static mut FT_80089824: Option<fn(G)> = None;
    pub fn ft_80089824(g: G) {
        match unsafe { FT_80089824 } {
            Some(f) => f(g),
            None => melee_fighter::ft0892::ft_80089824(g),
        }
    }
}

#[inline(always)]
unsafe fn fpp(gobj: *mut HSD_GObj) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}
#[inline(always)]
unsafe fn set_effect_hitlag_callbacks(fp: *mut Fighter) {
    (*fp).pre_hitlag_cb = Some(crate::chars::cb::ext__efLib_PauseAll);
    (*fp).post_hitlag_cb = Some(crate::chars::cb::ext__efLib_ResumeAll);
}

/// `callback` (static): the x21EC hook of the loop.
pub fn callback(gobj: *mut HSD_GObj) {
    ext::ft_80089824(gobj);
    ext::ft_800892A0(gobj);
}
unsafe extern "C" fn callback_cb(gobj: *mut HSD_GObj) {
    callback(gobj);
}

/// `ftDk_Init_8010DB3C` as a Fighter callback entry (`accessory4_cb`).
unsafe extern "C" fn cb_ftDk_Init_8010DB3C(gobj: *mut HSD_GObj) {
    ext::ftDk_Init_8010DB3C(gobj);
}

/// `ftDonkey_8010DE88_inner` (static).
pub fn ftDonkey_8010DE88_inner(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        if (*fp).x2219_b0() == 0 {
            let q = &mut (*fp).x60C as *mut _ as *mut core::ffi::c_void;
            ext::efAsync_Spawn(gobj, q, 1, 1228, (*(*fp).parts.offset(FtPart_TopN as isize)).joint);
            (*fp).set_x2219_b0(1);
        }
        set_effect_hitlag_callbacks(fp);
    }
}

/// `doAnim` (static): enter the loop.
pub fn doAnim(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        (*fp).x21EC = Some(callback_cb);
        (*fp).anon_2210.throw_flags = 0;
        st::Fighter_ChangeMotionState(gobj, ftDk_MS_SpecialLwLoop as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        ftDonkey_8010DE88_inner(gobj);
        (*fp).accessory4_cb = Some(cb_ftDk_Init_8010DB3C);
    }
}

pub fn ftDk_SpecialLwStart_Anim(gobj: *mut HSD_GObj) {
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        doAnim(gobj);
    }
}

pub fn ftDk_SpecialLwStart_IASA(_gobj: *mut HSD_GObj) {}

pub fn ftDk_SpecialLwLoop_Anim(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            if (*fp).mv.dk.speciallw.x0 != 0 {
                (*fp).mv.dk.speciallw.x0 = 0;
                doAnim(gobj);
            } else {
                doTransition(gobj);
            }
        }
    }
}

pub fn ftDk_SpecialLwEnd0_IASA(_gobj: *mut HSD_GObj) {}

pub fn ftDk_SpecialLwEnd1_IASA(_gobj: *mut HSD_GObj) {}

pub fn ftDk_SpecialLw_8010E0CC(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        if (*fp).x2219_b0() == 0 {
            ext::efSync_Spawn(1222, gobj, (*(*fp).parts.offset(FtPart_TransN as isize)).joint);
            (*fp).set_x2219_b0(1);
        }
        set_effect_hitlag_callbacks(fp);
        (*fp).accessory4_cb = None;
    }
}

pub fn ftDk_SpecialLw_8010E148(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        if (*fp).x2219_b0() == 0 {
            ext::efSync_Spawn(1223, gobj, (*(*fp).parts.offset(FtPart_TransN as isize)).joint);
            (*fp).set_x2219_b0(1);
        }
        set_effect_hitlag_callbacks(fp);
        (*fp).accessory4_cb = None;
    }
}

/// `ftDk_SpecialLw_8010E0CC` as a Fighter callback entry (what `chars::cb` points at).
pub unsafe extern "C" fn cb_ftDk_SpecialLw_8010E0CC(g: *mut HSD_GObj) {
    ftDk_SpecialLw_8010E0CC(g);
}
