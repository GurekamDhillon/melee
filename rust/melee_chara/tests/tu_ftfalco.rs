//! Smoke test of tu::ftfalco (no float math, so no C reference): OnLoad pushes Fox's attribute
//! block and reads two item kinds out of it as `s32[7]` / `s32[8]`; the rest delegates to ftFox.
#![allow(static_mut_refs)]

use core::ffi::c_void;
use melee_chara::stubs as st;
use melee_chara::tu::ftfalco as t;
use melee_types::*;

static mut LOG: std::vec::Vec<String> = std::vec::Vec::new();
fn log(s: String) {
    unsafe { LOG.push(s) }
}
fn take() -> String {
    unsafe { std::mem::take(&mut LOG).join(",") }
}

#[test]
fn smoke() {
    let mut f: Box<Fighter> = Box::new(Fighter::zeroed());
    let mut g: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    let fd: &'static mut ftData = Box::leak(Box::new(unsafe { core::mem::zeroed() }));
    let ext_attr: &'static mut ftFox_DatAttrs = Box::leak(Box::new(ftFox_DatAttrs::zeroed()));
    let backup: &'static mut ftFox_DatAttrs = Box::leak(Box::new(ftFox_DatAttrs::zeroed()));
    ext_attr.x1C_FOX_BLASTER_SHOT_ITKIND = 21;
    ext_attr.x20_FOX_BLASTER_GUN_ITKIND = 22;
    let mut items: [*mut c_void; 4] =
        [1000 as *mut c_void, 1001 as *mut c_void, 1002 as *mut c_void, 1003 as *mut c_void];
    fd.x48_items = items.as_mut_ptr();
    fd.ext_attr = ext_attr as *mut _ as *mut c_void;
    f.ft_data = fd;
    f.dat_attrs_backup = backup as *mut _ as *mut c_void;
    g.user_data = &mut *f as *mut Fighter as *mut _;
    let gp: *mut HSD_GObj = &mut *g;
    unsafe {
        t::ext::IT_8026B3F8 = Some(|a, k| log(format!("it {} {k}", a as usize)));
        st::ITISHEAVY = Some(|_| 1);
    }

    t::ftFc_Init_OnLoad(gp);
    assert_eq!(f.can_walljump(), 1);
    assert_eq!(f.dat_attrs, backup as *mut _ as *mut c_void);
    assert_eq!(take(), format!("it 1000 21,it 1001 22,it 1003 {}", It_Kind_Falco_Phantasm));

    // LoadSpecialAttrs is Fox's copy.
    backup.x1C_FOX_BLASTER_SHOT_ITKIND = 0;
    t::ftFc_Init_LoadSpecialAttrs(gp);
    assert_eq!(backup.x1C_FOX_BLASTER_SHOT_ITKIND, 21);

    t::ftFc_Init_OnItemPickup(gp, 1);
    t::ftFc_Init_OnItemDrop(gp, 0);
    assert_eq!(take(), "");
    assert_eq!(t::ftFc_Init_CostumeStrings[0][1], "PlyFalco5K_Share_joint");
    assert_eq!(t::ftFc_Init_DemoMotionFilenames[3], "ftDemoViWaitMotionFileFalco");
}
