//! tu::ftgamewatchattack11 smoke test: the Insecticide Sprayer's spawn / state / hitlag / removal
//! and the jab Enter through recording hooks (no float arithmetic in the TU).
//! One #[test]: hooks are globals.
#![allow(static_mut_refs, unused_unsafe)]
mod gw_common;
use gw_common::*;
use melee_chara::tu::ftgamewatchattack11 as t;
use melee_types::*;

#[test]
fn greenhouse_lifecycle() {
    install_common();
    unsafe {
        t::ext::ITGAMEWATCHGREENHOUSE_SPAWN = Some(|_, pos, part, facing| unsafe {
            log(format!("spawn {} {} {} part {part} facing {facing}", (*pos).x, (*pos).y, (*pos).z));
            fake(0x100)
        });
        t::ext::ITGAMEWATCHGREENHOUSE_802C6430 = Some(|i| log(format!("jab {:x}", i as usize)));
        t::ext::ITGAMEWATCHGREENHOUSE_802C6458 = Some(|i| log(format!("start {:x}", i as usize)));
        t::ext::ITGAMEWATCHGREENHOUSE_802C6480 = Some(|i| log(format!("loop {:x}", i as usize)));
        t::ext::ITGAMEWATCHGREENHOUSE_802C64A8 = Some(|i| log(format!("end {:x}", i as usize)));
        t::ext::ITGAMEWATCHGREENHOUSE_802C6328 = Some(|i| log(format!("destroy {:x}", i as usize)));
        t::ext::ITGAMEWATCHGREENHOUSE_802C6374 = Some(|i| log(format!("enter_hitlag {:x}", i as usize)));
        t::ext::ITGAMEWATCHGREENHOUSE_802C6394 = Some(|i| log(format!("exit_hitlag {:x}", i as usize)));
    }
    let mut w = World::new();
    let gp = w.gp();
    w.f.facing_dir = -1.0;

    // Setup with no item yet: spawn, install the removal callbacks, clear the accessory callback.
    w.f.accessory4_cb = Some(t::cb_ftGw_Attack11_ItemGreenhouseSetup);
    t::ftGw_Attack11_ItemGreenhouseSetup(gp);
    assert_eq!(take(), "spawn 1 2 3 part 32 facing -1");
    assert_eq!(unsafe { w.f.u.gw.x224C_greenhouseGObj }, fake(0x100));
    assert!(w.f.death2_cb.is_some() && w.f.take_dmg_cb.is_some());
    assert!(w.f.pre_hitlag_cb.is_some() && w.f.post_hitlag_cb.is_some());
    assert!(w.f.accessory4_cb.is_none());

    // Setup with the item already out: DecideAction instead of a second spawn.
    w.f.motion_id = ftGw_MS_Attack100Loop as FtMotionId;
    t::ftGw_Attack11_ItemGreenhouseSetup(gp);
    assert_eq!(take(), "loop 100");

    // DecideAction per jab state; other states touch nothing.
    for (ms, want) in [
        (ftGw_MS_Attack11, "jab 100"),
        (ftGw_MS_Attack100Start, "start 100"),
        (ftGw_MS_Attack100Loop, "loop 100"),
        (ftGw_MS_Attack100End, "end 100"),
        (ftGw_MS_AttackLw3, ""),
    ] {
        w.f.motion_id = ms as FtMotionId;
        t::ftGw_Attack11_DecideAction(gp);
        assert_eq!(take(), want, "motion {ms}");
    }

    // Hitlag and removal.
    t::ftGw_Attack11_ItemGreenhouseEnterHitlag(gp);
    t::ftGw_Attack11_ItemGreenhouseExitHitlag(gp);
    t::ftGw_Attack11_ItemGreenhouseRemove(gp);
    assert_eq!(take(), "enter_hitlag 100,exit_hitlag 100,destroy 100,exit_hitlag 100");
    assert!(unsafe { w.f.u.gw.x224C_greenhouseGObj }.is_null());
    t::ftGw_Attack11_ItemGreenhouseRemove(gp);
    t::ftGw_Attack11_ItemGreenhouseEnterHitlag(gp);
    assert_eq!(take(), "");

    // Coll in the air removes the item, on the ground it does not.
    unsafe { w.f.u.gw.x224C_greenhouseGObj = fake(0x100) };
    w.f.ground_or_air = GA_Ground;
    t::ftGw_Attack11_Coll(gp);
    assert_eq!(take(), "coll");
    w.f.ground_or_air = GA_Air;
    t::ftGw_Attack11_Coll(gp);
    assert_eq!(take(), "coll,destroy 100,exit_hitlag 100");

    // Enter (an item already held: x1978 non-null skips the pickup search).
    w.f.x1978 = fake(0x200);
    w.f.co_attrs.jab_2_input_window = 7.5;
    w.f.set_allow_interrupt(1);
    t::ftGw_Attack11_Enter(gp);
    assert_eq!(take(), "ms 341 0 0,anim");
    assert_eq!((w.f.allow_interrupt(), w.f.hitlag_mul, w.f.unk_msid, w.f.x1A54), (0, 7.5, 44, 0));
    assert!(w.f.accessory4_cb.is_some());
}
