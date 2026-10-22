//! Smoke test of tu::ftfox (no float math, so no C reference): the attribute push / copy macros
//! and the init callbacks, through recording hooks.
#![allow(static_mut_refs)]

use core::ffi::c_void;
use melee_chara::stubs as st;
use melee_chara::tu::ftfox as t;
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
    let live: &'static mut ftFox_DatAttrs = Box::leak(Box::new(ftFox_DatAttrs::zeroed()));
    ext_attr.x0_FOX_BLASTER_UNK1 = 1.5;
    ext_attr.x1C_FOX_BLASTER_SHOT_ITKIND = 11;
    ext_attr.x20_FOX_BLASTER_GUN_ITKIND = 12;
    let mut items: [*mut c_void; 3] = [1000 as *mut c_void, 1001 as *mut c_void, 1002 as *mut c_void];
    fd.x48_items = items.as_mut_ptr();
    fd.ext_attr = ext_attr as *mut _ as *mut c_void;
    f.ft_data = fd;
    f.dat_attrs_backup = backup as *mut _ as *mut c_void;
    g.user_data = &mut *f as *mut Fighter as *mut _;
    let gp: *mut HSD_GObj = &mut *g;
    unsafe {
        t::ext::IT_8026B3F8 = Some(|a, k| log(format!("it {} {k}", a as usize)));
        t::ext::FTFX_SPECIALN_REMOVEBLASTER = Some(|_| log("remove_blaster".into()));
        st::ITISHEAVY = Some(|_| 1);
    }

    // OnLoad: wall jump on, the attrs pushed into the backup buffer (which becomes dat_attrs),
    // then the blaster shot / gun items and the Illusion item.
    t::ftFx_Init_OnLoad(gp);
    assert_eq!(f.can_walljump(), 1);
    assert_eq!(f.dat_attrs, backup as *mut _ as *mut c_void);
    assert_eq!((backup.x0_FOX_BLASTER_UNK1, backup.x1C_FOX_BLASTER_SHOT_ITKIND), (1.5, 11));
    assert_eq!(take(), format!("it 1000 11,it 1001 12,it 1002 {}", It_Kind_Fox_Illusion));

    // OnLoadForFalco: the push alone.
    backup.x0_FOX_BLASTER_UNK1 = 0.0;
    t::ftFx_Init_OnLoadForFalco(&mut *f);
    assert_eq!(backup.x0_FOX_BLASTER_UNK1, 1.5);
    assert_eq!(take(), "");

    // LoadSpecialAttrs: copy ext_attr over the live dat_attrs block.
    f.dat_attrs = live as *mut _ as *mut c_void;
    t::ftFx_Init_LoadSpecialAttrs(gp);
    assert_eq!((live.x0_FOX_BLASTER_UNK1, live.x20_FOX_BLASTER_GUN_ITKIND), (1.5, 12));

    // Blaster state and removal.
    assert!(!t::ftFx_Init_800E5534(gp));
    f.u.fx.x222C_blasterGObj = gp;
    assert!(t::ftFx_Init_800E5534(gp));
    t::ftFx_Init_800E5588(gp);
    assert_eq!(take(), "remove_blaster");

    // Item callbacks with a heavy item skip the anim hooks.
    t::ftFx_Init_OnItemPickup(gp, 1);
    t::ftFx_Init_OnItemDrop(gp, 0);
    t::ftFx_Init_OnItemInvisible(gp);
    t::ftFx_Init_OnItemVisible(gp);
    assert_eq!(take(), "");

    assert_eq!(t::ftFx_Init_CostumeStrings[3][2], "PlyFox5KGr_Share_matanim_joint");
    assert_eq!(t::ftFx_Init_AnimDatFilename, "PlFxAJ.dat");
}
