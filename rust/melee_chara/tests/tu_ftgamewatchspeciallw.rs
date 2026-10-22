//! tu::ftgamewatchspeciallw smoke test: Oil Panic item lifecycle, the bucket model, Enter, the
//! loop restart, the fill / release states and the collision hand-offs through recording hooks
//! (release damage and the IASA decisions are compared with the C in tests/tu_ftgamewatch.rs).
//! One #[test]: hooks are globals.
#![allow(static_mut_refs, unused_unsafe)]
mod gw_common;
use gw_common::*;
use melee_chara::stubs as st;
use melee_chara::tu::ftgamewatchspeciallw as t;
use melee_types::*;

const FLAGS1: u32 = (Ft_MF_KeepGfx
    | Ft_MF_SkipHit
    | Ft_MF_SkipMatAnim
    | Ft_MF_SkipColAnim
    | Ft_MF_UpdateCmd
    | Ft_MF_SkipItemVis
    | Ft_MF_Unk19
    | Ft_MF_SkipModelPartVis
    | Ft_MF_SkipModelFlags
    | Ft_MF_Unk27) as u32;

#[test]
fn oil_panic() {
    install_common();
    unsafe {
        t::ext::IT_802C7D60 = Some(|_, pos, kind, facing| unsafe {
            log(format!("spawn {} {} {} kind {kind} facing {facing}", (*pos).x, (*pos).y, (*pos).z));
            fake(0x700)
        });
        t::ext::IT_802C7E94 = Some(|i| log(format!("destroy {:x}", i as usize)));
        t::ext::IT_802C7EE0 = Some(|i| log(format!("enter_hitlag {:x}", i as usize)));
        t::ext::IT_802C7F00 = Some(|i| log(format!("exit_hitlag {:x}", i as usize)));
        t::ext::FTCOLL_CREATEABSORBHIT = Some(|_, _| log("absorb"));
        t::ext::FTCOLL_8007ABD0 = Some(|h, dmg, _| unsafe { log(format!("hit {} dmg {dmg}", (*h).state)) });
        st::FTPARTS_80074B0C = Some(|_, idx, val| log(format!("part {idx} {val}")));
    }
    let mut w = World::new();
    let gp = w.gp();
    w.f.facing_dir = 1.0;

    // Item lifecycle.
    t::ftGw_SpecialLw_ItemPanicSetup(gp);
    assert_eq!(take(), "spawn 1 2 3 kind 0 facing 1");
    assert_eq!(unsafe { w.f.u.gw.x2268_panicGObj }, fake(0x700));
    assert!(w.f.death2_cb.is_some() && w.f.take_dmg_cb.is_some());
    assert!(w.f.pre_hitlag_cb.is_some() && w.f.post_hitlag_cb.is_some() && w.f.accessory4_cb.is_none());
    t::ftGw_SpecialLw_ItemPanicSetup(gp); // already out: callbacks only
    assert_eq!(take(), "");
    t::ftGw_SpecialLw_ItemPanicEnterHitlag(gp);
    t::ftGw_SpecialLw_ItemPanicExitHitlag(gp);
    t::ftGw_SpecialLw_ItemPanicRemove(gp);
    assert_eq!(take(), "enter_hitlag 700,exit_hitlag 700,destroy 700,exit_hitlag 700");
    assert!(unsafe { w.f.u.gw.x2268_panicGObj }.is_null());
    assert!(w.f.death2_cb.is_none() && w.f.take_dmg_cb.is_none());

    // Bucket model per charge level.
    for (charge, want) in [
        (ftGw_Panic_Empty, "part 5 2,part 6 -1,part 7 -1,part 8 -1"),
        (ftGw_Panic_Low, "part 5 2,part 6 0,part 7 -1,part 8 -1"),
        (ftGw_Panic_Mid, "part 5 2,part 6 0,part 7 0,part 8 -1"),
        (ftGw_Panic_Full, "part 5 2,part 6 0,part 7 0,part 8 0"),
        (9, "part 5 2,part 6 0,part 7 0,part 8 0"),
    ] {
        unsafe { w.f.u.gw.x2238_panicCharge = charge as i32 };
        t::ftGw_SpecialLw_UpdateBucketModel(gp);
        assert_eq!(take(), want, "charge {charge}");
    }

    // Enter: below Full starts the loop; at Full releases the oil straight away.
    w.f.self_vel = Vec { x: 8.0, y: 3.0, z: 0.0 };
    w.f.mv_gw_reset();
    unsafe { w.f.u.gw.x2238_panicCharge = ftGw_Panic_Low as i32 };
    t::ftGw_SpecialLw_Enter(gp);
    assert_eq!(take(), format!("ms {} 0 0,anim", ftGw_MS_SpecialLw));
    assert_eq!(w.f.self_vel.y, 0.0);
    w.da.x64_GAMEWATCH_PANIC_MOMENTUM_PRESERVE = 2.0;
    t::ftGw_SpecialAirLw_Enter(gp);
    assert_eq!(take(), format!("ms {} 0 0,anim", ftGw_MS_SpecialAirLw));
    assert_eq!(w.f.self_vel.x, 4.0);
    unsafe {
        w.f.u.gw.x2238_panicCharge = ftGw_Panic_Full as i32;
        w.f.u.gw.x223C_panicDamage = 100;
    }
    w.da.x78_GAMEWATCH_PANIC_DAMAGE_MUL = 0.5;
    w.da.x74_GAMEWATCH_PANIC_DAMAGE_ADD = 3.0;
    t::ftGw_SpecialLw_Enter(gp);
    let l = take();
    assert!(l.starts_with(&format!("ms {} 0 0,anim,part 5 2", ftGw_MS_SpecialLwShoot)), "{l}");
    assert_eq!(w.f.cmd_vars[1], 53);
    assert_eq!(unsafe { (w.f.u.gw.x2238_panicCharge, w.f.u.gw.x223C_panicDamage) }, (0, 0));
    assert!(w.f.accessory4_cb.is_some());

    // Loop animation: frame 38 without a release restarts the loop at frame 4 (5 - 1).
    w.f.cur_anim_frame = 38.0;
    unsafe {
        w.f.mv.gw.SpecialLw.isRelease = 0;
        FRAMES_REMAINING = 1;
    }
    w.f.cmd_vars[0] = 0;
    t::ftGw_SpecialLw_Anim(gp);
    assert_eq!(take(), format!("ms {} {:x} 4,anim", ftGw_MS_SpecialLw, FLAGS1));
    // first command frame creates the absorb hit once.
    w.f.cur_anim_frame = 3.0;
    w.f.cmd_vars[0] = 1;
    t::ftGw_SpecialAirLw_Anim(gp);
    let l = take();
    assert!(l.starts_with("absorb,part 5 2"), "{l}");
    assert_eq!(w.f.cmd_vars[0], 2);
    unsafe { FRAMES_REMAINING = 0 };
    t::ftGw_SpecialAirLw_Anim(gp);
    assert!(take().ends_with("fall"));

    // Fill: finished animation at Full -> wait; otherwise back into the loop at frame 4.
    unsafe { FRAMES_REMAINING = 1 };
    t::ftGw_SpecialLwCatch_Anim(gp);
    assert_eq!(take(), "");
    unsafe { FRAMES_REMAINING = 0 };
    unsafe { w.f.u.gw.x2238_panicCharge = ftGw_Panic_Full as i32 };
    t::ftGw_SpecialLwCatch_Anim(gp);
    t::ftGw_SpecialAirLwCatch_Anim(gp);
    assert_eq!(take(), "wait,fall");
    unsafe { w.f.u.gw.x2238_panicCharge = ftGw_Panic_Low as i32 };
    w.f.cmd_vars[0] = 0;
    t::ftGw_SpecialAirLwCatch_Anim(gp);
    assert_eq!(take(), format!("ms {} {:x} 4,anim", ftGw_MS_SpecialAirLw, FLAGS1));

    // Absorb think: tally hits / damage, pick the catch state by ground / air.
    unsafe { w.f.u.gw.x2238_panicCharge = 0 };
    w.f.AbsorbAttr.x1A48_hitsTaken = 1;
    w.f.AbsorbAttr.x1A44_damageTaken = 7;
    unsafe { w.f.u.gw.x223C_panicDamage = 5 };
    w.f.ground_or_air = GA_Ground;
    t::ftGw_SpecialLw_AbsorbThink_DecideAction(gp);
    assert!(take().starts_with(&format!("ms {} 0 0,part 5 2", ftGw_MS_SpecialLwCatch)));
    w.f.ground_or_air = GA_Air;
    t::ftGw_SpecialLw_AbsorbThink_DecideAction(gp);
    assert!(take().starts_with(&format!("ms {} 0 0,part 5 2", ftGw_MS_SpecialAirLwCatch)));
    assert_eq!(unsafe { (w.f.u.gw.x2238_panicCharge, w.f.u.gw.x223C_panicDamage) }, (2, 19));

    // Release animation hands the stored damage to the enabled hit capsules only.
    w.f.cmd_vars[1] = 42;
    w.f.x914[1].state = HitCapsule_Enabled;
    w.f.x914[3].state = HitCapsule_Enabled;
    unsafe { FRAMES_REMAINING = 1 };
    t::ftGw_SpecialLwShoot_Anim(gp);
    assert_eq!(take(), "hit 1 dmg 42,hit 1 dmg 42");

    // Collision hand-offs.
    w.f.cmd_vars[0] = 0;
    t::ftGw_SpecialLwShoot_Coll(gp); // ft_80082708 stand-in returns 0: leaves the ground
    let l = take();
    assert!(l.contains(&format!("ms {} ", ftGw_MS_SpecialAirLwShoot)) && l.contains("part 5 2"), "{l}");
    assert!(w.f.accessory4_cb.is_some());
    t::ftGw_SpecialAirLwShoot_Coll(gp); // ft_80081D0C stand-in returns 1: lands
    let l = take();
    assert!(l.contains(&format!("ms {} ", ftGw_MS_SpecialLwShoot)), "{l}");
    t::ftGw_SpecialLw_Coll(gp);
    t::ftGw_SpecialAirLw_Coll(gp);
    let l = take();
    assert!(l.contains(&format!("ms {} ", ftGw_MS_SpecialAirLw)) && l.contains(&format!("ms {} ", ftGw_MS_SpecialLw)), "{l}");
}

trait ResetMv {
    fn mv_gw_reset(&mut self);
}
impl ResetMv for Fighter {
    fn mv_gw_reset(&mut self) {
        unsafe {
            self.mv.gw.SpecialLw.isRelease = 1;
            self.mv.gw.SpecialLw.turnFrames = 9;
        }
    }
}
