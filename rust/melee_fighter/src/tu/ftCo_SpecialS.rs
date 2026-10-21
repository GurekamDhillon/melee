#![allow(non_snake_case, non_upper_case_globals)]
//! Native port of src/melee/ft/kinds/ftCommon/ftCo_SpecialS.c (side-B input check + entry), with
//! the TARGET_PC paths (script caps gate, Geno special override, m-ex SpecialS dispatcher).
//! One fused site: doEnter `gr_vel += -(gr_vel * (1 - r)) * mul` -> `fmadds`.
//! Calls out of the TU go through `ext`. Checked against the C (whole TU, shipping contraction)
//! by tests/tu_ftCo_SpecialS.rs.
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
        // TARGET_PC `FT_CAPS_ACTION_ALLOWED(fp, flag)`.
        ScriptGame_CapsActionAllowed(fp: &Fighter, flag: i32) -> i32 = |_, _| todo!("TODO(port) ScriptGame_CapsActionAllowed (pc/gameworld/script_game.c)");
        // `ftData_SpecialS[kind]` (ftdata.c table).
        ftData_SpecialS(kind: u32) -> HSD_GObjEvent = |_| todo!("TODO(port) ftData_SpecialS (ftdata.c)");
        ftCommon_UpdateFacing(fp: &mut Fighter) = |_| todo!("TODO(port) ftCommon_UpdateFacing (ftcommon.c)");
        ft_GetGroundFrictionMultiplier(fp: &mut Fighter) -> f32 = |_| todo!("TODO(port) ft_GetGroundFrictionMultiplier (ft_0877.c)");
        Geno_SpecialEnter(g: G, which: i32) -> i32 = |_, _| todo!("TODO(pc geno) Geno_SpecialEnter");
        Mex_SpecialSDispatch(kind: i32, g: G, vanilla: HSD_GObjEvent) = |_, _, _| todo!("TODO(pc mex) Mex_SpecialSDispatch");
    }
}

#[inline(always)]
fn caps(f: &Fighter) -> bool {
    ext::ScriptGame_CapsActionAllowed(f, 16) != 0
}

/// Check `SpecialS` input without entering the state.
pub fn ftCo_SpecialS_HasInput(f: &Fighter) -> bool {
    if !caps(f) {
        return false;
    }
    f.input.pressed_buttons & HSD_PAD_B as u32 != 0 && abs_c(f.input.lstick[0].x) >= cd().x218
}

pub fn ftCo_SpecialS_CheckInput(g: G) -> bool {
    let f = fp(g);
    if !caps(f) {
        return false;
    }
    if ext::ftData_SpecialS(f.kind as u32).is_none() {
        return false;
    }
    if f.x688 == 0 {
        if f.input.lstick[0].x * f.facing_dir < -cd().x220 {
            ext::ftCommon_UpdateFacing(fp(g));
        }
        doEnter(g);
        return true;
    }
    false
}

/// static `doEnter`.
pub fn doEnter(g: G) {
    let f = fp(g);
    let gv = f.gr_vel;
    let a = -(gv * (1.0 - f.co_attrs.specials_ground_speed_retention));
    let m = ext::ft_GetGroundFrictionMultiplier(f);
    let f = fp(g);
    f.gr_vel = fmadds(a, m, f.gr_vel);
    if ext::Geno_SpecialEnter(g, 1) == 0 {
        let kind = fp(g).kind;
        ext::Mex_SpecialSDispatch(kind as i32, g, ext::ftData_SpecialS(kind as u32));
    }
}
