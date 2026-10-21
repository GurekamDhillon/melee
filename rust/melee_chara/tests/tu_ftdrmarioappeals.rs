//! Smoke test of tu::ftdrmarioappeals (no float math, so no C reference): the vitamin taunt's
//! spawn / drop / end-of-motion flow through recording hooks.
#![allow(static_mut_refs)]

use melee_chara::stubs as st;
use melee_chara::tu::ftdrmarioappeals as t;
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
    let mut parts: [FighterBone; 1] = unsafe { core::mem::zeroed() };
    f.parts = parts.as_mut_ptr();
    g.user_data = &mut *f as *mut Fighter as *mut _;
    let gp: *mut HSD_GObj = &mut *g;
    unsafe {
        t::ext::LB_8000B1CC = Some(|_, p0, _| log(format!("lb null={}", p0.is_null())));
        t::ext::FTMR_SPECIALN_VITAMINRANDOM = Some(|_| 2);
        t::ext::ITDRMARIOPILL_APPEAL_SPAWN = Some(|_, _, idx, kind, facing| {
            log(format!("spawn {idx} {kind} {facing}"));
            &raw mut PILL
        });
        t::ext::ITDRMARIOPILL_802C0DBC = Some(|_| log("pill_destroy".into()));
        t::ext::FTCO_APPEALS_IASA = Some(|_| log("iasa".into()));
        t::ext::FT_80084280 = Some(|_| log("coll".into()));
        st::FTANIM_ISFRAMESREMAINING = Some(|_| 1);
        st::FT_8008A2BC = Some(|_| log("wait".into()));
    }

    // cmd_vars[0] == 1 with no pill: spawn it and arm the damage / death callbacks.
    f.cmd_vars[0] = 1;
    f.facing_dir = -1.0;
    t::ftDr_AppealS_Anim(gp);
    assert_eq!(take(), format!("lb null=true,spawn 2 {} -1", It_Kind_DrMario_Vitamin));
    assert!(!unsafe { f.u.mr.x2240 }.is_null());
    assert!(f.take_dmg_cb.is_some() && f.death2_cb.is_some());

    // Already holding one: no second spawn.
    t::ftDr_AppealS_Anim(gp);
    assert_eq!(take(), "");

    // cmd_vars[0] == 2: the pill is thrown away (destroyed) and the callbacks cleared.
    f.cmd_vars[0] = 2;
    t::ftDr_AppealS_Anim(gp);
    assert_eq!(take(), "pill_destroy");
    assert!(unsafe { f.u.mr.x2240 }.is_null() && f.take_dmg_cb.is_none() && f.death2_cb.is_none());

    // Animation over: a still-held pill is destroyed, then back to Wait.
    f.cmd_vars[0] = 0;
    f.u.mr.x2240 = unsafe { &raw mut PILL };
    f.take_dmg_cb = f.death2_cb;
    unsafe { st::FTANIM_ISFRAMESREMAINING = Some(|_| 0) };
    t::ftDr_AppealS_Anim(gp);
    assert_eq!(take(), "pill_destroy,wait");
    assert!(unsafe { f.u.mr.x2240 }.is_null());

    t::ftDr_AppealS_IASA(gp);
    t::ftDr_AppealS_Coll(gp);
    assert_eq!(take(), "iasa,coll");
}
