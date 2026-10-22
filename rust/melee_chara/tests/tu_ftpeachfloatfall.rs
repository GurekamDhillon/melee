//! tu::ftpeachfloatfall smoke test through recording hooks.
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

use melee_chara::tu::ftpeachfloatfall as t;
#[test]
fn smoke() {
    let (_f, mut g) = mk();
    let gp: *mut HSD_GObj = &mut *g;
    unsafe {
        st::FTANIM_ISFRAMESREMAINING = Some(|_| 0);
        t::ext::FTCO_FALLAERIAL_ENTER = Some(|_| log("fallaerial".into()));
    }
    t::ftPe_FloatFall_Anim(gp);
    t::ftPe_FloatFall_IASA(gp);
    assert_eq!(unsafe { LOG.join(",") }, "fallaerial");
}
