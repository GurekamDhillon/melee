//! tu::ftpeach smoke test through recording hooks.
#![allow(static_mut_refs)]
use melee_chara::stubs as st;
use melee_chara::tu::ftpeach as t;
use melee_types::*;
static mut LOG: std::vec::Vec<String> = std::vec::Vec::new();
fn log(s: String) {
    unsafe { LOG.push(s) }
}
#[test]
fn smoke() {
    let mut f: Box<Fighter> = Box::new(Fighter::zeroed());
    let mut g: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    g.user_data = &mut *f as *mut Fighter as *mut _;
    let gp: *mut HSD_GObj = &mut *g;
    unsafe {
        st::FTPARTS_80074A4C = Some(|_, i, v| log(format!("{i}:{v}")));
    }
    f.costume_id = 1;
    t::ftPe_Init_OnDeath(gp);
    assert_eq!(unsafe { f.u.pe.has_float }, 1);
    assert_eq!(unsafe { LOG.join(",") }, "0:0,2:0,3:-1,4:0,1:-1,5:0,6:-1");
    unsafe {
        LOG.clear();
        t::ext::FTPARTS_80074A74 = Some(|_, _| -1);
    }
    t::ftPe_Init_8011B93C(gp);
    assert_eq!(unsafe { LOG.join(",") }, "2:0");
}
