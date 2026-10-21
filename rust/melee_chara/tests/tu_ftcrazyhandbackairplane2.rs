//! Smoke test of tu::ftcrazyhandbackairplane2 and ::ftcrazyhandbackairplane3.
#![allow(static_mut_refs)]
#[path = "common_ch/mod.rs"]
mod common;
use common::*;
use melee_chara::tu::ftcrazyhandbackairplane2 as t;
use melee_chara::tu::ftcrazyhandbackairplane2::ext;
use melee_chara::tu::ftcrazyhandbackairplane3 as t3;
use melee_types::*;

#[test]
fn smoke() {
    let mut r = Rig::new();
    let mut victim = Rig::new();
    let gp = r.gp();
    let vp = victim.gp();
    hook_stubs(1);
    unsafe {
        ext::FTCH_GRABUNK1_8015B850 = Some(|_, m| log(format!("grab850 {m:#x}")));
        ext::FTCH_GRABUNK1_8015BC88 = Some(|_| log("grabBC88".into()));
        ext::FTCO_800DE2A8 = Some(|_, _| log("de2a8".into()));
        ext::FTCO_800DE7C0 = Some(|_, v, up| log(format!("de7c0 {} {up}", v.is_null())));
    }
    // Nothing pending, frames remain.
    t::ftCh_BackAirplane2_Anim(gp);
    assert_eq!(take_log(), "");
    // Pending with a victim: facing flips, state cleared.
    r.f.cmd_vars[0] = 1;
    r.f.victim_gobj = vp;
    victim.f.dmg.facing_dir_1 = 1.0;
    unsafe { r.f.mv.ch.unk0.x20 = 7 };
    t::ftCh_BackAirplane2_Anim(gp);
    assert_eq!(take_log(), "grab850 0x153,de2a8,de7c0 true false");
    assert_eq!(victim.f.dmg.facing_dir_1, -1.0);
    assert_eq!(r.f.cmd_vars[0], 0);
    assert_eq!(unsafe { r.f.mv.ch.unk0.x20 }, 0);
    // Pending with no victim, animation finished.
    hook_stubs(0);
    r.f.cmd_vars[0] = 1;
    r.f.victim_gobj = core::ptr::null_mut();
    t::ftCh_BackAirplane2_Anim(gp);
    assert_eq!(take_log(), "grab850 0x153,grabBC88");
    t::ftCh_BackAirplane2_Coll(gp);

    // airplane3: no facing flip.
    hook_stubs(1);
    r.f.cmd_vars[0] = 1;
    r.f.victim_gobj = vp;
    victim.f.dmg.facing_dir_1 = 1.0;
    t3::ftCh_BackAirplane3_Anim(gp);
    assert_eq!(take_log(), "grab850 0x153,de2a8,de7c0 true false");
    assert_eq!(victim.f.dmg.facing_dir_1, 1.0);
    hook_stubs(0);
    t3::ftCh_BackAirplane3_Anim(gp);
    assert_eq!(take_log(), "grabBC88");
    t3::ftCh_BackAirplane3_Coll(gp);
}
