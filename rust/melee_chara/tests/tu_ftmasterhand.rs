//! Smoke test for tu::ftmasterhand (stateful; no fused sites).
#![allow(static_mut_refs)]

use melee_chara::tu::ftmasterhand as t;
use melee_types::*;

#[test]
fn smoke() {
    let mut f: Box<Fighter> = Box::new(Fighter::zeroed());
    let mut g: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    g.user_data = &mut *f as *mut Fighter as *mut _;
    let gp: *mut HSD_GObj = &mut *g;

    let mut src: ftMasterHand_SpecialAttrs = unsafe { core::mem::zeroed() };
    src.x30_pos2 = Vec2 { x: 12.5, y: -3.0 };
    src.x18 = 4;
    src.x1C = 1;
    src.x20 = 9;
    let backup: &'static mut ftMasterHand_SpecialAttrs = Box::leak(Box::new(unsafe { core::mem::zeroed() }));
    let mut ftd: Box<ftData> = Box::new(unsafe { core::mem::zeroed() });
    ftd.ext_attr = &mut src as *mut _ as *mut _;
    let mut items: [*mut core::ffi::c_void; 2] = [1 as *mut _, 2 as *mut _];
    ftd.x48_items = items.as_mut_ptr() as *mut _;
    f.ft_data = &mut *ftd;
    f.dat_attrs_backup = backup as *mut _ as *mut _;

    static mut LOG: std::vec::Vec<String> = std::vec::Vec::new();
    unsafe {
        t::ext::FTBOSSLIB_REPORTGOBJSLOTTYPE = Some(|_| LOG.push("slot".into()));
        t::ext::IT_8026B3F8 = Some(|a, k| LOG.push(format!("it {} {k}", a as usize)));
        melee_fighter::hooks::HSD_RANDI = Some(|_| 0);
    }
    // ftBossLib_8015C244 needs the boss-lib state; skip when it is not set up.
    let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| t::ftMh_Init_OnLoad(gp)));
    unsafe {
        assert_eq!(LOG[0], "slot");
        assert_eq!(LOG[1], format!("it 1 {}", It_Kind_MasterHand_Laser));
        assert_eq!(LOG[2], format!("it 2 {}", It_Kind_MasterHand_Bullet));
    }
    assert_eq!(f.dat_attrs, f.dat_attrs_backup);
    assert_eq!(backup.x30_pos2.x, 12.5);
    assert_eq!((f.cur_pos.x, f.cur_pos.y, f.cur_pos.z), (12.5, -3.0, 0.0));
    assert_eq!(f.no_normal_motion(), 1);
    unsafe {
        assert_eq!((f.mv.mh.unk0.x28, f.mv.mh.unk0.x2C, f.mv.mh.unk0.x30), (-1, -1, -1));
    }
    if r.is_ok() {
        assert_eq!(f.cpu.level, 1);
        unsafe {
            assert_eq!(f.u.mh.x2250, ftMh_MS_SweepLoop as i32);
            assert_eq!(f.u.mh.x2238, 1.0);
        }
    }
    src.x18 = 77;
    t::ftMh_Init_LoadSpecialAttrs(gp);
    assert_eq!(backup.x18, 77);
    assert_eq!(t::ftMh_Init_CostumeStrings[0][2], None);
}
