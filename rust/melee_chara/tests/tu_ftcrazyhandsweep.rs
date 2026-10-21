//! Smoke test of tu::ftcrazyhandsweep (no multiply-add sites; hooks record the calls).
#![allow(static_mut_refs)]
mod ch_common;
use ch_common::*;
use melee_chara::tu::ftcrazyhandsweep as t;
use melee_types::*;

#[test]
fn smoke() {
    let mut r = Rig::new();
    let gp = r.gp();
    t::ftCh_SweepLoop_Coll(gp);
    unsafe { FRAMES = 1 };
    t::ftCh_SweepLoop_Anim(gp);
    assert_eq!(take(), "");
    t::fn_8015746C(gp);
    assert_eq!(take(), "cms 0x15b 0 0 1 0,8006EBA4");
}
