//! Smoke test of tu::ftmasterhandslam (victim hand-off in Anim, TagCrush entry).
#![allow(static_mut_refs)]
mod mh_common;
use mh_common::*;
use melee_chara::tu::ftmasterhandslam as t;
use melee_types::*;

#[test]
fn smoke() {
    let mut r = Rig::new();
    let gp = r.gp();
    t::ftMh_Slam_Coll(gp);
    t::ftMh_Slam_Phys(gp);
    assert_eq!(take(), "85134");
    t::ftMh_Slam_Anim(gp);
    assert_eq!(take(), "");
    let mut v: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    let vp: *mut HSD_GObj = &mut *v;
    unsafe {
        melee_chara::tu::ftmasterhandsqueezing::ext::FTMH_CAPTUREWAITMASTERHAND_80155D6C = Some(|_, ms| log(format!("capwait {ms}")));
        melee_chara::tu::ftcrazyhandbackairplane2::ext::FTCO_800DE2A8 = Some(|_, _| log("de2a8"));
        melee_chara::tu::ftcrazyhandbackairplane2::ext::FTCO_800DE7C0 = Some(|_, _, _| log("de7c0"));
    }
    r.f.victim_gobj = vp;
    r.f.cmd_vars[0] = 1;
    r.f.mv.mh.unk0.x20 = 5;
    unsafe { FRAMES = 0 };
    t::ftMh_Slam_Anim(gp);
    assert_eq!(take(), "capwait 330,de2a8,de7c0,389");
    assert_eq!((r.f.cmd_vars[0], unsafe { r.f.mv.mh.unk0.x20 }), (0, 0));
    // no victim: only the capture-wait call (with a null gobj)
    r.f.victim_gobj = core::ptr::null_mut();
    r.f.cmd_vars[0] = 1;
    unsafe { FRAMES = 1 };
    t::ftMh_Slam_Anim(gp);
    assert_eq!(take(), "capwait 330");
}
