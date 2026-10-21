//! Smoke test of tu::ftemblem (no float math, so no C reference): the init callbacks, through
//! recording hooks.
#![allow(static_mut_refs)]

use melee_chara::stubs as st;
use melee_chara::tu::ftemblem as t;
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
    g.user_data = &mut *f as *mut Fighter as *mut _;
    let gp: *mut HSD_GObj = &mut *g;
    unsafe {
        t::ext::FTMS_INIT_ONLOADFORROY = Some(|fp| log(format!("roy {}", fp.is_null())));
        t::ext::FTMS_INIT_LOADSPECIALATTRS = Some(|_| log("load_attrs".into()));
        st::ITISHEAVY = Some(|_| 0);
        st::FTANIM_80070CC4 = Some(|_, fl| log(format!("cc4 {fl}")));
        st::FTANIM_80070C48 = Some(|_, fl| log(format!("c48 {fl}")));
    }
    t::ftFe_Init_OnLoad(gp);
    t::ftFe_Init_LoadSpecialAttrs(gp);
    assert_eq!(take(), "roy false,load_attrs");

    // Roy passes flag 0 to the invisible / visible anim hooks (Fox passes 1).
    t::ftFe_Init_OnItemInvisible(gp);
    t::ftFe_Init_OnItemVisible(gp);
    assert_eq!(take(), "cc4 0,c48 0");

    assert_eq!(t::ftFe_Init_CostumeStrings[4][0], "PlFeYe.dat");
    assert_eq!(t::ftFe_Init_DataName, "ftDataEmblem");
}
