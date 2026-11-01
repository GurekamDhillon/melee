//! src/melee/ft/kinds/ftDonkey/ftdonkeyspecialn.c: Giant Punch (neutral B).
//! No fused sites (tools/rust-port/fma_sites.py): the damage argument of `ftColl_8007ABD0` is
//! integer math and the punch velocity is a single multiply chain. The Anim functions are
//! checked against the C by tests/tu_ftdonkeyspecialn.rs; the rest is smoke-tested through
//! recording hooks.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use melee_types::{Fighter, HSD_GObj};
use crate::stubs as st;
use melee_runtime::cabi::f2i32;
use melee_types::*;

/// Already ported (rounds 2, 3, 6, 12): re-exported under this TU.





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

// ---- folded from chars (consolidation step 20/21) ----
/// `setCallbacks` (static) (src/melee/ft/kinds/ftDonkey/ftdonkeyspecialn.c).
pub fn setCallbacks(gobj: *mut HSD_GObj) {
    melee_core::ct!("setCallbacks");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        (*fp).take_dmg_cb = Some(crate::chars::cb::ftdonkey__ftDk_Init_8010D774);
        (*fp).death2_cb = Some(crate::chars::cb::ftdonkey__ftDk_Init_8010D774);
        (*fp).take_dmg_2_cb = Some(crate::chars::cb::ftdonkeyspecialn__ftDk_SpecialN_DestroyAllEffects);
        (*fp).pre_hitlag_cb = Some(crate::chars::cb::ext__efLib_PauseAll);
        (*fp).post_hitlag_cb = Some(crate::chars::cb::ext__efLib_ResumeAll);
    }
}

/// `ftDk_SpecialNStart_Anim` (src/melee/ft/kinds/ftDonkey/ftdonkeyspecialn.c).
pub fn ftDk_SpecialNStart_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftDk_SpecialNStart_Anim");
    unsafe {
        if (st::ftAnim_IsFramesRemaining(gobj) == 0) {
            st::Fighter_ChangeMotionState(gobj, 370 /* ftDk_MS_SpecialNLoop */, 0u32, 0.0_f32, 1.0_f32, 0.0_f32, core::ptr::null_mut());
            crate::chars::r12::ftDonkey::ftdonkeyspecialn::setCallbacks(gobj);
        }
    }
}

/// `ftDk_SpecialAirNStart_Anim` (src/melee/ft/kinds/ftDonkey/ftdonkeyspecialn.c).
pub fn ftDk_SpecialAirNStart_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftDk_SpecialAirNStart_Anim");
    unsafe {
        if (st::ftAnim_IsFramesRemaining(gobj) == 0) {
            st::Fighter_ChangeMotionState(gobj, 375 /* ftDk_MS_SpecialAirNLoop */, 0u32, 0.0_f32, 1.0_f32, 0.0_f32, core::ptr::null_mut());
            crate::chars::r12::ftDonkey::ftdonkeyspecialn::setCallbacks(gobj);
        }
    }
}

/// `ftDk_SpecialAirNLoop_IASA` (src/melee/ft/kinds/ftDonkey/ftdonkeyspecialn.c).
pub fn ftDk_SpecialAirNLoop_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftDk_SpecialAirNLoop_IASA");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        if ((((*fp).input.pressed_buttons as u32) & 0x200u32 /* HSD_PAD_B */) != 0) {
            st::Fighter_ChangeMotionState(gobj, 377 /* ftDk_MS_SpecialAirN */, 0u32, 0.0_f32, 1.0_f32, 0.0_f32, core::ptr::null_mut());
            (*fp).mv.dk.specialn.xC = (*fp).u.dk.x222C;
            (*fp).u.dk.x222C = 0_i32;
            crate::chars::r12::ftDonkey::ftdonkeyspecialn::setCallbacks(gobj);
            st::ftAnim_8006EBA4(gobj);
        }
        if ((((*fp).input.pressed_buttons as u32) & 0x80000000u32 /* HSD_PAD_LR */) != 0) {
            (*fp).mv.dk.specialn.x0 = 1_i32;
        }
        if (((*fp).cur_anim_frame == f32::from_bits(0x00000000) /* 0 */) && (((*fp).mv.dk.specialn.x0 as i32) != 0i32 /* 0 */)) {
            st::Fighter_ChangeMotionState(gobj, 376 /* ftDk_MS_SpecialAirNCancel */, 0u32, 0.0_f32, 1.0_f32, 0.0_f32, core::ptr::null_mut());
            crate::chars::r12::ftDonkey::ftdonkeyspecialn::setCallbacks(gobj);
        }
    }
}

/// `ftDk_SpecialNStart_Coll` (src/melee/ft/kinds/ftDonkey/ftdonkeyspecialn.c).
pub fn ftDk_SpecialNStart_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftDk_SpecialNStart_Coll");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        if (st::ft_80082708(gobj) == 0) {
            st::ftCommon_GroundToAirStateChange(gobj, &mut *fp, 374 /* ftDk_MS_SpecialAirNStart */, 0xC4C5080u32 /* ftDk_MF_SpecialN_CollCancel */);
            crate::chars::r12::ftDonkey::ftdonkeyspecialn::setCallbacks(gobj);
        }
    }
}

/// `ftDk_SpecialNLoop_Coll` (src/melee/ft/kinds/ftDonkey/ftdonkeyspecialn.c).
pub fn ftDk_SpecialNLoop_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftDk_SpecialNLoop_Coll");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        if (st::ft_80082708(gobj) == 0) {
            st::ftCommon_GroundToAirStateChange(gobj, &mut *fp, 375 /* ftDk_MS_SpecialAirNLoop */, 0xC4C5080u32 /* ftDk_MF_SpecialN_CollCancel */);
            crate::chars::r12::ftDonkey::ftdonkeyspecialn::setCallbacks(gobj);
        }
    }
}

/// `ftDk_SpecialNCancel_Coll` (src/melee/ft/kinds/ftDonkey/ftdonkeyspecialn.c).
pub fn ftDk_SpecialNCancel_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftDk_SpecialNCancel_Coll");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        if (st::ft_80082708(gobj) == 0) {
            st::ftCommon_GroundToAirStateChange(gobj, &mut *fp, 376 /* ftDk_MS_SpecialAirNCancel */, 0xC4C5080u32 /* ftDk_MF_SpecialN_CollCancel */);
            crate::chars::r12::ftDonkey::ftdonkeyspecialn::setCallbacks(gobj);
        }
    }
}

/// `ftDk_SpecialN_Coll` (src/melee/ft/kinds/ftDonkey/ftdonkeyspecialn.c).
pub fn ftDk_SpecialN_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftDk_SpecialN_Coll");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        if (st::ft_800827A0(gobj) == 0) {
            st::ftCommon_GroundToAirStateChange(gobj, &mut *fp, 377 /* ftDk_MS_SpecialAirN */, 0xC4D508Eu32 /* ftDk_MF_SpecialN_Coll */);
            crate::chars::r12::ftDonkey::ftdonkeyspecialn::setCallbacks(gobj);
        }
    }
}

/// `ftDk_SpecialNFull_Coll` (src/melee/ft/kinds/ftDonkey/ftdonkeyspecialn.c).
pub fn ftDk_SpecialNFull_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftDk_SpecialNFull_Coll");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        if (st::ft_800827A0(gobj) == 0) {
            st::ftCommon_GroundToAirStateChange(gobj, &mut *fp, 378 /* ftDk_MS_SpecialAirNFull */, 0xC4D508Eu32 /* ftDk_MF_SpecialN_Coll */);
            crate::chars::r12::ftDonkey::ftdonkeyspecialn::setCallbacks(gobj);
        }
    }
}

/// `ftDk_SpecialAirNStart_Coll` (src/melee/ft/kinds/ftDonkey/ftdonkeyspecialn.c).
pub fn ftDk_SpecialAirNStart_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftDk_SpecialAirNStart_Coll");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        if (st::ft_80081D0C(gobj) == 1) {
            st::ftCommon_AirToGroundStateChange(gobj, &mut *fp, 369 /* ftDk_MS_SpecialNStart */, 0xC4C5080u32 /* ftDk_MF_SpecialN_CollCancel */);
            crate::chars::r12::ftDonkey::ftdonkeyspecialn::setCallbacks(gobj);
        }
    }
}

/// `ftDk_SpecialAirNLoop_Coll` (src/melee/ft/kinds/ftDonkey/ftdonkeyspecialn.c).
pub fn ftDk_SpecialAirNLoop_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftDk_SpecialAirNLoop_Coll");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        if (st::ft_80081D0C(gobj) == 1) {
            st::ftCommon_AirToGroundStateChange(gobj, &mut *fp, 370 /* ftDk_MS_SpecialNLoop */, 0xC4C5080u32 /* ftDk_MF_SpecialN_CollCancel */);
            crate::chars::r12::ftDonkey::ftdonkeyspecialn::setCallbacks(gobj);
        }
    }
}

/// `ftDk_SpecialAirNCancel_Coll` (src/melee/ft/kinds/ftDonkey/ftdonkeyspecialn.c).
pub fn ftDk_SpecialAirNCancel_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftDk_SpecialAirNCancel_Coll");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        if (st::ft_80081D0C(gobj) == 1) {
            st::ftCommon_AirToGroundStateChange(gobj, &mut *fp, 371 /* ftDk_MS_SpecialNCancel */, 0xC4C5080u32 /* ftDk_MF_SpecialN_CollCancel */);
            crate::chars::r12::ftDonkey::ftdonkeyspecialn::setCallbacks(gobj);
        }
    }
}

/// `ftDk_SpecialAirN_Coll` (src/melee/ft/kinds/ftDonkey/ftdonkeyspecialn.c).
pub fn ftDk_SpecialAirN_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftDk_SpecialAirN_Coll");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        if (st::ft_80081D0C(gobj) != 0) {
            st::ftCommon_AirToGroundStateChange(gobj, &mut *fp, 372 /* ftDk_MS_SpecialN */, 0xC4D508Eu32 /* ftDk_MF_SpecialN_Coll */);
            crate::chars::r12::ftDonkey::ftdonkeyspecialn::setCallbacks(gobj);
        }
    }
}

/// `ftDk_SpecialAirNFull_Coll` (src/melee/ft/kinds/ftDonkey/ftdonkeyspecialn.c).
pub fn ftDk_SpecialAirNFull_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftDk_SpecialAirNFull_Coll");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        if (st::ft_80081D0C(gobj) != 0) {
            st::ftCommon_AirToGroundStateChange(gobj, &mut *fp, 373 /* ftDk_MS_SpecialNFull */, 0xC4D508Eu32 /* ftDk_MF_SpecialN_Coll */);
            crate::chars::r12::ftDonkey::ftdonkeyspecialn::setCallbacks(gobj);
        }
    }
}

/// `ftDk_SpecialNStart_Phys` (src/melee/ft/kinds/ftDonkey/ftdonkeyspecialn.c).
pub fn ftDk_SpecialNStart_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftDk_SpecialNStart_Phys");
    st::ft_80084F3C(gobj);
}

/// `ftDk_SpecialNLoop_Phys` (src/melee/ft/kinds/ftDonkey/ftdonkeyspecialn.c).
pub fn ftDk_SpecialNLoop_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftDk_SpecialNLoop_Phys");
    st::ft_80084F3C(gobj);
}

/// `ftDk_SpecialNCancel_Phys` (src/melee/ft/kinds/ftDonkey/ftdonkeyspecialn.c).
pub fn ftDk_SpecialNCancel_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftDk_SpecialNCancel_Phys");
    st::ft_80084F3C(gobj);
}

/// `ftDk_SpecialN_Phys` (src/melee/ft/kinds/ftDonkey/ftdonkeyspecialn.c).
pub fn ftDk_SpecialN_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftDk_SpecialN_Phys");
    st::ft_80084F3C(gobj);
}

/// `ftDk_SpecialNFull_Phys` (src/melee/ft/kinds/ftDonkey/ftdonkeyspecialn.c).
pub fn ftDk_SpecialNFull_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftDk_SpecialNFull_Phys");
    st::ft_80084F3C(gobj);
}

/// `ftDk_SpecialAirNStart_Phys` (src/melee/ft/kinds/ftDonkey/ftdonkeyspecialn.c).
pub fn ftDk_SpecialAirNStart_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftDk_SpecialAirNStart_Phys");
    st::ft_80084EEC(gobj);
}

/// `ftDk_SpecialAirNLoop_Phys` (src/melee/ft/kinds/ftDonkey/ftdonkeyspecialn.c).
pub fn ftDk_SpecialAirNLoop_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftDk_SpecialAirNLoop_Phys");
    st::ft_80084EEC(gobj);
}

/// `ftDk_SpecialAirNCancel_Phys` (src/melee/ft/kinds/ftDonkey/ftdonkeyspecialn.c).
pub fn ftDk_SpecialAirNCancel_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftDk_SpecialAirNCancel_Phys");
    st::ft_80084EEC(gobj);
}

/// `ftDk_SpecialAirN_Phys` (src/melee/ft/kinds/ftDonkey/ftdonkeyspecialn.c).
pub fn ftDk_SpecialAirN_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftDk_SpecialAirN_Phys");
    st::ft_80084EEC(gobj);
}

/// `ftDk_SpecialAirNFull_Phys` (src/melee/ft/kinds/ftDonkey/ftdonkeyspecialn.c).
pub fn ftDk_SpecialAirNFull_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftDk_SpecialAirNFull_Phys");
    st::ft_80084EEC(gobj);
}

/// `clearCallbacks` (static) (src/melee/ft/kinds/ftDonkey/ftdonkeyspecialn.c).
pub fn clearCallbacks(gobj: *mut HSD_GObj) {
    melee_core::ct!("clearCallbacks");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        (*fp).take_dmg_2_cb = core::mem::zeroed(); /* NULL */
        (*fp).pre_hitlag_cb = core::mem::zeroed(); /* NULL */
        (*fp).post_hitlag_cb = core::mem::zeroed(); /* NULL */
    }
}

/// `ftDk_SpecialNCancel_Anim` (src/melee/ft/kinds/ftDonkey/ftdonkeyspecialn.c).
pub fn ftDk_SpecialNCancel_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftDk_SpecialNCancel_Anim");
    unsafe {
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            crate::chars::r3::ftDonkey::ftdonkeyspecialn::clearCallbacks(gobj);
            st::ft_8008A2BC(gobj);
        }
    }
}

/// `ftDk_SpecialAirNCancel_Anim` (src/melee/ft/kinds/ftDonkey/ftdonkeyspecialn.c).
pub fn ftDk_SpecialAirNCancel_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftDk_SpecialAirNCancel_Anim");
    unsafe {
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            crate::chars::r3::ftDonkey::ftdonkeyspecialn::clearCallbacks(gobj);
            st::ftCo_Fall_Enter(gobj);
        }
    }
}

/// `ftDk_SpecialN_DestroyAllEffects` (src/melee/ft/kinds/ftDonkey/ftdonkeyspecialn.c).
pub fn ftDk_SpecialN_DestroyAllEffects(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftDk_SpecialN_DestroyAllEffects");
    unsafe {
        st::efLib_DestroyAll(gobj);
    }
}

/// `ftDk_SpecialN_DestroyAllEffectsPlus` (src/melee/ft/kinds/ftDonkey/ftdonkeyspecialn.c).
pub fn ftDk_SpecialN_DestroyAllEffectsPlus(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftDk_SpecialN_DestroyAllEffectsPlus");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        (*fp).u.dk.x222C = 0_i32;
        st::efLib_DestroyAll(gobj);
    }
}
