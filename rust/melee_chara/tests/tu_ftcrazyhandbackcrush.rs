//! Smoke test of tu::ftcrazyhandbackcrush.
#![allow(static_mut_refs)]
#[path = "common_ch/mod.rs"]
mod common;
use common::*;
use melee_chara::tu::ftcrazyhandbackcrush as t;
use melee_types::*;

fn cb(_g: *mut HSD_GObj) {
    log("cb".into());
}
unsafe extern "C" fn cb_c(_g: *mut HSD_GObj) {
    log("cb".into());
}

#[test]
fn smoke() {
    let _ = cb;
    let mut r = Rig::new();
    let gp = r.gp();
    hook_stubs(1);
    t::ftCh_Init_801597F0(gp, Some(cb_c));
    assert_eq!(take_log(), "cms 0x173 0 0 1 0 true,anim8006EBA4");
    assert!(unsafe { r.f.mv.ch.unk0.x4 }.is_some());
    t::ftCh_BackCrush_Anim(gp);
    assert_eq!(take_log(), "");
    hook_stubs(0);
    t::ftCh_BackCrush_Anim(gp);
    assert_eq!(take_log(), "cb");
    t::ftCh_BackCrush_Coll(gp);
}
