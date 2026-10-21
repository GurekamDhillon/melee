//! Smoke test of tu::ftcrazyhandsweepwait (hooks record the calls).
#![allow(static_mut_refs)]
mod ch_common;
use ch_common::*;
use melee_chara::tu::ftcrazyhandsweepwait as t;
use melee_types::*;

#[test]
fn smoke() {
    let mut r = Rig::new();
    let gp = r.gp();
    t::ftCh_SweepWait_Coll(gp);
    t::ftCh_Slap_Coll(gp);
    // Slap_Phys: physics then self_vel.x += da->x40 (f32 add).
    r.da.x40 = 0.25;
    r.f.self_vel.x = 1.0;
    t::ftCh_Slap_Phys(gp);
    assert_eq!(take(), "85134");
    assert_eq!(r.f.self_vel.x, 1.25);
    // accessory callback: past x3C -> stop and start Walk2 (350 = 0x15e).
    r.da.x3C = 10.0;
    r.f.cur_pos.x = 10.0;
    r.f.self_vel.x = 3.0;
    unsafe { t::fn_801577B4(gp) };
    assert_eq!((take(), r.f.self_vel.x), ("".to_string(), 3.0));
    r.f.cur_pos.x = 10.5;
    unsafe { t::fn_801577B4(gp) };
    assert_eq!(take(), "cms 0x15e 0 0 1 0,8006EBA4");
    assert_eq!(r.f.self_vel.x, 0.0);
    // Init_8015764C installs the callback (the cb.rs wrapper forwards here).
    t::ftCh_Init_8015764C(gp);
    assert!(r.f.accessory4_cb.is_some());
    take();
    r.f.cur_pos.x = 11.0;
    unsafe { (r.f.accessory4_cb.unwrap())(gp) };
    assert_eq!(take(), "cms 0x15e 0 0 1 0,8006EBA4");
    // Walk2_Anim: frames remaining -> nothing; done -> tag cancel entry.
    unsafe { FRAMES = 1 };
    t::ftCh_Walk2_Anim(gp);
    assert_eq!(take(), "");
    unsafe { FRAMES = 0 };
    t::ftCh_Walk2_Anim(gp);
    assert_eq!(take(), "cms 0x184 0 0 1 0");
}
