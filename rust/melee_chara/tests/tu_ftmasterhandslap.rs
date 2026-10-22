//! Smoke test of tu::ftmasterhandslap.
#![allow(static_mut_refs)]
mod mh_common;
use mh_common::*;
use melee_chara::tu::ftmasterhandslap as t;

#[test]
fn smoke() {
    let mut r = Rig::new();
    let gp = r.gp();
    t::ftMh_Slap_Coll(gp);
    t::ftMh_Slap_Anim(gp);
    assert_eq!(take(), "");
    unsafe { FRAMES = 0 };
    t::ftMh_Slap_Anim(gp);
    assert_eq!(take(), "389");
    unsafe { FRAMES = 1 };
    r.da.x50.x = 7.5;
    r.f.cur_pos.y = 1.0;
    r.f.self_vel.x = 3.0;
    t::ftMh_MS_349_80151CA8(gp);
    assert_eq!(take(), "cms 353 0 0 1 0,8006EBA4");
    assert_eq!((r.f.cur_pos.y, r.f.self_vel.x), (7.5, 0.0));
    t::ftMh_Slap_Phys(gp);
    assert_eq!(take(), "85134");
}
