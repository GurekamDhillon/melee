//! Smoke test of tu::ftmasterhandfingergun3: Anim hand-off to the common wait; the re-exported
//! MS_366 enters BackAirplane1.
#![allow(static_mut_refs)]

use melee_chara::stubs as st;
use melee_chara::tu::ftmasterhandfingergun3 as t;
use melee_types::*;

static mut LOG: std::vec::Vec<String> = std::vec::Vec::new();
static mut FRAMES: i32 = 0;
fn take() -> String {
    unsafe { core::mem::take(&mut LOG).join(",") }
}

#[test]
fn smoke() {
    let mut g: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    let gp: *mut HSD_GObj = &mut *g;
    unsafe {
        st::FIGHTER_CHANGEMOTIONSTATE = Some(|_, m, _, _, _, _, _| LOG.push(format!("ms {m}")));
        st::FTANIM_8006EBA4 = Some(|_| LOG.push("anim".into()));
        st::FTANIM_ISFRAMESREMAINING = Some(|_| FRAMES);
        melee_chara::tu::ftmasterhanddamage0::ext::FTMH_MS_389_80151018 = Some(|_| LOG.push("wait".into()));
    }
    unsafe { FRAMES = 1 };
    t::ftMh_FingerGun3_Anim(gp);
    assert_eq!(take(), "");
    unsafe { FRAMES = 0 };
    t::ftMh_FingerGun3_Anim(gp);
    assert_eq!(take(), "wait");
    t::ftMh_FingerGun3_Coll(gp);
    t::ftMh_MS_366_80153820(gp);
    assert_eq!(take(), "ms 367,anim");
}
