#![allow(non_snake_case)]
//! Native port of src/melee/ft/kinds/ftCommon/ftCo_Down.c (lying on the ground: roll inputs).
//! Checked against the C by tests/tu_ftCo_Down.rs.

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
        ftCo_GetLStickAngle(fp: &Fighter) -> f32 = crate::ftcommon3::ftco_get_lstick_angle;
        ftCo_800DF678(fp: &Fighter) -> bool = crate::ft0df::ftco_800df678;
        ftCo_800DF644(fp: &Fighter) -> bool = crate::ft0df::ftco_800df644;
        Fighter_ChangeMotionState(g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G) = |_, _, _, _, _, _, _| todo!("TODO(port) Fighter_ChangeMotionState (fighter.c)");
        ftAnim_8006EBA4(g: G) = |_| todo!("TODO(port) ftAnim_8006EBA4 (ftanim.c)");
        ftCommon_SetGroundedKnockbackIfLanded(fp: &mut Fighter) = crate::ftcommon::set_grounded_knockback_if_landed;
        ftAnim_IsFramesRemaining(g: G) -> bool = crate::hooks::ftanim_is_frames_remaining;
        ft_8008A2BC(g: G) = crate::hooks::ft_8008a2bc;
        ft_80084FA8(g: G) = crate::ft084e::ft_80084fa8;
        ft_80084104(g: G) = |_| todo!("TODO(port) ft_80084104 (ft_081B.c)");
        ftCo_8009856C(g: G, msid: i32) = crate::tu::ftCo_DownAttack::ftCo_8009856C;
    }
}

/// static inline `inlineA0`: the stick is pushed far enough sideways to roll.
fn inlineA0(f: &Fighter) -> bool {
    let x = f.input.lstick[0].x;
    let ax = if x < 0.0 { -x } else { x };
    ax >= cd().x248 && ext::ftCo_GetLStickAngle(f) < cd().x20_radians
}

pub fn ftCo_Down_CheckInput(g: G) -> bool {
    let f = fp(g);
    let stick_x = if ext::ftCo_800DF678(f) {
        f.input.cstick[0].x
    } else if inlineA0(f) {
        f.input.lstick[0].x
    } else {
        return false;
    };
    let wait_u = f.motion_id == ftCo_MS_DownWaitU as _;
    let msid = if stick_x * f.facing_dir >= 0.0 {
        if wait_u { ftCo_MS_DownFowardU } else { ftCo_MS_DownFowardD }
    } else if wait_u {
        ftCo_MS_DownBackU
    } else {
        ftCo_MS_DownBackD
    };
    ftCo_80098324(g, msid as i32);
    true
}

/// static `ftCo_80098324`.
pub fn ftCo_80098324(g: G, msid: i32) {
    ext::Fighter_ChangeMotionState(g, msid, Ft_MF_None as u32, 0.0, 1.0, 0.0, core::ptr::null_mut());
    ext::ftAnim_8006EBA4(g);
    ext::ftCommon_SetGroundedKnockbackIfLanded(fp(g));
}

pub fn ftCo_Down_Anim(g: G) {
    if !ext::ftAnim_IsFramesRemaining(g) {
        ext::ft_8008A2BC(g);
    }
}

pub fn ftCo_Down_IASA(_g: G) {}

pub fn ftCo_Down_Phys(g: G) {
    ext::ft_80084FA8(g);
}

pub fn ftCo_Down_Coll(g: G) {
    ext::ft_80084104(g);
}

/// static inline `inlineB0`.
fn inlineB0(f: &Fighter) -> bool {
    (f.x67C as f32) < cd().x24C || (f.x67D as f32) < cd().x24C
}

pub fn ftCo_80098400(g: G) -> bool {
    let f = fp(g);
    if inlineB0(f) || ext::ftCo_800DF644(f) {
        let msid = if f.motion_id == ftCo_MS_DownBoundU as _ { ftCo_MS_DownAttackU } else { ftCo_MS_DownAttackD };
        ext::ftCo_8009856C(g, msid as i32);
        return true;
    }
    false
}
