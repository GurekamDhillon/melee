//! Joint-list edits and per-joint line/vertex updates, from src/melee/mp/mplib.c:
//! mpJointHide, mpJointUnhide, mpJointUpdateDynamics, mpLib_80055E24, mpJointUpdateBounding,
//! mpLib_8005667C, mpJointFromLine, mpLib_80056C54, mpLib_80057424, mpLib_80057528,
//! mpLib_800575B0, mpJointListAdd, mpJointListUnlink, mpLib_80057BC0, mpLib_80057FDC,
//! mpLib_80058044, mpJointSetB10.
//!
//! `mpScriptIslandRefresh` (`mpIsland_8005B334` on the console; on PC it may instead mark the
//! joint dirty in a script batch) is queued in [`GroundColl::island_refresh`]:
//! TODO(group2: mp/mpisland.c + the PC script batch).
//!
//! Not ported here (need HSD_JObj, group 1): mpLib_800552B0, mpLib_80055E9C; the callback
//! setters mpJointSetCb1/2, mpJointGetCb1/2, mpJointClearCb1 and mpLib_8005811C (they move raw
//! fn pointers that the gr/ft/it groups own).
//!
//! FMA (`python tools/rust-port/fma_sites.py -v --func 'mpJoint|mpLib_8005' src/melee/mp/mplib.c`):
//! only mpLib_80056C54 fuses: its 3 `t * (b - a) + a` (f32), plus `sqrtf_store`'s three f64
//! Newton steps (= [`super::geom::sqrtf`]). Everything else here fuses nothing.

use super::geom::sqrtf;
use super::lines::GroundColl;
use super::types::*;
use crate::vec::Vec3;
use melee_core::fp::fmadds;

/// One `mpScriptIslandRefresh(joint, start, count, enabled)` call.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IslandRefresh {
    pub joint: i32,
    pub vtx_start: i32,
    pub vtx_count: i32,
    pub enabled: bool,
}

/// What mpLib_80056C54 writes when it gets past its first two checks.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FloorWalk {
    /// The C return value.
    pub result: bool,
    /// `*line_id_out` (-1 when the walk ended off a floor).
    pub line_id: i32,
    /// `*flags_out` / `*normal_out`: written only when `line_id != -1`.
    pub flags: Option<u32>,
    pub normal: Option<Vec3>,
    /// `*vec_out`.
    pub vec: Vec3,
}

impl GroundColl {
    fn inner(&self, joint_id: i32) -> MapJoint {
        self.map_joints[self.joints[joint_id as usize].inner as usize]
    }

    /// `line->flags |= set; line->flags &= !clear` over one MapLineRange (`while (count-- > 0)`).
    fn range_flags(&mut self, r: MapLineRange, set: u32, clear: u32) {
        let mut count = r.count as i32;
        let mut i = r.start as i32 as usize;
        while count > 0 {
            count -= 1;
            let l = &mut self.lines[i];
            l.flags = (l.flags | set) & !clear;
            i += 1;
        }
    }

    fn all_ranges_flags(&mut self, joint_id: i32, set: u32, clear: u32) {
        // Floor, Ceiling, LeftWall, RightWall, Dynamic (the C's order; disjoint writes anyway).
        for g in [
            MAP_LINE_GROUP_FLOOR,
            MAP_LINE_GROUP_CEILING,
            MAP_LINE_GROUP_LEFT_WALL,
            MAP_LINE_GROUP_RIGHT_WALL,
            MAP_LINE_GROUP_DYNAMIC,
        ] {
            let r = self.inner(joint_id).ranges[g];
            self.range_flags(r, set, clear);
        }
    }

    fn refresh(&mut self, joint_id: i32, enabled: bool) {
        let mj = self.inner(joint_id);
        self.island_refresh.push(IslandRefresh {
            joint: joint_id,
            vtx_start: mj.vtx_start as i32,
            vtx_count: mj.vtx_count as i32,
            enabled,
        });
    }

    /// `!(flags & (Hidden | B11)) && flags & Enabled`
    fn joint_live(&self, joint_id: i32) -> bool {
        let f = self.joints[joint_id as usize].flags;
        f & (COLL_JOINT_HIDDEN | COLL_JOINT_B11) == 0 && f & COLL_JOINT_ENABLED != 0
    }

    /// `mpJointHide`
    pub fn mp_joint_hide(&mut self, joint_id: i32) {
        self.joints[joint_id as usize].flags |= COLL_JOINT_HIDDEN;
        self.all_ranges_flags(joint_id, LINE_FLAG_HIDDEN, 0);
    }

    /// `mpJointUnhide`
    pub fn mp_joint_unhide(&mut self, joint_id: i32) {
        self.joints[joint_id as usize].flags &= !COLL_JOINT_HIDDEN;
        self.all_ranges_flags(joint_id, 0, LINE_FLAG_HIDDEN);
        let mj = self.inner(joint_id);
        let mut v = mj.vtx_start as i32 as usize;
        let mut i = mj.vtx_count as i32;
        while i > 0 {
            let x = &mut self.vtx[v];
            x.x10 = x.pos.x;
            x.x14 = x.pos.y;
            i -= 1;
            v += 1;
        }
    }

    /// `mpJointUpdateDynamics`: reclassify the Dynamic lines by slope.
    pub fn mp_joint_update_dynamics(&mut self, joint_id: i32) {
        const TAN30: f64 = 0.577350295784245;
        const TAN60: f64 = 1.7320508368950045;
        let mj = self.inner(joint_id);
        let jflags = self.joints[joint_id as usize].flags;
        let count = mj.ranges[MAP_LINE_GROUP_DYNAMIC].count as i32;
        let start = mj.ranges[MAP_LINE_GROUP_DYNAMIC].start as i32 as usize;
        for i in 0..count.max(0) as usize {
            let li = start + i;
            let ml = self.lines[li].x0 as usize;
            let m = self.map_lines[ml];
            let v1 = self.vtx[m.v1_idx as usize].pos;
            let v0 = self.vtx[m.v0_idx as usize].pos;
            let dx = v1.x - v0.x;
            let dy = v1.y - v0.y;
            let kind = if dx > 0.0 {
                let s = (dy / dx) as f64;
                if s > TAN60 {
                    COLL_LINE_LEFT_WALL
                } else if s < -TAN60 {
                    COLL_LINE_RIGHT_WALL
                } else {
                    COLL_LINE_FLOOR
                }
            } else if dx < 0.0 {
                let s = (dy / dx) as f64;
                if s > TAN30 {
                    COLL_LINE_RIGHT_WALL
                } else if s < -TAN30 {
                    COLL_LINE_LEFT_WALL
                } else {
                    COLL_LINE_CEILING
                }
            } else if dy > 0.0 {
                COLL_LINE_LEFT_WALL
            } else if dy < 0.0 {
                COLL_LINE_RIGHT_WALL
            } else {
                // HSD_ASSERT(4884, 0)
                panic!("mplib.c:4884: zero-length dynamic line {li}");
            };
            let l = &mut self.lines[li];
            l.flags = (l.flags & !LINE_FLAG_KIND) | kind;
            if jflags & COLL_JOINT_ENABLED != 0 && self.map_lines[ml].lo_flags & 0x400 != 0 {
                if kind & COLL_LINE_FLOOR != 0 {
                    l.flags |= LINE_FLAG_ENABLED | LINE_FLAG_PLATFORM;
                    self.map_lines[ml].lo_flags |= LINE_FLAG_PLATFORM as u16;
                } else {
                    l.flags &= !LINE_FLAG_ENABLED;
                }
            }
        }
    }

    /// `mpLib_80055E24`
    pub fn mp_lib_80055e24(&mut self, joint_id: i32) {
        self.mp_joint_update_dynamics(joint_id);
        let live = self.joint_live(joint_id);
        self.refresh(joint_id, live);
    }

    /// `mpJointUpdateBounding`: grow the joint's box to every vertex +-30.
    pub fn mp_joint_update_bounding(&mut self, joint_id: i32) {
        let mj = self.inner(joint_id);
        let start = mj.vtx_start as i32 as usize;
        let count = mj.vtx_count as i32;
        for i in 0..count.max(0) as usize {
            let p = self.vtx[start + i].pos;
            let j = &mut self.joints[joint_id as usize];
            if j.bounding_min.x > p.x - 30.0 {
                j.bounding_min.x = p.x - 30.0;
            }
            if j.bounding_max.x < p.x + 30.0 {
                j.bounding_max.x = p.x + 30.0;
            }
            if j.bounding_min.y > p.y - 30.0 {
                j.bounding_min.y = p.y - 30.0;
            }
            if j.bounding_max.y < p.y + 30.0 {
                j.bounding_max.y = p.y + 30.0;
            }
        }
    }

    /// `mpLib_8005667C`
    pub fn mp_lib_8005667c(&mut self, joint_id: i32) {
        let live = self.joint_live(joint_id);
        self.refresh(joint_id, live);
    }

    /// `mpJointFromLine`: the first joint (by index, over `joint_count`) whose vertex range holds
    /// the line's v0.
    pub fn mp_joint_from_line(&self, line_id: i32) -> i32 {
        if line_id != -1 {
            self.lineid_check(5459, line_id);
            let v0 = self.map_line(line_id).v0_idx as i32;
            for (i, j) in self.joints.iter().enumerate() {
                let mj = &self.map_joints[j.inner as usize];
                let s = mj.vtx_start as i32;
                if s <= v0 && v0 < s + mj.vtx_count as i32 {
                    return i as i32;
                }
            }
        }
        -1
    }

    /// `mpLib_80056C54`: walk `dist` along the floor chain from `pos` projected onto `line_id`
    /// (forward if `dist > 0`, else backward), crossing walls for at most `wall_max` total.
    /// `None` = the C's early `return false` (nothing written).
    pub fn mp_lib_80056c54(&self, mut line_id: i32, pos: Vec3, mut dist: f32, wall_max: f32) -> Option<FloorWalk> {
        let mut result = true;
        if !self.mp_lib_80054ed8(line_id) {
            return None;
        }
        let sp64 = self.mp_lib_8004dd90_floor(line_id, pos)?.dist;
        let mut sp58 = pos;
        sp58.y += sp64;
        // The shipping build zero-initialises locals (-ftrivial-auto-var-init=zero), so the
        // C's possibly-uninitialised total starts at 0.
        let mut total: f32 = 0.0;
        let mut d: f32;
        let mut sp4c: Vec3;
        let fwd = dist > 0.0;
        if !fwd {
            dist = -dist;
        }
        loop {
            sp4c = if fwd { self.mp_line_get_v1_pos(line_id) } else { self.mp_line_get_v0_pos(line_id) };
            let x = (sp58.x - sp4c.x) * (sp58.x - sp4c.x);
            let y = (sp58.y - sp4c.y) * (sp58.y - sp4c.y);
            d = sqrtf(x + y);
            let kind = self.mp_line_get_kind(line_id);
            let go_on;
            if kind & 0xC != 0 {
                total += d;
                if total > wall_max {
                    result = false;
                    go_on = false;
                } else {
                    go_on = true;
                }
            } else if kind & 2 != 0 {
                result = false;
                go_on = false;
            } else {
                total = 0.0;
                go_on = true;
            }
            if go_on && !(d > dist) {
                let n = if fwd { self.mp_line_get_next(line_id) } else { self.mp_line_get_prev(line_id) };
                if n == -1 {
                    result = false;
                } else {
                    dist -= d;
                    line_id = n;
                    sp58 = sp4c;
                    continue;
                }
            }
            break;
        }
        if line_id != -1 && self.mp_line_get_kind(line_id) & COLL_LINE_FLOOR == 0 {
            line_id = -1;
        }
        let (flags, normal) = if line_id != -1 {
            (Some(self.mp_line_get_flags(line_id)), Some(self.mp_line_get_normal(line_id)))
        } else {
            (None, None)
        };
        let vec = if result {
            if (d as f64) < 0.0001 {
                sp58
            } else {
                let t = dist / d;
                Vec3 {
                    x: fmadds(t, sp4c.x - sp58.x, sp58.x),
                    y: fmadds(t, sp4c.y - sp58.y, sp58.y),
                    z: fmadds(t, sp4c.z - sp58.z, sp58.z),
                }
            }
        } else {
            sp4c
        };
        Some(FloorWalk { result, line_id, flags, normal, vec })
    }

    /// `mpLib_80057424`: snapshot every joint vertex's position into x10/x14.
    pub fn mp_lib_80057424(&mut self, joint_id: i32) {
        let mj = self.inner(joint_id);
        // C: `u32 count = vtx_count` (s16 sign-extended), compared unsigned.
        let count = mj.vtx_count as i32 as u32;
        let start = mj.vtx_start as i32 as usize;
        for i in 0..count as usize {
            let v = &mut self.vtx[start + i];
            v.x10 = v.pos.x;
            v.x14 = v.pos.y;
        }
    }

    fn set_line_enabled(&mut self, line_id: i32, on: bool) {
        let joint_id = self.mp_joint_from_line(line_id);
        if joint_id != -1 {
            let l = &mut self.lines[line_id as usize];
            if on {
                l.flags |= LINE_FLAG_ENABLED;
            } else {
                l.flags &= !LINE_FLAG_ENABLED;
            }
            let b11 = self.joints[joint_id as usize].flags & COLL_JOINT_B11 != 0;
            self.refresh(joint_id, !b11);
            self.joints[joint_id as usize].x_e = true;
        }
    }

    /// `mpLib_80057528`: enable one line.
    pub fn mp_lib_80057528(&mut self, line_id: i32) {
        self.set_line_enabled(line_id, true);
    }

    /// `mpLib_800575B0`: disable one line.
    pub fn mp_lib_800575b0(&mut self, line_id: i32) {
        self.set_line_enabled(line_id, false);
    }

    /// `mpJointListAdd`: enable a joint and append it to the list.
    pub fn mp_joint_list_add(&mut self, joint_id: i32) {
        let j = joint_id as usize;
        if self.joints[j].flags & COLL_JOINT_ENABLED != 0 {
            return;
        }
        self.joints[j].flags |= COLL_JOINT_ENABLED;
        if self.joint_list_start == -1 {
            self.joint_list_start = joint_id;
        } else {
            let e = self.joint_list_end as usize;
            self.joints[e].next = joint_id;
        }
        self.joint_list_end = joint_id;
        self.joints[j].next = -1;
        self.all_ranges_flags(joint_id, LINE_FLAG_ENABLED, 0);
        self.mp_lib_80057424(joint_id);
        let b11 = self.joints[j].flags & COLL_JOINT_B11 != 0;
        self.refresh(joint_id, !b11);
        self.joints[j].x_e = true;
    }

    /// `mpJointListUnlink`
    pub fn mp_joint_list_unlink(&mut self, joint_id: i32) {
        if joint_id == self.joint_list_start {
            if joint_id == self.joint_list_end {
                self.joint_list_end = -1;
                self.joint_list_start = -1;
            } else {
                self.joint_list_start = self.joints[self.joint_list_start as usize].next;
            }
            return;
        }
        let mut cur = self.joint_list_start;
        while cur != -1 {
            let next = self.joints[cur as usize].next;
            if next == joint_id {
                if joint_id == self.joint_list_end {
                    self.joint_list_end = cur;
                }
                self.joints[cur as usize].next = self.joints[next as usize].next;
                return;
            }
            cur = next;
        }
    }

    /// `mpLib_80057BC0`: disable a joint and unlink it.
    pub fn mp_lib_80057bc0(&mut self, joint_id: i32) {
        let j = joint_id as usize;
        if self.joints[j].flags & COLL_JOINT_ENABLED == 0 {
            return;
        }
        self.joints[j].flags &= !COLL_JOINT_ENABLED;
        self.mp_joint_list_unlink(joint_id);
        self.all_ranges_flags(joint_id, 0, LINE_FLAG_ENABLED);
        self.refresh(joint_id, false);
        self.joints[j].x_e = true;
    }

    /// `mpLib_80057FDC`: clear B11.
    pub fn mp_lib_80057fdc(&mut self, joint_id: i32) {
        self.joints[joint_id as usize].flags &= !COLL_JOINT_B11;
        let live = self.joint_live(joint_id);
        self.refresh(joint_id, live);
    }

    /// `mpLib_80058044`: set B11.
    pub fn mp_lib_80058044(&mut self, joint_id: i32) {
        self.joints[joint_id as usize].flags |= COLL_JOINT_B11;
        let live = self.joint_live(joint_id);
        self.refresh(joint_id, live);
    }

    /// `mpJointSetB10`
    pub fn mp_joint_set_b10(&mut self, joint_id: i32) {
        self.joints[joint_id as usize].flags |= COLL_JOINT_B10;
    }
}
