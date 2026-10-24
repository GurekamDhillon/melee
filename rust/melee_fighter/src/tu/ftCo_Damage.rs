#![allow(non_snake_case, unused_unsafe, non_upper_case_globals)]
//! Native port of src/melee/ft/kinds/ftCommon/ftCo_Damage.c (taking a hit: knockback, hitlag, SDI,
//! the Damage / DamageFly / DamageFlyRoll states). Every function of the TU, TARGET_PC branches
//! included; calls out of the TU go through `ext` (`ext::hook::<name>` overrides; unset = the
//! port, or a TODO panic).
//!
//! Fused float sites (shipping `-ffp-contract=on`, `melee_core::fp::fmadds`): `x*x + y*y` in
//! `calcAngle`, `ftCo_8008E5A4`, `ftCo_Damage_OnExitHitlag`, the VEC2/VEC3_SQ_LEN users,
//! `ftCo_Damage_CalcAngle`'s `x148 * q + 1`, `angle += scale * f30`, and `ftCo_Ucf084Sdi`.
//!
//! `ftCo_8008DA4C`: C returns an uninitialised `result` when `dmg.x1838_percentTemp == 0`; the
//! port returns false there (callers in `ftCo_8008DCE0` ignore it). `ftCo_8008D8E8` is declared
//! `bool` in C, so its 0..3 level collapses to 0/1.
//! Checked against the C (whole TU, shipping contraction) by tests/tu_ftCo_Damage.rs.
use crate::types::*;
use melee_core::fp::fmadds;
use melee_runtime::gekko::{cosf, sinf, sqrtf};

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
/// `MTXDegToRad(a)` is the macro `(a) * 0.01745329252f`.
#[inline(always)]
fn deg_to_rad(a: f32) -> f32 {
    a * 0.01745329252f32
}
/// C `(int) float` in the shipping build: `llvm.fptosi.sat` (gwtool `makeArithmeticSafe`), i.e. Rust `as`.
#[inline(always)]
fn f2i(x: f32) -> i32 {
    x as i32
}
#[inline(always)]
fn atan2f(y: f32, x: f32) -> f32 {
    ext::atan2f(y, x)
}
/// `M_PI_2_F`: `3.14159265358979323846F / 2.0F`.
const M_PI_2_F: f32 = core::f32::consts::FRAC_PI_2;
/// `M_PI_2` (MSL double).
const M_PI_2: f64 = core::f64::consts::FRAC_PI_2;
#[inline(always)]
fn cv(v: &Vec3) -> melee_core::vec::Vec3 {
    melee_core::vec::Vec3 { x: v.x, y: v.y, z: v.z }
}
/// `VEC2_SQ_LEN`: `x*x + y*y` (fused).
#[inline(always)]
fn sq_len2(x: f32, y: f32) -> f32 {
    fmadds(x, x, y * y)
}
/// `VEC3_SQ_LEN`: `(x*x + y*y) + z*z` (both adds fused).
#[inline(always)]
fn sq_len3(v: &Vec3) -> f32 {
    fmadds(v.z, v.z, fmadds(v.x, v.x, v.y * v.y))
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
        ScriptGame_ArmorResetReaction(fp: &mut Fighter) = |_| todo!("TODO(pc gameworld) ScriptGame_ArmorResetReaction");
        ScriptGame_ArmorSubtract(fp: &mut Fighter, kb: f32) -> f32 = |_, _| todo!("TODO(pc gameworld) ScriptGame_ArmorSubtract");
        ScriptGame_ArmorReact(fp: &mut Fighter) -> i32 = |_| todo!("TODO(pc gameworld) ScriptGame_ArmorReact");
        ScriptGame_CapsNoFlinch(fp: &mut Fighter) -> i32 = |_| todo!("TODO(pc gameworld) ScriptGame_CapsNoFlinch");
        ScriptGame_ShockTake(fp: &mut Fighter, base: i32) -> i32 = |_, _| todo!("TODO(pc gameworld) ScriptGame_ShockTake");
        Geno_TakeStunBonus(fp: &mut Fighter) -> i32 = |_| todo!("TODO(pc geno) Geno_TakeStunBonus");
        diag_geno_stun(port: i32, base: i32, bonus: i32) = |_, _, _| todo!("TODO(pc geno) diag_geno_stun");
        Replay_UcfVersion(port: i32) -> i32 = |_| todo!("TODO(pc replay) Replay_UcfVersion");
        ftUcf_PadRaw(port: i32, which_y: i32, back: i32) -> i32 = crate::fighter::ftucf_pad_raw;
        Mex_Enabled(name: &str) -> i32 = |_| todo!("TODO(pc mex) Mex_Enabled");
        ftColl_8007AC68(kb_angle: u32) -> bool = |_| todo!("TODO(port) ftColl_8007AC68 (ftcoll.c)");
        ftCo_CalcYScaledKnockback(a: f32, scale: f32, b: f32) -> f32 = crate::tu::ftchangeparam::ftco_calc_y_scaled_knockback;
        Fighter_804D6524_x0() -> f32 = || crate::tu::ftchangeparam::ext::d6524().x0;
        ftCo_800BFFD0(fp: &mut Fighter, a: i32, b: i32) -> bool = crate::tu::ftcolanim::ftco_800bffd0;
        lbBgFlash_80021C48(a: u32, b: u32) = |_, _| todo!("TODO(group1 lb) lbBgFlash_80021C48");
        efAsync_Spawn(g: G, q: *mut core::ffi::c_void, kind: u32, gfx: u32, jobj: *mut HSD_JObj, angle: *mut f32) = |_, _, _, _, _, _| todo!("TODO(group2 efasync) efAsync_Spawn");
        ftPartSetRotX(fp: &Fighter, idx: i32, v: f32) = crate::tu::ftparts::ftPartSetRotX;
        ftParts_GetBoneIndex(fp: &Fighter, part: i32) -> i32 = crate::ftdmgfall::ftparts_get_bone_index;
        Fighter_8006CDA4(fp: &mut Fighter, v: i32) = |fp: &mut Fighter, v: i32| { unsafe { let _ = crate::tu::fighter::Fighter_8006CDA4(fp as _, v as _); } };
        pl_80040270(port: i32, sub: i32, kb: f32) = crate::plbonus::pl_80040270;
        pl_800401F0(port: i32, sub: i32, x: f32, y: f32) = crate::plbonus::pl_800401f0;
        ftCommon_8007D5D4(fp: &mut Fighter) = crate::ftcommon2::ftcommon_8007d5d4;
        HSD_Randf() -> f32 = || todo!("TODO(group1 random) HSD_Randf");
        un_803222EC(mag: f32, angle: f32) -> f32 = |mag: f32, angle: f32| -> f32 { unsafe { crate::tu::crowdsfx::un_803222EC(mag as _, angle as _) as _ } };
        un_8032233C(a: u32, b: u32) = |a: u32, b: u32| { unsafe { let _ = crate::tu::crowdsfx::un_8032233C(a as _, b as _); } };
        Fighter_ChangeMotionState(g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G) = |g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G| { unsafe { let _ = crate::tu::fighter::Fighter_ChangeMotionState(g as _, msid as _, flags as _, start as _, speed as _, blend as _, other as _); } };
        ftAnim_8006EBA4(g: G) = |g: G| { unsafe { let _ = crate::tu::ftanim::ftAnim_8006EBA4(g as _); } };
        gm_8016B014() -> bool = || todo!("TODO(port) gm_8016B014 (gmvs.c)");
        ftColl_8007B6A0(g: G) = |_| todo!("TODO(port) ftColl_8007B6A0 (ftcoll.c)");
        ftCommon_8007EFC0(fp: &mut Fighter, v: u32) = crate::ftcommon2::ftcommon_8007efc0;
        Camera_RequestQuake(kind: u32, pos: &Vec3) = |_, _| todo!("TODO(group2 camera) Camera_RequestQuake");
        ftCommon_8007F824(g: G) = crate::ftcommon3::ftcommon_8007f824;
        ftCo_DamageIce_Init(g: G) = crate::tu::ftCo_DamageIce::ftCo_DamageIce_Init;
        ftCo_800DF608(fp: &Fighter) -> bool = crate::ft0df::ftco_800df608;
        ftColl_8007B7A4(g: G, v: i32) = |_, _| todo!("TODO(port) ftColl_8007B7A4 (ftcoll.c)");
        ftCo_800C318C(g: G, v: bool) = crate::tu::ftCo_DamageSong::ftCo_800C318C;
        ftCo_800C8D00(g: G) = crate::tu::ft_0c8c::ftco_800c8d00;
        ftCo_800DC284(g: G) = crate::tu::ftco_0dc2::ftco_800dc284;
        ftCo_800DC3A4(g: G) = crate::tu::ftco_0dc2::ftco_800dc3a4;
        ftCo_800C0408(g: G) = crate::tu::ftcolanim::ftco_800c0408;
        ftCommon_800804FC(fp: &mut Fighter) = crate::ftcommon2::ftcommon_800804fc;
        ftCo_800C3538(g: G) -> bool = crate::tu::ftCo_DamageSong::ftCo_800C3538;
        ftCo_800C44CC(g: G) -> bool = crate::tu::ftCo_DamageBind::ftCo_800C44CC;
        ftCo_800D2FA4(g: G) -> bool = crate::tu::ftCo_DamageScrew::ftCo_800D2FA4;
        ftCo_800DE854(g: G) = crate::tu::ftCo_Thrown::ftCo_800DE854;
        ftCo_800DCE34(a: G, b: G) = crate::tu::ftCo_CaptureCut::ftCo_800DCE34;
        ftCommon_8007DB58(g: G) = crate::ftcommon3::ftcommon_8007db58;
        ftCommon_CalcHitlag(dmg: i32, msid: i32, mul: f32) -> f32 = crate::ftcommon::calc_hitlag;
        ftCo_800DE2F0(g: G) = |g: G| { unsafe { let _ = crate::tu::ftCo_Throw::ftCo_800DE2F0(g as _); } };
        ftCo_800DCFD4(g: G) = crate::tu::ftCo_CaptureCut::ftCo_800DCFD4;
        it_8026B2D8(g: *mut HSD_GObj) -> bool = |g: *mut HSD_GObj| -> bool { unsafe { melee_items::tu::it_26B1::it_8026B2D8(g as _) as _ } };
        ftDk_MS_349_800E06D8(g: G) = |_| todo!("TODO(port) ftDk_MS_349_800E06D8 (ftdonkeyheavylanding.c)");
        ftCo_800C0CB8(g: G) -> bool = crate::tu::ftCo_Bury::ftCo_800C0CB8;
        ftCo_DamageIce_HitWhileFrozen(g: G) = crate::tu::ftCo_DamageIce::ftCo_DamageIce_HitWhileFrozen;
        ftCo_800C74F4(g: G) -> bool = crate::tu::ftco_800c7434::ftco_800c74f4;
        ftCo_800C3598(g: G) = crate::tu::ftco_0c35::ftco_800c3598;
        ftCommon_8007F86C(g: G) = crate::ftcommon3::ftcommon_8007f86c;
        ftCo_800C5240(g: G) -> bool = crate::tu::ftCo_HammerWait::ftCo_800C5240;
        ftCo_800C554C(fp: &mut Fighter) = crate::tu::ftCo_HammerWait::ftCo_800C554C;
        ftCo_800CB870(g: G) -> bool = crate::tu::ftCo_JumpAerial::ftCo_800CB870;
        ftAnim_IsFramesRemaining(g: G) -> bool = crate::hooks::ftanim_is_frames_remaining;
        ftCo_80090780(g: G) = crate::tu::ftCo_DamageFall::ftCo_80090780;
        ftCo_Fall_Enter(g: G) = crate::tu::ftCo_Fall::ftCo_Fall_Enter;
        ft_8008A2BC(g: G) = crate::tu::ft_08a1::ft_8008A2BC;
        ftCo_800D69C4(g: G) -> bool = crate::tu::ftCo_Attack100::ftCo_800D69C4;
        ftCo_800CB8E0(g: G) -> bool = crate::tu::ftCo_JumpAerial::ftCo_800CB8E0;
        ftCommon_8007EBAC(fp: &mut Fighter, a: u32, b: u32) = crate::ftcommon2::ftcommon_8007ebac;
        ftCo_Jump_GetInput(g: G) -> u32 = |g| crate::tu::ftCo_Jump::ftCo_Jump_GetInput(g) as u32;
        ftCo_HammerFall_IASA(g: G) = crate::tu::ftCo_HammerFall::ftCo_HammerFall_IASA;
        ftCo_HammerWait_IASA(g: G) = crate::tu::ftCo_HammerWait::ftCo_HammerWait_IASA;
        ftCo_Fall_IASA_Inner(g: G) -> bool = crate::tu::ftCo_Fall::ftCo_Fall_IASA_Inner;
        ftCo_Wait_IASA(g: G) = crate::tu::ftCo_Wait::ftCo_Wait_IASA;
        ft_80084DB0(g: G) = crate::ft084e::ft_80084db0;
        ft_80084EEC(g: G) = crate::ft084e::ft_80084eec;
        ft_80084F3C(g: G) = crate::ft084e::ft_80084f3c;
        ft_800848DC(g: G, cb: fn(G)) = crate::tu::ft_081b::ft_800848dc;
        ft_80081DD4(g: G) -> bool = crate::tu::ft_081b::ft_80081dd4;
        ftCo_80097D40(g: G) = crate::tu::ftCo_DownBound::ftCo_80097D40;
        ftCo_Landing_Enter_Basic(g: G) = crate::tu::ftCo_Landing::ftCo_Landing_Enter_Basic;
        ftCommon_8007D7FC(fp: &mut Fighter) = crate::ftcommon3::ftcommon_8007d7fc;
        ftColl_8007AFF8(g: G) = |_| todo!("TODO(port) ftColl_8007AFF8 (ftcoll.c)");
        ftCo_800C1D38(g: G) -> bool = crate::tu::ftCo_PassiveWall::ftCo_800C1D38;
        ftCo_800C23A0(g: G) -> bool = crate::tu::ftCo_PassiveCeil::ftCo_800C23A0;
        ftCo_800C17CC(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftCo_FlyReflect::ftCo_800C17CC(g as _) as _ } };
        ftCo_80098928(g: G) -> bool = crate::tu::ftCo_PassiveStand::ftCo_80098928;
        ftCo_8009872C(g: G) -> bool = crate::tu::ftCo_DownAttack::ftCo_8009872C;
        ftCo_8009F0F0(g: G) -> bool = crate::tu::ftCo_DownDamage::ftCo_8009F0F0;
        ftCo_DamageFall_IASA(g: G) = crate::tu::ftCo_DamageFall::ftCo_DamageFall_IASA;
        atan2f(y: f32, x: f32) -> f32 = melee_runtime::lbtrigf::atan2f;
        PSVECCrossProduct(a: &Vec3, b: &Vec3) -> Vec3 = |a, b| { let r = melee_runtime::mtx::vec_cross(&cv(a), &cv(b)); Vec3 { x: r.x, y: r.y, z: r.z } };
        lbVector_Angle(a: &Vec3, b: &Vec3) -> f32 = |a, b| melee_runtime::lbvector::angle(&cv(a), &cv(b));
    }
}

/// `ftCo_803C5520[2][4][3]`: motion ids by [air][knockback level][hurtbox].
pub static ftCo_803C5520: [[[i32; 3]; 4]; 2] = [
    [[81, 78, 75], [82, 79, 76], [83, 80, 77], [89, 88, 87]],
    [[84, 84, 84], [85, 85, 85], [86, 86, 86], [89, 88, 87]],
];

pub fn ftCo_Damage_CalcAngle(f: &mut Fighter, kb: f32) -> f32 {
    let c = cd();
    if f.dmg.x1848_kb_angle != 361 {
        if ext::ftColl_8007AC68(f.dmg.x1848_kb_angle as u32) {
            unsafe {
                f.mv.co.damage.x1A = 1;
                f.mv.co.damage.x1B = c.x7F0 as u8;
            }
        }
        return deg_to_rad(f.dmg.x1848_kb_angle as f32);
    }
    if f.ground_or_air == GA_Air {
        return c.x144_radians;
    } else if kb < c.x14C {
        return 0.0;
    }
    let mut result = deg_to_rad(fmadds(c.x148, (kb - c.x14C) / (c.x150 - c.x14C), 1.0));
    if result > deg_to_rad(c.x148) {
        result = deg_to_rad(c.x148);
    }
    result
}

pub fn ftCo_ScaleBy154(mul: f32) -> f32 {
    mul * cd().x154
}

/// C declares `bool` but this build keeps the 0..3 level.
pub fn ftCo_8008D8E8(v: f32) -> i32 {
    let c = cd();
    let lvl = if v < c.x158 {
        0
    } else if v < c.x15C {
        1
    } else if v < c.x160 {
        2
    } else {
        3
    };
    lvl
}

pub fn ftCo_Damage_CalcKnockback(f: &mut Fighter) {
    let c = cd();
    ext::ScriptGame_ArmorResetReaction(f);
    if f.dmg.kb_applied == 0.0 {
        return;
    }
    let m = f.motion_id as i32;
    if m == ftCo_MS_Squat || m == ftCo_MS_SquatWait {
        f.dmg.kb_applied *= c.kb_squat_mul;
    }
    if f.motion_id as i32 == ftCo_MS_DamageIce {
        f.dmg.kb_applied *= c.kb_ice_mul;
    }
    if f.smash_attrs.state == SmashState_Charging {
        f.dmg.kb_applied *= c.kb_smashcharge_mul;
    }
    if f.x34_scale.y != 1.0 {
        f.dmg.kb_applied = ext::ftCo_CalcYScaledKnockback(f.dmg.kb_applied, f.x34_scale.y, ext::Fighter_804D6524_x0());
    }
    let mut armor = if f.dmg.armor0 > f.dmg.armor1 { f.dmg.armor0 } else { f.dmg.armor1 };
    if f.is_metal() != 0 {
        armor += c.metal_armor;
    }
    f.dmg.kb_applied -= armor;
    f.dmg.kb_applied = ext::ScriptGame_ArmorSubtract(f, f.dmg.kb_applied);
    if f.dmg.kb_applied < c.kb_min {
        f.dmg.kb_applied = c.kb_min;
    }
}

/// static `ftCo_8008DA4C` (see the module note on the uninitialised `result`).
pub fn ftCo_8008DA4C(g: G, element: u32, arg2: i32) -> bool {
    let f = fp(g);
    if f.dmg.x1838_percentTemp != 0.0 {
        match element {
            HitElement_Fire => ext::ftCo_800BFFD0(f, arg2 + 11, 0),
            HitElement_Electric => ext::ftCo_800BFFD0(f, arg2 + 15, 0),
            HitElement_Ice => ext::ftCo_800BFFD0(f, arg2 + 31, 0),
            HitElement_Dark => ext::ftCo_800BFFD0(f, arg2 + 35, 0),
            _ => ext::ftCo_800BFFD0(f, 4, 0),
        }
    } else {
        false
    }
}

pub fn ftCo_8008DB10(g: G, element: i32, v: f32) {
    let c = cd();
    if fp(g).dmg.x1838_percentTemp == 0.0 {
        return;
    }
    match element as u32 {
        HitElement_Fire => {
            if v > c.x17C {
                ext::lbBgFlash_80021C48(3, 0);
            }
        }
        HitElement_Electric => {
            if v > c.x180 {
                ext::lbBgFlash_80021C48(4, 0);
            }
        }
        HitElement_Ice => {
            if v > c.x184 {
                ext::lbBgFlash_80021C48(5, 0);
            }
        }
        HitElement_Dark => {
            if v > c.x188 {
                ext::lbBgFlash_80021C48(6, 0);
            }
        }
        _ => {
            if v > c.x178 {
                ext::lbBgFlash_80021C48(2, 0);
            }
        }
    }
}

pub fn ftCo_Damage_CalcVel(f: &mut Fighter, x: f32, y: f32) {
    if f.dmg.x18ac_time_since_hit < cd().xFC {
        f.x8c_kb_vel.x = x;
        f.x8c_kb_vel.y = y;
    } else {
        let cur_x = f.x8c_kb_vel.x;
        let cur_y = f.x8c_kb_vel.y;
        if cur_x * x < 0.0 {
            f.x8c_kb_vel.x = cur_x + x;
        } else if abs_c(x) > abs_c(cur_x) {
            f.x8c_kb_vel.x = x;
        }
        if cur_y * y < 0.0 {
            f.x8c_kb_vel.y = cur_y + y;
        } else if abs_c(y) > abs_c(cur_y) {
            f.x8c_kb_vel.y = y;
        }
    }
}

/// static `calcAngle`.
pub fn calcAngle(angle: f32) -> f32 {
    let cm = cosf(M_PI_2_F);
    let ca = cosf(angle);
    let sm = M_PI_2_F;
    let x = ca + cm;
    let y = sinf(angle) + sinf(sm);
    if sq_len2(x, y) <= 0.0001f32 {
        return 0.0;
    }
    atan2f(y, x)
}

/// static `inlineA0`.
fn inlineA0(g: G, f: &mut Fighter, angle: *mut f32) {
    let q = core::ptr::addr_of_mut!(fp(g).x60C) as *mut core::ffi::c_void;
    let j = unsafe { (*f.parts.add(FtPart_TopN as usize)).joint };
    ext::efAsync_Spawn(g, q, 4, 0x406, j, angle);
}

/// static `inlineA1`.
fn inlineA1(g: G) {
    let f = fp(g);
    let bone = ext::ftParts_GetBoneIndex(f, FtPart_XRotN as i32);
    let r = f.facing_dir * atan2f(f.self_vel.x + f.x8c_kb_vel.x, f.self_vel.y + f.x8c_kb_vel.y);
    ext::ftPartSetRotX(f, bone, r);
}

pub fn ftCo_8008DCE0(g: G, arg1: i32, facing_dir: f32) {
    let f = fp(g);
    let c = cd();
    let mut var_r27 = 1;
    let kb_applied = f.dmg.kb_applied;
    let mut msid: i32;
    ext::Fighter_8006CDA4(f, f2i(f.dmg.x1838_percentTemp));
    f.dmg.x18d8.kb_applied1 = kb_applied;
    ext::pl_80040270(f.player_id as i32, f.is_sub_fighter() as i32, kb_applied);
    let scaled_kb_154 = kb_applied * c.x154;
    unsafe { f.mv.co.damage.x0 = f2i(scaled_kb_154) as f32 };
    if unsafe { f.mv.co.damage.x0 } == 0.0 {
        unsafe { f.mv.co.damage.x0 = 1.0 };
    }
    {
        let bonus = ext::Geno_TakeStunBonus(f);
        ext::diag_geno_stun(f.player_id as i32, f2i(unsafe { f.mv.co.damage.x0 }), bonus);
        unsafe { f.mv.co.damage.x0 += bonus as f32 };
    }
    {
        let base = f2i(unsafe { f.mv.co.damage.x0 });
        let take = ext::ScriptGame_ShockTake(f, base);
        unsafe { f.mv.co.damage.x0 += take as f32 };
    }
    let kb_level_base: i32 = if scaled_kb_154 < c.x158 {
        0
    } else if scaled_kb_154 < c.x15C {
        1
    } else if scaled_kb_154 < c.x160 {
        2
    } else {
        3
    };
    let mut kb_level = kb_level_base;
    if arg1 != -1 {
        kb_level = 3;
    }
    let mut scaled_kb_v = kb_applied * c.x100;
    unsafe {
        f.mv.co.damage.x1A = 0;
        f.mv.co.damage.x14 = 0.0;
    }
    let mut kb_angle = ftCo_Damage_CalcAngle(f, kb_applied);
    if kb_level_base >= 2 && f.dmg.x1860_element == HitElement_Ice {
        kb_angle = calcAngle(kb_angle);
    }
    let mut x = scaled_kb_v * cosf(kb_angle);
    let mut y = scaled_kb_v * sinf(kb_angle);
    f.facing_dir = f.dmg.facing_dir_1;
    let hb = f.dmg.x184c_damaged_hurtbox as usize;
    if f.ground_or_air == GA_Air {
        msid = ftCo_803C5520[1][kb_level as usize][hb];
        if ftCo_Damage_CheckAirMotion(f) {
            scaled_kb_v = scaled_kb_v * c.x190;
            x = scaled_kb_v * cosf(kb_angle);
            y = scaled_kb_v * sinf(kb_angle);
        }
        let fd = f.facing_dir;
        ftCo_Damage_CalcVel(f, -x * fd, y);
        f.xF0_ground_kb_vel = 0.0;
    } else {
        let normal: &Vec3 = unsafe { &*core::ptr::addr_of!(f.coll_data.floor.normal) };
        let pos = Vec3 { x: -x * f.facing_dir, y, z: 0.0 };
        let floor_angle = ext::lbVector_Angle(normal, &pos);
        if floor_angle < M_PI_2_F {
            msid = ftCo_803C5520[0][kb_level as usize][hb];
            ext::ftCommon_8007D5D4(f);
            ftCo_Damage_CalcVel(f, pos.x, pos.y);
            f.xF0_ground_kb_vel = 0.0;
        } else if kb_level == 3 {
            ext::ftCommon_8007D5D4(f);
            msid = ftCo_803C5520[0][kb_level as usize][hb];
            if (floor_angle as f64) > (M_PI_2 + c.x1E8_radians as f64) {
                ftCo_Damage_CalcVel(f, pos.x, -pos.y * c.x1EC);
                var_r27 = 0;
                f.xF0_ground_kb_vel = 0.0;
                let mut sp40 = atan2f(-normal.x, normal.y);
                inlineA0(g, f, &mut sp40);
            } else {
                ftCo_Damage_CalcVel(f, pos.x, pos.y);
                f.xF0_ground_kb_vel = 0.0;
            }
        } else {
            msid = ftCo_803C5520[0][kb_level as usize][hb];
            f.xF0_ground_kb_vel = pos.x;
            let t = f.xF0_ground_kb_vel;
            ftCo_Damage_CalcVel(f, normal.y * t, -normal.x * t);
        }
    }
    f.self_vel.x = 0.0;
    f.self_vel.y = 0.0;
    f.self_vel.z = 0.0;
    f.gr_vel = 0.0;
    if kb_level == 3 && f.ground_or_air == GA_Air {
        if kb_angle > c.x234_radians && kb_angle < c.x238_radians {
            msid = 0x5A;
        } else if f.dmg.x1830_percent >= c.x23C as f32 && ext::HSD_Randf() < c.x240 {
            msid = 0x5B;
        }
    }
    if arg1 != -1 {
        msid = arg1;
    }
    if msid != 0x145 && kb_level_base >= 2 && f.dmg.x1860_element == HitElement_Ice {
        msid = 0x5A;
    }
    f.dmg.x18A4_knockbackMagnitude = ext::un_803222EC(kb_applied, kb_angle);
    ext::un_8032233C(f.dmg.x18C0 as u32, f.x8_spawnNum as u32);
    if facing_dir != 0.0 {
        f.facing_dir = facing_dir;
    }

    ftCo_8008DA4C(g, f.dmg.x1860_element, kb_level);
    ftCo_8008DB10(g, f.dmg.x1860_element as i32, kb_applied);
    ext::Fighter_ChangeMotionState(g, msid, 0x40, 0.0, 1.0, 0.0, core::ptr::null_mut());
    ext::ftAnim_8006EBA4(g);
    if ext::gm_8016B014() {
        ext::ftColl_8007B6A0(g);
        fp(g).set_x2221_b1(1);
    }
    let f = fp(g);
    let should_collide = if kb_applied > c.x12C { 1 } else { 0 };
    unsafe {
        f.mv.co.damage.x4 = should_collide;
        f.mv.co.damage.x19 = 0;
    }
    f.hitlag_cb = Some(ftCo_Damage_OnEveryHitlag_cb);
    f.active_timer.lstick.x = 0xFE;
    f.active_timer.lstick.y = 0xFE;
    f.post_hitlag_cb = Some(ftCo_Damage_OnExitHitlag_cb);
    f.dmg.x18A8 = f.dmg.kb_applied;
    f.set_x221C_b6(1);
    f.dmg.x18ac_time_since_hit = 0;
    if msid == 0x5B {
        inlineA1(g);
    }
    let f = fp(g);
    f.x67F = 0xFF;
    ftCo_Damage_SetMv8FromKbThreshold(f);
    unsafe {
        if f.mv.co.damage.x8 != 0 {
            f.mv.co.damage.x8 = 1;
        }
    }
    if kb_level == 3 && scaled_kb_v >= c.x5E8 {
        ext::ftCommon_8007EFC0(f, c.x5EC as u32);
    }
    if var_r27 != 0 {
        if scaled_kb_154 >= c.x20C {
            f.dmg.x1908 = 0x4F;
            f.dmg.x190C = unsafe { (*(*f.ft_data).x4C_sfx).x20 } as _;
        } else if scaled_kb_154 >= c.x208 {
            f.dmg.x1908 = 0x50;
            f.dmg.x190C = unsafe { (*(*f.ft_data).x4C_sfx).x1C } as _;
        }
    }
    if kb_level == 3 && f.ground_or_air == GA_Air {
        let quake_kind = if scaled_kb_154 >= c.x174 {
            QuakeKind_Large
        } else if scaled_kb_154 >= c.x170 {
            QuakeKind_Medium
        } else {
            QuakeKind_Small
        };
        let pos = f.cur_pos;
        ext::Camera_RequestQuake(quake_kind, &pos);
    }
    ext::ftCommon_8007F824(g);
    let f = fp(g);
    if f.motion_id as i32 == 0x145 {
        return;
    }
    if kb_level_base < 2 {
        return;
    }
    if f.dmg.x1860_element == HitElement_Ice {
        ext::ftCo_DamageIce_Init(g);
    }
}

pub fn ftCo_Damage_CheckAirMotion(f: &Fighter) -> bool {
    let c = cd();
    match f.motion_id as i32 {
        ftCo_MS_JumpF | ftCo_MS_JumpB | ftCo_MS_JumpAerialF | ftCo_MS_JumpAerialB | ftCo_MS_Fall
        | ftCo_MS_FallF | ftCo_MS_FallB | ftCo_MS_FallAerial | ftCo_MS_FallAerialF
        | ftCo_MS_FallAerialB | ftCo_MS_FallSpecial | ftCo_MS_FallSpecialF
        | ftCo_MS_FallSpecialB | ftCo_MS_DamageFall | ftCo_MS_EscapeAir => {
            f.x680 as i32 <= c.x18C && f.x684 as i32 >= c.x1C
        }
        _ => false,
    }
}

/// TARGET_PC static `ftCo_Ucf084Sdi`: UCF 0.84's SDI window.
pub fn ftCo_Ucf084Sdi(f: &Fighter) -> bool {
    let c = cd();
    let port = f.x618_player_id as i32;
    if ext::Replay_UcfVersion(port) != 84 {
        return false;
    }
    if f.active_sticky.lstick.x > 1 && f.active_sticky.lstick.y > 1 {
        return false;
    }
    let yy = f.input.lstick[1].y * f.input.lstick[1].y;
    let prev = fmadds(f.input.lstick[1].x, f.input.lstick[1].x, yy);
    let thr = c.sdi_min_stick_mag * c.sdi_min_stick_mag;
    if !(thr > prev) {
        return false;
    }
    let dx = ext::ftUcf_PadRaw(port, 0, 0).wrapping_sub(ext::ftUcf_PadRaw(port, 0, 2));
    let dy = ext::ftUcf_PadRaw(port, 1, 0).wrapping_sub(ext::ftUcf_PadRaw(port, 1, 2));
    dx.wrapping_mul(dx).wrapping_add(dy.wrapping_mul(dy)) > 3844
}

pub fn ftCo_Damage_OnEveryHitlag(g: G) {
    let f = fp(g);
    let c = cd();
    if f.allow_sdi() != 0
        && sq_len2(f.input.lstick[0].x, f.input.lstick[0].y) >= c.sdi_min_stick_mag * c.sdi_min_stick_mag
        && ((f.active_timer.lstick.x as i32) < c.sdi_stick_window
            || (f.active_timer.lstick.y as i32) < c.sdi_stick_window
            || ftCo_Ucf084Sdi(f))
    {
        let sx = f.input.lstick[0].x * c.sdi_pos_scale;
        let sy = f.input.lstick[0].y * c.sdi_pos_scale;
        f.cur_pos.x += sx;
        f.cur_pos.y += sy;
        f.active_timer.lstick.x = 254;
        f.active_timer.lstick.y = 254;
        ext::pl_800401F0(f.player_id as i32, f.is_sub_fighter() as i32, sx, sy);
    }
}

pub fn ftCo_8008E5A4(f: &mut Fighter) {
    let c = cd();
    if f.input.lstick[0].x != 0.0 || f.input.lstick[0].y != 0.0 {
        let kb_x = f.x8c_kb_vel.x;
        let kb_y = f.x8c_kb_vel.y;
        let kb_vel_x_neg = -kb_x;
        let mut kb_mag = fmadds(kb_vel_x_neg, kb_vel_x_neg, kb_y * kb_y);
        if !(kb_mag < 0.00001f32) {
            let f3 = fmadds(kb_y, f.input.lstick[0].x, kb_vel_x_neg * f.input.lstick[0].y);
            let mut f30 = f3 * f3 / kb_mag;
            let lstick_vec3 = Vec3 { x: f.input.lstick[0].x, y: f.input.lstick[0].y, z: 0.0 };
            let kv = Vec3 { x: f.x8c_kb_vel.x, y: f.x8c_kb_vel.y, z: f.x8c_kb_vel.z };
            let cross = ext::PSVECCrossProduct(&kv, &lstick_vec3);
            if cross.z < 0.0 {
                f30 = -f30;
            }
            let mut angle = atan2f(kb_y, kb_x);
            kb_mag = sqrtf(sq_len2(kb_x, kb_y));
            let scale = deg_to_rad(c.x1A8);
            angle = fmadds(scale, f30, angle);
            f.x8c_kb_vel.x = kb_mag * cosf(angle);
            f.x8c_kb_vel.y = kb_mag * sinf(angle);
        }
    }
}

pub fn ftCo_Damage_OnExitHitlag(g: G) {
    let f = fp(g);
    let c = cd();
    let in_circle = sq_len2(f.input.lstick[0].x, f.input.lstick[0].y) >= c.sdi_min_stick_mag * c.sdi_min_stick_mag;
    if in_circle || ext::ftCo_800DF608(f) {
        let (x, y);
        if ext::ftCo_800DF608(f) {
            let k = c.x4BC;
            x = f.input.cstick[0].x * k;
            y = f.input.cstick[0].y * k;
        } else {
            let k = c.x4BC;
            x = f.input.lstick[0].x * k;
            y = f.input.lstick[0].y * k;
        }
        f.cur_pos.x += x;
        f.cur_pos.y += y;
        ext::pl_800401F0(f.player_id as i32, f.is_sub_fighter() as i32, x, y);
    }
    if unsafe { f.mv.co.damage.x4 } != 0 {
        unsafe { f.mv.co.damage.x4 = 0 };
        ext::ftColl_8007B7A4(g, c.x130);
    }
    ftCo_8008E5A4(fp(g));
    let f = fp(g);
    if f.input.held_buttons[0] & HSD_PAD_LR != 0 {
        let kb_x = f.x8c_kb_vel.x;
        let kb_y = f.x8c_kb_vel.y;
        if kb_x != 0.0 || kb_y != 0.0 {
            let kb_angle = atan2f(kb_y, kb_x);
            let m = sqrtf(sq_len2(kb_x, kb_y)) * c.x1AC;
            f.x8c_kb_vel.x = m * cosf(kb_angle);
            f.x8c_kb_vel.y = m * sinf(kb_angle);
        }
    }
}

// Rust-ABI callbacks stored in the fighter (C: HSD_GObjEvent).
pub unsafe extern "C" fn ftCo_Damage_OnEveryHitlag_cb(g: G) {
    ftCo_Damage_OnEveryHitlag(g)
}
pub unsafe extern "C" fn ftCo_Damage_OnExitHitlag_cb(g: G) {
    ftCo_Damage_OnExitHitlag(g)
}

pub fn ftCo_8008E908(g: G, facing_dir: f32) {
    let f = fp(g);
    if f.x221D_b7() != 0 {
        f.x2064_ledgeCooldown = cd().ledge_cooldown;
    }
    let el = f.dmg.x1860_element;
    if (el == 6 || el == 7) && f.is_sandbag() == 0 {
        ext::ftCo_800C318C(g, el != 6);
    } else {
        ftCo_8008DCE0(g, -1, facing_dir);
    }
}

pub fn ftCo_8008E984(f: &Fighter) -> bool {
    f.dmg.kb_applied == 0.0
        || (f.allow_sdi() != 0 && f.x221A_b3() != 0 && f.dmg.kb_applied < f.dmg.x18A8 + cd().x140)
}

/// static inline `inlineF0`.
fn inlineF0(g: G) {
    ext::ftCo_800C8D00(g);
    let m = fp(g).motion_id as i32;
    if m == 0xe0 || m == 0xe1 {
        ext::ftCo_800DC284(g);
    }
    if m == 0xe3 || m == 0xe4 {
        ext::ftCo_800DC3A4(g);
    }
}

pub fn ftCo_8008E9D0(g: G) {
    inlineF0(g);
    let scaled = ftCo_ScaleBy154(fp(g).dmg.kb_applied);
    let lvl = ftCo_8008D8E8(scaled);
    if ftCo_8008DA4C(g, fp(g).dmg.x1860_element, lvl) {
        ext::ftCo_800C0408(g);
    }
    ext::ftCommon_800804FC(fp(g));
}

/// static `inlineB0`.
fn inlineB0(f: &Fighter) -> bool {
    if f.dmg.kb_applied == 0.0 {
        return true;
    }
    f.allow_sdi() != 0 && f.x221A_b3() != 0 && f.dmg.kb_applied < f.dmg.x18A8 + cd().x140
}

pub fn ftCo_8008EB58(g: G) {
    let f = fp(g);
    let c = cd();
    let kb_applied = f.dmg.kb_applied;
    let should_update = kb_applied == 0.0
        || (f.allow_sdi() != 0 && f.x221A_b3() != 0 && kb_applied < f.dmg.x18A8 + c.x140);
    if should_update {
        ext::ftCo_800C8D00(g);
        let m = fp(g).motion_id as i32;
        if m == 0xe0 || m == 0xe1 {
            ext::ftCo_800DC284(g);
        }
        if m == 0xe3 || m == 0xe4 {
            ext::ftCo_800DC3A4(g);
        }
        let scaled_kb = fp(g).dmg.kb_applied * c.x154;
        let kb_level = if scaled_kb < c.x158 {
            0
        } else if scaled_kb < c.x15C {
            1
        } else if scaled_kb < c.x160 {
            2
        } else {
            3
        };
        if ftCo_8008DA4C(g, fp(g).dmg.x1860_element, kb_level) {
            ext::ftCo_800C0408(g);
        }
        ext::ftCommon_800804FC(fp(g));
    }
}

/// static `inlineB1`.
fn inlineB1(f: &Fighter) -> bool {
    f.x221C_b0() != 0 || f.dmg.x1838_percentTemp < cd().x3C0 as f32
}

/// static inline `inlineB2`.
fn inlineB2(g: G) {
    let m_ = fp(g).motion_id as i32;
    ext::ftCo_800C8D00(g);
    let _ = m_;
    let m = fp(g).motion_id as i32;
    if m == 0xe0 || m == 0xe1 {
        ext::ftCo_800DC284(g);
    }
    if m == 0xe3 || m == 0xe4 {
        ext::ftCo_800DC3A4(g);
    }
    let f = fp(g);
    let lvl = ftCo_8008D8E8(f.dmg.kb_applied * cd().x154);
    if ftCo_8008DA4C(g, f.dmg.x1860_element, lvl) {
        ext::ftCo_800C0408(g);
    }
    ext::ftCommon_800804FC(fp(g));
}

/// static inline `inlineB3`.
fn inlineB3(g: G) {
    let f = fp(g);
    let kb_applied = f.dmg.kb_applied;
    let should_update = kb_applied == 0.0
        || (f.allow_sdi() != 0 && f.x221A_b3() != 0 && f.dmg.kb_applied < f.dmg.x18A8 + cd().x140);
    if should_update {
        ftCo_8008E9D0(g);
    }
}

fn capturecut_trace(where_: &str, g: G) {
    let f = fp(g);
    let _ = (where_, f.victim_gobj);
    if ext::Mex_Enabled("test_log_capturecut") != 0 {
        // C: OSReport("CAPTURECUT-TRACE: %s P%d victim_gobj=%p motion=%d\n", ...); log only.
    }
}

pub fn ftCo_8008EC90(g: G) {
    let mut ret0 = false;
    let f = fp(g);
    let mut facing_dir = 0.0f32;
    let script_armor_absorbed = ext::ScriptGame_ArmorReact(f);
    'ret: {
        if f.x2220_b3() != 0
            || f.x2220_b4() != 0
            || f.dmg.kb_applied == 0.0
            || script_armor_absorbed != 0
            || ext::ScriptGame_CapsNoFlinch(f) != 0
        {
            inlineB2(g);
            break 'ret;
        } else if f.dmg.x1860_element == HitElement_Cape {
            if ext::ftCo_800C3538(g) {
                break 'ret;
            }
            facing_dir = f.facing_dir;
            ret0 = true;
        }
        if !ext::ftCo_800C44CC(g) && !ext::ftCo_800D2FA4(g) {
            if !fp(g).victim_gobj.is_null() {
                if fp(g).anon_221b.x221B_b5() == 0 {
                    let other_gobj = fp(g).victim_gobj;
                    let other_fp = fp(other_gobj);
                    if !ret0 && inlineB1(fp(g)) {
                        if other_fp.dmg.kb_applied != 0.0 {
                            if inlineB0(other_fp) {
                                inlineB2(g);
                                fp(g).dmg.x183C_applied = other_fp.dmg.x183C_applied;
                                fp(g).x1960_vibrateMult = other_fp.x1960_vibrateMult;
                                other_fp.x1828 = 3;
                                break 'ret;
                            }
                            ext::ftCo_800DE854(g);
                            ext::ftCo_800DCE34(other_gobj, g);
                            ext::ftCommon_8007DB58(g);
                            ftCo_8008E908(g, facing_dir);
                            other_fp.x1828 = 1;
                            break 'ret;
                        }
                        if fp(g).dmg.x183C_applied != 0 {
                            let hl = ext::ftCommon_CalcHitlag(
                                fp(g).dmg.x183C_applied,
                                other_fp.motion_id as i32,
                                other_fp.x1960_vibrateMult,
                            );
                            other_fp.dmg.x195c_hitlag_frames = hl;
                            other_fp.set_allow_sdi(1);
                            if other_fp.x2219_b5() == 0 {
                                if let Some(cb) = other_fp.pre_hitlag_cb {
                                    unsafe { cb(g) };
                                }
                                other_fp.set_x2219_b5(1);
                            }
                        }
                        fp(g).input.pressed_buttons = 0;
                        fp(g).input.released_buttons = 0;
                        inlineB2(g);
                        break 'ret;
                    }
                    if other_fp.dmg.kb_applied != 0.0 {
                        ext::ftCo_800DCE34(other_gobj, g);
                        ext::ftCommon_8007DB58(g);
                        ftCo_8008E908(g, facing_dir);
                        other_fp.x1828 = 1;
                        break 'ret;
                    }
                    ext::ftCo_800DCE34(other_gobj, g);
                    ext::ftCommon_8007DB58(g);
                    ftCo_8008E908(g, facing_dir);
                    ext::ftCommon_8007DB58(other_gobj);
                    ext::ftCo_800DE2F0(other_gobj);
                    break 'ret;
                }
                let vg = fp(g).victim_gobj;
                let other_fp = fp(vg);
                if inlineB0(fp(g)) {
                    if other_fp.dmg.kb_applied != 0.0 {
                        if inlineB1(other_fp) {
                            other_fp.dmg.x183C_applied = fp(g).dmg.x183C_applied;
                            other_fp.x1960_vibrateMult = fp(g).x1960_vibrateMult;
                            inlineB3(g);
                            other_fp.x1828 = 2;
                            break 'ret;
                        }
                        capturecut_trace("site A", g);
                        ext::ftCo_800DCE34(g, fp(g).victim_gobj);
                        ext::ftCommon_8007DB58(g);
                        ftCo_8008E908(g, facing_dir);
                        other_fp.x1828 = 1;
                        break 'ret;
                    }
                    inlineB3(g);
                    break 'ret;
                }
                if other_fp.dmg.kb_applied != 0.0 {
                    if inlineB1(other_fp) {
                        ext::ftCo_800DE854(fp(g).victim_gobj);
                    }
                    capturecut_trace("site B", g);
                    ext::ftCo_800DCE34(g, fp(g).victim_gobj);
                    ext::ftCommon_8007DB58(g);
                    ftCo_8008E908(g, facing_dir);
                    other_fp.x1828 = 1;
                    break 'ret;
                }
                ext::ftCommon_8007DB58(fp(g).victim_gobj);
                ext::ftCo_800DCFD4(fp(g).victim_gobj);
                capturecut_trace("site C (after 8007DB58 + 800DCFD4)", g);
                ext::ftCo_800DCE34(g, fp(g).victim_gobj);
                ext::ftCommon_8007DB58(g);
                ftCo_8008E908(g, facing_dir);
            } else if !fp(g).item_gobj.is_null()
                && ext::it_8026B2D8(fp(g).item_gobj as *mut HSD_GObj)
                && fp(g).x2222_b0() != 0
            {
                ext::ftDk_MS_349_800E06D8(g);
            } else if !ext::ftCo_8009F0F0(g) && !ext::ftCo_800C0CB8(g) {
                if fp(g).motion_id as i32 == ftCo_MS_DamageIce {
                    ftCo_8008DCE0(g, ftCo_MS_DamageIce, fp(g).facing_dir);
                    ext::ftCo_DamageIce_HitWhileFrozen(g);
                } else if !ext::ftCo_800C74F4(g) {
                    ext::ftCommon_8007DB58(g);
                    ftCo_8008E908(g, facing_dir);
                }
            }
        }
    }
    if ret0 {
        ext::ftCo_800C3598(g);
    }
}

pub fn ftCo_8008F744(g: G) {
    let f = fp(g);
    unsafe {
        if f.mv.co.damage.x0 > 0.0 {
            f.mv.co.damage.x0 -= 1.0;
        }
    }
    if f.x221C_b6() != 0 && unsafe { f.mv.co.damage.x0 } <= 0.0 {
        f.set_x221C_b6(0);
        f.x2098 = cd().x4CC as _;
        ext::ftCommon_8007F86C(g);
        if ext::ftCo_800C5240(g) {
            ext::ftCo_800C554C(fp(g));
        }
    }
}

/// static `inlineC0`.
fn inlineC0(g: G) -> bool {
    let f = fp(g);
    let x14 = unsafe { f.mv.co.damage.x14 };
    if x14 != 0.0 && x14 <= cd().x1D0 {
        f.input.pressed_buttons |= (HSD_PAD_X | HSD_PAD_Y) as u32;
        ext::ftCo_800CB870(g)
    } else {
        false
    }
}

pub fn ftCo_Damage_Anim(g: G) {
    ftCo_8008F744(g);
    let f = fp(g);
    if !ext::ftAnim_IsFramesRemaining(g) && f.x221C_b6() == 0 {
        if f.ground_or_air == GA_Air {
            if f.x2224_b2() != 0 {
                ext::ftCo_80090780(g);
            } else if !inlineC0(g) {
                ext::ftCo_Fall_Enter(g);
            }
        } else {
            ext::ft_8008A2BC(g);
        }
    }
}

pub fn doIasa(g: G) -> bool {
    let f = fp(g);
    'ret_inline: {
        if unsafe { f.mv.co.damage.x1A } == 0 {
            break 'ret_inline;
        }
        unsafe {
            if f.mv.co.damage.x1B != 0 {
                f.mv.co.damage.x1B -= 1;
            }
            if f.mv.co.damage.x1B != 0 {
                break 'ret_inline;
            }
        }
        if f.ground_or_air == GA_Air && f.x8c_kb_vel.y < 0.0 && (ext::ftCo_800D69C4(g) || ext::ftCo_800CB8E0(g)) {
            let f = fp(g);
            f.x8c_kb_vel.x = 0.0;
            f.x8c_kb_vel.y = 0.0;
            f.x8c_kb_vel.z = 0.0;
            ext::ftCommon_8007EBAC(f, 12, 0);
            ext::ftCo_800BFFD0(fp(g), 121, 0);
            fp(g).set_x2227_b4(1);
            return true;
        }
    }
    if ext::ftCo_Jump_GetInput(g) != 0 {
        let f = fp(g);
        unsafe { f.mv.co.damage.x14 = f.mv.co.damage.x0 };
        return true;
    }
    false
}

pub fn ftCo_Damage_IASA(g: G) {
    let f = fp(g);
    if f.x221C_b6() == 0 {
        if ext::ftCo_800C5240(g) {
            if fp(g).ground_or_air == GA_Air {
                ext::ftCo_HammerFall_IASA(g);
            } else {
                ext::ftCo_HammerWait_IASA(g);
            }
        } else {
            let f = fp(g);
            let x14 = unsafe { f.mv.co.damage.x14 };
            if x14 != 0.0 && x14 <= cd().x1D0 {
                f.input.pressed_buttons |= (HSD_PAD_X | HSD_PAD_Y) as u32;
            }
            if fp(g).ground_or_air == GA_Air {
                ext::ftCo_Fall_IASA_Inner(g);
            } else {
                ext::ftCo_Wait_IASA(g);
            }
        }
    } else {
        doIasa(g);
    }
}

pub fn ftCo_Damage_Phys(g: G) {
    let f = fp(g);
    if f.ground_or_air == GA_Air {
        if f.x221C_b6() == 0 {
            ext::ft_80084DB0(g);
        } else {
            ext::ft_80084EEC(g);
        }
    } else {
        ext::ft_80084F3C(g);
    }
}

pub fn ftCo_Damage_Coll(g: G) {
    let f = fp(g);
    if f.ground_or_air == GA_Ground {
        ext::ft_800848DC(g, ftCo_8008FC94);
    } else if ext::ft_80081DD4(g) {
        if f.x2224_b2() != 0 {
            ext::ftCo_80097D40(g);
        } else {
            let c = cd();
            let mag = sqrtf(sq_len2(f.x8c_kb_vel.x, f.x8c_kb_vel.y));
            if mag >= c.x1E0 {
                ext::ftCo_80097D40(g);
            } else if mag >= c.x1E4 {
                ext::ftCo_Landing_Enter_Basic(g);
            } else {
                ext::ftCommon_8007D7FC(fp(g));
            }
        }
    }
}

pub fn ftCo_8008FC94(g: G) {
    ext::ftCommon_8007D5D4(fp(g));
}

pub fn ftCo_Damage_SetMv8FromKbThreshold(f: &mut Fighter) {
    let c = cd();
    let kb_vel = if f.ground_or_air == GA_Air {
        sqrtf(sq_len3(&Vec3 { x: f.x8c_kb_vel.x, y: f.x8c_kb_vel.y, z: f.x8c_kb_vel.z }))
    } else {
        abs_c(f.xF0_ground_kb_vel)
    };
    let v = if kb_vel < c.x568 {
        0
    } else if kb_vel < c.x56C {
        c.x57C
    } else if kb_vel < c.x570 {
        c.x57C
    } else if kb_vel < c.x574 {
        c.x580
    } else if kb_vel < c.x578 {
        c.x584
    } else {
        c.x588
    };
    unsafe { f.mv.co.damage.x8 = v };
}

/// static inline `inlineD0`.
fn inlineD0(g: G) {
    let f = fp(g);
    if unsafe { f.mv.co.damage.x8 } != 0 {
        unsafe { f.mv.co.damage.x8 -= 1 };
        if unsafe { f.mv.co.damage.x8 } == 0 {
            let (x, y);
            if f.ground_or_air == GA_Air {
                y = f.x8c_kb_vel.x;
                x = f.x8c_kb_vel.y;
            } else {
                let kb_vel = f.xF0_ground_kb_vel;
                y = f.coll_data.floor.normal.y * kb_vel;
                x = -f.coll_data.floor.normal.x * kb_vel;
            }
            let mut trajectory = atan2f(-y, x);
            let q = core::ptr::addr_of_mut!(fp(g).x60C) as *mut core::ffi::c_void;
            let j = unsafe { (*f.parts.add(FtPart_TopN as usize)).joint };
            ext::efAsync_Spawn(g, q, 4, 1032, j, &mut trajectory);
            ftCo_Damage_SetMv8FromKbThreshold(fp(g));
        }
    }
}

pub fn ftCo_DamageFly_Anim(g: G) {
    inlineD0(g);
    ftCo_8008F744(g);
    let f = fp(g);
    if !ext::ftAnim_IsFramesRemaining(g) && f.x221C_b6() == 0 && !inlineC0(g) {
        ext::ftCo_80090780(g);
    }
}

pub fn ftCo_DamageFly_IASA(g: G) {
    if fp(g).x221C_b6() == 0 {
        ext::ftCo_DamageFall_IASA(g);
    } else {
        doIasa(g);
    }
}

/// static `doFlyRoll`.
fn doFlyRoll(g: G) {
    let f = fp(g);
    let trajectory = f.facing_dir * atan2f(f.self_vel.x + f.x8c_kb_vel.x, f.self_vel.y + f.x8c_kb_vel.y);
    let bone = ext::ftParts_GetBoneIndex(f, FtPart_XRotN as i32);
    ext::ftPartSetRotX(fp(g), bone, trajectory);
}

/// static `inlineE0`.
fn inlineE0(g: G) {
    if fp(g).x221C_b6() == 0 {
        ext::ft_80084DB0(g);
    } else {
        ext::ft_80084EEC(g);
    }
}

pub fn ftCo_DamageFly_Phys(g: G) {
    let f = fp(g);
    if f.ground_or_air == GA_Air {
        inlineE0(g);
    } else {
        ext::ft_80084F3C(g);
    }
    let f = fp(g);
    if f.motion_id as i32 == ftCo_MS_DamageFlyRoll {
        doFlyRoll(g);
    }
    let f = fp(g);
    if unsafe { !f.x1064_thrownHitbox.anon_134.owner.is_null() } {
        let v = Vec3 { x: f.x8c_kb_vel.x, y: f.x8c_kb_vel.y, z: f.x8c_kb_vel.z };
        if sqrtf(sq_len3(&v)) < cd().x1C8 {
            ext::ftColl_8007AFF8(g);
        }
    }
}

pub fn ftCo_DamageFly_Coll(g: G) {
    if ext::ft_80081DD4(g) {
        ftCo_80090184(g);
    } else {
        let env = fp(g).coll_data.env_flags;
        if env & Collide_RightWallHug != 0 || env & Collide_LeftWallHug != 0 || env & Collide_CeilingHug != 0 {
            if ext::ftCo_800C1D38(g) {
                return;
            }
            if ext::ftCo_800C23A0(g) {
                return;
            }
            if ext::ftCo_800C17CC(g) {
                return;
            }
        }
    }
}

pub fn ftCo_80090184(g: G) {
    if ext::ftCo_80098928(g) {
        return;
    }
    if ext::ftCo_8009872C(g) {
        return;
    }
    ext::ftCo_80097D40(g);
}

pub fn ftCo_DamageFlyRoll_Anim(g: G) {
    inlineD0(g);
    ftCo_8008F744(g);
    if fp(g).x221C_b6() == 0 {
        ext::ftCo_80090780(g);
    }
}

pub fn ftCo_DamageFlyRoll_IASA(g: G) {
    if fp(g).x221C_b6() == 0 {
        ext::ftCo_DamageFall_IASA(g);
    } else {
        doIasa(g);
    }
}

pub fn ftCo_DamageFlyRoll_Phys(g: G) {
    let f = fp(g);
    if f.ground_or_air == GA_Air {
        inlineE0(g);
    } else {
        ext::ft_80084F3C(g);
    }
    if fp(g).motion_id as i32 == ftCo_MS_DamageFlyRoll {
        doFlyRoll(g);
    }
    let f = fp(g);
    if unsafe { !f.x1064_thrownHitbox.anon_134.owner.is_null() } {
        let v = Vec3 { x: f.x8c_kb_vel.x, y: f.x8c_kb_vel.y, z: f.x8c_kb_vel.z };
        if sqrtf(sq_len3(&v)) < cd().x1C8 {
            ext::ftColl_8007AFF8(g);
        }
    }
    if fp(g).motion_id as i32 == ftCo_MS_DamageFlyRoll {
        doFlyRoll(g);
    }
}

pub fn ftCo_DamageFlyRoll_Coll(g: G) {
    if ext::ft_80081DD4(g) {
        if ext::ftCo_80098928(g) {
            return;
        }
        if ext::ftCo_8009872C(g) {
            return;
        }
        ext::ftCo_80097D40(g);
    } else {
        let env = fp(g).coll_data.env_flags;
        if env & Collide_RightWallHug != 0 || env & Collide_LeftWallHug != 0 || env & Collide_CeilingHug != 0 {
            if ext::ftCo_800C1D38(g) {
                return;
            }
            if ext::ftCo_800C23A0(g) {
                return;
            }
            ext::ftCo_800C17CC(g);
        }
    }
}
