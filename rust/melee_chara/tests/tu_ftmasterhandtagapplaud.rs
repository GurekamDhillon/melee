//! Smoke test of tu::ftmasterhandtagapplaud.
#![allow(static_mut_refs)]
mod mh_common;
use mh_common::*;
use melee_chara::tu::ftmasterhandtagapplaud as t;

#[test]
fn smoke() {
    let mut r = Rig::new();
    let gp = r.gp();
    t::ftMh_TagApplaud_Coll(gp);
    t::ftMh_TagApplaud_Phys(gp);
    assert_eq!(take(), "85134");
    // frames remain, no Crazy Hand: nothing
    t::ftMh_TagApplaud_Anim(gp);
    assert_eq!(take(), "");
    // end of animation: flag, clear x1A5C, back to wait
    r.f.x1A5C = gp;
    unsafe { FRAMES = 0 };
    t::ftMh_TagApplaud_Anim(gp);
    assert_eq!(take(), "setflag,389");
    assert!(r.f.x1A5C.is_null());
    // cmd_vars[1] set: the boss lib is poked once (it draws a random number) and the flag cleared
    unsafe {
        melee_fighter::hooks::HSD_RANDI = Some(|_| 2);
        melee_fighter::hooks::PLAY_SFX = Some(|_, id, _, _| log(format!("sfx {id:x}")));
    }
    unsafe { FRAMES = 1 };
    r.f.cmd_vars[1] = 1;
    t::ftMh_TagApplaud_Anim(gp);
    assert_eq!(r.f.cmd_vars[1], 0);
    assert_eq!(take(), "sfx 4e21c");
    t::ftMh_TagApplaud_IASA(gp);
}
