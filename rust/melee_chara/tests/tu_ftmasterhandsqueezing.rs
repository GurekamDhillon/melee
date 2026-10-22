//! Smoke test of tu::ftmasterhandsqueezing (Anim branch on x20, Throw_Anim victim hand-off).
#![allow(static_mut_refs)]
mod mh_common;
use mh_common::*;
use melee_chara::tu::ftmasterhandsqueezing as t;
use melee_types::*;

#[test]
fn smoke() {
    let mut r = Rig::new();
    let gp = r.gp();
    t::ftMh_Squeezing_Coll(gp);
    t::ftMh_Squeezing_Phys(gp);
    assert!(take().starts_with("85134"));
    t::ftMh_Squeezing_Anim(gp);
    assert_eq!(take(), "");
    // Throw: cmd_vars[0] set with a victim -> capture-wait, velocity reset, victim facing flipped
    let mut v: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    let mut vf: Box<Fighter> = Box::new(Fighter::zeroed());
    vf.dmg.facing_dir_1 = 1.0;
    v.user_data = &mut *vf as *mut Fighter as *mut _;
    let vp: *mut HSD_GObj = &mut *v;
    unsafe {
        t::ext::FTMH_CAPTUREWAITMASTERHAND_80155D6C = Some(|_, ms| log(format!("capwait {ms}")));
        melee_chara::tu::ftcrazyhandbackairplane2::ext::FTCO_800DE2A8 = Some(|_, _| log("de2a8"));
        melee_chara::tu::ftcrazyhandbackairplane2::ext::FTCO_800DE7C0 = Some(|_, _, _| log("de7c0"));
    }
    r.f.victim_gobj = vp;
    r.f.cmd_vars[0] = 1;
    r.f.mv.mh.unk0.x20 = 1;
    t::ftMh_Throw_Anim(gp);
    let l = take();
    assert!(l.starts_with("capwait 330,de2a8,de7c0"), "{l}");
    assert_eq!(vf.dmg.facing_dir_1, -1.0);
    assert_eq!(unsafe { r.f.mv.mh.unk0.x20 }, 0);
    assert_eq!(r.f.cmd_vars[0], 0);
}
