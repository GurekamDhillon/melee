//! src/melee/ft/kinds/ftDonkey/ftdonkeyspecialhi.c: Spinning Kong (up B).
//! No fused sites (tools/rust-port/fma_sites.py: 0 fmuladd in every function). Smoke-tested
//! through recording hooks by tests/tu_ftdonkeyspecialhi.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use crate::chars::da as da_g;
#[allow(unused_imports)]
use melee_fighter::ftcommon as co;
#[allow(unused_imports)]
use melee_types::{ftDonkeyAttributes};
#[allow(unused_imports)]
use melee_types::{Fighter, HSD_GObj};
use crate::chars::caps_max_jumps;
use crate::stubs as st;
use melee_fighter::ftcommon as fc;
use melee_types::*;





/// Calls into TUs that are not ported yet (settable hooks; unset they panic).
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
                    None => melee_core::unported!(concat!("TODO: ", stringify!($name), " is not ported")),
                }
            }
        };
    }
    pub static mut EFSYNC_SPAWN: Option<fn(i32, G, *mut HSD_JObj)> = None;
    #[allow(non_snake_case)]
    pub fn efSync_Spawn(id: i32, g: G, jobj: *mut HSD_JObj) {
        match unsafe { EFSYNC_SPAWN } {
            Some(f) => f(id, g, jobj),
            None => { unsafe { melee_world::tu::efsync::efSync_Spawn(id as _, g, &[jobj as *const core::ffi::c_void]); } },
        }
    }
    pub static mut FTCLIFFCOMMON_80081370: Option<fn(G) > = None;
            #[allow(non_snake_case)]
            pub fn ftCliffCommon_80081370(g: G)  {
                match unsafe { FTCLIFFCOMMON_80081370 } {
                    Some(f) => f(g),
                    None => { unsafe { melee_fighter::tu::ftcliffcommon::ftcliffcommon_80081370(g as _); } return; },
                }
            }
}

/// `coll_mf` (static const).
pub const coll_mf: u32 = (Ft_MF_SkipMatAnim
    | Ft_MF_SkipColAnim
    | Ft_MF_UpdateCmd
    | Ft_MF_SkipItemVis
    | Ft_MF_Unk19
    | Ft_MF_SkipModelPartVis
    | Ft_MF_SkipModelFlags
    | Ft_MF_Unk27) as u32;

#[inline(always)]
unsafe fn fpp(gobj: *mut HSD_GObj) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}
#[inline(always)]
unsafe fn da(fp: *mut Fighter) -> *const ftDonkeyAttributes {
    (*fp).dat_attrs as *const ftDonkeyAttributes
}

/// Spawns the spin effect (1226) on TopN.
unsafe fn spawn_effect(gobj: *mut HSD_GObj, fp: *mut Fighter) {
    ext::efSync_Spawn(1226, gobj, (*(*fp).parts.offset(FtPart_TopN as isize)).joint);
}

pub fn ftDk_SpecialHi_Enter(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftDk_SpecialHi_Enter");
    unsafe {
        let fp = fpp(gobj);
        let a = da(fp);
        st::Fighter_ChangeMotionState(gobj, ftDk_MS_SpecialHi as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        setCallbacks(gobj);
        st::Fighter_ClearCmdVars(&mut *fp);
        fc::clamp_ground_vel(&mut *fp, (*a).SpecialHi.x54_GROUNDED_HORIZONTAL_VELOCITY);
        (*fp).self_vel.x = (*fp).gr_vel;
        (*fp).self_vel.y = 0.0;
        (*fp).x1968_jumpsUsed = caps_max_jumps(&*fp) as u8;
        st::ftAnim_8006EBA4(gobj);
        spawn_effect(gobj, fp);
    }
}

pub fn ftDk_SpecialAirHi_Enter(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftDk_SpecialAirHi_Enter");
    unsafe {
        let fp = fpp(gobj);
        let a = da(fp);
        st::Fighter_ChangeMotionState(gobj, ftDk_MS_SpecialAirHi as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        setCallbacks(gobj);
        st::Fighter_ClearCmdVars(&mut *fp);
        fc::clamp_self_vel_x(&mut *fp, (*a).SpecialHi.x54_GROUNDED_HORIZONTAL_VELOCITY);
        (*fp).self_vel.y = (*a).SpecialHi.x4C_AERIAL_VERTICAL_VELOCITY;
        (*fp).x1968_jumpsUsed = caps_max_jumps(&*fp) as u8;
        st::ftAnim_8006EBA4(gobj);
        spawn_effect(gobj, fp);
    }
}

pub fn ftDk_SpecialAirHi_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftDk_SpecialAirHi_Anim");
    unsafe {
        let fp = fpp(gobj);
        let a = da(fp);
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            st::ftCommon_8007D60C(&mut *fp);
            if (*a).SpecialHi.x64_LANDING_LAG == 0.0 {
                st::ftCo_Fall_Enter(gobj);
            } else {
                st::ftCo_80096900(gobj, 1, 0, 1, 1.0, (*a).SpecialHi.x64_LANDING_LAG);
            }
        }
    }
}

pub fn ftDk_SpecialHi_IASA(_gobj: *mut HSD_GObj) { melee_core::ct!("ftDk_SpecialHi_IASA");}

pub fn ftDk_SpecialAirHi_IASA(_gobj: *mut HSD_GObj) { melee_core::ct!("ftDk_SpecialAirHi_IASA");}

pub fn ftDk_SpecialHi_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftDk_SpecialHi_Phys");
    unsafe {
        let fp = fpp(gobj);
        let a = da(fp);
        fc::calc_ground_accel_accel_to_lstick_x(
            &mut *fp,
            0.0,
            (*a).SpecialHi.x5C_GROUNDED_MOBILITY,
            (*a).SpecialHi.x54_GROUNDED_HORIZONTAL_VELOCITY,
        );
        fc::set_self_movement_from_grounded_movement(&mut *fp);
    }
}

/// Air Phys: the already-ported Fighter-level body, entered from the GObj.
pub fn ftDk_SpecialAirHi_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftDk_SpecialAirHi_Phys");
    unsafe { ftDk_SpecialAirHi_Phys_fp(&mut *fpp(gobj)) }
}

pub fn ftDk_SpecialHi_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftDk_SpecialHi_Coll");
    unsafe {
        let fp = fpp(gobj);
        let a = da(fp);
        if st::ft_80082708(gobj) == 0 {
            st::ftCommon_8007D60C(&mut *fp);
            st::Fighter_ChangeMotionState(gobj, ftDk_MS_SpecialAirHi as i32, coll_mf, (*fp).cur_anim_frame, 1.0, 0.0, core::ptr::null_mut());
            setCallbacks(gobj);
            fc::clamp_self_vel_x(&mut *fp, (*a).SpecialHi.x58_AERIAL_HORIZONTAL_VELOCITY);
        }
    }
}

pub fn ftDk_SpecialAirHi_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftDk_SpecialAirHi_Coll");
    unsafe {
        let fp = fpp(gobj);
        let a = da(fp);
        if (*fp).self_vel.y >= 0.0 {
            if st::ft_80081D0C(gobj) != 0 {
                st::ftCommon_AirToGroundStateChange(gobj, &mut *fp, ftDk_MS_SpecialHi as i32, coll_mf);
                setCallbacks(gobj);
                fc::clamp_ground_vel(&mut *fp, (*a).SpecialHi.x54_GROUNDED_HORIZONTAL_VELOCITY);
            }
        } else if st::ft_CheckGroundAndLedge(gobj, 0) != 0 {
            st::ftCommon_AirToGroundStateChange(gobj, &mut *fp, ftDk_MS_SpecialHi as i32, coll_mf);
            setCallbacks(gobj);
            fc::clamp_ground_vel(&mut *fp, (*a).SpecialHi.x54_GROUNDED_HORIZONTAL_VELOCITY);
        } else if st::ftCliffCommon_80081298(gobj) != 0 {
            ext::ftCliffCommon_80081370(gobj);
        }
    }
}

// ---- folded from chars (consolidation step 20/21) ----
/// `setCallbacks` (static) (src/melee/ft/kinds/ftDonkey/ftdonkeyspecialhi.c).
pub fn setCallbacks(gobj: *mut HSD_GObj) {
    melee_core::ct!("setCallbacks");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        (*fp).take_dmg_cb = Some(crate::chars::cb::ftdonkey__ftDk_Init_8010D774);
        (*fp).death2_cb = Some(crate::chars::cb::ftdonkey__ftDk_Init_8010D774);
        (*fp).pre_hitlag_cb = Some(crate::chars::cb::ext__efLib_PauseAll);
        (*fp).post_hitlag_cb = Some(crate::chars::cb::ext__efLib_ResumeAll);
    }
}

/// `ftDk_SpecialHi_Anim` (src/melee/ft/kinds/ftDonkey/ftdonkeyspecialhi.c).
pub fn ftDk_SpecialHi_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftDk_SpecialHi_Anim");
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        st::ft_8008A2BC(gobj);
    }
}

/// `ftDk_SpecialHi_DestroyAllEffects` (src/melee/ft/kinds/ftDonkey/ftdonkeyspecialhi.c).
pub fn ftDk_SpecialHi_DestroyAllEffects(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftDk_SpecialHi_DestroyAllEffects");
    unsafe {
        st::efLib_DestroyAll(gobj);
    }
}

// ---- folded from chars blocks (consolidation step 20) ----
pub fn ftDk_SpecialAirHi_Phys_fp(fp: &mut Fighter) {
    melee_core::ct!("ftDk_SpecialAirHi_Phys_fp");
    let a: &ftDonkeyAttributes = da_g(fp);
    let grav_mul = if fp.cmd_vars[0] != 0 { 1.0 } else { a.SpecialHi.x50_AERIAL_GRAVITY };
    co::fall(fp, grav_mul * fp.co_attrs.gravity, fp.co_attrs.terminal_velocity);
    co::calc_self_accel_drift_simple(
        fp,
        0.0,
        a.SpecialHi.x60_AERIAL_MOBILITY,
        a.SpecialHi.x58_AERIAL_HORIZONTAL_VELOCITY,
    );
}
