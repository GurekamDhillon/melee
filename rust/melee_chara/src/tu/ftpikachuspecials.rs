//! src/melee/ft/kinds/ftPikachu/ftpikachuspecials.c: Pikachu / Pichu Skull Bash (side special).
//! Fused sites (tools/rust-port/fma_sites.py): `ftPk_SpecialS0_Anim` and `ftPk_SpecialAirS0_Anim`
//! `damage = unk3.x0 * x2C + x28` = fmadds(x0 as f32, x2C, x28); `ftPk_SpecialS_ChangeMotion_Unk10`
//! `self_vel.x = x40 * unk3.x0 + x3C` = fmadds(x40, x0 as f32, x3C) and
//! `self_vel.y = 0.5f * x44 + x44 * temp` = fmadds(0.5, x44, x44 * temp). The charge counter
//! `mv.pk.unk3.x0` is a C `int`: it converts to float in the compares and products. The hit damage
//! reaches `ftColl_8007ABD0(HitCapsule*, u32, ..)`, so the float truncates to `u32`. Smoke-tested
//! by tests/tu_ftpikachuspecials.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use crate::chars::da as da_g;
#[allow(unused_imports)]
use melee_types::{ftPikachuAttributes};
#[allow(unused_imports)]
use melee_types::{Fighter, HSD_GObj};
use crate::stubs as st;
use crate::tu::ftpeachspecials::cb_ftCommon_8007F76C;
use crate::tu::ftpikachuspecialn::{
    cb_ftPk_SpecialN_80124DC8, cb_ftPk_SpecialN_SpawnEffect0, cb_ftPk_SpecialN_SpawnEffect1,
};
use melee_core::fp::fmadds;
use melee_fighter::ftcommon as co;
use melee_types::*;

/// Already ported (rounds 2/3 and the original pass): re-exported under this TU.



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
                    None => melee_core::unported!(concat!("TODO: ", stringify!($name), " is not ported")),
                }
            }
        };
    }
    crate::port_hook!(FTCOLL_8007ABD0, ftColl_8007ABD0, crate::adapt::ftColl_8007ABD0, (hit: *mut HitCapsule, damage: u32, gobj: *mut HSD_GObj));
}

type G = *mut HSD_GObj;

#[inline(always)]
unsafe fn fpp(gobj: G) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}
#[inline(always)]
unsafe fn da(fp: *mut Fighter) -> *mut ftPikachuAttributes {
    (*fp).dat_attrs as *mut ftPikachuAttributes
}

/// The three `fp->cmd_vars`-free motion changes below pass these `Ft_MF_*` words.
const TRANSITION_FLAGS3: u32 = (Ft_MF_KeepGfx | Ft_MF_SkipHit) as u32;

/// Gobj entries for the ported `&mut Fighter` callbacks.
pub fn ftPk_SpecialAirS1_Phys(gobj: G) {
    melee_core::ct!("ftPk_SpecialAirS1_Phys");
    unsafe { crate::chars::ftPikachu::ftpikachuspecials::ftPk_SpecialAirS1_Phys(&mut *fpp(gobj)) }
}
pub fn ftPk_SpecialAirSEnd_Phys(gobj: G) {
    melee_core::ct!("ftPk_SpecialAirSEnd_Phys");
    unsafe { crate::chars::ftPikachu::ftpikachuspecials::ftPk_SpecialAirSEnd_Phys(&mut *fpp(gobj)) }
}
pub fn ftPk_SpecialSEnd_Phys(gobj: G) {
    melee_core::ct!("ftPk_SpecialSEnd_Phys");
    unsafe { crate::chars::ftPikachu::ftpikachuspecials::ftPk_SpecialSEnd_Phys(&mut *fpp(gobj)) }
}

pub fn ftPk_SpecialS_Enter(gobj: G) {
    melee_core::ct!("ftPk_SpecialS_Enter");
    unsafe {
        let fp = fpp(gobj);
        let sa = da(fp);
        (*fp).x21EC = Some(cb_ftPk_SpecialN_80124DC8);
        (*fp).gr_vel /= (*sa).x30;
        st::Fighter_ChangeMotionState(gobj, 343, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
    }
}

pub fn ftPk_SpecialAirS_Enter(gobj: G) {
    melee_core::ct!("ftPk_SpecialAirS_Enter");
    unsafe {
        let fp = fpp(gobj);
        let sa = da(fp);
        (*fp).x21EC = Some(cb_ftPk_SpecialN_80124DC8);
        (*fp).self_vel.x /= (*sa).x30;
        (*fp).self_vel.y = 0.0;
        st::Fighter_ChangeMotionState(gobj, 348, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
    }
}

pub fn ftPk_SpecialS_ZeroVelocity(gobj: G) {
    melee_core::ct!("ftPk_SpecialS_ZeroVelocity");
    unsafe {
        let fp = fpp(gobj);
        (*fp).self_vel.x = 0.0;
        if (*fp).self_vel.y >= 0.0 {
            (*fp).self_vel.y = 0.0;
        }
        ftPk_SpecialS_ChangeMotion_Unk12(gobj);
    }
}
/// `ftPk_SpecialS_ZeroVelocity` as the `deal_dmg_cb` entry.
pub unsafe extern "C" fn cb_ftPk_SpecialS_ZeroVelocity(g: G) {
    melee_core::ct!("cb_ftPk_SpecialS_ZeroVelocity");
    ftPk_SpecialS_ZeroVelocity(g);
}

pub fn ftPk_SpecialSStart_Anim(gobj: G) {
    melee_core::ct!("ftPk_SpecialSStart_Anim");
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        ftPk_SpecialS_ChangeMotion_Unk04(gobj);
    }
}

pub fn ftPk_SpecialAirSStart_Anim(gobj: G) {
    melee_core::ct!("ftPk_SpecialAirSStart_Anim");
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        ftPk_SpecialS_ChangeMotion_Unk05(gobj);
    }
}

pub fn ftPk_SpecialSStart_IASA(_gobj: G) { melee_core::ct!("ftPk_SpecialSStart_IASA");}

pub fn ftPk_SpecialAirSStart_IASA(_gobj: G) { melee_core::ct!("ftPk_SpecialAirSStart_IASA");}

pub fn ftPk_SpecialSStart_Phys(gobj: G) {
    melee_core::ct!("ftPk_SpecialSStart_Phys");
    unsafe {
        let fp = fpp(gobj);
        let sa = da(fp);
        co::calc_ground_accel_deaccel(&mut *fp, (*sa).specials_start_friction);
        co::set_self_movement_from_grounded_movement(&mut *fp);
    }
}

pub fn ftPk_SpecialAirSStart_Phys(gobj: G) {
    melee_core::ct!("ftPk_SpecialAirSStart_Phys");
    unsafe {
        let fp = fpp(gobj);
        let sa = da(fp);
        if (*fp).cmd_vars[0] != 0 {
            co::fall(&mut *fp, (*sa).specials_start_gravity, (*fp).co_attrs.terminal_velocity);
        }
        co::calc_self_accel_deaccel(&mut *fp, (*sa).specials_start_friction);
    }
}

pub fn ftPk_SpecialSHold_Anim(gobj: G) {
    melee_core::ct!("ftPk_SpecialSHold_Anim");
    unsafe {
        let fp = fpp(gobj);
        let sa = da(fp);
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            st::ftCommon_8007DB24(gobj);
            (*fp).accessory4_cb = Some(cb_ftPk_SpecialN_SpawnEffect0);
        }
        (*fp).mv.pk.unk3.x0 = (*fp).mv.pk.unk3.x0.wrapping_add(1);
        if ((*fp).mv.pk.unk3.x0 as f32) > (*sa).x24 {
            ftPk_SpecialS_ChangeMotion_Unk08(gobj);
        }
    }
}

pub fn ftPk_SpecialAirSHold_Anim(gobj: G) {
    melee_core::ct!("ftPk_SpecialAirSHold_Anim");
    unsafe {
        let fp = fpp(gobj);
        let sa = da(fp);
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            st::ftCommon_8007DB24(gobj);
            (*fp).accessory4_cb = Some(cb_ftPk_SpecialN_SpawnEffect0);
        }
        (*fp).mv.pk.unk3.x0 = (*fp).mv.pk.unk3.x0.wrapping_add(1);
        if ((*fp).mv.pk.unk3.x0 as f32) > (*sa).x24 {
            ftPk_SpecialS_ChangeMotion_Unk09(gobj);
        }
    }
}

pub fn ftPk_SpecialSHold_IASA(gobj: G) {
    melee_core::ct!("ftPk_SpecialSHold_IASA");
    unsafe {
        let fp = fpp(gobj);
        if (*fp).input.held_buttons[0] & (HSD_PAD_B as u32) == 0 {
            ftPk_SpecialS_ChangeMotion_Unk08(gobj);
        }
    }
}

pub fn ftPk_SpecialAirSHold_IASA(gobj: G) {
    melee_core::ct!("ftPk_SpecialAirSHold_IASA");
    unsafe {
        let fp = fpp(gobj);
        if (*fp).input.held_buttons[0] & (HSD_PAD_B as u32) == 0 {
            ftPk_SpecialS_ChangeMotion_Unk09(gobj);
        }
    }
}

pub fn ftPk_SpecialS_ChangeMotion_Unk04(gobj: G) {
    melee_core::ct!("ftPk_SpecialS_ChangeMotion_Unk04");
    unsafe {
        st::Fighter_ChangeMotionState(gobj, 344, Ft_MF_KeepSfx as u32, 0.0, 1.0, 0.0, core::ptr::null_mut());
        let fp = fpp(gobj);
        (*fp).accessory4_cb = Some(cb_ftPk_SpecialN_SpawnEffect0);
    }
}

pub fn ftPk_SpecialS_ChangeMotion_Unk05(gobj: G) {
    melee_core::ct!("ftPk_SpecialS_ChangeMotion_Unk05");
    unsafe {
        st::Fighter_ChangeMotionState(gobj, 349, Ft_MF_KeepSfx as u32, 0.0, 1.0, 0.0, core::ptr::null_mut());
        let fp = fpp(gobj);
        (*fp).accessory4_cb = Some(cb_ftPk_SpecialN_SpawnEffect0);
    }
}

/// The hit-capsule damage scaling shared by the ground and air `S0_Anim`.
unsafe fn s0_hit_damage(gobj: G, fp: *mut Fighter, sa: *mut ftPikachuAttributes) {
    if (*fp).x914[0].state == HitCapsule_Enabled {
        let damage_amount = fmadds((*fp).mv.pk.unk3.x0 as f32, (*sa).x2C, (*sa).x28);
        ext::ftColl_8007ABD0(&mut (*fp).x914[0], melee_core::ppc::fcvt_u32(damage_amount), gobj);
    }
}

pub fn ftPk_SpecialS0_Anim(gobj: G) {
    melee_core::ct!("ftPk_SpecialS0_Anim");
    unsafe {
        let fp = fpp(gobj);
        let sa = da(fp);
        s0_hit_damage(gobj, fp, sa);
        if (*fp).cmd_vars[0] != 0 {
            st::ftCommon_8007D5D4(&mut *fp);
            ftPk_SpecialS_ChangeMotion_Unk10(gobj);
        }
    }
}

pub fn ftPk_SpecialAirS0_Anim(gobj: G) {
    melee_core::ct!("ftPk_SpecialAirS0_Anim");
    unsafe {
        let fp = fpp(gobj);
        let sa = da(fp);
        s0_hit_damage(gobj, fp, sa);
        if (*fp).cmd_vars[0] != 0 {
            ftPk_SpecialS_ChangeMotion_Unk10(gobj);
        }
    }
}

pub fn ftPk_SpecialS0_IASA(_gobj: G) { melee_core::ct!("ftPk_SpecialS0_IASA");}

pub fn ftPk_SpecialAirS0_IASA(_gobj: G) { melee_core::ct!("ftPk_SpecialAirS0_IASA");}

pub fn ftPk_SpecialS_ChangeMotion_Unk08(gobj: G) {
    melee_core::ct!("ftPk_SpecialS_ChangeMotion_Unk08");
    unsafe {
        st::Fighter_ChangeMotionState(gobj, 347, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        let fp = fpp(gobj);
        (*fp).cmd_vars[0] = 0;
        st::ftCommon_8007DB24(gobj);
        (*fp).accessory4_cb = Some(cb_ftPk_SpecialN_SpawnEffect1);
    }
}

pub fn ftPk_SpecialS_ChangeMotion_Unk09(gobj: G) {
    melee_core::ct!("ftPk_SpecialS_ChangeMotion_Unk09");
    unsafe {
        st::Fighter_ChangeMotionState(gobj, 352, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        let fp = fpp(gobj);
        (*fp).cmd_vars[0] = 0;
        st::ftCommon_8007DB24(gobj);
        (*fp).accessory4_cb = Some(cb_ftPk_SpecialN_SpawnEffect1);
    }
}

pub fn ftPk_SpecialS1_Anim(_gobj: G) { melee_core::ct!("ftPk_SpecialS1_Anim");}

pub fn ftPk_SpecialAirS1_Anim(gobj: G) {
    melee_core::ct!("ftPk_SpecialAirS1_Anim");
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        ftPk_SpecialS_ChangeMotion_Unk12(gobj);
    }
}

pub fn ftPk_SpecialS1_IASA(_gobj: G) { melee_core::ct!("ftPk_SpecialS1_IASA");}

pub fn ftPk_SpecialAirS1_IASA(_gobj: G) { melee_core::ct!("ftPk_SpecialAirS1_IASA");}

pub fn ftPk_SpecialS1_Phys(_gobj: G) { melee_core::ct!("ftPk_SpecialS1_Phys");}

pub fn ftPk_SpecialS1_Coll(_gobj: G) { melee_core::ct!("ftPk_SpecialS1_Coll");}

pub fn ftPk_SpecialAirS1_Coll(gobj: G) {
    melee_core::ct!("ftPk_SpecialAirS1_Coll");
    unsafe {
        let fp = fpp(gobj);
        if st::ft_80081D0C(gobj) != 0 {
            st::ftCommon_8007D7FC(&mut *fp);
            ftPk_SpecialS_ChangeMotion_Unk11(gobj);
        }
        let env = (*fp).coll_data.env_flags;
        if env & Collide_LeftWallMask != 0 || env & Collide_RightWallMask != 0 {
            ftPk_SpecialS_ChangeMotion_Unk12(gobj);
        }
    }
}

pub fn ftPk_SpecialS_ChangeMotion_Unk10(gobj: G) {
    melee_core::ct!("ftPk_SpecialS_ChangeMotion_Unk10");
    unsafe {
        let fp = fpp(gobj);
        let sa = da(fp);
        (*fp).cmd_vars[0] = 0;
        (*fp).self_vel.x = fmadds((*sa).x40, (*fp).mv.pk.unk3.x0 as f32, (*sa).x3C);
        (*fp).self_vel.x *= (*fp).facing_dir;
        let temp = 0.5f32 * ((*fp).mv.pk.unk3.x0 as f32) / (*sa).x24;
        (*fp).self_vel.y = fmadds(0.5, (*sa).x44, (*sa).x44 * temp);
        st::Fighter_ChangeMotionState(gobj, 350, TRANSITION_FLAGS3, (*fp).cur_anim_frame, 1.0, 0.0, core::ptr::null_mut());
        (*fp).x21F8 = Some(cb_ftCommon_8007F76C);
        (*fp).deal_dmg_cb = Some(cb_ftPk_SpecialS_ZeroVelocity);
    }
}

pub fn ftPk_SpecialSEnd_IASA(_gobj: G) { melee_core::ct!("ftPk_SpecialSEnd_IASA");}

pub fn ftPk_SpecialAirSEnd_IASA(_gobj: G) { melee_core::ct!("ftPk_SpecialAirSEnd_IASA");}

pub fn ftPk_SpecialAirSEnd_Coll(gobj: G) {
    melee_core::ct!("ftPk_SpecialAirSEnd_Coll");
    unsafe {
        let fp = fpp(gobj);
        if st::ft_80081D0C(gobj) != 0 {
            st::ftCommon_8007D7FC(&mut *fp);
            ftPk_SpecialS_ChangeMotion_Unk11(gobj);
        }
    }
}

pub fn ftPk_SpecialS_ChangeMotion_Unk11(gobj: G) {
    melee_core::ct!("ftPk_SpecialS_ChangeMotion_Unk11");
    unsafe {
        let fp = fpp(gobj);
        let sa = da(fp);
        (*fp).cmd_vars[0] = 0;
        (*fp).gr_vel /= (*sa).x50;
        st::Fighter_ChangeMotionState(gobj, 346, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
    }
}

pub fn ftPk_SpecialS_ChangeMotion_Unk12(gobj: G) {
    melee_core::ct!("ftPk_SpecialS_ChangeMotion_Unk12");
    unsafe {
        let fp = fpp(gobj);
        let sa = da(fp);
        (*fp).cmd_vars[0] = 0;
        (*fp).self_vel.x /= (*sa).x50;
        st::Fighter_ChangeMotionState(gobj, 351, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
    }
}

// ---- folded from chars (consolidation step 20/21) ----
/// `ftPk_SpecialS_ChangeMotion_Unk00` (src/melee/ft/kinds/ftPikachu/ftpikachuspecials.c).
pub fn ftPk_SpecialS_ChangeMotion_Unk00(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftPk_SpecialS_ChangeMotion_Unk00");
    let fp = unsafe { &mut *((*gobj).user_data as *mut Fighter) };
    st::ftCommon_GroundToAirStateChange(gobj, fp, 348, 0xC4C5284u32 /* transition_flags0 */);
}

/// `ftPk_SpecialS_ChangeMotion_Unk01` (src/melee/ft/kinds/ftPikachu/ftpikachuspecials.c).
pub fn ftPk_SpecialS_ChangeMotion_Unk01(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftPk_SpecialS_ChangeMotion_Unk01");
    let fp = unsafe { &mut *((*gobj).user_data as *mut Fighter) };
    st::ftCommon_AirToGroundStateChange(gobj, fp, 343, 0xC4C5284u32 /* transition_flags0 */);
}

/// `ftPk_SpecialSHold_Phys` (src/melee/ft/kinds/ftPikachu/ftpikachuspecials.c).
pub fn ftPk_SpecialSHold_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftPk_SpecialSHold_Phys");
    st::ft_80084F3C(gobj);
}

/// `ftPk_SpecialAirSHold_Phys` (src/melee/ft/kinds/ftPikachu/ftpikachuspecials.c).
pub fn ftPk_SpecialAirSHold_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftPk_SpecialAirSHold_Phys");
    st::ft_80084EEC(gobj);
}

/// `ftPk_SpecialS_ChangeMotion_Unk02` (src/melee/ft/kinds/ftPikachu/ftpikachuspecials.c).
pub fn ftPk_SpecialS_ChangeMotion_Unk02(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftPk_SpecialS_ChangeMotion_Unk02");
    let fp = unsafe { &mut *((*gobj).user_data as *mut Fighter) };
    st::ftCommon_GroundToAirStateChange(gobj, fp, 349, 0xC4C5286u32 /* transition_flags1 */);
}

/// `ftPk_SpecialS_ChangeMotion_Unk03` (src/melee/ft/kinds/ftPikachu/ftpikachuspecials.c).
pub fn ftPk_SpecialS_ChangeMotion_Unk03(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftPk_SpecialS_ChangeMotion_Unk03");
    let fp = unsafe { &mut *((*gobj).user_data as *mut Fighter) };
    st::ftCommon_AirToGroundStateChange(gobj, fp, 344, 0xC4C5286u32 /* transition_flags1 */);
}

/// `ftPk_SpecialS0_Phys` (src/melee/ft/kinds/ftPikachu/ftpikachuspecials.c).
pub fn ftPk_SpecialS0_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftPk_SpecialS0_Phys");
    st::ft_80084F3C(gobj);
}

/// `ftPk_SpecialAirS0_Phys` (src/melee/ft/kinds/ftPikachu/ftpikachuspecials.c).
pub fn ftPk_SpecialAirS0_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftPk_SpecialAirS0_Phys");
    st::ft_80084EEC(gobj);
}

/// `ftPk_SpecialS_ChangeMotion_Unk06` (src/melee/ft/kinds/ftPikachu/ftpikachuspecials.c).
pub fn ftPk_SpecialS_ChangeMotion_Unk06(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftPk_SpecialS_ChangeMotion_Unk06");
    let fp = unsafe { &mut *((*gobj).user_data as *mut Fighter) };
    st::ftCommon_GroundToAirStateChange(gobj, fp, 352, 0xC4C5086u32 /* transition_flags2 */);
}

/// `ftPk_SpecialS_ChangeMotion_Unk07` (src/melee/ft/kinds/ftPikachu/ftpikachuspecials.c).
pub fn ftPk_SpecialS_ChangeMotion_Unk07(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftPk_SpecialS_ChangeMotion_Unk07");
    let fp = unsafe { &mut *((*gobj).user_data as *mut Fighter) };
    st::ftCommon_AirToGroundStateChange(gobj, fp, 347, 0xC4C5086u32 /* transition_flags2 */);
}

/// `ftPk_SpecialSEnd_Anim` (src/melee/ft/kinds/ftPikachu/ftpikachuspecials.c).
pub fn ftPk_SpecialSEnd_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftPk_SpecialSEnd_Anim");
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        st::ft_8008A2BC(gobj);
    }
}

/// `ftPk_SpecialAirSEnd_Anim` (src/melee/ft/kinds/ftPikachu/ftpikachuspecials.c).
pub fn ftPk_SpecialAirSEnd_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftPk_SpecialAirSEnd_Anim");
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        st::ftCo_Fall_Enter(gobj);
    }
}

/// `ftPk_SpecialSEnd_Coll` (src/melee/ft/kinds/ftPikachu/ftpikachuspecials.c).
pub fn ftPk_SpecialSEnd_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftPk_SpecialSEnd_Coll");
    if st::ft_80082708(gobj) == 0 {
        st::ftCo_Fall_Enter(gobj);
    }
}

/// `ftPk_SpecialSStart_Coll` (src/melee/ft/kinds/ftPikachu/ftpikachuspecials.c).
pub fn ftPk_SpecialSStart_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftPk_SpecialSStart_Coll");
    unsafe {
        if st::ft_80082708(gobj) == 0 {
            crate::chars::r2::ftPikachu::ftpikachuspecials::ftPk_SpecialS_ChangeMotion_Unk00(gobj);
        }
    }
}

/// `ftPk_SpecialAirSStart_Coll` (src/melee/ft/kinds/ftPikachu/ftpikachuspecials.c).
pub fn ftPk_SpecialAirSStart_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftPk_SpecialAirSStart_Coll");
    unsafe {
        if st::ft_80081D0C(gobj) != 0 {
            crate::chars::r2::ftPikachu::ftpikachuspecials::ftPk_SpecialS_ChangeMotion_Unk01(gobj);
        }
    }
}

/// `ftPk_SpecialSHold_Coll` (src/melee/ft/kinds/ftPikachu/ftpikachuspecials.c).
pub fn ftPk_SpecialSHold_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftPk_SpecialSHold_Coll");
    unsafe {
        if st::ft_80082708(gobj) == 0 {
            crate::chars::r2::ftPikachu::ftpikachuspecials::ftPk_SpecialS_ChangeMotion_Unk02(gobj);
        }
    }
}

/// `ftPk_SpecialAirSHold_Coll` (src/melee/ft/kinds/ftPikachu/ftpikachuspecials.c).
pub fn ftPk_SpecialAirSHold_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftPk_SpecialAirSHold_Coll");
    unsafe {
        if st::ft_80081D0C(gobj) != 0 {
            crate::chars::r2::ftPikachu::ftpikachuspecials::ftPk_SpecialS_ChangeMotion_Unk03(gobj);
        }
    }
}

/// `ftPk_SpecialS0_Coll` (src/melee/ft/kinds/ftPikachu/ftpikachuspecials.c).
pub fn ftPk_SpecialS0_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftPk_SpecialS0_Coll");
    unsafe {
        if st::ft_80082708(gobj) == 0 {
            crate::chars::r2::ftPikachu::ftpikachuspecials::ftPk_SpecialS_ChangeMotion_Unk06(gobj);
        }
    }
}

/// `ftPk_SpecialAirS0_Coll` (src/melee/ft/kinds/ftPikachu/ftpikachuspecials.c).
pub fn ftPk_SpecialAirS0_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftPk_SpecialAirS0_Coll");
    unsafe {
        if st::ft_80081D0C(gobj) != 0 {
            crate::chars::r2::ftPikachu::ftpikachuspecials::ftPk_SpecialS_ChangeMotion_Unk07(gobj);
        }
    }
}

// ---- folded from chars blocks (consolidation step 20) ----
pub fn ftPk_SpecialAirS1_Phys_fp(fp: &mut Fighter) {
    melee_core::ct!("ftPk_SpecialAirS1_Phys_fp");
    let a: &ftPikachuAttributes = da_g(fp);
    if fp.cmd_vars[0] != 0 {
        co::fall(fp, a.x58, a.x4C);
    } else {
        co::fall(fp, a.x48, a.x4C);
    }
    if fp.cmd_vars[0] != 0 {
        co::calc_self_accel_deaccel(fp, a.x54);
    }
}

pub fn ftPk_SpecialAirSEnd_Phys_fp(fp: &mut Fighter) {
    melee_core::ct!("ftPk_SpecialAirSEnd_Phys_fp");
    let sa: &ftPikachuAttributes = da_g(fp);
    co::fall(fp, sa.x58, fp.co_attrs.terminal_velocity);
    co::calc_self_accel_deaccel(fp, sa.x54);
}

pub fn ftPk_SpecialSEnd_Phys_fp(fp: &mut Fighter) {
    melee_core::ct!("ftPk_SpecialSEnd_Phys_fp");
    let sa: &ftPikachuAttributes = da_g(fp);
    co::calc_ground_accel_deaccel(fp, sa.x54);
    co::set_self_movement_from_grounded_movement(fp);
}
