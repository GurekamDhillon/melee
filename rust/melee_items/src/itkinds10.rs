//! Port round 10 (night 3, round 2): hand ports of 23 item functions whose only unported callees
//! were math that group 1 has since ported natively: `lbVector_*` (`melee_runtime::lbvector`),
//! `atan2f` (`melee_runtime::lbtrigf`), MSL `sinf` / `cosf` (`melee_runtime::gekko`), plus
//! `HSD_Randi` / `HSD_Randf` (`melee_runtime::random::HsdRand`, passed in as in round 9).
//! Verified against verbatim C copies (tests/c_oracle/round10_bodies.h, extracted by
//! gen_round10.py) in tests/oracle_round10.rs; the oracle compiles the game's lbvector.c,
//! lbtrigf.c and pc/gameworld/gekko_fp.c into the same TU.
//!
//! C globals become parameters: `it_804D6D28` -> `common: &ItemCommonData`, `p_ftCommonData` ->
//! `cd: &ftCommonData`, the RNG -> `rand: &mut HsdRand`.
//!
//! Fused sites (shipping-build contraction; checked in the oracle IR):
//! - `it_8027770C`: the dot `vz*nz + (vx*nx + vy*ny)`, 2 f32;
//! - `it_8027BB1C`: 6 f32 (each `a*c -/+ b*s`, left product fused);
//! - `it_802E628C`: 1 f32 (`var += x10 * cosf(..)`).
//!
//! Uninitialised locals the C reads are 0 (the build's `-ftrivial-auto-var-init=zero`):
//! `it_80277040`'s surface normal and touched-surface normal when no collision flag is set,
//! `it_8027649C`'s.
#![allow(non_snake_case)]

use crate::helpers::{clamp_tau, it_8027b798_calc_angle, COLLIDE_CEILING_MASK, M_PI, M_PI_2, M_TAU};
use crate::item::norm_xf74;
use crate::itlib::{it_80276408, COLLIDE_FLOOR_MASK};
use crate::layout::{
    ftCommonData, itBoxAttributes, itHitodemanAttributes, itMarilAttributes,
    itPikachutJoltGroundAttributes, itTincleAttributes, it_2E5A_Attrs, Collide_LeftWallMask,
    Collide_RightWallMask, Item, ItemCommonData, Vec as GcVec, GA_Air, It_Kind_Flipper,
};
use melee_core::fp::{fmadds, fmsubs};
use melee_core::vec::Vec3 as RVec;
use melee_runtime::gekko::{cosf, sinf};
use melee_runtime::lbtrigf::atan2f;
use melee_runtime::lbvector;
use melee_runtime::random::HsdRand;

#[inline(always)]
fn rv(v: &GcVec) -> RVec {
    RVec { x: v.x, y: v.y, z: v.z }
}
#[inline(always)]
fn gv(v: &RVec) -> GcVec {
    GcVec { x: v.x, y: v.y, z: v.z }
}
#[inline(always)]
fn normalize(v: &mut GcVec) -> f32 {
    let mut r = rv(v);
    let l = lbvector::normalize(&mut r);
    *v = gv(&r);
    l
}
#[inline(always)]
fn mirror(v: &mut GcVec, m: &GcVec) {
    let mut r = rv(v);
    lbvector::mirror(&mut r, &rv(m));
    *v = gv(&r);
}
#[inline(always)]
fn zero() -> GcVec {
    GcVec { x: 0.0, y: 0.0, z: 0.0 }
}

/// `itHammerData` (ithammer.c, file-local): GC offsets 0x0 u32, 0x4 u32, 0x8 f32.
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct itHammerData {
    /// +0x0
    pub x0: u32,
    /// +0x4
    pub x4: u32,
    /// +0x8
    pub x8: f32,
}

// ---- itcoll.c --------------------------------------------------------------------------

/// `it_80271830`: rotate the item's ECB extents by `angle` (wrapped into [0, tau] in f64 steps)
/// about z and store the bounding values back.
pub fn it_80271830(item: &mut Item, mut angle: f32) {
    while angle < 0.0 {
        angle = (angle as f64 + M_TAU) as f32;
    }
    while angle as f64 > M_TAU {
        angle = (angle as f64 - M_TAU) as f32;
    }
    let axis = RVec { x: 0.0, y: 0.0, z: 1.0 };
    let mut a = RVec { x: 0.0, y: item.xBEC.top, z: 0.0 };
    lbvector::rotate_about_unit_axis(&mut a, &axis, angle);
    let mut b = RVec { x: 0.0, y: item.xBEC.bottom, z: 0.0 };
    lbvector::rotate_about_unit_axis(&mut b, &axis, angle);
    let mut c = RVec { x: item.xBEC.right, y: 0.0, z: 0.0 };
    lbvector::rotate_about_unit_axis(&mut c, &axis, angle);
    let mut d = RVec { x: item.xBEC.left, y: 0.0, z: 0.0 };
    lbvector::rotate_about_unit_axis(&mut d, &axis, angle);
    let mx = |p: f32, q: f32| if p > q { p } else { q };
    let mn = |p: f32, q: f32| if p < q { p } else { q };
    item.xBEC.left = mx(mx(mx(a.x, b.x), c.x), d.x);
    item.xBEC.top = mx(mx(mx(a.y, b.y), c.y), d.y);
    item.xBEC.right = mn(mn(mn(a.x, b.x), c.x), d.x);
    item.xBEC.bottom = mn(mn(mn(a.y, b.y), c.y), d.y);
}

// ---- itmaplib.c ------------------------------------------------------------------------

/// `it_8027649C_ref` (static inline): the signed angle of the touched surface's normal from up,
/// times the facing direction. `sp14` gets up, `sp20` the normal (left as passed when no
/// collision flag is set).
pub fn it_8027649c_ref(item: &Item, sp14: &mut GcVec, sp20: &mut GcVec) -> f32 {
    sp14.z = 0.0;
    sp14.x = 0.0;
    sp14.y = 1.0;
    it_80276408(&item.x378_itemColl, sp20);
    let angle = lbvector::angle(&rv(sp20), &rv(sp14));
    let int_dir: i32 = if sp20.x < 0.0 { -1 } else { 1 };
    item.facing_dir * (angle * int_dir as f32)
}

/// `it_8027649C`: as `it_8027649C_ref` with locals (the normal is 0 when nothing is touched).
pub fn it_8027649c(item: &Item) -> f32 {
    let mut sp14 = zero();
    let mut sp20 = zero();
    it_8027649c_ref(item, &mut sp14, &mut sp20)
}

/// `it_80277040`: slide along a steep surface. `common` is `it_804D6D28`.
/// # Safety
/// `item.xCC_item_attr` is valid.
pub unsafe fn it_80277040(item: &mut Item, common: &ItemCommonData) -> bool {
    let mut ret_val = true;
    let attr = &*item.xCC_item_attr;
    let x50 = attr.x50;
    if (if x50 < 0.0 { -x50 } else { x50 }) < 0.00001 {
        item.x88.y = 0.0;
        item.x88.x = 0.0;
        return false;
    }
    let up = RVec { x: 0.0, y: 1.0, z: 0.0 };
    let coll = &item.x378_itemColl;
    let mut n = zero();
    if coll.env_flags & Collide_LeftWallMask != 0 {
        n = coll.left_facing_wall.normal;
    }
    if coll.env_flags & Collide_RightWallMask != 0 {
        n = coll.right_facing_wall.normal;
    }
    if coll.env_flags & COLLIDE_CEILING_MASK != 0 {
        n = coll.ceiling.normal;
    }
    if coll.env_flags & COLLIDE_FLOOR_MASK != 0 {
        n = coll.floor.normal;
    }
    item.x94 = item.x88;
    let n = rv(&n);
    let mut angle1 = lbvector::angle_xy(&n, &up);
    if angle1 < 0.0 {
        angle1 = -angle1;
    }
    if angle1 >= common.xC0 {
        let down = RVec { x: 0.0, y: -1.0, z: 0.0 };
        let sp44 = lbvector::crossprod_normalized(&n, &down);
        let sp38 = lbvector::crossprod_normalized(&sp44, &n);
        let t3 = attr.x50;
        let t31 = item.x94.x / t3;
        let t30 = item.x94.y / t3;
        item.x88.x = sp38.x * t3;
        item.x88.y = sp38.y * attr.x50;
        let mut sp14 = zero();
        it_80276408(&item.x378_itemColl, &mut sp14);
        let mut angle2 = lbvector::angle(&rv(&sp14), &rv(&item.xAC_unk));
        if angle2 < 0.0 {
            angle2 = -angle2;
        }
        let chk = angle2 >= common.xC4 && item.xC30 == item.xC2C;
        if !chk {
            if item.x94.x != 0.0 || item.x94.y != 0.0 {
                let sgn = |f: f32| if f < 0.0 { -1 } else { 1 };
                let mismatch = sgn(t31) != sgn(sp38.x) || sgn(t30) != sgn(sp38.y);
                if mismatch {
                    item.xD58 = item.xD58.wrapping_add(1);
                    if item.xD58 >= common.x38_float {
                        item.xD58 = 0;
                        ret_val = false;
                        item.x88.y = 0.0;
                        item.x88.x = 0.0;
                    }
                }
            }
        } else {
            item.xD5C = 2;
        }
    } else {
        ret_val = false;
        item.x88.y = 0.0;
        item.x88.x = 0.0;
    }
    ret_val
}

/// `it_8027770C`: bounce off a touched wall (mirror the velocity if moving into it), then scale
/// the velocity by `attr.x58`. 2 fused sites in the dot product.
/// # Safety
/// `item.xCC_item_attr` is valid.
pub unsafe fn it_8027770c(item: &mut Item) -> bool {
    let coll = &item.x378_itemColl;
    let mut var_r5 = 0;
    let mut n = zero();
    if coll.env_flags & Collide_LeftWallMask != 0 {
        var_r5 = 8;
        item.xC30 = coll.left_facing_wall.index;
        n = coll.left_facing_wall.normal;
    }
    let coll = &item.x378_itemColl;
    if coll.env_flags & Collide_RightWallMask != 0 {
        var_r5 = 4;
        item.xC30 = coll.right_facing_wall.index;
        n = coll.right_facing_wall.normal;
    }
    if var_r5 == 0 {
        return false;
    }
    let v = item.x40_vel;
    if fmadds(v.z, n.z, fmadds(v.x, n.x, v.y * n.y)) < 0.0 {
        mirror(&mut item.x40_vel, &n);
    }
    let mult = (*item.xCC_item_attr).x58;
    item.x40_vel.x *= mult;
    item.x40_vel.y *= mult;
    item.x40_vel.z *= mult;
    true
}

// ---- itzako.c --------------------------------------------------------------------------

/// `it_8027B798`: launch velocity from knockback; true when on the ground and the launch points
/// away from the floor. `cd` is `p_ftCommonData`.
pub fn it_8027b798(ip: &mut Item, out_vel: &mut GcVec, cd: &ftCommonData) -> bool {
    let knockback = ip.xCC8_knockback;
    let mag = knockback * cd.x100;
    let angle = it_8027b798_calc_angle(ip.ground_or_air, ip.xCAC_angle, cd, knockback);
    let dx = mag * cosf(angle);
    let dy = mag * sinf(angle);
    ip.facing_dir = ip.xCCC_incDamageDirection;
    if ip.ground_or_air == GA_Air {
        out_vel.x = -dx * ip.facing_dir;
        out_vel.y = dy;
        out_vel.z = 0.0;
        return false;
    }
    let normal = ip.x378_itemColl.floor.normal;
    let v = GcVec { x: -dx * ip.facing_dir, y: dy, z: 0.0 };
    if lbvector::angle(&rv(&normal), &rv(&v)) < 1.5707964 {
        out_vel.x = v.x;
        out_vel.y = v.y;
        out_vel.z = 0.0;
        true
    } else {
        let x = v.x;
        out_vel.x = normal.y * x;
        out_vel.y = -normal.x * x;
        out_vel.z = 0.0;
        false
    }
}

/// `it_8027BB1C`: rotate `v` by the Euler angles `a` (x, then y, then z). 6 fused sites.
/// The C IR is `fmuladd(a, c, fneg(b * s))`; the optimiser folds the add of a negation into a
/// subtraction, which keeps the NaN sign of `b * s` (x86 default NaN is negative), so the three
/// subtracting sites are written as `fmsubs` (same rounding, same NaN as the optimised C).
pub fn it_8027bb1c(v: &mut GcVec, a: &GcVec) {
    let (c, s) = (cosf(a.x), sinf(a.x));
    let (t1, t2) = (v.y, v.z);
    v.y = fmsubs(t1, c, t2 * s);
    v.z = fmadds(t1, s, t2 * c);
    let (c, s) = (cosf(a.y), sinf(a.y));
    let (t1, t2) = (v.x, v.z);
    v.x = fmadds(t1, c, t2 * s);
    v.z = fmsubs(t2, c, t1 * s);
    let (c, s) = (cosf(a.z), sinf(a.z));
    let t1 = fmsubs(v.x, c, v.y * s);
    let t2 = fmadds(v.x, s, v.y * c);
    v.x = t1;
    v.y = t2;
}

// ---- it_2E5A.c -------------------------------------------------------------------------

/// `it_802E628C`: random launch velocity; extra x push when `arg8` is in [pi/3, 2pi/3] or
/// [4pi/3, 5pi/3] (bounds in f32 from `(f32)M_PI`). 1 fused site.
/// # Safety
/// The item's special attributes are an `it_2E5A_Attrs`.
pub unsafe fn it_802e628c(rand: &mut HsdRand, item: &mut Item, arg8: f32, arg9: f32) {
    let attr = &*((*item.xC4_article_data).x4_specialAttributes as *const it_2E5A_Attrs);
    let t31 = rand.randf();
    let t1 = (arg8 as f64 - M_PI_2) as f32;
    let pi = M_PI as f32;
    let mut v = t31 * attr.xC;
    v *= item.facing_dir;
    if (arg8 >= pi / 3.0 && arg8 <= 2.0 * (pi / 3.0))
        || (arg8 >= 4.0 * (pi / 3.0) && arg8 <= 5.0 * (pi / 3.0))
    {
        v = fmadds(attr.x10, cosf(t1), v);
    }
    item.x40_vel.x = v;
    let mut t0 = t31 * attr.x20;
    t0 += arg9;
    item.x40_vel.y = attr.x18 + t0;
    item.x40_vel.z = 0.0;
}

// ---- itbox.c ---------------------------------------------------------------------------

/// `it_802863BC`: on a floor flatter than the bounce threshold, zero the four velocity
/// accumulators and set `xD5C`.
/// # Safety
/// The item's special attributes are an `itBoxAttributes`.
pub unsafe fn it_802863bc(ip: &mut Item) -> bool {
    let attr = &*((*ip.xC4_article_data).x4_specialAttributes as *const itBoxAttributes);
    if ip.x378_itemColl.env_flags & COLLIDE_FLOOR_MASK != 0 {
        let mut dir = zero();
        it_80276408(&ip.x378_itemColl, &mut dir);
        let unit = RVec { x: 0.0, y: 1.0, z: 0.0 };
        if lbvector::angle_xy(&rv(&dir), &unit) < attr.bounce_angle_threshold {
            ip.x7C = zero();
            ip.x88 = zero();
            ip.x94 = zero();
            ip.xA0 = zero();
            ip.xD5C = 1;
            return true;
        }
    }
    false
}

// ---- itflipper.c -----------------------------------------------------------------------

/// `itFlipper_Repel_inline`: push a Flipper away from `pos` (0.2 along the xy direction).
pub fn it_flipper_repel_inline(ip: &mut Item, kind: i32, pos: &GcVec, vec: &mut GcVec) {
    *vec = zero();
    if kind == It_Kind_Flipper as i32 {
        vec.x = ip.pos.x - pos.x;
        vec.y = ip.pos.y - pos.y;
        let mut r = rv(vec);
        lbvector::normalize_xy(&mut r);
        *vec = gv(&r);
        vec.x *= 0.2;
        vec.y *= 0.2;
        ip.x40_vel.x += vec.x;
        ip.x40_vel.y += vec.y;
    }
}

// ---- itfoxblaster.c --------------------------------------------------------------------

/// `itFoxBlaster_SetShotAngle`.
pub fn it_fox_blaster_set_shot_angle(item: &mut Item, fighter_muzzle: &GcVec, item_muzzle: &GcVec) {
    // SAFETY: the item is a Fox blaster (plain-data union member).
    let v = unsafe { &mut item.xDD4_itemVar.foxblaster };
    v.angle[0] = atan2f(fighter_muzzle.y - item_muzzle.y, fighter_muzzle.x - item_muzzle.x);
    v.xDFC[0] = 0;
}

// ---- ithammer.c ------------------------------------------------------------------------

/// `it_80293DCC`: with probability `x8`%, a random value in [x0, x4).
/// # Safety
/// The item's special attributes are an `itHammerData`.
pub unsafe fn it_80293dcc(rand: &mut HsdRand, item: &Item) -> i32 {
    let d = &*((*item.xC4_article_data).x4_specialAttributes as *const itHammerData);
    if rand.randf() < d.x8 / 100.0 {
        let rng = rand.randi(d.x4.wrapping_sub(d.x0) as i32);
        return (rng as u32).wrapping_add(d.x0) as i32;
    }
    0
}

// ---- ithinoarashi.c --------------------------------------------------------------------

/// `it_802D61C8`: wrap `x78` into [-pi, pi] (f64 steps), sum the touched surfaces' normals
/// (`coll` bits 1 floor, 2 ceiling, 8 left wall, 4 right wall), normalise, and store the vector
/// over the 12 bytes at item-var +0x60.
pub fn it_802d61c8(ip: &mut Item, coll: i32) {
    let mut n = zero();
    // SAFETY: the item is a hinoarashi (plain-data union member).
    unsafe {
        let h = &mut ip.xDD4_itemVar.hinoarashi;
        while (h.x78 as f64) < -M_PI {
            h.x78 = (h.x78 as f64 + 2.0 * M_PI) as f32;
        }
        while h.x78 as f64 > M_PI {
            h.x78 = (h.x78 as f64 - 2.0 * M_PI) as f32;
        }
    }
    let c = &ip.x378_itemColl;
    if coll & 1 != 0 {
        n.x += c.floor.normal.x;
        n.y += c.floor.normal.y;
    }
    if coll & 2 != 0 {
        n.x += c.ceiling.normal.x;
        n.y += c.ceiling.normal.y;
    }
    if coll & 8 != 0 {
        n.x += c.left_facing_wall.normal.x;
        n.y += c.left_facing_wall.normal.y;
    }
    if coll & 4 != 0 {
        n.x += c.right_facing_wall.normal.x;
        n.y += c.right_facing_wall.normal.y;
    }
    normalize(&mut n);
    // SAFETY: bytes +0x60..+0x6C of the 504-byte union (x60 flags, x61 pad, x64, x68).
    unsafe {
        let p = (&mut ip.xDD4_itemVar.hinoarashi.x60 as *mut _ as *mut u8) as *mut GcVec;
        p.write_unaligned(n);
    }
}

// ---- ithitodeman.c ---------------------------------------------------------------------

/// `it_802D4494`: `x84 = x18 + randi((s32)(x14 - x18))` (the float-to-int conversion saturates,
/// NaN -> 0, as the build's lowering does).
/// # Safety
/// The item's special attributes are an `itHitodemanAttributes`.
pub unsafe fn it_802d4494(rand: &mut HsdRand, ip: &mut Item) {
    let a = &*((*ip.xC4_article_data).x4_specialAttributes as *const itHitodemanAttributes);
    let r = rand.randi((a.x14 - a.x18) as i32);
    ip.xDD4_itemVar.hitodeman.x84 = a.x18 + r as f32;
}

/// `it_802D4510`: `x88 = x44 + randi(x40 - x44)`.
/// # Safety
/// The item's special attributes are an `itHitodemanAttributes`.
pub unsafe fn it_802d4510(rand: &mut HsdRand, ip: &mut Item) {
    let a = &*((*ip.xC4_article_data).x4_specialAttributes as *const itHitodemanAttributes);
    ip.xDD4_itemVar.hitodeman.x88 = a.x44.wrapping_add(rand.randi(a.x40.wrapping_sub(a.x44)));
}

// ---- itkoopaflame.c / itlgunbeam.c -----------------------------------------------------

/// `itKoopaFlame_UnkMotion0_Phys`.
pub fn it_koopa_flame_unk_motion0_phys(it: &mut Item) {
    // SAFETY: the item is a Koopa flame (plain-data union member).
    let k = unsafe { &mut it.xDD4_itemVar.koopaflame };
    k.x0_pos = it.pos;
    let x = k.x28_speed * sinf(k.x24_angle);
    it.x40_vel.x = x;
    k.x18_vel.x = x;
    let y = k.x28_speed * cosf(k.x24_angle);
    it.x40_vel.y = y;
    k.x18_vel.y = y;
    it.x40_vel.z = 0.0;
    k.x18_vel.z = 0.0;
    normalize(&mut k.x18_vel);
}

/// `itLgunbeam_UnkMotion0_Phys`.
pub fn it_lgunbeam_unk_motion0_phys(ip: &mut Item) {
    // SAFETY: the item is a Ray Gun beam (plain-data union member).
    let b = unsafe { &mut ip.xDD4_itemVar.lgunbeam };
    b.position0 = ip.pos;
    let x = b.angle1 * sinf(b.angle0);
    ip.x40_vel.x = x;
    b.velocity.x = x;
    let y = b.angle1 * cosf(b.angle0);
    ip.x40_vel.y = y;
    b.velocity.y = y;
    ip.x40_vel.z = 0.0;
    b.velocity.z = 0.0;
    normalize(&mut b.velocity);
}

// ---- itlinkboomerang.c / itmewtwoshadowball.c ------------------------------------------

/// `it_802A2320`: mirror the velocity off `xC58`, re-derive the boomerang's angle.
pub fn it_802a2320(ip: &mut Item) -> bool {
    let m = ip.xC58;
    mirror(&mut ip.x40_vel, &m);
    // SAFETY: the item is a Link boomerang (plain-data union member).
    unsafe {
        let b = &mut ip.xDD4_itemVar.linkboomerang;
        b.xF74 = atan2f(ip.x40_vel.y, ip.x40_vel.x);
        clamp_tau(&mut b.xF74);
    }
    norm_xf74(ip);
    false
}

/// `it_2725_Logic101_ShieldBounced` (itmewtwoshadowball.c).
pub fn it_2725_logic101_shield_bounced(ip: &mut Item) -> bool {
    let m = ip.xC58;
    mirror(&mut ip.x40_vel, &m);
    // SAFETY: the item is a Shadow Ball (plain-data union member).
    unsafe {
        let s = &mut ip.xDD4_itemVar.mewtwoshadowball;
        s.x4.x = atan2f(ip.x40_vel.y, ip.x40_vel.x);
        while s.x4.x < 0.0 {
            s.x4.x = (s.x4.x as f64 + M_TAU) as f32;
        }
        while s.x4.x as f64 > M_TAU {
            s.x4.x = (s.x4.x as f64 - M_TAU) as f32;
        }
    }
    false
}

// ---- itmaril.c -------------------------------------------------------------------------

/// `it_802D6740`: gravity, then `x6C` = the normalised velocity.
/// # Safety
/// The item's special attributes are an `itMarilAttributes`.
pub unsafe fn it_802d6740(ip: &mut Item) {
    let a = &*((*ip.xC4_article_data).x4_specialAttributes as *const itMarilAttributes);
    ip.x40_vel.y += a.xC;
    let m = &mut ip.xDD4_itemVar.maril;
    m.x6C = ip.x40_vel;
    normalize(&mut m.x6C);
}

// ---- itpikachutjoltground.c / itsamuschargeshot.c --------------------------------------

/// `itPikachutjoltground_UnkMotion0_Phys`.
/// # Safety
/// The item's special attributes are an `itPikachutJoltGroundAttributes`.
pub unsafe fn it_pikachutjoltground_unk_motion0_phys(ip: &mut Item) {
    let a = &*((*ip.xC4_article_data).x4_specialAttributes as *const itPikachutJoltGroundAttributes);
    let ang = ip.xDD4_itemVar.pikachujoltground.xDD4;
    ip.x40_vel.x = a.x8 * cosf(ang);
    ip.x40_vel.y = a.x8 * sinf(ang);
}

/// `itSamuschargeshot_UnkMotion8_Phys`.
pub fn it_samuschargeshot_unk_motion8_phys(ip: &mut Item) {
    // SAFETY: the item is a charge shot (plain-data union member).
    let s = unsafe { ip.xDD4_itemVar.samuschargeshot };
    ip.x40_vel.x = s.xDDC * cosf(s.xDD8);
    ip.x40_vel.y = s.xDDC * sinf(s.xDD8);
}

// ---- ittincle.c ------------------------------------------------------------------------

/// `it_802EC870`: turn around (`arg1 != 0`), or pick a direction (1 in 3 reversed, clamped to
/// the area `[xC, x10]`) and a random speed. `x24 / 2` is an integer division.
/// # Safety
/// The item's special attributes are an `itTincleAttributes`.
pub unsafe fn it_802ec870(rand: &mut HsdRand, ip: &mut Item, arg1: i32) {
    let sa = &*((*ip.xC4_article_data).x4_specialAttributes as *const itTincleAttributes);
    if arg1 != 0 {
        ip.facing_dir = -ip.facing_dir;
        ip.x40_vel.x *= -1.0;
        ip.xDD4_itemVar.tincle.x34 *= -1.0;
        return;
    }
    ip.facing_dir = if ip.pos.x < ip.xDD4_itemVar.tincle.x58 { 1.0 } else { -1.0 };
    if rand.randi(3) == 0 {
        ip.facing_dir *= -1.0;
    }
    if ip.pos.x < sa.xC {
        ip.facing_dir = 1.0;
    }
    if ip.pos.x > sa.x10 {
        ip.facing_dir = -1.0;
    }
    let d = sa.x38 - sa.x34;
    let abs = if d < 0.0 { -(sa.x38 - sa.x34) } else { sa.x38 - sa.x34 };
    ip.x40_vel.x = abs * rand.randf();
    let t = &mut ip.xDD4_itemVar.tincle;
    t.x24 = sa.x3C;
    t.x34 = (sa.x38 - ip.x40_vel.x) / (t.x24.wrapping_div(2) as f32);
    ip.x40_vel.x *= ip.facing_dir;
    t.x34 *= ip.facing_dir;
    t.x28 = 0;
}
