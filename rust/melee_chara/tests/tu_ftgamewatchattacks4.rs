//! tu::ftgamewatchattacks4 smoke test: the Torch item's spawn / hitlag / removal and the Coll
//! callback through recording hooks. One #[test]: hooks are globals.
#![allow(static_mut_refs, unused_unsafe)]
mod gw_common;
use gw_common::*;
use melee_chara::tu::ftgamewatchattacks4 as t;
use melee_types::*;

#[test]
fn torch_lifecycle() {
    install_common();
    unsafe {
        t::ext::ITGAMEWATCHFIRE_SPAWN = Some(|_, pos, part, facing| unsafe {
            log(format!("spawn {} {} {} part {part} facing {facing}", (*pos).x, (*pos).y, (*pos).z));
            fake(0x300)
        });
        t::ext::ITGAMEWATCHFIRE_802C6A2C = Some(|i| log(format!("destroy {:x}", i as usize)));
        t::ext::ITGAMEWATCHFIRE_802C6A78 = Some(|i| log(format!("enter_hitlag {:x}", i as usize)));
        t::ext::ITGAMEWATCHFIRE_802C6A98 = Some(|i| log(format!("exit_hitlag {:x}", i as usize)));
    }
    let mut w = World::new();
    let gp = w.gp();
    w.f.facing_dir = 1.0;

    // An existing death callback is kept (`== NULL` guards), the missing one is installed.
    unsafe extern "C" fn mine(_: *mut HSD_GObj) {}
    w.f.death2_cb = Some(mine);
    w.f.accessory4_cb = Some(t::cb_ftGw_ItemTorchSetup);
    t::ftGw_ItemTorchSetup(gp);
    assert_eq!(take(), "spawn 1 2 3 part 32 facing 1");
    assert_eq!(unsafe { w.f.u.gw.x2254_fireGObj }, fake(0x300));
    assert!(w.f.death2_cb == Some(mine as unsafe extern "C" fn(*mut HSD_GObj)));
    assert!(w.f.take_dmg_cb.is_some() && w.f.pre_hitlag_cb.is_some() && w.f.post_hitlag_cb.is_some());
    assert!(w.f.accessory4_cb.is_none());

    t::ftGw_AttackS4_ItemTorchEnterHitlag(gp);
    t::ftGw_AttackS4_ItemTorchExitHitlag(gp);
    t::ftGw_AttackS4_ItemTorchOnDamage(gp);
    assert_eq!(take(), "enter_hitlag 300,exit_hitlag 300,destroy 300,exit_hitlag 300");
    assert!(unsafe { w.f.u.gw.x2254_fireGObj }.is_null());
    t::ftGw_AttackS4_ItemTorchOnDamage(gp);
    assert_eq!(take(), "");

    // Coll: shared collision, then the (air-only) item removal.
    unsafe { w.f.u.gw.x2254_fireGObj = fake(0x300) };
    w.f.ground_or_air = GA_Air;
    t::ftGw_AttackS4_Coll(gp);
    assert_eq!(take(), "coll,destroy 300,exit_hitlag 300");
}
