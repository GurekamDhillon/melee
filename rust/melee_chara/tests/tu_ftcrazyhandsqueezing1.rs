//! Smoke test of tu::ftcrazyhandsqueezing1 (hooks record the calls).
#![allow(static_mut_refs)]
mod ch_common;
use ch_common::*;
use melee_chara::tu::ftcrazyhandsqueezing1 as t;
use melee_types::*;

static mut MASTER: *mut HSD_GObj = core::ptr::null_mut();

#[test]
fn smoke() {
    let mut r = Rig::new();
    let gp = r.gp();
    let mut mf: Box<Fighter> = Box::new(Fighter::zeroed());
    mf.kind = Ft_Kind_MasterH;
    let mut mg: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    mg.user_data = &mut *mf as *mut Fighter as *mut _;
    unsafe {
        MASTER = &mut *mg;
        melee_fighter::hooks::PLINK_HEAD = Some(|_| MASTER);
    }
    t::ftCh_Squeezing1_Coll(gp);
    t::ftCh_Squeezing1_Phys(gp);
    assert_eq!(take(), "85134");
    t::ftCh_Init_8015A2B0(gp);
    assert_eq!(take(), "cms 0x179 0 0 1 0,8006EBA4");
    assert_eq!(r.f.x1A5C, unsafe { MASTER });
    unsafe { FRAMES = 1 };
    t::ftCh_Squeezing1_Anim(gp);
    assert_eq!(take(), "");
    assert_eq!(r.f.x1A5C, unsafe { MASTER });
    // Master Hand reports the x221F_b3 flag -> leave the squeeze.
    mf.set_x221F_b3(1);
    t::ftCh_Squeezing1_Anim(gp);
    assert!(r.f.x1A5C.is_null());
    assert_eq!(take(), "setflag,cms 0x184 0 0 1 0");
}
