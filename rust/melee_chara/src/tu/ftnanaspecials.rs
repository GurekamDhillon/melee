//! src/melee/ft/kinds/ftNana/ftnanaspecials.c: Nana's side-B (she copies Popo's Squall Hammer
//! motion) and the Popo-link helpers (`ftNn_Init_801238E4`, `ftNn_Init_80123954`).
//! Fused sites (tools/rust-port/fma_sites.py): `ftPp_SpecialS_0_Coll_inline4` 1 and
//! `ftPp_SpecialS_1_Coll_inline4` 1 (`-(1.5f * (x0 * facing) - popo.x)` = fmadds); the rest of
//! `ftNn_Init_80123954` is unfused (`dx*dx` and `dy*dy` are separate statements).
//! Smoke-tested through recording hooks by tests/tu_ftnanaspecials.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use crate::stubs as st;
use melee_core::fp::fmadds;
use melee_fighter::hooks as fh;
use melee_types::*;

/// Already ported (round 11): re-exported under this TU.
pub use crate::chars::r11::ftNana::ftnanaspecials::{
    ftNn_Init_80123B3C, ftNn_Init_80123B3C_inline, ftNn_Init_80123BF0,
};

/// Calls into TUs that are not ported yet. Each is a settable hook; unset it panics, except the
/// ones with a verified port in melee_fighter / melee_chara, which the unset hook calls. The other
/// Nana TUs reuse these.
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
    pub use crate::tu::ftmewtwospecialhi::ext::{lb_8000B1CC, Fighter_SetEffectHitlagCallbacks};

    fn player_get_entity_at_index_port(slot: i32, index: i32) -> *mut HSD_GObj {
        melee_fighter::player::get_entity_at_index(slot, index)
    }
    hook_or!(PLAYER_GETENTITYATINDEX, Player_GetEntityAtIndex, player_get_entity_at_index_port, (slot: i32, index: i32) -> *mut HSD_GObj);
    fn ft_800849ec_port(a: *mut Fighter, b: *mut Fighter) {
        melee_fighter::tu::ft_081b::ft_800849ec(unsafe { &mut *a }, unsafe { &mut *b })
    }
    hook_or!(FT_800849EC, ft_800849EC, ft_800849ec_port, (popo_fp: *mut Fighter, nana_fp: *mut Fighter));
    fn ft_800824a0_port(g: *mut HSD_GObj, ecb: *mut ftCollisionBox) -> bool {
        melee_fighter::tu::ft_081b::ft_800824a0(g, ecb)
    }
    hook_or!(FT_800824A0, ft_800824A0, ft_800824a0_port, (gobj: *mut HSD_GObj, ecb: *mut ftCollisionBox) -> bool);
    fn ft_80082888_port(g: *mut HSD_GObj, ecb: *mut ftCollisionBox) -> bool {
        melee_fighter::tu::ft_081b::ft_80082888(g, ecb)
    }
    hook_or!(FT_80082888, ft_80082888, ft_80082888_port, (gobj: *mut HSD_GObj, ecb: *mut ftCollisionBox) -> bool);
    fn ft_part_set_rotx_port(fp: *mut Fighter, part: i32, v: f32) {
        melee_fighter::tu::ftparts::ftPartSetRotX(unsafe { &*fp }, part, v)
    }
    hook_or!(FTPARTSETROTX, ftPartSetRotX, ft_part_set_rotx_port, (fp: *mut Fighter, part: i32, v: f32));
    fn ft_part_get_rotx_port(fp: *mut Fighter, part: i32) -> f32 {
        melee_fighter::tu::ftparts::ftPartGetRotX(unsafe { &*fp }, part)
    }
    hook_or!(FTPARTGETROTX, ftPartGetRotX, ft_part_get_rotx_port, (fp: *mut Fighter, part: i32) -> f32);
    fn ft_part_set_roty_port(fp: *mut Fighter, part: i32, v: f32) {
        melee_fighter::tu::ftparts::ftPartSetRotY(unsafe { &*fp }, part, v)
    }
    hook_or!(FTPARTSETROTY, ftPartSetRotY, ft_part_set_roty_port, (fp: *mut Fighter, part: i32, v: f32));
    // `ftPp_SpecialS_8011F964(popo_gobj)` (ftPopo/ftpopospecials.c): not in Squall Hammer end
    // motions; melee_chara chars::ftPopo port when the hook is unset.
    fn ftpp_specials_8011f964_port(g: *mut HSD_GObj) -> bool {
        crate::chars::ftPopo::ftpopospecials::ftPp_SpecialS_8011F964(unsafe {
            &*((*g).user_data as *const Fighter)
        })
    }
    hook_or!(FTPP_SPECIALS_8011F964, ftPp_SpecialS_8011F964, ftpp_specials_8011f964_port, (gobj: *mut HSD_GObj) -> bool);
    pub static mut FTPP_SPECIALS_8011F6FC: Option<fn(*mut HSD_GObj) -> bool> = None;
            #[allow(non_snake_case)]
            pub fn ftPp_SpecialS_8011F6FC(gobj: *mut HSD_GObj) -> bool {
                match unsafe { FTPP_SPECIALS_8011F6FC } {
                    Some(f) => f(gobj),
                    None => { return unsafe { crate::tu::ftpopospecials::ftPp_SpecialS_8011F6FC(gobj as _) != 0 }; },
                }
            }
    // TARGET_PC `Slippi_Version()` (pc/platform); `Slippi_Codes()` is `melee_fighter::hooks`.
    crate::port_hook!(SLIPPI_VERSION, Slippi_Version, melee_runtime::tu::mex_data::Slippi_Version, () -> i32);
}

#[inline(always)]
unsafe fn fpp(gobj: *mut HSD_GObj) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}

/// `M_PI_2 * facing_dir` (double math, narrowed at the f32 argument).
#[inline(always)]
fn pi_2_facing(facing: f32) -> f32 {
    (core::f64::consts::FRAC_PI_2 * facing as f64) as f32
}

/// `ftNn_Unk2_803CDD60`: the ECB box Nana's side-B tests against the stage.
pub static mut ftNn_Unk2_803CDD60: ftCollisionBox = ftCollisionBox {
    top: 12.0,
    bottom: 0.0,
    left: Vec2 { x: -6.0, y: 6.0 },
    right: Vec2 { x: 6.0, y: 6.0 },
};

/// `ftNn_Init_801238E4`: release the Popo link (also inlined into the Anim callbacks).
pub fn ftNn_Init_801238E4(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        ext::ftPartSetRotX(fp, 0, 0.0);
        st::Fighter_UnkSetFlag_8006CFBC(gobj);
        if !(*fp).x1A5C.is_null() {
            let fp2 = fpp((*fp).x1A5C);
            st::Fighter_UnkSetFlag_8006CFBC((*fp).x1A5C);
            (*fp2).x1A5C = core::ptr::null_mut();
        }
        (*fp).x1A5C = core::ptr::null_mut();
    }
}

/// `ftNn_Init_80123954`: Nana's attempt to rejoin Popo after a Popo-side state change. Returns
/// whether Nana is still free (false once she was snapped back to Popo).
pub fn ftNn_Init_80123954(nana_gobj: *mut HSD_GObj, pp_ga: GroundOrAir) -> bool {
    unsafe {
        if nana_gobj.is_null() {
            return true;
        }
        let nana_fp = fpp(nana_gobj);
        let popo_fp = fpp(ext::Player_GetEntityAtIndex((*nana_fp).player_id as i32, 0));
        let attrs = (*nana_fp).dat_attrs as *const ftIceClimberAttributes;
        match (*nana_fp).x2070.anon_0.x2071_b0_3() {
            1..=13 => {
                // TARGET_PC: Slippi's FreezeGlitchFix (@ 0x801239A8) removes this store in the
                // online codeset and in console codesets from 2020-07.
                let codes = fh::slippi_codes();
                if !(codes == 2 || (codes == 1 && ext::Slippi_Version() >= 20000)) {
                    (*nana_fp).x1A5C = core::ptr::null_mut();
                }
                true
            }
            _ => {
                let dd = (*attrs).xD0;
                let dx = (*nana_fp).cur_pos.x - (*popo_fp).cur_pos.x;
                let dy = (*nana_fp).cur_pos.y - (*popo_fp).cur_pos.y;
                let dxsq = dx * dx;
                let dysq = dy * dy;
                // `(s32)(scale * (dd * dd))`, compared as a float against `dxsq + dysq`.
                let lim = ((*popo_fp).x34_scale.y * (dd * dd)) as i32;
                if dxsq + dysq < lim as f32 {
                    if pp_ga as i32 == GA_Air as i32 {
                        if pp_ga as i32 != (*nana_fp).ground_or_air as i32 {
                            st::ftCommon_8007D5D4(&mut *nana_fp);
                        }
                        ftNn_Init_80123BF0(nana_gobj);
                    } else {
                        if pp_ga as i32 != (*nana_fp).ground_or_air as i32 {
                            ext::ft_800849EC(popo_fp, nana_fp);
                            st::ftCommon_8007D7FC(&mut *nana_fp);
                        }
                        ftNn_Init_80123B3C(nana_gobj);
                    }
                    (*nana_fp).mv.pp.unk_80123954.x0 = 0;
                    (*nana_fp).cur_pos = (*popo_fp).cur_pos;
                    (*nana_fp).self_vel = (*popo_fp).self_vel;
                    (*nana_fp).gr_vel = (*popo_fp).gr_vel;
                    (*nana_fp).facing_dir = (*popo_fp).facing_dir;
                    ext::ftPartSetRotY(nana_fp, 0, pi_2_facing((*nana_fp).facing_dir));
                    (*nana_fp).x1A5C = ext::Player_GetEntityAtIndex((*nana_fp).player_id as i32, 0);
                    false
                } else {
                    (*nana_fp).x1A5C = core::ptr::null_mut();
                    true
                }
            }
        }
    }
}

pub fn ftNn_Init_80123B10(gobj: *mut HSD_GObj) -> bool {
    unsafe {
        match (*fpp(gobj)).motion_id as i32 {
            359 | 360 => false,
            _ => true,
        }
    }
}

/// `ftPp_SpecialS_0_Anim_inline` (static inline).
fn ftPp_SpecialS_0_Anim_inline(nana_gobj: *mut HSD_GObj) {
    unsafe {
        let nana_fp = fpp(nana_gobj);
        (*nana_fp).take_dmg_cb = None;
        (*nana_fp).death2_cb = None;
        (*nana_fp).pre_hitlag_cb = None;
        (*nana_fp).post_hitlag_cb = None;
        (*nana_fp).x1A5C = core::ptr::null_mut();
    }
}

pub fn ftPp_SpecialS_0_Anim(nana_gobj: *mut HSD_GObj) {
    unsafe {
        let nana_fp = fpp(nana_gobj);
        let popo_gobj = ext::Player_GetEntityAtIndex((*nana_fp).player_id as i32, 0);
        let popo_fp = fpp(popo_gobj);
        if !ext::ftPp_SpecialS_8011F964(popo_gobj) {
            (*nana_fp).cur_anim_frame = (*popo_fp).cur_anim_frame;
            return;
        }
        ftNn_Init_801238E4(nana_gobj);
        ftPp_SpecialS_0_Anim_inline(nana_gobj);
        st::ft_8008A2BC(nana_gobj);
    }
}

pub fn ftPp_SpecialS_1_Anim(nana_gobj: *mut HSD_GObj) {
    unsafe {
        let nana_fp = fpp(nana_gobj);
        let attrs = (*nana_fp).dat_attrs as *const ftIceClimberAttributes;
        let popo_gobj = ext::Player_GetEntityAtIndex((*nana_fp).player_id as i32, 0);
        let popo_fp = fpp(popo_gobj);
        if !ext::ftPp_SpecialS_8011F964(popo_gobj) {
            (*nana_fp).cur_anim_frame = (*popo_fp).cur_anim_frame;
            return;
        }
        ftNn_Init_801238E4(nana_gobj);
        ftPp_SpecialS_0_Anim_inline(nana_gobj);
        if 0.0f64 == (*attrs).x12C as f64 {
            st::ftCo_Fall_Enter(nana_gobj);
        } else {
            st::ftCommon_8007D60C(&mut *nana_fp);
            st::ftCo_80096900(nana_gobj, 1, 0, 1, 1.0, (*attrs).x12C);
        }
    }
}

pub fn ftPp_SpecialS_0_IASA(_gobj: *mut HSD_GObj) {}
pub fn ftPp_SpecialS_1_IASA(_gobj: *mut HSD_GObj) {}

/// Shared body of `ftPp_SpecialS_0_Phys` / `ftPp_SpecialS_1_Phys`: mirror Popo's momentum.
unsafe fn copy_popo_phys(nana_gobj: *mut HSD_GObj) {
    let nana_fp = fpp(nana_gobj);
    let popo_fp = fpp(ext::Player_GetEntityAtIndex((*nana_fp).player_id as i32, 0));
    (*nana_fp).self_vel = (*popo_fp).self_vel;
    (*nana_fp).x74_self_accel = (*popo_fp).x74_self_accel;
    (*nana_fp).gr_vel = (*popo_fp).gr_vel;
    (*nana_fp).xE4_ground_accel_1 = (*popo_fp).xE4_ground_accel_1;
    (*nana_fp).facing_dir = (*popo_fp).facing_dir;
    ext::ftPartSetRotY(nana_fp, 0, pi_2_facing((*nana_fp).facing_dir));
}

pub fn ftPp_SpecialS_0_Phys(nana_gobj: *mut HSD_GObj) {
    unsafe { copy_popo_phys(nana_gobj) }
}

pub fn ftPp_SpecialS_1_Phys(nana_gobj: *mut HSD_GObj) {
    unsafe { copy_popo_phys(nana_gobj) }
}

/// `ftPp_SpecialS_0_Coll_inline` (static inline): copy Popo's X rotation.
fn ftPp_SpecialS_0_Coll_inline(nana_gobj: *mut HSD_GObj) {
    unsafe {
        let nana_fp = fpp(nana_gobj);
        let popo_gobj = ext::Player_GetEntityAtIndex((*nana_fp).player_id as i32, 0);
        let popo_fp = fpp(popo_gobj);
        let rot = ext::ftPartGetRotX(popo_fp, 0);
        ext::ftPartSetRotX(nana_fp, 0, rot);
    }
}

/// `ftPp_SpecialS_0_Coll_inline2` (static inline).
fn ftPp_SpecialS_0_Coll_inline2(nana_gobj: *mut HSD_GObj) {
    unsafe {
        let nana_fp = fpp(nana_gobj);
        (*nana_fp).take_dmg_cb = Some(crate::chars::cb::ftnana__ftNn_Init_80122FAC);
        (*nana_fp).death2_cb = Some(crate::chars::cb::ftnana__ftNn_Init_80122FAC);
        ext::Fighter_SetEffectHitlagCallbacks(nana_gobj);
    }
}

/// `ftPp_SpecialS_0_Coll_inline3` (static inline): ground -> air switch of Nana's copy motion.
fn ftPp_SpecialS_0_Coll_inline3(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        st::Fighter_ClearCmdVars(&mut *fp);
        ftNn_Init_80123B3C_inline(gobj);
        st::ftCommon_8007D5D4(&mut *fp);
        st::Fighter_ChangeMotionState(gobj, 360, 0x0C4C528A, (*fp).cur_anim_frame, 1.0, 0.0, core::ptr::null_mut());
        ftNn_Init_80123B3C_inline(gobj);
    }
}

/// `ftPp_SpecialS_1_Coll_inline3` (static inline): air -> ground switch.
fn ftPp_SpecialS_1_Coll_inline3(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        st::Fighter_ClearCmdVars(&mut *fp);
        ftNn_Init_80123B3C_inline(gobj);
        st::ftCommon_8007D7FC(&mut *fp);
        (*fp).self_vel.y = 0.0;
        st::Fighter_ChangeMotionState(gobj, 359, 0x0C4C528A, (*fp).cur_anim_frame, 1.0, 0.0, core::ptr::null_mut());
        ftNn_Init_80123B3C_inline(gobj);
    }
}

/// Shared head of `ftPp_SpecialS_{0,1}_Coll_inline4`: either snap to Popo or trail 1.5 units per
/// frame behind him (ramping up to 5 frames). Returns true when the ecb test asks for `inline3`.
unsafe fn coll_follow(
    nana_gobj: *mut HSD_GObj,
    popo_fp: *mut Fighter,
    nana_fp: *mut Fighter,
    popo_gobj: *mut HSD_GObj,
    motion: i32,
    ecb_wants_switch: fn(*mut HSD_GObj) -> bool,
) -> bool {
    if !ext::ftPp_SpecialS_8011F964(popo_gobj) && !ext::ftPp_SpecialS_8011F6FC(popo_gobj) {
        (*nana_fp).cur_pos = (*popo_fp).cur_pos;
        ext::ft_800849EC(popo_fp, nana_fp);
        if (*nana_fp).ground_or_air as i32 != (*popo_fp).ground_or_air as i32 {
            st::Fighter_ChangeMotionState(
                nana_gobj,
                motion,
                0x0C4C528A,
                (*nana_fp).cur_anim_frame,
                1.0,
                0.0,
                core::ptr::null_mut(),
            );
        }
        (*nana_fp).ground_or_air = (*popo_fp).ground_or_air;
        false
    } else {
        (*nana_fp).mv.pp.unk_80123954.x0 += 1;
        if (*nana_fp).mv.pp.unk_80123954.x0 > 5 {
            (*nana_fp).mv.pp.unk_80123954.x0 = 5;
        }
        // `-((1.5f * (x0 * facing)) - popo.x)`: the multiply-subtract contracts.
        (*nana_fp).cur_pos.x = -fmadds(
            1.5,
            (*nana_fp).mv.pp.unk_80123954.x0 as f32 * (*nana_fp).facing_dir,
            -(*popo_fp).cur_pos.x,
        );
        ecb_wants_switch(nana_gobj)
    }
}

pub fn ftPp_SpecialS_0_Coll(nana_gobj: *mut HSD_GObj) {
    unsafe {
        let nana_fp = fpp(nana_gobj);
        let popo_gobj = ext::Player_GetEntityAtIndex((*nana_fp).player_id as i32, 0);
        let popo_fp = fpp(popo_gobj);
        // inline4: switch when the stage test (`ft_80082888`) fails.
        let switch = coll_follow(nana_gobj, popo_fp, nana_fp, popo_gobj, 360, |g| unsafe {
            !ext::ft_80082888(g, &raw mut ftNn_Unk2_803CDD60)
        });
        if switch {
            ftPp_SpecialS_0_Coll_inline3(nana_gobj);
        }
        ftPp_SpecialS_0_Coll_inline(nana_gobj);
        ftPp_SpecialS_0_Coll_inline2(nana_gobj);
    }
}

pub fn ftPp_SpecialS_1_Coll(nana_gobj: *mut HSD_GObj) {
    unsafe {
        let nana_fp = fpp(nana_gobj);
        let popo_gobj = ext::Player_GetEntityAtIndex((*nana_fp).player_id as i32, 0);
        let popo_fp = fpp(popo_gobj);
        // inline4: switch when the stage test (`ft_800824A0`) passes.
        let switch = coll_follow(nana_gobj, popo_fp, nana_fp, popo_gobj, 359, |g| unsafe {
            ext::ft_800824A0(g, &raw mut ftNn_Unk2_803CDD60)
        });
        if switch {
            ftPp_SpecialS_1_Coll_inline3(nana_gobj);
        }
        ftPp_SpecialS_0_Coll_inline(nana_gobj);
        ftPp_SpecialS_0_Coll_inline2(nana_gobj);
    }
}
