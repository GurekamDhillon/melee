//! Smoke test of tu::ftdrmario (no float math, so no C reference): the pill bookkeeping and
//! the init callbacks, through recording hooks.
#![allow(static_mut_refs)]

use core::ffi::c_void;
use melee_chara::stubs as st;
use melee_chara::tu::ftdrmario as t;
use melee_types::*;

static mut LOG: std::vec::Vec<String> = std::vec::Vec::new();
fn log(s: String) {
    unsafe { LOG.push(s) }
}
fn take() -> String {
    unsafe { std::mem::take(&mut LOG).join(",") }
}

static mut PILL: HSD_GObj = unsafe { core::mem::zeroed() };

#[test]
fn smoke() {
    let mut f: Box<Fighter> = Box::new(Fighter::zeroed());
    let mut g: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    let fd: &'static mut ftData = Box::leak(Box::new(unsafe { core::mem::zeroed() }));
    let sa: &'static mut ftDrMarioAttributes = Box::leak(Box::new(ftDrMarioAttributes::zeroed()));
    let mut items: [*mut c_void; 4] = [1000 as *mut c_void, 1001 as *mut c_void, 1002 as *mut c_void, 1003 as *mut c_void];
    fd.x48_items = (items.as_mut_ptr()) as *mut _;
    f.ft_data = fd;
    f.dat_attrs = sa as *mut _ as *mut _;
    g.user_data = &mut *f as *mut Fighter as *mut _;
    let gp: *mut HSD_GObj = &mut *g;
    unsafe {
        t::ext::FTMR_INIT_ONLOADFORDRMARIO = Some(|_| log("onload_for_dr".into()));
        t::ext::FTMR_INIT_LOADSPECIALATTRS = Some(|_| log("load_attrs".into()));
        t::ext::IT_8026B3F8 = Some(|a, k| log(format!("it {} {k}", a as usize)));
        t::ext::ITDRMARIOPILL_802C0DBC = Some(|_| log("pill_destroy".into()));
        t::ext::FTCO_800DEAE8 = Some(|_, a, b| log(format!("deae8 {a} {b}")));
        st::ITISHEAVY = Some(|_| 1);
        st::FTANIM_80070CC4 = Some(|_, fl| log(format!("cc4 {fl}")));
        st::FTANIM_80070C48 = Some(|_, fl| log(format!("c48 {fl}")));
    }

    // OnLoad: Mario's hook first, then the vitamin item (Dr. Mario's kind) and attr x14.
    sa.cape_kind = 99;
    t::ftDr_Init_OnLoad(gp);
    assert_eq!(take(), format!("onload_for_dr,it 1001 {},it 1003 99", It_Kind_DrMario_Vitamin));
    t::ftDr_Init_LoadSpecialAttrs(gp);
    assert_eq!(take(), "load_attrs");

    // Item callbacks: heavy item skips the pickup anim, Visible/Invisible pass flag 1.
    t::ftDr_Init_OnItemPickup(gp, 1);
    t::ftDr_Init_OnItemInvisible(gp);
    assert_eq!(take(), "");
    unsafe { st::ITISHEAVY = Some(|_| 0) };
    t::ftDr_Init_OnItemInvisible(gp);
    assert_eq!(take(), "cc4 1");
    unsafe { st::ITISHEAVY = Some(|_| 1) };

    // 801497CC: destroys the pill, forgets it, clears the damage / death callbacks.
    unsafe extern "C" fn cb(_: *mut HSD_GObj) {}
    f.u.mr.x2240 = unsafe { &raw mut PILL };
    f.take_dmg_cb = Some(cb);
    f.death2_cb = Some(cb);
    t::ftDr_Init_801497CC(gp);
    assert_eq!(take(), "pill_destroy");
    assert!(unsafe { f.u.mr.x2240 }.is_null() && f.take_dmg_cb.is_none() && f.death2_cb.is_none());
    t::ftDr_Init_801497CC(core::ptr::null_mut());
    t::ftDr_Init_80149540(gp);
    assert_eq!(take(), "");

    // 80149844: false only while an Appeal motion (341 / 342) still holds a pill.
    assert!(t::ftDr_Init_80149844(core::ptr::null_mut()));
    f.motion_id = 0x155;
    assert!(t::ftDr_Init_80149844(gp));
    f.u.mr.x2240 = unsafe { &raw mut PILL };
    assert!(!t::ftDr_Init_80149844(gp));
    f.motion_id = 0x156;
    assert!(!t::ftDr_Init_80149844(gp));
    f.motion_id = 0x157;
    assert!(t::ftDr_Init_80149844(gp));

    // 801498A0: forget the pill without destroying it.
    f.take_dmg_cb = Some(cb);
    f.death2_cb = Some(cb);
    t::ftDr_Init_801498A0(gp);
    assert!(unsafe { f.u.mr.x2240 }.is_null() && f.take_dmg_cb.is_none() && f.death2_cb.is_none());
    assert_eq!(take(), "");

    // 801498EC / 80149910.
    f.cmd_vars[1] = 7;
    assert_eq!(t::ftDr_Init_801498EC(gp), 7);
    assert_eq!(t::ftDr_Init_801498EC(core::ptr::null_mut()), 0);
    t::ftDr_Init_80149910(gp);
    assert_eq!(take(), "deae8 341 342");
    assert_eq!((f.cmd_vars[0], f.cmd_vars[1]), (1, 0));

    assert_eq!(t::ftDr_Init_CostumeStrings[4][0], "PlDrBk.dat");
    assert_eq!(t::ftDr_Init_DatFilename, "PlDr.dat");
}
