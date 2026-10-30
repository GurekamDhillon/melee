#![allow(non_snake_case, non_upper_case_globals)]
//! Native port of src/melee/ft/kinds/ftCommon/ftCo_CaptureDamageKoopa.c (hit while held by
//! Bowser's side-B, ground and air). No fused sites. Smoke-tested by
//! tests/tu_ftCo_CaptureDamageKoopa.rs.

use crate::types::*;

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
        Fighter_ChangeMotionState(g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G) = |g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G| { unsafe { let _ = crate::tu::fighter::Fighter_ChangeMotionState(g as _, msid as _, flags as _, start as _, speed as _, blend as _, other as _); } };
        ftCommon_8007E2F4(f: &mut Fighter, v: i16) = crate::ftcommon2::ftcommon_8007e2f4;
        ftAnim_8006EBA4(g: G) = |g: G| { unsafe { let _ = crate::tu::ftanim::ftAnim_8006EBA4(g as _); } };
        ftAnim_IsFramesRemaining(g: G) -> bool = crate::hooks::ftanim_is_frames_remaining;
        ftCo_800BCC20(g: G) = |g: G| { unsafe { let _ = crate::tu::ftCo_CaptureWaitKoopa::ftCo_800BCC20(g as _); } };
        ftCo_800BCD00(g: G) = |g: G| { unsafe { let _ = crate::tu::ftCo_CaptureWaitKoopa::ftCo_800BCD00(g as _); } };
        ftCo_800BC458(g: G) = crate::tu::ftCo_CaptureKoopa::ftCo_800BC458;
        ftCo_800BC4A8(g: G) = crate::tu::ftCo_CaptureKoopa::ftCo_800BC4A8;
        ftCo_800DA698(g: G, b: bool) = |g: G, b: bool| { unsafe { let _ = crate::tu::ftCo_CatchCut::ftCo_800DA698(g as _, b as _); } };
        ftCo_CaptureCut_Enter(g: G) = crate::tu::ftCo_CaptureCut::ftCo_CaptureCut_Enter;
    }
}

fn enter(g: G, msid: i32) {
    let f = fp(g);
    let flags = if f.x2222_b6() != 0 { Ft_MF_FreezeState as u32 } else { Ft_MF_None as u32 };
    ext::Fighter_ChangeMotionState(g, msid, flags, 0.0, 1.0, 0.0, f.victim_gobj);
    let f = fp(g);
    f.accessory1_cb = Some(crate::tu::ftCo_CapturePulled::ftCo_800DB464_cb);
    ext::ftCommon_8007E2F4(f, 0x1FF);
    ext::ftAnim_8006EBA4(g);
}

/// `inlineA0`: the shared Anim.
fn anim(g: G, cb: fn(G)) {
    if !ext::ftAnim_IsFramesRemaining(g) {
        cb(g);
        return;
    }
    ext::ftCo_800BC458(g);
    let f = fp(g);
    if f.grab_timer <= 0.0 {
        ext::ftCo_800DA698(f.victim_gobj, false);
        let f = fp(g);
        f.facing_dir = -f.facing_dir;
        ext::ftCo_CaptureCut_Enter(g);
    } else {
        ext::ftCo_800BC4A8(g);
    }
}

/// `ftCo_800BC9C8`.
pub fn ftCo_800BC9C8(g: G) {
    melee_core::ct!("ftCo_800BC9C8");
    enter(g, ftCo_MS_CaptureDamageKoopa as i32);
}
/// `ftCo_CaptureDamageKoopa_Anim`.
pub fn ftCo_CaptureDamageKoopa_Anim(g: G) {
    melee_core::ct!("ftCo_CaptureDamageKoopa_Anim");
    anim(g, ext::ftCo_800BCC20);
}
/// `ftCo_CaptureDamageKoopa_IASA`.
pub fn ftCo_CaptureDamageKoopa_IASA(_g: G) { melee_core::ct!("ftCo_CaptureDamageKoopa_IASA");}
/// `ftCo_CaptureDamageKoopa_Phys`.
pub fn ftCo_CaptureDamageKoopa_Phys(_g: G) { melee_core::ct!("ftCo_CaptureDamageKoopa_Phys");}
/// `ftCo_CaptureDamageKoopa_Coll`.
pub fn ftCo_CaptureDamageKoopa_Coll(_g: G) { melee_core::ct!("ftCo_CaptureDamageKoopa_Coll");}

/// `ftCo_800BCAF4`.
pub fn ftCo_800BCAF4(g: G) {
    melee_core::ct!("ftCo_800BCAF4");
    enter(g, ftCo_MS_CaptureDamageKoopaAir as i32);
}
/// `ftCo_CaptureDamageKoopaAir_Anim`.
pub fn ftCo_CaptureDamageKoopaAir_Anim(g: G) {
    melee_core::ct!("ftCo_CaptureDamageKoopaAir_Anim");
    anim(g, ext::ftCo_800BCD00);
}
/// `ftCo_CaptureDamageKoopaAir_IASA`.
pub fn ftCo_CaptureDamageKoopaAir_IASA(_g: G) { melee_core::ct!("ftCo_CaptureDamageKoopaAir_IASA");}
/// `ftCo_CaptureDamageKoopaAir_Phys`.
pub fn ftCo_CaptureDamageKoopaAir_Phys(_g: G) { melee_core::ct!("ftCo_CaptureDamageKoopaAir_Phys");}
/// `ftCo_CaptureDamageKoopaAir_Coll`.
pub fn ftCo_CaptureDamageKoopaAir_Coll(_g: G) { melee_core::ct!("ftCo_CaptureDamageKoopaAir_Coll");}
