//! Smoke test of tu::ftcrazyhandsqueeze (hooks record the calls).
#![allow(static_mut_refs)]
mod ch_common;
use ch_common::*;
use melee_chara::tu::ftcrazyhandsqueeze as t;
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
    t::ftCh_Squeeze_Coll(gp);
    t::ftCh_Squeeze_Phys(gp);
    assert_eq!(take(), "85134");
    // Init: finds Master Hand, enters 0x17A.
    r.f.cmd_vars[1] = 5;
    t::ftCh_Init_8015A3F4(gp);
    assert_eq!(take(), "cms 0x17a 0 0 1 0,8006EBA4");
    assert_eq!(r.f.cmd_vars[1], 0);
    assert_eq!(r.f.x1A5C, unsafe { MASTER });
    // Anim: frames remain, master not in squeeze states: nothing.
    unsafe { FRAMES = 1 };
    t::ftCh_Squeeze_Anim(gp);
    assert_eq!(take(), "");
    // Anim: animation over -> flag, clear x1A5C, GrabUnk1_8015BC88 (enters 0x184).
    unsafe { FRAMES = 0 };
    r.da.x18 = 1.5;
    r.da.x1C = 2.5;
    t::ftCh_Squeeze_Anim(gp);
    assert!(r.f.x1A5C.is_null());
    assert_eq!(take(), "setflag,cms 0x184 0 0 1 0");
    unsafe {
        assert_eq!((r.f.mv.ch.unk0.xC.x, r.f.mv.ch.unk0.xC.y), (1.5, 2.5));
        assert_eq!(r.f.u.mh.x2258, 0x184);
    }
}
