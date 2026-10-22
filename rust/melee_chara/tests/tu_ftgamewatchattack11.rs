//! Smoke test for tu::ftgamewatchattack11 (stateful, no float arithmetic) through recording hooks.
//! One #[test]: the hooks are process globals.
#![allow(static_mut_refs)]

use melee_chara::stubs as st;
use melee_chara::tu::ftgamewatch as gw;
use melee_chara::tu::ftgamewatchattack11 as t;
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
    let mut parts: std::vec::Vec<FighterBone> = (0..80).map(|_| unsafe { core::mem::zeroed() }).collect();
    f.parts = parts.as_mut_ptr();
    g.user_data = &mut *f as *mut Fighter as *mut _;
    let gp: *mut HSD_GObj = &mut *g;
    let item = 0x1234usize as *mut HSD_GObj;

    unsafe {
        t::ext::LB_8000B1CC = Some(|_, off, out| {
            assert!(off.is_null());
            *out = Vec3 { x: 1.0, y: 2.0, z: 3.0 };
            log("joint pos");
        });
        t::ext::ITGAMEWATCHGREENHOUSE_SPAWN = Some(|_, pos, part, dir| {
            log(&format!("spawn {} {} {part} {dir}", (*pos).x, (*pos).z));
            0x1234usize as *mut HSD_GObj
        });
        t::ext::ITGAMEWATCHGREENHOUSE_802C6430 = Some(|_| log("jab"));
        t::ext::ITGAMEWATCHGREENHOUSE_802C6458 = Some(|_| log("100 start"));
        t::ext::ITGAMEWATCHGREENHOUSE_802C6480 = Some(|_| log("100 loop"));
        t::ext::ITGAMEWATCHGREENHOUSE_802C64A8 = Some(|_| log("100 end"));
        t::ext::ITGAMEWATCHGREENHOUSE_802C6328 = Some(|_| log("remove"));
        t::ext::ITGAMEWATCHGREENHOUSE_802C6374 = Some(|_| log("enter hitlag"));
        t::ext::ITGAMEWATCHGREENHOUSE_802C6394 = Some(|_| log("exit hitlag"));
        t::ext::FTPICKUPITEM_80094790 = Some(|_| false);
        t::ext::FTCO_ATTACK11_IASA = Some(|_| log("iasa"));
        st::FIGHTER_CHANGEMOTIONSTATE = Some(|_, ms, fl, _, _, _, _| log(&format!("ms {ms} {fl}")));
        st::FTANIM_8006EBA4 = Some(|_| {});
        st::FT_80084104 = Some(|_| log("coll"));
    }

    // Setup spawns the sprayer at the left hand and installs the damage / hitlag callbacks.
    f.facing_dir = -1.0;
    t::ftGw_Attack11_ItemGreenhouseSetup(gp);
    assert_eq!(take(), ["joint pos", "spawn 1 3 32 -1"]);
    assert_eq!(unsafe { f.u.gw.x224C_greenhouseGObj }, item);
    assert!(f.death2_cb.is_some() && f.take_dmg_cb.is_some() && f.pre_hitlag_cb.is_some() && f.post_hitlag_cb.is_some());
    assert!(f.accessory4_cb.is_none());

    // Second call with the item out re-decides from the motion state.
    for (msid, what) in [
        (ftGw_MS_Attack11, "jab"),
        (ftGw_MS_Attack100Start, "100 start"),
        (ftGw_MS_Attack100Loop, "100 loop"),
        (ftGw_MS_Attack100End, "100 end"),
    ] {
        f.motion_id = msid as FtMotionId;
        f.accessory4_cb = Some(t::cb_ftGw_Attack11_DecideAction);
        t::ftGw_Attack11_ItemGreenhouseSetup(gp);
        assert_eq!(take(), [what]);
        assert!(f.accessory4_cb.is_none());
    }

    // Hitlag and removal.
    unsafe { (f.pre_hitlag_cb.unwrap())(gp) };
    unsafe { (f.post_hitlag_cb.unwrap())(gp) };
    t::ftGw_Attack11_ItemGreenhouseRemove(gp);
    assert_eq!(take(), ["enter hitlag", "exit hitlag", "remove", "exit hitlag"]);
    assert!(unsafe { f.u.gw.x224C_greenhouseGObj }.is_null());
    t::ftGw_Attack11_ItemGreenhouseRemove(gp);
    assert!(take().is_empty());

    // Jab state ranges.
    f.motion_id = ftGw_MS_Attack100Start as FtMotionId;
    assert!(!t::ftGw_Attack11_ItemGreenhouse_CheckAll(gp));
    assert!(t::ftGw_Attack11_ItemGreenhouse_CheckAttack11(gp));
    f.motion_id = ftGw_MS_AttackLw3 as FtMotionId;
    assert!(t::ftGw_Attack11_ItemGreenhouse_CheckAll(gp));
    f.motion_id = ftGw_MS_Attack11 as FtMotionId;
    assert!(!t::ftGw_Attack11_ItemGreenhouse_CheckAttack11(gp));

    // Enter sets up the jab and the sprayer accessory.
    f.co_attrs.jab_2_input_window = 7.0;
    f.x1A54 = 9;
    t::ftGw_Attack11_Enter(gp);
    assert_eq!(take(), [format!("ms {} 0", ftGw_MS_Attack11)]);
    assert_eq!((f.hitlag_mul, f.unk_msid, f.x1A54), (7.0, 44, 0));
    assert!(f.accessory4_cb.is_some());
    unsafe { t::ext::FTPICKUPITEM_80094790 = Some(|_| true) };
    f.x1A54 = 9;
    t::ftGw_Attack11_Enter(gp);
    assert!(take().is_empty() && f.x1A54 == 9);

    // IASA and Coll (Coll: airborne drops the items).
    t::ftGw_Attack11_IASA(gp);
    assert_eq!(take(), ["iasa"]);
    f.ground_or_air = GA_Ground;
    t::ftGw_Attack11_Coll(gp);
    assert_eq!(take(), ["coll"]);
    let _ = gw::ftGw_Init_8014A538;
}
