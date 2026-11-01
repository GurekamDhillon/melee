
//! Environment collision, from src/melee/mp/mpcoll.c (and mplib.c's `mpLibLoad` /
//! `mpPruneEmptyLines`), over the ported mplib queries ([`GroundColl`]).
//!
//! Ported (each verified by the native golden run against the C build's per-step ECB trace, sim
//! round 2): mpColl_80041EE4 (CollDataInit), mpColl_SetECBSource_JObj, mpColl_SetLedgeSnap,
//! mpCollPrev, mpCollCheckBounding, mpColl_80042384, mpColl_LoadECB_JObj, mpColl_80042C58,
//! the `mpColl_LoadECB_inline` lock wrapper, mpCollInterpolateECB, mpColl_80043754 (the
//! substep loop), mpCollEnd, mpColl_80043BBC / 80043E90 (wall-at-floor-end probes),
//! mpColl_80044164 / 800443C4 (ledge grabs), mpColl_80044628_Floor, mpColl_80044838_Floor,
//! mpColl_80044948_Floor, mpColl_80044AD8_Ceiling, mpColl_80044C74_Ceiling,
//! mpColl_80044E10_RightWall, mpColl_80045B74_LeftWall (the wall *checks*), mpColl_80046904
//! (the air/ground step callback) and its entry points mpColl_800471F8 / 8004730C / 80047E14.
//!
//! Not ported, reached only when a wall / ceiling squeeze actually happens (they return
//! `Err(NotPorted)`, the TODO-panic convention, so a scenario that needs them stops instead of
//! diverging): mpColl_800454A4_RightWall / 80046224_LeftWall (wall push-out), mpColl_80043C6C /
//! 80043F40 (floor-end wall slide), mpCollSqueezeHorizontal / Vertical, mpCollCeilingInline,
//! mpColl_LoadECB_Fixed. Also not modelled: the stage joint callbacks `mpJointGetCb1/2` that
//! mpCollEnd / mpColl_80043268 call (no stage wired so far registers one) and
//! `grDynamicAttr_801CA284` (no dynamic attribute areas on the wired stages: 0).
//!
//! The C file statics (`mpColl_IsEcbTiny`, the wall id lists `mpColl_80458810`, their counts,
//! the `mpColl_804D6490..98` best-wall scratch and `mpColl_804D64AC`, the line-moved counter)
//! are [`CollGlobals`]. `mpColl_804D64A0/A4` (the floor callback and its gobj) are the `floor_cb`
//! argument. The ECB joints' world positions (`lb_8000B1CC` on `ecb_source.x10C_joint[i]`) come
//! from the caller, which owns the JObj tree.
//!
//! FMA (`python tools/rust-port/fma_sites.py src/melee/mp/mpcoll.c`): of the functions here only
//! `Vec2_Interpolate` (2 sites, in mpCollInterpolateECB) fuses; mpColl_LoadECB_Fixed (4),
//! 800439FC/80043ADC (2), 80043C6C/80043F40 (4), the wall push-outs (5/4) are not ported.
//! The TARGET_PC `half` statement in mpColl_LoadECB_JObj (one shared `0.5F * (b + t)`) is kept.

use super::check::LineFilter;

use super::lines::GroundColl;

use super::types::*;

use crate::vec::Vec3;

use melee_core::fp::fmadds;

use melee_types::{CollData, Vec as TVec, _ftECB, ftCollisionBox};

pub const COLLIDE_LEFT_WALL_PUSH: i32 = 0x1;

pub const COLLIDE_LEFT_WALL_HUG: i32 = 0x20;

pub const COLLIDE_LEFT_WALL_MASK: i32 = 0x3F;

pub const COLLIDE_RIGHT_WALL_PUSH: i32 = 0x40;

pub const COLLIDE_RIGHT_WALL_HUG: i32 = 0x800;

pub const COLLIDE_RIGHT_WALL_MASK: i32 = 0xFC0;

pub const COLLIDE_CEILING_PUSH: i32 = 0x2000;

pub const COLLIDE_CEILING_HUG: i32 = 0x4000;

pub const COLLIDE_FLOOR_PUSH: i32 = 0x8000;

pub const COLLIDE_FLOOR_HUG: i32 = 0x10000;

pub const COLLIDE_LEFT_EDGE: i32 = 0x100000;

pub const COLLIDE_RIGHT_EDGE: i32 = 0x200000;

pub const COLLIDE_EDGE: i32 = 0x800000;

pub const COLLIDE_LEFT_LEDGE_GRAB: i32 = 0x1000000;

pub const COLLIDE_RIGHT_LEDGE_GRAB: i32 = 0x2000000;

pub const COLLIDE_LEFT_LEDGE_SLIP: i32 = 0x10000000;

pub const COLLIDE_RIGHT_LEDGE_SLIP: i32 = 0x20000000;

pub const COLL_FLAG_AIR_STAY_AIRBORNE: u32 = 0x1;

pub const COLL_FLAG_AIR_PLATFORM_PASS_CALLBACK: u32 = 0x2;

pub const COLL_FLAG_AIR_CAN_GRAB_LEDGE: u32 = 0x4;

pub const COLLDATA_X130_LOCKED: u32 = 1 << 4;

pub const COLLDATA_X130_CLEAR: u32 = 1 << 5;

/// `ECBSource_JObj` / `ECBSource_Fixed` (mp/types.h).
pub const ECB_SOURCE_JOBJ: i32 = 1;

pub const ECB_SOURCE_FIXED: i32 = 2;

pub(crate) const MPCOLL_WALLID_MAX: usize = 9;

pub(crate) const F32_MAX: f32 = 3.4028235e38;

/// Code the step needs that is not ported (the scenario stops there).
#[derive(Clone, Debug, PartialEq)]
pub struct NotPorted(pub &'static str);

pub type R<T> = Result<T, NotPorted>;

/// mpcoll.c's file statics.
#[derive(Clone, Debug, Default)]
pub struct CollGlobals {
    /// `mpColl_804D64AC` (incremented by Ground when lines move; compared with `coll->x38`).
    pub x804d64ac: i32,
    /// `mpColl_IsEcbTiny`.
    pub is_ecb_tiny: bool,
    /// `mpColl_80458810.right` / `.left` and `mpColl_804D6488` / `804D648C`.
    right: [i32; MPCOLL_WALLID_MAX],
    n_right: usize,
    left: [i32; MPCOLL_WALLID_MAX],
    n_left: usize,
}

/// Which `mpCollCallback` mpColl_80043754 runs each substep.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StepCb {
    /// `mpColl_80046904`.
    Air,
    /// `mpColl_8004ACE4`.
    Ground,
}

#[inline]
pub(crate) fn v3(v: TVec) -> Vec3 {
    Vec3 { x: v.x, y: v.y, z: v.z }
}

#[inline]
pub(crate) fn tv(v: Vec3) -> TVec {
    TVec { x: v.x, y: v.y, z: v.z }
}

#[inline]
pub(crate) fn abs(x: f32) -> f32 {
    // C ABS(x): x < 0 ? -x : x (keeps -0.0).
    if x < 0.0 {
        -x
    } else {
        x
    }
}

pub(crate) fn zero_ecb() -> _ftECB {
    _ftECB::zeroed()
}

/// `mpLibLoad(coll_data)` (+ `mpPruneEmptyLines`, not Pura): builds [`GroundColl`] from the
/// stage DAT's `coll_data` (`MapCollData`, GC offsets in [`MapCollData`]'s layout) and
/// `Ground_801C0498()` (the stage scale, `grGroundParam->y`). Joints get no JObj, no callbacks.
pub fn mp_lib_load(d: &[u8], coll_data: u32, scale: f32) -> GroundColl {
    let be32 = |at: u32| u32::from_be_bytes(d[at as usize..at as usize + 4].try_into().unwrap());
    let be16 = |at: u32| u16::from_be_bytes(d[at as usize..at as usize + 2].try_into().unwrap());
    let bef = |at: u32| f32::from_bits(be32(at));
    let (verts, vert_count) = (be32(coll_data), be32(coll_data + 4) as i32);
    let (lines_at, line_count) = (be32(coll_data + 8), be32(coll_data + 0xC) as i32);
    let mut ranges = [MapLineRange::default(); MAP_LINE_GROUP_COUNT];
    for (g, r) in ranges.iter_mut().enumerate() {
        let at = coll_data + 0x10 + 4 * g as u32;
        *r = MapLineRange { start: be16(at) as i16, count: be16(at + 2) as i16 };
    }
    let (joints_at, joint_count) = (be32(coll_data + 0x24), be32(coll_data + 0x28) as i32);
    let mut map_lines: Vec<MapLine> = (0..line_count as u32)
        .map(|i| {
            let a = lines_at + 0x10 * i;
            MapLine {
                v0_idx: be16(a),
                v1_idx: be16(a + 2),
                prev_id0: be16(a + 4) as i16,
                next_id0: be16(a + 6) as i16,
                prev_id1: be16(a + 8) as i16,
                next_id1: be16(a + 0xA) as i16,
                hi_flags: be16(a + 0xC),
                lo_flags: be16(a + 0xE),
            }
        })
        .collect();
    let vxy: Vec<(f32, f32)> = (0..vert_count as u32).map(|i| (bef(verts + 8 * i), bef(verts + 8 * i + 4))).collect();
    let map_joints: Vec<MapJoint> = (0..joint_count as u32)
        .map(|i| {
            let a = joints_at + 0x28 * i;
            let mut r = [MapLineRange::default(); MAP_LINE_GROUP_COUNT];
            for (g, x) in r.iter_mut().enumerate() {
                let at = a + 4 * g as u32;
                *x = MapLineRange { start: be16(at) as i16, count: be16(at + 2) as i16 };
            }
            MapJoint {
                ranges: r,
                left_bound: bef(a + 0x14),
                bottom_bound: bef(a + 0x18),
                right_bound: bef(a + 0x1C),
                top_bound: bef(a + 0x20),
                vtx_start: be16(a + 0x24) as i16,
                vtx_count: be16(a + 0x26) as i16,
            }
        })
        .collect();

    let mut g = GroundColl::default();
    g.coll_box[0] = MpCollisionBox { top: F32_MAX, bottom: -F32_MAX, left: -F32_MAX, right: F32_MAX };
    g.joints = (0..joint_count)
        .map(|i| {
            let mj = &map_joints[i as usize];
            super::check::CollJointRt {
                next: if i + 1 < joint_count { i + 1 } else { -1 },
                inner: i,
                flags: COLL_JOINT_ENABLED,
                x_e: true,
                bounding_min: crate::vec::Vec2 { x: scale * mj.left_bound, y: scale * mj.bottom_bound },
                bounding_max: crate::vec::Vec2 { x: scale * mj.right_bound, y: scale * mj.top_bound },
            }
        })
        .collect();
    g.joint_list_start = if joint_count > 0 { 0 } else { -1 };
    g.joint_list_end = joint_count - 1;
    // mpPruneEmptyLines.
    for i in 0..line_count as usize {
        let l = map_lines[i];
        let (v0, v1) = (vxy[l.v0_idx as usize], vxy[l.v1_idx as usize]);
        if v0.0 != v1.0 || v0.1 != v1.1 {
            continue;
        }
        let ii = i as i16;
        for o in map_lines.iter_mut() {
            if o.prev_id0 == ii {
                o.prev_id0 = l.prev_id0;
            }
            if o.next_id0 == ii {
                o.next_id0 = l.next_id0;
            }
            if o.prev_id1 == ii {
                o.prev_id1 = l.prev_id0;
            }
            if o.next_id1 == ii {
                o.next_id1 = l.next_id0;
            }
        }
        let l = &mut map_lines[i];
        l.hi_flags |= LINE_FLAG_EMPTY as u16;
        l.prev_id0 = -1;
        l.next_id0 = -1;
        l.prev_id1 = -1;
        l.next_id1 = -1;
    }
    g.lines = vec![CollLine::default(); line_count as usize];
    for r in ranges {
        let mut s = r.start as i32;
        let mut c = r.count as i32;
        while c > 0 {
            g.lines[s as usize] = CollLine { x0: s as u32, flags: map_lines[s as usize].hi_flags as u32 | LINE_FLAG_ENABLED };
            s += 1;
            c -= 1;
        }
    }
    g.vtx = vxy
        .iter()
        .map(|&(x, y)| {
            let (f2, f1) = (scale * x, scale * y);
            if g.coll_box[0].top < f1 {
                g.coll_box[0].top = f1;
            }
            if g.coll_box[0].bottom > f1 {
                g.coll_box[0].bottom = f1;
            }
            if g.coll_box[0].right < f2 {
                g.coll_box[0].right = f2;
            }
            if g.coll_box[0].left > f2 {
                g.coll_box[0].left = f2;
            }
            CollVtx { x0: x, x4: y, pos: crate::vec::Vec2 { x: f2, y: f1 }, x10: f2, x14: f1 }
        })
        .collect();
    g.map_lines = map_lines;
    g.line_count = line_count;
    g.map_joints = map_joints;
    g.map_ranges = ranges;
    g.mp_island_8005a728();
    g.mp_uncheck_bounding();
    g
}

/// `mpColl_80041EE4` (CollDataInit).
pub fn mp_coll_80041ee4(cd: &mut CollData, g: &CollGlobals) {
    cd.x0_gobj = core::ptr::null_mut();
    cd.x34_flags.set_b0(1);
    cd.x34_flags.set_b6(0);
    cd.x34_flags.set_b7(0);
    cd.x35_flags.set_b0(1);
    cd.x34_flags.set_b1234(0);
    cd.env_flags = 0;
    cd.x130_flags = 0;
    cd.prev_pos = cd.cur_pos;
    cd.last_pos = cd.cur_pos;
    cd.x28_vec = cd.cur_pos;
    cd.floor_skip = -1;
    cd.ledge_id_right = -1;
    cd.ledge_id_left = -1;
    cd.floor.index = -1;
    cd.floor.flags = 0;
    cd.floor.normal = TVec { x: 0.0, y: 1.0, z: 0.0 };
    cd.ceiling.index = -1;
    cd.ceiling.flags = 0;
    cd.ceiling.normal = TVec { x: 0.0, y: -1.0, z: 0.0 };
    cd.right_facing_wall.index = -1;
    cd.right_facing_wall.flags = 0;
    cd.right_facing_wall.normal = TVec { x: 0.0, y: 1.0, z: 0.0 };
    cd.left_facing_wall.index = -1;
    cd.left_facing_wall.flags = 0;
    cd.left_facing_wall.normal = TVec { x: 0.0, y: -1.0, z: 0.0 };
    cd.x38 = g.x804d64ac;
    cd.x50 = 0.0;
    cd.joint_id_skip = -1;
    cd.joint_id_only = -1;
    cd.ledge_snap_x = 0.0;
    cd.ledge_snap_y = 0.0;
    cd.ledge_snap_height = 0.0;
    cd.ecb = zero_ecb();
    cd.prev_ecb = zero_ecb();
    cd.xE4_ecb = zero_ecb();
    cd.ecb_source = melee_types::ECBSource::zeroed();
    cd.desired_ecb = zero_ecb();
    cd.x64_ecb = zero_ecb();
}

/// `mpColl_SetECBSource_JObj` minus the joint pointers (the caller keeps the six ECB parts).
pub fn mp_coll_set_ecb_source_jobj(cd: &mut CollData, x124: f32) {
    cd.ecb_source.kind = ECB_SOURCE_JOBJ as _;
    cd.ecb_source.x124 = x124;
    cd.ecb_source.x128 = 10.0;
    cd.ecb_source.x12C = 10.0;
    if cd.x34_flags.b0() != 0 {
        cd.ecb.top.x = 0.0;
        cd.ecb.top.y = 8.0;
        cd.ecb.bottom.x = 0.0;
        cd.ecb.bottom.y = 0.0;
        cd.ecb.right.x = 4.0;
        cd.ecb.right.y = 4.0;
        cd.ecb.left.x = -4.0;
        cd.ecb.left.y = 4.0;
        cd.prev_ecb = cd.ecb;
        cd.xE4_ecb = cd.ecb;
        cd.x64_ecb = cd.ecb;
        cd.desired_ecb = cd.ecb;
    }
    cd.facing_dir = -1;
    cd.x50 = 0.0;
}

/// `mpColl_SetLedgeSnap`.
pub fn mp_coll_set_ledge_snap(cd: &mut CollData, x: f32, y: f32, h: f32) {
    cd.ledge_snap_x = x;
    cd.ledge_snap_y = y;
    cd.ledge_snap_height = h;
}

/// `mpCollSetFacingDir`.
pub fn mp_coll_set_facing_dir(cd: &mut CollData, dir: i32) {
    cd.facing_dir = dir as i16;
}

/// `mpCollPrev` (the DbLevel range assert is not run: no debug ROM).
pub fn mp_coll_prev(cd: &mut CollData) {
    cd.x28_vec = cd.cur_pos;
}

/// `mpCollCheckBounding`.
pub fn mp_coll_check_bounding(g: &mut GroundColl, cd: &CollData, flags: u32) {
    let mut left = cd.ecb.left.x + cd.cur_pos.x;
    let v = cd.prev_ecb.left.x + cd.prev_pos.x;
    if left > v {
        left = v;
    }
    let mut right = cd.ecb.right.x + cd.cur_pos.x;
    let v = cd.prev_ecb.right.x + cd.prev_pos.x;
    if right < v {
        right = v;
    }
    let mut bottom = cd.ecb.bottom.y + cd.cur_pos.y;
    let v = cd.prev_ecb.bottom.y + cd.prev_pos.y;
    if bottom > v {
        bottom = v;
    }
    let mut top = cd.ecb.top.y + cd.cur_pos.y;
    let v = cd.prev_ecb.top.y + cd.prev_pos.y;
    if top < v {
        top = v;
    }
    if flags & COLL_FLAG_AIR_CAN_GRAB_LEDGE != 0 {
        let ledge_snap_x = cd.ledge_snap_x;
        let half_height = 0.5f32 * cd.ledge_snap_height;
        right += ledge_snap_x;
        left -= ledge_snap_x;
        let v = cd.ledge_snap_y - half_height + cd.cur_pos.y;
        if bottom > v {
            bottom = v;
        }
        let v = cd.ledge_snap_y - half_height + cd.prev_pos.y;
        if bottom > v {
            bottom = v;
        }
        let offset = cd.ledge_snap_y + half_height;
        let v = cd.cur_pos.y + offset;
        if top < v {
            top = v;
        }
        let v = cd.prev_pos.y + offset;
        if top < v {
            top = v;
        }
    }
    g.mp_bounding_check(left, bottom, right, top);
}

/// `mpColl_80042384`.
pub fn mp_coll_80042384(cd: &mut CollData) {
    let d = &mut cd.desired_ecb;
    if abs(d.top.y - d.bottom.y) < 1.0 {
        d.top.y += 1.0;
        let mid = 0.5f32 * (d.top.y + d.bottom.y);
        d.left.y = mid;
        d.right.y = mid;
    }
    if d.top.y < 1.0 {
        d.top.y = 1.0;
    }
    if d.left.x > -1.0 {
        d.left.x = -1.0;
    }
    if d.right.x < 1.0 {
        d.right.x = 1.0;
    }
    if d.top.y < d.bottom.y {
        d.top.y = 1.0 + d.bottom.y;
    }
    if d.right.y > d.top.y || d.right.y < d.bottom.y {
        let mid = 0.5f32 * (d.top.y + d.bottom.y);
        d.left.y = mid;
        d.right.y = mid;
    }
    if d.top.y - d.right.y < 0.001 || d.right.y - d.bottom.y < 0.001 {
        d.right.y = 0.5f32 * (d.top.y + d.bottom.y);
    }
    if d.top.y - d.left.y < 0.001 || d.left.y - d.bottom.y < 0.001 {
        d.left.y = 0.5f32 * (d.top.y + d.bottom.y);
    }
}

/// `mpColl_LoadECB_JObj`. `joints[i]` = `lb_8000B1CC(ecb_source.x10C_joint[i], NULL)`.
pub fn mp_coll_load_ecb_jobj(cd: &mut CollData, flags: u32, joints: &[Vec3; 6]) {
    if cd.x130_flags & COLLDATA_X130_CLEAR != 0 {
        cd.ecb = zero_ecb();
        cd.x130_flags &= !COLLDATA_X130_CLEAR;
    }
    cd.xE4_ecb = cd.ecb;
    let (tx, ty) = (cd.cur_pos.x, cd.cur_pos.y);
    let mut left_x = joints[0].x - tx;
    let mut right_x = left_x;
    let mut bottom_y = joints[0].y - ty;
    let mut top_y = bottom_y;
    for v in &joints[1..] {
        let dx = v.x - tx;
        let dy = v.y - ty;
        update_min_max(&mut left_x, &mut right_x, dx);
        update_min_max(&mut bottom_y, &mut top_y, dy);
    }
    if flags & COLL_FLAG_AIR_CAN_GRAB_LEDGE == 0 {
        left_x -= 2.0;
        right_x += 2.0;
        bottom_y -= 2.0;
        top_y += 2.0;
    }
    let phi_f1 = if 4.0f32 > cd.ecb_source.x12C { 4.0 } else { cd.ecb_source.x12C };
    let phi_f2 = abs(right_x - left_x);
    if phi_f2 < phi_f1 {
        right_x = 0.5f32 * phi_f2;
        left_x = -right_x;
    }
    let phi_f1 = if 4.0f32 > cd.ecb_source.x128 { 4.0 } else { cd.ecb_source.x128 };
    let phi_f2 = abs(top_y - bottom_y);
    if phi_f2 < phi_f1 {
        let tmpval = 0.5f32 * phi_f2;
        let mid_y = 0.5f32 * (top_y + bottom_y);
        top_y = mid_y + tmpval;
        bottom_y = mid_y - tmpval;
    }
    if flags & 0x8 != 0 {
        left_x = -1.0;
        right_x = 1.0;
    } else {
        right_x = if right_x < 2.0 { 2.0 } else { right_x };
        left_x = if left_x > -2.0 { -2.0 } else { left_x };
    }
    if flags & 1 != 0 {
        bottom_y = 0.0;
        if flags & 0x10 != 0 {
            top_y = 2.0;
        }
    } else {
        if bottom_y < 0.0 {
            bottom_y = 0.0;
        }
        if flags & 0x10 != 0 {
            let mid_y = 0.5f32 * (bottom_y + top_y);
            bottom_y = mid_y - 1.0;
            top_y = mid_y + 1.0;
            if bottom_y < 0.0 {
                bottom_y = 0.0;
                top_y = 2.0;
            }
        }
    }
    let d = &mut cd.desired_ecb;
    d.top.x = 0.0;
    d.top.y = top_y;
    d.bottom.x = 0.0;
    d.bottom.y = bottom_y;
    d.right.x = right_x;
    // TARGET_PC: one shared `half` (retail's single fmuls), two plain adds.
    let half = 0.5f32 * (bottom_y + top_y);
    d.right.y = cd.ecb_source.x124 + half;
    d.left.x = left_x;
    d.left.y = cd.ecb_source.x124 + half;
    cd.x34_flags.set_b0(0);
}

/// `mpColl_80042C58`: the desired ECB from a fixed box.
pub fn mp_coll_80042c58(cd: &mut CollData, b: &ftCollisionBox) {
    if cd.x130_flags & COLLDATA_X130_CLEAR != 0 {
        cd.ecb = zero_ecb();
        cd.x130_flags &= !COLLDATA_X130_CLEAR;
    }
    cd.xE4_ecb = cd.ecb;
    let d = &mut cd.desired_ecb;
    d.top.x = 0.0;
    d.top.y = b.top;
    d.bottom.x = 0.0;
    d.bottom.y = b.bottom;
    d.right.x = b.right.x;
    d.right.y = b.right.y;
    d.left.x = b.left.x;
    d.left.y = b.left.y;
    cd.x34_flags.set_b0(0);
}

/// `mpColl_LoadECB_inline(coll, flags)` for a JObj source (the Fixed source is not ported).
pub fn mp_coll_load_ecb_inline(cd: &mut CollData, flags: u32, joints: &[Vec3; 6]) -> R<()> {
    let locked = cd.x130_flags & COLLDATA_X130_LOCKED != 0;
    let saved = (cd.desired_ecb.bottom.x, cd.desired_ecb.bottom.y);
    if cd.ecb_source.kind == ECB_SOURCE_JOBJ as _ {
        mp_coll_load_ecb_jobj(cd, flags, joints);
    } else {
        return Err(NotPorted("mpColl_LoadECB_Fixed"));
    }
    if locked {
        cd.desired_ecb.bottom.x = saved.0;
        cd.desired_ecb.bottom.y = saved.1;
    }
    mp_coll_80042384(cd);
    Ok(())
}

/// `mpCollInterpolateECB` (`Vec2_Interpolate` fuses `dest += time * (src - dest)`).
pub fn mp_coll_interpolate_ecb(cd: &mut CollData, time: f32) {
    cd.prev_ecb = cd.ecb;
    if cd.x34_flags.b6() != 0 {
        cd.ecb = cd.x64_ecb;
        cd.x34_flags.set_b6(0);
    }
    let d = cd.desired_ecb;
    let e = &mut cd.ecb;
    for (dst, src) in [(&mut e.top, d.top), (&mut e.bottom, d.bottom), (&mut e.left, d.left), (&mut e.right, d.right)] {
        dst.x = fmadds(time, src.x - dst.x, dst.x);
        dst.y = fmadds(time, src.y - dst.y, dst.y);
    }
    let e = cd.ecb;
    for v in [e.top.x, e.top.y, e.bottom.x, e.bottom.y, e.left.x, e.left.y, e.right.x, e.right.y] {
        assert!(!v.is_nan(), "mpcoll.c:1193: ECB NaN");
    }
}

/// `mpColl_80043754`: the substep loop. `cb` runs once per substep.
pub fn mp_coll_80043754(
    g: &mut GroundColl, st: &mut CollGlobals, cb: StepCb, cd: &mut CollData, flags: u32, floor_cb: &mut LineFilter,
) -> R<bool> {
    let mut vel = TVec { x: cd.cur_pos.x - cd.last_pos.x, y: cd.cur_pos.y - cd.last_pos.y, z: cd.cur_pos.z - cd.last_pos.z };
    let mut x = abs(vel.x);
    let mut y = abs(vel.y);
    let mut dist_left_x = abs(cd.desired_ecb.left.x - cd.ecb.left.x);
    let dist_right_x = abs(cd.desired_ecb.right.x - cd.ecb.right.x);
    if dist_left_x < dist_right_x {
        dist_left_x = dist_right_x;
    }
    let mut dist_top_y = abs(cd.desired_ecb.top.y - cd.ecb.top.y);
    let dist_right_y = abs(cd.desired_ecb.right.y - cd.ecb.right.y);
    if dist_top_y < dist_right_y {
        dist_top_y = dist_right_y;
    }
    // max_inline(a, b) = a > b ? a : b.
    x = if x > dist_left_x { x } else { dist_left_x };
    y = if y > dist_top_y { y } else { dist_top_y };
    x = if x > y { x } else { y };
    let steps: i32 = if x > 6.0 {
        let s = melee_runtime::cabi::f2i32((x / 6.0) as f64) + 1;
        vel.x /= s as f32;
        vel.y /= s as f32;
        vel.z /= s as f32;
        s
    } else {
        1
    };
    let mut step = 0;
    cd.cur_pos = cd.last_pos;
    cd.x34_flags.set_b5(0);
    let mut ret = false;
    while step < steps && cd.x34_flags.b5() == 0 {
        mp_coll_interpolate_ecb(cd, 1.0 / (steps - step) as f32);
        cd.prev_pos = cd.cur_pos;
        cd.cur_pos.x += vel.x;
        cd.cur_pos.y += vel.y;
        cd.cur_pos.z += vel.z;
        mp_coll_check_bounding(g, cd, flags);
        ret = match cb {
            StepCb::Air => mp_coll_80046904(g, st, cd, flags, floor_cb)?,
            StepCb::Ground => mp_coll_8004ace4(g, st, cd, flags)?,
        };
        g.mp_uncheck_bounding();
        step += 1;
        cd.x38 = st.x804d64ac;
    }
    if std::env::var_os("MELEE_GOLDEN_ECB").is_some() {
        let h = |f: f32| f.to_bits();
        eprintln!("STEP steps {steps} cur {:08X} {:08X} ret {} env {:X} ecb T{:08X} B{:08X} R{:08X},{:08X} L{:08X},{:08X}",
            h(cd.cur_pos.x), h(cd.cur_pos.y), ret as i32, cd.env_flags, h(cd.ecb.top.y), h(cd.ecb.bottom.y),
            h(cd.ecb.right.x), h(cd.ecb.right.y), h(cd.ecb.left.x), h(cd.ecb.left.y));
    }
    Ok(ret)
}

/// `mpCollEnd` (no stage callbacks, no dynamic attributes: see the module doc).
pub fn mp_coll_end(_cd: &mut CollData, _arg1: bool, _arg2: bool) {}

/// The `inline0` / `inline1` wrapper: ECB-tiny flag, the step loop with mpColl_80046904, End.
pub(crate) fn run_air(
    g: &mut GroundColl, st: &mut CollGlobals, cd: &mut CollData, flags: u32, floor_cb: &mut LineFilter,
) -> R<bool> {
    cd.prev_env_flags = cd.env_flags;
    cd.env_flags = 0;
    st.is_ecb_tiny = cd.ecb.top.y - cd.ecb.bottom.y < 6.0 && cd.ecb.right.y - cd.ecb.left.y < 6.0;
    let r = mp_coll_80043754(g, st, StepCb::Air, cd, flags, floor_cb)?;
    mp_coll_end(cd, r, true);
    Ok(r)
}

/// `mpColl_800471F8`: JObj ECB (flags 6), `inline0(coll, 0, true)`.
pub fn mp_coll_800471f8(g: &mut GroundColl, st: &mut CollGlobals, cd: &mut CollData, joints: &[Vec3; 6]) -> R<bool> {
    mp_coll_prev(cd);
    mp_coll_load_ecb_inline(cd, 6, joints)?;
    run_air(g, st, cd, 0, &mut None)
}

/// `mpColl_800477E0`: JObj ECB (flags 6), `inline0(coll, 1, true)`.
pub fn mp_coll_800477e0(g: &mut GroundColl, st: &mut CollGlobals, cd: &mut CollData, joints: &[Vec3; 6]) -> R<bool> {
    mp_coll_prev(cd);
    mp_coll_load_ecb_inline(cd, 6, joints)?;
    run_air(g, st, cd, 1, &mut None)
}

/// `mpColl_800473CC`: JObj ECB (flags 6), `inline0(coll, 4, true)`.
pub fn mp_coll_800473cc(g: &mut GroundColl, st: &mut CollGlobals, cd: &mut CollData, joints: &[Vec3; 6]) -> R<bool> {
    mp_coll_prev(cd);
    mp_coll_load_ecb_inline(cd, 6, joints)?;
    run_air(g, st, cd, 4, &mut None)
}

/// `mpColl_80048654`: JObj ECB (flags 5), `inline0(coll, 0, true)`.
pub fn mp_coll_80048654(g: &mut GroundColl, st: &mut CollGlobals, cd: &mut CollData, joints: &[Vec3; 6]) -> R<bool> {
    mp_coll_prev(cd);
    mp_coll_load_ecb_inline(cd, 5, joints)?;
    run_air(g, st, cd, 0, &mut None)
}

/// `mpColl_800478F4`: JObj ECB (flags 5), `inline0(coll, 1, true)`.
pub fn mp_coll_800478f4(g: &mut GroundColl, st: &mut CollGlobals, cd: &mut CollData, joints: &[Vec3; 6]) -> R<bool> {
    mp_coll_prev(cd);
    mp_coll_load_ecb_inline(cd, 5, joints)?;
    run_air(g, st, cd, 1, &mut None)
}

/// `mpColl_8004730C`: a fixed box, `inline0(coll, 0, true)`.
pub fn mp_coll_8004730c(g: &mut GroundColl, st: &mut CollGlobals, cd: &mut CollData, b: &ftCollisionBox) -> R<bool> {
    mp_coll_prev(cd);
    mp_coll_80042c58(cd, b);
    run_air(g, st, cd, 0, &mut None)
}

/// `mpColl_80047E14`: JObj ECB (flags 6), `inline1(coll, 6, floor_cb, gobj)`.
pub fn mp_coll_80047e14(
    g: &mut GroundColl, st: &mut CollGlobals, cd: &mut CollData, joints: &[Vec3; 6], floor_cb: &mut LineFilter,
) -> R<bool> {
    mp_coll_prev(cd);
    mp_coll_load_ecb_inline(cd, 6, joints)?;
    run_air(g, st, cd, 6, floor_cb)
}

/// `mpColl_80043BBC`.
pub(crate) fn mp_coll_80043bbc(g: &mut GroundColl, cd: &CollData) -> Option<i32> {
    let line_id = g.mp_line_prev_non_floor(cd.floor.index);
    let bx = cd.cur_pos.x + cd.ecb.bottom.x;
    let by = cd.cur_pos.y + cd.ecb.bottom.y;
    let rx = cd.cur_pos.x + cd.ecb.right.x;
    let ry = cd.cur_pos.y + cd.ecb.right.y;
    match g.mp_check_left_wall(bx, by, rx, ry, cd.joint_id_skip, cd.joint_id_only) {
        Some(h) if h.line_id != line_id => Some(h.line_id),
        _ => None,
    }
}

/// `mpColl_80043E90`.
pub(crate) fn mp_coll_80043e90(g: &mut GroundColl, cd: &CollData) -> Option<i32> {
    let line_id = g.mp_line_next_non_floor(cd.floor.index);
    let bx = cd.cur_pos.x + cd.ecb.bottom.x;
    let by = cd.cur_pos.y + cd.ecb.bottom.y;
    let lx = cd.cur_pos.x + cd.ecb.left.x;
    let ly = cd.cur_pos.y + cd.ecb.left.y;
    match g.mp_check_right_wall(bx, by, lx, ly, cd.joint_id_skip, cd.joint_id_only) {
        Some(h) if h.line_id != line_id => Some(h.line_id),
        _ => None,
    }
}

/// `mpColl_stage_same_joint` (no script slots: `ScriptGame_StageSlotSameJoint` is -1).
pub(crate) fn same_joint(g: &GroundColl, a: i32, b: i32) -> bool {
    g.mp_joint_from_line(a) == g.mp_joint_from_line(b)
}

/// `mpColl_80044164` (left ledge) and `mpColl_800443C4` (right ledge): `dir` 1 / -1.
pub(crate) fn mp_coll_ledge(g: &mut GroundColl, cd: &mut CollData, right_side: bool) -> Option<i32> {
    let half_height = 0.5f32 * cd.ledge_snap_height;
    let mut snap_x = cd.ledge_snap_x;
    let snap_y = cd.ledge_snap_y;
    let (left, right);
    if !right_side {
        if cd.prev_pos.x < cd.cur_pos.x {
            left = cd.prev_pos.x;
            right = snap_x + (cd.cur_pos.x + cd.ecb.right.x);
        } else {
            left = cd.cur_pos.x;
            right = snap_x + (cd.prev_pos.x + cd.ecb.right.x);
        }
    } else {
        snap_x = -snap_x;
        if cd.prev_pos.x > cd.cur_pos.x {
            right = cd.prev_pos.x;
            left = snap_x + (cd.cur_pos.x + cd.ecb.left.x);
        } else {
            right = cd.cur_pos.x;
            left = snap_x + (cd.prev_pos.x + cd.ecb.left.x);
        }
    }
    let (bottom, top) = if cd.prev_pos.y < cd.cur_pos.y {
        ((cd.prev_pos.y + snap_y) - half_height, half_height + (cd.cur_pos.y + snap_y))
    } else {
        ((cd.cur_pos.y + snap_y) - half_height, half_height + (cd.prev_pos.y + snap_y))
    };
    let already_checked = g.mp_checked_bounding();
    if !already_checked {
        g.mp_bounding_check(left, bottom, right, top);
    }
    let dir = if right_side { -1 } else { 1 };
    let (ledge_id, contact) =
        g.mp_lib_80051ba8_floor(cd.floor_skip, cd.joint_id_skip, cd.joint_id_only, dir, left, bottom, right, top);
    if let Some(c) = contact {
        cd.contact = tv(c);
    }
    let checks = if right_side { 10 } else { 6 };
    let mut grabbed = false;
    if ledge_id != -1 {
        let edge = if right_side { g.mp_floor_get_right(ledge_id) } else { g.mp_floor_get_left(ledge_id) };
        let near = if right_side { edge.x - cd.contact.x < 5.0 } else { cd.contact.x - edge.x < 5.0 };
        let bx = cd.cur_pos.x + cd.ecb.bottom.x;
        let by = cd.cur_pos.y + cd.ecb.bottom.y;
        let side = if right_side { bx > edge.x } else { bx < edge.x };
        if near && side && by < edge.y {
            let ok = by > cd.contact.y || {
                let a = match g.mp_check_multiple(cd.cur_pos.x + cd.ecb.top.x, cd.cur_pos.y + cd.ecb.top.y, cd.contact.x, cd.contact.y, checks, cd.joint_id_skip, cd.joint_id_only) {
                    None => true,
                    Some(h) => same_joint(g, ledge_id, h.line_id),
                };
                a && match g.mp_check_multiple(bx, -2.0f32 + by, cd.contact.x, cd.contact.y, checks, cd.joint_id_skip, cd.joint_id_only) {
                    None => true,
                    Some(h) => same_joint(g, ledge_id, h.line_id),
                }
            };
            grabbed = ok;
        }
    }
    if !already_checked {
        g.mp_uncheck_bounding();
    }
    if grabbed {
        Some(ledge_id)
    } else {
        None
    }
}

/// `mpColl_80044628_Floor`.
pub(crate) fn mp_coll_80044628_floor(
    g: &mut GroundColl, st: &CollGlobals, cd: &mut CollData, cb: &mut LineFilter, left_right: i32,
) -> bool {
    let pbx = cd.prev_pos.x + cd.prev_ecb.bottom.x;
    let pby = cd.prev_pos.y + cd.prev_ecb.bottom.y;
    let bottom = Vec3 { x: cd.cur_pos.x + cd.ecb.bottom.x, y: cd.cur_pos.y + cd.ecb.bottom.y, z: 0.0 };
    let hit = {
        let cbr: LineFilter = cb.as_mut().map(|f| &mut **f as &mut dyn FnMut(i32) -> bool);
        if cd.x38 != st.x804d64ac {
            g.mp_check_floor_remap(pbx, pby, bottom.x, bottom.y, 0.0, cd.floor_skip, cd.joint_id_skip, cd.joint_id_only, cbr)
        } else {
            g.mp_check_floor(pbx, pby, bottom.x, bottom.y, 0.0, cd.floor_skip, cd.joint_id_skip, cd.joint_id_only, cbr)
        }
    };
    if let Some(h) = hit {
        cd.contact = tv(h.pos);
        cd.floor.index = h.line_id;
        cd.floor.flags = h.flags;
        cd.floor.normal = tv(h.normal);
        if cd.floor.flags & LINE_FLAG_PLATFORM == 0 || cd.floor.index != cd.floor_skip {
            cd.env_flags |= COLLIDE_FLOOR_PUSH;
            cd.env_flags |= COLLIDE_FLOOR_HUG;
            return true;
        }
    }
    let mut line_id = -1;
    let found = (left_right & 1 != 0 && {
        line_id = g.mp_line_prev_non_left_wall(cd.left_facing_wall.index);
        line_id != -1
    }) || (left_right & 2 != 0 && {
        line_id = g.mp_line_next_non_right_wall(cd.right_facing_wall.index);
        line_id != -1
    });
    if found && g.mp_lib_80054ed8(line_id) && g.mp_line_get_kind(line_id) == COLL_LINE_FLOOR {
        if let Some(h) = g.mp_lib_8004dd90_floor(line_id, bottom) {
            cd.floor.flags = h.flags;
            cd.floor.normal = tv(h.normal);
            if h.dist > 0.0 {
                cd.floor.index = h.line_id;
                if cd.floor.flags & LINE_FLAG_PLATFORM == 0 || cd.floor.index != cd.floor_skip {
                    let pass = match cb.as_mut() {
                        None => true,
                        Some(f) => f(cd.floor.index),
                    };
                    if pass {
                        cd.env_flags |= COLLIDE_FLOOR_PUSH;
                        return true;
                    }
                }
            }
        }
    }
    false
}

/// `mpColl_80044838_Floor`.
pub(crate) fn mp_coll_80044838_floor(g: &mut GroundColl, cd: &mut CollData, ignore_bottom: bool) -> bool {
    let bottom = if ignore_bottom {
        Vec3 { x: cd.cur_pos.x, y: cd.cur_pos.y, z: 0.0 }
    } else {
        Vec3 { x: cd.cur_pos.x + cd.ecb.bottom.x, y: cd.cur_pos.y + cd.ecb.bottom.y, z: 0.0 }
    };
    match g.mp_lib_8004dd90_floor(cd.floor.index, bottom) {
        Some(h) => {
            cd.floor.flags = h.flags;
            cd.floor.normal = tv(h.normal);
            cd.floor.index = h.line_id;
            cd.cur_pos.y += h.dist;
        }
        None => {
            let mut edge = g.mp_floor_get_left(cd.floor.index);
            if edge.x <= bottom.x {
                edge = g.mp_floor_get_right(cd.floor.index);
            }
            cd.cur_pos.x = edge.x - cd.ecb.bottom.x;
            cd.cur_pos.y = edge.y - cd.ecb.bottom.y;
            if let Some(h) = g.mp_lib_8004dd90_floor(cd.floor.index, edge) {
                cd.floor.flags = h.flags;
                cd.floor.normal = tv(h.normal);
                cd.floor.index = h.line_id;
            }
        }
    }
    true
}

/// `mpColl_80044948_Floor`.
pub(crate) fn mp_coll_80044948_floor(g: &mut GroundColl, cd: &mut CollData) -> bool {
    let mut hit_wall = false;
    let bottom = if cd.ecb.bottom.y <= 0.0 {
        Vec3 { x: cd.cur_pos.x + cd.ecb.bottom.x, y: cd.cur_pos.y + cd.ecb.bottom.y, z: 0.0 }
    } else {
        Vec3 { x: cd.cur_pos.x, y: cd.cur_pos.y, z: 0.0 }
    };
    match g.mp_lib_8004dd90_floor(cd.floor.index, bottom) {
        Some(h) => {
            cd.floor.flags = h.flags;
            cd.floor.normal = tv(h.normal);
            cd.floor.index = h.line_id;
            hit_wall = true;
            cd.cur_pos.y += h.dist;
        }
        None => {
            let mut edge = g.mp_floor_get_left(cd.floor.index);
            if bottom.x < edge.x {
                let l = g.mp_line_prev_non_floor(cd.floor.index);
                if l != -1 && g.mp_line_get_kind(l) == COLL_LINE_RIGHT_WALL {
                    hit_wall = true;
                }
            } else {
                edge = g.mp_floor_get_right(cd.floor.index);
                let l = g.mp_line_next_non_floor(cd.floor.index);
                if l != -1 && g.mp_line_get_kind(l) == COLL_LINE_LEFT_WALL {
                    hit_wall = true;
                }
            }
            cd.cur_pos.y = edge.y - cd.ecb.bottom.y;
            if hit_wall {
                cd.cur_pos.x = edge.x;
                if let Some(h) = g.mp_lib_8004dd90_floor(cd.floor.index, edge) {
                    cd.floor.flags = h.flags;
                    cd.floor.normal = tv(h.normal);
                    cd.floor.index = h.line_id;
                }
            }
        }
    }
    hit_wall
}

/// `mpColl_80044AD8_Ceiling`.
pub(crate) fn mp_coll_80044ad8_ceiling(g: &mut GroundColl, st: &CollGlobals, cd: &mut CollData, left_right: i32) -> bool {
    let ptx = cd.prev_pos.x + cd.prev_ecb.top.x;
    let pty = cd.prev_pos.y + cd.prev_ecb.top.y;
    let top = Vec3 { x: cd.cur_pos.x + cd.ecb.top.x, y: cd.cur_pos.y + cd.ecb.top.y, z: 0.0 };
    let hit = if cd.x38 != st.x804d64ac {
        g.mp_check_ceiling_remap(ptx, pty, top.x, top.y, cd.joint_id_skip, cd.joint_id_only)
    } else {
        g.mp_check_ceiling(ptx, pty, top.x, top.y, cd.joint_id_skip, cd.joint_id_only)
    };
    if let Some(h) = hit {
        cd.contact = tv(h.pos);
        cd.ceiling.index = h.line_id;
        cd.ceiling.flags = h.flags;
        cd.ceiling.normal = tv(h.normal);
        cd.env_flags |= COLLIDE_CEILING_PUSH;
        cd.env_flags |= COLLIDE_CEILING_HUG;
        return true;
    }
    let mut line_id = -1;
    let found = (left_right & 1 != 0 && {
        line_id = g.mp_line_next_non_left_wall(cd.left_facing_wall.index);
        line_id != -1
    }) || (left_right & 2 != 0 && {
        line_id = g.mp_line_prev_non_right_wall(cd.right_facing_wall.index);
        line_id != -1
    });
    if found && g.mp_lib_80054ed8(line_id) && g.mp_line_get_kind(line_id) == COLL_LINE_CEILING {
        if let Some(h) = g.mp_lib_8004e090_ceiling(line_id, top) {
            cd.ceiling.flags = h.flags;
            cd.ceiling.normal = tv(h.normal);
            if h.dist < 0.0 {
                cd.ceiling.index = h.line_id;
                cd.env_flags |= COLLIDE_CEILING_PUSH;
                return true;
            }
        }
    }
    false
}

/// `mpColl_RightWall_inline` / `mpColl_LeftWall_inline`: remember a wall unless it, or a line
/// connected to it, is already listed.
pub(crate) fn add_wall(g: &GroundColl, list: &mut [i32; MPCOLL_WALLID_MAX], n: &mut usize, line_id: i32) {
    for i in 0..*n {
        let s = list[i];
        if line_id == s || g.mp_lines_connected(s, line_id) {
            return;
        }
    }
    assert!(*n < MPCOLL_WALLID_MAX, "mpcoll.c:330: i<MPCOLL_WALLID_MAX");
    list[*n] = line_id;
    *n += 1;
}

/// `mpColl_80044E10_RightWall` (`right` false) / `mpColl_80045B74_LeftWall` (`right` true is
/// the left-facing-wall check against the ECB's right side): the wall checks.
pub(crate) fn mp_coll_wall_check(g: &mut GroundColl, st: &mut CollGlobals, cd: &mut CollData, left_wall: bool) -> bool {
    mp_coll_wall_check_ex(g, st, cd, left_wall, false)
}

/// The wall checks; `ground` = the grounded variants (mpColl_80048AB0_RightWall /
/// mpColl_80049778_LeftWall), which ignore the two walls next to the floor (`line_id1/2`) in
/// the bottom-based checks.
pub(crate) fn mp_coll_wall_check_ex(g: &mut GroundColl, st: &mut CollGlobals, cd: &mut CollData, left_wall: bool, ground: bool) -> bool {
    let (ex1, ex2) = if ground && g.mp_lib_80054ed8(cd.floor.index) {
        let first = |g: &GroundColl, i: i32| if left_wall { g.mp_lib_80053448_floor(i) } else { g.mp_lib_80053394_floor(i) };
        let second = |g: &GroundColl, i: i32| if left_wall { g.mp_lib_800534fc_floor(i) } else { g.mp_lib_800536cc_floor(i) };
        let id1 = first(g, cd.floor.index);
        let t = second(g, cd.floor.index);
        (id1, if t != -1 { first(g, t) } else { -1 })
    } else {
        (-1, -1)
    };
    // `&& line_id != line_id1 && line_id != line_id2` (not in the air variant).
    let keep = |l: i32| !ground || (l != ex1 && l != ex2);
    let mut hit_wall = false;
    let mut list = if left_wall { st.left } else { st.right };
    let mut n = 0usize;
    // The ECB side that faces the wall: right for a left wall, left for a right wall.
    let side = |e: &_ftECB| if left_wall { e.right } else { e.left };
    let sx = cd.cur_pos.x + side(&cd.ecb).x;
    let sy = cd.cur_pos.y + side(&cd.ecb).y;
    let remap = cd.x38 != st.x804d64ac;
    let (js, jo) = (cd.joint_id_skip, cd.joint_id_only);
    let check = |g: &mut GroundColl, ax: f32, ay: f32, bx: f32, by: f32, rm: bool| -> Option<i32> {
        let h = match (left_wall, rm) {
            (true, true) => g.mp_check_left_wall_remap(ax, ay, bx, by, js, jo),
            (true, false) => g.mp_check_left_wall(ax, ay, bx, by, js, jo),
            (false, true) => g.mp_check_right_wall_remap(ax, ay, bx, by, js, jo),
            (false, false) => g.mp_check_right_wall(ax, ay, bx, by, js, jo),
        };
        h.map(|h| h.line_id)
    };
    let px = cd.prev_pos.x + side(&cd.prev_ecb).x;
    let py = cd.prev_pos.y + side(&cd.prev_ecb).y;
    if let Some(l) = check(g, px, py, sx, sy, remap) {
        add_wall(g, &mut list, &mut n, l);
        hit_wall = true;
        cd.env_flags |= if left_wall { COLLIDE_LEFT_WALL_HUG } else { COLLIDE_RIGHT_WALL_HUG };
    }
    let pbx = cd.prev_pos.x + cd.prev_ecb.bottom.x;
    let pby = cd.prev_pos.y + cd.prev_ecb.bottom.y;
    let bx = cd.cur_pos.x + cd.ecb.bottom.x;
    let by = cd.cur_pos.y + cd.ecb.bottom.y;
    if let Some(l) = check(g, pbx, pby, bx, by, remap).filter(|&l| keep(l)) {
        add_wall(g, &mut list, &mut n, l);
        hit_wall = true;
    }
    let ptx = cd.prev_pos.x + cd.prev_ecb.top.x;
    let pty = cd.prev_pos.y + cd.prev_ecb.top.y;
    let tx = cd.cur_pos.x + cd.ecb.top.x;
    let ty = cd.cur_pos.y + cd.ecb.top.y;
    if let Some(l) = check(g, ptx, pty, tx, ty, remap) {
        add_wall(g, &mut list, &mut n, l);
        hit_wall = true;
    }
    // bottom -> side (no remap variant).
    if let Some(l) = check(g, bx, by, sx, sy, false).filter(|&l| keep(l)) {
        add_wall(g, &mut list, &mut n, l);
        hit_wall = true;
    }
    let psx = cd.prev_pos.x + side(&cd.prev_ecb).x;
    let psy = cd.prev_pos.y + side(&cd.prev_ecb).y;
    if !st.is_ecb_tiny {
        let r = if left_wall {
            g.mp_lib_800515a0_left_wall(psx, psy, pbx, pby, sx, sy, bx, by, js, jo)
        } else {
            g.mp_lib_800511a4_right_wall(pbx, pby, psx, psy, bx, by, sx, sy, js, jo)
        };
        if let Some(l) = r.filter(|&l| keep(l)) {
            add_wall(g, &mut list, &mut n, l);
            hit_wall = true;
        }
    }
    if let Some(l) = check(g, tx, ty, sx, sy, false) {
        add_wall(g, &mut list, &mut n, l);
        hit_wall = true;
    }
    if !st.is_ecb_tiny {
        let r = if left_wall {
            g.mp_lib_800515a0_left_wall(ptx, pty, psx, psy, tx, ty, sx, sy, js, jo)
        } else {
            g.mp_lib_800511a4_right_wall(psx, psy, ptx, pty, sx, sy, tx, ty, js, jo)
        };
        if let Some(l) = r {
            add_wall(g, &mut list, &mut n, l);
            hit_wall = true;
        }
    }
    if hit_wall {
        cd.env_flags |= if left_wall { COLLIDE_LEFT_WALL_PUSH } else { COLLIDE_RIGHT_WALL_PUSH };
    }
    if left_wall {
        st.left = list;
        st.n_left = n;
    } else {
        st.right = list;
        st.n_right = n;
    }
    hit_wall
}

/// A candidate wall contact of the push-out searches (`mpColl_804D6490/94/98` + `.normal`).
#[derive(Clone, Copy)]
pub(crate) struct WallBest {
    max_x: f32,
    line_id: i32,
    flags: u32,
    normal: Vec3,
}

pub(crate) fn wall_take(b: &mut WallBest, max_x: f32, h: &crate::mp::geom::LineHit) {
    b.max_x = max_x;
    b.line_id = h.line_id;
    b.flags = h.flags;
    b.normal = h.normal;
}

/// `mpColl_80046224_LeftWall`: push the ECB out of the left-facing walls collected by the check
/// (`mpColl_80045B74_LeftWall`). True = `cur_pos.x` moved.
pub(crate) fn mp_coll_80046224_left_wall(g: &mut GroundColl, st: &CollGlobals, cd: &mut CollData) -> bool {
    let mut b = WallBest { max_x: F32_MAX, line_id: -1, flags: 0, normal: Vec3 { x: 0.0, y: 0.0, z: 0.0 } };
    for i in 0..st.n_left {
        let mut wall_id = st.left[i];
        let mut pos = g.mp_left_wall_get_top(wall_id);
        if pos.y < cd.cur_pos.y + cd.ecb.bottom.y {
            if b.max_x > pos.x {
                if let Some(h) = g.mp_lib_8004e398_left_wall(wall_id, pos) {
                    if h.line_id != -1 {
                        wall_take(&mut b, pos.x, &h);
                    }
                }
            }
            continue;
        }
        pos = g.mp_left_wall_get_bottom(wall_id);
        if pos.y > cd.cur_pos.y + cd.ecb.top.y {
            if b.max_x > pos.x {
                if let Some(h) = g.mp_lib_8004e398_left_wall(wall_id, pos) {
                    if h.line_id != -1 {
                        wall_take(&mut b, pos.x, &h);
                    }
                }
            }
            continue;
        }
        for pt in [cd.ecb.bottom, cd.ecb.right, cd.ecb.top] {
            pos.x = cd.cur_pos.x + pt.x;
            pos.y = cd.cur_pos.y + pt.y;
            if let Some(h) = g.mp_lib_8004e398_left_wall(wall_id, pos) {
                if h.line_id != -1 && b.max_x > cd.cur_pos.x + h.dist {
                    wall_take(&mut b, cd.cur_pos.x + h.dist, &h);
                }
            }
        }
        let line_id = g.mp_line_next_non_left_wall(wall_id);
        if line_id != -1 && g.mp_lib_80054ed8(line_id) && g.mp_line_get_kind(line_id) & COLL_LINE_CEILING != 0 {
            let vec = g.mp_left_wall_get_top(wall_id);
            if pos.y > vec.y {
                let line_id2 = g.mp_line_prev_non_ceiling(line_id);
                // (the C tests line_id, not line_id2, in the second half.)
                if line_id2 != -1 && g.mp_lib_80054ed8(line_id) && g.mp_line_get_kind(line_id) & COLL_LINE_LEFT_WALL != 0 {
                    let nrm = g.mp_line_get_normal(line_id2);
                    // `(pos.y - vec.y) / -nrm.x * nrm.y + vec.x - pos.x - 0.5F`: one fused site.
                    let x = fmadds((pos.y - vec.y) / -nrm.x, nrm.y, vec.x) - pos.x - 0.5;
                    if b.max_x > cd.cur_pos.x + x {
                        b.max_x = cd.cur_pos.x + x;
                        b.line_id = line_id2;
                        b.flags = g.mp_line_get_flags(line_id2);
                        b.normal = nrm;
                    }
                }
            }
        }
        let f7 = cd.ecb.top.y;
        let f6 = cd.ecb.right.y;
        let f5 = cd.ecb.bottom.y;
        let f4 = cd.ecb.right.x;
        let f27 = (f4 - cd.ecb.bottom.x) / (f6 - f5);
        let f26 = (f4 - cd.ecb.top.x) / (f6 - f7);
        let f30 = cd.cur_pos.y + f7;
        let f29 = cd.cur_pos.y + f6;
        let f28 = cd.cur_pos.y + f5;
        // The walls below (prev) then above (next) the first one.
        let mut j = wall_id;
        while j != -1 && g.mp_line_get_kind(j) == COLL_LINE_LEFT_WALL {
            let pos = g.mp_line_get_v0_pos(j);
            let x;
            if f28 <= pos.y && pos.y <= f29 {
                x = fmadds(f27, pos.y - f28, cd.ecb.bottom.x);
            } else if f29 <= pos.y && pos.y <= f30 {
                x = fmadds(f26, pos.y - f30, cd.ecb.top.x);
            } else if pos.y < f28 {
                break;
            } else {
                j = g.mp_line_get_prev(j);
                continue;
            }
            let x = pos.x - x;
            if b.max_x > x {
                b.max_x = x;
                b.line_id = j;
                b.flags = g.mp_line_get_flags(j);
                b.normal = g.mp_line_get_normal(j);
            }
            j = g.mp_line_get_prev(j);
        }
        while wall_id != -1 && g.mp_line_get_kind(wall_id) == COLL_LINE_LEFT_WALL {
            let pos = g.mp_line_get_v1_pos(wall_id);
            let x;
            if f28 <= pos.y && pos.y <= f29 {
                x = fmadds(f27, pos.y - f28, cd.ecb.bottom.x);
            } else if f29 <= pos.y && pos.y <= f30 {
                x = fmadds(f26, pos.y - f30, cd.ecb.top.x);
            } else if pos.y > f30 {
                break;
            } else {
                wall_id = g.mp_line_get_next(wall_id);
                continue;
            }
            let x = pos.x - x;
            if b.max_x > x {
                b.max_x = x;
                b.line_id = wall_id;
                b.flags = g.mp_line_get_flags(wall_id);
                b.normal = g.mp_line_get_normal(wall_id);
            }
            wall_id = g.mp_line_get_next(wall_id);
        }
    }
    if cd.cur_pos.x > b.max_x {
        cd.cur_pos.x = b.max_x;
        cd.left_facing_wall.index = b.line_id;
        cd.left_facing_wall.flags = b.flags;
        cd.left_facing_wall.normal = tv(b.normal);
        return true;
    }
    false
}

/// `mpColl_800454A4_RightWall`: the mirror of [`mp_coll_80046224_left_wall`].
pub(crate) fn mp_coll_800454a4_right_wall(g: &mut GroundColl, st: &CollGlobals, cd: &mut CollData) -> bool {
    let mut b = WallBest { max_x: -F32_MAX, line_id: -1, flags: 0, normal: Vec3 { x: 0.0, y: 0.0, z: 0.0 } };
    for i in 0..st.n_right {
        let mut wall_id = st.right[i];
        let mut pos = g.mp_right_wall_get_top(wall_id);
        if pos.y < cd.cur_pos.y + cd.ecb.bottom.y {
            if b.max_x < pos.x {
                if let Some(h) = g.mp_lib_8004e684_right_wall(wall_id, pos) {
                    if h.line_id != -1 {
                        wall_take(&mut b, pos.x, &h);
                    }
                }
            }
            continue;
        }
        pos = g.mp_right_wall_get_bottom(wall_id);
        if pos.y > cd.cur_pos.y + cd.ecb.top.y {
            if b.max_x < pos.x {
                if let Some(h) = g.mp_lib_8004e684_right_wall(wall_id, pos) {
                    if h.line_id != -1 {
                        wall_take(&mut b, pos.x, &h);
                    }
                }
            }
            continue;
        }
        for pt in [cd.ecb.bottom, cd.ecb.left, cd.ecb.top] {
            pos.x = cd.cur_pos.x + pt.x;
            pos.y = cd.cur_pos.y + pt.y;
            if let Some(h) = g.mp_lib_8004e684_right_wall(wall_id, pos) {
                if h.line_id != -1 && b.max_x < cd.cur_pos.x + h.dist {
                    wall_take(&mut b, cd.cur_pos.x + h.dist, &h);
                }
            }
        }
        let line_id = g.mp_line_prev_non_right_wall(wall_id);
        if line_id != -1 && g.mp_lib_80054ed8(line_id) && g.mp_line_get_kind(line_id) & COLL_LINE_CEILING != 0 {
            let top = g.mp_right_wall_get_top(wall_id);
            if pos.y > top.y {
                let line_id = g.mp_line_next_non_ceiling(line_id);
                if line_id != -1 && g.mp_lib_80054ed8(line_id) && g.mp_line_get_kind(line_id) & COLL_LINE_RIGHT_WALL != 0 {
                    let nrm = g.mp_line_get_normal(line_id);
                    // `(pos.y - top.y) / nrm.x * -nrm.y + top.x - pos.x + 0.5F`: one fused site.
                    let x = fmadds((pos.y - top.y) / nrm.x, -nrm.y, top.x) - pos.x + 0.5;
                    if b.max_x < cd.cur_pos.x + x {
                        b.max_x = cd.cur_pos.x + x;
                        b.line_id = line_id;
                        b.flags = g.mp_line_get_flags(line_id);
                        b.normal = nrm;
                    }
                }
            }
        }
        let f6 = cd.ecb.left.x;
        let f5 = cd.ecb.top.y;
        let f4 = cd.ecb.left.y;
        let f3 = cd.ecb.bottom.y;
        let f2 = cd.cur_pos.y;
        let top = f2 + f5;
        let mid = f2 + f4;
        let bot = f2 + f3;
        let f27 = f6 / (f4 - f3);
        let f26 = f6 / (f4 - f5);
        let mut j = wall_id;
        while j != -1 && g.mp_line_get_kind(j) == COLL_LINE_RIGHT_WALL {
            let pos = g.mp_line_get_v1_pos(j);
            let x;
            if bot <= pos.y && pos.y <= mid {
                x = fmadds(f27, pos.y - bot, cd.ecb.bottom.x);
            } else if mid <= pos.y && pos.y <= top {
                x = fmadds(f26, pos.y - top, cd.ecb.top.x);
            } else if pos.y < bot {
                break;
            } else {
                j = g.mp_line_get_next(j);
                continue;
            }
            let x = pos.x - x;
            if b.max_x < x {
                b.max_x = x;
                b.line_id = j;
                b.flags = g.mp_line_get_flags(j);
                b.normal = g.mp_line_get_normal(j);
            }
            j = g.mp_line_get_next(j);
        }
        while wall_id != -1 && g.mp_line_get_kind(wall_id) == COLL_LINE_RIGHT_WALL {
            let pos = g.mp_line_get_v0_pos(wall_id);
            let x;
            if bot <= pos.y && pos.y <= mid {
                x = fmadds(f27, pos.y - bot, cd.ecb.bottom.x);
            } else if mid <= pos.y && pos.y <= top {
                x = fmadds(f26, pos.y - top, cd.ecb.top.x);
            } else if pos.y > top {
                break;
            } else {
                wall_id = g.mp_line_get_prev(wall_id);
                continue;
            }
            let x = pos.x - x;
            if b.max_x < x {
                b.max_x = x;
                b.line_id = wall_id;
                b.flags = g.mp_line_get_flags(wall_id);
                b.normal = g.mp_line_get_normal(wall_id);
            }
            wall_id = g.mp_line_get_prev(wall_id);
        }
    }
    if cd.cur_pos.x < b.max_x {
        cd.cur_pos.x = b.max_x;
        cd.right_facing_wall.index = b.line_id;
        cd.right_facing_wall.flags = b.flags;
        cd.right_facing_wall.normal = tv(b.normal);
        return true;
    }
    false
}

/// `mpColl_80046904`: one substep of air (and ground-check) collision.
pub fn mp_coll_80046904(
    g: &mut GroundColl, st: &mut CollGlobals, cd: &mut CollData, flags: u32, floor_cb: &mut LineFilter,
) -> R<bool> {
    let platform_pass = flags & COLL_FLAG_AIR_PLATFORM_PASS_CALLBACK != 0;
    let stay_airborne = flags & COLL_FLAG_AIR_STAY_AIRBORNE != 0;
    let mut left_right_flags = 0;
    let mut touched_floor = false;
    let mut squeeze_flags_all = 0;
    let mut squeeze_flags = 0;
    loop {
        let old_squeeze_flags = squeeze_flags;
        let prev_b6 = cd.x34_flags.b6();
        squeeze_flags = 0;
        let mut x_after_left = 0.0f32;
        let mut x_after_right = 0.0f32;
        for left in [true, false, true, false] {
            if mp_coll_wall_check(g, st, cd, left) {
                let moved = if left { mp_coll_80046224_left_wall(g, st, cd) } else { mp_coll_800454a4_right_wall(g, st, cd) };
                if moved {
                    left_right_flags |= if left { 1 } else { 2 };
                    squeeze_flags |= if left { 8 } else { 4 };
                }
                if left {
                    x_after_left = cd.cur_pos.x;
                } else {
                    x_after_right = cd.cur_pos.x;
                }
            }
        }
        if squeeze_flags & 0xC == 0xC {
            let _ = (x_after_left, x_after_right);
            return Err(NotPorted("mpCollSqueezeHorizontal (air)"));
        }
        if mp_coll_80044ad8_ceiling(g, st, cd, left_right_flags) {
            return Err(NotPorted("mpColl_80044C74_Ceiling + mpCollCeilingInline (ceiling hit)"));
        }
        let r3 = if platform_pass {
            mp_coll_80044628_floor(g, st, cd, floor_cb, left_right_flags)
        } else {
            mp_coll_80044628_floor(g, st, cd, &mut None, left_right_flags)
        };
        let mut y_after_collide_floor = 0.0f32;
        if r3 {
            if stay_airborne {
                if mp_coll_80044948_floor(g, cd) {
                    if mp_coll_80043bbc(g, cd).is_some() || mp_coll_80043e90(g, cd).is_some() {
                        return Err(NotPorted("mpColl_80043C6C / 80043F40 (wall at a floor end)"));
                    }
                }
            } else {
                let ecb_unlocked = cd.ecb.bottom.y > 0.0;
                if mp_coll_80044838_floor(g, cd, ecb_unlocked && squeeze_flags & 1 == 0) {
                    if mp_coll_80043bbc(g, cd).is_some() || mp_coll_80043e90(g, cd).is_some() {
                        return Err(NotPorted("mpColl_80043C6C / 80043F40 (wall at a floor end)"));
                    }
                    cd.x34_flags.set_b5(1);
                    touched_floor = true;
                }
            }
            y_after_collide_floor = cd.cur_pos.y;
            squeeze_flags |= 2;
            if mp_coll_80044ad8_ceiling(g, st, cd, left_right_flags) {
                return Err(NotPorted("mpColl_80044C74_Ceiling + mpCollCeilingInline (ceiling hit)"));
            }
        }
        let _ = y_after_collide_floor;
        if squeeze_flags & 3 == 3 {
            return Err(NotPorted("mpCollSqueezeVertical"));
        }
        squeeze_flags_all |= squeeze_flags;
        let _ = &mut left_right_flags;
        if !(prev_b6 != cd.x34_flags.b6() || squeeze_flags != old_squeeze_flags) {
            break;
        }
    }
    if !touched_floor && flags & COLL_FLAG_AIR_CAN_GRAB_LEDGE != 0 {
        let on_edge = cd.env_flags & COLLIDE_LEFT_EDGE != 0 || cd.env_flags & COLLIDE_RIGHT_EDGE != 0;
        // mpColl_LedgeLevelOK is 0 (no Geno move asked for a level catch).
        if !on_edge && cd.cur_pos.y < cd.prev_pos.y {
            if cd.facing_dir == 1 || cd.facing_dir == 0 {
                if let Some(id) = mp_coll_ledge(g, cd, false) {
                    cd.ledge_id_left = id;
                    cd.env_flags |= COLLIDE_LEFT_LEDGE_GRAB;
                }
            }
            if cd.facing_dir == -1 || cd.facing_dir == 0 {
                if let Some(id) = mp_coll_ledge(g, cd, true) {
                    cd.ledge_id_right = id;
                    cd.env_flags |= COLLIDE_RIGHT_LEDGE_GRAB;
                }
            }
        }
    }
    if squeeze_flags_all & 8 == 0 {
        cd.env_flags &= !COLLIDE_LEFT_WALL_MASK;
    }
    if squeeze_flags_all & 4 == 0 {
        cd.env_flags &= !COLLIDE_RIGHT_WALL_MASK;
    }
    Ok(touched_floor)
}

/// `mpColl_800488F4`: follow the floor under a grounded ECB.
pub(crate) fn mp_coll_800488f4(g: &mut GroundColl, cd: &mut CollData) -> bool {
    let floor_id = cd.floor.index;
    let bottom = Vec3 { x: cd.cur_pos.x + cd.ecb.bottom.x, y: cd.cur_pos.y + cd.ecb.bottom.y, z: 0.0 };
    if !g.mp_lib_80054ed8(cd.floor.index) || g.mp_line_get_kind(cd.floor.index) != COLL_LINE_FLOOR {
        return false;
    }
    if let Some(h) = g.mp_lib_8004dd90_floor(floor_id, bottom) {
        cd.floor.flags = h.flags;
        cd.floor.normal = tv(h.normal);
        cd.cur_pos.y += h.dist;
        cd.floor.index = h.line_id;
        return true;
    }
    let mut hit_wall = false;
    let mut edge = g.mp_floor_get_left(floor_id);
    if cd.cur_pos.x < edge.x {
        let n = g.mp_line_prev_non_floor(floor_id);
        if n != -1 && g.mp_lib_80054ed8(n) && g.mp_line_get_kind(n) & COLL_LINE_RIGHT_WALL != 0 {
            hit_wall = true;
        } else {
            cd.env_flags |= COLLIDE_LEFT_LEDGE_SLIP;
        }
    } else {
        edge = g.mp_floor_get_right(floor_id);
        if cd.cur_pos.x > edge.x {
            let n = g.mp_line_next_non_floor(floor_id);
            if n != -1 && g.mp_lib_80054ed8(n) && g.mp_line_get_kind(n) & COLL_LINE_LEFT_WALL != 0 {
                hit_wall = true;
            } else {
                cd.env_flags |= COLLIDE_RIGHT_LEDGE_SLIP;
            }
        }
    }
    if hit_wall {
        cd.cur_pos = tv(edge);
        return true;
    }
    false
}

/// `mpColl_8004A45C_Floor` (`teeter` false: walk off the end, stand on the end point) /
/// `mpColl_8004A678_Floor` (`teeter` true: stop at the edge). True = the ECB was put on the end
/// point of the floor (`on_edge`).
pub(crate) fn mp_coll_floor_edge(g: &mut GroundColl, cd: &mut CollData, line_id: i32, teeter: bool) -> R<bool> {
    if !g.mp_lib_80054ed8(line_id) || g.mp_line_get_kind(line_id) != COLL_LINE_FLOOR {
        return Ok(false);
    }
    let mut on_edge = false;
    let mut edge = g.mp_floor_get_left(line_id);
    let mut hit = None;
    if cd.cur_pos.x <= edge.x {
        let cond = !teeter
            || (cd.env_flags & COLLIDE_LEFT_WALL_MASK == 0 && cd.facing_dir == -1 && (cd.lstick_x as f64) > -0.75);
        if cond {
            hit = g.mp_lib_8004dd90_floor(line_id, edge);
            if teeter {
                if let Some(h) = &hit {
                    if h.line_id != -1 {
                        edge.y += h.dist;
                    }
                }
            }
            let edge_x = edge.x + 1.0;
            let edge_y = edge.y + 1.0;
            let right_x = edge.x + cd.ecb.right.x - cd.ecb.bottom.x;
            let right_y = edge.y + cd.ecb.right.y - cd.ecb.bottom.y;
            // make sure a wall hasn't stopped us
            if g.mp_check_left_wall(edge_x, edge_y, right_x, right_y, cd.joint_id_skip, cd.joint_id_only).is_none() {
                on_edge = true;
                if !teeter {
                    cd.env_flags |= COLLIDE_RIGHT_EDGE;
                }
            }
        }
    } else {
        edge = g.mp_floor_get_right(line_id);
        let cond = cd.cur_pos.x >= edge.x
            && (!teeter
                || (cd.env_flags & COLLIDE_RIGHT_WALL_MASK == 0 && cd.facing_dir == 1 && (cd.lstick_x as f64) < 0.75));
        if cond {
            hit = g.mp_lib_8004dd90_floor(line_id, edge);
            if teeter {
                if let Some(h) = &hit {
                    if h.line_id != -1 {
                        edge.y += h.dist;
                    }
                }
            }
            let edge_x = edge.x - 1.0;
            let edge_y = edge.y + 1.0;
            let left_x = edge.x + cd.ecb.left.x - cd.ecb.bottom.x;
            let left_y = edge.y + cd.ecb.left.y - cd.ecb.bottom.y;
            if g.mp_check_right_wall(edge_x, edge_y, left_x, left_y, cd.joint_id_skip, cd.joint_id_only).is_none() {
                on_edge = true;
                if !teeter {
                    cd.env_flags |= COLLIDE_LEFT_EDGE;
                }
            }
        }
    }
    if on_edge {
        let Some(h) = hit else {
            return Err(NotPorted("mpColl_8004A45C/8004A678_Floor: no floor under the end point"));
        };
        cd.cur_pos.x = edge.x - cd.ecb.bottom.x;
        cd.cur_pos.y = edge.y - cd.ecb.bottom.y;
        cd.cur_pos.z = edge.z;
        cd.floor.index = h.line_id;
        cd.floor.flags = h.flags;
        cd.floor.normal = tv(h.normal);
        if teeter {
            cd.env_flags |= COLLIDE_EDGE;
        }
        return Ok(true);
    }
    Ok(false)
}

/// `mpColl_8004A908_Floor`: a different floor crossed by the bottom (then the ECB-middle) sweep.
pub(crate) fn mp_coll_8004a908_floor(g: &mut GroundColl, st: &CollGlobals, cd: &mut CollData, line_id: i32) -> bool {
    let pbx = cd.prev_pos.x + cd.prev_ecb.bottom.x;
    let mut pby = cd.prev_pos.y + cd.prev_ecb.bottom.y;
    let bx = cd.cur_pos.x + cd.ecb.bottom.x;
    let by = cd.cur_pos.y + cd.ecb.bottom.y;
    for pass in 0..2 {
        if pass == 1 {
            // `0.5F * (top + bottom) + prev_pos.y`: the function's one fused site.
            pby = fmadds(0.5, cd.prev_ecb.top.y + cd.prev_ecb.bottom.y, cd.prev_pos.y);
        }
        let hit = if cd.x38 != st.x804d64ac {
            g.mp_check_floor_remap(pbx, pby, bx, by, 0.0, cd.floor_skip, cd.joint_id_skip, cd.joint_id_only, None)
        } else {
            g.mp_check_floor(pbx, pby, bx, by, 0.0, cd.floor_skip, cd.joint_id_skip, cd.joint_id_only, None)
        };
        if let Some(h) = hit {
            if h.line_id != -1 && h.line_id != line_id && (line_id == -1 || !g.mp_lines_connected(h.line_id, line_id)) {
                cd.floor.index = h.line_id;
                cd.floor.flags = h.flags;
                cd.floor.normal = tv(h.normal);
                return true;
            }
        }
    }
    false
}

/// `mpColl_8004ACE4`: one substep of grounded collision. `flags` 1: stop at edges (teeter),
/// 2: walk off.
pub fn mp_coll_8004ace4(g: &mut GroundColl, st: &mut CollGlobals, cd: &mut CollData, flags: u32) -> R<bool> {
    let mut touching_floor = false;
    let left_right = 0;
    loop {
        let prev_b6 = cd.x34_flags.b6();
        for left in [true, false, true, false] {
            if mp_coll_wall_check_ex(g, st, cd, left, true) {
                return Err(NotPorted(if left {
                    "mpColl_80049EAC_LeftWall (ground wall push-out)"
                } else {
                    "mpColl_800491C8_RightWall (ground wall push-out)"
                }));
            }
        }
        if mp_coll_80044ad8_ceiling(g, st, cd, left_right) {
            return Err(NotPorted("mpColl_8004AB80 (grounded ceiling hit)"));
        }
        if mp_coll_800488f4(g, cd) {
            if mp_coll_80043bbc(g, cd).is_some() || mp_coll_80043e90(g, cd).is_some() {
                return Err(NotPorted("mpColl_80043C6C / 80043F40 (wall at a floor end)"));
            }
            touching_floor = true;
            if mp_coll_80044ad8_ceiling(g, st, cd, left_right) {
                return Err(NotPorted("mpColl_8004AB80 (grounded ceiling hit)"));
            }
        } else {
            let floor_id = cd.floor.index;
            if g.mp_lib_80054ed8(floor_id) && g.mp_line_get_kind(floor_id) == COLL_LINE_FLOOR {
                // flags & 1: stop at the edge (mpColl_8004A678_Floor); & 2: walk off (45C).
                let mut var_r23 = false;
                if flags & 1 != 0 {
                    if mp_coll_floor_edge(g, cd, floor_id, true)? {
                        cd.x34_flags.set_b5(1);
                        touching_floor = false;
                    } else {
                        var_r23 = true;
                    }
                } else if flags & 2 != 0 {
                    if mp_coll_floor_edge(g, cd, floor_id, false)? {
                        cd.x34_flags.set_b5(1);
                        touching_floor = true;
                    } else {
                        var_r23 = true;
                    }
                } else {
                    var_r23 = true;
                }
                if var_r23 {
                    let old_skip = cd.floor_skip;
                    cd.floor_skip = floor_id;
                    if mp_coll_80046904(g, st, cd, 0, &mut None)? {
                        touching_floor = true;
                    }
                    cd.floor_skip = old_skip;
                    cd.x34_flags.set_b5(1);
                }
            }
        }
        if prev_b6 == cd.x34_flags.b6() {
            break;
        }
    }
    let fi = cd.floor.index;
    if mp_coll_8004a908_floor(g, st, cd, fi) {
        if mp_coll_80044838_floor(g, cd, false)
            && (mp_coll_80043bbc(g, cd).is_some() || mp_coll_80043e90(g, cd).is_some())
        {
            return Err(NotPorted("mpColl_80043C6C / 80043F40 (wall at a floor end)"));
        }
        cd.x34_flags.set_b5(0);
        touching_floor = true;
    }
    if touching_floor {
        cd.env_flags |= COLLIDE_FLOOR_PUSH;
    }
    Ok(touching_floor)
}

/// `inline2(coll, i)`: the substep loop with mpColl_8004ACE4, then `mpCollEnd(.., false)`.
pub(crate) fn run_ground(g: &mut GroundColl, st: &mut CollGlobals, cd: &mut CollData, flags: u32) -> R<bool> {
    cd.prev_env_flags = cd.env_flags;
    cd.env_flags = 0;
    st.is_ecb_tiny = cd.ecb.top.y - cd.ecb.bottom.y < 6.0 && cd.ecb.right.y - cd.ecb.left.y < 6.0;
    let r = mp_coll_80043754(g, st, StepCb::Ground, cd, flags, &mut None)?;
    mp_coll_end(cd, r, false);
    Ok(r)
}

/// `mpColl_8004B4B0`: JObj ECB (flags 5), `inline2(coll, 1)` (stop at edges).
pub fn mp_coll_8004b4b0(g: &mut GroundColl, st: &mut CollGlobals, cd: &mut CollData, joints: &[Vec3; 6]) -> R<bool> {
    mp_coll_prev(cd);
    mp_coll_load_ecb_inline(cd, 5, joints)?;
    run_ground(g, st, cd, 1)
}

/// `mpColl_8004B108`: JObj ECB (flags 5), `inline2(coll, 0)`.
pub fn mp_coll_8004b108(g: &mut GroundColl, st: &mut CollGlobals, cd: &mut CollData, joints: &[Vec3; 6]) -> R<bool> {
    mp_coll_prev(cd);
    mp_coll_load_ecb_inline(cd, 5, joints)?;
    run_ground(g, st, cd, 0)
}

/// `mpColl_8004B2DC`: JObj ECB (flags 5), `inline2(coll, 2)` (walk off edges).
pub fn mp_coll_8004b2dc(g: &mut GroundColl, st: &mut CollGlobals, cd: &mut CollData, joints: &[Vec3; 6]) -> R<bool> {
    mp_coll_prev(cd);
    mp_coll_load_ecb_inline(cd, 5, joints)?;
    run_ground(g, st, cd, 2)
}

#[allow(dead_code)]
pub(crate) fn _unused(v: TVec) -> Vec3 {
    v3(v)
}

pub use crate::tu::mpcoll::update_min_max_fp as update_min_max;
