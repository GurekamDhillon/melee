//! Smoke test of tu::ftkoopaspecials (Koopa Klaw) through recording hooks, including the victim
//! side (ftCo_ThrownKoopa / ftCo_CaptureDamageKoopa from group 3) and the stick-flick direction.
//! The only float expression is the facing sign multiply. One #[test]: hooks are globals.
#![allow(static_mut_refs, unused_unsafe)]

use melee_chara::stubs as st;
use melee_chara::tu::ftkoopaspecials as t;
use melee_fighter::tu::ftCo_CaptureDamageKoopa::ext::hook as cdk;
use melee_fighter::tu::ftCo_ThrownKoopa::ext::hook as tk;
use melee_types::*;

static mut LOG: std::vec::Vec<String> = std::vec::Vec::new();
fn log(s: String) {
    unsafe { LOG.push(s) }
}
fn take_log() -> String {
    unsafe { std::mem::take(&mut LOG).join(",") }
}

const TF1: u32 = 0xC4C5088;

#[test]
fn smoke() {
    unsafe { body() }
}

unsafe fn body() {
    let mut f: Box<Fighter> = Box::new(Fighter::zeroed());
    let mut g: Box<HSD_GObj> = Box::new(core::mem::zeroed());
    let mut v: Box<Fighter> = Box::new(Fighter::zeroed());
    let mut vg: Box<HSD_GObj> = Box::new(core::mem::zeroed());
    let da: &'static mut ftKoopaAttributes = Box::leak(Box::new(core::mem::zeroed()));
    f.dat_attrs = da as *mut _ as *mut _;
    g.user_data = &mut *f as *mut Fighter as *mut _;
    vg.user_data = &mut *v as *mut Fighter as *mut _;
    v.victim_gobj = &mut *g;
    let gp: *mut HSD_GObj = &mut *g;
    let vp: *mut HSD_GObj = &mut *vg;

    st::FIGHTER_CHANGEMOTIONSTATE = Some(|_, ms, fl, start, _, _, _| log(format!("ms {ms} {fl:x} {start}")));
    st::FTANIM_8006EBA4 = Some(|_| {});
    st::FTANIM_ISFRAMESREMAINING = Some(|_| 0);
    st::FTCOMMON_8007D7FC = Some(|_| log("7D7FC".into()));
    st::FT_8008A2BC = Some(|_| log("Wait".into()));
    st::FTCO_FALL_ENTER = Some(|_| log("Fall".into()));
    tk::Fighter_ChangeMotionState = Some(|_, ms, fl, start, _, _, _| log(format!("victim-ms {ms} {fl:x} {start}")));
    tk::ftAnim_8006EBA4 = Some(|_| log("victim-anim".into()));
    cdk::Fighter_ChangeMotionState = Some(|_, ms, fl, _, _, _, _| log(format!("victim-dmg-ms {ms} {fl:x}")));
    cdk::ftAnim_8006EBA4 = Some(|_| {});
    t::ext::FTCOLL_8007ABD0 = Some(|_, flags, _| log(format!("abd0 {flags}")));
    t::ext::FTANIM_8006F0FC = Some(|_, r| log(format!("rate {r}")));
    t::ext::FTKP_SPECIALS_80132E30 = Some(|_| log("80132E30".into()));

    // Enter: the grab callbacks are armed, state 347 / 353.
    f.x2210.throw_flags = 7;
    f.cmd_vars[0] = 5;
    t::ftKp_SpecialS_Enter(gp);
    assert_eq!(take_log(), "ms 347 0 0");
    assert!(f.grab_cb.is_some() && f.grabbed_cb.is_some() && f.x2194.is_none());
    assert_eq!((f.x1A68, f.x2210.throw_flags, f.cmd_vars[0]), (8, 0, 5 * 0));
    f.mv.kp.specials.b_held = 1;
    t::ftKp_SpecialAirS_Enter(gp);
    assert_eq!(take_log(), "ms 353 0 0");
    assert_eq!((f.mv.kp.specials.b_held, f.mv.kp.specials.facing_dir, f.mv.kp.specials.x4, f.mv.kp.specials.xC), (0, 0, 0, 0));

    // Grab-hit entries: 8013302C / 801330E4 pick the loop or the B-held variant by x4.
    f.mv.kp.specials.x4 = 0;
    t::ftKp_SpecialS_8013302C(gp);
    f.mv.kp.specials.x4 = 1;
    t::ftKp_SpecialS_8013302C(gp);
    t::ftKp_SpecialS_801330E4(gp);
    f.mv.kp.specials.x4 = 0;
    t::ftKp_SpecialS_801330E4(gp);
    assert_eq!(take_log(), "ms 348 0 0,ms 349 80 0,ms 355 80 0,ms 354 0 0");
    assert_eq!(f.x2222_b2(), 1);
    assert_eq!(f.x1A6A, 511);

    // Ground <-> air transitions re-arm the callbacks.
    t::ftKp_SpecialS_8013319C(gp);
    assert_eq!(take_log(), format!("ms 353 {TF1:x} 0"));
    t::ftKp_SpecialS_8013322C(gp);
    assert_eq!(take_log(), format!("7D7FC,ms 347 {TF1:x} 0"));
    f.cur_anim_frame = 4.0;
    t::ftKp_SpecialS_801333F8(gp);
    assert_eq!(take_log(), "7D7FC,ms 350 44c1080 4,rate 0");

    // Falling out of a hold with no victim.
    f.victim_gobj = core::ptr::null_mut();
    f.ground_or_air = GA_Air;
    t::ftKp_SpecialS_801332C4(gp);
    assert_eq!(take_log(), "Fall");

    // set_dir through Hit_IASA: the flick must cross +-x30 this frame; the direction is
    // recorded relative to the facing.
    da.x30 = 0.5;
    f.victim_gobj = vp;
    v.facing_dir = -1.0;
    f.facing_dir = 1.0;
    f.input.lstick[1].x = 0.0;
    f.input.lstick[0].x = 0.75;
    f.input.pressed_buttons = HSD_PAD_B as u32;
    f.mv.kp.specials.b_held = 0;
    t::ftKp_SpecialSHit_IASA(gp);
    assert_eq!(f.mv.kp.specials.b_held, 1);
    assert_eq!(f.mv.kp.specials.facing_dir, 1);
    assert_eq!(take_log(), "ms 351 80000 0,victim-ms 281 0 0,victim-anim");
    f.facing_dir = -1.0;
    f.input.lstick[0].x = -0.75;
    f.input.pressed_buttons = 0;
    t::ftKp_SpecialAirSHit_IASA(gp);
    assert_eq!(f.mv.kp.specials.facing_dir, 1);
    assert_eq!(take_log(), "ms 357 80000 0,victim-ms 286 0 0,victim-anim");
    f.facing_dir = 1.0;
    t::ftKp_SpecialAirSHit_IASA(gp);
    assert_eq!(f.mv.kp.specials.facing_dir, -1);
    assert_eq!(take_log(), "ms 358 80000 0,victim-ms 287 0 0,victim-anim");
    // No flick: nothing happens.
    f.input.lstick[1].x = -0.75;
    t::ftKp_SpecialSHit_IASA(gp);
    assert_eq!(take_log(), "");

    // Wait IASA without a flick: B held switches to the Hit variant (x4 set) and the victim
    // enters CaptureDamageKoopa.
    f.input.lstick[1].x = 0.0;
    f.input.lstick[0].x = 0.0;
    f.mv.kp.specials.b_held = 1;
    f.mv.kp.specials.x4 = 0;
    t::ftKp_SpecialSWait_IASA(gp);
    assert_eq!(f.mv.kp.specials.x4, 1);
    assert_eq!(f.mv.kp.specials.b_held, 0);
    assert_eq!(take_log(), format!("ms 349 80 0,victim-dmg-ms {} 0", ftCo_MS_CaptureDamageKoopa));
    f.mv.kp.specials.b_held = 1;
    f.mv.kp.specials.x4 = 0;
    t::ftKp_SpecialAirSWait_IASA(gp);
    assert_eq!(take_log(), format!("ms 355 80 0,victim-dmg-ms {} 0", ftCo_MS_CaptureDamageKoopaAir));

    // Hit anim: x4 + cmd_vars[0] re-arm the hit capsule; at the end, B held -> Hit1, else Hit0_1.
    da.x2C = 21;
    f.cmd_vars[0] = 1;
    f.mv.kp.specials.x4 = 1;
    f.mv.kp.specials.b_held = 1;
    t::ftKp_SpecialSHit_Anim(gp);
    assert_eq!(take_log(), format!("abd0 21,ms 349 80 0,victim-dmg-ms {} 0", ftCo_MS_CaptureDamageKoopa));
    f.mv.kp.specials.b_held = 0;
    f.cur_anim_frame = 8.0;
    t::ftKp_SpecialSHit_Anim(gp);
    assert_eq!(take_log(), "ms 350 80080 8,rate 0");
    // Air hit anim.
    f.cmd_vars[0] = 1;
    f.mv.kp.specials.x4 = 1;
    f.mv.kp.specials.b_held = 0;
    t::ftKp_SpecialAirSHit_Anim(gp);
    assert_eq!(take_log(), "abd0 21,ms 356 80080 8,rate 0");
    f.mv.kp.specials.b_held = 1;
    f.mv.kp.specials.x4 = 0;
    t::ftKp_SpecialAirSHit_Anim(gp);
    assert_eq!(f.mv.kp.specials.x4, 1);
    assert_eq!(take_log(), format!("ms 355 80 0,victim-dmg-ms {} 0", ftCo_MS_CaptureDamageKoopaAir));

    // End anims: B4 pulse flips the facing and marks xC; End-B follows the victim position.
    f.victim_gobj = core::ptr::null_mut();
    f.x2210.x0.set_throw_flags_b4(1);
    f.facing_dir = 1.0;
    t::ftKp_SpecialSEndF_Anim(gp);
    assert_eq!((f.facing_dir, f.mv.kp.specials.xC), (-1.0, 1));
    assert_eq!(take_log(), "Wait");
    t::ftKp_SpecialSEndB_Anim(gp);
    t::ftKp_SpecialAirSEndB_Anim(gp);
    t::ftKp_SpecialAirSEndF_Anim(gp);
    assert_eq!(take_log(), "80132E30,Wait,80132E30,Fall,Fall");

    // Air end-to-ground: the facing is flipped around the state change when xC is set.
    f.mv.kp.specials.xC = 1;
    f.facing_dir = 1.0;
    f.cur_anim_frame = 2.0;
    t::ftKp_SpecialS_801334E4(gp);
    assert_eq!(f.facing_dir, 1.0);
    assert_eq!(take_log(), format!("7D7FC,ms 351 {TF1:x} 2"));

    // Phys: AirSStart keeps the vertical velocity the shared air physics reset.
    st::FT_80085134 = Some(|_| log("air-phys".into()));
    f.self_vel.y = 3.0;
    f.co_attrs.gravity = 0.5;
    f.co_attrs.terminal_velocity = 4.0;
    t::ftKp_SpecialAirSStart_Phys(gp);
    assert_eq!(take_log(), "air-phys");
    assert_eq!(f.self_vel.y, 2.5);
}
