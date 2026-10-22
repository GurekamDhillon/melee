//! Smoke test of tu::ftmasterhandcapturedamagemasterhand (empty callbacks + the round-7 entry).
#![allow(static_mut_refs)]
#[path = "common_mh/mod.rs"]
mod common;
use common::*;
use melee_chara::tu::ftmasterhandcapturedamagemasterhand as t;

#[test]
fn smoke() {
    let mut r = Rig::new();
    let gp = r.gp();
    hook_stubs(1);
    t::ftMh_CaptureDamageMasterHand_IASA(gp);
    t::ftMh_CaptureDamageMasterHand_Phys(gp);
    t::ftMh_CaptureDamageMasterHand_Coll(gp);
    assert_eq!(take_log(), "");
    t::ftMh_CaptureDamageMasterHand_80155C94(gp);
    assert_eq!(take_log(), "cms 329 0 0 1 0 true,anim8006EBA4");
    assert_eq!(r.f.invisible(), 1);
    assert_eq!(r.f.x1A6A, 511);
}
