//! Smoke test for src/tu/ftdata.rs: m-ex flag rewrite, costume installer, ref counts, Nana rows.
#![allow(static_mut_refs)]
use melee_fighter::tu::ftdata as d;
use melee_fighter::types::*;
use core::ffi::{c_char, c_void};

#[test]
fn mex_anim_flags_and_rows() {
    assert_eq!(d::ftData_MexAnimFlags(0xABCD_EF21, 40), 0xABCD_EF21); // thrown skeleton kept
    assert_eq!(d::ftData_MexAnimFlags(0x1234_5605, 40), 0x1234_5628);
    assert_eq!(d::ftData_MexAnimFlags(0x1234_5605, 63), 0x1234_563F);
    assert_eq!(d::ftData_MexAnimFlags(0x1234_5605, 64), 0x1234_5600 | FT_ANIM_KIND_SELF as u32);
    let buf = [0u8; 64];
    let lo = buf.as_ptr();
    let hi = unsafe { lo.add(64) };
    let mut row = Fighter_WaitAnimData::zeroed();
    assert!(!d::ftData_MexMotionRowOk(&row, lo, hi)); // no script
    row.xC = unsafe { lo.add(8) } as *mut CmdUnion;
    assert!(d::ftData_MexMotionRowOk(&row, lo, hi)); // NULL name ok
    row.x0 = unsafe { lo.add(70) } as *mut i8;
    assert!(!d::ftData_MexMotionRowOk(&row, lo, hi));
}

#[test]
fn bitfield_and_refcount() {
    let mut t = d::ft_8045993C_t { pad_x0: 0, pad_x4: [0; 2], bf_6: [0; 2] };
    t.set_x6_b0(1);
    t.set_x6_b1_b2(2);
    assert_eq!(t.bf_6, [0xC0, 0]);
    assert_eq!((t.x6_b0(), t.x6_b1_b2()), (1, 2));
    d::ft_8008549C();
    d::ftData_80085560(3, 2);
    d::ftData_80085560(3, -2);
    unsafe { assert_eq!(d::ft_8045996C[3], 0) };
    unsafe { d::ext::OS_REPORT = Some(|_| {}) };
    assert!(std::panic::catch_unwind(|| d::ftData_80085560(3, -1)).is_err());
    d::ft_8008549C();
}

fn alloc(n: usize) -> *mut c_void {
    Box::leak(vec![0u8; n].into_boxed_slice()).as_mut_ptr() as *mut c_void
}
static NAMES: [&[u8]; 3] = [b"f\0", b"j\0", b"m\0"];
fn at(_i: i32, _c: i32, w: i32) -> *const c_char { NAMES[w as usize].as_ptr() as *const c_char }

#[test]
fn install_costumes_keeps_retail_rows() {
    const K: usize = 120;
    let mut strings = [Fighter_CostumeStrings::zeroed(); 2];
    strings[1].joint_name = 7 as *mut i8;
    let mut lists = [UnkCostumeStruct::zeroed(); 2];
    lists[1].pad_x8 = 99;
    unsafe {
        d::ftData_803C2360[K] = strings.as_mut_ptr();
        d::CostumeListsForeachCharacter[K].costume_list = lists.as_mut_ptr();
        d::CostumeListsForeachCharacter[K].numCostumes = 2;
    }
    d::ftData_PcInstallCostumes(K as i32, 5, 2, at, 0, alloc);
    unsafe {
        assert_eq!(d::CostumeListsForeachCharacter[K].numCostumes, 5);
        let s = d::ftData_803C2360[K];
        assert_eq!((*s.add(1)).joint_name as usize, 7);
        assert_eq!((*s.add(4)).matanim_joint_name as usize, NAMES[2].as_ptr() as usize);
        assert_eq!((*d::CostumeListsForeachCharacter[K].costume_list.add(1)).pad_x8, 99);
        assert_eq!((*d::CostumeListsForeachCharacter[K].costume_list.add(3)).pad_x8, 0);
    }
}

#[test]
fn self_vel_and_rows() {
    let mut j = HSD_JObj::zeroed();
    j.translate = Vec3 { x: 1.0, y: 2.0, z: -0.0 };
    let mut f: Box<Fighter> = Box::new(Fighter::zeroed());
    f.cur_pos = Vec3 { x: 0.5, y: 3.0, z: 0.0 };
    let mut g = HSD_GObj::zeroed();
    g.hsd_obj = &mut j as *mut _ as *mut c_void;
    g.user_data = &mut *f as *mut Fighter as *mut c_void;
    d::ft_8008521C(&mut g);
    assert_eq!((f.self_vel.x, f.self_vel.y), (0.5, -1.0));
    assert_eq!(f.self_vel.z.to_bits(), (-0.0f32).to_bits());

    let mut rows = [Fighter_WaitAnimData::zeroed(); 4];
    rows[2].x8 = 0x40;
    f.x24 = rows.as_mut_ptr();
    f.kind = Ft_Kind_Mario;
    assert_eq!(d::ftData_80085FD4(&mut f, 2), unsafe { rows.as_mut_ptr().add(2) });
    assert_eq!(d::ftData_80085FD4_x8(&mut f, 2), 0x40);
    assert!(d::ftData_80086060(&mut f).is_null());
    // msid past x58C: no-op.
    f.x58C = 1;
    assert!(d::ftData_80085E50(&mut *f, 2).is_null());
    assert!(d::ftData_ThrownStandIn(-1).is_null());
}
