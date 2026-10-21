//! Floor/ceiling "islands" (connected surface runs), from src/melee/mp/mpisland.c:
//! mpIsland_8005A6F8, mpIsland_8005A728, mpIsland_8005AB54, mpIsland_8005AC14,
//! mpIsland_8005AC8C, mpIsland_8005ACE8, mpIsland_8005AE1C, mpIsland_8005B004,
//! mpIsland_8005B334, and the PC-only mpIsland_StageSlotFree, mpIsland_ScriptRecycle,
//! mpIsland_ScriptBatch.
//!
//! The C allocates each `mp_UnkStruct0` with `HSD_MemAlloc(0x2C)` and links them by pointer.
//! Natively they live in an arena, [`Islands::segs`], in allocation order, and every link is an
//! index into it (-1 = NULL). `HSD_Free` only marks a segment freed (arena slots are never
//! reused, so indices stay equal to the C's allocation order). `x2A` and `ptr` are never written
//! by mpisland.c (heap garbage in the C) and stay 0 here.
//!
//! FMA (`python tools/rust-port/fma_sites.py -v src/melee/mp/mpisland.c`): nothing fuses.

use super::lines::GroundColl;
use super::types::*;
use crate::vec::Vec3;

/// `F32_MAX` (src/Runtime/platform.h).
const F32_MAX: f32 = 3.4028235e38;

/// `mp_UnkStruct0`: one island (a run of connected floor or ceiling lines).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct IslandSeg {
    pub next: i32,
    /// First / last vertex.
    pub x4: u16,
    pub x6: u16,
    pub x8: Vec3,
    pub x14: Vec3,
    /// Bit 1: hidden / disabled.
    pub x20: i32,
    /// First / last line.
    pub x24: i16,
    pub x26: i16,
    /// Owning joint.
    pub x28: i16,
    pub x2a: i16,
    /// `mp_UnkStruct3* ptr` (never set by mpisland.c).
    pub ptr: i32,
    /// Natively only: `HSD_Free` was called on it.
    pub freed: bool,
}
melee_core::gc_layout!(IslandSeg, "mp_UnkStruct0", size 0x30, align 4, [
    next @ 0x0: 4 "mp_UnkStruct0*",
    x4 @ 0x4: 2 "u16",
    x6 @ 0x6: 2 "u16",
    x8 @ 0x8: 12 "Vec3",
    x14 @ 0x14: 12 "Vec3",
    x20 @ 0x20: 4 "int",
    x24 @ 0x24: 2 "s16",
    x26 @ 0x26: 2 "s16",
    x28 @ 0x28: 2 "s16",
    x2a @ 0x2A: 2 "s16",
    ptr @ 0x2C: 4 "mp_UnkStruct3*",
]);

/// `struct mpIsland_80458E88_t`: the island list heads (indices into [`Islands::segs`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IslandLists {
    /// Floor islands (then the script floor list x10 stitched on).
    pub next: i32,
    /// Ceiling islands (then x14).
    pub x4: i32,
    /// Tail of the map's own floor islands in `next`.
    pub x8: i32,
    /// Tail of the map's own ceiling islands in `x4`.
    pub xc: i32,
    /// Dynamic floor / ceiling islands (mpIsland_8005B004).
    pub x10: i32,
    pub x14: i32,
    /// Floor / ceiling islands parked as hidden by mpIsland_8005AE1C.
    pub x18: i32,
    pub x1c: i32,
    /// Free list for mpIsland_8005B004.
    pub x20: i32,
}
impl Default for IslandLists {
    fn default() -> Self {
        Self { next: -1, x4: -1, x8: -1, xc: -1, x10: -1, x14: -1, x18: -1, x1c: -1, x20: -1 }
    }
}
melee_core::gc_layout!(IslandLists, "mpIsland_80458E88_t", size 0x24, align 4, [
    next @ 0x0: 4 "mp_UnkStruct0*",
    x4 @ 0x4: 4 "mp_UnkStruct0*",
    x8 @ 0x8: 4 "mp_UnkStruct0*",
    xc @ 0xC: 4 "mp_UnkStruct0*",
    x10 @ 0x10: 4 "mp_UnkStruct0*",
    x14 @ 0x14: 4 "mp_UnkStruct0*",
    x18 @ 0x18: 4 "mp_UnkStruct0*",
    x1c @ 0x1C: 4 "mp_UnkStruct0*",
    x20 @ 0x20: 4 "mp_UnkStruct0*",
]);

/// mpisland.c's state: `mpIsland_80458E88` plus the segment heap.
#[derive(Clone, Debug, Default)]
pub struct Islands {
    pub heads: IslandLists,
    pub segs: Vec<IslandSeg>,
}

impl Islands {
    /// `HSD_MemAlloc(0x2C)` + `mpIsland_AssertSeg`.
    fn alloc(&mut self) -> i32 {
        self.segs.push(IslandSeg::default());
        (self.segs.len() - 1) as i32
    }
    fn s(&mut self, i: i32) -> &mut IslandSeg {
        &mut self.segs[i as usize]
    }
}

const VISITED: usize = 0x600;

impl GroundColl {
    fn seg_ends(&self, s: &mut IslandSeg) {
        let z = 0.0;
        let p = self.vtx[s.x4 as usize].pos;
        s.x8 = Vec3 { x: p.x, y: p.y, z };
        let p = self.vtx[s.x6 as usize].pos;
        s.x14 = Vec3 { x: p.x, y: p.y, z };
    }

    /// `mpIsland_8005A6F8`
    pub fn mp_island_8005a6f8(&mut self) {
        self.islands.heads = IslandLists::default();
    }

    /// One half of mpIsland_8005A728 (floor: follow `next_id0`, MapLine hi_flags Floor;
    /// ceiling: follow `prev_id0`, Ceiling, and the segment runs v1 -> v0). Returns the tail.
    fn island_build(&mut self, group: usize, ceiling: bool, visited: &mut [u8]) -> i32 {
        let mut prev = -1;
        let mut count = self.map_ranges[group].count as i32;
        if count == 0 {
            return prev;
        }
        let mut line_idx = self.map_ranges[group].start as i32;
        let bit = if ceiling { COLL_LINE_CEILING } else { COLL_LINE_FLOOR } as u16;
        while count != 0 {
            let seg = self.islands.alloc();
            if prev != -1 {
                self.islands.s(prev).next = seg;
            } else if ceiling {
                self.islands.heads.x4 = seg;
            } else {
                self.islands.heads.next = seg;
            }
            prev = seg;
            let mut end_idx = line_idx;
            let mut next = (line_idx == 0) as i32;
            let mut hidden = false;
            while line_idx != next {
                visited[end_idx as usize] = 1;
                let f = self.flags(end_idx);
                if f & LINE_FLAG_ENABLED == 0 || f & LINE_FLAG_HIDDEN != 0 {
                    hidden = true;
                }
                let m = self.map_line(end_idx);
                next = if ceiling { m.prev_id0 } else { m.next_id0 } as i32;
                if next == -1 || self.map_line(next).hi_flags & bit == 0 {
                    break;
                }
                end_idx = next;
            }
            let (a, b) = (*self.map_line(line_idx), *self.map_line(end_idx));
            let mut s = IslandSeg {
                next: -1,
                x20: if hidden { 2 } else { 0 },
                x24: line_idx as i16,
                x26: end_idx as i16,
                x4: if ceiling { a.v1_idx } else { a.v0_idx },
                x6: if ceiling { b.v0_idx } else { b.v1_idx },
                ..self.islands.segs[seg as usize]
            };
            self.seg_ends(&mut s);
            s.x28 = self.mp_joint_from_line(line_idx) as i16;
            *self.islands.s(seg) = s;
            count -= 1;
            line_idx += 1;
            let mut ctr = count;
            while ctr != 0 {
                if visited[line_idx as usize] == 0 {
                    break;
                }
                line_idx += 1;
                count -= 1;
                ctr -= 1;
            }
        }
        prev
    }

    /// `mpIsland_8005A728`: build the floor and ceiling islands of the whole map.
    pub fn mp_island_8005a728(&mut self) {
        self.mp_island_8005a6f8();
        let mut visited = vec![0u8; VISITED];
        let t = self.island_build(MAP_LINE_GROUP_FLOOR, false, &mut visited);
        self.islands.heads.x8 = t;
        let t = self.island_build(MAP_LINE_GROUP_CEILING, true, &mut visited);
        self.islands.heads.xc = t;
    }

    /// `mpIsland_8005AB54`: the floor island holding `surface_idx` (-1 = NULL).
    pub fn mp_island_8005ab54(&self, surface_idx: i32) -> i32 {
        if self.mp_lib_80054ed8(surface_idx) {
            let mut cur = self.islands.heads.next;
            while cur != -1 {
                let s = &self.islands.segs[cur as usize];
                let mut j = s.x24 as i32;
                while j != -1 {
                    if j == surface_idx {
                        return cur;
                    }
                    if j == s.x26 as i32 {
                        break;
                    }
                    let j_next = self.map_line(j).next_id0 as i32;
                    if !(j_next != -1 && self.flags(j_next) & COLL_LINE_FLOOR != 0) {
                        break;
                    }
                    j = j_next;
                }
                cur = s.next;
            }
        }
        -1
    }

    /// `mpIsland_8005AC14`: the floor island under `pos` within `dy` (mpCheckFloor from `pos`
    /// to `pos + (0, dy)`).
    pub fn mp_island_8005ac14(&mut self, pos: Vec3, dy: f32) -> i32 {
        match self.mp_check_floor(pos.x, pos.y, pos.x, pos.y + dy, 0.0, -1, -1, -1, None) {
            Some(hit) => self.mp_island_8005ab54(hit.line_id),
            None => -1,
        }
    }

    /// `mpIsland_8005AC8C`: is the island's joint flagged B8/B9/B10.
    pub fn mp_island_8005ac8c(&self, seg: i32) -> bool {
        let j = self.islands.segs[seg as usize].x28;
        self.joints[j as usize].flags & (COLL_JOINT_B10 | COLL_JOINT_B9 | COLL_JOINT_B8) != 0
    }

    /// `mpIsland_8005ACE8`: the island's end points. `want_left` / `want_right` are the C's
    /// non-NULL out-params; each output is `None` exactly when its pointer is NULL. With both
    /// present, the ends are refined by mpFloorGetLeft/Right over the joint's floor lines.
    pub fn mp_island_8005ace8(&self, seg: i32, want_left: bool, want_right: bool) -> (Option<Vec3>, Option<Vec3>) {
        let s = self.islands.segs[seg as usize];
        let mj = self.map_joints[self.joints[s.x28 as usize].inner as usize];
        let mut line = mj.ranges[MAP_LINE_GROUP_FLOOR].start as i32;
        let count = mj.ranges[MAP_LINE_GROUP_FLOOR].count as i32;
        let mut left = if want_left { Some(s.x8) } else { None };
        let mut right = if want_right { Some(s.x14) } else { None };
        if want_left && want_right {
            for _ in 0..count.max(0) {
                let m = *self.map_line(line);
                if m.v0_idx == s.x4 {
                    left = Some(self.mp_floor_get_left(line));
                } else if m.v1_idx == s.x6 {
                    right = Some(self.mp_floor_get_right(line));
                }
                line += 1;
            }
        }
        (left, right)
    }

    /// `mpIsland_8005AE1C(&a, &b, start, count, enabled)`: refresh the end points of every
    /// island touching vertices [start, start + count) and move them between the live list `a`
    /// and the hidden list `b` (both lists come back reversed). Returns the new `(a, b)`.
    pub fn mp_island_8005ae1c(&mut self, a: i32, b: i32, start: i32, count: i32, enabled: bool) -> (i32, i32) {
        let mut prev_b = -1;
        let mut prev_a = -1;
        let end = start + count;
        for pass in 0..2 {
            let mut cur = if pass == 0 { a } else { b };
            while cur != -1 {
                let mut s = self.islands.segs[cur as usize];
                let next = s.next;
                let (v0, v1) = (s.x4 as i32, s.x6 as i32);
                let touches = !(v0 < start && v1 < start) && (end > v0 || end > v1);
                let mut to_a = pass == 1;
                if touches {
                    self.seg_ends(&mut s);
                    if pass == 0 && !enabled && s.x20 & 2 == 0 {
                        s.x20 |= 2;
                        to_a = true;
                    } else if pass == 1 && enabled && s.x20 & 2 != 0 {
                        s.x20 &= !2;
                        to_a = false;
                    }
                }
                if to_a {
                    s.next = prev_a;
                    prev_a = cur;
                } else {
                    s.next = prev_b;
                    prev_b = cur;
                }
                self.islands.segs[cur as usize] = s;
                cur = next;
            }
        }
        (prev_b, prev_a)
    }

    /// `mpIsland_8005B004(&list, &free, joint, kind, start, count, enabled)`: move `joint`'s
    /// islands from `list` to the free list, then rebuild them from its Dynamic lines whose
    /// `flags & 0x1F == kind | 0x10` (kind 1 floor, 2 ceiling), taking segments from the free
    /// list first. Returns the new `(list, free)`. `start`/`count` are unused, as in the C.
    #[allow(clippy::too_many_arguments)]
    pub fn mp_island_8005b004(
        &mut self, list: i32, free: i32, joint: i32, kind: u32, _start: i32, _count: i32, enabled: bool,
    ) -> (i32, i32) {
        let type_flag = kind | 0x10;
        let (mut a0, mut a1) = (-1, free);
        let mut cur = list;
        while cur != -1 {
            let next = self.islands.segs[cur as usize].next;
            if self.islands.segs[cur as usize].x28 as i32 == joint {
                self.islands.s(cur).next = a1;
                a1 = cur;
            } else {
                self.islands.s(cur).next = a0;
                a0 = cur;
            }
            cur = next;
        }

        let mut visited = vec![0u8; VISITED];
        let mj = self.map_joints[self.joints[joint as usize].inner as usize];
        let dr = mj.ranges[MAP_LINE_GROUP_DYNAMIC];
        let end_total = dr.start as i32 + dr.count as i32;
        let mut line_idx = dr.start as i32;
        let typ = |g: &GroundColl, l: i32| g.flags(l) & 0x1F;
        while line_idx < end_total {
            if visited[line_idx as usize] != 0 || typ(self, line_idx) != type_flag {
                line_idx += 1;
                continue;
            }
            let mut end_idx = line_idx;
            let mut looped = false;
            loop {
                visited[end_idx as usize] = 1;
                let link = self.map_line(end_idx).prev_id0 as i32;
                if link == -1 || typ(self, link) != type_flag {
                    break;
                }
                end_idx = link;
                if link == line_idx {
                    looped = true;
                    break;
                }
            }
            let start_idx;
            if !looped {
                let mut si = line_idx;
                loop {
                    visited[si as usize] = 1;
                    let link = self.map_line(si).next_id0 as i32;
                    if link == -1 || typ(self, link) != type_flag {
                        break;
                    }
                    si = link;
                    if link == line_idx {
                        looped = true;
                        break;
                    }
                }
                if looped {
                    panic!("mpisland.c:518: !loop");
                }
                start_idx = si;
            } else {
                let mut i = end_idx;
                let mut si = end_idx;
                let mut min_x = -F32_MAX;
                let mut max_x = F32_MAX;
                let cycle_start = i;
                loop {
                    let m = *self.map_line(i);
                    let x0 = self.vtx[m.v0_idx as usize].pos.x;
                    if max_x > x0 {
                        max_x = x0;
                        end_idx = i;
                    }
                    let x1 = self.vtx[m.v1_idx as usize].pos.x;
                    if min_x > x1 {
                        min_x = x1;
                        si = i;
                    }
                    let link = m.prev_id0 as i32;
                    i = link;
                    if link == cycle_start {
                        break;
                    }
                }
                start_idx = si;
            }

            let seg = if a1 != -1 {
                let s = a1;
                a1 = self.islands.segs[s as usize].next;
                s
            } else {
                self.islands.alloc()
            };
            let (a, b) = (*self.map_line(end_idx), *self.map_line(start_idx));
            let mut s = IslandSeg {
                x20: if enabled { 0 } else { 2 },
                x24: end_idx as i16,
                x26: start_idx as i16,
                x4: a.v0_idx,
                x6: b.v1_idx,
                x28: joint as i16,
                next: a0,
                ..self.islands.segs[seg as usize]
            };
            self.seg_ends(&mut s);
            *self.islands.s(seg) = s;
            a0 = seg;
        }
        (a0, a1)
    }

    fn list_tail(&self, mut p: i32) -> i32 {
        while p != -1 && self.islands.segs[p as usize].next != -1 {
            p = self.islands.segs[p as usize].next;
        }
        p
    }

    /// Cut the map's own floor/ceiling lists off the script lists stitched after them.
    fn cut_tails(&mut self) {
        let h = self.islands.heads;
        if h.x8 != -1 {
            self.islands.s(h.x8).next = -1;
        } else {
            self.islands.heads.next = -1;
        }
        if h.xc != -1 {
            self.islands.s(h.xc).next = -1;
        } else {
            self.islands.heads.x4 = -1;
        }
    }

    /// `mpIsland_8005B334(joint, start, count, enabled)`: `mpScriptIslandRefresh` on the
    /// console; refresh the islands after a joint's lines changed.
    pub fn mp_island_8005b334(&mut self, joint: i32, start: i32, count: i32, enabled: bool) {
        self.cut_tails();
        let h = self.islands.heads;
        let (a, b) = self.mp_island_8005ae1c(h.next, h.x18, start, count, enabled);
        self.islands.heads.next = a;
        self.islands.heads.x18 = b;
        let h = self.islands.heads;
        let (a, b) = self.mp_island_8005ae1c(h.x4, h.x1c, start, count, enabled);
        self.islands.heads.x4 = a;
        self.islands.heads.x1c = b;
        let h = self.islands.heads;
        let (a, f) = self.mp_island_8005b004(h.x10, h.x20, joint, 1, start, count, enabled);
        self.islands.heads.x10 = a;
        self.islands.heads.x20 = f;
        let h = self.islands.heads;
        let (a, f) = self.mp_island_8005b004(h.x14, h.x20, joint, 2, start, count, enabled);
        self.islands.heads.x14 = a;
        self.islands.heads.x20 = f;

        let t = self.list_tail(self.islands.heads.next);
        if t != -1 {
            self.islands.s(t).next = self.islands.heads.x10;
        } else {
            self.islands.heads.next = self.islands.heads.x10;
        }
        self.islands.heads.x8 = t;
        let t = self.list_tail(self.islands.heads.x4);
        if t != -1 {
            self.islands.s(t).next = self.islands.heads.x14;
        } else {
            self.islands.heads.x4 = self.islands.heads.x14;
        }
        self.islands.heads.xc = t;
    }

    /// Run the queued `mpScriptIslandRefresh` calls ([`GroundColl::island_refresh`]) through
    /// mpIsland_8005B334, in order, as the console build does at each call site. Equal to the C
    /// as long as nothing between the queued call and this drain reads or writes island state
    /// (true of every ported caller: they only set `xE` or joint/line flags afterwards).
    /// TODO(group2: PC script batch): the PC build may route a script joint's refresh to the
    /// batch dirty mark instead (mpIsland_ScriptBatch); that routing is not ported.
    pub fn mp_island_drain_refresh(&mut self) {
        let q = std::mem::take(&mut self.island_refresh);
        for r in q {
            self.mp_island_8005b334(r.joint, r.vtx_start, r.vtx_count, r.enabled);
        }
    }

    /// `mpIsland_StageSlotFree` (PC): free every island of the outgoing map, then reset.
    pub fn mp_island_stage_slot_free(&mut self) {
        let h = self.islands.heads;
        for head in [h.next, h.x4, h.x20, h.x18, h.x1c] {
            let mut p = head;
            while p != -1 {
                let n = self.islands.segs[p as usize].next;
                self.islands.s(p).freed = true;
                p = n;
            }
        }
        self.mp_island_8005a6f8();
    }

    /// `mpIsland_ScriptRecycle` (PC): move every island of a dirty joint from the list onto the
    /// free list `x20`. Returns the new list head.
    fn mp_island_script_recycle(&mut self, list: i32, dirty: &[u8], count: i32) -> i32 {
        let mut head = list;
        let mut prev = -1;
        let mut cur = head;
        while cur != -1 {
            let s = self.islands.segs[cur as usize];
            let j = s.x28 as i32;
            if j >= 0 && j < count && dirty[j as usize] != 0 {
                if prev == -1 {
                    head = s.next;
                } else {
                    self.islands.s(prev).next = s.next;
                }
                self.islands.s(cur).next = self.islands.heads.x20;
                self.islands.heads.x20 = cur;
                cur = if prev == -1 { head } else { self.islands.segs[prev as usize].next };
            } else {
                prev = cur;
                cur = s.next;
            }
        }
        head
    }

    /// `mpIsland_ScriptBatch` (PC): rebuild the Dynamic islands of every dirty joint at once.
    pub fn mp_island_script_batch(&mut self, dirty: &[u8], count: i32) {
        let count = count.min(1024);
        let mut any = 0u8;
        for d in &dirty[..count.max(0) as usize] {
            any |= *d;
        }
        if any == 0 {
            return;
        }
        self.cut_tails();
        let h = self.islands.heads.x10;
        self.islands.heads.x10 = self.mp_island_script_recycle(h, dirty, count);
        let h = self.islands.heads.x14;
        self.islands.heads.x14 = self.mp_island_script_recycle(h, dirty, count);
        for kind in 1..=2u32 {
            let mut head = if kind == 1 { self.islands.heads.x10 } else { self.islands.heads.x14 };
            let mut tail = self.list_tail(head);
            for i in 0..count {
                if dirty[i as usize] == 0 {
                    continue;
                }
                let f = self.joints[i as usize].flags;
                let enabled = f & COLL_JOINT_ENABLED != 0 && f & (COLL_JOINT_HIDDEN | COLL_JOINT_B11) == 0;
                let mj = self.map_joints[self.joints[i as usize].inner as usize];
                let free = self.islands.heads.x20;
                let (fresh, free) = self.mp_island_8005b004(
                    -1, free, i, kind, mj.vtx_start as i32, mj.vtx_count as i32, enabled,
                );
                self.islands.heads.x20 = free;
                if tail == -1 {
                    head = fresh;
                } else {
                    self.islands.s(tail).next = fresh;
                }
                if fresh != -1 {
                    tail = self.list_tail(fresh);
                }
            }
            if kind == 1 {
                self.islands.heads.x10 = head;
            } else {
                self.islands.heads.x14 = head;
            }
        }
        let h = self.islands.heads;
        if h.x8 != -1 {
            self.islands.s(h.x8).next = h.x10;
        } else {
            self.islands.heads.next = h.x10;
        }
        if h.xc != -1 {
            self.islands.s(h.xc).next = h.x14;
        } else {
            self.islands.heads.x4 = h.x14;
        }
    }
}
