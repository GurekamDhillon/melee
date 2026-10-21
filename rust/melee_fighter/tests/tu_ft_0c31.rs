//! Smoke test for src/tu/ft_0c31.rs (match-start entry): stand height / scale over time.
#![allow(static_mut_refs)]

use melee_fighter::hooks;
use melee_fighter::tu::ft_0c31;
use melee_fighter::types::*;

type G = *mut HSD_GObj;
static mut MS: std::vec::Vec<(i32, u32)> = std::vec::Vec::new();

#[test]
fn entry_smoke() {
    let mut cd: Box<ftCommonData> = Box::new(unsafe { core::mem::zeroed() });
    cd.x6BC = 10;
    cd.x6C0 = 7;
    cd.x6C4 = 0.25;
    unsafe {
        hooks::P_FT_COMMON_DATA = &*cd;
        ft_0c31::ext::FIGHTER_CHANGE_MOTION_STATE = Some(|_, ms, fl, _, _, _, _| MS.push((ms, fl)));
        ft_0c31::ext::HSD_JOBJ_SET_MTX_DIRTY_SUB = Some(|_| {});
    }
    let mut root: Box<HSD_JObj> = Box::new(unsafe { core::mem::zeroed() });
    let mut acc: Box<HSD_JObj> = Box::new(unsafe { core::mem::zeroed() });
    let mut fp: Box<Fighter> = Box::new(Fighter::zeroed());
    let mut g: Box<HSD_GObj> = Box::new(HSD_GObj::zeroed());
    g.user_data = &mut *fp as *mut Fighter as *mut core::ffi::c_void;
    g.hsd_obj = &mut *root as *mut HSD_JObj as *mut core::ffi::c_void;
    let gp: G = &mut *g;
    fp.x20A0_accessory = &mut *acc;
    unsafe {
        fp.mv.co.entry.x4 = 5.0;
        fp.mv.co.entry.x20 = 8.0;
        fp.mv.co.entry.x24 = 2.0;
        fp.mv.co.entry.x8 = Vec3 { x: 1.0, y: 1.0, z: 1.0 };
        fp.mv.co.entry.timer = 5;
    }
    ft_0c31::ftCo_EntryEnd_Phys(gp);
    assert_eq!(fp.cur_pos.y, 5.0 + 8.0 * 0.5);
    assert_eq!(acc.scale.y, 1.0);

    ft_0c31::ftCo_EntryStart_Phys(gp);
    let t = 0.5f32;
    assert_eq!(root.scale.y.to_bits(), t.mul_add(1.0 - 0.25, 0.25).to_bits());

    ft_0c31::ftCo_800C6B6C(gp);
    assert_eq!(unsafe { fp.mv.co.entry.timer }, 7);
    assert_eq!(fp.cur_pos.y, 13.0);
    assert_eq!(unsafe { MS.as_slice() }, &[(ftCo_MS_EntryEnd as i32, 0x3000)]);
    assert!(fp.accessory1_cb.is_some());

    ft_0c31::HSD_JObjSetMtxDirty(core::ptr::null_mut());
}
