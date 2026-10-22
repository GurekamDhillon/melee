//! Smoke test of tu::ftmasterhandwait11 (the two states copy the attribute positions).
#![allow(static_mut_refs)]
mod mh_common;
use mh_common::*;
use melee_chara::tu::ftmasterhandwait11 as t;
use melee_types::*;

#[test]
fn smoke() {
    let mut r = Rig::new();
    let gp = r.gp();
    r.da.x118_pos = Vec2 { x: 1.5, y: 2.5 };
    r.da.x30_pos2 = Vec2 { x: 3.5, y: 4.5 };
    t::ftMh_MS_375_80154C78(gp);
    assert_eq!(take(), "cms 0x178 0 0 1 0,8006EBA4");
    unsafe { assert_eq!((r.f.mv.mh.unk0.xC.x, r.f.mv.mh.unk0.xC.y), (1.5, 2.5)) };
    t::ftMh_MS_378_80154CF8(gp);
    assert_eq!(take(), "cms 0x179 0 0 1 0,8006EBA4");
    unsafe { assert_eq!((r.f.mv.mh.unk0.xC.x, r.f.mv.mh.unk0.xC.y), (3.5, 4.5)) };
}
