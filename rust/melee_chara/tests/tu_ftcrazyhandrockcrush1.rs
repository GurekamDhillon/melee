//! Smoke test of tu::ftcrazyhandrockcrush1 (float countdown against an int attribute).
#![allow(static_mut_refs)]

use melee_chara::stubs as st;
use melee_chara::tu::ftcrazyhandrockcrush1 as t;
use melee_types::*;

#[repr(align(16))]
struct Heap([u8; 1 << 20]);
static mut HEAP: Heap = Heap([0; 1 << 20]);
static mut HEAP_TOP: usize = 0;

/// ftBossLib_8015C190 (the last call of Phys) asks mplib for floor line 0's ends, which are now the real
/// ports: load a one-line stage (floor -50..50) so they have something to read.
fn load_stage() -> (std::vec::Vec<Vec2>, std::vec::Vec<MapLine>, std::vec::Vec<MapJoint>, Box<MapCollData>) {
    use melee_world::tu::{mpisland, mplib};
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
        mplib::ext::hook::ScriptGame_StageJointCapacity = Some(|| 16);
        mplib::ext::hook::ScriptGame_StageSlotJointUpdate = Some(|_| 0);
        mplib::ext::hook::Ground_801C0498 = Some(|| 1.0);
        mplib::ext::hook::grDynamicAttr_801CA0B4 = Some(|| {});
    }
    let mut verts = std::vec![Vec2 { x: -50.0, y: 0.0 }, Vec2 { x: 50.0, y: 0.0 }];
    let mut l = MapLine::zeroed();
    l.v1_idx = 1;
    l.prev_id0 = -1;
    l.next_id0 = -1;
    l.prev_id1 = -1;
    l.next_id1 = -1;
    l.hi_flags = CollLine_Floor as u16;
    let mut lines = std::vec![l];
    let mut j = MapJoint::zeroed();
    j.ranges[MapLineGroup_Floor as usize] = MapLineRange { start: 0, count: 1 };
    j.left_bound = -100.0;
    j.bottom_bound = -50.0;
    j.right_bound = 100.0;
    j.top_bound = 50.0;
    j.vtx_count = 2;
    let mut joints = std::vec![j];
    let mut mcd = Box::new(MapCollData::zeroed());
    mcd.verts = verts.as_mut_ptr();
    mcd.vert_count = 2;
    mcd.lines = lines.as_mut_ptr();
    mcd.line_count = 1;
    mcd.ranges = joints[0].ranges;
    mcd.joints = joints.as_mut_ptr();
    mcd.joint_count = 1;
    unsafe { melee_world::tu::mplib::mpLibLoad(&mut *mcd) };
    (verts, lines, joints, mcd)
}

fn phys(gp: *mut HSD_GObj) {
    t::ftCh_RockCrushDown_Phys(gp);
}

#[test]
fn smoke() {
    let mut f: Box<Fighter> = Box::new(Fighter::zeroed());
    let mut g: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    let mut fd: Box<ftData> = Box::new(unsafe { core::mem::zeroed() });
    let mut da = Box::new(ftCrazyHand_DatAttrs::zeroed());
    da.x48 = 50;
    da.x44 = 1.5;
    fd.ext_attr = &mut *da as *mut _ as *mut _;
    f.ft_data = &mut *fd;
    g.user_data = &mut *f as *mut Fighter as *mut _;
    let gp: *mut HSD_GObj = &mut *g;
    let _stage = load_stage();
    unsafe {
        st::FT_80085134 = Some(|_| {});
        melee_fighter::hooks::PLINK_HEAD = Some(|_| core::ptr::null_mut());
    }
    // countdown above x48: velocity is cleared and the boss helpers are not asked for a target
    f.self_vel.x = 9.0;
    f.mv.ch.unk0.x0 = 100.0;
    phys(gp);
    assert_eq!(unsafe { f.mv.ch.unk0.x0 }, 99.0);
    assert_eq!(f.self_vel.x, 0.0);
    // negative countdown also clears
    f.self_vel.x = 9.0;
    f.mv.ch.unk0.x0 = 0.0;
    phys(gp);
    assert_eq!(unsafe { f.mv.ch.unk0.x0 }, -1.0);
    assert_eq!(f.self_vel.x, 0.0);
}
