//! Joint seam stitching and the floor-bounds pass, from src/melee/mp/mplib.c: mpLib_800581DC,
//! mpLib_80058560, mpLib_80058614_Floor, mpLib_800587FC, mpLib_80058820.
//!
//! The PC build's `ScriptGame_StageJoint` checks are the `GroundColl::stage_joint` hook, and its
//! `ScriptGame_StageJointActive` checks are `joint_active`, at exactly the C's places.
//! `mpLib_80458868` is `GroundColl::coll_box`.
//!
//! FMA (`python tools/rust-port/fma_sites.py -v --func 'mpLib_8005(81DC|8560|8614|87FC|8820)'
//! src/melee/mp/mplib.c`): none of these fuse anything.

use super::lines::GroundColl;
use super::types::*;

/// `F32_MAX` (src/Runtime/platform.h).
const F32_MAX: f32 = 3.4028235e38;

/// `grDynamicAttr_801CA224` (src/melee/gr/grdynamicattr.c).
/// TODO(group2: gr/grdynamicattr.c): not ported yet; a no-op here.
pub(crate) fn gr_dynamic_attr_801ca224(_g: &mut GroundColl) {}

/// `GObj_Create(1, 6, 0)` + `HSD_GObj_SetupProc(gobj, mpLib_800587FC, 4)`.
/// TODO(group1: baselib/gobj.c): the GObj system is not ported; nothing is registered.
pub(crate) fn gobj_create_mplib_proc() {}

impl GroundColl {
    fn ml(&self, line: i32) -> usize {
        self.lines[line as usize].x0 as usize
    }

    /// The first half of mpLib_800581DC for one joint group: drop every `*_id1` link whose
    /// target does not link back.
    fn unlink_one_way(&mut self, r: MapLineRange) {
        let count = r.count as i32;
        let mut idx = r.start as i32;
        for _ in 0..count.max(0) {
            let m = self.ml(idx);
            let temp = self.map_lines[m].prev_id1 as i32;
            if temp != -1 {
                let back = self.map_lines[self.ml(temp)].next_id1 as i32;
                if back != -1 && idx != back {
                    self.map_lines[m].prev_id1 = -1;
                }
            }
            let temp = self.map_lines[m].next_id1 as i32;
            if temp != -1 {
                let back = self.map_lines[self.ml(temp)].prev_id1 as i32;
                if back != -1 && idx != back {
                    self.map_lines[m].next_id1 = -1;
                }
            }
            idx += 1;
        }
    }

    /// `mpLib_800581DC`: stitch the `*_id1` links between two joints whose vertices touch
    /// (within 2 units on both axes).
    pub fn mp_lib_800581dc(&mut self, joint_id0: i32, joint_id1: i32) {
        if (self.stage_joint)(joint_id0) || (self.stage_joint)(joint_id1) {
            return;
        }
        let j0 = self.joints[joint_id0 as usize].inner as usize;
        let j1 = self.joints[joint_id1 as usize].inner as usize;
        for i in 0..MAP_LINE_GROUP_COUNT {
            let r = self.map_joints[j0].ranges[i];
            self.unlink_one_way(r);
            let r = self.map_joints[j1].ranges[i];
            self.unlink_one_way(r);
        }

        let mj0 = self.map_joints[j0];
        let mj1 = self.map_joints[j1];
        let mut vstart0 = mj0.vtx_start as i32;
        let vcount0 = mj0.vtx_count as i32;
        for _ in 0..vcount0.max(0) {
            let p0 = self.vtx[vstart0 as usize].pos;
            let vcount1 = mj1.vtx_count as i32;
            let mut vid = mj1.vtx_start as i32;
            for _ in 0..vcount1.max(0) {
                let p1 = self.vtx[vid as usize].pos;
                let (dx, dy) = (p0.x - p1.x, p0.y - p1.y);
                // C: !(ABS(dx) < 2.0) || !(ABS(dy) < 2.0) -> continue (f32 ABS, double compare;
                // 2.0 is exact in f32, so an f32 compare is the same).
                let near = |d: f32| (if d < 0.0 { -d } else { d }) < 2.0;
                if near(dx) && near(dy) {
                    for g0 in 0..MAP_LINE_GROUP_COUNT {
                        let lcount = mj0.ranges[g0].count as i32;
                        let mut l0 = mj0.ranges[g0].start as i32;
                        for _ in 0..lcount.max(0) {
                            let m0 = self.map_lines[self.ml(l0)];
                            if vstart0 == m0.v0_idx as i32 {
                                for g1 in 0..MAP_LINE_GROUP_COUNT {
                                    let lc1 = mj1.ranges[g1].count as i32;
                                    let mut l1 = mj1.ranges[g1].start as i32;
                                    for _ in 0..lc1.max(0) {
                                        if vid == self.map_lines[self.ml(l1)].v1_idx as i32 {
                                            let a = self.ml(l0);
                                            self.map_lines[a].prev_id1 = l1 as i16;
                                            let b = self.ml(l1);
                                            self.map_lines[b].next_id1 = l0 as i16;
                                        }
                                        l1 += 1;
                                    }
                                }
                            } else if vstart0 == m0.v1_idx as i32 {
                                for g1 in 0..MAP_LINE_GROUP_COUNT {
                                    let lc1 = mj1.ranges[g1].count as i32;
                                    let mut l1 = mj1.ranges[g1].start as i32;
                                    for _ in 0..lc1.max(0) {
                                        if vid == self.map_lines[self.ml(l1)].v0_idx as i32 {
                                            let a = self.ml(l0);
                                            self.map_lines[a].next_id1 = l1 as i16;
                                            let b = self.ml(l1);
                                            self.map_lines[b].prev_id1 = l0 as i16;
                                        }
                                        l1 += 1;
                                    }
                                }
                            }
                            l0 += 1;
                        }
                    }
                }
                vid += 1;
            }
            vstart0 += 1;
        }
    }

    /// `mpLib_80058560`: stitch every pair of enabled, visible (and, on PC, active) joints.
    pub fn mp_lib_80058560(&mut self) {
        let n = self.joints.len() as i32;
        let live = |g: &GroundColl, j: i32| {
            let f = g.joints[j as usize].flags;
            f & COLL_JOINT_ENABLED != 0 && f & COLL_JOINT_HIDDEN == 0
        };
        for i in 0..n - 1 {
            for j in i + 1..n {
                if (self.joint_active)(i) && (self.joint_active)(j) && live(self, i) && live(self, j) {
                    self.mp_lib_800581dc(i, j);
                }
            }
        }
    }

    /// `mpLib_80058614_Floor`: if any joint has `xE` set, clear every `xE` and recompute
    /// `mpLib_80458868[1]` as the bounds of all enabled floor (and Dynamic) line endpoints.
    pub fn mp_lib_80058614_floor(&mut self) {
        let n = self.joints.len();
        let mut c = 0;
        while c < n {
            if (self.stage_joint)(c as i32) {
                self.joints[c].x_e = false;
                c += 1;
                continue;
            }
            if self.joints[c].x_e {
                break;
            }
            c += 1;
        }
        if c == n {
            return;
        }
        let mut b = self.coll_box[1];
        b.right = -F32_MAX;
        b.top = -F32_MAX;
        b.left = F32_MAX;
        b.bottom = F32_MAX;
        for i in 0..n {
            self.joints[i].x_e = false;
            if (self.stage_joint)(i as i32) {
                continue;
            }
            let f = self.joints[i].flags;
            if f & COLL_JOINT_ENABLED == 0 || f & COLL_JOINT_HIDDEN != 0 {
                continue;
            }
            for l in self.joint_lines(i, MAP_LINE_GROUP_FLOOR) {
                let lf = self.flags(l);
                if lf & COLL_LINE_FLOOR == 0 || lf & LINE_FLAG_ENABLED == 0 {
                    continue;
                }
                let m = *self.map_line(l);
                let (x0, y0) = self.pos(m.v0_idx);
                let (x1, y1) = self.pos(m.v1_idx);
                for (x, y) in [(x0, y0), (x1, y1)] {
                    if b.top < y {
                        b.top = y;
                    }
                    if b.bottom > y {
                        b.bottom = y;
                    }
                    if b.right < x {
                        b.right = x;
                    }
                    if b.left > x {
                        b.left = x;
                    }
                }
            }
        }
        self.coll_box[1] = b;
    }

    /// `mpLib_800587FC` (the per-frame proc mpLib_80058820 registers).
    pub fn mp_lib_800587fc(&mut self) {
        self.mp_lib_80058614_floor();
        gr_dynamic_attr_801ca224(self);
    }

    /// `mpLib_80058820`: register the bounds proc and reset both boxes to +-10000.
    pub fn mp_lib_80058820(&mut self) {
        gobj_create_mplib_proc();
        let b = &mut self.coll_box;
        b[0].right = 10000.0;
        b[0].top = 10000.0;
        b[0].left = -10000.0;
        b[0].bottom = -10000.0;
        b[1] = b[0];
    }
}
