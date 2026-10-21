//! Smoke test for tu::ftdonkeyheavylanding (stateful; no fused sites) through recording hooks.
#![allow(static_mut_refs)]

use melee_chara::stubs as st;
use melee_chara::tu::ftdonkeyheavylanding as t;
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
        t::ext::FTCO_8008E984 = Some(|_| false);
        t::ext::FTCO_SCALEBY154 = Some(|x| x * 2.0);
        t::ext::FTCO_8008D8E8 = Some(|x| x as i32);
        t::ext::FTCO_8008DCE0 = Some(|_, m, fd| log(format!("dce0 {m} {fd}")));
        t::ext::ITEM_8026ABD8 = Some(|_, v, f| log(format!("item {} {} {} {f}", (*v).x, (*v).y, (*v).z)));
        t::ext::FTCO_8008E908 = Some(|_, f| log(format!("e908 {f}")));
        t::ext::FTCO_8008F744 = Some(|_| log("f744".into()));
        t::ext::FTCO_DAMAGE_PHYS = Some(|_| log("dmgphys".into()));
        st::FIGHTER_CHANGEMOTIONSTATE = Some(|_, ms, fl, _, sp, _, _| log(format!("ms {ms} {fl:x} {sp}")));
        st::FTANIM_SETANIMRATE = Some(|_, r| log(format!("rate {r}")));
        st::FTCOMMON_8007D7FC = Some(|_| log("7D7FC".into()));
    }
    // kb_applied * 2 as an int: 1.0 -> 2 (< 3) -> doSomething; 2.0 -> 4 -> drop the item.
    f.dmg.kb_applied = 1.0;
    t::ftDk_MS_349_800E06D8(gp);
    assert_eq!(take(), ["dce0 350 0"]);
    f.dmg.kb_applied = 2.0;
    t::ftDk_MS_349_800E06D8(gp);
    assert_eq!(take(), ["item 0 0 0 1", "e908 0"]);
    unsafe { t::ext::FTCO_8008E984 = Some(|_| true) };
    t::ftDk_MS_349_800E06D8(gp);
    assert_eq!(take(), ["dce0 350 0"]);
    // Wait2 anim: the x221C_b6 flag suppresses the transition; otherwise air -> turn, ground -> wait.
    f.set_x221C_b6(1);
    t::ftDk_HeavyWait2_Anim(gp);
    assert_eq!(take(), ["f744"]);
    f.set_x221C_b6(0);
    f.ground_or_air = GA_Air;
    t::ftDk_HeavyWait2_Anim(gp);
    assert_eq!(take(), ["f744", "ms 347 1 1", "rate 0"]);
    f.ground_or_air = GA_Ground;
    t::ftDk_HeavyWait2_Anim(gp);
    assert_eq!(take(), ["f744", "ms 341 0 1"]);
    t::ftDk_HeavyWait2_Phys(gp);
    assert_eq!(take(), ["dmgphys"]);
}
