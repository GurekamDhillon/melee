//! Smoke test for tu::ftgamewatchattackair (stateful, no float arithmetic) through recording hooks.
//! One #[test]: the hooks are process globals.
#![allow(static_mut_refs)]

use melee_chara::stubs as st;
use melee_chara::tu::ftgamewatchattackair as t;
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

    unsafe {
        t::ext::LB_8000B1CC = Some(|_, _, out| *out = Vec3 { x: 4.0, y: 5.0, z: 6.0 });
        t::ext::IT_802C6C38 = Some(|_, p, part, d| {
            log(&format!("parachute {} {part} {d}", (*p).x));
            0x10usize as *mut HSD_GObj
        });
        t::ext::IT_802C6F40 = Some(|_, p, part, d| {
            log(&format!("turtle {} {part} {d}", (*p).y));
            0x20usize as *mut HSD_GObj
        });
        t::ext::IT_802C720C = Some(|_, p, part, d| {
            log(&format!("sparky {} {part} {d}", (*p).z));
            0x30usize as *mut HSD_GObj
        });
        t::ext::IT_802C6E50 = Some(|_| log("parachute land"));
        t::ext::IT_802C7158 = Some(|_| log("turtle land"));
        t::ext::IT_802C7424 = Some(|_| log("sparky land"));
        t::ext::IT_802C6D6C = Some(|_| log("parachute remove"));
        t::ext::IT_802C7074 = Some(|_| log("turtle remove"));
        t::ext::IT_802C7340 = Some(|_| log("sparky remove"));
        t::ext::IT_802C6DB8 = Some(|_| log("parachute +hitlag"));
        t::ext::IT_802C70C0 = Some(|_| log("turtle +hitlag"));
        t::ext::IT_802C738C = Some(|_| log("sparky +hitlag"));
        t::ext::IT_802C6DD8 = Some(|_| log("parachute -hitlag"));
        t::ext::IT_802C70E0 = Some(|_| log("turtle -hitlag"));
        t::ext::IT_802C73AC = Some(|_| log("sparky -hitlag"));
        t::ext::FTCO_ATTACKAIR_GETMSIDFROMCSTICK = Some(|fp| (*fp).cmd_vars[3] as i32);
        t::ext::FTCO_ATTACKAIR_ENTERFROMMSID = Some(|_, msid| log(&format!("enter {msid}")));
        t::ext::FTCO_ATTACKAIR_ANIM = Some(|_| log("air anim"));
        t::ext::FTCO_ATTACKAIRN_IASA = Some(|_| log("n iasa"));
        t::ext::FTCO_ATTACKAIRB_IASA = Some(|_| log("b iasa"));
        t::ext::FTCO_ATTACKAIRHI_IASA = Some(|_| log("hi iasa"));
        t::ext::FTCO_LANDING_ENTER_BASIC = Some(|_| log("landing basic"));
        t::ext::FTCO_LANDINGAIR_ENTERWITHMSIDLAG = Some(|_, msid, lag| log(&format!("landing air {msid} {lag}")));
        t::ext::FTCO_LANDING_ANIM = Some(|g| {
            log("landing anim");
            (*((*g).user_data as *mut Fighter)).motion_id = 0;
        });
        t::ext::FTCO_LANDING_COLL = Some(|_| log("landing coll"));
        t::ext::FTCO_LANDING_PHYS = Some(|_| log("landing phys"));
        melee_chara::tu::ftgamewatch::ext::FTGW_ATTACKLW3_ITEMMANHOLEONDAMAGE = Some(|_| {});
        melee_chara::tu::ftgamewatch::ext::FTGW_ATTACKS4_ITEMTORCHONDAMAGE = Some(|_| {});
        melee_chara::tu::ftgamewatch::ext::FTGW_SPECIALS_ITEMJUDGEMENTREMOVE = Some(|_| {});
        melee_chara::tu::ftgamewatch::ext::FTGW_SPECIALLW_ITEMPANICREMOVE = Some(|_| {});
        melee_chara::tu::ftgamewatch::ext::FTGW_SPECIALHI_ITEMRESCUEREMOVE = Some(|_| {});
        melee_chara::tu::ftgamewatch::ext::FTGW_SPECIALLW_ABSORBTHINK_DECIDEACTION = Some(|_| {});
        melee_chara::tu::ftgamewatch::ext::FTGW_ATTACKLW3_ITEMMANHOLEONDAMAGE = Some(|_| log("manhole"));
        st::FT_80082C74 = Some(|_, cb| log(&format!("coll cb {}", cb.is_some())));
        st::FT_80084DB0 = Some(|_| log("air phys"));
        melee_chara::tu::ftgamewatchattack11::ext::ITGAMEWATCHGREENHOUSE_802C6328 = Some(|_| {});
    }

    // Setup: spawns at the right bone and installs the shared callbacks.
    f.facing_dir = 1.0;
    t::ftGw_AttackAirN_ItemParachuteSetup(gp);
    t::ftGw_AttackAirN_ItemTurtleSetup(gp);
    t::ftGw_AttackAirN_ItemSparkySetup(gp);
    assert_eq!(
        take(),
        [
            format!("parachute 4 {} 1", FtPart_TransN),
            format!("turtle 5 {} 1", FtPart_LShoulderN),
            format!("sparky 6 {} 1", FtPart_LHandN)
        ]
    );
    unsafe {
        assert_eq!((f.u.gw.x2258_parachuteGObj as usize, f.u.gw.x225C_turtleGObj as usize, f.u.gw.x2260_sparkyGObj as usize), (0x10, 0x20, 0x30));
    }
    assert!(f.death2_cb.is_some() && f.take_dmg_cb.is_some() && f.pre_hitlag_cb.is_some() && f.post_hitlag_cb.is_some());
    assert!(f.accessory4_cb.is_none());

    // OnLand is a no-op for the wrong motion, calls the landing item for LandingAirN.
    f.motion_id = ftGw_MS_LandingAirN as FtMotionId;
    f.accessory4_cb = Some(t::cb_ftGw_AttackAirN_ItemParachuteOnLand);
    t::ftGw_AttackAirN_ItemParachuteOnLand(gp);
    assert_eq!(take(), ["parachute -hitlag", "turtle -hitlag", "sparky -hitlag", "parachute land"].map(String::from));
    assert!(f.accessory4_cb.is_none());
    f.motion_id = ftGw_MS_AttackAirN as FtMotionId;
    t::ftGw_AttackAirN_ItemTurtleOnLand(gp);
    t::ftGw_AttackAirN_ItemSparkyOnLand(gp);
    assert_eq!(take().iter().filter(|s| s.ends_with("land")).count(), 0);

    // Hitlag.
    t::ftGw_AttackAirN_EnterItemHitlag(gp);
    assert_eq!(take(), ["parachute +hitlag", "turtle +hitlag", "sparky +hitlag"].map(String::from));

    // Remove: item remove call, hitlag exit for everything still out, then clear.
    t::ftGw_AttackAirN_ItemTurtleRemove(gp);
    assert_eq!(take(), ["turtle remove", "parachute -hitlag", "turtle -hitlag", "sparky -hitlag"].map(String::from));
    assert!(unsafe { f.u.gw.x225C_turtleGObj }.is_null() && f.death2_cb.is_none() && f.take_dmg_cb.is_none());
    t::ftGw_AttackAirN_ItemTurtleRemove(gp);
    assert!(take().is_empty());

    // Motion ranges.
    f.motion_id = ftGw_MS_AttackAirN as FtMotionId; // the ranges overlap: N 347..350, B 348..351, Hi 349..352
    assert!(!t::ftGw_AttackAirN_ItemCheckParachuteRemove(gp));
    assert!(t::ftGw_AttackAirN_ItemCheckTurtleRemove(gp));
    assert!(t::ftGw_AttackAirN_ItemCheckSparkyRemove(gp));
    f.motion_id = ftGw_MS_LandingAirHi as FtMotionId;
    assert!(t::ftGw_AttackAirN_ItemCheckParachuteRemove(gp));
    assert!(t::ftGw_AttackAirN_ItemCheckTurtleRemove(gp));
    assert!(!t::ftGw_AttackAirN_ItemCheckSparkyRemove(gp));

    // DecideAction: C-stick motion picks the aerial.
    f.accessory4_cb = None;
    for (stick, expect) in [
        (ftCo_MS_AttackAirN, format!("enter {}", ftGw_MS_AttackAirN)),
        (ftCo_MS_AttackAirF, format!("enter {}", ftCo_MS_AttackAirF)),
        (ftCo_MS_AttackAirB, format!("enter {}", ftGw_MS_AttackAirB)),
        (ftCo_MS_AttackAirHi, format!("enter {}", ftGw_MS_AttackAirHi)),
        (ftCo_MS_AttackAirLw, format!("enter {}", ftCo_MS_AttackAirLw)),
    ] {
        f.cmd_vars[3] = stick as u32;
        f.accessory4_cb = None;
        t::ftGw_AttackAirN_DecideAction(gp);
        assert_eq!(take(), [expect]);
    }
    f.cmd_vars[3] = 0;
    t::ftGw_AttackAirN_DecideAction(gp);
    assert!(take().is_empty());
    assert!(f.accessory4_cb.is_none());
    f.cmd_vars[3] = ftCo_MS_AttackAirB as u32;
    t::ftGw_AttackAirN_DecideAction(gp);
    assert!(f.accessory4_cb.is_some());
    take();

    // Pass-throughs.
    t::ftGw_AttackAirN_Anim(gp);
    t::ftGw_AttackAirN_IASA(gp);
    t::ftGw_AttackAirB_IASA(gp);
    t::ftGw_AttackAirHi_IASA(gp);
    t::ftGw_LandingAirN_Phys(gp);
    assert_eq!(take(), ["air anim", "n iasa", "b iasa", "hi iasa", "landing phys"].map(String::from));

    // Landing enter: script flag set -> landing lag state and the OnLand accessory.
    f.cmd_vars[0] = 1;
    f.co_attrs.landingairn_lag = 11.0;
    f.co_attrs.landingairb_lag = 22.0;
    unsafe {
        f.u.gw.x2258_parachuteGObj = 0x10 as *mut HSD_GObj;
        f.u.gw.x2260_sparkyGObj = core::ptr::null_mut();
    }
    t::ftGw_LandingAirN_Enter(gp);
    assert_eq!(take(), [format!("parachute -hitlag"), format!("landing air {} 11", ftGw_MS_LandingAirN)]);
    assert!(f.accessory4_cb.is_some());
    // Up air landing erroneously uses back air's lag.
    t::ftGw_LandingAirHi_Enter(gp);
    assert_eq!(take(), [format!("parachute -hitlag"), format!("landing air {} 22", ftGw_MS_LandingAirHi)]);
    t::ftGw_LandingAirB_Enter(gp);
    assert_eq!(take(), [format!("parachute -hitlag"), format!("landing air {} 22", ftGw_MS_LandingAirB)]);
    // No script flag: the basic landing, then the items go (OnDamage; here only the parachute is out).
    f.cmd_vars[0] = 0;
    t::ftGw_LandingAirN_Enter(gp);
    let l = take();
    assert_eq!(&l[..2], ["parachute -hitlag", "landing basic"]);
    assert!(l.contains(&"parachute remove".to_string()));

    // Landing anim / coll: when the landing state is left, the items are dropped.
    unsafe { f.u.gw.x2258_parachuteGObj = 0x10 as *mut HSD_GObj };
    f.motion_id = ftGw_MS_LandingAirN as FtMotionId;
    t::ftGw_LandingAirN_Anim(gp); // the hook sets motion_id = 0
    let l = take();
    assert_eq!(l[0], "landing anim");
    assert!(l.contains(&"parachute remove".to_string()));
    t::ftGw_LandingAirHi_Coll(gp); // motion_id 0: left the landing, so OnDamage ran (the manhole hook logs)
    assert_eq!(take(), ["landing coll", "manhole"]);

    // Coll passes the landing-enter callback.
    t::ftGw_AttackAirN_Coll(gp);
    assert_eq!(take(), ["coll cb true"]);
}
