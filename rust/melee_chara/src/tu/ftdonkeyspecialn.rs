//! src/melee/ft/kinds/ftDonkey/ftdonkeyspecialn.c: Giant Punch (neutral B).
//! No fused sites (tools/rust-port/fma_sites.py): the damage argument of `ftColl_8007ABD0` is
//! integer math and the punch velocity is a single multiply chain. The Anim functions are
//! checked against the C by tests/tu_ftdonkeyspecialn.rs; the rest is smoke-tested through
//! recording hooks.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use crate::stubs as st;
use melee_runtime::cabi::f2i32;
use melee_types::*;

/// Already ported (rounds 2, 3, 6, 12): re-exported under this TU.
pub use crate::chars::r12::ftDonkey::ftdonkeyspecialn::{
    ftDk_SpecialAirNCancel_Coll, ftDk_SpecialAirNFull_Coll, ftDk_SpecialAirNLoop_Coll,
    ftDk_SpecialAirNLoop_IASA, ftDk_SpecialAirNStart_Anim, ftDk_SpecialAirNStart_Coll,
    ftDk_SpecialAirN_Coll, ftDk_SpecialNCancel_Coll, ftDk_SpecialNFull_Coll,
    ftDk_SpecialNLoop_Coll, ftDk_SpecialNStart_Anim, ftDk_SpecialNStart_Coll, ftDk_SpecialN_Coll,
    setCallbacks,
};
pub use crate::chars::r2::ftDonkey::ftdonkeyspecialn::{
    ftDk_SpecialAirNCancel_Phys, ftDk_SpecialAirNFull_Phys, ftDk_SpecialAirNLoop_Phys,
    ftDk_SpecialAirNStart_Phys, ftDk_SpecialAirN_Phys, ftDk_SpecialNCancel_Phys,
    ftDk_SpecialNFull_Phys, ftDk_SpecialNLoop_Phys, ftDk_SpecialNStart_Phys, ftDk_SpecialN_Phys,
};
pub use crate::chars::r3::ftDonkey::ftdonkeyspecialn::{
    clearCallbacks, ftDk_SpecialAirNCancel_Anim, ftDk_SpecialNCancel_Anim,
};
pub use crate::chars::r6::ftDonkey::ftdonkeyspecialn::{
    ftDk_SpecialN_DestroyAllEffects, ftDk_SpecialN_DestroyAllEffectsPlus,
};

/// Calls into TUs that are not ported yet. Each is a settable hook; unset it panics, except the
/// ones with a verified port in melee_fighter, which the unset hook calls.
pub mod ext {
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
    /// `efSync_Spawn(gfx_id, gobj, ...)` (ef/efsync.c, variadic): shared with ftCaptain's TU.
    pub use crate::tu::ftcaptainspecialn::ext::{efSync_Spawn, EFSYNC_SPAWN};
    crate::port_hook!(FTCOLL_8007ABD0, ftColl_8007ABD0, crate::adapt::ftColl_8007ABD0, (hit: *mut HitCapsule, damage: u32, gobj: *mut HSD_GObj));
    fn ftco_800bffd0_port(fp: *mut Fighter, a: i32, b: i32) -> bool {
        melee_fighter::tu::ftcolanim::ftco_800bffd0(unsafe { &mut *fp }, a, b)
    }
    hook_or!(FTCO_800BFFD0, ftCo_800BFFD0, ftco_800bffd0_port, (fp: *mut Fighter, a: i32, b: i32) -> bool);
    hook_or!(FTCO_8009917C, ftCo_8009917C, melee_fighter::tu::ftCo_Escape::ftCo_8009917C, (gobj: *mut HSD_GObj) -> bool);
}

#[inline(always)]
unsafe fn fpp(gobj: *mut HSD_GObj) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}

/// `updateVelocity` (static): punch ground velocity from the charge count.
pub fn updateVelocity(gobj: *mut HSD_GObj) {
    melee_core::ct!("updateVelocity");
    unsafe {
        let fp = fpp(gobj);
        let da = (*fp).dat_attrs as *mut ftDonkeyAttributes;
        (*fp).gr_vel = (*fp).facing_dir
            * ((*da).SpecialN.x34_PUNCH_HORIZONTAL_VEL * ((*fp).mv.dk.specialn.xC as f32));
    }
}

unsafe fn enter_common(gobj: *mut HSD_GObj, full_ms: i32, start_ms: i32) {
    let fp = fpp(gobj);
    let da = (*fp).dat_attrs as *mut ftDonkeyAttributes;
    if (*fp).u.dk.x222C == (*da).SpecialN.x2C_MAX_ARM_SWINGS {
        st::Fighter_ChangeMotionState(gobj, full_ms, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        (*fp).mv.dk.specialn.x8 = 1;
        (*fp).mv.dk.specialn.xC = (*fp).u.dk.x222C;
        (*fp).u.dk.x222C = 0;
    } else {
        st::Fighter_ChangeMotionState(gobj, start_ms, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        (*fp).mv.dk.specialn.x8 = 0;
        (*fp).mv.dk.specialn.xC = 0;
    }
    st::Fighter_ClearCmdVars(&mut *fp);
    (*fp).mv.dk.specialn.x0 = 0;
    (*fp).mv.dk.specialn.x4 = 0;
    (*fp).mv.dk.specialn.x14 = -1;
    (*fp).mv.dk.specialn.x10 = -1;
}

pub fn ftDk_SpecialN_Enter(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftDk_SpecialN_Enter");
    unsafe {
        let fp = fpp(gobj);
        enter_common(gobj, ftDk_MS_SpecialNFull as i32, ftDk_MS_SpecialNStart as i32);
        st::ftCommon_8007D7FC(&mut *fp);
        (*fp).self_vel.y = 0.0;
        setCallbacks(gobj);
        st::ftAnim_8006EBA4(gobj);
    }
}

pub fn ftDk_SpecialAirN_Enter(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftDk_SpecialAirN_Enter");
    unsafe {
        enter_common(gobj, ftDk_MS_SpecialAirNFull as i32, ftDk_MS_SpecialAirNStart as i32);
        setCallbacks(gobj);
        st::ftAnim_8006EBA4(gobj);
    }
}

pub fn ftDk_SpecialNLoop_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftDk_SpecialNLoop_Anim");
    unsafe {
        let fp = fpp(gobj);
        let da = (*fp).dat_attrs as *mut ftDonkeyAttributes;
        if (*fp).cur_anim_frame == 0.0 {
            (*fp).u.dk.x222C += 1;
            if (*fp).u.dk.x222C >= (*da).SpecialN.x2C_MAX_ARM_SWINGS {
                ext::ftCo_800BFFD0(fp, 57, 0);
                (*fp).u.dk.x222C = (*da).SpecialN.x2C_MAX_ARM_SWINGS;
                clearCallbacks(gobj);
                st::ft_8008A2BC(gobj);
            }
        }
    }
}

/// The punch-effect spawn all four punch Anim functions open with (cmd_vars[0] latches `x8` to
/// 1, then the effect spawns once).
unsafe fn punch_effect(gobj: *mut HSD_GObj, fp: *mut Fighter) {
    if (*fp).cmd_vars[0] != 0 && (*fp).mv.dk.specialn.x8 == 0 {
        (*fp).mv.dk.specialn.x8 += 1;
    }
    if (*fp).mv.dk.specialn.x8 == 1 {
        (*fp).mv.dk.specialn.x8 += 1;
        let top = (*(*fp).parts.add(FtPart_TopN as usize)).joint as *mut core::ffi::c_void;
        let dir = &mut (*fp).facing_dir as *mut f32 as *mut core::ffi::c_void;
        let id = if (*fp).ground_or_air == GA_Air { 1225 } else { 1224 };
        ext::efSync_Spawn(id, gobj, &[top, dir]);
    }
}

/// Scale both hit capsules' damage by the charge (`x10`/`x14` latch the base damage).
unsafe fn scale_damage(gobj: *mut HSD_GObj, fp: *mut Fighter, da: *mut ftDonkeyAttributes) {
    let per = (*da).SpecialN.x30_DAMAGE_PER_SWING;
    if (*fp).mv.dk.specialn.x10 == -1 {
        (*fp).mv.dk.specialn.x10 = f2i32((*fp).x914[0].damage as f64);
    }
    let d0 = (*fp).mv.dk.specialn.x10.wrapping_add((*fp).mv.dk.specialn.xC.wrapping_mul(per));
    ext::ftColl_8007ABD0(&mut (*fp).x914[0], d0 as u32, gobj);
    if (*fp).mv.dk.specialn.x14 == -1 {
        (*fp).mv.dk.specialn.x14 = f2i32((*fp).x914[1].damage as f64);
    }
    let d1 = (*fp).mv.dk.specialn.x14.wrapping_add((*fp).mv.dk.specialn.xC.wrapping_mul(per));
    ext::ftColl_8007ABD0(&mut (*fp).x914[1], d1 as u32, gobj);
}

pub fn ftDk_SpecialN_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftDk_SpecialN_Anim");
    unsafe {
        let fp = fpp(gobj);
        let da = (*fp).dat_attrs as *mut ftDonkeyAttributes;
        punch_effect(gobj, fp);
        if (*fp).x914[0].state != HitCapsule_Disabled {
            if (*fp).mv.dk.specialn.x4 == 0 {
                (*fp).mv.dk.specialn.x4 = 1;
            }
            scale_damage(gobj, fp, da);
        }
        if (*fp).mv.dk.specialn.x4 == 1 {
            (*fp).mv.dk.specialn.x4 = 2;
            updateVelocity(gobj);
        }
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            (*fp).u.dk.x222C = 0;
            clearCallbacks(gobj);
            st::ft_8008A2BC(gobj);
        }
    }
}

pub fn ftDk_SpecialNFull_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftDk_SpecialNFull_Anim");
    unsafe {
        let fp = fpp(gobj);
        punch_effect(gobj, fp);
        if (*fp).x914[0].state != HitCapsule_Disabled && (*fp).mv.dk.specialn.x4 == 0 {
            (*fp).mv.dk.specialn.x4 = 1;
        }
        if (*fp).mv.dk.specialn.x4 == 1 {
            (*fp).mv.dk.specialn.x4 = 2;
            updateVelocity(gobj);
        }
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            (*fp).u.dk.x222C = 0;
            clearCallbacks(gobj);
            st::ft_8008A2BC(gobj);
        }
    }
}

pub fn ftDk_SpecialAirNLoop_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftDk_SpecialAirNLoop_Anim");
    unsafe {
        let fp = fpp(gobj);
        let da = (*fp).dat_attrs as *mut ftDonkeyAttributes;
        if (*fp).cur_anim_frame == 0.0 {
            (*fp).u.dk.x222C += 1;
            if (*fp).u.dk.x222C >= (*da).SpecialN.x2C_MAX_ARM_SWINGS {
                ext::ftCo_800BFFD0(fp, 57, 0);
                (*fp).u.dk.x222C = (*da).SpecialN.x2C_MAX_ARM_SWINGS;
                clearCallbacks(gobj);
                st::ftCo_Fall_Enter(gobj);
            }
        }
    }
}

/// End of the aerial punch: fall, or land with the attribute's landing lag.
unsafe fn air_end(gobj: *mut HSD_GObj, fp: *mut Fighter, da: *mut ftDonkeyAttributes) {
    (*fp).u.dk.x222C = 0;
    clearCallbacks(gobj);
    if (*da).SpecialN.x38_LANDING_LAG == 0.0 {
        st::ftCo_Fall_Enter(gobj);
    } else {
        st::ftCo_80096900(gobj, 1, 0, 1, 1.0, (*da).SpecialN.x38_LANDING_LAG);
    }
}

pub fn ftDk_SpecialAirN_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftDk_SpecialAirN_Anim");
    unsafe {
        let fp = fpp(gobj);
        let da = (*fp).dat_attrs as *mut ftDonkeyAttributes;
        punch_effect(gobj, fp);
        if (*fp).x914[0].state != HitCapsule_Disabled {
            scale_damage(gobj, fp, da);
        }
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            air_end(gobj, fp, da);
        }
    }
}

pub fn ftDk_SpecialAirNFull_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftDk_SpecialAirNFull_Anim");
    unsafe {
        let fp = fpp(gobj);
        let da = (*fp).dat_attrs as *mut ftDonkeyAttributes;
        punch_effect(gobj, fp);
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            air_end(gobj, fp, da);
        }
    }
}

pub fn ftDk_SpecialNStart_IASA(_gobj: *mut HSD_GObj) { melee_core::ct!("ftDk_SpecialNStart_IASA");}

pub fn ftDk_SpecialNLoop_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftDk_SpecialNLoop_IASA");
    unsafe {
        let fp = fpp(gobj);
        if !ext::ftCo_8009917C(gobj) {
            if ((*fp).input.pressed_buttons as u32) & (HSD_PAD_B as u32) != 0 {
                st::Fighter_ChangeMotionState(gobj, ftDk_MS_SpecialN as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
                (*fp).mv.dk.specialn.xC = (*fp).u.dk.x222C;
                (*fp).u.dk.x222C = 0;
                setCallbacks(gobj);
                st::ftAnim_8006EBA4(gobj);
            }
            if ((*fp).input.pressed_buttons as u32) & HSD_PAD_LR != 0 {
                (*fp).mv.dk.specialn.x0 = 1;
            }
            if (*fp).cur_anim_frame == 0.0 && (*fp).mv.dk.specialn.x0 != 0 {
                st::Fighter_ChangeMotionState(gobj, ftDk_MS_SpecialNCancel as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
                setCallbacks(gobj);
            }
        }
    }
}

pub fn ftDk_SpecialNCancel_IASA(_gobj: *mut HSD_GObj) { melee_core::ct!("ftDk_SpecialNCancel_IASA");}
pub fn ftDk_SpecialN_IASA(_gobj: *mut HSD_GObj) { melee_core::ct!("ftDk_SpecialN_IASA");}
pub fn ftDk_SpecialNFull_IASA(_gobj: *mut HSD_GObj) { melee_core::ct!("ftDk_SpecialNFull_IASA");}
pub fn ftDk_SpecialAirNStart_IASA(_gobj: *mut HSD_GObj) { melee_core::ct!("ftDk_SpecialAirNStart_IASA");}
pub fn ftDk_SpecialAirNCancel_IASA(_gobj: *mut HSD_GObj) { melee_core::ct!("ftDk_SpecialAirNCancel_IASA");}
pub fn ftDk_SpecialAirN_IASA(_gobj: *mut HSD_GObj) { melee_core::ct!("ftDk_SpecialAirN_IASA");}
pub fn ftDk_SpecialAirNFull_IASA(_gobj: *mut HSD_GObj) { melee_core::ct!("ftDk_SpecialAirNFull_IASA");}
