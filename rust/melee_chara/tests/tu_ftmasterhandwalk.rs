//! Smoke test of tu::ftmasterhandwalk: the Walk-shoot / WalkLoop Anim callbacks through
//! recording hooks. The fused `my_lbVector_Len` is checked at its threshold.
#![allow(static_mut_refs)]

use melee_chara::stubs as st;
use melee_chara::tu::ftmasterhandwalk as t;
use melee_types::*;

static mut LOG: std::vec::Vec<String> = std::vec::Vec::new();
fn log(s: String) {
    unsafe { LOG.push(s) }
}
fn take() -> String {
    unsafe { std::mem::take(&mut LOG).join(",") }
}
static mut FRAMES_LEFT: i32 = 0;
static mut TARGET_POS: Vec3 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };

#[test]
fn smoke() {
    let mut attrs: Box<ftMasterHand_SpecialAttrs> = Box::new(unsafe { core::mem::zeroed() });
    let mut ftd: Box<ftData> = Box::new(unsafe { core::mem::zeroed() });
    ftd.ext_attr = &mut *attrs as *mut _ as *mut _;
    let mut f: Box<Fighter> = Box::new(Fighter::zeroed());
    f.ft_data = &mut *ftd;
    let mut g: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    g.user_data = &mut *f as *mut Fighter as *mut _;
    let gp: *mut HSD_GObj = &mut *g;
    unsafe {
        st::FTANIM_ISFRAMESREMAINING = Some(|_| FRAMES_LEFT);
        st::FIGHTER_CHANGEMOTIONSTATE = Some(|_, ms, _, _, _, _, _| log(format!("M{ms}")));
        st::FTANIM_8006EBA4 = Some(|_| log("8006EBA4".into()));
        t::ext::FTMH_MS_389_80151018 = Some(|_| log("389".into()));
        t::ext::FTMH_MS_349_80151CA8 = Some(|_| log("349".into()));
        t::ext::FTBOSSLIB_8015C244 = Some(|_, _| core::ptr::null_mut());
        t::ext::FTLIB_800866DC = Some(|_, out| *out = TARGET_POS);
        t::ext::MPFLOORGETLEFT = Some(|_, out| (*out).x = -1000.0);
    }

    // WalkShoot_Anim: only when the animation ran out.
    unsafe { FRAMES_LEFT = 1 };
    t::ftMh_WalkShoot_Anim(gp);
    assert_eq!(take(), "");
    unsafe { FRAMES_LEFT = 0 };
    t::ftMh_WalkShoot_Anim(gp);
    assert_eq!(take(), "389");
    t::ftMh_WalkShoot_Coll(gp);
    t::ftMh_Walk2_Coll(gp);
    t::ftMh_WalkLoop_Coll(gp);

    // WalkLoop_Anim: target (3,4,0) away from the origin is 5 units; within x4C -> 349.
    f.cur_pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    unsafe { TARGET_POS = Vec3 { x: 3.0, y: 4.0, z: 0.0 } };
    attrs.x4C = 5.5;
    unsafe { FRAMES_LEFT = 1 };
    t::ftMh_WalkLoop_Anim(gp);
    assert_eq!(take(), "349");
    attrs.x4C = 4.5;
    t::ftMh_WalkLoop_Anim(gp);
    assert_eq!(take(), "");
    // Animation over: restart the walk loop.
    unsafe { FRAMES_LEFT = 0 };
    t::ftMh_WalkLoop_Anim(gp);
    assert_eq!(take(), format!("M{},8006EBA4", ftMh_MS_WalkLoop as i32));
}
