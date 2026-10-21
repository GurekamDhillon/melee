//! Smoke test of tu::ftclinkappeals (no float math to check against the C).
#![allow(static_mut_refs)]

use melee_chara::stubs as st;
use melee_chara::tu::ftclinkappeals as t;
use melee_types::*;

static mut LOG: std::vec::Vec<String> = std::vec::Vec::new();
fn log(s: String) {
    unsafe { LOG.push(s) }
}
static mut MILK: HSD_GObj = unsafe { core::mem::zeroed() };

#[test]
fn smoke() {
    let mut f: Box<Fighter> = Box::new(Fighter::zeroed());
    let mut g: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    g.user_data = &mut *f as *mut Fighter as *mut _;
    let gp: *mut HSD_GObj = &mut *g;
    unsafe {
        melee_fighter::hooks::FTPARTS_GET_BONE_INDEX = Some(|_, p| p + 100);
        t::ext::IT_802C8B28 = Some(|_, _, bone, facing| {
            log(format!("spawn {bone} {facing}"));
            &raw mut MILK
        });
        t::ext::FTCO_APPEALS_IASA = Some(|_| log("iasa".into()));
        t::ext::FT_80084280 = Some(|_| log("coll".into()));
        melee_chara::tu::ftclink::ext::IT_802C8C34 = Some(|_| log("drop".into()));
        st::FTANIM_ISFRAMESREMAINING = Some(|_| 1);
    }
    f.facing_dir = -1.0;
    f.cmd_vars[1] = 1;
    t::ftCl_AppealS_Anim(gp);
    assert!(!unsafe { f.u.lk.x18 }.is_null());
    assert!(f.take_dmg_cb.is_some() && f.death2_cb.is_some());
    f.cmd_vars[1] = 2;
    t::ftCl_AppealS_Anim(gp);
    assert!(unsafe { f.u.lk.x18 }.is_null());
    t::ftCl_AppealS_IASA(gp);
    t::ftCl_AppealS_Coll(gp);
    assert_eq!(unsafe { LOG.join(",") }, "spawn 131 -1,drop,iasa,coll");
}
