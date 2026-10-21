//! Smoke test of tu::ftcrazyhandtagrockpaper (no multiply-add sites; hooks record the calls).
#![allow(static_mut_refs)]
mod ch_common;
use ch_common::*;
use melee_chara::tu::ftcrazyhandtagrockpaper as t;
use melee_types::*;

#[test]
fn smoke() {
    let mut r = Rig::new();
    let gp = r.gp();
    t::ftCh_TagRockPaper_Phys(gp);
    t::ftCh_TagRockPaper_Coll(gp);
    t::fn_8015AAC8(gp);
    assert_eq!(take(), "cms 0x180 0 0 1 0");
}
