//! Smoke test for tu::ftmarsspecials (no fused sites) through recording hooks.
#![allow(static_mut_refs)]

use melee_chara::stubs as st;
use melee_chara::tu::ftmarsspecials as t;
use melee_types::*;

static mut LOG: std::vec::Vec<String> = std::vec::Vec::new();
fn log(s: String) {
    unsafe { LOG.push(s) }
}
fn take() -> std::vec::Vec<String> {
    unsafe { core::mem::take(&mut LOG) }
}

#[test]
fn smoke() {
    let mut f: Box<Fighter> = Box::new(Fighter::zeroed());
    let mut g: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    let da: &'static mut MarsAttributes = Box::leak(Box::new(MarsAttributes::zeroed()));
    let cd: &'static mut ftCommonData = Box::leak(Box::new(unsafe { core::mem::zeroed() }));
    cd.x21C = 0.5;
    f.dat_attrs = da as *mut _ as *mut _;
    g.user_data = &mut *f as *mut Fighter as *mut _;
    let gp: *mut HSD_GObj = &mut *g;
    unsafe {
        melee_fighter::hooks::P_FT_COMMON_DATA = cd;
        st::FIGHTER_CHANGEMOTIONSTATE = Some(|_, ms, fl, s, sp, _, _| log(format!("ms {ms} {fl:x} {s} {sp}")));
        st::FTANIM_8006EBA4 = Some(|_| log("anim".into()));
        st::FT_800827A0 = Some(|_| 0);
        st::FT_80081D0C = Some(|_| 1);
        st::FTCOMMON_8007D7FC = Some(|_| log("7D7FC".into()));
        st::FT_80084F3C = Some(|_| log("84F3C".into()));
        t::ext::FT_800892A0 = Some(|_| log("892A0".into()));
    }

    // Enters.
    f.ground_or_air = GA_Ground;
    f.self_vel.y = 5.0;
    f.cmd_vars = [3, 3, 3, 3];
    t::ftMs_SpecialS_Enter(gp);
    assert_eq!(take(), ["ms 349 0 0 1", "anim"]);
    assert_eq!((f.self_vel.y, f.cmd_vars[..2].to_vec(), unsafe { f.mv.ms.specials.x0 }), (0.0, vec![0, 0], 0));
    da.x14 = 4.0;
    da.x1C = 1.25;
    f.ground_or_air = GA_Air;
    f.self_vel.x = 8.0;
    unsafe { f.u.ms.x222C = 0 };
    t::ftMs_SpecialAirS_Enter(gp);
    assert_eq!((f.self_vel.x, f.self_vel.y, unsafe { f.u.ms.x222C }), (2.0, 1.25, 1));
    assert_eq!(take(), ["ms 358 0 0 1", "anim"]);
    t::ftMs_SpecialAirS_Enter(gp);
    assert_eq!(f.self_vel.y, 0.0);
    take();

    // IASA: buffers a press before the window, chains once open.
    f.cmd_vars = [0, 0, 0, 0];
    f.input.pressed_buttons = 0x100;
    t::ftMs_SpecialAirS1_IASA(gp);
    assert_eq!(f.cmd_vars[1], 1);
    f.cmd_vars = [1, 0, 0, 0];
    f.input.lstick[0].y = 0.9;
    f.ground_or_air = GA_Ground;
    t::ftMs_SpecialAirS1_IASA(gp);
    assert_eq!(take(), ["ms 350 2000000 0 1"]);
    assert!(f.x21EC.is_some());
    f.cmd_vars = [1, 0, 0, 0];
    f.input.lstick[0].y = 0.1;
    f.ground_or_air = GA_Air;
    t::ftMs_SpecialAirS1_IASA(gp);
    assert_eq!(take(), ["ms 360 2000000 0 1"]);
    // Stage 3 chain with the three stick regions.
    for (y, ground, want) in [(0.9f32, true, 355), (-0.9, true, 357), (0.0, true, 356), (0.9, false, 364), (-0.9, false, 366), (0.0, false, 365)] {
        f.cmd_vars = [1, 0, 0, 0];
        f.input.lstick[0].y = y;
        f.ground_or_air = if ground { GA_Ground } else { GA_Air };
        t::ftMs_SpecialS3_IASA(gp);
        assert_eq!(take(), [format!("ms {want} 2000000 0 1")]);
    }
    f.input.pressed_buttons = 0;

    // Phys 1/2 (air) uses fall + deaccel; ground uses ft_80084F3C.
    f.ground_or_air = GA_Ground;
    t::ftMs_SpecialS2_Phys(gp);
    assert_eq!(take(), ["84F3C"]);
    f.ground_or_air = GA_Air;
    da.x20 = 0.2;
    da.x24 = 3.0;
    f.self_vel = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    t::ftMs_SpecialAirS1_Phys(gp);
    assert_eq!(f.self_vel.y, -0.2);

    // Ground <-> air transitions pick the paired state by motion id.
    const TF: &str = "c4e508c";
    f.cur_anim_frame = 7.0;
    f.motion_id = 351;
    t::ftMs_SpecialS_80137940(gp);
    assert_eq!(take(), [format!("ms 360 {TF} 7 1")]);
    f.motion_id = 360;
    unsafe { f.u.ms.x222C = 9 };
    t::ftMs_SpecialS_801379D0(gp);
    assert_eq!(unsafe { f.u.ms.x222C }, 0);
    assert_eq!(take(), ["7D7FC".to_string(), format!("ms 351 {TF} 7 1")]);
    for (from, to) in [(352, 361), (354, 363), (353, 362)] {
        f.motion_id = from;
        t::ftMs_SpecialS_80137CBC(gp);
        assert_eq!(take(), [format!("ms {to} {TF} 7 1")]);
        f.motion_id = to;
        t::ftMs_SpecialS_80137D60(gp);
        assert_eq!(take(), ["7D7FC".to_string(), format!("ms {from} {TF} 7 1")]);
    }
    for (from, to) in [(355, 364), (357, 366), (356, 365)] {
        f.motion_id = from;
        t::ftMs_SpecialS_80137FF8(gp);
        assert_eq!(take(), [format!("ms {to} {TF} 7 1")]);
        f.motion_id = to;
        t::ftMs_SpecialS_8013809C(gp);
        assert_eq!(take(), ["7D7FC".to_string(), format!("ms {from} {TF} 7 1")]);
    }

    // Coll dispatch.
    f.ground_or_air = GA_Ground;
    f.motion_id = 355;
    t::ftMs_SpecialS4_Coll(gp);
    assert_eq!(take().len(), 1);
    // The attack-count reset callback.
    t::ftMs_SpecialS_80137A68(gp);
}
