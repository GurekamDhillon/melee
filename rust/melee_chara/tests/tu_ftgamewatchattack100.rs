//! tu::ftgamewatchattack100 smoke test: Start Enter (through the ported ftCo_800D6B00 with its
//! hooks recording), the End animation hand-off and the collision callbacks.
//! One #[test]: hooks are globals.
#![allow(static_mut_refs, unused_unsafe)]
mod gw_common;
use gw_common::*;
use melee_chara::tu::ftgamewatchattack100 as t;
use melee_fighter::tu::ftCo_Attack100::ext::hook as ch;
use melee_types::*;

#[test]
fn rapid_jab() {
    install_common();
    unsafe {
        ch::ftpickupitem_80094790 = Some(|_| false);
        ch::Fighter_ChangeMotionState = Some(|_, ms, fl, start, _, _, _| log(format!("ms {ms} {fl:x} {start}")));
        ch::ftAnim_8006EBA4 = Some(|_| log("anim"));
        ch::ftAnim_IsFramesRemaining = Some(|_| unsafe { FRAMES_REMAINING != 0 });
        ch::ft_8008A2BC = Some(|_| log("wait"));
    }
    let mut w = World::new();
    let gp = w.gp();

    t::ftGw_Attack100Start_Enter(gp);
    assert_eq!(take(), "ms 342 0 0,anim");
    assert!(w.f.accessory4_cb.is_some());

    unsafe { FRAMES_REMAINING = 1 };
    t::ftGw_Attack100End_Anim(gp);
    assert_eq!(take(), "");
    unsafe { FRAMES_REMAINING = 0 };
    t::ftGw_Attack100End_Anim(gp);
    assert_eq!(take(), "wait");

    // Coll: the shared collision, then the item removal only when airborne.
    unsafe { w.f.u.gw.x224C_greenhouseGObj = core::ptr::null_mut() };
    w.f.ground_or_air = GA_Ground;
    t::ftGw_Attack100Start_Coll(gp);
    t::ftGw_Attack100Loop_Coll(gp);
    t::ftGw_Attack100End_Coll(gp);
    assert_eq!(take(), "coll,coll,coll");
}
