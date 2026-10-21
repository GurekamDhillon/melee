//! Segment-vs-map collision queries over the joint list, from src/melee/mp/mplib.c:
//! mpCheckFloor/Ceiling/LeftWall/RightWall and their `Remap` variants, mpLib_800511A4_RightWall,
//! mpLib_800515A0_LeftWall, mpLib_8005199C_Floor, mpLib_80051BA8_Floor, mpCheckMultiple,
//! mpCheckAll(Remap), and the joint bounding-box culling (mpBoundingCheck{,2,3},
//! mpUncheckBounding, mpCheckedBounding, mpScriptInvalidateBounding).
//!
//! The C globals `groundCollJoint`, `jointListStart` and `didCheckBounding` live in
//! [`GroundColl`]. The PC build's `ScriptGame_StageJointActive` checks are the
//! `GroundColl::joint_active` hook, at exactly the C's places.
//!
//! Each C function walks a joint's line group and then, through a `goto` back into the loop
//! body, its Dynamic group; the goto runs the body once before testing the count. That is kept
//! (see [`GroundColl::joint_lines`]).
//!
//! FMA (`python tools/rust-port/fma_sites.py -v --func 'mpCheck|mpLib_8005' src/melee/mp/mplib.c`):
//! mpCheckFloor / mpCheckCeiling fuse `SQ(dx) + SQ(dy)` (2 each); the four Remap variants fuse
//! only the direction dot product (their `dist2 = dx2 + dy2` is split over statements);
//! mpCheckLeftWall / mpCheckRightWall fuse nothing; mpLib_800511A4/800515A0 fuse 6 (3 per
//! vertex); mpLib_8005199C fuses `dy / dx * (x - x0) + y0`; the rest fuse nothing.

use super::geom::{mp_line_intersection, mp_line_intersection_h, mp_line_intersection_v, mp_remap2d};
use super::lines::GroundColl;
use super::types::*;
use crate::vec::{ps_vec_normalize, Vec2, Vec3};
use melee_core::fp::fmadds;

/// `F32_MAX` (src/Runtime/platform.h).
const F32_MAX: f32 = 3.4028235e38;

/// Native `CollJoint` as the collision queries use it. `next`/`inner` become indices
/// (-1 = NULL); the HSD_JObj / callback fields stay on [`CollJoint`] (raw, group 1/3/5).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CollJointRt {
    pub next: i32,
    pub inner: i32,
    pub flags: u32,
    /// `xE` (set when the joint's lines changed; read by mpcoll/ground code).
    pub x_e: bool,
    pub bounding_min: Vec2,
    pub bounding_max: Vec2,
}
melee_core::gc_layout!(CollJointRt, "CollJoint", size 0x34, align 4, [
    next @ 0x0: 4 "CollJoint*",
    inner @ 0x4: 4 "MapJoint*",
    flags @ 0x8: 4 "u32",
    x_e @ 0xE: 1 "bool",
    bounding_min @ 0x10: 8 "Vec2",
    bounding_max @ 0x18: 8 "Vec2",
]);

/// Out-params of the mpCheck* queries (written by the C only when it returns true).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SegHit {
    pub pos: Vec3,
    pub line_id: i32,
    pub flags: u32,
    pub normal: Vec3,
}

/// The fighter callback of mpCheckFloor(Remap): `cb(gobj, line_id)`; the gobj is captured.
pub type LineFilter<'a> = Option<&'a mut dyn FnMut(i32) -> bool>;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Floor,
    Ceiling,
    LeftWall,
    RightWall,
}

impl Kind {
    fn group(self) -> usize {
        match self {
            Kind::Floor => MAP_LINE_GROUP_FLOOR,
            Kind::Ceiling => MAP_LINE_GROUP_CEILING,
            Kind::LeftWall => MAP_LINE_GROUP_LEFT_WALL,
            Kind::RightWall => MAP_LINE_GROUP_RIGHT_WALL,
        }
    }
    fn bit(self) -> u32 {
        match self {
            Kind::Floor => COLL_LINE_FLOOR,
            Kind::Ceiling => COLL_LINE_CEILING,
            Kind::LeftWall => COLL_LINE_LEFT_WALL,
            Kind::RightWall => COLL_LINE_RIGHT_WALL,
        }
    }
    fn flat_normal(self) -> Vec3 {
        let (x, y) = match self {
            Kind::Floor => (0.0, 1.0),
            Kind::Ceiling => (0.0, -1.0),
            Kind::LeftWall => (-1.0, 0.0),
            Kind::RightWall => (1.0, 0.0),
        };
        Vec3 { x, y, z: 0.0 }
    }
}

#[inline]
fn sq(x: f32) -> f32 {
    x * x
}

/// C `ABS(x) > 0.0001` (f32 ABS, double compare).
#[inline]
fn abs_gt(x: f32) -> bool {
    let a = if x < 0.0 { -x } else { x };
    (a as f64) > 0.0001
}

fn usable(flags: u32, bit: u32) -> bool {
    flags & bit != 0 && flags & LINE_FLAG_ENABLED != 0 && flags & LINE_FLAG_EMPTY == 0
}

impl GroundColl {
    /// The line ids a joint's group walk visits: the group's range, then (if the Dynamic count
    /// is nonzero) the Dynamic range entered by the C's `goto`, which runs the body once before
    /// the `i < count` test.
    pub(crate) fn joint_lines(&self, joint: usize, group: usize) -> Vec<i32> {
        let mj = &self.map_joints[self.joints[joint].inner as usize];
        let (s, c) = (mj.ranges[group].start as i32, mj.ranges[group].count as i32);
        let mut v: Vec<i32> = (0..c.max(0)).map(|i| s + i).collect();
        let dc = mj.ranges[MAP_LINE_GROUP_DYNAMIC].count as i32;
        if dc != 0 {
            let ds = mj.ranges[MAP_LINE_GROUP_DYNAMIC].start as i32;
            v.push(ds);
            v.extend((1..dc.max(1)).map(|i| ds + i));
        }
        v
    }

    /// Joint ids in list order (`for (joint = jointListStart; joint; joint = joint->next)`).
    fn joint_order(&self) -> Vec<usize> {
        let mut v = Vec::new();
        let mut j = self.joint_list_start;
        while j != -1 {
            v.push(j as usize);
            j = self.joints[j as usize].next;
        }
        v
    }

    /// The checks every query applies to a joint before walking it.
    fn joint_skipped(&self, j: usize, joint_id_skip: i32, joint_id_only: i32) -> bool {
        !(self.joint_active)(j as i32)
            || self.joints[j].flags & COLL_JOINT_TOO_FAR != 0
            || joint_id_skip == j as i32
            || (joint_id_only != -1 && joint_id_only != j as i32)
    }

    fn raw_xy(&self, line_id: i32) -> (f32, f32, f32, f32) {
        let l = *self.map_line(line_id);
        let (x0, y0) = self.pos(l.v0_idx);
        let (x1, y1) = self.pos(l.v1_idx);
        (x0, y0, x1, y1)
    }

    /// `mpScriptInvalidateBounding`
    pub fn mp_script_invalidate_bounding(&mut self) {
        self.did_check_bounding = false;
    }

    /// `mpCheckedBounding`
    pub fn mp_checked_bounding(&self) -> bool {
        self.did_check_bounding
    }

    /// `mpBoundingCheck`: mark joints whose bounding box misses the given box as TooFar.
    pub fn mp_bounding_check(&mut self, left: f32, bottom: f32, right: f32, top: f32) {
        for j in self.joint_order() {
            let active = (self.joint_active)(j as i32);
            let c = &mut self.joints[j];
            if active && c.flags & COLL_JOINT_ENABLED != 0 && c.flags & COLL_JOINT_HIDDEN == 0 {
                if c.flags & COLL_JOINT_B10 != 0 {
                    c.flags &= !COLL_JOINT_TOO_FAR;
                } else if left > c.bounding_max.x
                    || right < c.bounding_min.x
                    || bottom > c.bounding_max.y
                    || top < c.bounding_min.y
                {
                    c.flags |= COLL_JOINT_TOO_FAR;
                } else {
                    c.flags &= !COLL_JOINT_TOO_FAR;
                }
            } else {
                c.flags |= COLL_JOINT_TOO_FAR;
            }
        }
        self.did_check_bounding = true;
    }

    /// `mpBoundingCheck2`: box around two points.
    pub fn mp_bounding_check2(&mut self, x1: f32, y1: f32, x2: f32, y2: f32) {
        let (left, right) = if x1 > x2 { (x2, x1) } else { (x1, x2) };
        let (bottom, top) = if y1 > y2 { (y2, y1) } else { (y1, y2) };
        self.mp_bounding_check(left, bottom, right, top);
    }

    /// `mpBoundingCheck3`: box around four points.
    #[allow(clippy::too_many_arguments)]
    pub fn mp_bounding_check3(
        &mut self, x0: f32, y0: f32, x1: f32, y1: f32, x2: f32, y2: f32, x3: f32, y3: f32,
    ) {
        let (mut left, mut right) = if x0 > x1 { (x1, x0) } else { (x0, x1) };
        let (mut bottom, mut top) = if y0 > y1 { (y1, y0) } else { (y0, y1) };
        for (x, y) in [(x2, y2), (x3, y3)] {
            if right < x {
                right = x;
            } else if left > x {
                left = x;
            }
            if top < y {
                top = y;
            } else if bottom > y {
                bottom = y;
            }
        }
        self.mp_bounding_check(left, bottom, right, top);
    }

    /// `mpUncheckBounding`
    pub fn mp_uncheck_bounding(&mut self) {
        for j in self.joint_order() {
            self.joints[j].flags &= !COLL_JOINT_TOO_FAR;
        }
        self.did_check_bounding = false;
    }

    /// Shared body of the eight mpCheck{Floor,Ceiling,LeftWall,RightWall}{,Remap}.
    #[allow(clippy::too_many_arguments)]
    fn check_kind(
        &mut self, kind: Kind, remap: bool, ax: f32, ay: f32, bx: f32, by: f32,
        y_offset: Option<f32>, line_id_skip: i32, joint_id_skip: i32, joint_id_only: i32,
        mut cb: LineFilter,
    ) -> Option<SegHit> {
        let mut min_dist2 = F32_MAX;
        let mut hit = None;
        let already_checked = self.mp_checked_bounding();
        if !already_checked {
            self.mp_bounding_check2(ax, ay, bx, by);
        }
        let (old_x, old_y) = (ax, ay);
        let (mut ax, mut ay) = (ax, ay);
        // Floor and Ceiling use the ledge-extended ends, except mpCheckFloorRemap.
        let extended = kind == Kind::Ceiling || (kind == Kind::Floor && !remap);
        let horizontal = matches!(kind, Kind::Floor | Kind::Ceiling);
        for j in self.joint_order() {
            if self.joint_skipped(j, joint_id_skip, joint_id_only) {
                continue;
            }
            let jflags = self.joints[j].flags;
            for id in self.joint_lines(j, kind.group()) {
                if kind == Kind::Floor {
                    if let Some(cb) = cb.as_mut() {
                        if !cb(id) {
                            continue;
                        }
                    }
                    if line_id_skip == id {
                        continue;
                    }
                }
                let lflags = self.flags(id);
                if !usable(lflags, kind.bit()) {
                    continue;
                }
                let (x0, mut y0, x1, mut y1) =
                    if extended { self.mp_lib_8004ed5c(id) } else { self.raw_xy(id) };
                if let Some(off) = y_offset {
                    y0 += off;
                    y1 += off;
                }
                let (mut dx, mut dy) = (0.0f32, 0.0f32);
                if remap {
                    if jflags & (COLL_JOINT_B10 | COLL_JOINT_B9 | COLL_JOINT_B8) != 0 {
                        let l = *self.map_line(id);
                        let v0 = self.vtx[l.v0_idx as usize];
                        let v1 = self.vtx[l.v1_idx as usize];
                        (ax, ay) =
                            mp_remap2d(v0.x10, v0.x14, v1.x10, v1.x14, x0, y0, x1, y1, old_x, old_y);
                    } else {
                        ax = old_x;
                        ay = old_y;
                    }
                    dx = bx - ax;
                    dy = by - ay;
                }
                let sloped = if horizontal { abs_gt(y0 - y1) } else { abs_gt(x0 - x1) };
                let (p, normal) = if sloped {
                    match mp_line_intersection(x0, y0, x1, y1, ax, ay, bx, by) {
                        Some(p) => (p, None),
                        None => continue,
                    }
                } else {
                    let p = match kind {
                        Kind::Floor if ay >= by => mp_line_intersection_h(x0, y0, x1, ax, ay, bx, by),
                        Kind::Ceiling if ay <= by => mp_line_intersection_h(x0, y0, x1, ax, ay, bx, by),
                        Kind::LeftWall if ax <= bx => mp_line_intersection_v(x0, y0, y1, ax, ay, bx, by),
                        Kind::RightWall if ax >= bx => mp_line_intersection_v(x0, y0, y1, ax, ay, bx, by),
                        _ => None,
                    };
                    match p {
                        Some(p) => (p, Some(kind.flat_normal())),
                        None => continue,
                    }
                };
                let (ix, iy) = p;
                let dist2 = if remap {
                    let dx2 = sq(ix - old_x);
                    let dy2 = sq(iy - old_y);
                    let d = dx2 + dy2;
                    if fmadds(dx, ix - old_x, dy * (iy - old_y)) < 0.0 {
                        -d
                    } else {
                        d
                    }
                } else if horizontal {
                    fmadds(ix - ax, ix - ax, sq(iy - ay))
                } else {
                    let dx2 = sq(ix - ax);
                    let dy2 = sq(iy - ay);
                    dx2 + dy2
                };
                if min_dist2 > dist2 {
                    min_dist2 = dist2;
                    hit = Some(SegHit {
                        pos: Vec3 { x: ix, y: iy, z: 0.0 },
                        line_id: id,
                        flags: self.map_line(id).lo_flags as u32,
                        normal: normal.unwrap_or_else(|| {
                            ps_vec_normalize(Vec3 { x: -(y1 - y0), y: x1 - x0, z: 0.0 })
                        }),
                    });
                }
            }
        }
        if !already_checked {
            self.mp_uncheck_bounding();
        }
        hit
    }

    /// `mpCheckFloor`
    #[allow(clippy::too_many_arguments)]
    pub fn mp_check_floor(
        &mut self, ax: f32, ay: f32, bx: f32, by: f32, y_offset: f32, line_id_skip: i32,
        joint_id_skip: i32, joint_id_only: i32, cb: LineFilter,
    ) -> Option<SegHit> {
        self.check_kind(Kind::Floor, false, ax, ay, bx, by, Some(y_offset), line_id_skip,
            joint_id_skip, joint_id_only, cb)
    }

    /// `mpCheckFloorRemap`
    #[allow(clippy::too_many_arguments)]
    pub fn mp_check_floor_remap(
        &mut self, ax: f32, ay: f32, bx: f32, by: f32, y_offset: f32, line_id_skip: i32,
        joint_id_skip: i32, joint_id_only: i32, cb: LineFilter,
    ) -> Option<SegHit> {
        self.check_kind(Kind::Floor, true, ax, ay, bx, by, Some(y_offset), line_id_skip,
            joint_id_skip, joint_id_only, cb)
    }

    /// `mpCheckCeiling`
    pub fn mp_check_ceiling(
        &mut self, ax: f32, ay: f32, bx: f32, by: f32, joint_id_skip: i32, joint_id_only: i32,
    ) -> Option<SegHit> {
        self.check_kind(Kind::Ceiling, false, ax, ay, bx, by, None, -1, joint_id_skip, joint_id_only, None)
    }

    /// `mpCheckCeilingRemap`
    pub fn mp_check_ceiling_remap(
        &mut self, ax: f32, ay: f32, bx: f32, by: f32, joint_id_skip: i32, joint_id_only: i32,
    ) -> Option<SegHit> {
        self.check_kind(Kind::Ceiling, true, ax, ay, bx, by, None, -1, joint_id_skip, joint_id_only, None)
    }

    /// `mpCheckLeftWall`
    pub fn mp_check_left_wall(
        &mut self, ax: f32, ay: f32, bx: f32, by: f32, joint_id_skip: i32, joint_id_only: i32,
    ) -> Option<SegHit> {
        self.check_kind(Kind::LeftWall, false, ax, ay, bx, by, None, -1, joint_id_skip, joint_id_only, None)
    }

    /// `mpCheckLeftWallRemap`
    pub fn mp_check_left_wall_remap(
        &mut self, ax: f32, ay: f32, bx: f32, by: f32, joint_id_skip: i32, joint_id_only: i32,
    ) -> Option<SegHit> {
        self.check_kind(Kind::LeftWall, true, ax, ay, bx, by, None, -1, joint_id_skip, joint_id_only, None)
    }

    /// `mpCheckRightWall`
    pub fn mp_check_right_wall(
        &mut self, ax: f32, ay: f32, bx: f32, by: f32, joint_id_skip: i32, joint_id_only: i32,
    ) -> Option<SegHit> {
        self.check_kind(Kind::RightWall, false, ax, ay, bx, by, None, -1, joint_id_skip, joint_id_only, None)
    }

    /// `mpCheckRightWallRemap`
    pub fn mp_check_right_wall_remap(
        &mut self, ax: f32, ay: f32, bx: f32, by: f32, joint_id_skip: i32, joint_id_only: i32,
    ) -> Option<SegHit> {
        self.check_kind(Kind::RightWall, true, ax, ay, bx, by, None, -1, joint_id_skip, joint_id_only, None)
    }

    /// Shared body of mpLib_800511A4_RightWall / mpLib_800515A0_LeftWall: does a wall vertex's
    /// motion (previous position `x10`,`x14` remapped from segment a to b, to its current
    /// position) cross segment b? Returns the nearest such line (the C's `line_id_out`).
    #[allow(clippy::too_many_arguments)]
    fn wall_sweep(
        &mut self, kind: Kind, ax: f32, ay: f32, bx: f32, by: f32, cx: f32, cy: f32, dx: f32,
        dy: f32, joint_id_skip: i32, joint_id_only: i32,
    ) -> Option<i32> {
        let mut min_dist2 = F32_MAX;
        let mut out = None;
        let already_checked = self.mp_checked_bounding();
        if !already_checked {
            self.mp_bounding_check3(ax, ay, bx, by, cx, cy, dx, dy);
        }
        for j in self.joint_order() {
            if self.joint_skipped(j, joint_id_skip, joint_id_only) {
                continue;
            }
            for id in self.joint_lines(j, kind.group()) {
                if !usable(self.flags(id), kind.bit()) {
                    continue;
                }
                let l = *self.map_line(id);
                for vi in [l.v0_idx, l.v1_idx] {
                    let v = self.vtx[vi as usize];
                    let (x0, y0, x1, y1) = (v.pos.x, v.pos.y, v.x10, v.x14);
                    let (x, y) = mp_remap2d(ax, ay, bx, by, cx, cy, dx, dy, x1, y1);
                    let vdx = x0 - x;
                    let vdy = y0 - y;
                    if fmadds(vdx, vdx, sq(vdy)) > 0.001f32 {
                        if let Some((ix, iy)) = mp_line_intersection(cx, cy, dx, dy, x, y, x0, y0) {
                            let mut dist2 = fmadds(ix - x1, ix - x1, sq(iy - y1));
                            if fmadds(vdx, ix - x1, vdy * (iy - y1)) < 0.0 {
                                dist2 = -dist2;
                            }
                            if min_dist2 > dist2 {
                                min_dist2 = dist2;
                                out = Some(id);
                            }
                        }
                    }
                }
            }
        }
        if !already_checked {
            self.mp_uncheck_bounding();
        }
        out
    }

    /// `mpLib_800511A4_RightWall`
    #[allow(clippy::too_many_arguments)]
    pub fn mp_lib_800511a4_right_wall(
        &mut self, ax: f32, ay: f32, bx: f32, by: f32, cx: f32, cy: f32, dx: f32, dy: f32,
        joint_id_skip: i32, joint_id_only: i32,
    ) -> Option<i32> {
        self.wall_sweep(Kind::RightWall, ax, ay, bx, by, cx, cy, dx, dy, joint_id_skip, joint_id_only)
    }

    /// `mpLib_800515A0_LeftWall`
    #[allow(clippy::too_many_arguments)]
    pub fn mp_lib_800515a0_left_wall(
        &mut self, a0x: f32, a0y: f32, a1x: f32, a1y: f32, b0x: f32, b0y: f32, b1x: f32,
        b1y: f32, joint_id_skip: i32, joint_id_only: i32,
    ) -> Option<i32> {
        self.wall_sweep(Kind::LeftWall, a0x, a0y, a1x, a1y, b0x, b0y, b1x, b1y, joint_id_skip,
            joint_id_only)
    }

    /// `mpLib_8005199C_Floor`: the first floor (list order) at or below `vec` within its x span.
    /// Returns the line id or -1.
    pub fn mp_lib_8005199c_floor(&mut self, vec: Vec3, joint_id_skip: i32, joint_id_only: i32) -> i32 {
        let (x, y) = (vec.x, vec.y);
        let already_checked = self.mp_checked_bounding();
        if !already_checked {
            self.mp_bounding_check2(x, y, x, y - 30000.0);
        }
        let mut line_id = -1;
        'joints: for j in self.joint_order() {
            if self.joint_skipped(j, joint_id_skip, joint_id_only) {
                continue;
            }
            for id in self.joint_lines(j, MAP_LINE_GROUP_FLOOR) {
                if !usable(self.flags(id), COLL_LINE_FLOOR) {
                    continue;
                }
                let (x0, y0, x1, y1) = self.raw_xy(id);
                if x >= x0 && x <= x1 {
                    if y >= y0 && y >= y1 {
                        line_id = id;
                        break 'joints;
                    }
                    if abs_gt(x1 - x0) {
                        let dx = x1 - x0;
                        let dy = y1 - y0;
                        if y >= fmadds(dy / dx, x - x0, y0) {
                            line_id = id;
                            break 'joints;
                        }
                    }
                }
            }
        }
        if !already_checked {
            self.mp_uncheck_bounding();
        }
        line_id
    }

    /// `mpLib_80051BA8_Floor`: the outermost ledge floor (`dir` > 0: leftmost v0, < 0: rightmost
    /// v1) whose bounding box overlaps the given box. Returns the line id (-1 for none) and, when
    /// found, the C's `out_vec` (x clamped to [left, right]). `dir == 0` is the C's HSD_ASSERT.
    #[allow(clippy::too_many_arguments)]
    pub fn mp_lib_80051ba8_floor(
        &mut self, line_id_skip: i32, joint_id_skip: i32, joint_id_only: i32, dir: i32,
        left: f32, bottom: f32, right: f32, top: f32,
    ) -> (i32, Option<Vec3>) {
        let mut min = if dir > 0 {
            F32_MAX
        } else if dir < 0 {
            -F32_MAX
        } else {
            panic!("mplib.c:3821: 0");
        };
        let mut ledge_id = -1;
        let (mut out_x, mut out_y) = (0.0f32, 0.0f32);
        let already_checked = self.mp_checked_bounding();
        if !already_checked {
            self.mp_bounding_check(left, bottom, right, top);
        }
        for j in self.joint_order() {
            if self.joint_skipped(j, joint_id_skip, joint_id_only) {
                continue;
            }
            for id in self.joint_lines(j, MAP_LINE_GROUP_FLOOR) {
                if line_id_skip == id || !usable(self.flags(id), COLL_LINE_FLOOR) {
                    continue;
                }
                if self.map_line(id).lo_flags as u32 & LINE_FLAG_LEDGE == 0 {
                    continue;
                }
                let (x0, y0, x1, y1) = self.raw_xy(id);
                let (line_left, line_right) = if x0 > x1 { (x1, x0) } else { (x0, x1) };
                let (line_bottom, line_top) = if y0 > y1 { (y1, y0) } else { (y0, y1) };
                let fabs = |v: f32| if v < 0.0 { -v } else { v };
                let dist_h = fabs((line_right + line_left) - (right + left));
                if dist_h < (line_right - line_left) + (right - left) {
                    let dist_v = fabs((line_top + line_bottom) - (top + bottom));
                    if dist_v < (line_top - line_bottom) + (top - bottom) {
                        if dir > 0 {
                            if min > x0 {
                                min = x0;
                                ledge_id = id;
                                (out_x, out_y) = (x0, y0);
                            }
                        } else if min < x1 {
                            min = x1;
                            ledge_id = id;
                            (out_x, out_y) = (x1, y1);
                        }
                    }
                }
            }
        }
        if !already_checked {
            self.mp_uncheck_bounding();
        }
        if ledge_id != -1 {
            if out_x > right {
                out_x = right;
            } else if out_x < left {
                out_x = left;
            }
            return (ledge_id, Some(Vec3 { x: out_x, y: out_y, z: 0.0 }));
        }
        (ledge_id, None)
    }

    /// `mpCheckMultiple`: nearest hit among the kinds selected by `checks` (1 floor, 2 ceiling,
    /// 4 left wall, 8 right wall; 0x10 selects the Remap variants).
    #[allow(clippy::too_many_arguments)]
    pub fn mp_check_multiple(
        &mut self, x0: f32, y0: f32, x1: f32, y1: f32, checks: u32, joint_id_skip: i32,
        joint_id_only: i32,
    ) -> Option<SegHit> {
        let mut min_dist2 = F32_MAX;
        let mut best: Option<SegHit> = None;
        let already_checked = self.mp_checked_bounding();
        if !already_checked {
            self.mp_bounding_check2(x0, y0, x1, y1);
        }
        let remap = checks & 0x10 != 0;
        let (js, jo) = (joint_id_skip, joint_id_only);
        for (bit, kind) in [(1, Kind::Floor), (2, Kind::Ceiling), (4, Kind::LeftWall), (8, Kind::RightWall)] {
            if checks & bit == 0 {
                continue;
            }
            let off = if kind == Kind::Floor { Some(0.0) } else { None };
            if let Some(t) = self.check_kind(kind, remap, x0, y0, x1, y1, off, -1, js, jo, None) {
                let dx = sq(t.pos.x - x0);
                let dy = sq(t.pos.y - y0);
                // The floor result is taken unconditionally (it is checked first).
                if kind == Kind::Floor || min_dist2 > dx + dy {
                    min_dist2 = dx + dy;
                    best = Some(t);
                }
            }
        }
        if !already_checked {
            self.mp_uncheck_bounding();
        }
        if min_dist2 < F32_MAX {
            best
        } else {
            None
        }
    }

    /// `mpCheckAllRemap`
    pub fn mp_check_all_remap(
        &mut self, joint_id_skip: i32, joint_id_only: i32, x0: f32, y0: f32, x1: f32, y1: f32,
    ) -> Option<SegHit> {
        self.mp_check_multiple(x0, y0, x1, y1, 0x1F, joint_id_skip, joint_id_only)
    }

    /// `mpCheckAll`
    pub fn mp_check_all(
        &mut self, joint_id_skip: i32, joint_id_only: i32, x0: f32, y0: f32, x1: f32, y1: f32,
    ) -> Option<SegHit> {
        self.mp_check_multiple(x0, y0, x1, y1, 0xF, joint_id_skip, joint_id_only)
    }
}
