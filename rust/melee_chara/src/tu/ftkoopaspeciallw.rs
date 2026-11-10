//! src/melee/ft/kinds/ftKoopa/ftkoopaspeciallw.c: Whirling Fortress (down B).
//! No fused sites (tools/rust-port/fma_sites.py: 0 fmuladd in every function). Smoke-tested
//! through recording hooks by tests/tu_ftkoopaspeciallw.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use melee_types::Fighter;
#[allow(unused_imports)]
use melee_types::{HSD_GObj};
use crate::stubs as st;
use melee_fighter::ftcommon as fc;
use melee_types::*;

/// Already ported (rounds 2/3/4/10 and hand-written): re-exported under this TU.






/// Calls into TUs that are not ported yet. Each is a settable hook; unset it panics.
pub mod ext {
    use melee_types::*;
    use std::ffi::c_void;
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
    // `efSync_Spawn(gfx_id, gobj, ...)` (ef/efsync.c, variadic): the trailing pointers as a slice.
    crate::port_hook!(EFSYNC_SPAWN, efSync_Spawn, crate::adapt::efSync_Spawn, (gfx_id: i32, gobj: *mut HSD_GObj, args: &[*mut c_void]) -> *mut c_void);
    /// Recording hook; unset, the native `melee_fighter::tu::ftcliffcommon` port runs.
    pub static mut FTCLIFFCOMMON_80081370: Option<fn(*mut HSD_GObj)> = None;
    #[allow(non_snake_case)]
    pub fn ftCliffCommon_80081370(gobj: *mut HSD_GObj) {
        match unsafe { FTCLIFFCOMMON_80081370 } {
            Some(f) => f(gobj),
            None => melee_fighter::tu::ftcliffcommon::ftcliffcommon_80081370(gobj),
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

/// `fn_80134518` (static): the landing-quake effect at the feet, spawned once.
pub fn fn_80134518(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_80134518");
    unsafe {
        let fp = fpp(gobj);
        if (*fp).x2219_b0() == 0 {
            ext::efSync_Spawn(0x4D8, gobj, &[&mut (*fp).cur_pos as *mut Vec3 as *mut _]);
            (*fp).set_x2219_b0(1);
        }
        set_effect_hitlag_callbacks(fp);
        (*fp).accessory4_cb = None;
    }
}

/// `fn_80134590` (static): the spin effect on the root bone, spawned once.
pub fn fn_80134590(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_80134590");
    unsafe {
        let fp = fpp(gobj);
        if (*fp).x2219_b0() == 0 {
            ext::efSync_Spawn(0x4DF, gobj, &[(*(*fp).parts).joint as *mut _]);
            (*fp).set_x2219_b0(1);
        }
        set_effect_hitlag_callbacks(fp);
        (*fp).accessory4_cb = None;
    }
}

/// `fn_80134590` as a Fighter callback entry (`accessory4_cb`).
pub unsafe extern "C" fn cb_fn_80134590(g: *mut HSD_GObj) {
    melee_core::ct!("cb_fn_80134590");
    fn_80134590(g);
}

/// `fn_80134518` as a Fighter callback entry (`accessory4_cb`).
pub unsafe extern "C" fn cb_fn_80134518(g: *mut HSD_GObj) {
    melee_core::ct!("cb_fn_80134518");
    fn_80134518(g);
}

pub fn ftKp_SpecialAirLw_Enter(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKp_SpecialAirLw_Enter");
    unsafe {
        let fp = fpp(gobj);
        let da = (*fp).dat_attrs as *const ftKoopaAttributes;
        (*fp).self_vel.x *= (*da).x80;
        (*fp).self_vel.y *= (*da).x84;
        st::Fighter_ChangeMotionState(gobj, 0x16A, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
        ftKp_SpecialLw_Enter_inline(&mut *fp);
    }
}

pub fn ftKp_SpecialAirLw_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKp_SpecialAirLw_Phys");
    unsafe {
        let fp = fpp(gobj);
        let da = (*fp).dat_attrs as *const ftKoopaAttributes;
        st::ftCommon_Fall(&mut *fp, (*da).x8C, (*da).x90);
        fc::calc_self_accel_deaccel(&mut *fp, (*da).x88);
        if (*fp).self_vel.y < (*da).x94 || (*fp).cmd_vars[1] != 0 {
            (*fp).self_vel.y = (*da).x94;
        }
        if (*fp).cmd_vars[1] != 0 {
            (*fp).self_vel.x = 0.0;
            if (*fp).x2219_b0() == 0 {
                (*fp).accessory4_cb = Some(cb_fn_80134590);
            }
        }
    }
}

pub fn ftKp_SpecialLw_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKp_SpecialLw_Coll");
    ftKp_SpecialAirLw_Coll(gobj);
}

pub fn ftKp_SpecialAirLw_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKp_SpecialAirLw_Coll");
    unsafe {
        let fp = fpp(gobj);
        let dir = if (*fp).facing_dir < 0.0 { -1 } else { 1 };
        if st::ft_CheckGroundAndLedge(gobj, dir) != 0 {
            if (*fp).cmd_vars[0] != 0 && (*fp).self_vel.y <= 0.0 {
                ftKp_SpecialLw_80134A5C(gobj);
            } else {
                st::ftCommon_8007D5D4(&mut *fp);
                melee_fighter::ftcommon2::unlock_ecb(&mut *fp);
            }
        } else if (*fp).cmd_vars[0] != 0 && st::ftCliffCommon_80081298(gobj) != 0 {
            ext::ftCliffCommon_80081370(gobj);
        }
    }
}

// ---- folded from chars (consolidation step 20/21) ----
/// `ftKp_SpecialLw_Enter_inline` (static).
pub fn ftKp_SpecialLw_Enter_inline(fp: &mut Fighter) {
    melee_core::ct!("ftKp_SpecialLw_Enter_inline");
    fp.cmd_vars[1] = 0;
    fp.cmd_vars[0] = 0;
    unsafe { fp.x2210.x0.set_throw_flags_b0(0) };
    fp.x6A4_transNOffset.y = 0.0;
    fp.set_x2223_b4(1);
}

/// `ftKp_SpecialLw_Enter` (src/melee/ft/kinds/ftKoopa/ftkoopaspeciallw.c).
pub fn ftKp_SpecialLw_Enter(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKp_SpecialLw_Enter");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        (*fp).self_vel.y = f32::from_bits(0x00000000) /* 0.0f */;
        (*fp).self_vel.x = f32::from_bits(0x00000000) /* 0.0f */;
        st::ftCommon_8007D5D4(&mut *fp);
        st::Fighter_ChangeMotionState(gobj, 361 /* 0x169 */, 0u32, 0.0_f32, 1.0_f32, 0.0_f32, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
        crate::chars::ftKoopa::ftkoopaspeciallw::ftKp_SpecialLw_Enter_inline(&mut *fp);
    }
}

/// `ftKp_SpecialLw_80134988` (src/melee/ft/kinds/ftKoopa/ftkoopaspeciallw.c).
pub fn ftKp_SpecialLw_80134988(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKp_SpecialLw_80134988");
    st::Fighter_ChangeMotionState(gobj, 362 /* ftKp_MS_SpecialAirLw */, 0xC4C5088u32 /* 0x0C4C5088 */, 30.0_f32, 1.0_f32, 0.0_f32, core::ptr::null_mut());
}

/// `ftKp_SpecialLwLanding_Anim` (src/melee/ft/kinds/ftKoopa/ftkoopaspeciallw.c).
pub fn ftKp_SpecialLwLanding_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKp_SpecialLwLanding_Anim");
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        st::ft_8008A2BC(gobj);
    }
}

/// `ftKp_SpecialLwLanding_Phys` (src/melee/ft/kinds/ftKoopa/ftkoopaspeciallw.c).
pub fn ftKp_SpecialLwLanding_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKp_SpecialLwLanding_Phys");
    st::ft_80084F3C(gobj);
}

/// `ftKp_SpecialLwLanding_Coll` (src/melee/ft/kinds/ftKoopa/ftkoopaspeciallw.c).
pub fn ftKp_SpecialLwLanding_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKp_SpecialLwLanding_Coll");
    let fp = unsafe { &mut *((*gobj).user_data as *mut Fighter) };
    if st::ft_80082708(gobj) == 0 /* GA_Ground */ {
        st::ftCo_Fall_Enter(gobj);
    }
}

/// `ftKp_SpecialLw_Anim` (src/melee/ft/kinds/ftKoopa/ftkoopaspeciallw.c).
pub fn ftKp_SpecialLw_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKp_SpecialLw_Anim");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            crate::chars::r2::ftKoopa::ftkoopaspeciallw::ftKp_SpecialLw_80134988(gobj);
        }
    }
}

/// `ftKp_SpecialLw_80134A5C` (src/melee/ft/kinds/ftKoopa/ftkoopaspeciallw.c).
pub fn ftKp_SpecialLw_80134A5C(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKp_SpecialLw_80134A5C");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        (*fp).gr_vel = f32::from_bits(0x00000000) /* 0.0f */;
        st::ftCommon_8007D7FC(&mut *fp);
        st::Fighter_ChangeMotionState(gobj, 363 /* 0x16B */, 0x0u32 /* 0U */, 0.0_f32, 1.0_f32, 0.0_f32, core::ptr::null_mut());
        (*fp).accessory4_cb = Some(crate::chars::cb::ftkoopaspeciallw__fn_80134518);
    }
}

/// `ftKp_SpecialAirLw_Anim` (src/melee/ft/kinds/ftKoopa/ftkoopaspeciallw.c).
pub fn ftKp_SpecialAirLw_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKp_SpecialAirLw_Anim");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        if (st::ftAnim_IsFramesRemaining(gobj) == 0) {
            (*fp).cmd_vars[1] = 1_u32;
        }
    }
}

/// `ftKp_SpecialLw_Phys` (src/melee/ft/kinds/ftKoopa/ftkoopaspeciallw.c).
pub fn ftKp_SpecialLw_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKp_SpecialLw_Phys");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        st::ft_80085134(gobj);
        if ((*fp).self_vel.y < f32::from_bits(0x00000000) /* 0.0f */) {
            (*fp).self_vel.y = f32::from_bits(0x00000000) /* 0.0f */;
        }
    }
}
