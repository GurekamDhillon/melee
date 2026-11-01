#![allow(non_snake_case, non_upper_case_globals)]
//! Native port of src/melee/ft/kinds/ftCommon/ftCo_StopWall.c (bonk on a wall). One fused site:
//! ftCo_8009EE30 `-(transN.z * -facing - (pos.x + vec.x))` -> `-fmadds(z, -facing, -(pos.x + vec.x))`.
//! Calls out of the TU go through `ext` (unported ones panic until rewired; ported ones default
//! to the port). Checked against the C (whole TU, shipping contraction) by tests/tu_ftCo_StopWall.rs.
use crate::types::*;
use melee_core::fp::fmadds;

pub type G = *mut HSD_GObj;

#[inline(always)]
fn fp<'a>(g: G) -> &'a mut Fighter {
    unsafe { &mut *((*g).user_data as *mut Fighter) }
}
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
        Fighter_ChangeMotionState(g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G) = |g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G| { unsafe { crate::tu::fighter::Fighter_ChangeMotionState(g as _, msid as _, flags as _, start as _, speed as _, blend as _, other as _); } };
        ftKb_SpecialN_800F1F1C(g: G, v: &mut Vec3) = |_, _| todo!("TODO(port) ftKb_SpecialN_800F1F1C (ftkirbyspecialn.c)");
        ft_800843FC(g: G) = |g: G| { unsafe { crate::tu::ft_081b::ft_800843fc(g as _); } };
        ftCommon_8007E2FC(g: G) = |_| todo!("ftCommon_8007E2FC: installed by melee_link");
        ftAnim_IsFramesRemaining(g: G) -> bool = crate::hooks::ftanim_is_frames_remaining;
        ft_8008A2BC(g: G) = crate::hooks::ft_8008a2bc;
    }
}

pub fn ftCo_8009EDA4(g: G) -> bool {
    melee_core::ct!("ftCo_8009EDA4");
    let f = fp(g);
    let env = f.coll_data.env_flags;
    if (f.facing_dir == -1.0 && env & Collide_RightWallHug as i32 != 0)
        || (f.facing_dir == 1.0 && env & Collide_LeftWallHug as i32 != 0)
    {
        if abs_c(f.gr_vel) > f.co_attrs.walk_max_vel {
            ftCo_8009EE30(g);
            return true;
        }
    }
    false
}

/// static `ftCo_8009EE30`.
pub fn ftCo_8009EE30(g: G) {
    melee_core::ct!("ftCo_8009EE30");
    let f = fp(g);
    let coll = &f.coll_data;
    let mut vec = if coll.env_flags & Collide_RightWallMask as i32 != 0 {
        Vec3 { x: coll.ecb.left.x, y: coll.ecb.left.y, z: 0.0 }
    } else {
        Vec3 { x: coll.ecb.right.x, y: coll.ecb.right.y, z: 0.0 }
    };
    ext::ftKb_SpecialN_800F1F1C(g, &mut vec);
    ext::Fighter_ChangeMotionState(g, ftCo_MS_StopWall as i32, Ft_MF_None as u32, 0.0, 1.0, 0.0, core::ptr::null_mut());
    let f = fp(g);
    f.cur_pos.x = -fmadds(f.x68C_transNPos.z, -f.facing_dir, -(f.cur_pos.x + vec.x));
    ext::ft_800843FC(g);
    ext::ftCommon_8007E2FC(g);
}

pub fn ftCo_StopWall_Anim(g: G) {
    melee_core::ct!("ftCo_StopWall_Anim");
    if !ext::ftAnim_IsFramesRemaining(g) {
        ext::ft_8008A2BC(g);
    }
}

pub fn ftCo_StopWall_IASA(_g: G) { melee_core::ct!("ftCo_StopWall_IASA");}

pub fn ftCo_StopWall_Phys(_g: G) { melee_core::ct!("ftCo_StopWall_Phys");}

pub fn ftCo_StopWall_Coll(g: G) {
    melee_core::ct!("ftCo_StopWall_Coll");
    ext::ft_800843FC(g);
}
