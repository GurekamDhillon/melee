//! Smoke test of tu::ftcrazyhandthrowncrazyhand.
#![allow(static_mut_refs)]
#[path = "common_ch/mod.rs"]
mod common;
use common::*;
use melee_chara::tu::ftcrazyhandthrowncrazyhand as t;
use melee_types::*;

#[test]
fn smoke() {
    let mut r = Rig::new();
    let gp = r.gp();
    hook_stubs(1);
    let mut vic = Box::new(Fighter::zeroed());
    vic.facing_dir = -1.0;
    let mut vg: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    vg.user_data = &mut *vic as *mut Fighter as *mut _;
    r.f.victim_gobj = &mut *vg;
    r.f.facing_dir = 1.0;
    r.f.set_invisible(1);
    t::ftCh_GrabUnk1_8015B850(gp, 0);
    assert_eq!(take_log(), "cms 0x153 0 0 1 0 true,anim8006EBA4");
    assert_eq!(r.f.facing_dir, -1.0);
    assert_eq!(r.f.invisible(), 0);
    assert_eq!(r.f.x1A6A, 511);
    assert!(r.f.accessory1_cb.is_some());
    t::ftCo_ThrownCrazyHand_Anim(gp);
    t::ftCo_ThrownCrazyHand_IASA(gp);
    t::ftCo_ThrownCrazyHand_Phys(gp);
    t::ftCo_ThrownCrazyHand_Coll(gp);
}
