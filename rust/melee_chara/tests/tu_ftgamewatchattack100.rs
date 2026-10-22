//! Smoke test for tu::ftgamewatchattack100 (stateful, no float arithmetic) through recording hooks.
//! One #[test]: the hooks are process globals.
#![allow(static_mut_refs)]

use melee_chara::stubs as st;
use melee_chara::tu::ftgamewatchattack100 as t;
use melee_types::*;

static mut LOG: std::vec::Vec<String> = std::vec::Vec::new();
fn log(s: &str) {
    unsafe { LOG.push(s.to_string()) }
}
fn take() -> std::vec::Vec<String> {
    unsafe { core::mem::take(&mut LOG) }
}

#[test]
fn smoke() {
    let mut f: Box<Fighter> = Box::new(Fighter::zeroed());
    let mut g: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    g.user_data = &mut *f as *mut Fighter as *mut _;
    let gp: *mut HSD_GObj = &mut *g;

    unsafe {
        t::ext::FTCO_800D6B00 = Some(|_, msid| log(&format!("800D6B00 {msid}")));
        t::ext::FTCO_800D6C60 = Some(|g, cb| {
            log("800D6C60");
            cb(g);
        });
        t::ext::FTCO_ATTACK100LOOP_IASA = Some(|_| log("loop iasa"));
        t::ext::FTCO_ATTACK100END_ANIM = Some(|_| log("end anim"));
        st::FIGHTER_CHANGEMOTIONSTATE = Some(|_, ms, fl, _, _, _, _| log(&format!("ms {ms} {fl}")));
        st::FTANIM_ISFRAMESREMAINING = Some(|_| 0);
        st::FT_80084104 = Some(|_| log("coll"));
    }

    // Start: pickup-aware common entry, sprayer setup accessory.
    t::ftGw_Attack100Start_Enter(gp);
    assert_eq!(take(), [format!("800D6B00 {}", ftGw_MS_Attack100Start)]);
    assert!(f.accessory4_cb.is_some());
    f.accessory4_cb = None;

    // Start anim ends -> Loop enter (re-exported r3 port) installs DecideAction.
    unsafe { st::FTANIM_ISFRAMESREMAINING = Some(|_| 0) };
    t::ftGw_Attack100Start_Anim(gp);
    assert_eq!(take(), [format!("ms {} 0", ftGw_MS_Attack100Loop)]);
    assert!(f.accessory4_cb.is_some());
    f.accessory4_cb = None;
    unsafe { st::FTANIM_ISFRAMESREMAINING = Some(|_| 1) };
    t::ftGw_Attack100Start_Anim(gp);
    assert!(take().is_empty());

    // Loop anim defers to the common helper with End_Enter as the callback.
    t::ftGw_Attack100Loop_Anim(gp);
    assert_eq!(take(), ["800D6C60".to_string(), format!("ms {} 0", ftGw_MS_Attack100End)]);
    assert!(f.accessory4_cb.is_some());

    t::ftGw_Attack100Loop_IASA(gp);
    t::ftGw_Attack100End_Anim(gp);
    assert_eq!(take(), ["loop iasa", "end anim"]);

    // Coll: ground collision, then the airborne-only item drop (no items out: nothing more).
    f.ground_or_air = GA_Ground;
    t::ftGw_Attack100Start_Coll(gp);
    t::ftGw_Attack100Loop_Coll(gp);
    t::ftGw_Attack100End_Coll(gp);
    assert_eq!(take(), ["coll", "coll", "coll"]);
}
