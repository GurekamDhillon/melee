//! Smoke test of tu::ftmewtwo: the damage / death callbacks, the full-charge check and the data strings,
//! through recording hooks.
#![allow(static_mut_refs)]
use melee_chara::stubs as st;
use melee_types::*;

static mut LOG: std::vec::Vec<String> = std::vec::Vec::new();
fn log(s: String) {
    unsafe { LOG.push(s) }
}
fn take() -> String {
    unsafe { std::mem::take(&mut LOG).join(",") }
}
static mut FRAMES_LEFT: i32 = 0;

use melee_chara::tu::ftmewtwo as t;
use melee_chara::tu::{ftmewtwospeciallw as lw, ftmewtwospecialn as sn};

#[test]
fn smoke() {
    let mut attrs: Box<ftMewtwoAttributes> = Box::new(unsafe { core::mem::zeroed() });
    let mut f: Box<Fighter> = Box::new(Fighter::zeroed());
    f.dat_attrs = &mut *attrs as *mut _ as *mut _;
    let mut g: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    g.user_data = &mut *f as *mut Fighter as *mut _;
    let gp: *mut HSD_GObj = &mut *g;

    // Death2 / TakeDamage: destroy the Disable item (lw) and drop the held ball (n).
    let mut disable: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    let mut ball: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    unsafe {
        lw::ext::ITMEWTWODISABLE_LOGIC67_DESTROY = Some(|_| log("destroy_disable".into()));
        sn::ext::IT_802C573C = Some(|_| log("destroy_ball".into()));
        sn::ext::FTCO_800BFFAC = Some(|_| log("ffac".into()));
        st::EFLIB_DESTROYALL = Some(|_| log("efdestroy".into()));
    }
    f.u.mt.x222C_disableGObj = &mut *disable;
    f.u.mt.x2230_shadowHeldGObj = &mut *ball;
    t::ftMt_Init_OnDeath2(gp);
    assert_eq!(take(), "destroy_disable,destroy_ball,ffac,efdestroy");
    assert!(unsafe { f.u.mt.x222C_disableGObj.is_null() && f.u.mt.x2230_shadowHeldGObj.is_null() });
    t::ftMt_Init_OnTakeDamage(gp);
    // Nothing held now; the C's OnDeath still runs its charge reset (charge 0 != cycles 0.0 is false).
    assert_eq!(take(), "");
    t::ftMt_Init_OnItemPickup(gp, 1);
    t::ftMt_Init_OnItemDrop(gp, 1);

    // UnkMotionStates4: only at full charge.
    attrs.x0_MEWTWO_SHADOWBALL_CHARGE_CYCLES = 7.0;
    unsafe { t::ext::FTCO_800BFFD0 = Some(|_, a, b| { log(format!("ffd0 {a} {b}")); true }) };
    f.u.mt.x2234_shadowBallCharge = 6;
    t::ftMt_Init_UnkMotionStates4(gp);
    assert_eq!(take(), "");
    f.u.mt.x2234_shadowBallCharge = 7;
    t::ftMt_Init_UnkMotionStates4(gp);
    assert_eq!(take(), "ffd0 92 0");

    assert_eq!(t::ftMt_Init_DatFilename, "PlMt.dat");
    assert_eq!(t::ftMt_Init_CostumeStrings[3][0], "PlMtGr.dat");
}
