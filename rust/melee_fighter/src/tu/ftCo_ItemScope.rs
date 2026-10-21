#![allow(non_snake_case)]
//! Native port of src/melee/ft/kinds/ftCommon/ftCo_ItemScope.c (Super Scope end lag motions).
//! Every function of the TU; calls out go through `ext`. Checked against the C by
//! tests/tu_ftCo_ItemScope.rs. No float arithmetic (values are only passed through).

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
        fn_800D769C(fp: &Fighter, msid: i32) -> i32 = crate::tu::ftCo_ItemScopeStart::fn_800D769C;
        Fighter_ChangeMotionState(g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G) = |_, _, _, _, _, _, _| todo!("TODO(port) Fighter_ChangeMotionState (fighter.c)");
        ftCommon_ClampAirDrift(fp: &mut Fighter) = crate::ftcommon::clamp_air_drift;
        ftCommon_8007D7FC(fp: &mut Fighter) = crate::ftcommon3::ftcommon_8007d7fc;
        ftCommon_8007D5D4(fp: &mut Fighter) = crate::ftcommon2::ftcommon_8007d5d4;
        fn_800D8378(g: G) = crate::tu::ftCo_ItemScopeFire::fn_800D8378;
        ftCo_ItemScopeFire_Anim(g: G) = crate::tu::ftCo_ItemScopeFire::ftCo_ItemScopeFire_Anim;
        ftCo_800CDE54(g: G) = crate::tu::ft_0cdd::ftco_800cde54;
        ftCo_800CDE74(g: G) = crate::tu::ft_0cdd::ftco_800cde74;
        ft_800841B8(g: G, cb: fn(G)) = crate::tu::ft_081b::ft_800841b8;
        ft_80082C74(g: G, cb: fn(G)) = crate::tu::ft_081b::ft_80082c74;
    }
}

pub unsafe extern "C" fn fn_800D8378_c(g: G) {
    ext::fn_800D8378(g)
}

pub fn fn_800D86E0(g: G) {
    let msid = ext::fn_800D769C(fp(g), ftCo_MS_ItemScopeEnd as i32);
    ext::Fighter_ChangeMotionState(g, msid, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
    fp(g).take_dmg_cb = Some(fn_800D8378_c);
}

pub fn fn_800D874C(g: G) {
    let msid = ext::fn_800D769C(fp(g), ftCo_MS_ItemScopeAirEnd as i32);
    ext::Fighter_ChangeMotionState(g, msid, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
    ext::ftCommon_ClampAirDrift(fp(g));
    fp(g).take_dmg_cb = Some(fn_800D8378_c);
}

pub fn fn_800D87C0(g: G) {
    ext::ftCommon_8007D7FC(fp(g));
    let msid = ext::fn_800D769C(fp(g), ftCo_MS_ItemScopeEnd as i32);
    let f = fp(g);
    ext::Fighter_ChangeMotionState(g, msid, 0x0C4C5080, f.cur_anim_frame, f.frame_speed_mul, 0.0, core::ptr::null_mut());
    fp(g).take_dmg_cb = Some(fn_800D8378_c);
}

pub fn fn_800D8838(g: G) {
    ext::ftCommon_8007D5D4(fp(g));
    let msid = ext::fn_800D769C(fp(g), ftCo_MS_ItemScopeAirEnd as i32);
    let f = fp(g);
    ext::Fighter_ChangeMotionState(g, msid, 0x0C4C5080, f.cur_anim_frame, f.frame_speed_mul, 0.0, core::ptr::null_mut());
    ext::ftCommon_ClampAirDrift(fp(g));
    fp(g).take_dmg_cb = Some(fn_800D8378_c);
}

pub fn ftCo_ItemScopeEnd_Anim(g: G) {
    ext::ftCo_ItemScopeFire_Anim(g);
}

pub fn ftCo_ItemScopeAirEnd_Anim(g: G) {
    ext::ftCo_ItemScopeFire_Anim(g);
}

pub fn ftCo_ItemScopeEnd_IASA(_g: G) {}

pub fn ftCo_ItemScopeAirEnd_IASA(_g: G) {}

pub fn ftCo_ItemScopeEnd_Phys(g: G) {
    ext::ftCo_800CDE54(g);
}

pub fn ftCo_ItemScopeAirEnd_Phys(g: G) {
    ext::ftCo_800CDE74(g);
}

pub fn ftCo_ItemScopeEnd_Coll(g: G) {
    ext::ft_800841B8(g, fn_800D8838);
}

pub fn ftCo_ItemScopeAirEnd_Coll(g: G) {
    ext::ft_80082C74(g, fn_800D87C0);
}
