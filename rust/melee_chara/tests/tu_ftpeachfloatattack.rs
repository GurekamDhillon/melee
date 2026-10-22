//! tu::ftpeachfloatattack smoke test through recording hooks.
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

use melee_chara::tu::ftpeachfloatattack as t;
#[test]
fn smoke() {
    let (mut f, mut g) = mk();
    let gp: *mut HSD_GObj = &mut *g;
    unsafe {
        st::FTANIM_ISFRAMESREMAINING = Some(|_| 0);
        st::FTCO_FALL_ENTER = Some(|_| log("Fall".into()));
        st::FT_80084DB0 = Some(|_| log("84DB0".into()));
    }
    f.u.pe.x4 = 2.0;
    f.facing_dir = 1.0;
    unsafe { f.anon_2210.anon_0.set_throw_flags_b3(1) };
    f.mv.pe.floatattack.x0 = 1;
    t::ftPe_FloatAttackAir_Anim(gp);
    assert_eq!(unsafe { f.u.pe.x4 }, 1.0);
    assert_eq!(f.facing_dir, -1.0);
    t::ftPe_FloatAttackAir_Phys(gp);
    assert_eq!(unsafe { LOG.join(",") }, "Fall,84DB0");
}
