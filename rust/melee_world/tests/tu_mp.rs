//! Written by gen/mtu/mk_mp.py from the same table as gen/mtu/tu/mp.c.
//! tu::{mplib, mpcoll, mpisland} vs the C (the three files as one TU, shipping contraction; regenerate
//! the expected file with `python rust/melee_world/gen/mtu/mk_mp.py run`).
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables, unused_unsafe)]
mod mtu_harness;
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;
use melee_world::tu as m;
use melee_world::tu::mpcoll as pc;
use melee_world::tu::mpisland as pi_;
use melee_world::tu::mplib as pl;
use mtu_harness::*;
#[path = "mtu/mp_gen.rs"]
mod g;

const NLN: usize = 24;
const NVX: usize = 64;
const NJ: usize = 3;
static mut M: MapCollData = unsafe { core::mem::zeroed() };
static mut VERTS: [Vec2; NVX] = unsafe { core::mem::zeroed() };
static mut LINES: [MapLine; NLN] = unsafe { core::mem::zeroed() };
static mut JOINTS: [MapJoint; NJ] = unsafe { core::mem::zeroed() };
static mut VTXA: [CollVtx; 2048] = unsafe { core::mem::zeroed() };
static mut LINEA: [CollLine; 1536] = unsafe { core::mem::zeroed() };
static mut JOINTA: [CollJoint; 8] = unsafe { core::mem::zeroed() };
static mut ISL: [mpIsland; 48] = unsafe { core::mem::zeroed() };
static mut ISLN: i32 = 0;
static mut CD: CollData = unsafe { core::mem::zeroed() };
static mut GO: HSD_GObj = unsafe { core::mem::zeroed() };
static mut JO: [HSD_JObj; 8] = unsafe { core::mem::zeroed() };
static mut OUTI: [i32; 2] = [0; 2];
static mut OUTF: [f32; 2] = [0.0; 2];
static mut OUTU: [u32; 2] = [0; 2];
static mut V3: [Vec3; 3] = unsafe { core::mem::zeroed() };
static mut NLN_: i32 = 0;
static mut NVX_: i32 = 0;

unsafe extern "C" fn jcb_a(ud: *mut c_void, jid: i32, c: *mut CollData, x50: i32, k: u32, dy: f32) {
    t(0xCA); t(pi(ud)); t(jid as u32); t(pi(c)); t(x50 as u32); t(k); t(dy.to_bits()); (*c).x50 += 1.0;
}
unsafe extern "C" fn jcb_b(ud: *mut c_void, jid: i32, c: *mut CollData, x50: i32, k: u32, dy: f32) {
    t(0xCB); t(pi(ud)); t(jid as u32); t(pi(c)); t(x50 as u32); t(k); t(dy.to_bits()); (*c).x50 -= 0.5;
}
unsafe extern "C" fn cb_pipe(c: *mut CollData, f: u32) -> i32 {
    t(0xCC); t(pi(c)); t(f); (*c).cur_pos.x += 0.5; (fv() & 1) as i32
}
unsafe extern "C" fn cb_floor(g: *mut HSD_GObj, i: isize) -> i32 {
    t(0xCD); t(pi(g)); t(i as u32); (fv() & 1) as i32
}
unsafe fn jp(i: usize) -> *mut HSD_JObj {
    if rnd() % 5 == 0 { null_mut() } else { &raw mut JO[i] }
}
fn ri() -> f32 {
    if rnd() % 8 == 0 { return rg(); }
    ((rnd() % 21) as i32 - 10) as f32
}
fn rc() -> f32 {
    ((rnd() % 61) as i32 - 30) as f32 + if rnd() & 1 != 0 { 0.5 } else { 0.0 }
}
unsafe fn rvid() -> i32 {
    (rnd() % NLN_ as u32) as i32
}
unsafe fn rlid() -> i32 {
    if rnd() % 4 == 0 { -1 } else { (rnd() % NLN_ as u32) as i32 }
}
fn rnn() -> f32 {
    const K: [f32; 4] = [1.0, 13.0, 100.0, 1000.0];
    let a = ((rnd() % 41) as i32 - 20) as f32 / 20.0;
    a * K[(rnd() % 4) as usize]
}
unsafe fn mkecb(e: *mut ftECB, w: f32, h: f32) {
    (*e).top.x = 0.0; (*e).top.y = h; (*e).bottom.x = 0.0; (*e).bottom.y = 0.0;
    (*e).right.x = w; (*e).right.y = h * 0.5; (*e).left.x = -w; (*e).left.y = h * 0.5;
}

unsafe fn build_world() {
    let mut n = [0i32; 5];
    let mut st = [0i32; 5];
    let mut tot = 0i32;
    let mut slot = 0i32;
    let mut joint_of = [0i32; NLN];
    let mut grp_of = [0usize; NLN];
    let mut pos_of = [0i32; NLN];
    let mut p0x = [0f32; NLN];
    let mut p0y = [0f32; NLN];
    let mut p1x = [0f32; NLN];
    let mut p1y = [0f32; NLN];
    n[0] = 2 + (rnd() % 4) as i32; n[1] = 1 + (rnd() % 3) as i32; n[2] = 1 + (rnd() % 2) as i32; n[3] = 1 + (rnd() % 2) as i32; n[4] = (rnd() % 3) as i32;
    for g in 0..5 { st[g] = tot; tot += n[g]; }
    for g in 0..5 {
        let mut cx = ((rnd() % 61) as i32 - 30) as f32;
        let mut cy = ((rnd() % 41) as i32 - 20) as f32;
        for k in 0..n[g] {
            let len = 4 + (rnd() % 12) as i32;
            let d = (rnd() % 9) as i32 - 4;
            let i = (st[g] + k) as usize;
            grp_of[i] = g; pos_of[i] = k;
            p0x[i] = cx; p0y[i] = cy;
            if g == 0 { cx += len as f32; cy += d as f32; }
            else if g == 1 { cx -= len as f32; cy += d as f32; }
            else if g == 2 { cy -= len as f32; cx += d as f32; }
            else if g == 3 { cy += len as f32; cx += d as f32; }
            else { let mut dx = (rnd() % 21) as i32 - 10; let dy = (rnd() % 21) as i32 - 10; if dx == 0 && dy == 0 { dx = 1; } cx += dx as f32; cy += dy as f32; }
            p1x[i] = cx; p1y[i] = cy;
        }
    }
    for g in 0..5 { for k in 0..n[g] { joint_of[(st[g] + k) as usize] = (k * 3) / n[g]; } }
    for j in 0..NJ {
        let first = slot;
        let mut cnt = 0i32;
        let (mut lo_x, mut lo_y, mut hi_x, mut hi_y) = (1e9f32, 1e9f32, -1e9f32, -1e9f32);
        for g in 0..5 {
            let mut s = 0i32;
            let mut c = 0i32;
            for k in 0..n[g] { if joint_of[(st[g] + k) as usize] == j as i32 { if c == 0 { s = st[g] + k; } c += 1; } }
            JOINTS[j].ranges[g].start = (if c != 0 { s } else { st[g] }) as i16; JOINTS[j].ranges[g].count = c as i16;
        }
        for i in 0..tot as usize {
            if joint_of[i] == j as i32 {
                LINES[i].v0_idx = slot as u16; LINES[i].v1_idx = (slot + 1) as u16;
                let sl = slot as usize;
                VERTS[sl].x = p0x[i]; VERTS[sl].y = p0y[i]; VERTS[sl + 1].x = p1x[i]; VERTS[sl + 1].y = p1y[i];
                if p0x[i] < lo_x { lo_x = p0x[i]; } if p1x[i] < lo_x { lo_x = p1x[i]; } if p0x[i] > hi_x { hi_x = p0x[i]; } if p1x[i] > hi_x { hi_x = p1x[i]; }
                if p0y[i] < lo_y { lo_y = p0y[i]; } if p1y[i] < lo_y { lo_y = p1y[i]; } if p0y[i] > hi_y { hi_y = p0y[i]; } if p1y[i] > hi_y { hi_y = p1y[i]; }
                slot += 2; cnt += 2;
            }
        }
        JOINTS[j].left_bound = lo_x - 2.0; JOINTS[j].bottom_bound = lo_y - 2.0; JOINTS[j].right_bound = hi_x + 2.0; JOINTS[j].top_bound = hi_y + 2.0;
        JOINTS[j].vtx_start = first as i16; JOINTS[j].vtx_count = cnt as i16;
    }
    for i in 0..tot as usize {
        LINES[i].prev_id0 = (if pos_of[i] > 0 { i as i32 - 1 } else { -1 }) as i16;
        LINES[i].next_id0 = (if pos_of[i] < n[grp_of[i]] - 1 { i as i32 + 1 } else { -1 }) as i16;
        let r = rnd() as i32; if r % 8 == 0 { LINES[i].prev_id0 = (if i > 0 { (rnd() % i as u32) as i32 } else { -1 }) as i16; }
        let r = rnd() as i32; if r % 8 == 0 { LINES[i].next_id0 = (if tot - 1 - i as i32 > 0 { i as i32 + 1 + (rnd() % (tot - 1 - i as i32) as u32) as i32 } else { -1 }) as i16; }
        LINES[i].prev_id1 = (if rnd() % 3 == 0 { if i > 0 { (rnd() % i as u32) as i32 } else { -1 } } else { -1 }) as i16;
        LINES[i].next_id1 = (if rnd() % 3 == 0 { if tot - 1 - i as i32 > 0 { i as i32 + 1 + (rnd() % (tot - 1 - i as i32) as u32) as i32 } else { -1 } } else { -1 }) as i16;
        let mut hf = if grp_of[i] < 4 { 1u32 << grp_of[i] } else { 1u32 << (rnd() % 4) };
        if rnd() & 1 != 0 { hf |= 0x100; }
        if rnd() & 1 != 0 { hf |= 0x200; }
        if rnd() % 4 == 0 { hf |= 0x400; }
        hf |= fv() & 0xF870;
        LINES[i].hi_flags = hf as u16;
        LINES[i].lo_flags = (fv() & 0xFFFF) as u16;
        if rnd() % 3 == 0 { LINES[i].lo_flags |= 0x400; }
    }
    M.verts = VERTS.as_mut_ptr(); M.vert_count = slot; M.lines = LINES.as_mut_ptr(); M.line_count = tot; M.joints = JOINTS.as_mut_ptr(); M.joint_count = NJ as i32;
    for g in 0..5 { M.ranges[g].start = st[g] as i16; M.ranges[g].count = n[g] as i16; }
    NLN_ = tot; NVX_ = slot;
}

unsafe fn setup() {
    M = core::mem::zeroed(); VERTS = core::mem::zeroed(); LINES = core::mem::zeroed(); JOINTS = core::mem::zeroed();
    VTXA = core::mem::zeroed(); LINEA = core::mem::zeroed(); JOINTA = core::mem::zeroed(); ISL = core::mem::zeroed(); ISLN = 0;
    CD = core::mem::zeroed(); GO = core::mem::zeroed(); JO = core::mem::zeroed(); OUTI = [0; 2]; OUTF = [0.0; 2]; OUTU = [0; 2]; V3 = core::mem::zeroed();
    pl::ext::stage_info.grkind = 0;
    pc::ext::DbLevel = 0;
    pc::mpColl_80458810 = core::mem::zeroed(); pc::mpColl_804D6488 = 0; pc::mpColl_804D648C = 0; pc::mpColl_804D6490_max_x = 0.0; pc::mpColl_804D6494_line_id = 0; pc::mpColl_804D6498_flags = 0;
    pc::mpColl_IsEcbTiny = 0; pc::mpColl_804D64A0 = None; pc::mpColl_804D64A4 = null_mut(); pc::mpColl_804D64A8 = None; pc::mpColl_804D64AC = 0; pc::mpColl_LedgeLevelOK = 0;
    pl::mpLib_804D64CC = 0; pl::mpLib_804D64D0 = 0; pl::mpLib_804D64D4 = 0; pl::mpLib_804D64D8 = 0; pl::mpLib_804D64DC = 0; pl::mpLib_804D64E0 = 0; pl::mpLib_804D64E4 = 0;
    pl::mpLib_80458868 = core::mem::zeroed(); pl::scriptBatchDepth = 0; pl::mp_script_isolation = core::mem::zeroed();
    build_world();
    pl::mpLibLoad(&raw mut M);
    for j in 0..NJ {
        let cj = &raw mut JOINTA[j];
        if rnd() % 5 == 0 { (*cj).flags &= !(CollJoint_Enabled as u32); }
        if rnd() % 6 == 0 { (*cj).flags |= CollJoint_Hidden as u32; }
        (*cj).flags |= fv() & 0x1F00;
        (*cj).set_xE((rnd() & 1) as u8);
        (*cj).jobj = if rnd() & 1 != 0 { &raw mut JO[j] } else { null_mut() };
        (*cj).cb_0 = if rnd() % 3 != 0 { if rnd() & 1 != 0 { Some(jcb_a) } else { Some(jcb_b) } } else { None };
        (*cj).cb_data_0 = if rnd() & 1 != 0 { &raw mut GO as *mut _ } else { null_mut() };
        (*cj).cb_1 = if rnd() % 3 != 0 { if rnd() & 1 != 0 { Some(jcb_a) } else { Some(jcb_b) } } else { None };
        (*cj).cb_data_1 = if rnd() & 1 != 0 { &raw mut JO[1] as *mut _ } else { null_mut() };
    }
    g::fill_HSD_GObj(&mut GO);
    for i in 0..8 { g::fill_HSD_JObj(&mut JO[i]); }
    g::fill_CollData(&mut CD);
    CD.x0_gobj = if rnd() & 3 != 0 { &raw mut GO } else { null_mut() };
    CD.cur_pos.x = rc(); CD.cur_pos.y = rc(); CD.cur_pos.z = 0.0;
    if rnd() % 4 != 0 {
        let li = (rnd() % NLN_ as u32) as usize;
        let vi = rnd() & 1;
        let ox = ((rnd() % 9) as i32 - 4) as f32 * 0.5;
        let oy = ((rnd() % 9) as i32 - 4) as f32 * 0.5;
        let vp = &VERTS[(if vi != 0 { LINES[li].v1_idx } else { LINES[li].v0_idx }) as usize];
        CD.cur_pos.x = vp.x + ox; CD.cur_pos.y = vp.y + oy;
    }
    CD.prev_pos.x = CD.cur_pos.x + ((rnd() % 7) as i32 - 3) as f32 * 0.5; CD.prev_pos.y = CD.cur_pos.y + ((rnd() % 7) as i32 - 3) as f32 * 0.5; CD.prev_pos.z = 0.0;
    CD.last_pos.x = CD.prev_pos.x + ((rnd() % 5) as i32 - 2) as f32 * 0.5; CD.last_pos.y = CD.prev_pos.y + ((rnd() % 5) as i32 - 2) as f32 * 0.5; CD.last_pos.z = 0.0;
    CD.x28_vec = CD.prev_pos;
    let w = (3 + (rnd() % 3)) as f32;
    let h = (8 + (rnd() % 8)) as f32;
    mkecb(&raw mut CD.ecb, w, h);
    mkecb(&raw mut CD.prev_ecb, w + (rnd() % 2) as f32, h);
    mkecb(&raw mut CD.desired_ecb, w, h + (rnd() % 3) as f32);
    mkecb(&raw mut CD.x64_ecb, w, h);
    mkecb(&raw mut CD.xE4_ecb, w, h);
    CD.floor_skip = if rnd() % 4 == 0 { (rnd() % NLN_ as u32) as i32 } else { -1 };
    CD.ledge_id_right = if rnd() % 3 == 0 { (rnd() % NLN_ as u32) as i32 } else { -1 };
    CD.ledge_id_left = if rnd() % 3 == 0 { (rnd() % NLN_ as u32) as i32 } else { -1 };
    CD.joint_id_skip = if rnd() % 6 == 0 { (rnd() % NJ as u32) as i32 } else { -1 };
    CD.joint_id_only = if rnd() % 6 == 0 { (rnd() % NJ as u32) as i32 } else { -1 };
    CD.x50 = rg(); CD.lstick_x = rg();
    CD.ledge_snap_x = (2 + (rnd() % 4)) as f32; CD.ledge_snap_y = (rnd() % 8) as f32; CD.ledge_snap_height = (4 + (rnd() % 8)) as f32;
    CD.facing_dir = if rnd() & 1 != 0 { 1 } else { -1 };
    CD.env_flags = rnd() as i32; CD.prev_env_flags = rnd() as i32; CD.x130_flags = rnd() & 0xFF; CD.x13C = rnd() as i32;
    if rnd() & 1 != 0 {
        CD.ecb_source.kind = ECBSource_Fixed as _;
        CD.ecb_source.x4.x0_1.up = (8 + (rnd() % 8)) as f32; CD.ecb_source.x4.x0_1.down = (rnd() % 3) as f32; CD.ecb_source.x4.x0_1.front = (3 + (rnd() % 3)) as f32; CD.ecb_source.x4.x0_1.back = (3 + (rnd() % 3)) as f32;
        CD.ecb_source.x4.x0_1.angle = ((rnd() % 13) as i32 - 6) as f32 * 0.5;
    } else {
        CD.ecb_source.kind = ECBSource_JObj as _;
        CD.ecb_source.x4.x0.x108_joint = &raw mut JO[0];
        for i in 0..6 { CD.ecb_source.x4.x0.x10C_joint[i] = &raw mut JO[i + 1]; }
    }
    CD.ecb_source.x124 = ((rnd() % 41) as i32 - 20) as f32 / 20.0; CD.ecb_source.x128 = (rnd() % 12) as f32; CD.ecb_source.x12C = (rnd() % 12) as f32;
    CD.floor.index = rlid(); CD.floor.flags = fv(); CD.ceiling.index = rlid(); CD.ceiling.flags = fv();
    CD.left_facing_wall.index = rlid(); CD.left_facing_wall.flags = fv(); CD.right_facing_wall.index = rlid(); CD.right_facing_wall.flags = fv();
    CD.contact.x = rg(); CD.contact.y = rg(); CD.contact.z = 0.0;
    if rnd() % 2 == 0 {
        let li = (rnd() % NLN_ as u32) as usize;
        if LINES[li].hi_flags & 1 != 0 { CD.floor.index = li as i32; } else if LINES[li].hi_flags & 2 != 0 { CD.ceiling.index = li as i32; } else if LINES[li].hi_flags & 4 != 0 { CD.right_facing_wall.index = li as i32; } else { CD.left_facing_wall.index = li as i32; }
    }
}

unsafe fn hash_all() {
    g::hash_CollData(&CD);
    hm(CD.ecb_source.kind as u32);
    if CD.ecb_source.kind == ECBSource_Fixed as _ {
        hm(CD.ecb_source.x4.x0_1.up.to_bits()); hm(CD.ecb_source.x4.x0_1.down.to_bits()); hm(CD.ecb_source.x4.x0_1.front.to_bits());
        hm(CD.ecb_source.x4.x0_1.back.to_bits()); hm(CD.ecb_source.x4.x0_1.angle.to_bits());
    } else {
        hm(pi(CD.ecb_source.x4.x0.x108_joint)); for i in 0..6 { hm(pi(CD.ecb_source.x4.x0.x10C_joint[i])); }
    }
    hm(CD.ecb_source.x124.to_bits()); hm(CD.ecb_source.x128.to_bits()); hm(CD.ecb_source.x12C.to_bits());
    g::hash_MapCollData(&M);
    for i in 0..NVX { hm(VERTS[i].x.to_bits()); hm(VERTS[i].y.to_bits()); }
    for i in 0..NLN { g::hash_MapLine(&LINES[i]); }
    for i in 0..NJ { g::hash_MapJoint(&JOINTS[i]); }
    for i in 0..NVX { g::hash_CollVtx(&VTXA[i]); }
    for i in 0..NLN { g::hash_CollLine(&LINEA[i]); }
    for i in 0..8 { g::hash_CollJoint(&JOINTA[i]); }
    for i in 0..48 { g::hash_mp_UnkStruct0(&ISL[i]); }
    hm(ISLN as u32);
    g::hash_HSD_GObj(&GO);
    hm(pi(pl::groundCollVtx)); hm(pi(pl::groundCollLine)); hm(pi(pl::groundCollJoint)); hm(pi(pl::jointListStart)); hm(pi(pl::jointListEnd)); hm(pi(pl::mpLib_804D64B4));
    hm(pl::didCheckBounding as u32); hm(pl::scriptBatchDepth as u32);
    hm(pi(pl::mp_script_isolation.map)); hm(pi(pl::mp_script_isolation.lines)); hm(pi(pl::mp_script_isolation.joints)); hm(pl::mp_script_isolation.nlines as u32); hm(pl::mp_script_isolation.njoints as u32);
    for i in 0..NLN { hm(pl::mp_script_isolation.line_flags[i]); }
    for i in 0..8 { hm(pl::mp_script_isolation.joint_flags[i]); }
    for i in 0..3 { for kk in 0..5 { hm((pl::mp_script_isolation.ranges[i][kk].start as u16 as u32) | ((pl::mp_script_isolation.ranges[i][kk].count as u16 as u32) << 16)); } }
    hm(pl::mpLib_804D64CC as u32); hm(pl::mpLib_804D64D0 as u32); hm(pl::mpLib_804D64D4 as u32); hm(pl::mpLib_804D64D8 as u32); hm(pl::mpLib_804D64DC as u32); hm(pl::mpLib_804D64E0 as u32); hm(pl::mpLib_804D64E4 as u32);
    for i in 0..2 { hm(pl::mpLib_80458868[i].top.to_bits()); hm(pl::mpLib_80458868[i].bottom.to_bits()); hm(pl::mpLib_80458868[i].left.to_bits()); hm(pl::mpLib_80458868[i].right.to_bits()); }
    for i in 0..9 { hm(pc::mpColl_80458810.right[i] as u32); hm(pc::mpColl_80458810.left[i] as u32); }
    hm(pc::mpColl_80458810.normal.x.to_bits()); hm(pc::mpColl_80458810.normal.y.to_bits()); hm(pc::mpColl_80458810.normal.z.to_bits());
    hm(pc::mpColl_804D6488 as u32); hm(pc::mpColl_804D648C as u32); hm(pc::mpColl_804D6490_max_x.to_bits()); hm(pc::mpColl_804D6494_line_id as u32); hm(pc::mpColl_804D6498_flags as u32); hm(pc::mpColl_IsEcbTiny as u32);
    hm(fi(pc::mpColl_804D64A0)); hm(pi(pc::mpColl_804D64A4)); hm(fi(pc::mpColl_804D64A8)); hm(pc::mpColl_804D64AC as u32); hm(pc::mpColl_LedgeLevelOK as u32);
    hm(pi(pi_::mpIsland_80458E88.floors)); hm(pi(pi_::mpIsland_80458E88.ceilings)); hm(pi(pi_::mpIsland_80458E88.non_dynamic_floors_tail)); hm(pi(pi_::mpIsland_80458E88.non_dynamic_ceilings_tail)); hm(pi(pi_::mpIsland_80458E88.dynamic_floors));
    hm(pi(pi_::mpIsland_80458E88.dynamic_ceilings)); hm(pi(pi_::mpIsland_80458E88.b1_floors)); hm(pi(pi_::mpIsland_80458E88.b1_ceilings)); hm(pi(pi_::mpIsland_80458E88.free_list));
    hm(pl::ext::stage_info.grkind as u32);
    for i in 0..3 { hm(V3[i].x.to_bits()); hm(V3[i].y.to_bits()); hm(V3[i].z.to_bits()); }
    hm(OUTU[0]); hm(OUTU[1]); hm(OUTI[0] as u32); hm(OUTI[1] as u32); hm(OUTF[0].to_bits()); hm(OUTF[1].to_bits());
}

#[test]
fn mp_matches_c() {
    unsafe {
        std::panic::set_hook(Box::new(|i| { let m = i.to_string(); if !(m.contains(": assertion failed") || m.contains("lineID")) { eprintln!("{}", m); } }));
        g::install();
        pl::ext::hook::OSReport = Some(|_| {});
        pc::ext::hook::OSReport = Some(|_| {});
        preg(&raw const M, core::mem::size_of::<MapCollData>());
        preg(&raw const VERTS, core::mem::size_of_val(&VERTS));
        preg(&raw const LINES, core::mem::size_of_val(&LINES));
        preg(&raw const JOINTS, core::mem::size_of_val(&JOINTS));
        preg(&raw const VTXA, core::mem::size_of_val(&VTXA));
        preg(&raw const LINEA, core::mem::size_of_val(&LINEA));
        preg(&raw const JOINTA, core::mem::size_of_val(&JOINTA));
        preg(&raw const ISL, core::mem::size_of_val(&ISL));
        preg(&raw const CD, core::mem::size_of::<CollData>());
        preg(&raw const GO, core::mem::size_of::<HSD_GObj>());
        preg(&raw const JO, core::mem::size_of_val(&JO));
        preg(&raw const OUTI, core::mem::size_of_val(&OUTI));
        preg(&raw const V3, core::mem::size_of_val(&V3));
        freg(jcb_a as *const () as usize); freg(jcb_b as *const () as usize); freg(cb_pipe as *const () as usize); freg(cb_floor as *const () as usize);
        g::freg_all();
        let mut c = Check::new(include_str!("data/mtu_mp.txt"));
        for op in 0..95u32 {
            for k in 0..600u32 {
                case(op, k);
                setup();
                let mut ret = 0u32;
                let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                match op {
                    0 => { pc::mpCollPrev(&raw mut CD); }
                    1 => { pc::mpColl_80042384(&raw mut CD); }
                    2 => { ret = pc::mpColl_80044E10_RightWall(&raw mut CD) as u32; }
                    3 => { ret = pc::mpColl_80045B74_LeftWall(&raw mut CD) as u32; }
                    4 => { ret = pc::mpColl_800478F4(&raw mut CD) as u32; }
                    5 => { ret = pc::mpColl_800488F4(&raw mut CD) as u32; }
                    6 => { ret = pc::mpColl_80048AB0_RightWall(&raw mut CD) as u32; }
                    7 => { ret = pc::mpColl_80049778_LeftWall(&raw mut CD) as u32; }
                    8 => { ret = pc::mpColl_8004B4B0(&raw mut CD) as u32; }
                    9 => { { if CD.ecb_source.kind != ECBSource_Fixed as _ { CD.ecb_source.kind = ECBSource_Fixed as _; CD.ecb_source.x4.x0_1.up = 12.0; CD.ecb_source.x4.x0_1.down = 1.0; CD.ecb_source.x4.x0_1.front = 4.0; CD.ecb_source.x4.x0_1.back = 3.0; CD.ecb_source.x4.x0_1.angle = 0.5; } pc::mpColl_LoadECB_Fixed(&raw mut CD); } }
                    10 => { pc::mpCollCheckBounding(&raw mut CD, fv()); }
                    11 => { { let a = (fv() & 1) as i32; let b = (fv() & 1) as i32; pc::mpCollEnd(&raw mut CD, a, b); } }
                    12 => { pc::mpCollInterpolateECB(&raw mut CD, (rnd() % 11) as f32 / 10.0); }
                    13 => { { let f = fv(); ret = pc::mpColl_80043754(Some(cb_pipe), &raw mut CD, f) as u32; } }
                    14 => { { let lr = (fv() & 1) as i32; ret = pc::mpColl_80044628_Floor(&raw mut CD, if rnd() & 1 != 0 { Some(cb_floor) } else { None }, &raw mut GO, lr) as u32; } }
                    15 => { { let lr = (fv() & 1) as i32; ret = pc::mpColl_80044AD8_Ceiling(&raw mut CD, lr) as u32; } }
                    16 => { { let f = fv(); ret = pc::mpColl_80046904(&raw mut CD, f) as u32; } }
                    17 => { pc::mpColl_LoadECB_JObj(&raw mut CD, fv()); }
                    18 => { pc::mpColl_SetECBSource_JObj(&raw mut CD, &raw mut GO, jp(0), jp(1), jp(2), jp(3), jp(4), jp(5), jp(6), rg()); }
                    19 => { { let a = rw(); let b = rw(); let c = rw(); pc::mpColl_SetLedgeSnap(&raw mut CD, a, b, c); } }
                    20 => { ret = pc::mpColl_80044164(&raw mut CD, &raw mut OUTI[0]) as u32; hm(OUTI[0] as u32); }
                    21 => { ret = pc::mpColl_800443C4(&raw mut CD, &raw mut OUTI[0]) as u32; hm(OUTI[0] as u32); }
                    22 => { ret = pc::mpColl_80043BBC(&raw mut CD, &raw mut OUTI[0]) as u32; hm(OUTI[0] as u32); }
                    23 => { ret = pc::mpColl_80043E90(&raw mut CD, &raw mut OUTI[0]) as u32; hm(OUTI[0] as u32); }
                    24 => { { let b = (fv() & 1) as i32; ret = pc::mpColl_80044838_Floor(&raw mut CD, b) as u32; } }
                    25 => { { let id = rlid(); ret = pc::mpColl_8004A908_Floor(&raw mut CD, id) as u32; } }
                    26 => { ret = pc::mpColl_8004ACE4(&raw mut CD, fv() as i32) as u32; }
                    27 => { { let f = fv(); pc::mpCollPrev(&raw mut CD); for _ in 0..4 { ret = ret.wrapping_mul(3).wrapping_add(pc::mpColl_80043754(Some(cb_pipe), &raw mut CD, f) as u32); CD.cur_pos.x += ((rnd() % 9) as i32 - 4) as f32; CD.cur_pos.y += ((rnd() % 9) as i32 - 4) as f32; pc::mpCollPrev(&raw mut CD); } let a = (fv() & 1) as i32; let b = (fv() & 1) as i32; pc::mpCollEnd(&raw mut CD, a, b); } }
                    28 => { { let l = rw(); let b = rw(); let r = rw(); let t = rw(); pl::mpBoundingCheck(l, b, r, t); } }
                    29 => { { let l = rc(); let b = rc(); let r = rc(); let t = rc(); pl::mpBoundingCheck(l, b, r, t); } }
                    30 => { ret = pl::mpCheckedBounding() as u32; }
                    31 => { { pl::didCheckBounding = (rnd() & 1) as i32; pl::mpUncheckBounding(); } }
                    32 => { ret = pi(pl::mpGetGroundCollVtx()); }
                    33 => { { let i = (rnd() % 7) as usize; if (i as i32) < NLN_ { LINES[i].v1_idx = LINES[i].v0_idx; } pl::mpPruneEmptyLines(&raw mut M); } }
                    34 => { { let j = (rnd() % 3) as i32; pl::mpJointUpdateDynamics(j); } }
                    35 => { ret = pi(pl::mpLib_8004D164()); }
                    36 => { pl::mpLib_80058614_Floor(); }
                    37 => { pl::mpLib_800587FC(&raw mut GO); }
                    38 => { { let l = (rnd() % (NLN_ + 4) as u32) as i32 - 1; let j = (rnd() % 6) as i32 - 1; let iso = (rnd() % 4 != 0) as i32; let bad = (rnd() % 6) as i32; let bj = (rnd() % NJ as u32) as usize; let bk = (rnd() % 5) as usize; let bc = (rnd() % 5) as i32 - 1; if bad == 0 { JOINTS[bj].ranges[bk].count = bc as i16; } ret = (pl::mpScriptStageIsolationSet(l, j, iso) as u32).wrapping_mul(3).wrapping_add(pl::mpScriptStageIsolationActive() as u32); } }
                    39 => { { let j = NJ as i32; let a = pl::mpScriptStageIsolationSet(NLN_, j, 1); if rnd() & 1 != 0 { let i = (rnd() % NLN_ as u32) as usize; let fx = fv() & 0xFF; LINEA[i].flags ^= fx; } if rnd() % 4 == 0 { let i = (rnd() % NJ as u32) as usize; let fx = fv() & 0xFF00; JOINTA[i].flags ^= fx; } if rnd() % 5 == 0 { pl::mpLib_804D64B4 = if rnd() & 1 != 0 { null_mut() } else { &raw mut GO as *mut MapCollData }; } let b = pl::mpScriptStageIsolationActive(); let c = pl::mpScriptStageIsolationSet(0, 0, 0); ret = (a * 4 + b * 2 + c) as u32; } }
                    40 => { { let a = (rnd() % 20) as i32; pl::ext::stage_info.grkind = (rnd() % 80) as _; ret = pl::mpLib_80056A54(a, &raw mut OUTI[0]) as u32; hm(OUTI[0] as u32); } }
                    41 => { { let a = (rnd() % 20) as i32; pl::ext::stage_info.grkind = (rnd() % 80) as _; ret = pl::mpLib_80056AC4(a, &raw mut OUTI[0]) as u32; hm(OUTI[0] as u32); } }
                    42 => { { let a = (rnd() % 20) as i32; pl::ext::stage_info.grkind = (rnd() % 80) as _; let p = pl::mpLib_80056A8C(a, &raw mut OUTI[0]); hm(OUTI[0] as u32); hm(*p as u32); hm(*p.add(1) as u32); hm(*p.add(2) as u32); hm(*p.add(3) as u32); } }
                    43 => { { let (a, b, c, d, e, f, g, h) = (ri(), ri(), ri(), ri(), ri(), ri(), ri(), ri()); ret = pl::mpLineIntersection(a, b, c, d, e, f, g, h, &raw mut OUTF[0], &raw mut OUTF[1]) as u32; hm(OUTF[0].to_bits()); hm(OUTF[1].to_bits()); } }
                    44 => { { let (a, b, c, d, e, f, g) = (ri(), ri(), ri(), ri(), ri(), ri(), ri()); ret = pl::mpLineIntersectionH(&raw mut OUTF[0], &raw mut OUTF[1], a, b, c, d, e, f, g) as u32; hm(OUTF[0].to_bits()); hm(OUTF[1].to_bits()); } }
                    45 => { { let (a, b, c, d, e, f, g) = (ri(), ri(), ri(), ri(), ri(), ri(), ri()); ret = pl::mpLineIntersectionV(&raw mut OUTF[0], &raw mut OUTF[1], a, b, c, d, e, f, g) as u32; hm(OUTF[0].to_bits()); hm(OUTF[1].to_bits()); } }
                    46 => { pi_::mpIsland_8005A728(); }
                    47 => { { let s = (rnd() % 30) as i32; let c = (rnd() % 12) as i32; let b = (fv() & 1) as i32; pi_::mpIsland_8005AE1C(&raw mut pi_::mpIsland_80458E88.floors, &raw mut pi_::mpIsland_80458E88.b1_floors, s, c, b); } }
                    48 => { { let j = (rnd() % 3) as i32; let k = (1 + (rnd() & 1)) as i32; let s = (rnd() % 30) as i32; let c = (rnd() % 12) as i32; let b = (fv() & 1) as i32; pi_::mpIsland_8005B004(if k == 1 { &raw mut pi_::mpIsland_80458E88.dynamic_floors } else { &raw mut pi_::mpIsland_80458E88.dynamic_ceilings }, &raw mut pi_::mpIsland_80458E88.free_list, j, k, s, c, b); } }
                    49 => { { let j = (rnd() % 3) as i32; let s = (rnd() % 30) as i32; let c = (rnd() % 12) as i32; let b = (fv() & 1) as i32; pi_::mpIsland_8005B334(j, s, c, b); } }
                    50 => { { let id = rvid(); ret = pl::mpLineGetNext(id) as u32; } }
                    51 => { { let id = rvid(); ret = pl::mpLineGetPrev(id) as u32; } }
                    52 => { { let id = rvid(); ret = pl::mpLineNextNonFloor(id) as u32; } }
                    53 => { { let id = rvid(); ret = pl::mpLinePrevNonFloor(id) as u32; } }
                    54 => { { let id = rvid(); ret = pl::mpLinePrevNonCeiling(id) as u32; } }
                    55 => { { let id = rvid(); ret = pl::mpLineNextNonCeiling(id) as u32; } }
                    56 => { { let id = rvid(); ret = pl::mpLineNextNonLeftWall(id) as u32; } }
                    57 => { { let id = rvid(); ret = pl::mpLinePrevNonLeftWall(id) as u32; } }
                    58 => { { let id = rvid(); ret = pl::mpLinePrevNonRightWall(id) as u32; } }
                    59 => { { let id = rvid(); ret = pl::mpLineNextNonRightWall(id) as u32; } }
                    60 => { { let id = rvid(); ret = pl::mpLib_80053394_Floor(id) as u32; } }
                    61 => { { let id = rvid(); ret = pl::mpLib_80053448_Floor(id) as u32; } }
                    62 => { { let id = rvid(); ret = pl::mpLib_800534FC_Floor(id) as u32; } }
                    63 => { { let id = rvid(); ret = pl::mpLib_800536CC_Floor(id) as u32; } }
                    64 => { { let id = rvid(); ret = pl::mpLib_8005389C_Ceiling(id) as u32; } }
                    65 => { { let id = rvid(); ret = pl::mpLib_80053950_Ceiling(id) as u32; } }
                    66 => { { let id = rvid(); ret = pl::mpLib_80053A04_Ceiling(id) as u32; } }
                    67 => { { let id = rvid(); ret = pl::mpLib_80053BD4_Ceiling(id) as u32; } }
                    68 => { { let id = rvid(); ret = pl::mpLineGetKind(id) as u32; } }
                    69 => { { let id = rvid(); ret = pl::mpLineGetFlags(id) as u32; } }
                    70 => { { let id = rvid(); ret = pl::mpLib_80054ED8(id) as u32; } }
                    71 => { { let id = rvid(); ret = pl::mpJointFromLine(id) as u32; } }
                    72 => { { let id = rvid(); pl::mpLib_80053DA4_Floor(id, &raw mut V3[0]); } }
                    73 => { { let id = rvid(); pl::mpLib_80053ECC_Floor(id, &raw mut V3[0]); } }
                    74 => { { let id = rvid(); pl::mpFloorGetRight(id, &raw mut V3[0]); } }
                    75 => { { let id = rvid(); pl::mpFloorGetLeft(id, &raw mut V3[0]); } }
                    76 => { { let id = rvid(); pl::mpCeilingGetRight(id, &raw mut V3[0]); } }
                    77 => { { let id = rvid(); pl::mpCeilingGetLeft(id, &raw mut V3[0]); } }
                    78 => { { let id = rvid(); pl::mpLeftWallGetTop(id, &raw mut V3[0]); } }
                    79 => { { let id = rvid(); pl::mpLeftWallGetBottom(id, &raw mut V3[0]); } }
                    80 => { { let id = rvid(); pl::mpRightWallGetTop(id, &raw mut V3[0]); } }
                    81 => { { let id = rvid(); pl::mpRightWallGetBottom(id, &raw mut V3[0]); } }
                    82 => { { let id = rvid(); pl::mpLineGetV1Pos(id, &raw mut V3[0]); } }
                    83 => { { let id = rvid(); pl::mpLineGetV0Pos(id, &raw mut V3[0]); } }
                    84 => { { let id = rvid(); V3[0].x = rc(); V3[0].y = rc(); V3[0].z = 0.0; ret = pl::mpLib_8004DD90_Floor(id, &raw mut V3[0], &raw mut OUTF[0], &raw mut OUTU[0], &raw mut V3[1]) as u32; } }
                    85 => { { let id = rvid(); V3[0].x = rc(); V3[0].y = rc(); V3[0].z = 0.0; ret = pl::mpLib_8004E090_Ceiling(id, &raw mut V3[0], &raw mut OUTF[0], &raw mut OUTU[0], &raw mut V3[1]) as u32; } }
                    86 => { { let id = rvid(); V3[0].x = rc(); V3[0].y = rc(); V3[0].z = 0.0; ret = pl::mpLib_8004E398_LeftWall(id, &raw mut V3[0], &raw mut OUTF[0], &raw mut OUTU[0], &raw mut V3[1]) as u32; } }
                    87 => { { let id = rvid(); V3[0].x = rc(); V3[0].y = rc(); V3[0].z = 0.0; ret = pl::mpLib_8004E684_RightWall(id, &raw mut V3[0], &raw mut OUTF[0], &raw mut OUTU[0], &raw mut V3[1]) as u32; } }
                    88 => { { let a = rvid(); let b = rvid(); ret = pl::mpLinesConnected(a, b) as u32; } }
                    89 => { { let id = rvid(); V3[0].x = rc(); V3[0].y = rc(); V3[0].z = 0.0; ret = pl::mpGetSpeed(id, &raw mut V3[0], &raw mut V3[1]) as u32; } }
                    90 => { { let id = rvid(); ret = pi(pl::mpLineGetNormal(id, &raw mut V3[0])); } }
                    91 => { { let a = rc(); let b = rc(); let c = rc(); let d = rc(); let s = fv() & 3; let sk = if s == 0 { rlid() } else { -1 }; let js = (rnd() % 4) as i32 - 1; let jo = (rnd() % 4) as i32 - 1; ret = pl::mpCheckFloor(a, b, c, d, 0.0, &raw mut V3[0], &raw mut OUTI[0], &raw mut OUTU[0], &raw mut V3[1], sk, js, jo, None, null_mut()) as u32; } }
                    92 => { { let a = rc(); let b = rc(); let c = rc(); let d = rc(); let js = (rnd() % 4) as i32 - 1; let jo = (rnd() % 4) as i32 - 1; ret = pl::mpCheckCeiling(a, b, c, d, &raw mut V3[0], &raw mut OUTI[0], &raw mut OUTU[0], &raw mut V3[1], js, jo) as u32; } }
                    93 => { { let a = rc(); let b = rc(); let c = rc(); let d = rc(); let js = (rnd() % 4) as i32 - 1; let jo = (rnd() % 4) as i32 - 1; ret = pl::mpCheckLeftWall(a, b, c, d, &raw mut V3[0], &raw mut OUTI[0], &raw mut OUTU[0], &raw mut V3[1], js, jo) as u32; } }
                    94 => { { let a = rc(); let b = rc(); let c = rc(); let d = rc(); let js = (rnd() % 4) as i32 - 1; let jo = (rnd() % 4) as i32 - 1; ret = pl::mpCheckRightWall(a, b, c, d, &raw mut V3[0], &raw mut OUTI[0], &raw mut OUTU[0], &raw mut V3[1], js, jo) as u32; } }
                    _ => unreachable!(),
                }
                }));
                if let Err(e) = r {
                    let msg = e.downcast_ref::<String>().cloned().or_else(|| e.downcast_ref::<&str>().map(|s| s.to_string())).unwrap_or_default();
                    if !(msg.contains(": assertion failed") || msg.contains("lineID")) { std::panic::resume_unwind(e); }
                    hm(0xA55E87);
                }
                hash_all();
                c.line(op, k, ret);
            }
        }
        assert_eq!(c.done(), 95 * 600);
    }
}
