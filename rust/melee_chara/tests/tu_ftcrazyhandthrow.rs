//! Smoke test of tu::ftcrazyhandthrow.
#![allow(static_mut_refs)]
#[path = "common_ch/mod.rs"]
mod common;
use common::*;
use melee_chara::tu::ftcrazyhandthrow as t;
use melee_types::*;

#[test]
fn smoke() {
    let mut r = Rig::new();
    let gp = r.gp();
    hook_stubs(1);
    unsafe {
        // No fighters on the p-link: ftBossLib_GetFighterGObj finds none.
        melee_fighter::hooks::PLINK_HEAD = Some(|_| core::ptr::null_mut());
        melee_chara::stubs::FIGHTER_UNKSETFLAG_8006CFBC = Some(|_| log("setflag".into()));
        t::ext::FTCH_GRABUNK1_8015BC88 = Some(|_| log("bc88".into()));
    }
    r.f.cmd_vars[1] = 5;
    r.f.x1A5C = 1usize as *mut _;
    t::ftCh_Init_8015A560(gp);
    assert_eq!(take_log(), "cms 0x17b 0 0 1 0 true,anim8006EBA4");
    assert_eq!(r.f.cmd_vars[1], 0);
    assert!(r.f.x1A5C.is_null());

    // Anim: frames remain and no Master Hand: nothing happens.
    t::ftCh_Throw_Anim(gp);
    assert_eq!(take_log(), "");
    // Anim: animation done -> flag set, x1A5C cleared, hand to the next state.
    hook_stubs(0);
    r.f.x1A5C = 1usize as *mut _;
    t::ftCh_Throw_Anim(gp);
    assert_eq!(take_log(), "setflag,bc88");
    assert!(r.f.x1A5C.is_null());
    t::ftCh_Throw_Coll(gp);
}
