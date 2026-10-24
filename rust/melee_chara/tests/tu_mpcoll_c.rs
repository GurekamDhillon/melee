//! melee_world::tu::mpcoll::mpColl_8004730C (with mpCollPrev, mpColl_80042C58 and the whole floor / wall /
//! ceiling pass under it, plus the mpLibLoad stage load) vs the C: gen/turef/tu_mpcoll_ref.c #includes
//! mplib.c, mpisland.c and mpcoll.c verbatim (shipping contraction), loads a synthetic stage and prints the
//! result and every CollData field per case (data in tests/data/tu_mpcoll.txt; regenerate with
//! `python rust/melee_chara/gen/turef/tu_pre.py tu_mpcoll`). The Rust side builds the same stage from the same
//! xorshift stream and feeds the same CollData / ftCollisionBox.
#![allow(static_mut_refs)]

use melee_types::*;
use melee_world::tu::{mpcoll, mpisland, mplib};
use std::fmt::Write;

#[repr(align(16))]
struct Heap([u8; 1 << 20]);
static mut HEAP: Heap = Heap([0; 1 << 20]);
static mut HEAP_TOP: usize = 0;

struct Rng(u32);
impl Rng {
    fn rnd(&mut self) -> u32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 17;
        self.0 ^= self.0 << 5;
        self.0
    }
}
fn qf(n: i32) -> f32 {
    n as f32 * 0.25
}
fn b(f: f32) -> String {
    format!("{:08x}", f.to_bits())
}

/// What the joint callbacks were handed (the C driver's `CBREC`): tag, user data, joint, x50, kind, delta-y bits.
static mut CBS: std::vec::Vec<(i32, u32, i32, i32, i32, u32)> = std::vec::Vec::new();
unsafe extern "C" fn cb_a(ud: *mut core::ffi::c_void, joint: i32, _c: *mut CollData, x50: i32, kind: u32, dy: f32) {
    CBS.push((1, ud as usize as u32, joint, x50, kind as i32, dy.to_bits()));
}
unsafe extern "C" fn cb_b(ud: *mut core::ffi::c_void, joint: i32, _c: *mut CollData, x50: i32, kind: u32, dy: f32) {
    CBS.push((2, ud as usize as u32, joint, x50, kind as i32, dy.to_bits()));
}

struct World {
    verts: std::vec::Vec<Vec2>,
    lines: std::vec::Vec<MapLine>,
    joints: std::vec::Vec<MapJoint>,
    mcd: MapCollData,
}

fn build_world(r: &mut Rng) -> Box<World> {
    let mut w = Box::new(World {
        verts: std::vec::Vec::with_capacity(32),
        lines: std::vec::Vec::with_capacity(32),
        joints: std::vec![MapJoint::zeroed()],
        mcd: MapCollData::zeroed(),
    });
    fn line(w: &mut World, v0: i32, v1: i32, prev: i32, next: i32, hi: i32, lo: u32) {
        let mut l = MapLine::zeroed();
        l.v0_idx = v0 as u16;
        l.v1_idx = v1 as u16;
        l.prev_id0 = prev as i16;
        l.next_id0 = next as i16;
        l.prev_id1 = -1;
        l.next_id1 = -1;
        l.hi_flags = hi as u16;
        l.lo_flags = lo as u16;
        w.lines.push(l);
    }
    let mut y = 0.0f32;
    for i in 0..=8 {
        if i == 0 || (r.rnd() % 3) != 0 {
            y = qf((r.rnd() % 33) as i32 - 16);
        }
        w.verts.push(Vec2 { x: qf(-320 + 80 * i), y });
    }
    for i in 0..8 {
        let lo = r.rnd() % 16;
        line(&mut w, i, i + 1, if i == 0 { -1 } else { i - 1 }, if i == 7 { -1 } else { i + 1 }, CollLine_Floor as i32 | if i == 0 || i == 7 { LINE_FLAG_LEDGE } else { 0 }, lo);
    }
    for (x, y) in [(-40.0, 18.0), (-10.0, 18.0), (10.0, 30.0), (40.0, 30.0)] {
        w.verts.push(Vec2 { x, y });
    }
    line(&mut w, 9, 10, -1, -1, CollLine_Floor as i32 | LINE_FLAG_PLATFORM, 1);
    line(&mut w, 11, 12, -1, -1, CollLine_Floor as i32 | LINE_FLAG_PLATFORM, 2);
    for (x, y) in [(80.0, 70.0), (0.0, 60.0), (-80.0, 70.0)] {
        w.verts.push(Vec2 { x, y });
    }
    line(&mut w, 13, 14, -1, 11, CollLine_Ceiling as i32, 3);
    line(&mut w, 14, 15, 10, -1, CollLine_Ceiling as i32, 3);
    for (x, y) in [(80.0, 40.0), (-80.0, 40.0)] {
        w.verts.push(Vec2 { x, y });
    }
    line(&mut w, 8, 16, -1, -1, CollLine_LeftWall as i32, 4);
    line(&mut w, 17, 0, -1, -1, CollLine_RightWall as i32, 5);

    let rg = |s: i16, c: i16| MapLineRange { start: s, count: c };
    let ranges = [rg(0, 10), rg(10, 2), rg(13, 1), rg(12, 1), rg(14, 0)];
    // MapLineGroup order: Floor, Ceiling, RightWall, LeftWall, Dynamic
    w.joints[0].ranges = ranges;
    w.joints[0].left_bound = -100.0;
    w.joints[0].bottom_bound = -50.0;
    w.joints[0].right_bound = 100.0;
    w.joints[0].top_bound = 80.0;
    w.joints[0].vtx_start = 0;
    w.joints[0].vtx_count = w.verts.len() as i16;
    w.mcd.verts = w.verts.as_mut_ptr();
    w.mcd.vert_count = w.verts.len() as i32;
    w.mcd.lines = w.lines.as_mut_ptr();
    w.mcd.line_count = w.lines.len() as i32;
    w.mcd.ranges = ranges;
    w.mcd.joints = w.joints.as_mut_ptr();
    w.mcd.joint_count = 1;
    w
}

fn pecb(s: &mut String, e: &ftECB) {
    write!(s, " {} {} {} {} {} {} {} {}", b(e.top.x), b(e.top.y), b(e.bottom.x), b(e.bottom.y), b(e.right.x), b(e.right.y), b(e.left.x), b(e.left.y)).unwrap();
}
fn psurf(s: &mut String, d: &SurfaceData) {
    write!(s, " {} {} {} {} {}", d.index, d.flags, b(d.normal.x), b(d.normal.y), b(d.normal.z)).unwrap();
}
fn pvec(s: &mut String, v: &Vec) {
    write!(s, " {} {} {}", b(v.x), b(v.y), b(v.z)).unwrap();
}
fn dump(i: usize, r: i32, c: &CollData) -> String {
    let mut s = format!("{} {}", i, r);
    pvec(&mut s, &c.cur_pos);
    pvec(&mut s, &c.prev_pos);
    pvec(&mut s, &c.last_pos);
    pvec(&mut s, &c.x28_vec);
    let (a, f) = (&c.x34_flags, &c.x35_flags);
    write!(s, " f {} {} {} {} {} {} {} {} {} {}", a.b0(), a.b1234(), a.b5(), a.b6(), a.b7(), f.b0(), f.b1234(), f.b5(), f.b6(), f.b7()).unwrap();
    write!(s, " {} {} {} {} {} {} {}", c.facing_dir, c.x38, c.floor_skip, c.ledge_id_right, c.ledge_id_left, c.joint_id_skip, c.joint_id_only).unwrap();
    write!(s, " {} {} {} {} {}", b(c.x50), b(c.ledge_snap_x), b(c.ledge_snap_y), b(c.ledge_snap_height), b(c.lstick_x)).unwrap();
    pecb(&mut s, &c.x64_ecb);
    pecb(&mut s, &c.desired_ecb);
    pecb(&mut s, &c.ecb);
    pecb(&mut s, &c.prev_ecb);
    pecb(&mut s, &c.xE4_ecb);
    write!(s, " {} {} {} {}", c.x130_flags, c.env_flags, c.prev_env_flags, c.x13C).unwrap();
    pvec(&mut s, &c.contact);
    psurf(&mut s, &c.floor);
    psurf(&mut s, &c.left_facing_wall);
    psurf(&mut s, &c.right_facing_wall);
    psurf(&mut s, &c.ceiling);
    unsafe {
        write!(s, " cb {}", CBS.len()).unwrap();
        for k in CBS.iter() {
            write!(s, " {} {} {} {} {} {:08x}", k.0, k.1, k.2, k.3, k.4, k.5).unwrap();
        }
    }
    s
}

fn rbits(r: &mut Rng, f: &mut ECBFlagStruct) {
    let v = r.rnd() & 1;
    f.set_b0(v as u8);
    let v = r.rnd() % 16;
    f.set_b1234(v as u8);
    let v = r.rnd() & 1;
    f.set_b5(v as u8);
    let v = r.rnd() & 1;
    f.set_b6(v as u8);
    let v = r.rnd() & 1;
    f.set_b7(v as u8);
}
fn recb(r: &mut Rng, e: &mut ftECB) {
    let w = qf(12 + (r.rnd() % 12) as i32);
    e.top.x = 0.0;
    e.top.y = qf(30 + (r.rnd() % 30) as i32);
    e.bottom.x = 0.0;
    e.bottom.y = qf((r.rnd() % 9) as i32 - 4);
    e.right.x = w;
    e.right.y = qf(24 + (r.rnd() % 20) as i32);
    e.left.x = -qf(12 + (r.rnd() % 12) as i32);
    e.left.y = e.right.y;
}

#[test]
fn matches_c() {
    let want = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/tu_mpcoll.txt")).unwrap();
    unsafe {
        mplib::ext::hook::HSD_MemAlloc = Some(|n| {
            let p = HEAP.0.as_mut_ptr().add(HEAP_TOP);
            HEAP_TOP += (n as usize + 15) & !15;
            p as *mut _
        });
        mpisland::ext::hook::HSD_MemAlloc = mplib::ext::hook::HSD_MemAlloc;
        mpisland::ext::hook::HSD_Free = Some(|_| {});
        mplib::ext::hook::ScriptGame_StageJointActive = Some(|_| 1);
        mplib::ext::hook::ScriptGame_StageLineActive = Some(|_| 1);
        mplib::ext::hook::ScriptGame_StageJoint = Some(|_| 0);
        mplib::ext::hook::ScriptGame_StageJointCapacity = Some(|| 256);
        mplib::ext::hook::ScriptGame_StageSlotJointUpdate = Some(|_| 0);
        mplib::ext::hook::Ground_801C0498 = Some(|| 1.0);
        mplib::ext::hook::grDynamicAttr_801CA0B4 = Some(|| {});
        mplib::ext::hook::grDynamicAttr_801CA224 = Some(|| {});
        mpcoll::ext::hook::grDynamicAttr_801CA284 = Some(|_, i| if i % 3 == 1 { 0x35 + i } else { 0 });
        mpcoll::ext::hook::ScriptGame_StageSlotSameJoint = Some(|a, b| (a == b) as i32);
    }
    let mut r = Rng(0x8004730C);
    let mut w = build_world(&mut r);
    unsafe {
        mplib::mpLibLoad(&mut w.mcd);
        mplib::mpJointSetCb1(0, 0x11 as *mut _, Some(cb_a));
        mplib::mpJointSetCb2(0, 0x22 as *mut _, Some(cb_b));
    }

    let mut c: Box<CollData> = Box::new(CollData::zeroed());
    let mut bx = ftCollisionBox::zeroed();
    let mut n = 0;
    for (i, line) in want.lines().enumerate() {
        let mode = r.rnd() % 10;
        let (dx, dy): (i32, i32);
        let nv = w.verts.len() as u32;
        let vt = |v: usize| w.verts[v];
        if mode == 0 {
            c.cur_pos.x = qf((r.rnd() % 761) as i32 - 380);
            c.cur_pos.y = qf((r.rnd() % 421) as i32 - 120);
            dx = (r.rnd() % 65) as i32 - 32;
            dy = (r.rnd() % 65) as i32 - 32;
        } else if mode <= 3 {
            let v = if r.rnd() & 1 != 0 { (r.rnd() % 9) as usize } else { 9 + (r.rnd() % 4) as usize };
            c.cur_pos.x = vt(v).x + qf((r.rnd() % 9) as i32 - 4);
            c.cur_pos.y = vt(v).y + qf((r.rnd() % 9) as i32 - 4);
            dx = (r.rnd() % 9) as i32 - 4;
            dy = -((r.rnd() % 25) as i32);
        } else if mode <= 5 {
            let k = r.rnd() % 4;
            let v = [8usize, 0, 16, 17][k as usize];
            c.cur_pos.x = vt(v).x + qf((r.rnd() % 49) as i32 - 24);
            c.cur_pos.y = vt(v).y + qf((r.rnd() % 81) as i32 - 8);
            dx = (r.rnd() % 49) as i32 - 24;
            dy = (r.rnd() % 9) as i32 - 4;
        } else if mode <= 7 {
            let v = 13 + (r.rnd() % 3) as usize;
            c.cur_pos.x = vt(v).x + qf((r.rnd() % 49) as i32 - 24);
            c.cur_pos.y = vt(v).y - qf((r.rnd() % 49) as i32);
            dx = (r.rnd() % 9) as i32 - 4;
            dy = (r.rnd() % 25) as i32;
        } else {
            let v = (r.rnd() % nv) as usize;
            c.cur_pos.x = vt(v).x + qf((r.rnd() % 49) as i32 - 24);
            c.cur_pos.y = vt(v).y + qf((r.rnd() % 49) as i32 - 24);
            if r.rnd() & 1 != 0 {
                dx = (r.rnd() % 65) as i32 - 32;
                dy = (r.rnd() % 65) as i32 - 32;
            } else {
                dx = (r.rnd() % 13) as i32 - 6;
                dy = (r.rnd() % 13) as i32 - 6;
            }
        }
        c.cur_pos.z = 0.0;
        c.last_pos.x = c.cur_pos.x - qf(dx);
        c.last_pos.y = c.cur_pos.y - qf(dy);
        c.last_pos.z = 0.0;
        c.prev_pos.x = c.last_pos.x - qf((r.rnd() % 25) as i32 - 12);
        c.prev_pos.y = c.last_pos.y - qf((r.rnd() % 25) as i32 - 12);
        c.prev_pos.z = 0.0;
        c.x28_vec.x = qf((r.rnd() % 41) as i32 - 20);
        c.x28_vec.y = qf((r.rnd() % 41) as i32 - 20);
        c.x28_vec.z = 0.0;
        let (mut f34, mut f35) = (c.x34_flags, c.x35_flags);
        rbits(&mut r, &mut f34);
        rbits(&mut r, &mut f35);
        c.x34_flags = f34;
        c.x35_flags = f35;
        c.facing_dir = if r.rnd() & 1 != 0 { 1 } else { -1 };
        c.x38 = (r.rnd() % 5) as i32;
        c.floor_skip = if r.rnd() & 1 != 0 { -1 } else { (r.rnd() % 14) as i32 };
        c.ledge_id_right = if r.rnd() & 1 != 0 { -1 } else { (r.rnd() % 14) as i32 };
        c.ledge_id_left = if r.rnd() & 1 != 0 { -1 } else { (r.rnd() % 14) as i32 };
        c.joint_id_skip = if r.rnd() % 4 == 0 { 0 } else { -1 };
        c.joint_id_only = if r.rnd() % 4 == 0 { 0 } else { -1 };
        c.x50 = qf((r.rnd() % 17) as i32 - 8);
        c.ledge_snap_x = qf((r.rnd() % 17) as i32 - 8);
        c.ledge_snap_y = qf((r.rnd() % 17) as i32 - 8);
        c.ledge_snap_height = qf((r.rnd() % 17) as i32 - 8);
        c.lstick_x = qf((r.rnd() % 9) as i32 - 4);
        recb(&mut r, &mut c.x64_ecb);
        recb(&mut r, &mut c.desired_ecb);
        recb(&mut r, &mut c.ecb);
        recb(&mut r, &mut c.prev_ecb);
        recb(&mut r, &mut c.xE4_ecb);
        if r.rnd() & 1 != 0 {
            c.prev_ecb = c.ecb;
        }
        let a = if r.rnd() % 4 == 0 { CollData_X130_Clear as u32 } else { 0 };
        let bb = if r.rnd() % 4 == 0 { CollData_X130_Locked as u32 } else { 0 };
        c.x130_flags = a | bb | (r.rnd() & 0xF);
        c.env_flags = (r.rnd() & 0xFFFF) as i32;
        c.prev_env_flags = (r.rnd() & 0xFFFF) as i32;
        c.x13C = (r.rnd() % 3) as i32;
        c.contact.x = qf((r.rnd() % 17) as i32 - 8);
        c.contact.y = qf((r.rnd() % 17) as i32 - 8);
        c.contact.z = 0.0;
        c.floor.index = (r.rnd() % 14) as i32 - 1;
        c.floor.flags = r.rnd() & 0xFF;
        c.floor.normal.x = qf((r.rnd() % 5) as i32 - 2);
        c.floor.normal.y = qf((r.rnd() % 5) as i32 - 2);
        c.floor.normal.z = 0.0;
        c.left_facing_wall = c.floor;
        c.right_facing_wall = c.floor;
        c.ceiling = c.floor;
        c.left_facing_wall.index = (r.rnd() % 14) as i32 - 1;
        c.right_facing_wall.index = (r.rnd() % 14) as i32 - 1;
        c.ceiling.index = (r.rnd() % 14) as i32 - 1;
        bx.top = qf(30 + (r.rnd() % 30) as i32);
        bx.bottom = if r.rnd() & 1 != 0 { 0.0 } else { qf((r.rnd() % 9) as i32 - 4) };
        bx.left.x = -qf(12 + (r.rnd() % 12) as i32);
        bx.left.y = qf(24 + (r.rnd() % 20) as i32);
        bx.right.x = qf(12 + (r.rnd() % 12) as i32);
        bx.right.y = qf(24 + (r.rnd() % 20) as i32);
        unsafe { CBS.clear() };
        let res = unsafe { mpcoll::mpColl_8004730C(&mut *c, &mut bx) };
        let got = dump(i, res, &c);
        assert_eq!(got, line, "case {i}");
        n += 1;
    }
    assert_eq!(n, 6000);
}
