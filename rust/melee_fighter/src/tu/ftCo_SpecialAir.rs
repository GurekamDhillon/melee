#![allow(non_snake_case, non_upper_case_globals)]
//! Native port of src/melee/ft/kinds/ftCommon/ftCo_SpecialAir.c (TARGET_PC build). Every function of the TU;
//! calls out of it go through `ext` (unported ones panic until rewired). Checked against the C
//! (whole TU, shipping contraction) by tests/tu_ftCo_SpecialAir.rs.
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
        // TARGET_PC `FT_CAPS_ACTION_ALLOWED(fp, flag)` = `ScriptGame_CapsActionAllowed`.
        ScriptGame_CapsActionAllowed(fp: &Fighter, flag: i32) -> i32 = |_, _| todo!("TODO(port) ScriptGame_CapsActionAllowed (pc/gameworld/script_game.c)");
        // `ftData_SpecialAir*[kind]` (ftdata.c tables).
        ftData_SpecialAirHi(kind: u32) -> HSD_GObjEvent = |k: u32| unsafe { crate::tu::ftdata::ftData_SpecialAirHi[k as usize] };
        ftData_SpecialAirLw(kind: u32) -> HSD_GObjEvent = |k: u32| unsafe { crate::tu::ftdata::ftData_SpecialAirLw[k as usize] };
        ftData_SpecialAirS(kind: u32) -> HSD_GObjEvent = |k: u32| unsafe { crate::tu::ftdata::ftData_SpecialAirS[k as usize] };
        ftData_SpecialAirN(kind: u32) -> HSD_GObjEvent = |k: u32| unsafe { crate::tu::ftdata::ftData_SpecialAirN[k as usize] };
        Geno_SpecialEnter(g: G, which: i32) -> i32 = |_, _| todo!("TODO(pc geno) Geno_SpecialEnter");
        Mex_SpecialHiAirDispatch(kind: i32, g: G, vanilla: HSD_GObjEvent) = |kind: i32, g: G, vanilla: HSD_GObjEvent| { unsafe { let _ = crate::platform::mex_dispatch_ev(crate::platform::MEX_EVENT_SPECIAL_HI_AIR, kind, g, vanilla); } };
        Mex_SpecialLwAirDispatch(kind: i32, g: G, vanilla: HSD_GObjEvent) = |kind: i32, g: G, vanilla: HSD_GObjEvent| { unsafe { let _ = crate::platform::mex_dispatch_ev(crate::platform::MEX_EVENT_SPECIAL_LW_AIR, kind, g, vanilla); } };
        Mex_SpecialSAirDispatch(kind: i32, g: G, vanilla: HSD_GObjEvent) = |kind: i32, g: G, vanilla: HSD_GObjEvent| { unsafe { let _ = crate::platform::mex_dispatch_ev(crate::platform::MEX_EVENT_SPECIAL_S_AIR, kind, g, vanilla); } };
        Mex_SpecialNAirDispatch(kind: i32, g: G, vanilla: HSD_GObjEvent) = |kind: i32, g: G, vanilla: HSD_GObjEvent| { unsafe { let _ = crate::platform::mex_dispatch_ev(crate::platform::MEX_EVENT_SPECIAL_N_AIR, kind, g, vanilla); } };
        ftCommon_UpdateFacing(f: &mut Fighter) = crate::ftcommon::update_facing;
    }
}

pub fn ftCo_SpecialAir_CheckInput(g: G) -> bool {
    melee_core::ct!("ftCo_SpecialAir_CheckInput");
    if ext::ScriptGame_CapsActionAllowed(fp(g), 16) == 0 {
        return false;
    }
    let c = cd();
    let f = fp(g);
    if f.input.pressed_buttons & HSD_PAD_B as u32 == 0 {
        return false;
    }
    let kind = f.kind as u32;
    if f.input.lstick[0].y >= c.x21C {
        let v = ext::ftData_SpecialAirHi(kind);
        if v.is_none() {
            return false;
        }
        if ext::Geno_SpecialEnter(g, 6) == 0 {
            ext::Mex_SpecialHiAirDispatch(kind as i32, g, v);
        }
        fp(g).set_x2227_b5(1);
        return true;
    }
    if f.input.lstick[0].y <= -c.x21C {
        let v = ext::ftData_SpecialAirLw(kind);
        if v.is_none() {
            return false;
        }
        if ext::Geno_SpecialEnter(g, 7) == 0 {
            ext::Mex_SpecialLwAirDispatch(kind as i32, g, v);
        }
        fp(g).set_x2227_b5(1);
        return true;
    }
    let lsx = f.input.lstick[0].x;
    if (if lsx < 0.0 { -lsx } else { lsx }) >= c.x218 {
        let v = ext::ftData_SpecialAirS(kind);
        if v.is_none() {
            return false;
        }
        if f.input.lstick[0].x * f.facing_dir < -c.x220 {
            ext::ftCommon_UpdateFacing(f);
        }
        if ext::Geno_SpecialEnter(g, 5) == 0 {
            ext::Mex_SpecialSAirDispatch(kind as i32, g, v);
        }
        fp(g).set_x2227_b5(1);
        return true;
    }
    let v = ext::ftData_SpecialAirN(kind);
    if v.is_none() {
        return false;
    }
    if (f.active_duration.lstick.x as i32) < c.x224
        && ((f.facing_dir == -1.0 && f.x2228_b7() == 1) || (f.facing_dir == 1.0 && f.x2228_b7() == 0))
    {
        f.facing_dir = -f.facing_dir;
    }
    if ext::Geno_SpecialEnter(g, 4) == 0 {
        ext::Mex_SpecialNAirDispatch(kind as i32, g, v);
    }
    fp(g).set_x2227_b5(1);
    true
}
