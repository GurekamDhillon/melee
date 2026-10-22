//! tu::ftpeachspeciallw smoke test through recording hooks.
#![allow(static_mut_refs)]
use melee_chara::stubs as st;
use melee_types::*;
static mut LOG: std::vec::Vec<String> = std::vec::Vec::new();
fn log(s: String) {
    unsafe { LOG.push(s) }
}
fn mk() -> (Box<Fighter>, Box<HSD_GObj>) {
    let mut f: Box<Fighter> = Box::new(Fighter::zeroed());
    let mut g: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    g.user_data = &mut *f as *mut Fighter as *mut _;
    (f, g)
}

use melee_chara::tu::ftpeachspeciallw as t;
#[test]
fn smoke() {
    let (mut f, mut g) = mk();
    let gp: *mut HSD_GObj = &mut *g;
    unsafe {
        st::FIGHTER_CHANGEMOTIONSTATE = Some(|_, ms, _, _, _, _, _| log(format!("ms {ms}")));
        st::FTANIM_8006EBA4 = Some(|_| {});
        st::FTANIM_ISFRAMESREMAINING = Some(|_| 0);
        st::FT_8008A2BC = Some(|_| log("8A2BC".into()));
        st::FTCO_FALL_ENTER = Some(|_| log("Fall".into()));
    }
    t::ftPe_SpecialLw_Enter(gp);
    assert!(f.accessory4_cb.is_some());
    t::ftPe_SpecialLw_Anim(gp);
    t::ftPe_SpecialAirLw_Anim(gp);
    assert_eq!(unsafe { LOG.join(",") }, format!("ms {},8A2BC,Fall", ftPe_MS_SpecialLw as i32));
}
