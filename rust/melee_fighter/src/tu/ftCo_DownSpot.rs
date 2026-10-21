#![allow(non_snake_case, unused_unsafe)]
//! Native port of src/melee/ft/kinds/ftCommon/ftCo_DownSpot.c. Every function of the TU; calls
//! out of it go through `ext`. Checked against the C by tests/tu_ftCo_DownSpot.rs.
use crate::types::*;

pub type G = *mut HSD_GObj;

#[inline(always)]
fn fp<'a>(g: G) -> &'a mut Fighter {
    unsafe { &mut *((*g).user_data as *mut Fighter) }
}
#[allow(dead_code)]
#[inline(always)]
fn cd() -> &'static ftCommonData {
    crate::hooks::common_data()
}
#[allow(dead_code)]
#[inline(always)]
fn abs_c(x: f32) -> f32 {
    if x < 0.0 { -x } else { x }
}
/// Calls out of the TU. `hook::<name>` overrides; unset = the port, or a TODO panic.
pub mod ext {
    #[allow(unused_imports)]
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
        ftData_80085FD4(fp: &mut Fighter, msid: i32) -> *mut ftData_80085FD4_ret = |_, _| todo!("TODO(port) ftData_80085FD4 (ftdata.c)");
        Fighter_ChangeMotionState(g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G) = |_, _, _, _, _, _, _| todo!("TODO(port) Fighter_ChangeMotionState (fighter.c)");
        ftAnim_8006EBA4(g: G) = |_| todo!("TODO(port) ftAnim_8006EBA4 (ftanim.c)");
        ftCommon_8007E2F4(fp: &mut Fighter, v: i16) = crate::ftcommon2::ftcommon_8007e2f4;
        ftAnim_IsFramesRemaining(g: G) -> bool = crate::hooks::ftanim_is_frames_remaining;
        ftCo_80097E8C(g: G) = |_| todo!("TODO(port) ftCo_80097E8C (ftCo_DownBound.c)");
        ft_80084F3C(g: G) = crate::ft084e::ft_80084f3c;
        ft_80083F88(g: G) = |_| todo!("TODO(port) ft_80083F88 (ft_081B.c)");
    }
}

pub fn ftCo_DownSpot_Enter(g: G) {
    let r = ext::ftData_80085FD4(fp(g), ftCo_MS_DownSpotU);
    let msid = if unsafe { (*r).x14 } != 0 { ftCo_MS_DownSpotU } else { ftCo_MS_DownSpotD };
    ext::Fighter_ChangeMotionState(g, msid, Ft_MF_None as u32, 0.0, 1.0, 0.0, core::ptr::null_mut());
    ext::ftAnim_8006EBA4(g);
    ext::ftCommon_8007E2F4(fp(g), 0x1FF);
    unsafe { fp(g).mv.co.downspot.x4 = 0 };
}

pub fn ftCo_DownSpot_Anim(g: G) {
    if !ext::ftAnim_IsFramesRemaining(g) {
        ext::ftCo_80097E8C(g);
    }
}

pub fn ftCo_DownSpot_IASA(_g: G) {}

pub fn ftCo_DownSpot_Phys(g: G) {
    ext::ft_80084F3C(g);
}

pub fn ftCo_DownSpot_Coll(g: G) {
    ext::ft_80083F88(g);
}
