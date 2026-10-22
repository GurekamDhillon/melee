//! Smoke test of tu::ftmasterhandthrow.
#![allow(static_mut_refs)]
mod mh_common;
use mh_common::*;
use melee_chara::tu::ftmasterhandthrow as t;

#[test]
fn smoke() {
    let mut r = Rig::new();
    let gp = r.gp();
    t::ftMh_Throw_Phys(gp);
    t::ftMh_Throw_Coll(gp);
    assert_eq!(take(), "85134");
    unsafe { r.f.cmd_vars[0] = 3 };
    t::ftMh_MS_379_80155014(gp);
    assert_eq!(take(), "cms 380 0 0 1 0,8006EBA4");
    assert_eq!(r.f.cmd_vars[0], 0);
    // IASA: the CPU-slot guard (slot type 0 calls the empty ftBossLib_8015BD20).
    t::ftMh_Throw_IASA(gp);
}
