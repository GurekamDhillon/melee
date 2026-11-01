
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
pub(crate) fn rv(v: &GcVec) -> RVec {
    RVec { x: v.x, y: v.y, z: v.z }
}

#[inline(always)]
pub(crate) fn gv(v: &RVec) -> GcVec {
    GcVec { x: v.x, y: v.y, z: v.z }
}

#[inline(always)]
pub(crate) fn normalize(v: &mut GcVec) -> f32 {
    let mut r = rv(v);
    let l = lbvector::normalize(&mut r);
    *v = gv(&r);
    l
}

#[inline(always)]
pub(crate) fn mirror(v: &mut GcVec, m: &GcVec) {
    let mut r = rv(v);
    lbvector::mirror(&mut r, &rv(m));
    *v = gv(&r);
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

pub use crate::tu::itcoll::it_80271830_fp as it_80271830;
pub use crate::tu::itmaplib::it_8027649c;
pub use crate::tu::itmaplib::it_8027649c_ref;
pub use crate::tu::itmaplib::it_80277040_fp as it_80277040;
pub use crate::tu::itmaplib::it_8027770c;
pub use crate::tu::itzako::it_8027b798;
pub use crate::tu::itzako::it_8027bb1c;
pub use crate::tu::itbox::it_802863bc;
pub use crate::tu::ithammer::it_80293dcc;
pub use crate::tu::itlinkboomerang::it_802a2320;
pub use crate::tu::ithitodeman::it_802d4494;
pub use crate::tu::ithitodeman::it_802d4510;
pub use crate::tu::ithinoarashi::it_802d61c8;
pub use crate::tu::itmaril::it_802d6740;
pub use crate::tu::it_2E5A::it_802e628c;
pub use crate::tu::ittincle::it_802ec870;
pub use crate::tu::grcastle::zero_fp as zero;
