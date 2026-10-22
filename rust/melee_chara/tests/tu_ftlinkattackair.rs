//! Smoke test of tu::ftlinkattackair: the Hylian-shield setup, the down-air drill callbacks and
//! `ftLk_AttackAir_Enter` through recording hooks (the arithmetic is two subtractions; no C
//! reference). One #[test]: hooks are globals.
#![allow(static_mut_refs, unused_unsafe)]

use melee_chara::stubs as st;
use melee_chara::tu::ftlinkattackair as t;
use melee_fighter::hooks as fh;
use melee_fighter::tu::ftCo_AttackAir::ext::hook as aa;
use melee_types::*;

static mut LOG: std::vec::Vec<String> = std::vec::Vec::new();
fn log(s: String) {
    unsafe { LOG.push(s) }
}
fn take_log() -> String {
    unsafe { std::mem::take(&mut LOG).join(",") }
}

#[test]
fn smoke() {
    unsafe { body() }
}

unsafe fn body() {
    let mut f: Box<Fighter> = Box::new(Fighter::zeroed());
    let mut g: Box<HSD_GObj> = Box::new(core::mem::zeroed());
    let da: &'static mut ftLk_DatAttrs = Box::leak(Box::new(ftLk_DatAttrs::zeroed()));
    let cd: &'static mut ftCommonData = Box::leak(Box::new(ftCommonData::zeroed()));
    f.dat_attrs = da as *mut _ as *mut _;
    g.user_data = &mut *f as *mut Fighter as *mut _;
    let gp: *mut HSD_GObj = &mut *g;
    fh::P_FT_COMMON_DATA = cd as *const _;

    da.attackairlw_hit_vel_y = 2.5;
    da.attackairlw_hit_anim_frame_start = 10.0;
    da.attackairlw_hit_anim_frame_end = 30.0;
    da.attackairlw_anim_flags = [0xA, 0xB, 0xC];

    // Shield setup: only when x5F4_arr[2].prev == 0.
    t::ext::FTCOLL_8007B1B8 = Some(|_, shield, cb| log(format!("shield {} {}", !shield.is_null(), cb.is_some())));
    f.x5F4_arr[2].prev = 1;
    t::ftLk_AttackAir_800EB3BC(gp);
    assert_eq!(take_log(), "");
    f.x5F4_arr[2].prev = 0;
    t::ftLk_AttackAir_800EB3BC(gp);
    assert_eq!(take_log(), "shield true true");
    assert_eq!((f.anon_221b.x221B_b2(), f.anon_221b.x221B_b3(), f.anon_221b.x221B_b4()), (1, 1, 1));

    // lwOnHit: bounce, restart the drill when the animation is past frame_len (= end - start).
    t::ext::FTCOLL_8007AFF8 = Some(|_| log("8007AFF8".into()));
    st::FIGHTER_CHANGEMOTIONSTATE = Some(|_, ms, fl, start, speed, blend, _| log(format!("ms {ms} {fl:x} {start} {speed} {blend}")));
    f.cur_anim_frame = 20.0;
    f.set_fall_fast(1);
    t::lwOnHit(gp);
    assert_eq!(take_log(), "8007AFF8");
    assert_eq!((f.self_vel.y, f.fall_fast(), f.mv.lk.attackair.lw_frame_start), (2.5, 0, 10.0));
    f.cur_anim_frame = 21.0;
    t::lwOnHit(gp);
    assert_eq!(take_log(), "8007AFF8,ms 69 0 20 1 0");

    // lwOnAnim: the delay counts down by frame_speed_mul; at <= 0 all three capsules re-arm.
    t::ext::FTCOLL_8007B064 = Some(|_, idx| log(format!("b064 {idx}")));
    t::ext::FTCOLL_8007ABD0 = Some(|_, flags, _| log(format!("abd0 {flags:x}")));
    fh::FTANIM_IS_FRAMES_REMAINING = Some(|_| true);
    f.frame_speed_mul = 4.0;
    f.cur_anim_frame = 5.0;
    f.mv.lk.attackair.lw_frame_start = 6.0;
    t::lwOnAnim(gp);
    assert_eq!((take_log().as_str(), f.mv.lk.attackair.lw_frame_start), ("", 2.0));
    t::lwOnAnim(gp);
    assert_eq!(take_log(), "b064 0,abd0 a,b064 1,abd0 b,b064 2,abd0 c");
    assert_eq!(f.mv.lk.attackair.lw_frame_start, -2.0);
    t::lwOnAnim(gp);
    assert_eq!(take_log(), "");
    // Past the end frame the capsules stay off.
    f.cur_anim_frame = 31.0;
    f.mv.lk.attackair.lw_frame_start = 1.0;
    t::lwOnAnim(gp);
    assert_eq!(take_log(), "");

    // Enter: a C-stick down press selects AttackAirLw and installs the drill callbacks.
    aa::ftCo_800DF478 = Some(|_| true);
    aa::ftCo_GetCStickAngle = Some(|_| -2.0);
    aa::Fighter_ChangeMotionState = Some(|_, ms, _, _, _, _, _| log(format!("enter {ms}")));
    aa::ftAnim_8006EBA4 = Some(|_| {});
    cd.x20_radians = 1.0;
    cd.xDC = 0.5;
    cd.xE0 = 0.5;
    f.input.cstick[0].x = 0.0;
    f.input.cstick[0].y = -1.0;
    f.motion_id = ftCo_MS_AttackAirLw as _;
    f.mv.lk.attackair.lw_frame_start = 9.0;
    t::ftLk_AttackAir_Enter(gp);
    assert_eq!(take_log(), "enter 69");
    assert!(f.deal_dmg_cb.is_some() && f.anim_cb.is_some());
    assert_eq!(f.mv.lk.attackair.lw_frame_start, 0.0);
    f.motion_id = ftCo_MS_AttackAirN as _;
    f.deal_dmg_cb = None;
    t::ftLk_AttackAir_Enter(gp);
    assert!(f.deal_dmg_cb.is_none());
}
