#![allow(non_snake_case)]
//! Native port of src/melee/ft/kinds/ftCommon/ftCo_ItemScrew.c (Screw Attack item jump). Every
//! function of the TU; calls out go through `ext`. Checked against the C by
//! tests/tu_ftCo_ItemScrew.rs.

use crate::types::*;

pub type G = *mut HSD_GObj;

#[inline(always)]
fn fp<'a>(g: G) -> &'a mut Fighter {
    unsafe { &mut *((*g).user_data as *mut Fighter) }
}
#[inline(always)]
fn cd() -> &'static ftCommonData {
    crate::hooks::common_data()
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
        ftCommon_8007D5D4(fp: &mut Fighter) = crate::ftcommon2::ftcommon_8007d5d4;
        Fighter_ChangeMotionState(g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G) = |g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G| { unsafe { let _ = crate::tu::fighter::Fighter_ChangeMotionState(g as _, msid as _, flags as _, start as _, speed as _, blend as _, other as _); } };
        ftCo_800CB110(g: G, arg1: bool, jump_mul: f32) = crate::tu::ftCo_Jump::ftCo_800CB110;
        ftAnim_IsFramesRemaining(g: G) -> bool = crate::hooks::ftanim_is_frames_remaining;
        ftCo_Fall_Enter(g: G) = |g| crate::tu::ftCo_Jump::ext::ftCo_Fall_Enter(g);
        ftCo_FallAerial_Enter(g: G) = |g| crate::tu::ftCo_JumpAerial::ext::ftCo_FallAerial_Enter(g);
        ftCo_Jump_IASA(g: G) = crate::tu::ftCo_Jump::ftCo_Jump_IASA;
        ftCo_Jump_Phys_Inner(g: G) = crate::tu::ftCo_Jump::ftCo_Jump_Phys_Inner;
        ftCo_JumpAerial_IASA(g: G) = crate::tu::ftCo_JumpAerial::ftCo_JumpAerial_IASA;
        ft_80084DB0(g: G) = crate::ft084e::ft_80084db0;
        ftCo_800CBAC4(g: G, msid: i32, vel: &Vec3, arg3: bool) = crate::tu::ftCo_JumpAerial::ftCo_800CBAC4;
        ft_800831CC(g: G, a: fn(G, i32) -> bool, b: fn(G)) = |g: G, a: fn(G, i32) -> bool, b: fn(G)| { unsafe { let _ = crate::tu::ft_081b::ft_800831cc(g as _, a as _, b as _); } };
        ftCo_80096CC8(g: G, line: i32) -> bool = |g: G, line: i32| -> bool { unsafe { crate::tu::ftCo_FallSpecial::ftCo_80096CC8(g as _, line as _) as _ } };
        ft_80082B1C(g: G) = |g: G| { unsafe { let _ = crate::tu::ft_081b::ft_80082b1c(g as _); } };
    }
}

pub fn ftCo_ItemScrew_Enter(g: G) {
    melee_core::ct!("ftCo_ItemScrew_Enter");
    ext::ftCommon_8007D5D4(fp(g));
    ext::Fighter_ChangeMotionState(g, ftCo_MS_ItemScrew as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
    ext::ftCo_800CB110(g, false, cd().x800);
    fp(g).set_x2227_b0(1);
}

pub fn ftCo_ItemScrew_Anim(g: G) {
    melee_core::ct!("ftCo_ItemScrew_Anim");
    if !ext::ftAnim_IsFramesRemaining(g) {
        ext::ftCo_Fall_Enter(g);
    }
}

pub fn ftCo_ItemScrew_IASA(g: G) {
    melee_core::ct!("ftCo_ItemScrew_IASA");
    ext::ftCo_Jump_IASA(g);
}

pub fn ftCo_ItemScrew_Phys(g: G) {
    melee_core::ct!("ftCo_ItemScrew_Phys");
    ext::ftCo_Jump_Phys_Inner(g);
}

pub fn ftCo_ItemScrew_Coll(g: G) {
    melee_core::ct!("ftCo_ItemScrew_Coll");
    ext::ft_800831CC(g, ext::ftCo_80096CC8, ext::ft_80082B1C);
}

pub fn ft_800D2E7C(g: G, v: &mut Vec3) {
    melee_core::ct!("ft_800D2E7C");
    v.x *= cd().x800;
    v.y *= cd().x800;
    ext::ftCommon_8007D5D4(fp(g));
    ext::ftCo_800CBAC4(g, ftCo_MS_ItemScrewAir as i32, v, true);
}

pub fn ftCo_ItemScrewAir_Anim(g: G) {
    melee_core::ct!("ftCo_ItemScrewAir_Anim");
    if !ext::ftAnim_IsFramesRemaining(g) {
        ext::ftCo_FallAerial_Enter(g);
    }
}

pub fn ftCo_ItemScrewAir_IASA(g: G) {
    melee_core::ct!("ftCo_ItemScrewAir_IASA");
    ext::ftCo_JumpAerial_IASA(g);
}

pub fn ftCo_ItemScrewAir_Phys(g: G) {
    melee_core::ct!("ftCo_ItemScrewAir_Phys");
    ext::ft_80084DB0(g);
}

pub fn ftCo_ItemScrewAir_Coll(g: G) {
    melee_core::ct!("ftCo_ItemScrewAir_Coll");
    ext::ft_800831CC(g, ext::ftCo_80096CC8, ext::ft_80082B1C);
}
