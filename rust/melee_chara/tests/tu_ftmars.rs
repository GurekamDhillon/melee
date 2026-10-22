//! Smoke test for tu::ftmars (stateful; no fused sites) through recording hooks.
#![allow(static_mut_refs)]

use melee_chara::tu::ftmars as t;
use melee_types::*;

static mut LOG: std::vec::Vec<String> = std::vec::Vec::new();
fn take() -> std::vec::Vec<String> {
    unsafe { core::mem::take(&mut LOG) }
}

#[test]
fn smoke() {
    let mut f: Box<Fighter> = Box::new(Fighter::zeroed());
    let mut g: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    g.user_data = &mut *f as *mut Fighter as *mut _;
    let gp: *mut HSD_GObj = &mut *g;

    // PUSH_ATTRS / COPY_ATTRS
    let mut src = MarsAttributes::zeroed();
    src.x0 = 7;
    src.x5C = 2.5;
    src.x64.x0_bone_id = 3;
    let backup: &'static mut MarsAttributes = Box::leak(Box::new(MarsAttributes::zeroed()));
    let mut ftd: Box<ftData> = Box::new(unsafe { core::mem::zeroed() });
    ftd.ext_attr = &mut src as *mut _ as *mut _;
    f.ft_data = &mut *ftd;
    f.dat_attrs_backup = backup as *mut _ as *mut _;
    t::ftMs_Init_OnLoad(gp);
    assert_eq!(f.dat_attrs, f.dat_attrs_backup);
    assert_eq!((backup.x0, backup.x5C, backup.x64.x0_bone_id), (7, 2.5, 3));
    src.x0 = 9;
    t::ftMs_Init_LoadSpecialAttrs(gp);
    assert_eq!(backup.x0, 9);
    backup.x0 = 0;
    f.dat_attrs = core::ptr::null_mut();
    t::ftMs_Init_OnLoadForRoy(&mut *f);
    assert_eq!(backup.x0, 9);
    assert_eq!(f.dat_attrs, f.dat_attrs_backup);

    // Neutral-B end effects: Marth id, Roy id, none for other kinds; flag set once.
    unsafe {
        melee_chara::tu::ftmars::ext::EFSYNC_SPAWN = Some(|id, _, a| {
            LOG.push(format!("ef {id} {}", a.len()));
            core::ptr::null_mut()
        });
    }
    let mut parts: std::vec::Vec<FighterBone> = (0..4).map(|_| unsafe { core::mem::zeroed() }).collect();
    f.parts = parts.as_mut_ptr();
    f.accessory4_cb = Some(melee_chara::tu::ftmars::cb_ftMs_SpecialN_801365A8);
    unsafe { melee_fighter::hooks::FTLIB_GET_KIND = Some(|_| Ft_Kind_Mars) };
    t::ftMs_SpecialN_801365A8(gp);
    assert_eq!(take(), ["ef 1266 1"]);
    assert_eq!(f.x2219_b0(), 1);
    assert!(f.accessory4_cb.is_none() && f.pre_hitlag_cb.is_some() && f.post_hitlag_cb.is_some());
    t::ftMs_SpecialN_801365A8(gp);
    assert!(take().is_empty());
    f.set_x2219_b0(0);
    unsafe { melee_fighter::hooks::FTLIB_GET_KIND = Some(|_| Ft_Kind_Emblem) };
    t::ftMs_SpecialN_8013666C(gp);
    assert_eq!(take(), ["ef 1298 1"]);
    f.set_x2219_b0(0);
    unsafe { melee_fighter::hooks::FTLIB_GET_KIND = Some(|_| Ft_Kind_Fox) };
    t::ftMs_SpecialN_8013666C(gp);
    assert!(take().is_empty());
    assert_eq!(f.x2219_b0(), 1);

    assert_eq!(t::ftMs_Init_CostumeStrings[4], ["PlMsWh.dat", "PlyMars5KWh_Share_joint", "PlyMars5KWh_Share_matanim_joint"]);
}
