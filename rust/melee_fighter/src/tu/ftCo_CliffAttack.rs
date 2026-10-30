#![allow(non_snake_case)]
//! Native port of src/melee/ft/kinds/ftCommon/ftCo_CliffAttack.c (ledge attack). Every function
//! of the TU. Checked against the C (whole TU, shipping contraction) by tests/tu_ftCo_CliffAttack.rs.

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
        Fighter_ChangeMotionState(g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G) = |g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G| { unsafe { let _ = crate::tu::fighter::Fighter_ChangeMotionState(g as _, msid as _, flags as _, start as _, speed as _, blend as _, other as _); } };
        ftAnim_8006EBA4(g: G) = |g: G| { unsafe { let _ = crate::tu::ftanim::ftAnim_8006EBA4(g as _); } };
        ftCommon_8007E2F4(fp: &mut Fighter, v: i16) = crate::ftcommon2::ftcommon_8007e2f4;
        ftCo_CliffCatch_Phys(g: G) = crate::tu::ftcliffcommon::ftco_cliff_catch_phys;
        ftCommon_8007D7FC(fp: &mut Fighter) = crate::ftcommon3::ftcommon_8007d7fc;
        ftCo_800DF6F8(fp: &Fighter) -> bool = crate::ft0df::ftco_800df6f8;
        ftCo_800DF72C(fp: &Fighter) -> bool = crate::ft0df::ftco_800df72c;
        ftCo_CliffClimb_Anim(g: G) = crate::tu::ftCo_CliffClimb::ftCo_CliffClimb_Anim;
        ftCo_CliffClimb_Phys(g: G) = crate::tu::ftCo_CliffClimb::ftCo_CliffClimb_Phys;
        ftCo_CliffClimb_Coll(g: G) = crate::tu::ftCo_CliffClimb::ftCo_CliffClimb_Coll;
        ftCo_8009B040(g: G) = crate::tu::ftCo_CliffEscape::ftCo_8009B040;
    }
}

/// `inlines.h` `ftCo_Cliff_EnterState`.
fn cliff_enter_state(g: G, msid: i32) {
    ext::Fighter_ChangeMotionState(g, msid, Ft_MF_None as u32, 0.0, 1.0, 0.0, core::ptr::null_mut());
    ext::ftAnim_8006EBA4(g);
    ext::ftCommon_8007E2F4(fp(g), 32);
    fp(g).set_x221D_b7(1);
}

pub fn ftCo_8009AE14(g: G) {
    melee_core::ct!("ftCo_8009AE14");
    ext::ftCommon_8007D7FC(fp(g));
}

pub fn ftCo_8009AE38(g: G) -> bool {
    melee_core::ct!("ftCo_8009AE38");
    let f = fp(g);
    if f.input.pressed_buttons & (HSD_PAD_A | HSD_PAD_B) as u32 != 0 || ext::ftCo_800DF6F8(f) {
        ftCo_8009AEA4(g);
        return true;
    }
    false
}

/// static `ftCo_8009AEA4`.
pub fn ftCo_8009AEA4(g: G) {
    melee_core::ct!("ftCo_8009AEA4");
    let msid = if fp(g).dmg.x1830_percent < cd().x488 as f32 { ftCo_MS_CliffAttackQuick } else { ftCo_MS_CliffAttackSlow };
    cliff_enter_state(g, msid as i32);
    fp(g).set_x221D_b5(1);
    ext::ftCo_CliffCatch_Phys(g);
}

pub fn ftCo_CliffAttack_Anim(g: G) {
    melee_core::ct!("ftCo_CliffAttack_Anim");
    ext::ftCo_CliffClimb_Anim(g);
}

pub fn ftCo_CliffAttack_IASA(_g: G) { melee_core::ct!("ftCo_CliffAttack_IASA");}

pub fn ftCo_CliffAttack_Phys(g: G) {
    melee_core::ct!("ftCo_CliffAttack_Phys");
    ext::ftCo_CliffClimb_Phys(g);
}

pub fn ftCo_CliffAttack_Coll(g: G) {
    melee_core::ct!("ftCo_CliffAttack_Coll");
    ext::ftCo_CliffClimb_Coll(g);
}

pub fn ftCo_8009AFD4(g: G) -> bool {
    melee_core::ct!("ftCo_8009AFD4");
    let f = fp(g);
    if f.input.pressed_buttons & HSD_PAD_LR as u32 != 0 || ext::ftCo_800DF72C(f) {
        ext::ftCo_8009B040(g);
        return true;
    }
    false
}
