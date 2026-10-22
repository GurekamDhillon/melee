//! tu::ftkirbyspecialgamewatch (src/melee/ft/kinds/ftKirby/ftkirbyspecialgamewatch.c): no fused sites,
//! no pure math. Smoke test of the Chef entries, sausage throw bookkeeping and the repeat logic
//! through recording hooks.
#![allow(static_mut_refs)]

use melee_chara::stubs as st;
use melee_chara::tu::ftkirbyspecialgamewatch as t;
use melee_types::*;

static mut LOG: std::vec::Vec<String> = std::vec::Vec::new();
fn log(s: String) {
    unsafe { LOG.push(s) }
}
fn take() -> String {
    unsafe { core::mem::take(&mut LOG).join(",") }
}
fn gw(f: &Fighter) -> ftGameWatch_SpecialNVars {
    unsafe { f.mv.gw.SpecialN }
}

#[test]
fn smoke() {
    let mut f: Box<Fighter> = Box::new(Fighter::zeroed());
    let mut g: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    let da: &'static mut ftKb_DatAttrs = Box::leak(Box::new(ftKb_DatAttrs::zeroed()));
    let mut parts: std::vec::Vec<FighterBone> = (0..40).map(|_| unsafe { core::mem::zeroed() }).collect();
    let mut item: Box<Item_GObj> = Box::new(unsafe { core::mem::zeroed() });
    f.dat_attrs = da as *mut _ as *mut _;
    f.parts = parts.as_mut_ptr();
    g.user_data = &mut *f as *mut Fighter as *mut _;
    let gp: *mut HSD_GObj = &mut *g;
    let ip: *mut Item_GObj = &mut *item;
    da.specialn_gw_max_sausages_per_use = 3.0;
    da.specialn_gw_frame_on_repeat = 10.0;
    unsafe {
        st::FIGHTER_CHANGEMOTIONSTATE = Some(|_, msid, flags, a, b, c, _| log(format!("cms {msid} {flags:x} {a} {b} {c}")));
        st::FTANIM_8006EBA4 = Some(|_| log("8006EBA4".into()));
        st::FTANIM_ISFRAMESREMAINING = Some(|_| 1);
        st::FT_8008A2BC = Some(|_| log("8A2BC".into()));
        st::FTCO_FALL_ENTER = Some(|_| log("fall".into()));
        t::ext::LB_8000B1CC = Some(|_, off, out| unsafe {
            if off.is_null() {
                log("lb null".into());
            } else {
                log(format!("lb {} {} {}", (*off).x, (*off).y, (*off).z));
            }
            *out = Vec3 { x: 5.0, y: 6.0, z: 7.0 };
        });
        t::ext::HSD_RANDI = Some(|n| {
            log(format!("randi {n}"));
            n - 1
        });
        t::ext::IT_802C837C = Some(|_, pos, kind, sausage, dir| log(format!("chef {} {kind} {sausage} {dir}", unsafe { (*pos).x })));
        t::ext::IT_802C74D8 = Some(|_, _, bone, dir| {
            log(format!("chefitem {bone} {dir}"));
            core::ptr::null_mut::<Item_GObj>().wrapping_add(0x1000)
        });
        t::ext::IT_802C760C = Some(|_| log("760C".into()));
        t::ext::IT_802C7658 = Some(|_| log("7658".into()));
        t::ext::IT_802C7678 = Some(|_| log("7678".into()));
    }
    // ftPartsTable[kind]->part_to_joint[FtPart_LThumbNb] = 5.
    let mut p2j = [0u8; 128];
    p2j[FtPart_LThumbNb as usize] = 5;
    let mut table = FighterPartsTable { joint_to_part: core::ptr::null_mut(), part_to_joint: p2j.as_mut_ptr(), parts_num: 40 };
    let mut tables: [*mut FighterPartsTable; 8] = [&mut table; 8];
    unsafe { melee_fighter::hooks::FT_PARTS_TABLE = tables.as_mut_ptr() };
    let mut jobj5: Box<HSD_JObj> = Box::new(unsafe { core::mem::zeroed() });
    parts[5].joint = &mut *jobj5;
    f.kind = Ft_Kind_Kirby;

    // Entry: motion + script variables cleared + Chef callback armed.
    f.cmd_vars = [5, 5, 5, 5];
    f.self_vel.y = 3.0;
    f.facing_dir = 1.0;
    t::ftKb_SpecialNGw_8010D188(gp);
    assert_eq!(take(), format!("cms {} 0 0 1 0,8006EBA4", ftKb_MS_GwSpecialN));
    assert_eq!((f.cmd_vars[0], f.cmd_vars[1], f.cmd_vars[2], f.cmd_vars[3]), (0, 0, 0, 5));
    assert_eq!((gw(&f).isChefLoopDisable, gw(&f).maxSausage), (0, 0));
    assert_eq!(f.self_vel.y, 0.0);
    // The armed callback (fn_8010CFB0) spawns the chef item at the thumb bone and arms the hitlag pair.
    unsafe { (f.accessory4_cb.unwrap())(gp) };
    assert_eq!(take(), "lb null,chefitem 5 1");
    assert!(unsafe { f.u.kb.xDC }.is_null() == false && f.death2_cb.is_some() && f.take_dmg_cb.is_some());
    assert!(f.pre_hitlag_cb.is_some() && f.post_hitlag_cb.is_some());
    unsafe { (f.pre_hitlag_cb.unwrap())(gp) };
    unsafe { (f.post_hitlag_cb.unwrap())(gp) };
    assert_eq!(take(), "7658,7678");

    // Sausage throw (fn_8010CE5C): slot chosen among {0..4} minus the last two (1, 3) via HSD_Randi.
    unsafe {
        f.u.kb.xD4 = 1;
        f.u.kb.xD8 = 3;
    }
    // fn_8010CFB0 left fn_8010CE5C armed in accessory4_cb.
    f.cmd_vars[0] = 1;
    unsafe { (f.accessory4_cb.unwrap())(gp) };
    assert_eq!(take(), format!("lb 2.5 6.5 0,randi 3,chef 5 {} 4 1", It_Kind_Kirby_GameWatchChef));
    assert_eq!(gw(&f).maxSausage, 1);
    assert_eq!((unsafe { f.u.kb.xD4 }, unsafe { f.u.kb.xD8 }), (4, 1));
    assert!(f.accessory4_cb.is_none() && f.cmd_vars[0] == 0);

    // Anim: cmd_vars[2] pulse restarts the swing at frame_on_repeat - 1 while under the cap.
    f.cmd_vars[2] = 1;
    t::ftKb_GwSpecialN_Anim(gp);
    assert_eq!(take(), format!("cms {} c4c508c 9 1 0,8006EBA4", ftKb_MS_GwSpecialN));
    assert!(f.accessory4_cb.is_some());
    // ... not once the Chef loop is disabled or the cap is reached.
    f.cmd_vars[2] = 1;
    unsafe { f.mv.gw.SpecialN.isChefLoopDisable = 1 };
    t::ftKb_GwSpecialAirN_Anim(gp);
    assert_eq!(take(), "");
    // IASA: releasing B disables the loop; B press with cmd_vars[1] under the cap restarts (air state id).
    f.input.held_buttons[0] = 0;
    f.input.pressed_buttons = HSD_PAD_B as u32;
    f.cmd_vars[1] = 1;
    unsafe { f.mv.gw.SpecialN.isChefLoopDisable = 0 };
    t::ftKb_GwSpecialAirN_IASA(gp);
    assert_eq!(gw(&f).isChefLoopDisable, 0); // reset by the restart (inline1)
    assert_eq!(take(), format!("cms {} c4c508c 9 1 0,8006EBA4", ftKb_MS_GwSpecialAirN));
    unsafe { f.mv.gw.SpecialN.maxSausage = 3 };
    f.cmd_vars[1] = 1;
    t::ftKb_GwSpecialN_IASA(gp);
    assert_eq!((gw(&f).isChefLoopDisable, take().as_str()), (1, ""));

    // Cleanup: ability end releases the chef item and forgets it.
    t::ftKb_SpecialNGw_8010D0A8(gp);
    assert_eq!(take(), "760C,7678");
    assert!(unsafe { f.u.kb.xDC }.is_null());
    t::ftKb_SpecialNGw_8010D0A8(gp);
    assert_eq!(take(), "");
    drop(parts);
    let _ = (ip, &mut item);
}
