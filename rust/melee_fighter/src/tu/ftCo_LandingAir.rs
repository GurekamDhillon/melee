#![allow(non_snake_case, non_upper_case_globals)]
//! Native port of src/melee/ft/kinds/ftCommon/ftCo_LandingAir.c (aerial landing lag, L-cancel).
//! Every function of the TU; calls out go through `ext`. No fused site. Checked against the C
//! (whole TU, shipping contraction, saturating float->int) by tests/tu_ftCo_LandingAir.rs.

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
        // TARGET_PC skill telemetry (pc/gameworld/script_skill.h), read-only.
        ScriptGame_SkillLanding(fp: &mut Fighter, mode: i32, lag: f32, lag_final: f32) = |_, _, _, _| todo!("TODO(pc script) ScriptGame_SkillLanding");
        ftCommon_8007D7FC(fp: &mut Fighter) = crate::ftcommon3::ftcommon_8007d7fc;
        Fighter_ChangeMotionState(g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G) = |g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G| { unsafe { let _ = crate::tu::fighter::Fighter_ChangeMotionState(g as _, msid as _, flags as _, start as _, speed as _, blend as _, other as _); } };
        ftAnim_SetAnimRate(g: G, rate: f32) = |g, r| unsafe { crate::tu::ftanim::ftAnim_SetAnimRate(g, r) };
        ftAnim_8006F484(g: G) -> f32 = |g| unsafe { crate::tu::ftanim::ftAnim_8006F484(g) };
        ftCo_Landing_Enter_Basic(g: G) = crate::tu::ftCo_Landing::ftCo_Landing_Enter_Basic;
        ftCo_Landing_Anim(g: G) = crate::tu::ftCo_Landing::ftCo_Landing_Anim;
        ftCo_Landing_Phys(g: G) = crate::tu::ftCo_Landing::ftCo_Landing_Phys;
        ftCo_Landing_Coll(g: G) = crate::tu::ftCo_Landing::ftCo_Landing_Coll;
    }
}

pub fn ftCo_LandingAir_EnterWithLag(g: G) {
    melee_core::ct!("ftCo_LandingAir_EnterWithLag");
    // `lag` is uninitialised in the C when the motion matches no case (then msid stays None and
    // the value only reaches the telemetry call); 0 here.
    let mut lag: f32 = 0.0;
    let mut msid: i32 = ftCo_MS_None as i32;
    let mut skill_lag0: f32 = 0.0;
    let mut skill_mode: i32 = -1;
    let f = fp(g);
    if f.cmd_vars[0] != 0 {
        let ca = &f.co_attrs;
        match f.motion_id as i32 {
            x if x == ftCo_MS_AttackAirN as i32 => { lag = ca.landingairn_lag; msid = ftCo_MS_LandingAirN as i32; }
            x if x == ftCo_MS_AttackAirF as i32 => { lag = ca.landingairf_lag; msid = ftCo_MS_LandingAirF as i32; }
            x if x == ftCo_MS_AttackAirB as i32 => { lag = ca.landingairb_lag; msid = ftCo_MS_LandingAirB as i32; }
            x if x == ftCo_MS_AttackAirHi as i32 => { lag = ca.landingairhi_lag; msid = ftCo_MS_LandingAirHi as i32; }
            x if x == ftCo_MS_AttackAirLw as i32 => { msid = ftCo_MS_LandingAirLw as i32; lag = ca.landingairlw_lag; }
            _ => {}
        }
        skill_lag0 = lag;
        skill_mode = 0;
        let cdat = crate::hooks::common_data();
        if msid != ftCo_MS_None as i32 && (f.x67F as i32) < cdat.xE4 {
            skill_mode = 1;
            let div_lag = lag / cdat.xE8;
            let mut int_lag = div_lag as i32;
            if div_lag as i32 == 0 {
                int_lag = 1;
            }
            lag = int_lag as f32;
        }
    }
    let none = msid == ftCo_MS_None as i32;
    ext::ScriptGame_SkillLanding(fp(g), if !none { skill_mode } else { -1 }, skill_lag0, if !none { lag } else { 0.0 });
    if !none {
        ftCo_LandingAir_EnterWithMsidLag(g, msid, lag);
    } else {
        ext::ftCo_Landing_Enter_Basic(g);
    }
}

pub fn ftCo_LandingAir_EnterWithMsidLag(g: G, msid: i32, lag: f32) {
    melee_core::ct!("ftCo_LandingAir_EnterWithMsidLag");
    ext::ftCommon_8007D7FC(fp(g));
    ext::Fighter_ChangeMotionState(g, msid, Ft_MF_None as u32, 0.0, 1.0, 0.0, core::ptr::null_mut());
    let r = (ext::ftAnim_8006F484(g) + 0.1f32) / lag;
    ext::ftAnim_SetAnimRate(g, r);
}

pub fn ftCo_LandingAir_Anim(g: G) {
    melee_core::ct!("ftCo_LandingAir_Anim");
    ext::ftCo_Landing_Anim(g);
}

pub fn ftCo_LandingAir_IASA(_g: G) { melee_core::ct!("ftCo_LandingAir_IASA");}

pub fn ftCo_LandingAir_Phys(g: G) {
    melee_core::ct!("ftCo_LandingAir_Phys");
    ext::ftCo_Landing_Phys(g);
}

pub fn ftCo_LandingAir_Coll(g: G) {
    melee_core::ct!("ftCo_LandingAir_Coll");
    ext::ftCo_Landing_Coll(g);
}
