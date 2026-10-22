//! Smoke test of tu::ftmasterhandtagcrush (hooks record the calls).
#![allow(static_mut_refs)]
mod mh_common;
use mh_common::*;
use melee_chara::tu::ftmasterhandtagcrush as t;
use melee_types::*;

static mut CRAZY: *mut HSD_GObj = core::ptr::null_mut();

#[test]
fn smoke() {
    let mut r = Rig::new();
    let gp = r.gp();
    // Crazy Hand on the p-link, in a non-squeeze motion.
    let mut cf: Box<Fighter> = Box::new(Fighter::zeroed());
    cf.kind = Ft_Kind_CrezyH;
    let mut cg: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    cg.user_data = &mut *cf as *mut Fighter as *mut _;
    unsafe {
        CRAZY = &mut *cg;
        melee_fighter::hooks::PLINK_HEAD = Some(|_| CRAZY);
    }
    t::ftMh_TagCrush_Phys(gp);
    t::ftMh_TagCrush_Coll(gp);
    assert_eq!(take(), "85134");
    t::ftMh_TagCrush_IASA(gp);
    // 382_801552F8: Crazy Hand is not in 0x181/0x182 -> ftCh_Init_8015A3F4 runs on it (0x17A).
    r.f.cmd_vars[1] = 4;
    t::ftMh_MS_382_801552F8(gp);
    assert_eq!(r.f.cmd_vars[1], 0);
    assert_eq!(r.f.x1A5C, unsafe { CRAZY });
    assert_eq!(take(), "cms 0x17a 0 0 1 0,8006EBA4,cms 0x17f 0 0 1 0,8006EBA4");
    // Anim: frames remain and Crazy Hand not in the cancel states: nothing.
    unsafe { FRAMES = 1 };
    t::ftMh_TagCrush_Anim(gp);
    assert_eq!(take(), "");
    // Animation over: flag, clear x1A5C, back to the wait states (80151018 -> Wait1_2).
    unsafe { FRAMES = 0 };
    r.f.u.mh.x2258 = 0;
    t::ftMh_TagCrush_Anim(gp);
    assert!(r.f.x1A5C.is_null());
    assert_eq!(take(), "setflag,cms 0x185 0 0 1 0,8006EBA4");
}
