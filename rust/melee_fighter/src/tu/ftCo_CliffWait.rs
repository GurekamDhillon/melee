#![allow(non_snake_case)]
//! Native port of src/melee/ft/kinds/ftCommon/ftCo_CliffWait.c (ledge hang). Every function of
//! the TU. Checked against the C (whole TU, shipping contraction) by tests/tu_ftCo_CliffWait.rs.

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
        Fighter_ChangeMotionState(g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G) = |g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G| { unsafe { crate::tu::fighter::Fighter_ChangeMotionState(g as _, msid as _, flags as _, start as _, speed as _, blend as _, other as _); } };
        ftCommon_8007E2F4(fp: &mut Fighter, v: i16) = crate::ftcommon2::ftcommon_8007e2f4;
        ftColl_8007B760(g: G, v: i32) = |g: G, v: i32| { unsafe { crate::tu::ftcoll::ftcoll_8007b760(g as _, v as _); } };
        ftCo_8009AE38(g: G) -> bool = crate::tu::ftCo_CliffAttack::ftCo_8009AE38;
        ftCo_8009AFD4(g: G) -> bool = crate::tu::ftCo_CliffAttack::ftCo_8009AFD4;
        ftCo_8009B170(g: G) -> bool = crate::tu::ftCo_CliffJump::ftCo_8009B170;
        ftCo_8009AA0C(g: G) -> bool = crate::tu::ftCo_CliffClimb::ftCo_8009AA0C;
        ftCo_CliffCatch_Phys(g: G) = crate::tu::ftcliffcommon::ftco_cliff_catch_phys;
        ftCo_CliffCatch_Coll(g: G) = crate::tu::ftcliffcommon::ftco_cliff_catch_coll;
        ftCo_80090780(g: G) = |g: G| { unsafe { crate::tu::ftCo_DamageFall::ftCo_80090780(g as _); } };
    }
}

pub fn ftCo_8009A804(g: G) {
    melee_core::ct!("ftCo_8009A804");
    ext::Fighter_ChangeMotionState(g, ftCo_MS_CliffWait as i32, Ft_MF_SkipNametagVis as u32, 0.0, 1.0, 0.0, core::ptr::null_mut());
    let f = fp(g);
    ext::ftCommon_8007E2F4(f, 511);
    f.set_x221D_b7(1);
    let c = cd();
    f.mv.co.cliff.x8 = 0;
    f.mv.co.cliff.x4 = if f.dmg.x1830_percent < c.x488 as f32 { c.x48C } else { c.x490 };
    ext::ftColl_8007B760(g, c.x49C);
    ext::ftCommon_8007E2F4(fp(g), 511);
}

pub fn ftCo_CliffWait_Anim(g: G) {
    melee_core::ct!("ftCo_CliffWait_Anim");
    let f = fp(g);
    if unsafe { f.mv.co.cliff.x4 } > 0.0 {
        f.mv.co.cliff.x4 = unsafe { f.mv.co.cliff.x4 } - 1.0;
    }
}

pub fn ftCo_CliffWait_IASA(g: G) {
    melee_core::ct!("ftCo_CliffWait_IASA");
    if ext::ftCo_8009AE38(g) {
        return;
    }
    if ext::ftCo_8009AFD4(g) {
        return;
    }
    if ext::ftCo_8009B170(g) {
        return;
    }
    if ext::ftCo_8009AA0C(g) {
        return;
    }
    ftCo_8009A9AC(g);
}

pub fn ftCo_CliffWait_Phys(g: G) {
    melee_core::ct!("ftCo_CliffWait_Phys");
    ext::ftCo_CliffCatch_Phys(g);
}

pub fn ftCo_CliffWait_Coll(g: G) {
    melee_core::ct!("ftCo_CliffWait_Coll");
    ext::ftCo_CliffCatch_Coll(g);
}

/// static `ftCo_8009A9AC`.
pub fn ftCo_8009A9AC(g: G) -> bool {
    melee_core::ct!("ftCo_8009A9AC");
    let f = fp(g);
    if unsafe { f.mv.co.cliff.x4 } <= 0.0 {
        f.x2064_ledgeCooldown = cd().ledge_cooldown;
        f.set_x2227_b1(1);
        ext::ftCo_80090780(g);
        return true;
    }
    false
}
