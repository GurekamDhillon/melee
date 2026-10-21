//! Smoke test of tu::ftcrazyhandslam (float countdown, the two ftBossLib motion probes, the Coll
//! velocity clear).
#![allow(static_mut_refs)]

use melee_chara::stubs as st;
use melee_chara::tu::{ftcrazyhandfingerbeam0 as fb, ftcrazyhandslam as t};
use melee_types::*;

static mut LOG: std::vec::Vec<String> = std::vec::Vec::new();
fn log(s: String) {
    unsafe { LOG.push(s) }
}

#[test]
fn smoke() {
    let mut f: Box<Fighter> = Box::new(Fighter::zeroed());
    let mut g: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    let mut v: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    g.user_data = &mut *f as *mut Fighter as *mut _;
    let gp: *mut HSD_GObj = &mut *g;
    let vp: *mut HSD_GObj = &mut *v;
    unsafe {
        // no MasterHand / CrazyHand on the field: the ftBossLib probes see an empty p-link
        melee_fighter::hooks::PLINK_HEAD = Some(|_| core::ptr::null_mut());
        st::FTANIM_ISFRAMESREMAINING = Some(|_| 1);
        st::FT_80085134 = Some(|_| {});
        st::FIGHTER_CHANGEMOTIONSTATE = Some(|_, m, _, _, _, _, _| log(format!("ms {m:x}")));
        st::FTANIM_8006EBA4 = Some(|_| log("anim".into()));
        st::FIGHTER_UNKSETFLAG_8006CFBC = Some(|_| log("flag".into()));
        fb::ext::FTCH_GRABUNK1_8015BC88 = Some(|_| log("wait".into()));
        t::ext::FTCH_GRABUNK1_8015B800 = Some(|_| log("tag".into()));
    }
    // Slam: frames remain and no boss probe -> nothing
    t::ftCh_Slam_Anim(gp);
    assert!(unsafe { LOG.is_empty() });
    unsafe { st::FTANIM_ISFRAMESREMAINING = Some(|_| 0) };
    f.self_vel = Vec3 { x: 1.0, y: 2.0, z: 3.0 };
    f.x1A5C = gp;
    t::ftCh_Slam_Anim(gp);
    assert_eq!((f.self_vel.x, f.self_vel.y, f.self_vel.z), (0.0, 0.0, 0.0));
    assert!(f.x1A5C.is_null());
    // Fail
    f.x1A5C = gp;
    t::ftCh_Fail_Anim(gp);
    assert!(f.x1A5C.is_null());
    t::ftCh_TagCrush_Anim(gp);
    // TagApplaud: countdown reaches 0 with the flag set -> tag the victim once
    f.victim_gobj = vp;
    f.cmd_vars[0] = 1;
    f.mv.ch.unk0.x24 = 2.0;
    unsafe { st::FTANIM_ISFRAMESREMAINING = Some(|_| 1) };
    t::ftCh_TagApplaud_Anim(gp);
    assert_eq!((unsafe { f.mv.ch.unk0.x24 }, f.cmd_vars[0]), (1.0, 1));
    t::ftCh_TagApplaud_Anim(gp);
    assert_eq!((unsafe { f.mv.ch.unk0.x24 }, f.cmd_vars[0]), (0.0, 0));
    t::ftCh_TagApplaud_Anim(gp);
    // Coll clears velocity only while x18 == 0
    f.self_vel = Vec3 { x: 1.0, y: 2.0, z: 3.0 };
    f.mv.ch.unk0.x18 = 4.0;
    t::ftCh_TagApplaud_Coll(gp);
    assert_eq!(f.self_vel.y, 2.0);
    f.mv.ch.unk0.x18 = 0.0;
    t::ftCh_TagApplaud_Coll(gp);
    assert_eq!((f.self_vel.x, f.self_vel.y, f.self_vel.z), (0.0, 0.0, 0.0));
    t::ftCh_GrabUnk1_8015A888(gp);
    assert_eq!(unsafe { LOG.join(",") }, "flag,ms 17e,anim,flag,wait,wait,tag,ms 17e,anim");
}
