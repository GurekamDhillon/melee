#![allow(non_snake_case)]
//! Native port of src/melee/ft/kinds/ftCommon/ftCo_CliffClimb.c (ledge getup). Every function
//! of the TU; calls out of it go through `ext` (unset hook = the port, or a TODO panic).
//! One fused site: ftCo_CliffClimb_Phys `x68C.z * facing + vec.x` -> `fmadds`.
//! Checked against the C (whole TU, shipping contraction) by tests/tu_ftCo_CliffClimb.rs.

use crate::types::*;
use melee_core::fp::fmadds;

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
        ftCo_GetLStickAngle(fp: &Fighter) -> f32 = crate::ftcommon3::ftco_get_lstick_angle;
        ftCo_GetCStickAngle(fp: &Fighter) -> f32 = crate::ftcommon3::ftco_get_cstick_angle;
        ftCo_800DF79C(fp: &Fighter) -> bool = crate::ft0df::ftco_800df79c;
        ftCo_Fall_Enter(g: G) = crate::hooks::ftco_fall_enter;
        Fighter_ChangeMotionState(g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G) = |g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G| { unsafe { let _ = crate::tu::fighter::Fighter_ChangeMotionState(g as _, msid as _, flags as _, start as _, speed as _, blend as _, other as _); } };
        ftAnim_8006EBA4(g: G) = |g: G| { unsafe { let _ = crate::tu::ftanim::ftAnim_8006EBA4(g as _); } };
        ftCommon_8007E2F4(fp: &mut Fighter, v: i16) = crate::ftcommon2::ftcommon_8007e2f4;
        ftCo_CliffCatch_Phys(g: G) = crate::tu::ftcliffcommon::ftco_cliff_catch_phys;
        ftAnim_IsFramesRemaining(g: G) -> bool = crate::hooks::ftanim_is_frames_remaining;
        ftCommon_8007D92C(g: G) = crate::ftcommon3::ftcommon_8007d92c;
        mpLib_80054ED8(line: i32) -> bool = |_| todo!("TODO(port) mpLib_80054ED8 (mplib.c)");
        // rp3 mpLib_80053ECC_Floor
        mpLib_80053ECC_Floor(line: i32, v: &mut Vec3) = |line: i32, v: &mut Vec3| unsafe { melee_world::tu::mplib::mpLib_80053ECC_Floor(line, v as _); };
        // rp3-end
        // rp3 mpLib_80053DA4_Floor
        mpLib_80053DA4_Floor(line: i32, v: &mut Vec3) = |line: i32, v: &mut Vec3| unsafe { melee_world::tu::mplib::mpLib_80053DA4_Floor(line, v as _); };
        // rp3-end
        ftCommon_8007D7FC(fp: &mut Fighter) = crate::ftcommon3::ftcommon_8007d7fc;
        ft_80084FA8(g: G) = crate::ft084e::ft_80084fa8;
        ft_800821DC(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ft_081b::ft_800821dc(g as _) as _ } };
        ftCo_8009AE14(g: G) = crate::tu::ftCo_CliffAttack::ftCo_8009AE14;
        ftCo_8009EF68(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftCo_StopCeil::ftCo_8009EF68(g as _) as _ } };
        ft_80084104(g: G) = |g: G| { unsafe { let _ = crate::tu::ft_081b::ft_80084104(g as _); } };
    }
}

/// `inlines.h` `ftCo_Cliff_EnterState`.
pub fn ftCo_Cliff_EnterState(g: G, msid: i32) {
    ext::Fighter_ChangeMotionState(g, msid, Ft_MF_None as u32, 0.0, 1.0, 0.0, core::ptr::null_mut());
    ext::ftAnim_8006EBA4(g);
    ext::ftCommon_8007E2F4(fp(g), 32);
    fp(g).set_x221D_b7(1);
}

#[inline(always)]
fn abs_c(x: f32) -> f32 {
    if x < 0.0 { -x } else { x }
}

pub fn ftCo_8009AA0C(g: G) -> bool {
    let f = fp(g);
    if abs_c(f.input.lstick[0].x) >= cd().x494 || abs_c(f.input.lstick[0].y) >= cd().x494 {
        let x = f.input.lstick[0].x;
        let a = ext::ftCo_GetLStickAngle(f);
        return ftCo_8009AAFC(g, true, x, a);
    }
    if ext::ftCo_800DF79C(f) {
        let x = f.input.cstick[0].x;
        let a = ext::ftCo_GetCStickAngle(f);
        return ftCo_8009AAFC(g, false, x, a);
    }
    fp(g).mv.co.cliff.x8 = 1;
    false
}

/// static `ftCo_8009AAFC`.
pub fn ftCo_8009AAFC(g: G, arg1: bool, stick_x: f32, angle: f32) -> bool {
    let f = fp(g);
    let c = cd();
    if angle > c.x20_radians || (angle > -c.x20_radians && stick_x * f.facing_dir >= 0.0) {
        if arg1 && unsafe { f.mv.co.cliff.x8 } != 0 {
            ftCo_8009AB9C(g);
            return true;
        }
        return false;
    }
    if unsafe { f.mv.co.cliff.x8 } != 0 {
        f.x2064_ledgeCooldown = c.ledge_cooldown;
        ext::ftCo_Fall_Enter(g);
        return true;
    }
    false
}

/// static `ftCo_8009AB9C`.
pub fn ftCo_8009AB9C(g: G) {
    let msid = if fp(g).dmg.x1830_percent < cd().x488 as f32 { ftCo_MS_CliffClimbQuick } else { ftCo_MS_CliffClimbSlow };
    ftCo_Cliff_EnterState(g, msid as i32);
    fp(g).set_x221D_b5(1);
    ext::ftCo_CliffCatch_Phys(g);
}

pub fn ftCo_CliffClimb_Anim(g: G) {
    if !ext::ftAnim_IsFramesRemaining(g) {
        ext::ftCommon_8007D92C(g);
    }
}

pub fn ftCo_CliffClimb_IASA(_g: G) {}

pub fn ftCo_CliffClimb_Phys(g: G) {
    let f = fp(g);
    if f.ground_or_air == GA_Air {
        let lid = unsafe { f.mv.co.cliff.ledge_id };
        if ext::mpLib_80054ED8(lid) {
            let mut vec = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
            if fp(g).facing_dir > 0.0 {
                ext::mpLib_80053ECC_Floor(lid, &mut vec);
            } else {
                ext::mpLib_80053DA4_Floor(lid, &mut vec);
            }
            let f = fp(g);
            f.cur_pos.x = fmadds(f.x68C_transNPos.z, f.facing_dir, vec.x);
            f.cur_pos.y = vec.y + f.x68C_transNPos.y;
            if f.ground_or_air == GA_Air && f.x68C_transNPos.z >= 0.0 && f.x68C_transNPos.y >= 0.0 {
                f.coll_data.floor.index = unsafe { f.mv.co.cliff.ledge_id };
                ext::ftCommon_8007D7FC(f);
            }
        } else {
            ext::ftCo_Fall_Enter(g);
        }
    } else {
        ext::ft_80084FA8(g);
    }
}

pub fn ftCo_CliffClimb_Coll(g: G) {
    if fp(g).ground_or_air == GA_Air {
        if ext::ft_800821DC(g) {
            ext::ftCo_8009AE14(g);
        } else if ext::ftCo_8009EF68(g) {
        }
    } else {
        ext::ft_80084104(g);
    }
}
