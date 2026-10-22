//! tu::ftgamewatchattackair smoke test: the Parachute / Turtle / Sparky items (spawn, landing,
//! hitlag, removal), the aerial dispatch on the stick, and the landing-state bookkeeping through
//! recording hooks (no float arithmetic in the TU). One #[test]: hooks are globals.
#![allow(static_mut_refs, unused_unsafe)]
mod gw_common;
use gw_common::*;
use melee_chara::tu::ftgamewatchattackair as t;
use melee_fighter::hooks as fh;
use melee_fighter::tu::ftCo_AttackAir::ext::hook as ah;
use melee_types::*;

#[test]
fn aerial_items() {
    install_common();
    unsafe {
        t::ext::IT_802C6C38 = Some(|_, pos, part, facing| unsafe {
            log(format!("para {} {} {} part {part} facing {facing}", (*pos).x, (*pos).y, (*pos).z));
            fake(0x10)
        });
        t::ext::IT_802C6F40 = Some(|_, _, part, _| {
            log(format!("turtle part {part}"));
            fake(0x20)
        });
        t::ext::IT_802C720C = Some(|_, _, part, _| {
            log(format!("sparky part {part}"));
            fake(0x30)
        });
        t::ext::IT_802C6D6C = Some(|i| log(format!("para_destroy {:x}", i as usize)));
        t::ext::IT_802C6DB8 = Some(|i| log(format!("para_enter {:x}", i as usize)));
        t::ext::IT_802C6DD8 = Some(|i| log(format!("para_exit {:x}", i as usize)));
        t::ext::IT_802C6E50 = Some(|i| log(format!("para_land {:x}", i as usize)));
        t::ext::IT_802C7074 = Some(|i| log(format!("turtle_destroy {:x}", i as usize)));
        t::ext::IT_802C70C0 = Some(|i| log(format!("turtle_enter {:x}", i as usize)));
        t::ext::IT_802C70E0 = Some(|i| log(format!("turtle_exit {:x}", i as usize)));
        t::ext::IT_802C7158 = Some(|i| log(format!("turtle_land {:x}", i as usize)));
        t::ext::IT_802C7340 = Some(|i| log(format!("sparky_destroy {:x}", i as usize)));
        t::ext::IT_802C738C = Some(|i| log(format!("sparky_enter {:x}", i as usize)));
        t::ext::IT_802C73AC = Some(|i| log(format!("sparky_exit {:x}", i as usize)));
        t::ext::IT_802C7424 = Some(|i| log(format!("sparky_land {:x}", i as usize)));
        ah::ftCo_800DF478 = Some(|_| false);
        ah::ftCo_GetLStickAngle = Some(|_| 0.0);
        ah::Fighter_ChangeMotionState = Some(|_, ms, fl, start, _, _, _| log(format!("ms {ms} {fl:x} {start}")));
        ah::ftAnim_8006EBA4 = Some(|_| log("anim"));
    }
    let mut w = World::new();
    let gp = w.gp();
    w.f.facing_dir = 1.0;

    // Each setup spawns its item once, then falls through to OnLand when it exists.
    t::ftGw_AttackAirN_ItemParachuteSetup(gp);
    t::ftGw_AttackAirN_ItemTurtleSetup(gp);
    t::ftGw_AttackAirN_ItemSparkySetup(gp);
    assert_eq!(
        take(),
        format!("para 1 2 3 part {FtPart_TransN} facing 1,turtle part {FtPart_LShoulderN},sparky part {FtPart_LHandN}")
    );
    unsafe {
        assert_eq!(w.f.u.gw.x2258_parachuteGObj, fake(0x10));
        assert_eq!(w.f.u.gw.x225C_turtleGObj, fake(0x20));
        assert_eq!(w.f.u.gw.x2260_sparkyGObj, fake(0x30));
    }
    assert!(w.f.death2_cb.is_some() && w.f.take_dmg_cb.is_some());
    assert!(w.f.pre_hitlag_cb.is_some() && w.f.post_hitlag_cb.is_some() && w.f.accessory4_cb.is_none());

    // Hitlag applies to all three.
    t::ftGw_AttackAirN_EnterItemHitlag(gp);
    t::ftGw_AttackAirN_ExitItemHitlag(gp);
    assert_eq!(take(), "para_enter 10,turtle_enter 20,sparky_enter 30,para_exit 10,turtle_exit 20,sparky_exit 30");

    // OnLand: the item is told to land only in the LandingAirN state (the turtle's check is
    // deliberately the same state in the C); the callbacks are re-installed and accessory cleared.
    w.f.accessory4_cb = Some(t::cb_ftGw_AttackAirN_ItemParachuteSetup);
    w.f.motion_id = ftGw_MS_AttackAirN as FtMotionId;
    t::ftGw_AttackAirN_ItemParachuteOnLand(gp);
    assert_eq!(take(), "para_exit 10,turtle_exit 20,sparky_exit 30");
    assert!(w.f.accessory4_cb.is_none());
    w.f.motion_id = ftGw_MS_LandingAirN as FtMotionId;
    t::ftGw_AttackAirN_ItemParachuteOnLand(gp);
    t::ftGw_AttackAirN_ItemTurtleOnLand(gp);
    t::ftGw_AttackAirN_ItemSparkyOnLand(gp);
    let l = take();
    assert!(l.contains("para_land 10") && l.contains("turtle_land 20") && l.contains("sparky_land 30"), "{l}");

    // Remove destroys the item, ends its hitlag, clears the pointer and both removal callbacks.
    t::ftGw_AttackAirN_ItemParachuteRemove(gp);
    t::ftGw_AttackAirN_ItemTurtleRemove(gp);
    t::ftGw_AttackAirN_ItemSparkyRemove(gp);
    let l = take();
    assert!(l.contains("para_destroy 10") && l.contains("turtle_destroy 20") && l.contains("sparky_destroy 30"), "{l}");
    unsafe {
        assert!(w.f.u.gw.x2258_parachuteGObj.is_null());
        assert!(w.f.u.gw.x225C_turtleGObj.is_null());
        assert!(w.f.u.gw.x2260_sparkyGObj.is_null());
    }
    assert!(w.f.death2_cb.is_none() && w.f.take_dmg_cb.is_none());
    t::ftGw_AttackAirN_ItemParachuteRemove(gp);
    assert_eq!(take(), "");

    // Dispatch on the stick: centred = N, back = B (own handlers), forward = the shared one.
    let cd: &'static mut ftCommonData = Box::leak(Box::new(unsafe { core::mem::zeroed() }));
    cd.xDC = 0.5;
    cd.xE0 = 0.5;
    cd.x20_radians = 0.7;
    unsafe { fh::P_FT_COMMON_DATA = &*cd };
    w.f.input.lstick[0].x = 0.0;
    t::ftGw_AttackAirN_DecideAction(gp);
    assert_eq!(take(), format!("ms {} {:x} 0,anim", ftGw_MS_AttackAirN, Ft_MF_KeepFastFall));
    assert!(w.f.accessory4_cb.is_some());
    w.f.input.lstick[0].x = -1.0;
    t::ftGw_AttackAirN_DecideAction(gp);
    assert_eq!(take(), format!("ms {} {:x} 0,anim", ftGw_MS_AttackAirB, Ft_MF_KeepFastFall));
    w.f.input.lstick[0].x = 1.0;
    t::ftGw_AttackAirN_DecideAction(gp);
    assert_eq!(take(), format!("ms {} {:x} 0,anim", ftCo_MS_AttackAirF, Ft_MF_KeepFastFall));
    unsafe { ah::ftCo_GetLStickAngle = Some(|_| 1.0) };
    w.f.input.lstick[0].x = 0.0;
    w.f.input.lstick[0].y = 1.0;
    t::ftGw_AttackAirN_DecideAction(gp);
    assert_eq!(take(), format!("ms {} {:x} 0,anim", ftGw_MS_AttackAirHi, Ft_MF_KeepFastFall));
}
