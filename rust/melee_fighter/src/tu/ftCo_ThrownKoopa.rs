#![allow(non_snake_case, non_upper_case_globals)]
//! Native port of src/melee/ft/kinds/ftCommon/ftCo_ThrownKoopa.c (every function). Calls out of
//! the TU go through `ext` (unported ones panic until rewired). Checked against the C (whole TU,
//! shipping contraction) by tests/tu_ftCo_ThrownKoopa.rs.
use crate::types::*;

pub type G = *mut HSD_GObj;

#[inline(always)]
fn fp<'a>(g: G) -> &'a mut Fighter {
    unsafe { &mut *((*g).user_data as *mut Fighter) }
}

macro_rules! ext_fns {
    ($($name:ident($($a:ident: $t:ty),*) $(-> $r:ty)? = $def:expr;)*) => {
        pub mod hook {
            #[allow(unused_imports)] use super::*;
            $( #[allow(non_upper_case_globals)] pub static mut $name: Option<fn($($t),*) $(-> $r)?> = None; )*
        }
        $( #[allow(non_snake_case)] pub fn $name($($a: $t),*) $(-> $r)? {
            match unsafe { hook::$name } { Some(f) => f($($a),*), None => ($def)($($a),*) }
        } )*
    };
}

/// Calls out of the TU. `hook::<name>` overrides; unset = the port, or a TODO panic.
pub mod ext {
    use super::*;
    ext_fns! {
        Fighter_ChangeMotionState(g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G) = |g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G| { unsafe { crate::tu::fighter::Fighter_ChangeMotionState(g as _, msid as _, flags as _, start as _, speed as _, blend as _, other as _); } };
        ftCo_800DE508(g: G) = |g: G| { unsafe { crate::tu::ftCo_Thrown::ftCo_800DE508(g as _); } };
        ftCommon_8007E2F4(f: &mut Fighter, v: i16) = crate::ftcommon2::ftcommon_8007e2f4;
        ftAnim_8006EBA4(g: G) = |g| unsafe { crate::tu::ftanim::ftAnim_8006EBA4(g) };
    }
}

/// `ftCo_800DE508` as the `accessory1_cb` value.
pub unsafe extern "C" fn ftCo_800DE508_cb(g: G) {
    melee_core::ct!("ftCo_800DE508_cb");
    ext::ftCo_800DE508(g)
}

/// static inline `ftCo_Thrown_Enter` (ftCommon/inlines.h).
fn thrown_enter(g: G, msid: i32, mf: u32, anim_start: f32) {
    let f = fp(g);
    f.facing_dir = fp(f.victim_gobj).facing_dir;
    unsafe { f.mv.co.capturekoopa.x0 = 0 };
    let victim = f.victim_gobj;
    ext::Fighter_ChangeMotionState(g, msid, mf, anim_start, 1.0, 0.0, victim);
    fp(g).accessory1_cb = Some(ftCo_800DE508_cb);
    ext::ftCommon_8007E2F4(fp(g), 0x1FF);
}

pub fn ftCo_800BCDE0(g: G, msid: i32) {
    melee_core::ct!("ftCo_800BCDE0");
    thrown_enter(g, msid, Ft_MF_None, 0.0);
    ext::ftAnim_8006EBA4(g);
}

pub fn ftCo_800BCE64(g: G, msid: i32) {
    melee_core::ct!("ftCo_800BCE64");
    let mf = Ft_MF_SkipMatAnim | Ft_MF_SkipColAnim | Ft_MF_UpdateCmd | Ft_MF_SkipItemVis | Ft_MF_Unk19
        | Ft_MF_SkipModelPartVis | Ft_MF_SkipModelFlags | Ft_MF_Unk27;
    let start = fp(g).cur_anim_frame;
    thrown_enter(g, msid, mf, start);
}

pub fn ftCo_ThrownKoopaF_Anim(_g: G) { melee_core::ct!("ftCo_ThrownKoopaF_Anim");}
pub fn ftCo_ThrownKoopaF_IASA(_g: G) { melee_core::ct!("ftCo_ThrownKoopaF_IASA");}
pub fn ftCo_ThrownKoopaF_Phys(_g: G) { melee_core::ct!("ftCo_ThrownKoopaF_Phys");}
pub fn ftCo_ThrownKoopaF_Coll(_g: G) { melee_core::ct!("ftCo_ThrownKoopaF_Coll");}
pub fn ftCo_ThrownKoopaB_Anim(_g: G) { melee_core::ct!("ftCo_ThrownKoopaB_Anim");}
pub fn ftCo_ThrownKoopaB_IASA(_g: G) { melee_core::ct!("ftCo_ThrownKoopaB_IASA");}
pub fn ftCo_ThrownKoopaB_Phys(_g: G) { melee_core::ct!("ftCo_ThrownKoopaB_Phys");}
pub fn ftCo_ThrownKoopaB_Coll(_g: G) { melee_core::ct!("ftCo_ThrownKoopaB_Coll");}
pub fn ftCo_ThrownKoopaAirF_Anim(_g: G) { melee_core::ct!("ftCo_ThrownKoopaAirF_Anim");}
pub fn ftCo_ThrownKoopaAirF_IASA(_g: G) { melee_core::ct!("ftCo_ThrownKoopaAirF_IASA");}
pub fn ftCo_ThrownKoopaAirF_Phys(_g: G) { melee_core::ct!("ftCo_ThrownKoopaAirF_Phys");}
pub fn ftCo_ThrownKoopaAirF_Coll(_g: G) { melee_core::ct!("ftCo_ThrownKoopaAirF_Coll");}
pub fn ftCo_ThrownKoopaAirB_Anim(_g: G) { melee_core::ct!("ftCo_ThrownKoopaAirB_Anim");}
pub fn ftCo_ThrownKoopaAirB_IASA(_g: G) { melee_core::ct!("ftCo_ThrownKoopaAirB_IASA");}
pub fn ftCo_ThrownKoopaAirB_Phys(_g: G) { melee_core::ct!("ftCo_ThrownKoopaAirB_Phys");}
pub fn ftCo_ThrownKoopaAirB_Coll(_g: G) { melee_core::ct!("ftCo_ThrownKoopaAirB_Coll");}
