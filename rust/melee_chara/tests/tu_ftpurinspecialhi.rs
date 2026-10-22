//! tu::ftpurinspecialhi smoke test: Sing Enter (stadium flag), Anim, Coll hand-offs, callback clear.
#![allow(static_mut_refs, unused_unsafe)]
mod gw_common;
use gw_common::*;
use melee_chara::tu::ftpurinspecialhi as t;
use melee_types::*;

#[test]
fn sing() {
    install_common();
    unsafe {
        t::ext::GM_8016B1D8 = Some(|| 1);
        t::ext::GRSTADIUM_801D4FF8 = Some(|_| 1);
        melee_chara::stubs::FT_CHECKGROUNDANDLEDGE = Some(|_, _| 0);
        melee_chara::stubs::FTCLIFFCOMMON_80081298 = Some(|_| 0);
    }
    let mut w = World::new();
    let gp = w.gp();
    w.f.facing_dir = 1.0;
    t::ftPr_SpecialHi_Enter(gp);
    assert_eq!(take(), format!("ms {} 0 0,anim", ftPr_MS_SpecialHiR));
    assert_eq!(unsafe { w.f.mv.pr.specialhi.x0 }, 1);
    assert!(w.f.accessory4_cb.is_some());
    unsafe { t::ext::GM_8016B1D8 = Some(|| 0) };
    w.f.facing_dir = -1.0;
    t::ftPr_SpecialAirHi_Enter(gp);
    assert_eq!(take(), format!("ms {} 0 0,anim", ftPr_MS_SpecialAirHiL));
    assert_eq!(unsafe { w.f.mv.pr.specialhi.x0 }, 0);

    unsafe { FRAMES_REMAINING = 0 };
    t::ftPr_SpecialHi_Anim(gp);
    assert_eq!(take(), "wait");
    t::ftPr_SpecialAirHi_Anim(gp);
    assert_eq!(take(), "fall");

    t::ftPr_SpecialHi_Coll(gp);
    assert!(take().contains(&format!("ms {}", ftPr_MS_SpecialAirHiL)));
    t::ftPr_SpecialHi_8013CDD8(gp);
    assert!(take().contains("7D7FC"));
    t::ftPr_SpecialHi_8013CE7C(gp);
    assert!(w.f.accessory4_cb.is_none());
}
