#![allow(non_snake_case, unused_unsafe)]
//! Native port of src/melee/ft/kinds/ftCommon/ftCo_FallSpecial.c (helpless fall). Every function
//! of the TU (the static inline `inline0` too; the game's `bool` is `int`, kept as i32); calls out of it go through `ext`. No fused float
//! sites (each `drift` term is its own statement). NaN-sign caveat: `drift += +-base` with both
//! operands NaN keeps a different NaN's sign than the C build (x86 operand order); the oracle
//! compares those arguments as any-NaN. The PC-only `Geno_FallSpecialColl` hook in
//! ftCo_FallSpecial_Coll is kept. Checked against the C by tests/tu_ftCo_FallSpecial.rs.
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
        ftCo_80090780(g: G) = |g: G| { unsafe { crate::tu::ftCo_DamageFall::ftCo_80090780(g as _); } };
        Fighter_ChangeMotionState(g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G) = |g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G| { unsafe { crate::tu::fighter::Fighter_ChangeMotionState(g as _, msid as _, flags as _, start as _, speed as _, blend as _, other as _); } };
        ftCommon_8007D60C(fp: &mut Fighter) = crate::ftcommon2::ftcommon_8007d60c;
        ftCommon_UseAllJumps(fp: &mut Fighter) = crate::ftcommon2::use_all_jumps;
        un_80322598(spawn: i32, y: f32) -> i32 = |spawn: i32, y: f32| -> i32 { unsafe { crate::tu::crowdsfx::un_80322598(spawn as _, y as _) as _ } };
        ftCo_Fall_Anim_Inner(g: G, mv_x4: *mut f32, n: i32, f: i32, b: i32) = crate::tu::ftCo_Fall::ftCo_Fall_Anim_Inner;
        ftCo_800CC988(g: G, a: f32) = crate::tu::ftCo_Fall::ftCo_800CC988;
        ftCo_800CEE70(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftCo_ItemParasolOpen::ftCo_800CEE70(g as _) as _ } };
        ftCo_800D705C(g: G) -> bool = crate::tu::ftCo_Attack100::ftCo_800D705C;
        ftCo_800CB870(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftCo_JumpAerial::ftCo_800CB870(g as _) as _ } };
        ftCommon_CheckFallFast(fp: &mut Fighter) -> bool = crate::ftcommon::check_fall_fast;
        ftCommon_FallFast(fp: &mut Fighter) = crate::ftcommon::fall_fast;
        ftCommon_Fall(fp: &mut Fighter, gravity: f32, terminal: f32) = crate::ftcommon::fall;
        ftCommon_CalcSelfAccel_AccelToVelClamped(fp: &mut Fighter, accel: f32, target: f32, friction: f32) = crate::ftcommon::calc_self_accel_accel_to_vel_clamped;
        Geno_FallSpecialColl(g: G) -> i32 = |_| todo!("TODO(pc) Geno_FallSpecialColl (pc geno)");
        ft_80083090(g: G, a: unsafe extern "C" fn(G, isize) -> bool, b: HSD_GObjEvent) = |g: G, a: unsafe extern "C" fn(G, isize) -> bool, b: HSD_GObjEvent| { unsafe { let _ = crate::bridge::with_coll_ev(a, b, |a, b| crate::tu::ft_081b::ft_80083090(g as _, a, b)); } };
        mpLineGetFlags(line: i32) -> u32 = |line: i32| -> u32 { unsafe { melee_world::tu::mplib::mpLineGetFlags(line as _) } };
        ftCo_800D0EC8(fp: &Fighter) -> f32 = crate::tu::ftchangeparam::ftco_800d0ec8;
        ftCo_LandingFallSpecial_Enter(g: G, allow_interrupt: i32, landing_lag: f32) = |g: G, allow_interrupt: i32, landing_lag: f32| { unsafe { crate::tu::ftCo_Landing::ftCo_LandingFallSpecial_Enter(g as _, (allow_interrupt != 0 as _), landing_lag as _); } };
        ft_8008A2BC(g: G) = crate::tu::ft_08a1::ft_8008A2BC;
    }
}

pub fn ftCo_800968C8(g: G) {
    melee_core::ct!("ftCo_800968C8");
    let l = fp(g).x2EC;
    ftCo_80096900(g, 1, 0, 1, 1.0, l);
}

/// static inline `inline0`.
#[allow(clippy::too_many_arguments)]
pub fn inline0(g: G, arg1: i32, arg2: i32, allow_interrupt: i32, mobility: f32, landing_lag: f32, arg6: f32, unk: bool) {
    melee_core::ct!("inline0");
    if fp(g).stamina_dead() != 0 {
        ext::ftCo_80090780(g);
        return;
    }
    ext::Fighter_ChangeMotionState(g, ftCo_MS_FallSpecial, Ft_MF_KeepFastFall, 0.0, 1.0, arg6, core::ptr::null_mut());
    let f = fp(g);
    unsafe {
        let m = &mut f.mv.co.fallspecial;
        m.mobility = f.co_attrs.air_drift_max * mobility;
        m.xC = arg1;
        m.x10 = arg2;
        m.landing_lag = landing_lag;
        m.allow_interrupt = allow_interrupt;
        m.x0 = 26;
        m.x4 = 0.0;
    }
    if f.ground_or_air == GA_Ground {
        ext::ftCommon_8007D60C(f);
    } else if unk {
        ext::ftCommon_UseAllJumps(f);
    }
    let f = fp(g);
    ext::un_80322598(f.x8_spawnNum, f.cur_pos.y);
}

pub fn ftCo_80096900(g: G, arg1: i32, arg2: i32, allow_interrupt: i32, mobility: f32, landing_lag: f32) {
    melee_core::ct!("ftCo_80096900");
    inline0(g, arg1, arg2, allow_interrupt, mobility, landing_lag, 0.0, true);
}

pub fn ftCo_800969D8(g: G, arg1: i32, arg2: i32, allow_interrupt: i32, mobility: f32, landing_lag: f32, arg6: f32) {
    melee_core::ct!("ftCo_800969D8");
    inline0(g, arg1, arg2, allow_interrupt, mobility, landing_lag, arg6, false);
}

pub fn ftCo_FallSpecial_Anim(g: G) {
    melee_core::ct!("ftCo_FallSpecial_Anim");
    let f = fp(g);
    let p = unsafe { core::ptr::addr_of_mut!(f.mv.co.fallspecial.x4) };
    ext::ftCo_Fall_Anim_Inner(g, p, ftCo_SM_FallSpecial, ftCo_SM_FallSpecialF, ftCo_SM_FallSpecialB);
    ext::ftCo_800CC988(g, unsafe { fp(g).mv.co.fallspecial.x4 });
}

pub fn ftCo_FallSpecial_IASA(g: G) {
    melee_core::ct!("ftCo_FallSpecial_IASA");
    if ext::ftCo_800CEE70(g) {
        return;
    }
    if ext::ftCo_800D705C(g) {
        return;
    }
    ext::ftCo_800CB870(g);
}

pub fn ftCo_FallSpecial_Phys(g: G) {
    melee_core::ct!("ftCo_FallSpecial_Phys");
    ext::ftCommon_CheckFallFast(fp(g));
    let f = fp(g);
    let ca = f.co_attrs;
    if unsafe { f.mv.co.fallspecial.xC } != 0 {
        if f.fall_fast() != 0 {
            ext::ftCommon_FallFast(f);
        } else {
            ext::ftCommon_Fall(f, ca.gravity, ca.terminal_velocity);
        }
        let f = fp(g);
        let lstick_x = f.input.lstick[0].x;
        let mut drift = lstick_x * ca.air_drift_stick_mul;
        drift += if lstick_x > 0.0 { ca.aerial_drift_base } else { -ca.aerial_drift_base };
        let target_vel = lstick_x * ca.air_drift_max;
        ext::ftCommon_CalcSelfAccel_AccelToVelClamped(f, drift, target_vel, ca.aerial_friction);
    } else {
        if f.fall_fast() != 0 {
            ext::ftCommon_FallFast(f);
        } else {
            ext::ftCommon_Fall(f, ca.gravity, ca.fast_fall_velocity);
        }
        let f = fp(g);
        let lstick_x = f.input.lstick[0].x;
        let mut drift = lstick_x * ca.air_drift_stick_mul;
        drift += if lstick_x > 0.0 { ca.aerial_drift_base } else { -ca.aerial_drift_base };
        let mut target_vel = lstick_x * ca.air_drift_max;
        let mob = unsafe { f.mv.co.fallspecial.mobility };
        if abs_c(target_vel) > mob {
            target_vel = if target_vel < 0.0 { -mob } else { mob };
        }
        ext::ftCommon_CalcSelfAccel_AccelToVelClamped(f, drift, target_vel, ca.aerial_friction);
    }
}

pub fn ftCo_FallSpecial_Coll(g: G) {
    melee_core::ct!("ftCo_FallSpecial_Coll");
    if ext::Geno_FallSpecialColl(g) != 0 {
        return;
    }
    ext::ft_80083090(g, ftCo_80096CC8, Some(ftCo_80096D28));
}

/// Passed as a callback (`bool (*)(Fighter_GObj*, int)`), hence `extern "C"`.
pub extern "C" fn ftCo_80096CC8(g: G, line_id: isize) -> bool {
    melee_core::ct!("ftCo_80096CC8");
    let f = fp(g);
    line_id != -1
        && (ext::mpLineGetFlags(line_id as i32) & LINE_FLAG_PLATFORM as u32 == 0 || f.input.lstick[0].y > cd().x25C)
}

/// Passed as a callback (`HSD_GObjEvent`), hence `extern "C"`.
pub extern "C" fn ftCo_80096D28(g: G) {
    melee_core::ct!("ftCo_80096D28");
    let f = fp(g);
    let (x10, ai, ll) = unsafe { (f.mv.co.fallspecial.x10, f.mv.co.fallspecial.allow_interrupt, f.mv.co.fallspecial.landing_lag) };
    if x10 != 0 || f.self_vel.y < ext::ftCo_800D0EC8(f) {
        ext::ftCo_LandingFallSpecial_Enter(g, ai, ll);
    } else {
        ext::ft_8008A2BC(g);
    }
}
