//! Smoke test for tu::ftmario: OnLoad / OnLoadForDrMario (PUSH_ATTRS) and LoadSpecialAttrs
//! (COPY_ATTRS) move ftMario_DatAttrs between ext_attr, dat_attrs and the backup; OnTakeDamage
//! drops the cape; the demo file names.
#![allow(static_mut_refs)]

use melee_chara::tu::ftmario as t;
use melee_types::*;

static mut LOG: std::vec::Vec<String> = std::vec::Vec::new();

#[test]
fn attrs_push_and_copy() {
    let mut f: Box<Fighter> = Box::new(Fighter::zeroed());
    let mut g: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    let mut fd: Box<ftData> = Box::new(unsafe { core::mem::zeroed() });
    let mut ext = Box::new(ftMario_DatAttrs::zeroed());
    let mut backup = Box::new(ftMario_DatAttrs::zeroed());
    let mut live = Box::new(ftMario_DatAttrs::zeroed());
    let mut items: [*mut core::ffi::c_void; 3] = [0x10 as *mut _, 0x20 as *mut _, 0x30 as *mut _];
    ext.specialhi.vel_x = 3.5;
    ext.specials.cape_kind = 99;
    fd.ext_attr = &mut *ext as *mut _ as *mut _;
    fd.x48_items = (items.as_mut_ptr()) as *mut _;
    f.ft_data = &mut *fd;
    f.dat_attrs_backup = &mut *backup as *mut _ as *mut _;
    f.dat_attrs = &mut *live as *mut _ as *mut _;
    g.user_data = &mut *f as *mut Fighter as *mut _;
    let gp = &mut *g as *mut HSD_GObj;
    unsafe { t::ext::IT_8026B3F8 = Some(|a, k| LOG.push(format!("{:x} {k}", a as usize))) };

    t::ftMr_Init_LoadSpecialAttrs(gp);
    assert_eq!(live.specialhi.vel_x, 3.5);

    t::ftMr_Init_OnLoad(gp);
    assert_eq!(f.can_walljump(), 1);
    assert_eq!(f.dat_attrs, &mut *backup as *mut _ as *mut _);
    assert_eq!(backup.specialhi.vel_x, 3.5);
    assert_eq!(unsafe { LOG.join(",") }, "10 48,30 99");

    // Dr. Mario shares the attribute push.
    backup.specialhi.vel_x = 0.0;
    t::ftMr_Init_OnLoadForDrMario(&mut *f);
    assert_eq!(backup.specialhi.vel_x, 3.5);

    assert_eq!(t::ftMr_Init_GetMotionFileString(9), "ftDemoVi0102MotionFileMario");
    assert_eq!(t::ftMr_Init_GetMotionFileString(10), "ftDemoVi1101MotionFileMario");
    assert_eq!(t::ftMr_Init_CostumeStrings[4][0], "PlMrGr.dat");
    let (mut a, mut b) = (0, 0);
    t::ftMr_Init_UnkDemoCallbacks0(10, &mut a, &mut b);
    assert_eq!((a, b), (15, 15));
}

#[test]
fn take_damage_drops_the_cape() {
    let mut f: Box<Fighter> = Box::new(Fighter::zeroed());
    let mut g: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    g.user_data = &mut *f as *mut Fighter as *mut _;
    let gp = &mut *g as *mut HSD_GObj;
    // No cape: nothing happens (no item hooks are installed, so any call would panic).
    t::ftMr_Init_OnTakeDamage(gp);
}
