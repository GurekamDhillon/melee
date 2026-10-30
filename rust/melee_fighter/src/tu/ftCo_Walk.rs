#![allow(non_snake_case, non_upper_case_globals)]
//! Native port of src/melee/ft/kinds/ftCommon/ftCo_Walk.c (every function). Calls out of
//! the TU go through `ext` (unported ones panic until rewired). Checked against the C (whole TU,
//! shipping contraction) by tests/tu_ftCo_Walk.rs.
use crate::types::*;

pub type G = *mut HSD_GObj;

#[inline(always)]
fn fp<'a>(g: G) -> &'a mut Fighter {
    unsafe { &mut *((*g).user_data as *mut Fighter) }
}

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

/// Calls out of the TU. `hook::<name>` overrides; unset = the port, or a TODO panic.
pub mod ext {
    use super::*;
    ext_fns! {
        ftCo_Catch_CheckInput(g: G) -> bool = crate::tu::ftCo_Catch::ftCo_Catch_CheckInput;
        ftCo_SpecialS_CheckInput(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftCo_SpecialS::ftCo_SpecialS_CheckInput(g as _) as _ } };
        ftCo_Attack100_CheckInput(g: G) -> bool = crate::tu::ftCo_Attack100::ftCo_Attack100_CheckInput;
        ftCo_800D6824(g: G) -> bool = crate::tu::ftCo_Attack100::ftCo_800D6824;
        ftCo_800D68C0(g: G) -> bool = crate::tu::ftCo_Attack100::ftCo_800D68C0;
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
        ftCo_Dash_CheckInput(g: G) -> bool = crate::tu::ftCo_Dash::ftCo_Dash_CheckInput;
        ftCo_800D5FB0(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftCo_Squat::ftCo_800D5FB0(g as _) as _ } };
        ft_8008A244(g: G) -> bool = |g: G| -> bool { unsafe { crate::ft084e::ft_8008a244(g as _) as _ } };
        ftWalkCommon_800DFC70(g: G) -> bool = crate::tu::ftwalkcommon::ftwalkcommon_800dfc70;
        ftWalkCommon_800DFCA4(g: G, msid: FtMotionId, fl: MotionFlags, start: f32, a: f32, b: f32, c: f32, d: f32, e: f32, h: f32, accel: f32) = crate::tu::ftwalkcommon::ftwalkcommon_800dfca4;
        ftWalkCommon_800DFDDC(g: G) = crate::tu::ftwalkcommon::ftwalkcommon_800dfddc;
        ftWalkCommon_800DFEC8(g: G, cb: fn(G, f32)) = crate::tu::ftwalkcommon::ftwalkcommon_800dfec8;
        ftWalkCommon_800E0060(g: G) = crate::tu::ftwalkcommon::ftwalkcommon_800e0060;
        ft_80084280(g: G) = crate::tu::ft_081b::ft_80084280;
        ftCo_CalcYScaledKnockback(a: f32, scale: f32, c: f32) -> f32 = crate::tu::ftchangeparam::ftco_calc_y_scaled_knockback;
        // `Fighter_804D651C->x0`, `Fighter_804D6524->xC`, `Fighter_804D6520->x0` (PlCo data).
        Fighter_804D651C_x0() -> f32 = || crate::tu::ftchangeparam::ext::d651c().x0;
        Fighter_804D6524_xC() -> f32 = || crate::tu::ftchangeparam::ext::d6524().xC;
        Fighter_804D6520_x0() -> f32 = || crate::hooks::fighter_804d6520().x0;
    }
}

pub fn ftCo_Walk_CheckInput(g: G) -> bool {
    melee_core::ct!("ftCo_Walk_CheckInput");
    if ext::ftWalkCommon_800DFC70(g) {
        ftCo_Walk_Enter(g, 0.0);
        return true;
    }
    false
}

pub fn ftCo_Walk_CheckInput_Ottotto(g: G) -> bool {
    melee_core::ct!("ftCo_Walk_CheckInput_Ottotto");
    let f = fp(g);
    if f.input.lstick[0].x * f.facing_dir >= crate::hooks::common_data().teeter_walk_threshold && ext::ftWalkCommon_800DFC70(g) {
        ftCo_Walk_Enter(g, 0.0);
        return true;
    }
    false
}

pub fn ftCo_Walk_Enter(g: G, arg8: f32) {
    melee_core::ct!("ftCo_Walk_Enter");
    let f = fp(g);
    let mut accel_mul = 1.0f32;
    if f.is_metal() != 0 {
        accel_mul = ext::Fighter_804D651C_x0();
    }
    if f.x34_scale.y != 1.0 {
        accel_mul = ext::ftCo_CalcYScaledKnockback(accel_mul, f.x34_scale.y, ext::Fighter_804D6524_xC());
    }
    if !f.x197C.is_null() {
        accel_mul *= ext::Fighter_804D6520_x0();
    }
    let (a0, a1, a2) = (f.x2DC, f.x2E0, f.x2E4);
    let (s, m, fa) = (f.co_attrs.slow_walk_max, f.co_attrs.mid_walk_point, f.co_attrs.fast_walk_min);
    ext::ftWalkCommon_800DFCA4(g, ftCo_MS_WalkSlow as FtMotionId, Ft_MF_None, arg8, a0, a1, a2, s, m, fa, accel_mul);
}

pub fn ftCo_Walk_Anim(g: G) {
    melee_core::ct!("ftCo_Walk_Anim");
    ext::ftWalkCommon_800DFDDC(g);
}

pub fn ftCo_Walk_IASA(g: G) {
    melee_core::ct!("ftCo_Walk_IASA");
    if ext::ftCo_Catch_CheckInput(g) {
        return;
    }
    if ext::ftCo_SpecialS_CheckInput(g) {
        return;
    }
    if ext::ftCo_Attack100_CheckInput(g) {
        return;
    }
    if ext::ftCo_800D6824(g) {
        return;
    }
    if ext::ftCo_800D68C0(g) {
        return;
    }
    if ext::ftCo_AttackS4_CheckInput(g) {
        return;
    }
    if ext::ftCo_AttackHi4_CheckInput(g) {
        return;
    }
    if ext::ftCo_AttackLw4_CheckInput(g) {
        return;
    }
    if ext::ftCo_AttackS3_CheckInput(g) {
        return;
    }
    if ext::ftCo_AttackHi3_CheckInput(g) {
        return;
    }
    if ext::ftCo_AttackLw3_CheckInput(g) {
        return;
    }
    if ext::ftCo_Attack1_CheckInput(g) {
        return;
    }
    if ext::ftCo_80091A4C(g) {
        return;
    }
    if ext::ftCo_800DE9D8(g) {
        return;
    }
    if ext::ftCo_Jump_CheckInput(g) {
        return;
    }
    if ext::ftCo_Dash_CheckInput(g) {
        return;
    }
    if ext::ftCo_800D5FB0(g) {
        return;
    }
    if ext::ft_8008A244(g) {
        return;
    }
    ext::ftWalkCommon_800DFEC8(g, ftCo_Walk_Enter);
}

pub fn ftCo_Walk_Phys(g: G) {
    melee_core::ct!("ftCo_Walk_Phys");
    ext::ftWalkCommon_800E0060(g);
}

pub fn ftCo_Walk_Coll(g: G) {
    melee_core::ct!("ftCo_Walk_Coll");
    ext::ft_80084280(g);
}
