//! Line-graph queries from src/melee/mp/mplib.c (mpLineGetNext .. mpLinesConnected).
//!
//! The C globals `groundCollVtx`, `groundCollLine` and `mpLib_804D64B4` become [`GroundColl`].
//! Every PC-only `ScriptGame_StageLineActive` check is kept exactly where the C has it
//! (including where the C *lacks* it: the `...Vtx` and `mpLineGetNextInline` variants do not
//! gate `next_id0`/`prev_id0`), because the C PC build is the behavioural reference.

use super::types::*;
use crate::vec::{ps_vec_normalize, Vec3};

/// `ScriptGame_StageLineActive` hook (pc/gameworld script layer). Default: every line active.
pub type LineActiveFn = fn(i32) -> bool;

fn all_lines_active(_: i32) -> bool {
    true
}

fn no_stage_joint(_: i32) -> bool {
    false
}

/// The ground collision state mplib keeps in globals.
#[derive(Clone, Debug)]
pub struct GroundColl {
    /// `groundCollVtx`.
    pub vtx: Vec<CollVtx>,
    /// `groundCollLine`.
    pub lines: Vec<CollLine>,
    /// The DAT `MapLine` array that `CollLine::x0` indexes.
    pub map_lines: Vec<MapLine>,
    /// `mpLib_804D64B4->line_count`.
    pub line_count: i32,
    pub line_active: LineActiveFn,
    /// `groundCollJoint` (natively: list links are indices, see [`super::check::CollJointRt`]).
    pub joints: Vec<super::check::CollJointRt>,
    /// The DAT `MapJoint` array that `CollJointRt::inner` indexes.
    pub map_joints: Vec<MapJoint>,
    /// `jointListStart` (-1 = NULL).
    pub joint_list_start: i32,
    /// `didCheckBounding`.
    pub did_check_bounding: bool,
    /// `ScriptGame_StageJointActive` hook (PC build). Default: every joint active.
    pub joint_active: LineActiveFn,
    /// `jointListEnd` (-1 = NULL).
    pub joint_list_end: i32,
    /// Calls to `mpScriptIslandRefresh` (`mpIsland_8005B334`, or the PC script-batch dirty
    /// mark), queued in order. TODO(group2: mpisland.c + script batch): drain into the island
    /// rebuild once mp/mpisland.c is ported.
    pub island_refresh: Vec<super::joint::IslandRefresh>,
    /// `ScriptGame_StageJoint` hook (PC build: the joint belongs to a Lua script room).
    /// Default: no joint is a script joint.
    pub stage_joint: LineActiveFn,
    /// `mpLib_80458868[2]` (stage bounds; [1] is the floor-line box of mpLib_80058614_Floor).
    pub coll_box: [MpCollisionBox; 2],
    /// `mpLib_804D64B4->ranges` (the whole map's per-group line ranges).
    pub map_ranges: [MapLineRange; MAP_LINE_GROUP_COUNT],
    /// mpisland.c state (`mpIsland_80458E88` and its heap segments).
    pub islands: super::island::Islands,
}

impl Default for GroundColl {
    fn default() -> Self {
        Self {
            vtx: Vec::new(),
            lines: Vec::new(),
            map_lines: Vec::new(),
            line_count: 0,
            line_active: all_lines_active,
            joints: Vec::new(),
            map_joints: Vec::new(),
            joint_list_start: -1,
            did_check_bounding: false,
            joint_active: all_lines_active,
            joint_list_end: -1,
            island_refresh: Vec::new(),
            stage_joint: no_stage_joint,
            coll_box: [MpCollisionBox::default(); 2],
            map_ranges: [MapLineRange::default(); MAP_LINE_GROUP_COUNT],
            islands: Default::default(),
        }
    }
}

use melee_runtime::tu::controller::sq;

/// Which side of the line graph a walk follows.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Dir {
    Next,
    Prev,
}

impl GroundColl {
    #[inline]
    pub(crate) fn active(&self, id: i32) -> bool {
        (self.line_active)(id)
    }

    /// `LINEID_CHECK`: HSD_ASSERTREPORT on the console.
    #[inline]
    #[track_caller]
    pub(crate) fn lineid_check(&self, src_line: u32, line_id: i32) {
        if line_id == -1 || line_id >= self.line_count {
            panic!("mplib.c:{src_line}:not found lineID={line_id}");
        }
    }

    #[inline]
    pub fn map_line(&self, line_id: i32) -> &MapLine {
        &self.map_lines[self.lines[line_id as usize].x0 as usize]
    }

    #[inline]
    pub fn flags(&self, line_id: i32) -> u32 {
        self.lines[line_id as usize].flags
    }

    #[inline]
    pub(crate) fn pos(&self, v: u16) -> (f32, f32) {
        let p = self.vtx[v as usize].pos;
        (p.x, p.y)
    }

    /// The `...CheckInline` family: follow `result` (the `*_id1` link) when it is enabled,
    /// visible, active and its touching endpoint is within 2 units; else the `*_id0` link.
    /// `gate_tail`: the PC build checks the fallback link with StageLineActive.
    fn check_link(&self, line: &MapLine, result: i32, dir: Dir, gate_tail: bool) -> i32 {
        if result != -1 {
            let flags = self.flags(result);
            if flags & LINE_FLAG_ENABLED != 0 && flags & LINE_FLAG_HIDDEN == 0 && self.active(result)
            {
                let other = self.map_line(result);
                let (ax, ay, bx, by) = match dir {
                    Dir::Next => {
                        let (ax, ay) = self.pos(line.v1_idx);
                        let (bx, by) = self.pos(other.v0_idx);
                        (ax, ay, bx, by)
                    }
                    Dir::Prev => {
                        let (ax, ay) = self.pos(line.v0_idx);
                        let (bx, by) = self.pos(other.v1_idx);
                        (ax, ay, bx, by)
                    }
                };
                // C: SQ(dx) + SQ(dy) < 4.0. The shipping build contracts the sum into one
                // llvm.fmuladd.f32 (fmadds(dx, dx, dy*dy); tools/rust-port/fma_sites.py), then
                // compares in double (4.0 is exact in f32).
                let (dx, dy) = (ax - bx, ay - by);
                if (melee_core::fp::fmadds(dx, dx, sq(dy)) as f64) < 4.0 {
                    return result;
                }
            }
        }
        let tail = match dir {
            Dir::Next => line.next_id0,
            Dir::Prev => line.prev_id0,
        } as i32;
        if gate_tail && tail != -1 && !self.active(tail) {
            return -1;
        }
        tail
    }

    fn link1(line: &MapLine, dir: Dir) -> i32 {
        match dir {
            Dir::Next => line.next_id1 as i32,
            Dir::Prev => line.prev_id1 as i32,
        }
    }

    /// `mpLineGetNext`
    pub fn mp_line_get_next(&self, line_id: i32) -> i32 {
        self.get_adjacent(line_id, Dir::Next)
    }

    /// `mpLineGetPrev`
    pub fn mp_line_get_prev(&self, line_id: i32) -> i32 {
        self.get_adjacent(line_id, Dir::Prev)
    }

    fn get_adjacent(&self, line_id: i32, dir: Dir) -> i32 {
        let line = *self.map_line(line_id);
        if !self.active(line_id) {
            return -1;
        }
        self.check_link(&line, Self::link1(&line, dir), dir, false)
    }

    /// `mpLineIterNonResult`
    #[inline]
    fn iter_non_result(new_id: i32, line_id: i32) -> i32 {
        if new_id != -1 && new_id != line_id {
            new_id
        } else {
            -1
        }
    }

    /// Body shared by mpLine{Next,Prev}Non{Floor,Ceiling,LeftWall,RightWall}.
    fn walk_non(&self, src_line: u32, line_id: i32, kind: u32, dir: Dir) -> i32 {
        self.lineid_check(src_line, line_id);
        let first = *self.map_line(line_id);
        let mut new_id = self.check_link(&first, Self::link1(&first, dir), dir, true);
        while new_id != -1 && new_id != line_id && self.flags(new_id) & kind != 0 {
            let line = *self.map_line(new_id);
            new_id = self.check_link(&line, Self::link1(&line, dir), dir, false);
        }
        Self::iter_non_result(new_id, line_id)
    }

    pub fn mp_line_next_non_floor(&self, id: i32) -> i32 {
        self.walk_non(4139, id, COLL_LINE_FLOOR, Dir::Next)
    }
    pub fn mp_line_prev_non_floor(&self, id: i32) -> i32 {
        self.walk_non(4148, id, COLL_LINE_FLOOR, Dir::Prev)
    }
    pub fn mp_line_prev_non_ceiling(&self, id: i32) -> i32 {
        self.walk_non(4157, id, COLL_LINE_CEILING, Dir::Prev)
    }
    pub fn mp_line_next_non_ceiling(&self, id: i32) -> i32 {
        self.walk_non(4166, id, COLL_LINE_CEILING, Dir::Next)
    }
    pub fn mp_line_next_non_left_wall(&self, id: i32) -> i32 {
        self.walk_non(4175, id, COLL_LINE_LEFT_WALL, Dir::Next)
    }
    pub fn mp_line_prev_non_left_wall(&self, id: i32) -> i32 {
        self.walk_non(4184, id, COLL_LINE_LEFT_WALL, Dir::Prev)
    }
    pub fn mp_line_prev_non_right_wall(&self, id: i32) -> i32 {
        self.walk_non(4193, id, COLL_LINE_RIGHT_WALL, Dir::Prev)
    }
    pub fn mp_line_next_non_right_wall(&self, id: i32) -> i32 {
        self.walk_non(4202, id, COLL_LINE_RIGHT_WALL, Dir::Next)
    }

    /// `mpLineWalkNon`: follow `*_id0` while the line has `kind`. Unbounded, like the C.
    fn walk_id0(&self, line_id: i32, kind: u32, dir: Dir) -> i32 {
        let step = |id: i32| -> i32 {
            let l = self.map_line(id);
            (if dir == Dir::Next { l.next_id0 } else { l.prev_id0 }) as i32
        };
        let mut new_id = step(line_id);
        while new_id != -1 && self.flags(new_id) & kind != 0 {
            new_id = step(new_id);
        }
        new_id
    }

    pub fn mp_lib_80053394_floor(&self, id: i32) -> i32 {
        self.lineid_check(4252, id);
        self.walk_id0(id, COLL_LINE_FLOOR, Dir::Next)
    }
    pub fn mp_lib_80053448_floor(&self, id: i32) -> i32 {
        self.lineid_check(4261, id);
        self.walk_id0(id, COLL_LINE_FLOOR, Dir::Prev)
    }
    pub fn mp_lib_8005389c_ceiling(&self, id: i32) -> i32 {
        self.lineid_check(4314, id);
        self.walk_id0(id, COLL_LINE_CEILING, Dir::Prev)
    }
    pub fn mp_lib_80053950_ceiling(&self, id: i32) -> i32 {
        self.lineid_check(4323, id);
        self.walk_id0(id, COLL_LINE_CEILING, Dir::Next)
    }

    /// mpLib_800534FC_Floor / 800536CC_Floor / 80053A04_Ceiling / 80053BD4_Ceiling.
    /// `gate_tail`: 80053BD4 uses mpLineGetNextInline (no PC tail gate); the others gate.
    fn walk_same_kind(&self, mut line_id: i32, kind: u32, dir: Dir, gate_tail: bool) -> i32 {
        let first = *self.map_line(line_id);
        let mut new_id = self.check_link(&first, Self::link1(&first, dir), dir, gate_tail);
        while new_id != -1 {
            if self.flags(new_id) & kind == 0 {
                new_id = -1;
            } else if new_id != Self::link1(self.map_line(line_id), dir) {
                line_id = new_id;
                let line = *self.map_line(new_id);
                new_id = self.check_link(&line, Self::link1(&line, dir), dir, gate_tail);
                continue;
            }
            break;
        }
        new_id
    }

    pub fn mp_lib_800534fc_floor(&self, id: i32) -> i32 {
        self.lineid_check(4272, id);
        self.walk_same_kind(id, COLL_LINE_FLOOR, Dir::Next, true)
    }
    pub fn mp_lib_800536cc_floor(&self, id: i32) -> i32 {
        self.lineid_check(4293, id);
        self.walk_same_kind(id, COLL_LINE_FLOOR, Dir::Prev, true)
    }
    pub fn mp_lib_80053a04_ceiling(&self, id: i32) -> i32 {
        self.lineid_check(4334, id);
        self.walk_same_kind(id, COLL_LINE_CEILING, Dir::Prev, true)
    }
    pub fn mp_lib_80053bd4_ceiling(&self, id: i32) -> i32 {
        self.lineid_check(4355, id);
        self.walk_same_kind(id, COLL_LINE_CEILING, Dir::Next, false)
    }

    fn vtx3(&self, v: u16) -> Vec3 {
        let (x, y) = self.pos(v);
        Vec3 { x, y, z: 0.0 }
    }

    /// `mpLib_80053DA4_Floor`: v1 of the last floor in the `next_id0` chain.
    pub fn mp_lib_80053da4_floor(&self, mut line_id: i32) -> Vec3 {
        self.lineid_check(4428, line_id);
        loop {
            let next = self.map_line(line_id).next_id0 as i32;
            if next != -1 && self.flags(next) & COLL_LINE_FLOOR != 0 {
                line_id = next;
            } else {
                break;
            }
        }
        self.lineid_check(4433, line_id);
        self.vtx3(self.map_line(line_id).v1_idx)
    }

    /// `mpLib_80053ECC_Floor`: v0 of the first floor in the `prev_id0` chain.
    pub fn mp_lib_80053ecc_floor(&self, mut line_id: i32) -> Vec3 {
        self.lineid_check(4448, line_id);
        loop {
            let prev = self.map_line(line_id).prev_id0 as i32;
            if prev != -1 && self.flags(prev) & COLL_LINE_FLOOR != 0 {
                line_id = prev;
            } else {
                break;
            }
        }
        self.lineid_check(4453, line_id);
        self.vtx3(self.map_line(line_id).v0_idx)
    }

    /// mpFloorGetRight & co: walk while the linked line has the same kind; return the far
    /// vertex of the last line walked (`v1` going Next, `v0` going Prev). Unbounded, like the C.
    fn same_kind_end(&self, src_line: u32, line_id: i32, dir: Dir) -> Vec3 {
        self.lineid_check(src_line, line_id);
        let kind = self.flags(line_id) & LINE_FLAG_KIND;
        let mut id = line_id;
        let last = loop {
            let line = *self.map_line(id);
            let cur = id;
            id = self.check_link(&line, Self::link1(&line, dir), dir, true);
            if id == -1 || kind != self.flags(id) & LINE_FLAG_KIND {
                break cur;
            }
        };
        let l = self.map_line(last);
        self.vtx3(if dir == Dir::Next { l.v1_idx } else { l.v0_idx })
    }

    pub fn mp_floor_get_right(&self, id: i32) -> Vec3 {
        self.same_kind_end(4465, id, Dir::Next)
    }
    pub fn mp_floor_get_left(&self, id: i32) -> Vec3 {
        self.same_kind_end(4474, id, Dir::Prev)
    }
    pub fn mp_ceiling_get_right(&self, id: i32) -> Vec3 {
        self.same_kind_end(4483, id, Dir::Prev)
    }
    pub fn mp_ceiling_get_left(&self, id: i32) -> Vec3 {
        self.same_kind_end(4492, id, Dir::Next)
    }
    pub fn mp_left_wall_get_top(&self, id: i32) -> Vec3 {
        self.same_kind_end(4501, id, Dir::Next)
    }
    pub fn mp_left_wall_get_bottom(&self, id: i32) -> Vec3 {
        self.same_kind_end(4510, id, Dir::Prev)
    }
    pub fn mp_right_wall_get_top(&self, id: i32) -> Vec3 {
        self.same_kind_end(4519, id, Dir::Prev)
    }
    pub fn mp_right_wall_get_bottom(&self, id: i32) -> Vec3 {
        self.same_kind_end(4528, id, Dir::Next)
    }

    /// `mpLineGetV1Pos`
    pub fn mp_line_get_v1_pos(&self, id: i32) -> Vec3 {
        self.lineid_check(4540, id);
        self.vtx3(self.map_line(id).v1_idx)
    }

    /// `mpLineGetV0Pos`
    pub fn mp_line_get_v0_pos(&self, id: i32) -> Vec3 {
        self.lineid_check(4555, id);
        self.vtx3(self.map_line(id).v0_idx)
    }

    /// `mpLineGetKind`
    pub fn mp_line_get_kind(&self, id: i32) -> u32 {
        self.lineid_check(4573, id);
        self.flags(id) & LINE_FLAG_KIND
    }

    /// `mpLineGetFlags` (the MapLine's `lo_flags`).
    pub fn mp_line_get_flags(&self, id: i32) -> u32 {
        self.lineid_check(4583, id);
        self.map_line(id).lo_flags as u32
    }

    /// `mpLib_80054D68`: replace the low byte of `lo_flags` (u16 truncation as in C).
    pub fn mp_lib_80054d68(&mut self, id: i32, flags: u32) {
        self.lineid_check(4595, id);
        let idx = self.lines[id as usize].x0 as usize;
        let l = &mut self.map_lines[idx];
        l.lo_flags = ((l.lo_flags as u32 & !0xFF) | flags) as u16;
    }

    /// `mpLineGetNormal`
    pub fn mp_line_get_normal(&self, id: i32) -> Vec3 {
        self.lineid_check(4609, id);
        let line = self.map_line(id);
        let (x0, y0) = self.pos(line.v0_idx);
        let (x1, y1) = self.pos(line.v1_idx);
        ps_vec_normalize(Vec3 { x: -(y1 - y0), y: x1 - x0, z: 0.0 })
    }

    /// `mpLib_80054ED8`: line exists, is enabled and not hidden.
    pub fn mp_lib_80054ed8(&self, id: i32) -> bool {
        if !self.active(id) {
            return false;
        }
        if id == -1 {
            return false;
        }
        if id < 0 || id >= self.line_count {
            // OSReport + infinite loop on the console.
            panic!("mplib.c:4636:not found lineID={id}");
        }
        let f = self.flags(id);
        f & LINE_FLAG_ENABLED != 0 && f & LINE_FLAG_HIDDEN == 0
    }

    /// `mpLinesConnected`: is `target` reachable from `start` along same-kind lines?
    pub fn mp_lines_connected(&self, start_id: i32, target_id: i32) -> bool {
        self.lineid_check(4656, start_id);
        self.lineid_check(4657, target_id);
        if start_id == target_id {
            return true;
        }
        let start = *self.map_line(start_id);
        let kind = self.flags(start_id) & LINE_FLAG_KIND;
        let mut id = self.check_link(&start, start.next_id1 as i32, Dir::Next, true);
        while id != -1 && kind == self.flags(id) & LINE_FLAG_KIND {
            if id == target_id {
                return true;
            }
            id = self.mp_line_get_next(id);
        }
        id = self.check_link(&start, start.prev_id1 as i32, Dir::Prev, true);
        while id != -1 && kind == self.flags(id) & LINE_FLAG_KIND {
            if id == target_id {
                return true;
            }
            id = self.mp_line_get_prev(id);
        }
        false
    }
}
