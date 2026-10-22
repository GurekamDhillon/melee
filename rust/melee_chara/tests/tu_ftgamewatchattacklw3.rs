//! tu::ftgamewatchattacklw3 smoke test: the Manhole item (and the swapped-out held item) through
//! recording hooks, plus Enter / Anim / IASA / Coll. One #[test]: hooks are globals.
#![allow(static_mut_refs, unused_unsafe)]
mod gw_common;
use gw_common::*;
use melee_chara::tu::ftgamewatchattacklw3 as t;
use melee_types::*;

#[test]
fn manhole() {
    install_common();
    unsafe {
        t::ext::IT_8026BB44 = Some(|i| log(format!("BB44 {:x}", i as usize)));
        t::ext::IT_8026B724 = Some(|i| log(format!("B724 {:x}", i as usize)));
        t::ext::IT_8026BB20 = Some(|i| log(format!("BB20 {:x}", i as usize)));
        t::ext::IT_8026B73C = Some(|i| log(format!("B73C {:x}", i as usize)));
        t::ext::IT_802C65E4 = Some(|_, pos, part, facing| unsafe {
            log(format!("spawn {} {} {} part {part} facing {facing}", (*pos).x, (*pos).y, (*pos).z));
            fake(0x400)
        });
        t::ext::IT_802C6718 = Some(|i| log(format!("destroy {:x}", i as usize)));
        t::ext::IT_802C6764 = Some(|i| log(format!("enter_hitlag {:x}", i as usize)));
        t::ext::IT_802C6784 = Some(|i| log(format!("exit_hitlag {:x}", i as usize)));
    }
    let mut w = World::new();
    let gp = w.gp();
    w.f.facing_dir = 1.0;

    // Without a held item: just the manhole spawns.
    t::ftGw_AttackLw3_ItemManholeSetup(gp);
    assert_eq!(take(), "spawn 1 2 3 part 32 facing 1");
    assert_eq!(unsafe { w.f.u.gw.x2250_manholeGObj2 }, fake(0x400));
    assert!(unsafe { w.f.u.gw.x2248_manholeGObj }.is_null());
    assert!(w.f.death2_cb.is_some() && w.f.take_dmg_cb.is_some());
    assert!(w.f.pre_hitlag_cb.is_some() && w.f.post_hitlag_cb.is_some() && w.f.accessory4_cb.is_none());

    t::ftGw_AttackLw3_ItemManholeEnterHitlag(gp);
    t::ftGw_AttackLw3_ItemManholeExitHitlag(gp);
    assert_eq!(take(), "enter_hitlag 400,exit_hitlag 400");

    // OnDamage destroys it; with nothing held, Remove only clears the pointer.
    t::ftGw_AttackLw3_ItemManholeOnDamage(gp);
    assert_eq!(take(), "destroy 400,exit_hitlag 400");
    assert!(unsafe { w.f.u.gw.x2250_manholeGObj2 }.is_null());
    t::ftGw_AttackLw3_ItemManholeOnDamage(gp);
    assert_eq!(take(), "");

    // Coll: shared collision, then the (air-only) removal.
    unsafe { w.f.u.gw.x2250_manholeGObj2 = fake(0x400) };
    w.f.ground_or_air = GA_Air;
    t::ftGw_AttackLw3_Coll(gp);
    assert_eq!(take(), "coll,destroy 400,exit_hitlag 400");

    // Enter / Anim.
    w.f.x1978 = fake(0x200);
    w.f.set_allow_interrupt(1);
    t::ftGw_AttackLw3_Enter(gp);
    assert_eq!(take(), "ms 345 0 0,anim");
    assert_eq!(w.f.allow_interrupt(), 0);
    assert!(w.f.accessory4_cb.is_some());
    unsafe { FRAMES_REMAINING = 1 };
    // IASA with allow_interrupt clear checks nothing.
    t::ftGw_AttackLw3_IASA(gp);
    assert_eq!(take(), "");
}
