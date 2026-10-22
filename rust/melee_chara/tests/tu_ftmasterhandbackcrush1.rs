//! Smoke test of tu::ftmasterhandbackcrush1.
#![allow(static_mut_refs)]
#[path = "common_mh/mod.rs"]
mod common;
use common::*;
use melee_chara::tu::ftmasterhandbackcrush1 as t;

unsafe extern "C" fn cb(_: *mut melee_types::HSD_GObj) {}

#[test]
fn smoke() {
    let mut r = Rig::new();
    let gp = r.gp();
    hook_stubs(1);
    t::ftMh_MS_371_801541C8(gp, Some(cb));
    assert_eq!(take_log(), "cms 372 0 0 1 0 true,anim8006EBA4");
    assert_eq!(unsafe { r.f.mv.mh.unk0.x4 }.map(|f| f as usize), Some(cb as usize));
    t::ftMh_BackCrush_Coll(gp);
}
