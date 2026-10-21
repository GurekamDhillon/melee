//! Smoke test of tu::ftcrazyhandgrab (no float math).
#![allow(static_mut_refs)]

use melee_chara::stubs as st;
use melee_chara::tu::ftcrazyhandgrab as t;
use melee_types::*;

static mut LOG: std::vec::Vec<String> = std::vec::Vec::new();
fn log(s: String) {
    unsafe { LOG.push(s) }
}

#[test]
fn smoke() {
    let mut g: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    let gp: *mut HSD_GObj = &mut *g;
    unsafe {
        st::FTANIM_ISFRAMESREMAINING = Some(|_| 1);
        t::ext::FTCH_INIT_8015A030 = Some(|_| log("cancel".into()));
        st::FIGHTER_CHANGEMOTIONSTATE = Some(|_, m, _, _, _, _, _| log(format!("ms {m:x}")));
        st::FTANIM_8006EBA4 = Some(|_| log("anim".into()));
    }
    t::ftCh_Grab_Anim(gp);
    unsafe { st::FTANIM_ISFRAMESREMAINING = Some(|_| 0) };
    t::ftCh_Grab_Anim(gp);
    t::ftCh_Init_80159F40(gp);
    t::ftCh_Grab_Coll(gp);
    assert_eq!(unsafe { LOG.join(",") }, "cancel,ms 176,anim");
}
