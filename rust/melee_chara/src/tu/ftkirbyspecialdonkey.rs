//! src/melee/ft/kinds/ftKirby/ftkirbyspecialdonkey.c: Kirby with Donkey Kong's hat, neutral B
//! (Giant Punch). No fused sites (tools/rust-port/fma_sites.py). `sdata2_order0` is
//! `MUST_MATCH`-only (not built on PC). Stateful only: smoke-tested by
//! tests/tu_ftkirbyspecialdonkey.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use crate::stubs as st;
use melee_types::*;

/// Already ported (rounds 2, 3, 6, 11): re-exported under this TU.
pub use crate::chars::r11::ftKirby::ftkirbyspecialdonkey::{
    ftKb_DkSpecialAirNLoop_Coll, ftKb_DkSpecialAirNStart_Anim, ftKb_DkSpecialAirNStart_Coll,
    ftKb_DkSpecialAirN_Coll, ftKb_DkSpecialAirNCancel_Coll, ftKb_DkSpecialAirNFull_Coll,
    ftKb_DkSpecialNCancel_Coll, ftKb_DkSpecialNFull_Coll, ftKb_DkSpecialNLoop_Coll,
    ftKb_DkSpecialNStart_Anim, ftKb_DkSpecialNStart_Coll, ftKb_DkSpecialNStart_Coll_inline,
    ftKb_DkSpecialN_Coll,
};
pub use crate::chars::r2::ftKirby::ftkirbyspecialdonkey::{
    ftKb_DkSpecialAirNCancel_Phys, ftKb_DkSpecialAirNFull_Phys, ftKb_DkSpecialAirNLoop_Phys,
    ftKb_DkSpecialAirNStart_Phys, ftKb_DkSpecialAirN_Phys, ftKb_DkSpecialNCancel_Phys,
    ftKb_DkSpecialNFull_Phys, ftKb_DkSpecialNLoop_Phys, ftKb_DkSpecialNStart_Phys,
    ftKb_DkSpecialN_Phys,
};
pub use crate::chars::r3::ftKirby::ftkirbyspecialdonkey::{ftKb_DkSpecialAirNCancel_Anim, ftKb_DkSpecialNCancel_Anim};
pub use crate::chars::r6::ftKirby::ftkirbyspecialdonkey::ftKb_SpecialNPr_80100DE0;

/// Calls into TUs that are not ported yet. Each is a settable hook; unset it panics.
pub mod ext {
    use core::ffi::c_void;
    use melee_types::{HSD_GObj, HitCapsule};
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
    crate::port_hook!(FTCOLL_8007ABD0, ftColl_8007ABD0, crate::adapt::ftColl_8007ABD0, (hit: *mut HitCapsule, damage: i32, gobj: *mut HSD_GObj));

    /// `ftCo_800BFFD0(fp, a, b)` (colanim start): group 3's port when the hook is unset.
    pub static mut FTCO_800BFFD0: Option<fn(*mut melee_types::Fighter, i32, i32)> = None;
    #[allow(non_snake_case)]
    pub fn ftCo_800BFFD0(fp: *mut melee_types::Fighter, a: i32, b: i32) {
        match unsafe { FTCO_800BFFD0 } {
            Some(f) => f(fp, a, b),
            None => unsafe {
                melee_fighter::tu::ftcolanim::ftco_800bffd0(&mut *fp, a, b);
            },
        }
    }
}

/// `Fighter_SetEffectHitlagCallbacks(fp)` (ft/inlines.h): efLib_PauseAll / efLib_ResumeAll as the hitlag
/// callbacks (the entries panic `TODO` when the game calls them, until efLib is ported).
#[inline(always)]
unsafe fn set_effect_hitlag_cbs(fp: *mut Fighter) {
    (*fp).pre_hitlag_cb = Some(crate::chars::cb::ext__efLib_PauseAll);
    (*fp).post_hitlag_cb = Some(crate::chars::cb::ext__efLib_ResumeAll);
}

#[inline(always)]
unsafe fn fpp(gobj: *mut HSD_GObj) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}

#[inline(always)]
unsafe fn joint(fp: *mut Fighter, part: usize) -> *mut core::ffi::c_void {
    (*(*fp).parts.add(part)).joint as *mut core::ffi::c_void
}

/// `Fighter_ClearCmdVars` (ft/inlines.h).
#[inline(always)]
fn clear_cmd_vars(fp: &mut Fighter) {
    fp.cmd_vars[3] = 0;
    fp.cmd_vars[2] = 0;
    fp.cmd_vars[1] = 0;
    fp.cmd_vars[0] = 0;
}

/// `fp->pre_hitlag_cb = fp->post_hitlag_cb = NULL`.
#[inline(always)]
unsafe fn clear_hitlag_cbs(fp: *mut Fighter) {
    (*fp).pre_hitlag_cb = None;
    (*fp).post_hitlag_cb = None;
}

/// The swing-effect block shared by every `*_Anim` here: cmd_vars[0] arms x8, x8 == 1 spawns the
/// ground (1193) or air (1194) swing effect once.
unsafe fn swing_effect(gobj: *mut HSD_GObj, fp: *mut Fighter) {
    if (*fp).cmd_vars[0] != 0 && (*fp).mv.dk.specialn.x8 == 0 {
        (*fp).mv.dk.specialn.x8 += 1;
    }
    if (*fp).mv.dk.specialn.x8 == 1 {
        (*fp).mv.dk.specialn.x8 += 1;
        let facing = &mut (*fp).facing_dir as *mut f32 as *mut core::ffi::c_void;
        if (*fp).ground_or_air == GA_Air {
            ext::efSync_Spawn(1194, gobj, &[joint(fp, FtPart_TopN as usize), facing]);
        } else {
            ext::efSync_Spawn(1193, gobj, &[joint(fp, FtPart_TopN as usize), facing]);
        }
    }
}

/// `x10 + (xC * damage_increase_per_swing)` (int + int * u32: unsigned arithmetic, passed as int).
#[inline(always)]
fn swing_damage(base: i32, swings: i32, inc: u32) -> i32 {
    (base as u32).wrapping_add((swings as u32).wrapping_mul(inc)) as i32
}

/// The hitbox damage ramp of the punch swings: records each capsule's base damage once, then
/// re-applies `base + swings * increase` every frame a capsule is live.
unsafe fn ramp_hit_damage(gobj: *mut HSD_GObj, fp: *mut Fighter, da: *const ftKb_DatAttrs) {
    if (*fp).mv.dk.specialn.x10 == -1 {
        (*fp).mv.dk.specialn.x10 = (*fp).x914[0].damage as i32;
    }
    ext::ftColl_8007ABD0(
        &mut (*fp).x914[0],
        swing_damage((*fp).mv.dk.specialn.x10, (*fp).mv.dk.specialn.xC, (*da).specialn_dk_damage_increase_per_swing),
        gobj,
    );
    if (*fp).mv.dk.specialn.x14 == -1 {
        (*fp).mv.dk.specialn.x14 = (*fp).x914[1].damage as i32;
    }
    ext::ftColl_8007ABD0(
        &mut (*fp).x914[1],
        swing_damage((*fp).mv.dk.specialn.x14, (*fp).mv.dk.specialn.xC, (*da).specialn_dk_damage_increase_per_swing),
        gobj,
    );
}

/// `fp2->gr_vel = facing_dir * (grounded_punch_horizontal_velocity * xC)` (the lunge on the swing).
unsafe fn punch_lunge(fp: *mut Fighter) {
    let da2 = (*fp).dat_attrs as *const ftKb_DatAttrs;
    (*fp).gr_vel = (*fp).facing_dir * ((*da2).specialn_dk_grounded_punch_horizontal_velocity * (*fp).mv.dk.specialn.xC as f32);
}

/// `ftKb_SpecialNDk_800FF8EC`: enter the ground Giant Punch charge (or the full-charge swing).
pub fn ftKb_SpecialNDk_800FF8EC(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_SpecialNDk_800FF8EC");
    unsafe {
        let fp = fpp(gobj);
        let da = (*fp).dat_attrs as *const ftKb_DatAttrs;
        if (*fp).u.kb.xBC == (*da).specialn_dk_swings_to_full_charge {
            st::Fighter_ChangeMotionState(gobj, ftKb_MS_DkSpecialNFull as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
            (*fp).mv.dk.specialn.x8 = 1;
            (*fp).mv.dk.specialn.xC = (*fp).u.kb.xBC;
            (*fp).u.kb.xBC = 0;
        } else {
            st::Fighter_ChangeMotionState(gobj, ftKb_MS_DkSpecialNStart as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
            (*fp).mv.dk.specialn.x8 = 0;
            (*fp).mv.dk.specialn.xC = 0;
        }
        clear_cmd_vars(&mut *fp);
        (*fp).mv.dk.specialn.x0 = 0;
        (*fp).mv.dk.specialn.x4 = 0;
        (*fp).mv.dk.specialn.x14 = -1;
        (*fp).mv.dk.specialn.x10 = -1;
        st::ftCommon_8007D7FC(&mut *fp);
        (*fp).self_vel.y = 0.0;
        let fp2 = fpp(gobj);
        (*fp2).death2_cb = Some(crate::chars::cb::ftkirby__ftKb_Init_800EE74C);
        (*fp2).take_dmg_cb = Some(crate::chars::cb::ftkirby__ftKb_Init_800EE7B8);
        set_effect_hitlag_cbs(fp2);
        st::ftAnim_8006EBA4(gobj);
    }
}

/// `ftKb_SpecialNDk_800FFA10`: the aerial entry (same, without the ground-to-air `ftCommon_8007D7FC`).
pub fn ftKb_SpecialNDk_800FFA10(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_SpecialNDk_800FFA10");
    unsafe {
        let fp = fpp(gobj);
        let da = (*fp).dat_attrs as *const ftKb_DatAttrs;
        if (*fp).u.kb.xBC == (*da).specialn_dk_swings_to_full_charge {
            st::Fighter_ChangeMotionState(gobj, ftKb_MS_DkSpecialAirNFull as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
            (*fp).mv.dk.specialn.x8 = 1;
            (*fp).mv.dk.specialn.xC = (*fp).u.kb.xBC;
            (*fp).u.kb.xBC = 0;
        } else {
            st::Fighter_ChangeMotionState(gobj, ftKb_MS_DkSpecialAirNStart as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
            (*fp).mv.dk.specialn.x8 = 0;
            (*fp).mv.dk.specialn.xC = 0;
        }
        clear_cmd_vars(&mut *fp);
        (*fp).mv.dk.specialn.x0 = 0;
        (*fp).mv.dk.specialn.x4 = 0;
        (*fp).mv.dk.specialn.x14 = -1;
        (*fp).mv.dk.specialn.x10 = -1;
        let fp2 = fpp(gobj);
        (*fp2).death2_cb = Some(crate::chars::cb::ftkirby__ftKb_Init_800EE74C);
        (*fp2).take_dmg_cb = Some(crate::chars::cb::ftkirby__ftKb_Init_800EE7B8);
        set_effect_hitlag_cbs(fp2);
        st::ftAnim_8006EBA4(gobj);
    }
}

/// `ftKb_DkSpecialNLoop_Anim`: count a swing at the start of each loop; at full charge leave to Wait.
pub fn ftKb_DkSpecialNLoop_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_DkSpecialNLoop_Anim");
    unsafe {
        let fp = fpp(gobj);
        let da = (*fp).dat_attrs as *const ftKb_DatAttrs;
        if (*fp).cur_anim_frame == 0.0 {
            (*fp).u.kb.xBC += 1;
            if (*fp).u.kb.xBC >= (*da).specialn_dk_swings_to_full_charge {
                ext::ftCo_800BFFD0(fp, 58, 0);
                (*fp).u.kb.xBC = (*da).specialn_dk_swings_to_full_charge;
                clear_hitlag_cbs(fpp(gobj));
                st::ft_8008A2BC(gobj);
            }
        }
    }
}

/// `ftKb_DkSpecialN_Anim`: the ground swing.
pub fn ftKb_DkSpecialN_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_DkSpecialN_Anim");
    unsafe {
        let fp = fpp(gobj);
        let da = (*fp).dat_attrs as *const ftKb_DatAttrs;
        swing_effect(gobj, fp);
        if (*fp).x914[0].state != HitCapsule_Disabled {
            if (*fp).mv.dk.specialn.x4 == 0 {
                (*fp).mv.dk.specialn.x4 = 1;
            }
            ramp_hit_damage(gobj, fp, da);
        }
        if (*fp).mv.dk.specialn.x4 == 1 {
            (*fp).mv.dk.specialn.x4 = 2;
            punch_lunge(fpp(gobj));
        }
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            (*fp).u.kb.xBC = 0;
            clear_hitlag_cbs(fpp(gobj));
            st::ft_8008A2BC(gobj);
        }
    }
}

/// `ftKb_DkSpecialNFull_Anim`: the full-charge ground swing (no damage ramp).
pub fn ftKb_DkSpecialNFull_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_DkSpecialNFull_Anim");
    unsafe {
        let fp = fpp(gobj);
        swing_effect(gobj, fp);
        if (*fp).x914[0].state != HitCapsule_Disabled && (*fp).mv.dk.specialn.x4 == 0 {
            (*fp).mv.dk.specialn.x4 = 1;
        }
        if (*fp).mv.dk.specialn.x4 == 1 {
            (*fp).mv.dk.specialn.x4 = 2;
            punch_lunge(fpp(gobj));
        }
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            (*fp).u.kb.xBC = 0;
            clear_hitlag_cbs(fpp(gobj));
            st::ft_8008A2BC(gobj);
        }
    }
}

/// `ftKb_DkSpecialAirNLoop_Anim`: as the ground loop, leaving to Fall.
pub fn ftKb_DkSpecialAirNLoop_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_DkSpecialAirNLoop_Anim");
    unsafe {
        let fp = fpp(gobj);
        let da = (*fp).dat_attrs as *const ftKb_DatAttrs;
        if (*fp).cur_anim_frame == 0.0 {
            (*fp).u.kb.xBC += 1;
            if (*fp).u.kb.xBC >= (*da).specialn_dk_swings_to_full_charge {
                ext::ftCo_800BFFD0(fp, 58, 0);
                (*fp).u.kb.xBC = (*da).specialn_dk_swings_to_full_charge;
                clear_hitlag_cbs(fpp(gobj));
                st::ftCo_Fall_Enter(gobj);
            }
        }
    }
}

/// End of the aerial swings: freefall by `freefall_toggle` landing lag, or plain Fall.
unsafe fn air_swing_end(gobj: *mut HSD_GObj, fp: *mut Fighter, da: *const ftKb_DatAttrs) {
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        (*fp).u.kb.xBC = 0;
        clear_hitlag_cbs(fpp(gobj));
        if (*da).specialn_dk_freefall_toggle == 0.0 {
            st::ftCo_Fall_Enter(gobj);
        } else {
            st::ftCo_80096900(gobj, 1, 0, 1, 1.0, (*da).specialn_dk_freefall_toggle);
        }
    }
}

/// `ftKb_DkSpecialAirN_Anim`: the aerial swing.
pub fn ftKb_DkSpecialAirN_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_DkSpecialAirN_Anim");
    unsafe {
        let fp = fpp(gobj);
        let da = (*fp).dat_attrs as *const ftKb_DatAttrs;
        swing_effect(gobj, fp);
        if (*fp).x914[0].state != HitCapsule_Disabled {
            ramp_hit_damage(gobj, fp, da);
        }
        air_swing_end(gobj, fp, da);
    }
}

/// `ftKb_DkSpecialAirNFull_Anim`: the full-charge aerial swing.
pub fn ftKb_DkSpecialAirNFull_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_DkSpecialAirNFull_Anim");
    unsafe {
        let fp = fpp(gobj);
        let da = (*fp).dat_attrs as *const ftKb_DatAttrs;
        swing_effect(gobj, fp);
        air_swing_end(gobj, fp, da);
    }
}

pub fn ftKb_DkSpecialNStart_IASA(_gobj: *mut HSD_GObj) { melee_core::ct!("ftKb_DkSpecialNStart_IASA");}

/// `ftKb_DkSpecialNLoop_IASA`: B releases the punch, a shoulder press queues the cancel.
pub fn ftKb_DkSpecialNLoop_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_DkSpecialNLoop_IASA");
    unsafe {
        let fp = fpp(gobj);
        if !melee_fighter::tu::ftCo_Escape::ftCo_8009917C(gobj) {
            if (*fp).input.pressed_buttons & (HSD_PAD_B as u32) != 0 {
                st::Fighter_ChangeMotionState(gobj, ftKb_MS_DkSpecialN as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
                (*fp).mv.dk.specialn.xC = (*fp).u.kb.xBC;
                (*fp).u.kb.xBC = 0;
                ftKb_DkSpecialNStart_Coll_inline(gobj);
                st::ftAnim_8006EBA4(gobj);
            }
            if (*fp).input.pressed_buttons & HSD_PAD_LR != 0 {
                (*fp).mv.dk.specialn.x0 = 1;
            }
            if (*fp).cur_anim_frame == 0.0 && (*fp).mv.dk.specialn.x0 != 0 {
                st::Fighter_ChangeMotionState(gobj, ftKb_MS_DkSpecialNCancel as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
                ftKb_DkSpecialNStart_Coll_inline(gobj);
            }
        }
    }
}

pub fn ftKb_DkSpecialNCancel_IASA(_gobj: *mut HSD_GObj) { melee_core::ct!("ftKb_DkSpecialNCancel_IASA");}
pub fn ftKb_DkSpecialN_IASA(_gobj: *mut HSD_GObj) { melee_core::ct!("ftKb_DkSpecialN_IASA");}
pub fn ftKb_DkSpecialNFull_IASA(_gobj: *mut HSD_GObj) { melee_core::ct!("ftKb_DkSpecialNFull_IASA");}
pub fn ftKb_DkSpecialAirNStart_IASA(_gobj: *mut HSD_GObj) { melee_core::ct!("ftKb_DkSpecialAirNStart_IASA");}

/// `ftKb_DkSpecialAirNLoop_IASA`: as the ground loop without the `ftCo_8009917C` guard.
pub fn ftKb_DkSpecialAirNLoop_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_DkSpecialAirNLoop_IASA");
    unsafe {
        let fp = fpp(gobj);
        if (*fp).input.pressed_buttons & (HSD_PAD_B as u32) != 0 {
            st::Fighter_ChangeMotionState(gobj, ftKb_MS_DkSpecialAirN as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
            (*fp).mv.dk.specialn.xC = (*fp).u.kb.xBC;
            (*fp).u.kb.xBC = 0;
            ftKb_DkSpecialNStart_Coll_inline(gobj);
            st::ftAnim_8006EBA4(gobj);
        }
        if (*fp).input.pressed_buttons & HSD_PAD_LR != 0 {
            (*fp).mv.dk.specialn.x0 = 1;
        }
        if (*fp).cur_anim_frame == 0.0 && (*fp).mv.dk.specialn.x0 != 0 {
            st::Fighter_ChangeMotionState(gobj, ftKb_MS_DkSpecialAirNCancel as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
            ftKb_DkSpecialNStart_Coll_inline(gobj);
        }
    }
}

pub fn ftKb_DkSpecialAirNCancel_IASA(_gobj: *mut HSD_GObj) { melee_core::ct!("ftKb_DkSpecialAirNCancel_IASA");}
pub fn ftKb_DkSpecialAirN_IASA(_gobj: *mut HSD_GObj) { melee_core::ct!("ftKb_DkSpecialAirN_IASA");}
pub fn ftKb_DkSpecialAirNFull_IASA(_gobj: *mut HSD_GObj) { melee_core::ct!("ftKb_DkSpecialAirNFull_IASA");}
