//! Smoke test of tu::ftcrazyhandsqueezing0 (no multiply-add sites; hooks record the calls).
#![allow(static_mut_refs)]
mod ch_common;
use ch_common::*;
use melee_chara::tu::ftcrazyhandsqueezing0 as t;
use melee_types::*;

#[test]
fn smoke() {
    let mut r = Rig::new();
    let gp = r.gp();
    unsafe { FRAMES = 1 };
    t::ftCh_Squeezing0_Anim(gp);
    assert_eq!(take(), "");
    t::ftCh_Squeezing0_Coll(gp);
    t::ftCh_Squeezing0_Phys(gp);
    assert_eq!(take(), "85134");
    r.da.xE4 = 3.0;
    r.da.xE8 = -4.0;
    r.f.self_vel.x = 9.0;
    t::ftCh_Init_8015A184(gp);
    assert_eq!((r.f.cur_pos.x, r.f.cur_pos.y, r.f.self_vel.x), (3.0, -4.0, 0.0));
}
