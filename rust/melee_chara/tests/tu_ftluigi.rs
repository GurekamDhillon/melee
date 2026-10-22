//! Smoke test of tu::ftluigi (init callbacks, no float arithmetic). One #[test]: hooks are globals.
#![allow(static_mut_refs, unused_unsafe)]

use melee_chara::stubs as st;
use melee_chara::tu::ftluigi as t;
use melee_types::*;

static mut LOG: std::vec::Vec<String> = std::vec::Vec::new();
fn log(s: String) {
    unsafe { LOG.push(s) }
}
fn take_log() -> String {
    unsafe { std::mem::take(&mut LOG).join(",") }
}

#[test]
fn smoke() {
    unsafe { body() }
}

unsafe fn body() {
    let mut f: Box<Fighter> = Box::new(Fighter::zeroed());
    let mut g: Box<HSD_GObj> = Box::new(core::mem::zeroed());
    let fd: &'static mut ftData = Box::leak(Box::new(core::mem::zeroed()));
    let ea: &'static mut ftLuigiAttributes = Box::leak(Box::new(core::mem::zeroed()));
    let backup: &'static mut ftLuigiAttributes = Box::leak(Box::new(core::mem::zeroed()));
    let items: &'static mut [*mut core::ffi::c_void; 8] = Box::leak(Box::new([core::ptr::null_mut(); 8]));
    items[0] = 0x77 as *mut _;
    fd.ext_attr = ea as *mut _ as *mut _;
    fd.x48_items = items.as_mut_ptr();
    f.ft_data = fd;
    f.dat_attrs_backup = backup as *mut _ as *mut _;
    g.user_data = &mut *f as *mut Fighter as *mut _;
    let gp: *mut HSD_GObj = &mut *g;

    // OnLoad: PUSH_ATTRS copy + the PC zeroing of the cyclone charge + the fireball article.
    t::ext::IT_8026B3F8 = Some(|article, kind| log(format!("article {:x} {kind}", article as usize)));
    f.u.lg.x222C_cycloneCharge = 5;
    t::ftLg_Init_OnLoad(gp);
    assert_eq!(take_log(), format!("article 77 {}", It_Kind_Luigi_Fire));
    assert_eq!(f.u.lg.x222C_cycloneCharge, 0);
    assert_eq!(f.dat_attrs, f.dat_attrs_backup);

    // LoadSpecialAttrs: COPY_ATTRS from ext_attr into dat_attrs.
    let words = core::mem::size_of::<ftLuigiAttributes>() / 4;
    let src = ea as *mut _ as *mut u32;
    for i in 0..words {
        *src.add(i) = 0x100 + i as u32;
    }
    t::ftLg_Init_LoadSpecialAttrs(gp);
    let dst = backup as *mut _ as *mut u32;
    for i in 0..words {
        assert_eq!(*dst.add(i), 0x100 + i as u32);
    }

    // Item / knockback callbacks.
    st::ITISHEAVY = Some(|_| 0);
    st::FTANIM_80070C48 = Some(|_, fl| log(format!("C48 {fl}")));
    st::FTANIM_80070CC4 = Some(|_, fl| log(format!("CC4 {fl}")));
    t::ftLg_Init_OnItemDrop(gp, 1);
    assert_eq!(f.x8B0[1].x10, -1);
    assert_eq!(take_log(), "CC4 1");

    // GetMotionFileString / UnkDemoCallbacks0.
    assert_eq!(t::ftLg_Init_GetMotionFileString(9), Some("ftDemoVi0102MotionFileLuigi"));
    assert_eq!(t::ftLg_Init_GetMotionFileString(10), Some("ftDemoVi1101MotionFileLuigi"));
    assert_eq!(t::ftLg_Init_GetMotionFileString(3), None);
    let (mut a, mut b) = (0, 0);
    t::ftLg_Init_UnkDemoCallbacks0(10, &mut a, &mut b);
    assert_eq!((a, b), (15, 15));
    t::ftLg_Init_UnkDemoCallbacks0(9, &mut a, &mut b);
    assert_eq!((a, b), (14, 14));
    t::ftLg_Init_UnkDemoCallbacks0(1, &mut a, &mut b);
    assert_eq!((a, b), (14, 14));
    assert_eq!(t::ftLg_Init_CostumeStrings[3][2], "PlyLuigi5KPi_Share_matanim_joint");
}
