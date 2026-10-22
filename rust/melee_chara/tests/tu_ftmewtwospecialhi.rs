//! Smoke test of tu::ftmewtwospecialhi (Teleport): the aerial zoom entry (two fused sites,
//! checked against an independent double-precision fused multiply-add), the start entries and the
//! timer countdown, through recording hooks.
#![allow(static_mut_refs)]

use melee_chara::stubs as st;
use melee_chara::tu::ftmewtwospecialhi as t;
use melee_runtime::gekko::{cosf, sinf};
use melee_runtime::lbtrigf::atan2f;
use melee_types::*;

static mut LOG: std::vec::Vec<String> = std::vec::Vec::new();
fn log(s: String) {
    unsafe { LOG.push(s) }
}
fn take() -> String {
    unsafe { std::mem::take(&mut LOG).join(",") }
}

#[test]
fn smoke() {
    let mut attrs: Box<ftMewtwoAttributes> = Box::new(unsafe { core::mem::zeroed() });
    let mut f: Box<Fighter> = Box::new(Fighter::zeroed());
    f.dat_attrs = &mut *attrs as *mut _ as *mut _;
    let mut g: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    g.user_data = &mut *f as *mut Fighter as *mut _;
    let gp: *mut HSD_GObj = &mut *g;
    unsafe {
        st::FIGHTER_CHANGEMOTIONSTATE = Some(|_, ms, _, frame, _, _, _| log(format!("M{ms}@{frame}")));
        st::FTANIM_8006EBA4 = Some(|_| log("8006EBA4".into()));
        st::FTANIM_SETANIMRATE = Some(|_, r| log(format!("rate {r}")));
        st::FTANIM_ISFRAMESREMAINING = Some(|_| 1);
        t::ext::FTCOLL_8007B62C = Some(|_, v| log(format!("coll {v}")));
        t::ext::FT_PLAYSFX = Some(|_, id, _, _| log(format!("sfx {id:#x}")));
        t::ext::FTCOMMON_8007DA24 = Some(|_| log("da24".into()));
    }

    // Start entries: cleared velocity (ground) / divided velocity (air), callback armed.
    f.gr_vel = 3.0;
    f.self_vel = Vec3 { x: 2.0, y: 4.0, z: 0.0 };
    t::ftMt_SpecialHiStart_Enter(gp);
    assert_eq!(take(), format!("M{}@0,8006EBA4", ftMt_MS_SpecialHiStart));
    assert_eq!((f.gr_vel, f.self_vel.x, f.self_vel.y), (0.0, 0.0, 0.0));
    assert!(f.accessory4_cb.is_some());
    attrs.x40_MEWTWO_TELEPORT_VEL_DIV_X = 2.0;
    attrs.x44_MEWTWO_TELEPORT_VEL_DIV_Y = 4.0;
    f.self_vel = Vec3 { x: 2.0, y: 4.0, z: 0.0 };
    t::ftMt_SpecialAirHiStart_Enter(gp);
    assert_eq!(take(), format!("M{}@0,8006EBA4", ftMt_MS_SpecialAirHiStart));
    assert_eq!((f.self_vel.x, f.self_vel.y), (1.0, 1.0));

    // Anim: frames remaining keeps the start state.
    t::ftMt_SpecialHiStart_Anim(gp);
    assert_eq!(take(), "");

    // Lost timer: counts down.
    f.mv.mt.SpecialHi.travelFrames = 2;
    t::ftMt_SpecialHiLost_Anim(gp);
    assert_eq!(unsafe { f.mv.mt.SpecialHi.travelFrames }, 1);
    assert_eq!(take(), "");

    // Aerial zoom entry, stick pushed up-right: the momentum is fmadds(m, s, add) * dir.
    attrs.x58_MEWTWO_TELEPORT_STICK_RANGE_MIN = 0.2;
    attrs.x5C_MEWTWO_TELEPORT_MOMENTUM = 3.3;
    attrs.x60_MEWTWO_TELEPORT_MOMENTUM_ADD = 0.7;
    attrs.x50_MEWTWO_TELEPORT_DURATION = 42;
    f.facing_dir = 1.0;
    f.input.lstick[0] = Vec2 { x: 0.5, y: 0.5 };
    t::ftMt_SpecialAirHi_Enter(gp);
    let s = melee_runtime::gekko::sqrtf(0.5f32 * 0.5 + 0.5 * 0.5);
    let m = ((3.3f32 as f64) * (s as f64) + 0.7f32 as f64) as f32;
    let ang = atan2f(0.5, 0.5 * 1.0);
    assert_eq!(f.self_vel.x.to_bits(), (1.0f32 * (m * cosf(ang))).to_bits());
    assert_eq!(f.self_vel.y.to_bits(), (m * sinf(ang)).to_bits());
    assert_eq!(unsafe { (f.mv.mt.SpecialHi.stickX, f.mv.mt.SpecialHi.stickY) }, (0.5, 0.5));
    assert_eq!(unsafe { f.mv.mt.SpecialHi.travelFrames }, 42);
    assert_eq!(f.invisible(), 1);
    assert_eq!(
        take(),
        format!("M{}@35,8006EBA4,rate 0,coll 2,sfx 0x30da1", ftMt_MS_SpecialAirHiLost)
    );

    // Stick in the dead zone: straight up with the plain momentum.
    f.input.lstick[0] = Vec2 { x: 0.0, y: 0.0 };
    t::ftMt_SpecialAirHi_Enter(gp);
    take();
    assert_eq!(unsafe { (f.mv.mt.SpecialHi.stickX, f.mv.mt.SpecialHi.stickY) }, (0.0, 1.0));
    let m = ((3.3f32 as f64) * 1.0 + 0.7f32 as f64) as f32;
    assert_eq!(f.self_vel.y.to_bits(), (m * sinf(core::f64::consts::FRAC_PI_2 as f32)).to_bits());
}
