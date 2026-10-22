//! Smoke test for tu::ftgamewatch (stateful, no float arithmetic) through recording hooks.
//! One #[test]: the hooks are process globals.
#![allow(static_mut_refs)]

use melee_chara::tu::ftgamewatch as t;
use melee_chara::tu::ftgamewatchattack11 as a11;
use melee_chara::tu::ftgamewatchattackair as air;
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
    let fd: &'static mut ftData = Box::leak(Box::new(unsafe { core::mem::zeroed() }));
    let ea: &'static mut ftGameWatchAttributes = Box::leak(Box::new(ftGameWatchAttributes::zeroed()));
    let backup: &'static mut ftGameWatchAttributes = Box::leak(Box::new(ftGameWatchAttributes::zeroed()));
    let mut items: [*mut core::ffi::c_void; 11] = [core::ptr::null_mut(); 11];
    for (i, it) in items.iter_mut().enumerate() {
        *it = (0x1000 + i * 0x10) as *mut core::ffi::c_void;
    }
    let items: &'static mut [*mut core::ffi::c_void; 11] = Box::leak(Box::new(items));
    fd.ext_attr = ea as *mut _ as *mut _;
    fd.x48_items = items.as_mut_ptr();
    f.ft_data = fd;
    f.dat_attrs_backup = backup as *mut _ as *mut _;
    f.kind = Ft_Kind_GameWatch;
    f.x619_costume_id = 2;
    g.user_data = &mut *f as *mut Fighter as *mut _;
    let gp: *mut HSD_GObj = &mut *g;

    ea.x0_GAMEWATCH_WIDTH = 1.5;
    ea.x4_GAMEWATCH_COLOR[2] = _GXColor { r: 1, g: 2, b: 3, a: 4 };
    ea.x14_GAMEWATCH_OUTLINE = _GXColor { r: 9, g: 8, b: 7, a: 6 };

    unsafe {
        t::ext::IT_8026B3F8 = Some(|a, kind| log(&format!("it {:x} {kind}", a as usize)));
        t::ext::FTMATERIAL_800BFB4C = Some(|_, c| log(&format!("mat {}", (*c).r)));
        t::ext::FTCO_800BFFD0 = Some(|_, a, b| {
            log(&format!("overlay {a} {b}"));
            true
        });
        t::ext::FTGW_ATTACKLW3_ITEMMANHOLEONDAMAGE = Some(|_| log("manhole"));
        t::ext::FTGW_ATTACKS4_ITEMTORCHONDAMAGE = Some(|_| log("torch"));
        t::ext::FTGW_SPECIALS_ITEMJUDGEMENTREMOVE = Some(|_| log("judge"));
        t::ext::FTGW_SPECIALLW_ITEMPANICREMOVE = Some(|_| log("panic"));
        t::ext::FTGW_SPECIALHI_ITEMRESCUEREMOVE = Some(|_| log("rescue"));
        t::ext::FTGW_SPECIALLW_ABSORBTHINK_DECIDEACTION = Some(|_| log("absorb"));
        a11::ext::ITGAMEWATCHGREENHOUSE_802C6328 = Some(|_| log("greenhouse remove"));
        a11::ext::ITGAMEWATCHGREENHOUSE_802C6394 = Some(|_| log("greenhouse exit hitlag"));
        air::ext::IT_802C6D6C = Some(|_| log("parachute remove"));
        air::ext::IT_802C6DD8 = Some(|_| log("parachute exit hitlag"));
        air::ext::IT_802C7074 = Some(|_| log("turtle remove"));
        air::ext::IT_802C70E0 = Some(|_| log("turtle exit hitlag"));
        air::ext::IT_802C7340 = Some(|_| log("sparky remove"));
        air::ext::IT_802C73AC = Some(|_| log("sparky exit hitlag"));
    }

    // OnLoad: PUSH_ATTRS copies the DAT block, items get their kinds in order.
    t::ftGw_Init_OnLoad(gp);
    assert_eq!(f.dat_attrs, f.dat_attrs_backup);
    assert_eq!(f.x34_scale.z, 1.5);
    assert_eq!(f.x610_color_rgba[1].r, 9);
    assert_eq!(unsafe { f.u.gw.x2238_panicCharge }, ftGw_Panic_Empty as i32);
    assert_eq!(f.x5AC.xC[4] as usize, 0x1000 + 10 * 0x10);
    let l = take();
    assert_eq!(l[0], "mat 1");
    assert_eq!(l[1], format!("it 1000 {It_Kind_GameWatch_Greenhouse}"));
    assert_eq!(l[10], format!("it 1090 {It_Kind_GameWatch_Rescue}"));
    assert_eq!(l.len(), 11);

    // OnDamage removes every item, in C order; here three are out.
    f.u.gw.x224C_greenhouseGObj = 0x10 as *mut HSD_GObj;
    f.u.gw.x2258_parachuteGObj = 0x20 as *mut HSD_GObj;
    f.u.gw.x2260_sparkyGObj = 0x30 as *mut HSD_GObj;
    f.death2_cb = Some(t::cb_ftGw_Init_OnDamage);
    t::ftGw_Init_OnDamage(gp);
    assert_eq!(
        take(),
        [
            "greenhouse remove",
            "greenhouse exit hitlag",
            "manhole",
            "torch",
            "parachute remove",
            "parachute exit hitlag",
            "sparky exit hitlag", // ExitItemHitlag releases every item still out
            "sparky remove",
            "sparky exit hitlag",
            "judge",
            "panic",
            "rescue"
        ]
    );
    assert!(unsafe { f.u.gw.x224C_greenhouseGObj.is_null() && f.u.gw.x2258_parachuteGObj.is_null() });
    assert!(f.death2_cb.is_none() && f.take_dmg_cb.is_none());

    // 8014A538 only while airborne.
    f.ground_or_air = GA_Ground;
    t::ftGw_Init_8014A538(gp);
    assert!(take().is_empty());
    f.ground_or_air = GA_Air;
    t::ftGw_Init_8014A538(gp);
    assert_eq!(take().len(), 5);

    // Oil Panic overlay only at full charge.
    f.u.gw.x2238_panicCharge = ftGw_Panic_Mid as i32;
    t::ftGw_Init_UnkMotionStates4(gp);
    assert!(take().is_empty());
    f.u.gw.x2238_panicCharge = ftGw_Panic_Full as i32;
    t::ftGw_Init_UnkMotionStates4(gp);
    assert_eq!(take(), ["overlay 5 0"]);

    // Item colours come from the costume's table entry.
    f.dat_attrs = ea as *mut _ as *mut _;
    let mut m: ItemModStruct = ItemModStruct::zeroed();
    t::ftGw_Init_8014A7F4(gp, &mut m);
    assert_eq!((m.x0_unk.r, m.x0_unk.a), (1, 4));
    t::ftGw_Init_8014A814(gp, &mut m);
    assert_eq!((m.x0_unk.r, m.x0_unk.a), (9, 6));

    // COPY_ATTRS and the absorb hook.
    ea.x0_GAMEWATCH_WIDTH = 2.5;
    f.dat_attrs = backup as *mut _ as *mut _;
    t::ftGw_Init_LoadSpecialAttrs(gp);
    assert_eq!(backup.x0_GAMEWATCH_WIDTH, 2.5);
    t::ftGw_Init_OnAbsorb(gp);
    assert_eq!(take(), ["absorb"]);

    assert_eq!(t::ftGw_Init_CostumeStrings[3], ["PlGwNr.dat", "PlyGamewatch5K_Share_joint"]);
}
