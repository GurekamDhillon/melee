//! Smoke test of tu::ftmasterhandbackairplane2.
#![allow(static_mut_refs)]
#[path = "common_mh/mod.rs"]
mod common;
use common::*;
use melee_chara::tu::ftmasterhandbackairplane2 as t;

#[test]
fn smoke() {
    let mut r = Rig::new();
    let gp = r.gp();
    hook_stubs(1);
    // MS_368_80153A64: next state 369, position from the data.
    r.da.x60 = 1.5;
    r.da.x64 = -2.5;
    r.f.self_vel = melee_types::Vec3 { x: 1.0, y: 2.0, z: 3.0 };
    t::ftMh_MS_368_80153A64(gp);
    assert_eq!(take_log(), "cms 369 0 0 1 0 true,anim8006EBA4");
    assert_eq!((r.f.cur_pos.x, r.f.cur_pos.y), (1.5, -2.5));
    assert_eq!((r.f.self_vel.x, r.f.self_vel.y, r.f.self_vel.z), (0.0, 0.0, 0.0));
    t::ftMh_BackAirplane2_Coll(gp);
    let _: fn(*mut melee_types::HSD_GObj) = t::ftMh_BackAirplane2_Phys;
    let _: fn(*mut melee_types::HSD_GObj) = t::ftMh_BackAirplane2_IASA;
}
