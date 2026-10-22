//! src/melee/ft/kinds/ftMario/ftmariospecials.c: Cape (side B). Fused sites (fma_sites.py):
//! `coords.x += 3 * facing_dir` in `ftMr_SpecialS_Phys` and `ftMr_SpecialAirS_Phys` (fmadds).
//! The wind-puff position passed to lb_800119DC is checked against the C by
//! tests/tu_ftmariospecials.rs; the rest is smoke-tested through recording hooks.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use crate::stubs as st;
use melee_core::fp::fmadds;
use melee_fighter::ftcommon as fc;
use melee_fighter::hooks as fh;
use melee_types::*;

/// Already ported (round 2 and the pure helper): re-exported under this TU.
pub use crate::chars::ftMario::ftmariospecials::ftMr_SpecialS_CheckItemCapeRemove;
pub use crate::chars::r2::ftMario::ftmariospecials::{ftMr_SpecialAirS_Anim, ftMr_SpecialS_Anim};

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
    // `lb_8000B1CC(joint, offset, out)` (lb/lb_00B0.c); the C passes a NULL offset here.
    hook!(LB_8000B1CC, lb_8000B1CC, (joint: *mut HSD_JObj, off: *const Vec3, out: *mut Vec3));
    hook!(LB_800119DC, lb_800119DC, (pos: *mut Vec3, a: i32, b: f32, c: f32, d: f32));
    hook!(IT_802B2560, it_802B2560, (gobj: *mut HSD_GObj, facing: f32, pos: *mut Vec3, bone: i32, kind: i32) -> *mut HSD_GObj);
    hook!(IT_802B2674, it_802B2674, (cape: *mut HSD_GObj));
    hook!(IT_802B26C0, it_802B26C0, (cape: *mut HSD_GObj));
    hook!(IT_802B26E0, it_802B26E0, (cape: *mut HSD_GObj));
    hook!(FTCOLL_CREATEREFLECTHIT, ftColl_CreateReflectHit, (gobj: *mut HSD_GObj, desc: *mut ReflectDesc, on_reflect: HSD_GObjEvent));
}

#[inline(always)]
unsafe fn fpp(gobj: *mut HSD_GObj) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}

/// `ftMr_Init_OnTakeDamage` as a Fighter callback entry (`take_dmg_cb` / `death2_cb`).
pub unsafe extern "C" fn cb_ftMr_Init_OnTakeDamage(g: *mut HSD_GObj) {
    crate::tu::ftmario::ftMr_Init_OnTakeDamage(g);
}
/// `ftMr_SpecialS_EnterHitlag` as a Fighter callback entry (`pre_hitlag_cb`).
pub unsafe extern "C" fn cb_ftMr_SpecialS_EnterHitlag(g: *mut HSD_GObj) {
    ftMr_SpecialS_EnterHitlag(g);
}
/// `ftMr_SpecialS_ExitHitlag` as a Fighter callback entry (`post_hitlag_cb`).
pub unsafe extern "C" fn cb_ftMr_SpecialS_ExitHitlag(g: *mut HSD_GObj) {
    ftMr_SpecialS_ExitHitlag(g);
}
/// `ftMr_SpecialS_CreateCape` as a Fighter callback entry (`accessory4_cb`).
pub unsafe extern "C" fn cb_ftMr_SpecialS_CreateCape(g: *mut HSD_GObj) {
    ftMr_SpecialS_CreateCape(g);
}

/// `setCallbacks` (static).
pub fn setCallbacks(fp: *mut Fighter) {
    unsafe {
        if !(*fp).u.mr.x223C_capeGObj.is_null() {
            (*fp).death2_cb = Some(cb_ftMr_Init_OnTakeDamage);
            (*fp).take_dmg_cb = Some(cb_ftMr_Init_OnTakeDamage);
        }
        (*fp).pre_hitlag_cb = Some(cb_ftMr_SpecialS_EnterHitlag);
        (*fp).post_hitlag_cb = Some(cb_ftMr_SpecialS_ExitHitlag);
    }
}

pub fn ftMr_SpecialS_CreateCape(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let sa = (*fp).dat_attrs as *const ftMario_DatAttrs;
        if (*fp).cmd_vars[2] == 0 {
            let mut coords = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
            (*fp).cmd_vars[2] = 1;
            let bone = fh::ftparts_get_bone_index(&*fp, FtPart_RThumbNb as i32);
            ext::lb_8000B1CC((*(*fp).parts.offset(bone as isize)).joint, core::ptr::null(), &mut coords);
            let bone = fh::ftparts_get_bone_index(&*fp, FtPart_RThumbNb as i32);
            let cape = ext::it_802B2560(gobj, (*fp).facing_dir, &mut coords, bone, (*sa).specials.cape_kind);
            (*fp).u.mr.x223C_capeGObj = cape;
            (*fp).x1984_heldItemSpec = (*fp).u.mr.x223C_capeGObj;
            setCallbacks(fp);
            (*fp).accessory4_cb = None;
        }
    }
}

pub fn ftMr_SpecialS_Reset(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        ftMr_SpecialS_ExitHitlag(gobj);
        (*fp).u.mr.x223C_capeGObj = core::ptr::null_mut();
        (*fp).death2_cb = None;
        (*fp).take_dmg_cb = None;
    }
}

pub fn ftMr_SpecialS_RemoveCape(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        if !(*fp).u.mr.x223C_capeGObj.is_null() {
            ext::it_802B2674((*fp).u.mr.x223C_capeGObj);
            ftMr_SpecialS_Reset(gobj);
        }
    }
}

pub fn ftMr_SpecialS_EnterHitlag(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        if !(*fp).u.mr.x223C_capeGObj.is_null() {
            ext::it_802B26C0((*fp).u.mr.x223C_capeGObj);
        }
    }
}

pub fn ftMr_SpecialS_ExitHitlag(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        if !(*fp).u.mr.x223C_capeGObj.is_null() {
            ext::it_802B26E0((*fp).u.mr.x223C_capeGObj);
        }
    }
}

/// `changeAction` (static).
pub fn changeAction(gobj: *mut HSD_GObj, msid: i32) {
    unsafe {
        st::Fighter_ChangeMotionState(gobj, msid, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
        let fp = fpp(gobj);
        (*fp).cmd_vars[2] = 0;
        (*fp).cmd_vars[1] = 0;
        (*fp).cmd_vars[0] = 0;
        (*fp).mv.mr.SpecialS.reflecting = 0;
        (*fp).accessory4_cb = Some(cb_ftMr_SpecialS_CreateCape);
    }
}

pub fn ftMr_SpecialS_Enter(gobj: *mut HSD_GObj) {
    unsafe {
        (*fpp(gobj)).self_vel.y = 0.0;
        changeAction(gobj, ftMr_MS_SpecialS as i32);
    }
}

pub fn ftMr_SpecialAirS_Enter(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let sa = (*fp).dat_attrs as *const ftMario_DatAttrs;
        (*fp).self_vel.x /= (*sa).specials.vel_x_decay;
        changeAction(gobj, ftMr_MS_SpecialAirS as i32);
    }
}

pub fn ftMr_SpecialS_IASA(_gobj: *mut HSD_GObj) {}

pub fn ftMr_SpecialAirS_IASA(_gobj: *mut HSD_GObj) {}

/// `reflect` (static): open / close the cape's reflect bubble from cmd_vars[1].
pub fn reflect(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let sa = (*fp).dat_attrs as *mut ftMario_DatAttrs;
        if (*fp).cmd_vars[1] == 1 && (*fp).mv.mr.SpecialS.reflecting == 0 {
            (*fp).mv.mr.SpecialS.reflecting = 1;
            ext::ftColl_CreateReflectHit(gobj, &mut (*sa).cape_reflection, None);
        } else if (*fp).cmd_vars[1] == 0 && (*fp).mv.mr.SpecialS.reflecting == 1 {
            (*fp).mv.mr.SpecialS.reflecting = 0;
            (*fp).set_reflecting(0);
        }
        st::ftColl_8007AEF8(gobj);
    }
}

/// `M_PI_3` as the `float` argument of lb_800119DC.
#[inline(always)]
fn pi_3() -> f32 {
    (core::f64::consts::PI / 3.0) as f32
}

/// The cape's wind puff: `coords` = HipN + (3 * facing_dir, 0, 0), a fused multiply-add.
unsafe fn wind_puff(fp: *mut Fighter, coords: &mut Vec3) {
    let bone = fh::ftparts_get_bone_index(&*fp, FtPart_HipN as i32);
    ext::lb_8000B1CC((*(*fp).parts.offset(bone as isize)).joint, core::ptr::null(), coords);
    coords.x = fmadds(3.0, (*fp).facing_dir, coords.x);
}

pub fn ftMr_SpecialS_Phys(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        if (*fp).cmd_vars[0] == 1 {
            let mut coords = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
            (*fp).cmd_vars[0] = 2;
            wind_puff(fp, &mut coords);
            ext::lb_800119DC(&mut coords, 120, 0.9, 0.02, pi_3());
        }
        st::ft_80084F3C(gobj);
        reflect(gobj);
    }
}

pub fn ftMr_SpecialAirS_Phys(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let sa = (*fp).dat_attrs as *const ftMario_DatAttrs;
        let ftcmd_var0_tmp = (*fp).cmd_vars[0];
        if ftcmd_var0_tmp >= 1 {
            if ftcmd_var0_tmp == 1 {
                let mut coords = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
                (*fp).cmd_vars[0] = 2;
                if (*fp).u.mr.x2238_isCapeBoost == 0 {
                    (*fp).u.mr.x2238_isCapeBoost = 1;
                    (*fp).self_vel.y = (*sa).specials.vel.y;
                } else {
                    (*fp).self_vel.y = 0.0;
                }
                wind_puff(fp, &mut coords);
                ext::lb_800119DC(&mut coords, 120, 3.0, 0.1, pi_3());
            }
            fc::fall(&mut *fp, (*sa).specials.grav, (*sa).specials.terminal_vel);
        } else {
            fc::fall_basic(&mut *fp);
        }
        fc::calc_self_accel_deaccel(&mut *fp, (*sa).specials.vel.x);
        reflect(gobj);
    }
}

pub fn ftMr_SpecialS_Coll(gobj: *mut HSD_GObj) {
    if st::ft_800827A0(gobj) == 0 {
        ftMr_SpecialS_GroundToAir(gobj);
    }
}

pub fn ftMr_SpecialAirS_Coll(gobj: *mut HSD_GObj) {
    if st::ft_80081D0C(gobj) != 0 {
        ftMr_SpecialAirS_AirToGround(gobj);
    }
}

/// `collUpdateVars` (static).
pub fn collUpdateVars(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        if (*fp).mv.mr.SpecialS.reflecting != 0 {
            (*fp).set_reflecting(1);
        }
        setCallbacks(fp);
        (*fp).accessory4_cb = Some(cb_ftMr_SpecialS_CreateCape);
    }
}

/// `transition_flags` (static const): the ground / air hand-over flags of the cape.
const transition_flags: u32 = (Ft_MF_KeepColAnimHitStatus
    | Ft_MF_SkipHit
    | Ft_MF_SkipMatAnim
    | Ft_MF_UpdateCmd
    | Ft_MF_SkipColAnim
    | Ft_MF_SkipItemVis
    | Ft_MF_Unk19
    | Ft_MF_SkipModelPartVis
    | Ft_MF_SkipModelFlags
    | Ft_MF_Unk27) as u32;

pub fn ftMr_SpecialS_GroundToAir(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        st::ftCommon_GroundToAirStateChange(gobj, &mut *fp, ftMr_MS_SpecialAirS as i32, transition_flags);
        if (*fp).cmd_vars[0] == 1 {
            (*fp).cmd_vars[0] = 2;
        }
        collUpdateVars(gobj);
    }
}

pub fn ftMr_SpecialAirS_AirToGround(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        (*fp).u.mr.x2238_isCapeBoost = 0;
        st::ftCommon_AirToGroundStateChange(gobj, &mut *fp, ftMr_MS_SpecialS as i32, transition_flags);
        collUpdateVars(gobj);
    }
}
