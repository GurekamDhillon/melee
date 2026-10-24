#![allow(non_snake_case, non_upper_case_globals)]
//! Native port of src/melee/ft/kinds/ftCommon/ftCo_ThrownKirby.c (every function, TARGET_PC build:
//! the Geno define copy policy and the m-ex Kirby-like FTKB_IS_KIRBY). Fused sites (shipping
//! contraction): `SQ(x) + SQ(y)` in inlineA0 and the dot product in the two Coll callbacks, each
//! one `fmadds`. Calls out of the TU go through `ext` (unported ones panic until rewired). Checked
//! against the C (whole TU, shipping contraction) by tests/tu_ftCo_ThrownKirby.rs.
use crate::types::*;
use melee_core::fp::fmadds;

pub type G = *mut HSD_GObj;

#[inline(always)]
fn fp<'a>(g: G) -> &'a mut Fighter {
    unsafe { &mut *((*g).user_data as *mut Fighter) }
}
#[inline(always)]
fn jobj(g: G) -> *mut HSD_JObj {
    unsafe { (*g).hsd_obj as *mut HSD_JObj }
}
#[inline(always)]
fn tk(f: &mut Fighter) -> &mut ftCommon_MotionVars_thrownkirby {
    unsafe { &mut f.mv.co.thrownkirby }
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
        Geno_DefineKirbyCopy(kind: i32) -> i32 = |_| todo!("TODO(port) Geno_DefineKirbyCopy (pc/geno)");
        // `OSReport("geno: Kirby inhales define kind ...")`: log only.
        OSReport_GenoKirbyCopy(kind: i32, copy: i32) = |_, _| {};
        ftKb_MexKirbyLike(kind: i32) -> i32 = |_| todo!("TODO(port) ftKb_MexKirbyLike (ftkirby.c)");
        ft_800824A0(g: G, b: *mut ftCollisionBox) -> bool = |g: G, b: *mut ftCollisionBox| -> bool { unsafe { crate::tu::ft_081b::ft_800824a0(g as _, b as _) as _ } };
        Fighter_UpdateModelScale(g: G) = |g: G| { unsafe { let _ = crate::tu::fighter::Fighter_UpdateModelScale(g as _); } };
        Fighter_ChangeMotionState(g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G) = |g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G| { unsafe { let _ = crate::tu::fighter::Fighter_ChangeMotionState(g as _, msid as _, flags as _, start as _, speed as _, blend as _, other as _); } };
        ftCommon_8007E2F4(f: &mut Fighter, v: i16) = crate::ftcommon2::ftcommon_8007e2f4;
        ftColl_8007B62C(g: G, v: i32) = |_, _| todo!("TODO(port) ftColl_8007B62C (ftcoll.c)");
        ftCommon_8007EFC0(f: &mut Fighter, v: u32) = crate::ftcommon2::ftcommon_8007efc0;
        ftCommon_InitGrab(f: &mut Fighter, a: i32, v: f32) = |_, _, _| todo!("TODO(port) ftCommon_InitGrab (ftcommon.c)");
        ftAnim_8006EBA4(g: G) = |g| unsafe { crate::tu::ftanim::ftAnim_8006EBA4(g) };
        ftCommon_8007D5D4(f: &mut Fighter) = crate::ftcommon2::ftcommon_8007d5d4;
        ftCommon_SetAccessory(f: &mut Fighter, j: *mut HSD_Joint) = |_, _| todo!("TODO(port) ftCommon_SetAccessory (ftcommon.c)");
        ftKb_SpecialN_800F5898(g: G) -> *mut HSD_Joint = |_| todo!("TODO(port) ftKb_SpecialN_800F5898 (ftKb_Init.c)");
        HSD_JObjSetMtxDirtySub(j: *mut HSD_JObj) = |_| todo!("TODO(port) HSD_JObjSetMtxDirtySub (jobj.c)");
        lb_8000C2F8(a: *mut HSD_JObj, b: *mut HSD_JObj) = |a: *mut HSD_JObj, b: *mut HSD_JObj| { unsafe { let _ = melee_runtime::tu::lb_00B0::lb_8000C2F8(a as _, b as _); } };
        ftParts_GetBoneIndex(f: &mut Fighter, part: i32) -> i32 = |_, _| todo!("TODO(port) ftParts_GetBoneIndex (ftparts.c)");
        ftColl_8007ABD0(hit: *mut HitCapsule, dmg: u32, g: G) = |_, _, _| todo!("TODO(port) ftColl_8007ABD0 (ftcoll.c)");
        ftColl_8007AFF8(g: G) = |_| todo!("TODO(port) ftColl_8007AFF8 (ftcoll.c)");
        ftKb_SpecialN_800F5820(g: G, b: *mut ftCollisionBox, s: f32) = |_, _, _| todo!("TODO(port) ftKb_SpecialN_800F5820 (ftKb_Init.c)");
        ftKb_SpecialN_800F58AC(g: G, v: *mut Vec3, facing: f32) -> f32 = |_, _, _| todo!("TODO(port) ftKb_SpecialN_800F58AC (ftKb_Init.c)");
        ftKb_SpecialN_800F58D8(g: G, v: *mut Vec3, facing: f32) -> f32 = |_, _, _| todo!("TODO(port) ftKb_SpecialN_800F58D8 (ftKb_Init.c)");
        ftKb_SpecialN_800F5A88(g: G) -> f32 = |_| todo!("TODO(port) ftKb_SpecialN_800F5A88 (ftKb_Init.c)");
        ftKb_SpecialN_800F5AB0(g: G) -> f32 = |_| todo!("TODO(port) ftKb_SpecialN_800F5AB0 (ftKb_Init.c)");
        ftKb_SpecialN_800F5AC0() -> f32 = || todo!("TODO(port) ftKb_SpecialN_800F5AC0 (ftKb_Init.c)");
        ftKb_SpecialN_800F5AD8() -> f32 = || todo!("TODO(port) ftKb_SpecialN_800F5AD8 (ftKb_Init.c)");
        ftKb_SpecialN_800F5874(v: *mut Vec2) = |_| todo!("TODO(port) ftKb_SpecialN_800F5874 (ftKb_Init.c)");
        ftKb_SpecialN_800F190C(g: G, kind: i32) = |_, _| todo!("TODO(port) ftKb_SpecialN_800F190C (ftKb_Init.c)");
        ftKb_SpecialN_800EEEC4(g: G, kind: i32) = |_, _| todo!("TODO(port) ftKb_SpecialN_800EEEC4 (ftKb_Init.c)");
        ftKb_SpecialN_800F5A98() -> f32 = || todo!("TODO(port) ftKb_SpecialN_800F5A98 (ftKb_Init.c)");
        ftKb_SpecialN_800F5A70() -> f32 = || todo!("TODO(port) ftKb_SpecialN_800F5A70 (ftKb_Init.c)");
        ftCommon_GrabMash(f: &mut Fighter, v: f32) -> bool = |_, _| todo!("TODO(port) ftCommon_GrabMash (ftcommon.c)");
        lbVector_Mirror(a: *mut Vec3, b: *mut Vec3) = |a: *mut Vec3, b: *mut Vec3| { unsafe { let _ = melee_runtime::tu::lbvector::lbVector_Mirror(a as _, b as _); } };
        ftCo_Fall_Enter(g: G) = crate::tu::ftCo_Fall::ftCo_Fall_Enter;
    }
}

/// TARGET_PC `FTKB_IS_KIRBY(k)` (ft/forward.h).
fn is_kirby(k: i32) -> bool {
    k == Ft_Kind_Kirby as i32 || (k >= Ft_Kind_Mex0 as i32 && k < Ft_Kind_Max as i32 && ext::ftKb_MexKirbyLike(k) != 0)
}

/// `HSD_JObjSetScale` (jobj.h inline).
fn jobj_set_scale(j: *mut HSD_JObj, s: &Vec3) {
    assert!(!j.is_null(), "jobj.h:760 jobj");
    let jr = unsafe { &mut *j };
    jr.scale = *s;
    if jr.flags & JOBJ_MTX_INDEP_SRT as u32 == 0 && !melee_types::inline::HSD_JObjMtxIsDirty(jr) {
        ext::HSD_JObjSetMtxDirtySub(j);
    }
}

pub unsafe extern "C" fn ftCo_800BE7C0_cb(g: G) {
    ftCo_800BE7C0(g)
}
pub unsafe extern "C" fn ftCo_800BE6AC_cb(g: G) {
    ftCo_800BE6AC(g)
}

pub fn ftCo_800BD9E0(g: G, victim_gobj: G) -> i32 {
    let mut victim_kind = fp(victim_gobj).kind as i32;
    {
        let copy = ext::Geno_DefineKirbyCopy(victim_kind);
        if copy != -2 {
            ext::OSReport_GenoKirbyCopy(victim_kind, copy);
            return if copy < 0 { Ft_Kind_Kirby as i32 } else { copy };
        }
    }
    if is_kirby(victim_kind) {
        ftCo_800BDA50(victim_gobj);
        return unsafe { fp(victim_gobj).u.kb.hat.kind } as i32;
    }
    if (victim_kind.wrapping_sub(Ft_Kind_Boy as i32) as u32) <= 1 || victim_kind == Ft_Kind_Sandbag as i32 {
        return unsafe { fp(g).u.kb.hat.kind } as i32;
    }
    if victim_kind == Ft_Kind_Nana as i32 {
        victim_kind = Ft_Kind_Popo as i32;
    }
    victim_kind
}

pub fn ftCo_800BDA50(g: G) {
    let f = fp(g);
    if unsafe { f.u.kb.hat.kind } as i32 != Ft_Kind_Kirby as i32 {
        unsafe { tk(f).anon_18.anon_0.set_x18_b1(1) };
    }
}

pub fn ftCo_800BDA74(g: G, normal: &mut Vec3) {
    let f = fp(g);
    let bx = &mut tk(f).coll_box as *mut ftCollisionBox;
    if ext::ft_800824A0(g, bx) {
        *normal = fp(g).coll_data.floor.normal;
        return;
    }
    let c = &fp(g).coll_data;
    if c.env_flags & Collide_RightWallMask != 0 {
        *normal = c.right_facing_wall.normal;
    } else if c.env_flags & Collide_LeftWallMask != 0 {
        *normal = c.left_facing_wall.normal;
    } else if c.env_flags & (Collide_CeilingPush | Collide_CeilingHug) != 0 {
        *normal = c.ceiling.normal;
    } else {
        normal.x = 0.0;
        normal.y = 0.0;
        normal.z = 0.0;
    }
}

/// static inline `inlineB2`: shared entry of ThrownKirbyStar / ThrownCopyStar.
fn inline_b2(g: G, thrower: G, scale: &mut Vec3, msid: i32, vel_func: fn(G, *mut Vec3, f32) -> f32, get_float: fn(G) -> f32, x18_b0: bool) {
    let j = jobj(g);
    let f = fp(g);
    f.facing_dir = -fp(thrower).facing_dir;
    tk(f).thrower_gobj = thrower;
    tk(f).x8 = 0.0;
    let sv = &mut f.self_vel as *mut Vec3;
    let facing = f.facing_dir;
    let v = vel_func(thrower, sv, facing);
    tk(fp(g)).x4 = v;
    ext::Fighter_UpdateModelScale(g);
    tk(fp(g)).scale = unsafe { (*j).scale };
    ext::Fighter_ChangeMotionState(g, msid, Ft_MF_SkipThrowException, 0.0, 1.0, 0.0, thrower);
    let f = fp(g);
    f.take_dmg_2_cb = Some(ftCo_800BE7C0_cb);
    f.take_dmg_cb = Some(ftCo_800BE7C0_cb);
    ext::ftCommon_8007E2F4(f, 0x1FF);
    fp(g).set_invisible(1);
    ext::ftColl_8007B62C(g, 2);
    ext::ftCommon_8007EFC0(fp(g), 1);
    let gf = get_float(thrower);
    ext::ftCommon_InitGrab(fp(g), 0, gf);
    ext::ftAnim_8006EBA4(g);
    ext::ftCommon_8007D5D4(fp(g));
    let f = fp(g);
    unsafe {
        tk(f).anon_18.anon_0.set_x18_b0(x18_b0 as u8);
        tk(f).anon_18.anon_0.set_x18_b1(0);
    }
    let acc = ext::ftKb_SpecialN_800F5898(thrower);
    ext::ftCommon_SetAccessory(fp(g), acc);
    let s = fp(g).co_attrs.xDC;
    scale.x = s;
    scale.y = s;
    scale.z = s;
    jobj_set_scale(fp(g).x20A0_accessory, scale);
    let idx = ext::ftParts_GetBoneIndex(fp(g), FtPart_YRotN as i32);
    let f = fp(g);
    let joint = unsafe { (*f.parts.offset(idx as isize)).joint };
    ext::lb_8000C2F8(f.x20A0_accessory, joint);
}

pub fn ftCo_800BDB58(g: G, thrower: G) {
    let mut scale = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    inline_b2(g, thrower, &mut scale, ftCo_MS_ThrownKirbyStar as i32, ext::ftKb_SpecialN_800F58AC, ext::ftKb_SpecialN_800F5A88, false);
    let f = fp(g);
    let dmg = melee_runtime::cabi::f2u32(f.co_attrs.kirby_b_star_damage as f64);
    ext::ftColl_8007ABD0(f.x914.as_mut_ptr(), dmg, g);
    let bx = &mut tk(fp(g)).coll_box as *mut ftCollisionBox;
    ext::ftKb_SpecialN_800F5820(thrower, bx, scale.x);
}

pub fn ftCo_ThrownKirbyStar_Anim(_g: G) {}
pub fn ftCo_ThrownKirbyStar_IASA(_g: G) {}

/// static inline `inlineA0`: star speed decay. `SQ(x) + SQ(y)` is one fmuladd -> fmadds.
fn inline_a0(g: G) {
    let f = fp(g);
    let (x, y) = (f.self_vel.x, f.self_vel.y);
    let vel_mag = melee_runtime::gekko::sqrtf(fmadds(x, x, y * y));
    let x4 = tk(f).x4;
    if vel_mag > x4 {
        f.self_vel.x = (f.self_vel.x * (vel_mag - x4)) / vel_mag;
        let svx = unsafe { f.mv.ca.specialhi.vel.x };
        f.self_vel.y = (f.self_vel.y * (vel_mag - svx)) / vel_mag;
        f.facing_dir = if f.self_vel.y < 0.0 { -1.0 } else { 1.0 };
    } else {
        f.self_vel.x = 0.0;
    }
    let d = ext::ftKb_SpecialN_800F5AC0();
    let f = fp(g);
    f.grab_timer -= d;
}

fn star_phys(g: G) {
    inline_a0(g);
    let v = ext::ftKb_SpecialN_800F5AD8();
    let r = ext::ftCommon_GrabMash(fp(g), v);
    tk(fp(g)).x14 = r as i32;
    if fp(g).grab_timer <= 0.0 {
        ftCo_800BE494(g);
    }
}

fn star_coll(g: G) {
    let mut normal = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    ftCo_800BDA74(g, &mut normal);
    if normal.x != 0.0 || normal.y != 0.0 || normal.z != 0.0 {
        let f = fp(g);
        let sv = f.self_vel;
        ext::lbVector_Mirror(&mut f.self_vel, &mut normal);
        let f = fp(g);
        // `a.x * b.x + a.y * b.y`: one fmuladd -> fmadds.
        if fmadds(sv.x, f.self_vel.x, sv.y * f.self_vel.y) < 0.0 {
            ftCo_800BE494(g);
            tk(fp(g)).x8 = if normal.x < 0.0 { -1.0 } else { 1.0 };
        }
    }
}

pub fn ftCo_ThrownKirbyStar_Phys(g: G) {
    star_phys(g);
}

pub fn ftCo_ThrownKirbyStar_Coll(g: G) {
    star_coll(g);
}

pub fn ftCo_800BE000(g: G, thrower: G) {
    let mut scale = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    inline_b2(g, thrower, &mut scale, ftCo_MS_ThrownCopyStar as i32, ext::ftKb_SpecialN_800F58D8, ext::ftKb_SpecialN_800F5AB0, true);
    ext::ftColl_8007AFF8(g);
    let bx = &mut tk(fp(g)).coll_box as *mut ftCollisionBox;
    ext::ftKb_SpecialN_800F5820(thrower, bx, scale.x);
}

pub fn ftCo_ThrownCopyStar_Anim(_g: G) {}
pub fn ftCo_ThrownCopyStar_IASA(_g: G) {}

pub fn ftCo_ThrownCopyStar_Phys(g: G) {
    star_phys(g);
}

pub fn ftCo_ThrownCopyStar_Coll(g: G) {
    star_coll(g);
}

pub fn ftCo_800BE494(g: G) {
    ext::Fighter_ChangeMotionState(g, ftCo_MS_ThrownKirby as i32, Ft_MF_Unk06, 0.0, 1.0, 0.0, core::ptr::null_mut());
    let f = fp(g);
    f.take_dmg_2_cb = Some(ftCo_800BE7C0_cb);
    f.take_dmg_cb = Some(ftCo_800BE7C0_cb);
    ext::ftCommon_8007E2F4(f, 0x1FF);
    fp(g).set_invisible(0);
    ext::ftColl_8007B62C(g, 0);
    let f = fp(g);
    if unsafe { tk(f).anon_18.anon_0.x18_b0() } == 0 {
        let mut sv = Vec2 { x: 0.0, y: 0.0 };
        ext::ftKb_SpecialN_800F5874(&mut sv);
        let f = fp(g);
        f.self_vel.y = sv.y;
        let vy = unsafe { f.mv.ca.specialhi.vel.y };
        if vy != 0.0 {
            f.self_vel.x = vy * sv.x;
        } else {
            f.self_vel.x = sv.x * if f.self_vel.x < 0.0 { -1.0 } else { 1.0 };
        }
    } else if is_kirby(f.kind as i32) && unsafe { tk(fp(g)).anon_18.anon_0.x18_b1() } != 0 && unsafe { fp(g).u.kb.hat.x8_b0() } == 0 {
        let k = unsafe { fp(g).u.kb.hat.kind } as i32;
        ext::ftKb_SpecialN_800F190C(g, k);
        let k = unsafe { fp(g).u.kb.hat.kind } as i32;
        ext::ftKb_SpecialN_800EEEC4(g, k);
        unsafe { fp(g).u.kb.hat.kind = Ft_Kind_Kirby as _ };
    }
    let x10 = ext::ftKb_SpecialN_800F5A98();
    tk(fp(g)).x10 = x10;
    let x10 = tk(fp(g)).x10;
    ext::ftCommon_InitGrab(fp(g), 0, x10);
    let xc = ext::ftKb_SpecialN_800F5A70();
    tk(fp(g)).xC = xc;
    ext::ftAnim_8006EBA4(g);
    ext::ftCommon_8007D5D4(fp(g));
    fp(g).accessory4_cb = Some(ftCo_800BE6AC_cb);
}

pub fn ftCo_ThrownKirby_Anim(_g: G) {}
pub fn ftCo_ThrownKirby_IASA(_g: G) {}

pub fn ftCo_ThrownKirby_Phys(g: G) {
    let d = ext::ftKb_SpecialN_800F5AC0();
    let f = fp(g);
    f.grab_timer -= d;
    if unsafe { tk(f).anon_18.anon_0.x18_b0() } == 0 {
        if fp(g).grab_timer <= 0.0 {
            ext::Fighter_UpdateModelScale(g);
            ext::ftCo_Fall_Enter(g);
        }
    } else {
        ext::Fighter_UpdateModelScale(g);
        ext::ftCo_Fall_Enter(g);
    }
}

pub fn ftCo_ThrownKirby_Coll(_g: G) {}

/// SSE mulss operand rule (first NaN operand wins); matches the C oracle's operand order for NaN sign.
#[inline(never)]
fn mulss(a: f32, b: f32) -> f32 {
    if a.is_nan() { f32::from_bits(a.to_bits() | 0x0040_0000) } else if b.is_nan() { f32::from_bits(b.to_bits() | 0x0040_0000) } else { a * b }
}

pub fn ftCo_800BE6AC(g: G) {
    let j = jobj(g);
    let gt = fp(g).grab_timer;
    let t = tk(fp(g));
    let xc = t.xC;
    let mut fr = (t.x10 - gt) / t.x10;
    fr *= xc;
    let s = (1.0 - xc) - -fr;
    let scale = Vec3 { x: mulss(t.scale.x, s), y: mulss(t.scale.y, s), z: mulss(t.scale.z, s) };
    jobj_set_scale(j, &scale);
}

pub fn ftCo_800BE7C0(g: G) {
    ext::Fighter_UpdateModelScale(g);
}
