#![allow(non_snake_case, non_upper_case_globals)]
//! Native port of src/melee/ft/kinds/ftCommon/ftCo_CaptureWaitKoopa.c. Every function of the TU; calls out of it go
//! through `ext` (unported ones panic until rewired). Checked against the C (whole TU, shipping
//! contraction) by tests/tu_ftCo_CaptureWaitKoopa.rs.
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
        Fighter_ChangeMotionState(g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G) = |_, _, _, _, _, _, _| todo!("TODO(port) Fighter_ChangeMotionState (fighter.c)");
        ftCo_800DB464(g: G) = |_| todo!("TODO(port) ftCo_800DB464 (ftCo_CatchCut.c)");
        ftCommon_8007E2F4(f: &mut Fighter, v: i16) = crate::ftcommon2::ftcommon_8007e2f4;
        ftCo_800BC458(g: G) = |_| todo!("TODO(port) ftCo_800BC458 (ftCo_CaptureKoopa.c)");
        ftCo_800BC4A8(g: G) = |_| todo!("TODO(port) ftCo_800BC4A8 (ftCo_CaptureKoopa.c)");
        ftCo_800DA698(g: G, b: bool) = |_, _| todo!("TODO(port) ftCo_800DA698 (ftCo_CatchCut.c)");
        ftCo_CaptureCut_Enter(g: G) = crate::tu::ftCo_CaptureCut::ftCo_CaptureCut_Enter;
    }
}

/// `ftCo_800DB464` as the `accessory1_cb` value.
pub unsafe extern "C" fn ftCo_800DB464_cb(g: G) {
    ext::ftCo_800DB464(g)
}

pub fn ftCo_800BCC20(g: G) {
    ext::Fighter_ChangeMotionState(g, ftCo_MS_CaptureWaitKoopa as i32, Ft_MF_None, 0.0, 1.0, 0.0, core::ptr::null_mut());
    let f = fp(g);
    f.accessory1_cb = Some(ftCo_800DB464_cb);
    ext::ftCommon_8007E2F4(f, 0x1FF);
}

pub fn ftCo_CaptureWaitKoopa_Anim(g: G) {
    ext::ftCo_800BC458(g);
    if fp(g).grab_timer <= 0.0 {
        ext::ftCo_800DA698(fp(g).victim_gobj, false);
        let f = fp(g);
        f.facing_dir = -f.facing_dir;
        ext::ftCo_CaptureCut_Enter(g);
    } else {
        ext::ftCo_800BC4A8(g);
    }
}

pub fn ftCo_CaptureWaitKoopa_IASA(_g: G) {}

pub fn ftCo_CaptureWaitKoopa_Phys(_g: G) {}

pub fn ftCo_CaptureWaitKoopa_Coll(_g: G) {}

pub fn ftCo_800BCD00(g: G) {
    ext::Fighter_ChangeMotionState(g, ftCo_MS_CaptureWaitKoopaAir as i32, Ft_MF_None, 0.0, 1.0, 0.0, core::ptr::null_mut());
    let f = fp(g);
    f.accessory1_cb = Some(ftCo_800DB464_cb);
    ext::ftCommon_8007E2F4(f, 0x1FF);
}

pub fn ftCo_CaptureWaitKoopaAir_Anim(g: G) {
    ftCo_CaptureWaitKoopa_Anim(g);
}

pub fn ftCo_CaptureWaitKoopaAir_IASA(_g: G) {}

pub fn ftCo_CaptureWaitKoopaAir_Phys(_g: G) {}

pub fn ftCo_CaptureWaitKoopaAir_Coll(_g: G) {}
