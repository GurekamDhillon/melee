//! tu::ftkirbyspecialdonkey (src/melee/ft/kinds/ftKirby/ftkirbyspecialdonkey.c): no fused sites; the
//! only float math (the punch lunge) is a single multiply chain. Smoke test of the entries, the swing
//! animation callbacks and the IASA callbacks through recording hooks.
#![allow(static_mut_refs)]

use melee_chara::stubs as st;
use melee_chara::tu::ftkirbyspecialdonkey as t;
use melee_types::*;

static mut LOG: std::vec::Vec<String> = std::vec::Vec::new();
fn log(s: String) {
    unsafe { LOG.push(s) }
}
fn sn(f: &Fighter) -> ftDonkey_SpecialNVars {
    unsafe { f.mv.dk.specialn }
}
fn xbc(f: &Fighter) -> i32 {
    unsafe { f.u.kb.xBC }
}
fn take() -> String {
    unsafe { core::mem::take(&mut LOG).join(",") }
}

#[test]
fn smoke() {
    let mut f: Box<Fighter> = Box::new(Fighter::zeroed());
    let mut g: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    let da: &'static mut ftKb_DatAttrs = Box::leak(Box::new(ftKb_DatAttrs::zeroed()));
    let mut parts: std::vec::Vec<FighterBone> = (0..4).map(|_| unsafe { core::mem::zeroed() }).collect();
    f.dat_attrs = da as *mut _ as *mut _;
    f.parts = parts.as_mut_ptr();
    g.user_data = &mut *f as *mut Fighter as *mut _;
    let gp: *mut HSD_GObj = &mut *g;
    da.specialn_dk_swings_to_full_charge = 3;
    da.specialn_dk_damage_increase_per_swing = 2;
    da.specialn_dk_grounded_punch_horizontal_velocity = 1.5;
    unsafe {
        st::FIGHTER_CHANGEMOTIONSTATE = Some(|_, msid, flags, a, b, c, _| log(format!("cms {msid} {flags} {a} {b} {c}")));
        st::FTANIM_8006EBA4 = Some(|_| log("8006EBA4".into()));
        st::FTANIM_ISFRAMESREMAINING = Some(|_| 0);
        st::FT_8008A2BC = Some(|_| log("8A2BC".into()));
        st::FTCO_FALL_ENTER = Some(|_| log("fall".into()));
        st::FTCO_80096900 = Some(|_, a, b, c, m, l| log(format!("96900 {a} {b} {c} {m} {l}")));
        st::FTCOMMON_8007D7FC = Some(|_| log("7D7FC".into()));
        t::ext::EFSYNC_SPAWN = Some(|id, _, args| {
            log(format!("ef {id} {}", args.len()));
            core::ptr::null_mut()
        });
        t::ext::FTCO_800BFFD0 = Some(|_, a, b| log(format!("bffd0 {a} {b}")));
        t::ext::FTCOLL_8007ABD0 = Some(|h, d, _| log(format!("abd0 {} {d}", unsafe { (*h).damage })));
    }

    // Ground entry below full charge: Start state, swing counters cleared, damage trackers -1.
    f.u.kb.xBC = 1;
    f.self_vel.y = 9.0;
    t::ftKb_SpecialNDk_800FF8EC(gp);
    assert_eq!(take(), format!("cms {} 0 0 1 0,7D7FC,8006EBA4", ftKb_MS_DkSpecialNStart));
    assert_eq!((sn(&f).x8, sn(&f).xC), (0, 0));
    assert_eq!((sn(&f).x10, sn(&f).x14), (-1, -1));
    assert_eq!(f.self_vel.y, 0.0);
    assert!(f.death2_cb.is_some() && f.take_dmg_cb.is_some() && f.pre_hitlag_cb.is_some());
    // At full charge: Full state, charge moved into xC.
    f.u.kb.xBC = 3;
    t::ftKb_SpecialNDk_800FFA10(gp);
    assert_eq!(take(), format!("cms {} 0 0 1 0,8006EBA4", ftKb_MS_DkSpecialAirNFull));
    assert_eq!((sn(&f).x8, sn(&f).xC, xbc(&f)), (1, 3, 0));

    // Loop anim: a swing counted on frame 0; reaching the cap clears the hitlag callbacks and leaves.
    f.cur_anim_frame = 0.0;
    f.u.kb.xBC = 2;
    t::ftKb_DkSpecialNLoop_Anim(gp);
    assert_eq!(xbc(&f), 3);
    assert!(f.pre_hitlag_cb.is_none() && f.post_hitlag_cb.is_none());
    assert_eq!(take(), "bffd0 58 0,8A2BC");

    // N_Anim: swing effect once (ground 1193 by ground_or_air), damage ramp, lunge velocity, then exit.
    f.cmd_vars[0] = 1;
    f.mv.dk.specialn.x8 = 0;
    f.mv.dk.specialn.x4 = 0;
    f.mv.dk.specialn.x10 = -1;
    f.mv.dk.specialn.x14 = -1;
    f.mv.dk.specialn.xC = 2;
    f.ground_or_air = GA_Ground;
    f.facing_dir = -1.0;
    f.x914[0].state = 1;
    f.x914[0].damage = 12.0;
    f.x914[1].damage = 5.0;
    f.u.kb.xBC = 4;
    t::ftKb_DkSpecialN_Anim(gp);
    assert_eq!(take(), "ef 1193 2,abd0 12 16,abd0 5 9,8A2BC");
    assert_eq!(sn(&f).x4, 2);
    assert_eq!(f.gr_vel, -1.0 * (1.5 * 2.0));
    assert_eq!((xbc(&f), sn(&f).x8), (0, 2));
    // AirN_Anim at freefall toggle 0 falls, otherwise enters landing-fall with the toggle.
    f.ground_or_air = GA_Air;
    f.mv.dk.specialn.x8 = 1;
    da.specialn_dk_freefall_toggle = 0.0;
    t::ftKb_DkSpecialAirN_Anim(gp);
    assert_eq!(take(), "ef 1194 2,abd0 12 16,abd0 5 9,fall");
    da.specialn_dk_freefall_toggle = 20.0;
    f.cmd_vars[0] = 0;
    t::ftKb_DkSpecialAirNFull_Anim(gp);
    assert_eq!(take(), "96900 1 0 1 1 20");

    // IASA: B releases the punch, LR queues the cancel which fires on frame 0.
    f.u.kb.xBC = 2;
    f.input.pressed_buttons = HSD_PAD_B as u32;
    t::ftKb_DkSpecialAirNLoop_IASA(gp);
    assert_eq!(sn(&f).xC, 2);
    assert_eq!(xbc(&f), 0);
    assert_eq!(take(), format!("cms {} 0 0 1 0,8006EBA4", ftKb_MS_DkSpecialAirN));
    f.input.pressed_buttons = HSD_PAD_LR;
    f.cur_anim_frame = 0.0;
    t::ftKb_DkSpecialAirNLoop_IASA(gp);
    assert_eq!(sn(&f).x0, 1);
    assert_eq!(take(), format!("cms {} 0 0 1 0", ftKb_MS_DkSpecialAirNCancel));
    drop(parts);
}
