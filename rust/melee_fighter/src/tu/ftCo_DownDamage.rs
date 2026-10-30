#![allow(non_snake_case, unused_unsafe)]
//! Native port of src/melee/ft/kinds/ftCommon/ftCo_DownDamage.c (hit while lying down). Every
//! function of the TU; calls out of it go through `ext`. No fused float sites.
//! Checked against the C (whole TU, shipping contraction) by tests/tu_ftCo_DownDamage.rs.
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
        ftCo_8008DCE0(g: G, msid: i32, facing: f32) = |g: G, msid: i32, facing: f32| { unsafe { let _ = crate::tu::ftCo_Damage::ftCo_8008DCE0(g as _, msid as _, facing as _); } };
        ftCommon_8007E2F4(fp: &mut Fighter, v: i16) = crate::ftcommon2::ftcommon_8007e2f4;
        ftAnim_IsFramesRemaining(g: G) -> bool = crate::hooks::ftanim_is_frames_remaining;
        ftCo_80090780(g: G) = |g: G| { unsafe { let _ = crate::tu::ftCo_DamageFall::ftCo_80090780(g as _); } };
        ftCo_Fall_Enter(g: G) = crate::tu::ftCo_Fall::ftCo_Fall_Enter;
        ftCo_80098160(g: G, msid: i32) = crate::tu::ftCo_DownStand::ftCo_80098160;
        ftCo_80097F38(g: G) = |g: G| { unsafe { let _ = crate::tu::ftCo_DownBound::ftCo_80097F38(g as _); } };
        ftCo_Damage_Phys(g: G) = |g: G| { unsafe { let _ = crate::tu::ftCo_Damage::ftCo_Damage_Phys(g as _); } };
        ft_80082708(g: G) -> u32 = |g: G| -> u32 { unsafe { crate::tu::ft_081b::ft_80082708(g as _) as _ } };
        ftCo_8008FC94(g: G) = |g: G| { unsafe { let _ = crate::tu::ftCo_Damage::ftCo_8008FC94(g as _); } };
        ftCo_800C7CA0(g: G) -> bool = crate::tu::ftco_800c7ca0::ftco_800c7ca0;
        ft_80081DD4(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ft_081b::ft_80081dd4(g as _) as _ } };
        ftCommon_8007D7FC(fp: &mut Fighter) = crate::ftcommon3::ftcommon_8007d7fc;
        ftCo_800C1D38(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftCo_PassiveWall::ftCo_800C1D38(g as _) as _ } };
        ftCo_800C17CC(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftCo_FlyReflect::ftCo_800C17CC(g as _) as _ } };
    }
}

pub fn ftCo_8009F0F0(g: G) -> bool {
    melee_core::ct!("ftCo_8009F0F0");
    let f = fp(g);
    let m = f.motion_id as i32;
    if (m == ftCo_MS_DownBoundU
        || m == ftCo_MS_DownWaitU
        || m == ftCo_MS_DownDamageU
        || m == ftCo_MS_DownBoundD
        || m == ftCo_MS_DownWaitD
        || m == ftCo_MS_DownDamageD)
        && (f.x2224_b2() != 0 || f.dmg.x1838_percentTemp < cd().x428 as f32)
    {
        ftCo_8009F184(g);
        return true;
    }
    false
}

/// static `ftCo_8009F184`.
pub fn ftCo_8009F184(g: G) {
    melee_core::ct!("ftCo_8009F184");
    let f = fp(g);
    let msid = if f.motion_id as i32 == ftCo_MS_DownWaitU { ftCo_MS_DownDamageU } else { ftCo_MS_DownDamageD };
    ext::ftCo_8008DCE0(g, msid, f.facing_dir);
    ext::ftCommon_8007E2F4(fp(g), 1);
}

pub fn ftCo_DownDamage_Anim(g: G) {
    melee_core::ct!("ftCo_DownDamage_Anim");
    let f = fp(g);
    if f.x2224_b2() == 0 {
        unsafe { f.mv.co.downdamage.x0 -= 1.0 };
    }
    if ext::ftAnim_IsFramesRemaining(g) {
        return;
    }
    let f = fp(g);
    if f.ground_or_air == GA_Air {
        if f.x2224_b2() != 0 {
            ext::ftCo_80090780(g);
        } else {
            ext::ftCo_Fall_Enter(g);
        }
    } else if unsafe { f.mv.co.downdamage.x0 } <= 0.0 {
        let msid = if f.motion_id as i32 == ftCo_MS_DownDamageU { ftCo_MS_DownStandU } else { ftCo_MS_DownStandD };
        ext::ftCo_80098160(g, msid);
    } else {
        ext::ftCo_80097F38(g);
    }
}

pub fn ftCo_DownDamage_IASA(_g: G) { melee_core::ct!("ftCo_DownDamage_IASA");}

pub fn ftCo_DownDamage_Phys(g: G) {
    melee_core::ct!("ftCo_DownDamage_Phys");
    ext::ftCo_Damage_Phys(g);
}

pub fn ftCo_DownDamage_Coll(g: G) {
    melee_core::ct!("ftCo_DownDamage_Coll");
    if fp(g).ground_or_air == GA_Ground {
        if ext::ft_80082708(g) == GA_Ground {
            ext::ftCo_8008FC94(g);
        } else {
            ext::ftCo_800C7CA0(g);
        }
    } else if ext::ft_80081DD4(g) {
        ext::ftCommon_8007D7FC(fp(g));
    } else {
        let ef = fp(g).coll_data.env_flags;
        if ef & Collide_RightWallHug != 0 || ef & Collide_LeftWallHug != 0 {
            if ext::ftCo_800C1D38(g) {
                return;
            }
            ext::ftCo_800C17CC(g);
        }
    }
}
