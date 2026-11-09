//! Smoke test of tu::ftnanaspecials: Nana's copy of Popo's side-B. The fused trailing offset
//! (`-(1.5f * (x0 * facing) - popo.x)`) is checked against an independent double-precision fused
//! multiply-add; the rest runs through recording hooks.
#![allow(static_mut_refs)]

use melee_chara::stubs as st;
use melee_chara::tu::ftnanaspecials as t;
use melee_fighter::hooks as fh;
use melee_types::*;

static mut LOG: std::vec::Vec<String> = std::vec::Vec::new();
fn log(s: String) {
    unsafe { LOG.push(s) }
}
fn take() -> String {
    unsafe { std::mem::take(&mut LOG).join(",") }
}
static mut POPO: *mut HSD_GObj = core::ptr::null_mut();
static mut POPO_DONE: bool = false;
static mut POPO_6FC: bool = false;
static mut ECB_OK: bool = true;

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
    let mut ng: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    let mut pg: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    ng.user_data = &mut *nana as *mut Fighter as *mut _;
    pg.user_data = &mut *popo as *mut Fighter as *mut _;
    let np: *mut HSD_GObj = &mut *ng;
    unsafe { POPO = &mut *pg };
    unsafe {
        t::ext::PLAYER_GETENTITYATINDEX = Some(|_, _| POPO);
        t::ext::FT_800849EC = Some(|_, _| log("849ec".into()));
        t::ext::FTPARTSETROTX = Some(|_, part, v| log(format!("rotx {part} {v}")));
        t::ext::FTPARTGETROTX = Some(|_, _| 0.25);
        t::ext::FTPARTSETROTY = Some(|_, part, v| log(format!("roty {part} {v}")));
        t::ext::FTPP_SPECIALS_8011F964 = Some(|_| POPO_DONE);
        t::ext::FTPP_SPECIALS_8011F6FC = Some(|_| POPO_6FC);
        t::ext::FT_80082888 = Some(|_, _| ECB_OK);
        t::ext::FT_800824A0 = Some(|_, _| ECB_OK);
        t::ext::SLIPPI_VERSION = Some(|| 20001);
        melee_chara::tu::ftmewtwospecialhi::ext::FIGHTER_SETEFFECTHITLAGCALLBACKS = Some(|_| log("hitlag".into()));
        fh::SLIPPI_CODES = Some(|| 0);
        st::FIGHTER_UNKSETFLAG_8006CFBC = Some(|_| log("flag".into()));
        st::FIGHTER_CHANGEMOTIONSTATE = Some(|_, ms, fl, _, _, _, _| log(format!("M{ms} {fl:#x}")));
        st::FTCO_FALL_ENTER = Some(|_| log("fall".into()));
        st::FT_8008A2BC = Some(|_| log("wait".into()));
        st::FTCOMMON_8007D7FC = Some(|_| log("7d7fc".into()));
        st::FTANIM_8006EBA4 = Some(|_| log("ebA4".into()));
    }

    // Anim: Popo's move still running -> mirror his frame; over -> release the link and leave.
    popo.cur_anim_frame = 12.5;
    unsafe { POPO_DONE = false };
    t::ftPp_SpecialS_0_Anim(np);
    assert_eq!(nana.cur_anim_frame, 12.5);
    assert_eq!(take(), "");
    nana.x1A5C = unsafe { POPO };
    unsafe { POPO_DONE = true };
    t::ftPp_SpecialS_0_Anim(np);
    assert_eq!(take(), "rotx 0 0,flag,flag,wait");
    assert!(nana.x1A5C.is_null());
    assert!(nana.take_dmg_cb.is_none() && nana.death2_cb.is_none());
    attrs.x12C = 0.0;
    t::ftPp_SpecialS_1_Anim(np);
    assert_eq!(take(), "rotx 0 0,flag,fall");
    unsafe { POPO_DONE = false };

    // Phys: copies Popo's velocities and facing, then turns to face.
    popo.self_vel = Vec3 { x: 1.0, y: 2.0, z: 0.0 };
    popo.gr_vel = 3.0;
    popo.facing_dir = -1.0;
    t::ftPp_SpecialS_0_Phys(np);
    assert_eq!((nana.self_vel.x, nana.self_vel.y, nana.gr_vel, nana.facing_dir), (1.0, 2.0, 3.0, -1.0));
    assert!(take().starts_with("roty 0 -1.57"));

    // Coll_0 while Popo's move runs: snap to him, switch motion if the ground state differs.
    popo.cur_pos = Vec3 { x: 7.0, y: 8.0, z: 0.0 };
    popo.ground_or_air = GA_Air as _;
    nana.ground_or_air = GA_Ground as _;
    t::ftPp_SpecialS_0_Coll(np);
    assert_eq!(take(), "849ec,M360 0xc4c528a,rotx 0 0.25,hitlag");
    assert_eq!((nana.cur_pos.x, nana.cur_pos.y), (7.0, 8.0));
    assert_eq!(nana.ground_or_air, popo.ground_or_air);
    assert!(nana.take_dmg_cb.is_some() && nana.death2_cb.is_some());

    // Coll_0 once Popo's move ended: Nana trails 1.5 units per frame (fused), ramping to 5.
    unsafe { POPO_DONE = true };
    nana.facing_dir = 1.0;
    unsafe { nana.mv.pp.unk_80123954.x0 = 0 };
    for step in 1..=7 {
        t::ftPp_SpecialS_0_Coll(np);
        let k = step.min(5);
        let want = -fma32(1.5, k as f32 * 1.0, -7.0);
        assert_eq!(nana.cur_pos.x.to_bits(), want.to_bits(), "step {step}");
        assert_eq!(unsafe { nana.mv.pp.unk_80123954.x0 }, k);
        assert_eq!(take(), "rotx 0 0.25,hitlag");
    }
    // ... and a failed ground test (SpecialS_0) triggers the air switch.
    unsafe { ECB_OK = false };
    t::ftPp_SpecialS_0_Coll(np);
    assert!(take().contains("M360 0xc4c528a"));
    nana.facing_dir = -1.0;
    unsafe { ECB_OK = true };
    t::ftPp_SpecialS_0_Coll(np);
    assert_eq!(nana.cur_pos.x.to_bits(), (-fma32(1.5, 5.0 * -1.0, -7.0)).to_bits());
    take();
    // SpecialS_1 triggers its switch when the test passes.
    t::ftPp_SpecialS_1_Coll(np);
    assert!(take().contains("7d7fc"));

    // 80123B10: motions 359 / 360 are the "following" ones.
    nana.motion_id = 359;
    assert!(!t::ftNn_Init_80123B10(np));
    nana.motion_id = 361;
    assert!(t::ftNn_Init_80123B10(np));

    // 80123954: sub-states 1..=13 free Nana unless Slippi's FreezeGlitchFix is active.
    nana.x2070.x0 = unsafe { core::mem::zeroed() };
    unsafe { nana.x2070.x0.set_x2071_b0_3(4) };
    nana.x1A5C = unsafe { POPO };
    assert!(t::ftNn_Init_80123954(np, GA_Ground as _));
    assert!(nana.x1A5C.is_null());
    nana.x1A5C = unsafe { POPO };
    unsafe { fh::SLIPPI_CODES = Some(|| 1) };
    assert!(t::ftNn_Init_80123954(np, GA_Ground as _));
    assert!(!nana.x1A5C.is_null(), "FreezeGlitchFix keeps the link");
    assert!(t::ftNn_Init_80123954(core::ptr::null_mut(), GA_Ground as _));

    // Otherwise: close enough to Popo -> snapped back (returns false), else released.
    unsafe { nana.x2070.x0.set_x2071_b0_3(0) };
    attrs.xD0 = 10.0;
    popo.x34_scale.y = 1.0;
    popo.cur_pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    nana.cur_pos = Vec3 { x: 3.0, y: 4.0, z: 0.0 };
    nana.ground_or_air = GA_Ground as _;
    popo.facing_dir = 1.0;
    assert!(!t::ftNn_Init_80123954(np, GA_Ground as _));
    assert_eq!((nana.cur_pos.x, nana.cur_pos.y), (0.0, 0.0));
    assert_eq!(nana.x1A5C, unsafe { POPO });
    take();
    nana.cur_pos = Vec3 { x: 30.0, y: 40.0, z: 0.0 };
    assert!(t::ftNn_Init_80123954(np, GA_Ground as _));
    assert!(nana.x1A5C.is_null());
    // Exactly at the limit: 25 < (s32)(1 * 25) is false.
    attrs.xD0 = 5.0;
    nana.cur_pos = Vec3 { x: 3.0, y: 4.0, z: 0.0 };
    assert!(t::ftNn_Init_80123954(np, GA_Ground as _));
}
