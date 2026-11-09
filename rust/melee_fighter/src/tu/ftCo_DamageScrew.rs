#![allow(non_snake_case)]
//! Native port of src/melee/ft/kinds/ftCommon/ftCo_DamageScrew.c. Checked against the C by
//! tests/tu_ftCo_DamageScrew.rs.

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
        ftCommon_8007DB58(g: G) = |a0| crate::tu::ftcommon::ftcommon_8007db58(a0);
        ftCo_8009750C(g: G) = |g: G| { unsafe { crate::tu::ftCo_Lift::ftCo_8009750C(g as _); } };
        ftCo_800DD168(g: G) = |g: G| { unsafe { crate::tu::ftCo_Throw::ftCo_800DD168(g as _); } };
        ftCommon_8007D5D4(fp: &mut Fighter) = crate::ftcommon2::ftcommon_8007d5d4;
        Fighter_ChangeMotionState(g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G) = |g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G| { unsafe { crate::tu::fighter::Fighter_ChangeMotionState(g as _, msid as _, flags as _, start as _, speed as _, blend as _, other as _); } };
        ftAnim_IsFramesRemaining(g: G) -> bool = crate::hooks::ftanim_is_frames_remaining;
        ftCo_800968C8(g: G) = |g: G| { unsafe { crate::tu::ftCo_FallSpecial::ftCo_800968C8(g as _); } };
        ft_80084DB0(g: G) = crate::ft084e::ft_80084db0;
        ftCo_AirCatchHit_Coll(g: G) = |g: G| { unsafe { crate::tu::ft_081b::ftco_air_catch_hit_coll(g as _); } };
        ft_80083B68(g: G) = |g: G| { unsafe { crate::tu::ft_081b::ft_80083b68(g as _); } };
    }
}

/// `ftCo_800D2FA4`: start the Screw Attack tumble if hit by a Screw Attack ball.
pub fn ftCo_800D2FA4(g: G) -> bool {
    melee_core::ct!("ftCo_800D2FA4");
    let f = fp(g);
    if f.dmg.x1860_element == HitElement_Scball as u32 {
        let x = unsafe { f.x2070.x0.x2071_b0_3() } as i32;
        match x {
            11 | 12 => return false,
            _ => {}
        }
        ftCo_800D3004(g);
        return true;
    }
    false
}

/// static inline `ftCo_ReleaseItemAndVictim` (ftCommon/inlines.h).
pub fn ftCo_ReleaseItemAndVictim(g: G) {
    melee_core::ct!("ftCo_ReleaseItemAndVictim");
    ext::ftCommon_8007DB58(g);
    ext::ftCo_8009750C(g);
    ext::ftCo_800DD168(g);
}

/// `ftCo_800D3004`.
pub fn ftCo_800D3004(g: G) {
    melee_core::ct!("ftCo_800D3004");
    ftCo_ReleaseItemAndVictim(g);
    fp(g).cmd_vars[0] = 0;
    if fp(g).ground_or_air == GA_Ground {
        ext::ftCommon_8007D5D4(fp(g));
        ext::Fighter_ChangeMotionState(g, ftCo_MS_DamageScrew as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
    } else {
        ext::Fighter_ChangeMotionState(g, ftCo_MS_DamageScrewAir as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
    }
    let f = fp(g);
    f.self_vel.x = 0.0;
    f.self_vel.y = f.co_attrs.screw_attack_launch_velocity;
    f.self_vel.z = 0.0;
}

pub fn ftCo_DamageScrew_Anim(g: G) {
    melee_core::ct!("ftCo_DamageScrew_Anim");
    if !ext::ftAnim_IsFramesRemaining(g) {
        ext::ftCo_800968C8(g);
    }
}

pub fn ftCo_DamageScrew_IASA(_g: G) { melee_core::ct!("ftCo_DamageScrew_IASA");}

pub fn ftCo_DamageScrew_Phys(g: G) {
    melee_core::ct!("ftCo_DamageScrew_Phys");
    ext::ft_80084DB0(g);
}

pub fn ftCo_DamageScrew_Coll(g: G) {
    melee_core::ct!("ftCo_DamageScrew_Coll");
    if fp(g).cmd_vars[0] != 0 {
        ext::ftCo_AirCatchHit_Coll(g);
        return;
    }
    ext::ft_80083B68(g);
}
