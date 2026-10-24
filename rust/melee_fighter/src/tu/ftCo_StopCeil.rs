#![allow(non_snake_case, non_upper_case_globals)]
//! Native port of src/melee/ft/kinds/ftCommon/ftCo_StopCeil.c (bonk on a ceiling). No fused sites.
//! Calls out of the TU go through `ext` (unported ones panic until rewired; ported ones default
//! to the port). Checked against the C (whole TU, shipping contraction) by tests/tu_ftCo_StopCeil.rs.
use crate::types::*;

pub type G = *mut HSD_GObj;

#[inline(always)]
fn fp<'a>(g: G) -> &'a mut Fighter {
    unsafe { &mut *((*g).user_data as *mut Fighter) }
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
        Fighter_ChangeMotionState(g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G) = |g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G| { unsafe { let _ = crate::tu::fighter::Fighter_ChangeMotionState(g as _, msid as _, flags as _, start as _, speed as _, blend as _, other as _); } };
        ftKb_SpecialN_800F1F1C(g: G, v: &mut Vec3) = |_, _| todo!("TODO(port) ftKb_SpecialN_800F1F1C (ftkirbyspecialn.c)");
        ft_80082D40(g: G, y: f32) = |g: G, y: f32| { unsafe { let _ = crate::tu::ft_081b::ft_80082d40(g as _, y as _); } };
        ftAnim_IsFramesRemaining(g: G) -> bool = crate::hooks::ftanim_is_frames_remaining;
        ftCo_Fall_Enter(g: G) = crate::tu::ftCo_Fall::ftCo_Fall_Enter;
        ft_80083464(g: G, line_cb: unsafe extern "C" fn(G, i32) -> bool, cb: unsafe extern "C" fn(G)) = |g: G, line_cb: unsafe extern "C" fn(G, i32) -> bool, cb: unsafe extern "C" fn(G)| { unsafe { let _ = crate::bridge::with_coll_ev(line_cb, Some(cb), |a, b| crate::tu::ft_081b::ft_80083464(g as _, a, b)); } };
        // Address-taken only (passed to `ft_80083464`).
        ftCo_80096CC8(g: G, line_id: i32) -> bool = |g: G, line_id: i32| -> bool { unsafe { crate::tu::ftCo_FallSpecial::ftCo_80096CC8(g as _, line_id as _) as _ } };
        ft_80082B1C(g: G) = |g: G| { unsafe { let _ = crate::tu::ft_081b::ft_80082b1c(g as _); } };
    }
}

/// `ftCo_80096CC8` as a callback value.
pub unsafe extern "C" fn ftCo_80096CC8_cb(g: G, line_id: i32) -> bool {
    ext::ftCo_80096CC8(g, line_id)
}
/// `ft_80082B1C` as a callback value.
pub unsafe extern "C" fn ft_80082B1C_cb(g: G) {
    ext::ft_80082B1C(g)
}

pub fn ftCo_8009EF68(g: G) -> bool {
    if fp(g).coll_data.env_flags & Collide_CeilingHug as i32 != 0 {
        ftCo_8009EFA4(g);
        return true;
    }
    false
}

/// static `ftCo_8009EFA4`.
pub fn ftCo_8009EFA4(g: G) {
    let f = fp(g);
    f.anon_2210.throw_flags = 0;
    let mut vec = Vec3 { x: 0.0, y: f.coll_data.ecb.top.y, z: 0.0 };
    ext::ftKb_SpecialN_800F1F1C(g, &mut vec);
    ext::Fighter_ChangeMotionState(g, ftCo_MS_StopCeil as i32, Ft_MF_None as u32, 0.0, 1.0, 0.0, core::ptr::null_mut());
    let f = fp(g);
    let pos_y = f.cur_pos.y;
    let transn_y = f.x68C_transNPos.y;
    f.cur_pos.y = transn_y + (pos_y + vec.y);
    ext::ft_80082D40(g, pos_y);
    let f = fp(g);
    f.self_vel.z = 0.0;
    f.self_vel.y = 0.0;
}

pub fn ftCo_StopCeil_Anim(g: G) {
    let f = fp(g);
    // ftCheckThrowB3
    let b3 = unsafe {
        if f.anon_2210.anon_0.throw_flags_b3() != 0 {
            f.anon_2210.anon_0.set_throw_flags_b3(0);
            true
        } else {
            false
        }
    };
    if b3 || !ext::ftAnim_IsFramesRemaining(g) {
        ext::ftCo_Fall_Enter(g);
    }
}

pub fn ftCo_StopCeil_IASA(_g: G) {}

pub fn ftCo_StopCeil_Phys(_g: G) {}

pub fn ftCo_StopCeil_Coll(g: G) {
    ext::ft_80083464(g, ftCo_80096CC8_cb, ft_80082B1C_cb);
}
