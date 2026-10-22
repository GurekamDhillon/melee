//! Smoke test of tu::ftmasterhandbackairplane1 (no float math to check against the C). The
//! frames-done path calls the native ftBossLib_8015C208 (needs a fighter list), so only the
//! waiting path and Coll are exercised here.
#![allow(static_mut_refs)]
#[path = "common_mh/mod.rs"]
mod common;
use common::*;
use melee_chara::tu::ftmasterhandbackairplane1 as t;

#[test]
fn smoke() {
    let mut r = Rig::new();
    let gp = r.gp();
    hook_stubs(1);
    t::ftMh_BackAirplane1_Anim(gp);
    assert_eq!(take_log(), "");
    t::ftMh_BackAirplane1_Coll(gp);
    // re-exports resolve
    let _: fn(*mut melee_types::HSD_GObj) = t::ftMh_BackAirplane1_Phys;
    let _: fn(*mut melee_types::HSD_GObj) = t::ftMh_BackAirplane1_IASA;
    let _: fn(*mut melee_types::HSD_GObj) = t::ftMh_BackAirplane2_Anim;
}
