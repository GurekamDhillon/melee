//! Smoke test of tu::ftmasterhandtagrockpaper (hooks record the calls).
#![allow(static_mut_refs)]
mod mh_common;
use mh_common::*;
use melee_chara::tu::ftmasterhandtagrockpaper as t;
use melee_types::*;

#[test]
fn smoke() {
    let mut r = Rig::new();
    let gp = r.gp();
    unsafe {
        // No Crazy Hand on the p-link.
        melee_fighter::hooks::PLINK_HEAD = Some(|_| core::ptr::null_mut());
        t::ext::FTCO_800DE2A8 = Some(|_, _| log("e2a8"));
        t::ext::FTCO_800DE7C0 = Some(|_, _, up| log(format!("e7c0 {up}")));
        melee_chara::tu::ftmasterhandthrownmasterhand::ext::FTCO_CAPTURECUT_ENTER = Some(|_| log("capturecut"));
    }
    t::ftMh_TagRockPaper_Phys(gp);
    t::ftMh_TagGrab_Phys(gp);
    t::ftMh_TagSqueeze_Phys(gp);
    t::ftMh_TagFail_Phys(gp);
    assert_eq!(take(), "85134,85134,85134,85134");
    t::ftMh_TagRockPaper_Coll(gp);
    t::ftMh_TagGrab_Coll(gp);
    t::ftMh_TagSqueeze_Coll(gp);
    t::ftMh_TagFail_Coll(gp);

    // Anim with frames remaining: nothing.
    unsafe { FRAMES = 1 };
    t::ftMh_TagRockPaper_Anim(gp);
    t::ftMh_TagGrab_Anim(gp);
    t::ftMh_TagFail_Anim(gp);
    assert_eq!(take(), "");
    // Animation over.
    unsafe { FRAMES = 0 };
    r.f.x1A5C = 1usize as *mut _;
    t::ftMh_TagRockPaper_Anim(gp);
    assert_eq!((take(), r.f.x1A5C.is_null()), ("setflag,cms 389 0 0 1 0,8006EBA4".to_string(), true));
    r.f.x1A5C = 1usize as *mut _;
    t::ftMh_TagGrab_Anim(gp);
    assert_eq!(take(), "setflag,cms 387 0 0 1 0,8006EBA4");
    t::ftMh_TagFail_Anim(gp);
    assert_eq!(take(), "cms 389 0 0 1 0,8006EBA4");

    // TagSqueeze_Anim with cmd_vars[0] set: victim thrown, facing negated.
    let mut vf: Box<Fighter> = Box::new(Fighter::zeroed());
    vf.facing_dir = 1.0;
    vf.dmg.facing_dir_1 = 2.0;
    vf.victim_gobj = gp;
    let mut vg: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    vg.user_data = &mut *vf as *mut Fighter as *mut _;
    r.f.victim_gobj = &mut *vg;
    r.f.cmd_vars[0] = 1;
    unsafe { FRAMES = 1 };
    t::ftMh_TagSqueeze_Anim(gp);
    assert_eq!(r.f.cmd_vars[0], 0);
    // victim (as the "hand") enters ThrownMasterHand (0x14a), then the hooks run.
    assert_eq!(take(), "cms 330 0 0 1 0,8006EBA4,e2a8,e7c0 false");
    assert_eq!(vf.dmg.facing_dir_1, -2.0);

    // TagCancel_Anim: counter 1 -> 0 releases the victim (CaptureCut), then 389_80151018.
    unsafe {
        r.f.mv.mh.unk0.x24 = 1.0;
        r.f.cmd_vars[0] = 1;
        FRAMES = 1;
    }
    r.f.set_invisible(1);
    t::ftMh_TagCancel_Anim(gp);
    assert_eq!(take(), "capturecut");
    assert_eq!(r.f.cmd_vars[0], 0);
    // x24 stays > 0: no release.
    unsafe {
        r.f.mv.mh.unk0.x24 = 5.0;
        r.f.cmd_vars[0] = 1;
    }
    t::ftMh_TagCancel_Anim(gp);
    assert_eq!((take(), r.f.cmd_vars[0], unsafe { r.f.mv.mh.unk0.x24 }), ("".to_string(), 1, 4.0));
    // Coll: stops only with x18 == 0.
    unsafe { r.f.mv.mh.unk0.x18 = 1.0 };
    r.f.self_vel.x = 4.0;
    t::ftMh_TagCancel_Coll(gp);
    assert_eq!(r.f.self_vel.x, 4.0);
    unsafe { r.f.mv.mh.unk0.x18 = 0.0 };
    t::ftMh_TagCancel_Coll(gp);
    assert_eq!(r.f.self_vel.x, 0.0);
}
