#![allow(non_snake_case, non_upper_case_globals)]
//! Native port of src/melee/ft/kinds/ftCommon/ftCo_CaptureKoopa.c (held by Bowser's side-B:
//! the mash, the struggle wobble on YRotN and the two entries). No fused sites. Smoke-tested by
//! tests/tu_ftCo_CaptureKoopa.rs.

use crate::types::*;
use melee_types::inline as ji;

pub type G = *mut HSD_GObj;

#[inline(always)]
fn fp<'a>(g: G) -> &'a mut Fighter {
    unsafe { &mut *((*g).user_data as *mut Fighter) }
}

/// Calls out of the TU. `hook::<name>` overrides; unset = the port, or a TODO panic.
pub mod ext {
    use super::*;
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
    ext_fns! {
        Fighter_UpdateModelScale(g: G) = |g: G| { unsafe { crate::tu::fighter::Fighter_UpdateModelScale(g as _); } };
        ftKp_SpecialS_80132DC0(g: G) -> f32 = |_| todo!("ftKp_SpecialS_80132DC0: installed by melee_link");
        ftKp_SpecialS_80132DD0(g: G) -> f32 = |_| todo!("ftKp_SpecialS_80132DD0: installed by melee_link");
        ftKp_SpecialS_80132DE0(g: G) -> f32 = |_| todo!("ftKp_SpecialS_80132DE0: installed by melee_link");
        ftKp_SpecialS_80132DF0(g: G) -> f32 = |_| todo!("ftKp_SpecialS_80132DF0: installed by melee_link");
        ftKp_SpecialS_80132E00(g: G) -> f32 = |_| todo!("ftKp_SpecialS_80132E00: installed by melee_link");
        ftKp_SpecialS_80132E10(g: G) -> f32 = |_| todo!("ftKp_SpecialS_80132E10: installed by melee_link");
        ftKp_SpecialS_80132E20(g: G) -> f32 = |_| todo!("ftKp_SpecialS_80132E20: installed by melee_link");
        ftCommon_GrabMash(f: &mut Fighter, v: f32) -> bool = crate::ftcommon2::grab_mash;
        ftCommon_InitGrab(f: &mut Fighter, a: i32, timer: f32) = crate::ftcommon2::init_grab;
        ftParts_GetBoneIndex(f: &Fighter, part: i32) -> i32 = crate::ftdmgfall::ftparts_get_bone_index;
        ftAnim_SetAnimRate(g: G, rate: f32) = |g: G, rate: f32| { unsafe { crate::tu::ftanim::ftAnim_SetAnimRate(g as _, rate as _); } };
        HSD_JObjSetMtxDirtySub(j: *mut HSD_JObj) = |a0| unsafe { melee_runtime::tu::jobj_ptr::HSD_JObjSetMtxDirtySub(a0 as _) };
        ftCo_ReleaseItemAndVictim(g: G) = crate::tu::ftco_800c7590::ftco_release_item_and_victim;
        ftCo_800DB368(att: &mut Fighter, f: &mut Fighter) = crate::tu::ftCo_CapturePulled::ftCo_800DB368;
        ftCommon_8007D5D4(f: &mut Fighter) = crate::ftcommon2::ftcommon_8007d5d4;
        ftCommon_8007E2FC(g: G) = |a0| crate::tu::ftcommon::ftcommon_8007e2fc(unsafe { &mut *((*(a0 as *mut HSD_GObj)).user_data as *mut Fighter) });
        ftCo_800BC9C8(g: G) = crate::tu::ftCo_CaptureDamageKoopa::ftCo_800BC9C8;
        ftCo_800BCAF4(g: G) = crate::tu::ftCo_CaptureDamageKoopa::ftCo_800BCAF4;
    }
}

struct Dirty;
impl ji::JObjDirty for Dirty {
    fn set_mtx_dirty(&mut self, j: &mut HSD_JObj) {
        if !ji::HSD_JObjMtxIsDirty(j) {
            ext::HSD_JObjSetMtxDirtySub(j);
        }
    }
    fn set_mtx_dirty_sub(&mut self, j: &mut HSD_JObj) {
        ext::HSD_JObjSetMtxDirtySub(j);
    }
}

#[inline(always)]
fn abs_c(x: f32) -> f32 {
    if x < 0.0 { -x } else { x }
}

/// `ftCo_800BC438`.
pub fn ftCo_800BC438(g: G) {
    melee_core::ct!("ftCo_800BC438");
    ext::Fighter_UpdateModelScale(g);
}

/// `ftCo_800BC458`: the grab timer drains by Bowser's rate; mashing.
pub fn ftCo_800BC458(g: G) {
    melee_core::ct!("ftCo_800BC458");
    let f = fp(g);
    f.grab_timer -= ext::ftKp_SpecialS_80132DD0(f.victim_gobj);
    let v = ext::ftKp_SpecialS_80132DE0(f.victim_gobj);
    let m = ext::ftCommon_GrabMash(f, v);
    f.mv.co.capturekoopa.x0 = m as i32;
}

/// `ftCo_800BC4A8`: the struggle wobble and the anim-rate reset.
pub fn ftCo_800BC4A8(g: G) {
    melee_core::ct!("ftCo_800BC4A8");
    let f = fp(g);
    unsafe {
        if f.mv.ca.specialhi.vel.y != 0.0 {
            let i = ext::ftParts_GetBoneIndex(f, FtPart_YRotN as i32);
            let j = &mut *(*f.parts.offset(i as isize)).joint;
            let xc = f.mv.co.capturekoopa.xC;
            let a = f.x1A50 as f32 * xc;
            let b = f.x1A51 as f32 * xc;
            if abs_c(a + ji::HSD_JObjGetTranslationZ(j)) <= f.mv.co.capturekoopa.x10 {
                ji::HSD_JObjAddTranslationZ(j, a, &mut Dirty);
            }
            if abs_c(b + ji::HSD_JObjGetTranslationY(j)) <= f.mv.co.capturekoopa.x10 { // C reads walk.fast_anim_frame (same GC offset 0x2350)
                ji::HSD_JObjAddTranslationY(j, b, &mut Dirty);
            }
            f.mv.ca.specialhi.vel.y -= 1.0; // capturekoopa.x8 shares the GC offset (0x2348) with specialhi.vel.y; the native layouts do not
            if f.mv.ca.specialhi.vel.y <= 0.0 && f.mv.co.capturekoopa.x0 == 0 {
                ext::ftAnim_SetAnimRate(g, 1.0);
                fp(g).mv.ca.specialhi.vel.y = 0.0;
            }
        }
        let f = fp(g);
        if f.mv.ca.specialhi.vel.y <= 0.0 && f.mv.co.capturekoopa.x0 != 0 {
            f.mv.ca.specialhi.vel.y = ext::ftKp_SpecialS_80132DF0(f.victim_gobj);
            let r = ext::ftKp_SpecialS_80132E00(fp(g).victim_gobj);
            ext::ftAnim_SetAnimRate(g, r);
        }
    }
}

/// `inlineA0`: the shared entry.
fn enter(g: G, vg: G, cb: fn(G)) {
    let vf = fp(vg);
    let f = fp(g);
    ext::ftCo_ReleaseItemAndVictim(g);
    f.x1A5C = vg;
    f.victim_gobj = vg;
    f.anon_221b.set_x221B_b5(0);
    f.anon_221b.set_x221B_b7(0);
    f.facing_dir = vf.facing_dir;
    let t = ext::ftKp_SpecialS_80132DC0(f.victim_gobj);
    ext::ftCommon_InitGrab(f, 0, t);
    f.mv.ca.specialhi.vel.y = 0.0;
    ext::ftCo_800DB368(vf, f);
    cb(g);
    let f = fp(g);
    ext::ftCommon_8007D5D4(f);
    {
        f.mv.co.capturekoopa.xC = ext::ftKp_SpecialS_80132E10(f.victim_gobj);
        f.mv.co.capturekoopa.x10 = ext::ftKp_SpecialS_80132E20(f.victim_gobj);
    }
    ext::ftCommon_8007E2FC(g);
}

/// `ftCo_800BC7E0`.
pub fn ftCo_800BC7E0(g: G, vg: G) {
    melee_core::ct!("ftCo_800BC7E0");
    enter(g, vg, ext::ftCo_800BC9C8);
}
/// `ftCo_CaptureKoopa_Anim`.
pub fn ftCo_CaptureKoopa_Anim(_g: G) { melee_core::ct!("ftCo_CaptureKoopa_Anim");}
/// `ftCo_CaptureKoopa_IASA`.
pub fn ftCo_CaptureKoopa_IASA(_g: G) { melee_core::ct!("ftCo_CaptureKoopa_IASA");}
/// `ftCo_CaptureKoopa_Phys`.
pub fn ftCo_CaptureKoopa_Phys(_g: G) { melee_core::ct!("ftCo_CaptureKoopa_Phys");}
/// `ftCo_CaptureKoopa_Coll`.
pub fn ftCo_CaptureKoopa_Coll(_g: G) { melee_core::ct!("ftCo_CaptureKoopa_Coll");}

/// `ftCo_800BC8D4`.
pub fn ftCo_800BC8D4(g: G, vg: G) {
    melee_core::ct!("ftCo_800BC8D4");
    enter(g, vg, ext::ftCo_800BCAF4);
}
/// `ftCo_CaptureKoopaAir_Anim`.
pub fn ftCo_CaptureKoopaAir_Anim(_g: G) { melee_core::ct!("ftCo_CaptureKoopaAir_Anim");}
/// `ftCo_CaptureKoopaAir_IASA`.
pub fn ftCo_CaptureKoopaAir_IASA(_g: G) { melee_core::ct!("ftCo_CaptureKoopaAir_IASA");}
/// `ftCo_CaptureKoopaAir_Phys`.
pub fn ftCo_CaptureKoopaAir_Phys(_g: G) { melee_core::ct!("ftCo_CaptureKoopaAir_Phys");}
/// `ftCo_CaptureKoopaAir_Coll`.
pub fn ftCo_CaptureKoopaAir_Coll(_g: G) { melee_core::ct!("ftCo_CaptureKoopaAir_Coll");}
