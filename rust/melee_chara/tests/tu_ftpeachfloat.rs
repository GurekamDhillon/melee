//! tu::ftpeachfloat smoke test through recording hooks.
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

use melee_chara::tu::ftpeachfloat as t;
#[test]
fn smoke() {
    let (mut f, mut g) = mk();
    let gp: *mut HSD_GObj = &mut *g;
    let da: &'static mut ftPe_DatAttrs = Box::leak(Box::new(unsafe { core::mem::zeroed() }));
    f.dat_attrs = da as *mut _ as *mut _;
    unsafe {
        st::FIGHTER_CHANGEMOTIONSTATE = Some(|_, ms, _, _, _, _, _| log(format!("ms {ms}")));
        t::ext::FT_800831CC = Some(|_, l, c| {
            log(format!("coll {} {}", l.is_some(), c.is_some()));
            false
        });
    }
    // Float_Anim: timer ticking down to zero would call UpdateFloatDir; with timer > 1 it only decrements.
    f.u.pe.x4 = 3.0;
    t::ftPe_Float_Anim(gp);
    assert_eq!(unsafe { f.u.pe.x4 }, 2.0);
    t::ftPe_Float_Coll(gp);
    assert_eq!(unsafe { LOG.join(",") }, "coll true true");
}
