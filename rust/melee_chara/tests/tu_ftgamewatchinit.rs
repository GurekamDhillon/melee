//! tu::ftgamewatch / ftganon / ftgkoopa smoke test: the init callbacks (OnLoad attribute push and
//! item registration, OnDamage removing every item, the costume colour lookups, Ganondorf's and
//! Giga Bowser's forwarding) through recording hooks. One #[test]: hooks are globals.
#![allow(static_mut_refs, unused_unsafe)]
mod gw_common;
use gw_common::*;
use melee_chara::tu::{ftganon, ftgamewatch as t, ftgkoopa};
use melee_types::*;

#[test]
fn init_callbacks() {
    install_common();
    unsafe {
        t::ext::FTMATERIAL_800BFB4C = Some(|_, c| unsafe { log(format!("material {} {} {} {}", (*c).r, (*c).g, (*c).b, (*c).a)) });
        t::ext::IT_8026B3F8 = Some(|a, kind| log(format!("it {:x} {kind}", a as usize)));
        // the removals reached through OnDamage
        use melee_chara::tu::{
            ftgamewatchattack11 as a11, ftgamewatchattackair as aair, ftgamewatchattacklw3 as alw3,
            ftgamewatchattacks4 as as4, ftgamewatchspecialhi as shi, ftgamewatchspeciallw as slw,
            ftgamewatchspecials as ss,
        };
        a11::ext::ITGAMEWATCHGREENHOUSE_802C6328 = Some(|_| log("greenhouse"));
        a11::ext::ITGAMEWATCHGREENHOUSE_802C6394 = Some(|_| {});
        alw3::ext::IT_802C6718 = Some(|_| log("manhole"));
        alw3::ext::IT_802C6784 = Some(|_| {});
        as4::ext::ITGAMEWATCHFIRE_802C6A2C = Some(|_| log("fire"));
        as4::ext::ITGAMEWATCHFIRE_802C6A98 = Some(|_| {});
        aair::ext::IT_802C6D6C = Some(|_| log("parachute"));
        aair::ext::IT_802C7074 = Some(|_| log("turtle"));
        aair::ext::IT_802C7340 = Some(|_| log("sparky"));
        aair::ext::IT_802C6DD8 = Some(|_| {});
        aair::ext::IT_802C70E0 = Some(|_| {});
        aair::ext::IT_802C73AC = Some(|_| {});
        ss::ext::IT_802C7A84 = Some(|_| log("judgement"));
        ss::ext::IT_802C7AF0 = Some(|_| {});
        slw::ext::IT_802C7E94 = Some(|_| log("panic"));
        slw::ext::IT_802C7F00 = Some(|_| {});
        shi::ext::IT_802C8158 = Some(|_| log("rescue"));
    }
    let mut w = World::new();
    let gp = w.gp();

    // OnLoad: PUSH_ATTRS (copy the file's block into the backup and point dat_attrs at it), the
    // width / outline / costume colour, and the ten items registered by kind.
    let mut src = ftGameWatchAttributes::zeroed();
    src.x0_GAMEWATCH_WIDTH = 1.5;
    src.x14_GAMEWATCH_OUTLINE = _GXColor { r: 9, g: 8, b: 7, a: 6 };
    src.x4_GAMEWATCH_COLOR[2] = _GXColor { r: 1, g: 2, b: 3, a: 4 };
    let backup: &'static mut ftGameWatchAttributes = Box::leak(Box::new(ftGameWatchAttributes::zeroed()));
    let mut items: std::vec::Vec<*mut core::ffi::c_void> = (0..11).map(|i| (0x1000 + i * 0x10) as *mut core::ffi::c_void).collect();
    let mut fd: Box<ftData> = Box::new(unsafe { core::mem::zeroed() });
    fd.ext_attr = &mut src as *mut _ as *mut _;
    fd.x48_items = (items.as_mut_ptr()) as *mut _;
    w.f.ft_data = &mut *fd;
    w.f.dat_attrs_backup = backup as *mut _ as *mut _;
    w.f.costume_id = 2;
    w.f.kind = Ft_Kind_GameWatch;
    w.f.set_x2222_b6(1);
    t::ftGw_Init_OnLoad(gp);
    assert_eq!(w.f.dat_attrs, backup as *mut _ as *mut core::ffi::c_void);
    assert_eq!(backup.x0_GAMEWATCH_WIDTH, 1.5);
    assert_eq!(w.f.x34_scale.z, 1.5);
    assert_eq!((w.f.x2222_b6(), unsafe { w.f.u.gw.x2238_panicCharge }), (0, 0));
    let c = w.f.x610_color_rgba[1];
    assert_eq!((c.r, c.g, c.b, c.a), (9, 8, 7, 6));
    assert_eq!(w.f.x5AC.xC[4] as usize, 0x1000 + 10 * 0x10);
    let kinds = [
        It_Kind_GameWatch_Greenhouse,
        It_Kind_GameWatch_Manhole,
        It_Kind_GameWatch_Fire,
        It_Kind_GameWatch_Parachute,
        It_Kind_GameWatch_Turtle,
        It_Kind_GameWatch_Breath,
        It_Kind_GameWatch_Judge,
        It_Kind_GameWatch_Panic,
        It_Kind_GameWatch_Chef,
        It_Kind_GameWatch_Rescue,
    ];
    let mut want = vec!["material 1 2 3 4".to_string()];
    for (i, k) in kinds.iter().enumerate() {
        want.push(format!("it {:x} {k}", 0x1000 + i * 0x10));
    }
    assert_eq!(take(), want.join(","));

    // Costume colour lookups for the item mods.
    let mut m = ItemModStruct::zeroed();
    t::ftGw_Init_8014A7F4(gp, &mut m);
    assert_eq!((m.x0_unk.r, m.x0_unk.a), (1, 4));
    t::ftGw_Init_8014A814(gp, &mut m);
    assert_eq!((m.x0_unk.r, m.x0_unk.a), (9, 6));
    // an installed Mex_CostumeVisIdx maps the costume (Akaneia's extra costumes -> 0..3)
    unsafe { t::ext::MEX_COSTUMEVISIDX = Some(|_, c| c % 2) };
    t::ftGw_Init_8014A7F4(gp, &mut m);
    assert_eq!(m.x0_unk.r, 0);
    unsafe { t::ext::MEX_COSTUMEVISIDX = None };

    // OnDamage removes every item once; 8014A538 only in the air.
    unsafe {
        w.f.u.gw.x224C_greenhouseGObj = fake(1);
        w.f.u.gw.x2250_manholeGObj2 = fake(2);
        w.f.u.gw.x2254_fireGObj = fake(3);
        w.f.u.gw.x2258_parachuteGObj = fake(4);
        w.f.u.gw.x225C_turtleGObj = fake(5);
        w.f.u.gw.x2260_sparkyGObj = fake(6);
        w.f.u.gw.x2264_judgementGObj = fake(7);
        w.f.u.gw.x2268_panicGObj = fake(8);
        w.f.u.gw.x226C_rescueGObj = fake(9);
    }
    w.f.ground_or_air = GA_Ground;
    t::ftGw_Init_8014A538(gp);
    assert_eq!(take(), "");
    t::ftGw_Init_OnDamage(gp);
    assert_eq!(take(), "greenhouse,manhole,fire,parachute,turtle,sparky,judgement,panic,rescue");
    t::ftGw_Init_OnDamage(gp);
    assert_eq!(take(), "");


    // Ganondorf shares Captain Falcon's attribute block.
    let mut cs = ftCaptain_DatAttrs::zeroed();
    cs.specialn_vel_x = 4.25;
    let cbk: &'static mut ftCaptain_DatAttrs = Box::leak(Box::new(ftCaptain_DatAttrs::zeroed()));
    fd.ext_attr = &mut cs as *mut _ as *mut _;
    w.f.dat_attrs_backup = cbk as *mut _ as *mut _;
    ftganon::ftGn_Init_OnLoad(gp);
    assert_eq!(cbk.specialn_vel_x, 4.25);
    assert_eq!(w.f.dat_attrs, cbk as *mut _ as *mut core::ffi::c_void);
    cs.specialn_vel_x = 5.5;
    ftganon::ftGn_Init_LoadSpecialAttrs(gp);
    assert_eq!(cbk.specialn_vel_x, 5.5);

    // Giga Bowser forwards to ftKoopa's init and registers the flame.
    unsafe {
        ftgkoopa::ext::FTKP_INIT_ONLOADFORGKOOPA = Some(|_| log("kp_onload"));
        ftgkoopa::ext::IT_8026B3F8 = Some(|a, k| log(format!("it {:x} {k}", a as usize)));
        ftgkoopa::ext::FTKP_INIT_UNKMOTIONSTATES3 = Some(|_| log("kp_unk3"));
        ftgkoopa::ext::FTKP_INIT_LOADSPECIALATTRS = Some(|_| log("kp_attrs"));
    }
    ftgkoopa::ftGk_Init_OnLoad(gp);
    ftgkoopa::ftGk_Init_UnkMotionStates3(gp);
    ftgkoopa::ftGk_Init_LoadSpecialAttrs(gp);
    assert_eq!(take(), format!("kp_onload,it 1000 {},kp_unk3,kp_attrs", It_Kind_Koopa_Flame));
    assert_eq!(ftgkoopa::ftGk_Init_GetMotionFileString(15), Some("ftDemoVi1201V2MotionFileGkoopa"));
    assert_eq!(ftgkoopa::ftGk_Init_GetMotionFileString(3), None);
    let (mut a1, mut a2) = (0, 0);
    ftgkoopa::ftGk_Init_UnkDemoCallbacks0(15, &mut a1, &mut a2);
    assert_eq!((a1, a2), (14, 14));
}
