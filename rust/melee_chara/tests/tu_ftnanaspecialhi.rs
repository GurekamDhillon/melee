//! Smoke test of tu::ftnanaspecialhi: Nana's Belay follow. The three fused sites
//! (`ftNn_Init_801237F8` x2, `ftPp_SpecialHi_0_Phys`) are checked against an independent
//! double-precision fused multiply-add; the rest runs through recording hooks.
#![allow(static_mut_refs)]

use melee_chara::stubs as st;
use melee_chara::tu::ftnanaspecialhi as t;
use melee_chara::tu::ftnanaspecials as sp;
use melee_runtime::gekko::{cosf, sinf};
use melee_types::*;

static mut LOG: std::vec::Vec<String> = std::vec::Vec::new();
fn log(s: String) {
    unsafe { LOG.push(s) }
}
fn take() -> String {
    unsafe { std::mem::take(&mut LOG).join(",") }
}
static mut POPO: *mut HSD_GObj = core::ptr::null_mut();
static mut FRAMES_LEFT: i32 = 0;

fn fma32(a: f32, b: f32, c: f32) -> f32 {
    ((a as f64) * (b as f64) + (c as f64)) as f32
}

#[test]
fn smoke() {
    let mut attrs: Box<ftIceClimberAttributes> = Box::new(unsafe { core::mem::zeroed() });
    let mut nana: Box<Fighter> = Box::new(Fighter::zeroed());
    let mut popo: Box<Fighter> = Box::new(Fighter::zeroed());
    nana.dat_attrs = &mut *attrs as *mut _ as *mut _;
    popo.dat_attrs = &mut *attrs as *mut _ as *mut _;
    let mut parts: std::vec::Vec<FighterBone> = (0..64).map(|_| unsafe { core::mem::zeroed() }).collect();
    nana.parts = parts.as_mut_ptr();
    popo.parts = parts.as_mut_ptr();
    let mut ng: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    let mut pg: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    ng.user_data = &mut *nana as *mut Fighter as *mut _;
    pg.user_data = &mut *popo as *mut Fighter as *mut _;
    let np: *mut HSD_GObj = &mut *ng;
    unsafe { POPO = &mut *pg };
    unsafe {
        sp::ext::PLAYER_GETENTITYATINDEX = Some(|_, _| POPO);
        sp::ext::FT_800849EC = Some(|_, _| log("849ec".into()));
        sp::ext::FTPARTSETROTY = Some(|_, part, v| log(format!("roty {part} {v}")));
        melee_chara::tu::ftmewtwospecialhi::ext::LB_8000B1CC = Some(|joint, _, out| {
            // Distinguish the two joints by their address: parts[R4thNb] is Popo's hand.
            let _ = joint;
            *out = Vec3 { x: 1.0, y: 2.0, z: 3.0 };
        });
        st::FTANIM_ISFRAMESREMAINING = Some(|_| FRAMES_LEFT);
        st::FTANIM_SETANIMRATE = Some(|_, r| log(format!("rate {r}")));
        st::FIGHTER_CHANGEMOTIONSTATE = Some(|_, ms, fl, _, _, _, _| log(format!("M{ms} {fl:#x}")));
        st::FTCO_FALL_ENTER = Some(|_| log("fall".into()));
        st::FT_8008A2BC = Some(|_| log("wait".into()));
        st::FT_80084F3C = Some(|_| log("84f3c".into()));
        st::FT_80081D0C = Some(|_| GA_Ground as i32);
        st::FTCO_80096900 = Some(|_, a, b, c, m, l| log(format!("96900 {a} {b} {c} {m} {l}")));
    }

    // Anim: only when the animation ran out.
    unsafe { FRAMES_LEFT = 1 };
    t::ftPp_SpecialHi_1_Anim(np);
    assert_eq!(take(), "");
    unsafe { FRAMES_LEFT = 0 };
    t::ftPp_SpecialHi_1_Anim(np);
    assert_eq!(take(), "wait");
    attrs.x130 = 0.5;
    attrs.x134 = 12.0;
    t::ftPp_SpecialHi_4_Anim(np);
    assert_eq!(take(), "96900 0 1 0 0.5 12");
    t::ftPp_SpecialHi_1_Phys(np);
    assert_eq!(take(), "84f3c");

    // 801237F8: launch velocity from x13C / x140, position nudge fused with the scale.
    attrs.x13C = 2.5;
    attrs.x140 = 0.6;
    nana.facing_dir = -1.0;
    nana.x34_scale.y = 1.1;
    nana.cur_pos = Vec3 { x: 10.0, y: 20.0, z: 0.0 };
    t::ftNn_Init_801237F8(np);
    assert_eq!(take(), "849ec,M365 0x0");
    assert_eq!(nana.self_vel.x.to_bits(), (-1.0f32 * (2.5f32 * cosf(0.6))).to_bits());
    assert_eq!(nana.self_vel.y.to_bits(), (2.5f32 * sinf(0.6)).to_bits());
    assert_eq!(nana.cur_pos.x.to_bits(), fma32(4.0 * -1.0, 1.1, 10.0).to_bits());
    assert_eq!(nana.cur_pos.y.to_bits(), fma32(7.0, 1.1, 20.0).to_bits());
    assert!(nana.accessory4_cb.is_some());

    // Phys_0 while Popo is not throwing (motion outside 347..=352): fused `-(8*facing - x)`.
    popo.motion_id = 100;
    for (facing, x) in [(1.0f32, 3.3f32), (-1.0, 3.3), (1.0, -123.456)] {
        nana.facing_dir = facing;
        nana.cur_pos.x = x;
        t::ftPp_SpecialHi_0_Phys(np);
        assert_eq!(take(), "fall");
        assert_eq!(nana.cur_pos.x.to_bits(), (-fma32(8.0, facing, -x)).to_bits());
    }

    // Phys_0 while Popo throws: glued to his hand (1,2,3) minus her own offset (1,2,3), anim rate
    // follows Popo's.
    popo.motion_id = 350;
    popo.facing_dir = nana.facing_dir;
    popo.frame_speed_mul = 2.0;
    nana.frame_speed_mul = 1.0;
    nana.cur_pos = Vec3 { x: 5.0, y: 6.0, z: 7.0 };
    t::ftPp_SpecialHi_0_Phys(np);
    assert_eq!(take(), "rate 2");
    assert_eq!((nana.cur_pos.x, nana.cur_pos.y, nana.cur_pos.z), (5.0, 6.0, 7.0));
    popo.set_x2219_b5(1);
    t::ftPp_SpecialHi_0_Phys(np);
    assert_eq!(take(), "rate 0");

    // fn_80123218 publishes Nana's hand position to Popo.
    t::fn_80123218(np);
    assert_eq!(unsafe { (popo.u.pp.x2240.x, popo.u.pp.x2240.y, popo.u.pp.x2240.z) }, (1.0, 2.0, 3.0));

    // Coll_4: airborne -> back to the ground motion; ground with a wall bounce reverses.
    attrs.x14C = 0.5;
    nana.self_vel.x = 4.0;
    nana.coll_data.env_flags = Collide_LeftWallMask as _;
    nana.facing_dir = 1.0;
    t::ftPp_SpecialHi_4_Coll(np);
    assert_eq!(nana.self_vel.x, -2.0);
    assert_eq!(nana.facing_dir, -1.0);
    assert!(take().starts_with("roty 0 -1.57"));
    // The ground state change goes through the AirToGround stub path when airborne.
    nana.coll_data.env_flags = Collide_CeilingPush as _;
    nana.self_vel.y = 3.0;
    t::ftPp_SpecialHi_4_Coll(np);
    assert_eq!(nana.self_vel.y, 0.0);
}
