//! Smoke test for tu::ftdonkeyheavyturn (stateful; no fused sites) through recording hooks.
#![allow(static_mut_refs)]

use melee_chara::stubs as st;
use melee_chara::tu::ftdonkeyheavyturn as t;
use melee_types::*;

static mut LOG: std::vec::Vec<String> = std::vec::Vec::new();
fn log(s: String) {
    unsafe { LOG.push(s) }
}
fn take() -> std::vec::Vec<String> {
    unsafe { core::mem::take(&mut LOG) }
}

#[test]
fn smoke() {
    let mut f: Box<Fighter> = Box::new(Fighter::zeroed());
    let mut g: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    let a: &'static mut ftDonkeyAttributes = Box::leak(Box::new(ftDonkeyAttributes::zeroed()));
    a.motion_state = 341;
    f.x2CC = a;
    g.user_data = &mut *f as *mut Fighter as *mut _;
    let gp: *mut HSD_GObj = &mut *g;
    unsafe {
        t::ext::FTCO_TURN_ANIM_INNER = Some(|_| log("inner".into()));
        t::ext::FTCO_80094EA4 = Some(|_| false);
        t::ext::FTDK_MS_347_800E0378 = Some(|_| {
            log("347".into());
            true
        });
        st::FTANIM_ISFRAMESREMAINING = Some(|_| 1);
        st::FIGHTER_CHANGEMOTIONSTATE = Some(|_, ms, fl, _, sp, _, _| log(format!("ms {ms} {fl:x} {sp}")));
        st::FTANIM_SETANIMRATE = Some(|_, r| log(format!("rate {r}")));
    }
    // Anim: frames remain -> only the common turn step.
    t::ftDk_HeavyTurn_Anim(gp);
    assert_eq!(take(), ["inner"]);
    // Anim: done -> wait state.
    unsafe { st::FTANIM_ISFRAMESREMAINING = Some(|_| 0) };
    f.ground_or_air = GA_Ground;
    t::ftDk_HeavyTurn_Anim(gp);
    assert_eq!(take(), ["inner", "ms 341 0 1"]);
    // IASA: gated on mv.dk.unk5.x0.
    t::ftDk_HeavyTurn_IASA(gp);
    assert!(take().is_empty());
    unsafe { f.mv.dk.unk5.x0 = 1 };
    t::ftDk_HeavyTurn_IASA(gp);
    assert_eq!(take(), ["347"]);
    unsafe { t::ext::FTCO_80094EA4 = Some(|_| true) };
    t::ftDk_HeavyTurn_IASA(gp);
    assert!(take().is_empty());
    // 345_800E0294: msid + 6, flag 1, rate 0; grounded -> ftCommon_8007D5D4.
    t::ftDk_MS_345_800E0294(gp);
    assert_eq!(take(), ["ms 347 1 1", "rate 0"]);
    f.ground_or_air = GA_Air;
    unsafe { t::cb_ftDk_MS_345_800E0294(gp) };
    assert_eq!(take(), ["ms 347 1 1", "rate 0"]);
}
