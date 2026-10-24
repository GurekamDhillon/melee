#![allow(non_snake_case, non_upper_case_globals)]
//! Native port of src/melee/ft/kinds/ftCommon/ftCo_SquatRv.c (crouch release), with the
//! TARGET_PC m-ex `ucf_dbooc_squatrv_fix`. Two fused sites: `fabsf(s) * 80 - 0.0001` -> `fmadds`.
//! Calls out of the TU go through `ext` (unported ones panic until rewired; ported ones default
//! to the port). Checked against the C (whole TU, shipping contraction) by tests/tu_ftCo_SquatRv.rs.
use crate::types::*;
use melee_core::fp::fmadds;
use melee_runtime::cabi::f2i32;

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
        Mex_Enabled(name: &str) -> i32 = |name: &str| -> i32 { unsafe { crate::platform::Mex_Enabled(name) as i32 } };
        ftAnim_IsFramesRemaining(g: G) -> bool = crate::hooks::ftanim_is_frames_remaining;
        ft_8008A2BC(g: G) = crate::hooks::ft_8008a2bc;
        ftCo_800D68C0(g: G) -> bool = crate::tu::ftCo_Attack100::ftCo_800D68C0;
        ftCo_Attack100_CheckInput(g: G) -> bool = crate::tu::ftCo_Attack100::ftCo_Attack100_CheckInput;
        ftCo_AttackS4_CheckInput(g: G) -> bool = crate::tu::ftCo_AttackS4::ftCo_AttackS4_CheckInput;
        ftCo_AttackHi4_CheckInput(g: G) -> bool = crate::tu::ftCo_AttackHi4::ftCo_AttackHi4_CheckInput;
        ftCo_AttackLw4_CheckInput(g: G) -> bool = crate::tu::ftCo_AttackLw4::ftCo_AttackLw4_CheckInput;
        ftCo_AttackS3_CheckInput(g: G) -> bool = crate::tu::ftCo_AttackS3::ftCo_AttackS3_CheckInput;
        ftCo_AttackHi3_CheckInput(g: G) -> bool = crate::tu::ftCo_AttackHi3::ftCo_AttackHi3_CheckInput;
        ftCo_AttackLw3_CheckInput(g: G) -> bool = crate::tu::ftCo_AttackLw3::ftCo_AttackLw3_CheckInput;
        ftCo_Attack1_CheckInput(g: G) -> bool = crate::tu::ftCo_Attack1::ftCo_Attack1_CheckInput;
        ftCo_80091A4C(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftCo_Guard::ftCo_80091A4C(g as _) as _ } };
        ftCo_800DE9D8(g: G) -> bool = crate::tu::ftCo_AppealS::ftCo_800DE9D8;
        ftCo_Jump_CheckInput(g: G) -> bool = crate::tu::ftCo_Jump::ftCo_Jump_CheckInput;
        ftCo_Walk_CheckInput(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftCo_Walk::ftCo_Walk_CheckInput(g as _) as _ } };
        ft_80084F3C(g: G) = |g: G| { unsafe { let _ = crate::ft084e::ft_80084f3c(g as _); } };
        ft_80083F88(g: G) = |g: G| { unsafe { let _ = crate::tu::ft_081b::ft_80083f88(g as _); } };
    }
}

/// `(int) (fabsf(v) * 80.0f - 0.0001f) + 2` (fused; gwtool float->int).
#[inline(always)]
fn ucf_axis(v: f32) -> i32 {
    f2i32(fmadds(v.abs(), 80.0, -0.0001) as f64).wrapping_add(2)
}

pub fn ftCo_SquatRv_CheckInput(g: G) -> bool {
    // TARGET_PC: m-ex UCF 0.84 DBOOC SquatRv fix (opt-in).
    if ext::Mex_Enabled("ucf_dbooc_squatrv_fix") != 0 {
        let f = fp(g);
        let mut deadzone = cd().x94;
        if f.active_timer.lstick.x < 1 {
            let x = ucf_axis(f.input.lstick[0].x);
            let y = ucf_axis(f.input.lstick[0].y);
            if x.wrapping_mul(x).wrapping_add(y.wrapping_mul(y)) > 6400 {
                deadzone = 0.59;
            }
        }
        if f.input.lstick[0].y > -deadzone {
            ftCo_SquatRv_Enter(g);
            return true;
        }
        return false;
    }
    if fp(g).input.lstick[0].y > -cd().x94 {
        ftCo_SquatRv_Enter(g);
        return true;
    }
    false
}

/// static `ftCo_SquatRv_Enter`.
pub fn ftCo_SquatRv_Enter(g: G) {
    ext::Fighter_ChangeMotionState(g, ftCo_MS_SquatRv as i32, Ft_MF_None as u32, 0.0, 1.0, 0.0, core::ptr::null_mut());
}

pub fn ftCo_SquatRv_Anim(g: G) {
    if !ext::ftAnim_IsFramesRemaining(g) {
        ext::ft_8008A2BC(g);
    }
}

pub fn ftCo_SquatRv_IASA(g: G) {
    let _ = ext::ftCo_800D68C0(g)
        || ext::ftCo_Attack100_CheckInput(g)
        || ext::ftCo_AttackS4_CheckInput(g)
        || ext::ftCo_AttackHi4_CheckInput(g)
        || ext::ftCo_AttackLw4_CheckInput(g)
        || ext::ftCo_AttackS3_CheckInput(g)
        || ext::ftCo_AttackHi3_CheckInput(g)
        || ext::ftCo_AttackLw3_CheckInput(g)
        || ext::ftCo_Attack1_CheckInput(g)
        || ext::ftCo_80091A4C(g)
        || ext::ftCo_800DE9D8(g)
        || ext::ftCo_Jump_CheckInput(g)
        || ext::ftCo_Walk_CheckInput(g);
}

pub fn ftCo_SquatRv_Phys(g: G) {
    ext::ft_80084F3C(g);
}

pub fn ftCo_SquatRv_Coll(g: G) {
    ext::ft_80083F88(g);
}
