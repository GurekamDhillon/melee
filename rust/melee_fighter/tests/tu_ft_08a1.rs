//! Smoke test for src/tu/ft_08a1.rs (ft_8008A2BC / ft_8008A324 / ft_8008A348): every callee is
//! a recording stub; checks the branch order and the state written.
#![allow(static_mut_refs)]

use melee_fighter::hooks;
use melee_fighter::tu::ft_08a1::{self as m, ext};
use melee_fighter::types::*;

static mut LOG: std::vec::Vec<String> = std::vec::Vec::new();
fn log(s: String) {
    unsafe { LOG.push(s) }
}
static mut KIND: FighterKind = 0;
static mut C5240: bool = false;
static mut A698: bool = false;

fn setup() {
    unsafe {
        LOG.clear();
        hooks::FTLIB_GET_KIND = Some(|_| KIND);
        hooks::FTCO_800C5240 = Some(|_| C5240);
        hooks::IT_GET_KIND = Some(|_| It_Kind_Peach_Parasol as i32);
        hooks::FTDATA_80085FD4_X8 = Some(|_, i| { log(format!("FD4 {i}")); 1 });
        ext::FTMH_MS_389_80151018 = Some(|_| log("MH".into()));
        ext::FTCH_GRABUNK1_8015BC88 = Some(|_| log("CH".into()));
        ext::FTCO_DOWNSPOT_ENTER = Some(|_| log("DOWNSPOT".into()));
        ext::FTCO_800C4ED8 = Some(|_| log("4ED8".into()));
        ext::IT_802BDB94 = Some(|_| log("PARASOL".into()));
        ext::FIGHTER_CHANGE_MOTION_STATE = Some(|_, ms, fl, a, r, b, o| log(format!("CMS {ms} {fl} {a} {r} {b} {}", o.is_null())));
        ext::FTCO_8008A698 = Some(|_| A698);
        ext::FTCO_8008A6D8 = Some(|_, ms| log(format!("A6D8 {ms}")));
        ext::FTLK_ATTACKAIR_800EB3BC = Some(|_| log("LINK".into()));
        ext::FTCL_INIT_8014919C = Some(|_| log("CLINK".into()));
    }
}

#[test]
fn ft_08a1_smoke() {
    let mut cd: Box<ftCommonData> = Box::new(ftCommonData::zeroed());
    cd.x5F0 = 0x1234;
    unsafe { hooks::P_FT_COMMON_DATA = &*cd };
    let mut fp: Box<Fighter> = Box::new(Fighter::zeroed());
    let mut g: Box<HSD_GObj> = Box::new(HSD_GObj::zeroed());
    g.user_data = &mut *fp as *mut Fighter as *mut core::ffi::c_void;
    let gp: *mut HSD_GObj = &mut *g;
    setup();

    unsafe { KIND = Ft_Kind_MasterH };
    m::ft_8008A2BC(gp);
    unsafe { KIND = Ft_Kind_CrezyH };
    m::ft_8008A2BC(gp);
    assert_eq!(unsafe { LOG.clone() }, ["MH", "CH"]);

    setup();
    unsafe { KIND = Ft_Kind_Mario };
    fp.kind = Ft_Kind_Link;
    m::ft_8008A2BC(gp);
    assert_eq!(unsafe { LOG.clone() }, ["CMS 14 0 0 1 0 true", "LINK"]);
    assert_eq!(fp.x209A, 0x1234);

    setup();
    fp.kind = Ft_Kind_Peach;
    let mut item: Box<HSD_GObj> = Box::new(HSD_GObj::zeroed());
    fp.item_gobj = &mut *item;
    unsafe { A698 = true };
    m::ft_8008A324(gp);
    assert_eq!(unsafe { LOG.clone() }, ["PARASOL", "CMS 14 0 0 1 -1 true", "FD4 6", "A6D8 6"]);

    setup();
    unsafe { C5240 = true };
    m::ft_8008A348(gp, 0.0);
    assert_eq!(unsafe { LOG.clone() }, ["4ED8"]);
    fp.set_x2224_b2(1);
    setup();
    m::ft_8008A348(gp, 0.0);
    assert_eq!(unsafe { LOG.clone() }, ["DOWNSPOT"]);
}
