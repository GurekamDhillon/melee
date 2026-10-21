//! Smoke test for tu::ftcaptainspecialhi (stateful; no fused sites) through recording hooks.
//! One #[test]: the hooks are process globals.
#![allow(static_mut_refs)]

use melee_chara::stubs as st;
use melee_chara::tu::ftcaptainspecialhi as t;
use melee_types::*;

static mut LOG: std::vec::Vec<String> = std::vec::Vec::new();
fn log(s: String) {
    unsafe { LOG.push(s) }
}
fn take() -> std::vec::Vec<String> {
    unsafe { core::mem::take(&mut LOG) }
}

#[test]
fn smoke() {
    let mut f: Box<Fighter> = Box::new(Fighter::zeroed());
    let mut v: Box<Fighter> = Box::new(Fighter::zeroed());
    let mut g: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    let mut vg: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    let mut cd: Box<ftCommonData> = Box::new(unsafe { core::mem::zeroed() });
    let da: &'static mut ftCaptain_DatAttrs = Box::leak(Box::new(ftCaptain_DatAttrs::zeroed()));
    cd.aerial_friction_oob = 0.5;
    unsafe { melee_fighter::hooks::P_FT_COMMON_DATA = &*cd };
    f.dat_attrs = da as *mut _ as *mut _;
    g.user_data = &mut *f as *mut Fighter as *mut _;
    vg.user_data = &mut *v as *mut Fighter as *mut _;
    f.victim_gobj = &mut *vg;
    let gp: *mut HSD_GObj = &mut *g;
    unsafe {
        st::FT_80085134 = Some(|_| {});
        st::FIGHTER_CHANGEMOTIONSTATE = Some(|_, ms, fl, _, _, _, _| log(format!("ms {ms} {fl:x}")));
        st::FTANIM_8006EBA4 = Some(|_| {});
        st::FTANIM_ISFRAMESREMAINING = Some(|_| 0);
        st::FTCO_FALL_ENTER = Some(|_| log("Fall".into()));
        st::FTCO_80096900 = Some(|_, _, _, _, m, lag| log(format!("fallspecial {m} {lag}")));
        t::ext::FTCO_800DB368 = Some(|_, _| log("DB368".into()));
        t::ext::FTCO_800DE2A8 = Some(|_, _| log("DE2A8".into()));
        t::ext::FTCO_800DE7C0 = Some(|_, _, _| log("DE7C0".into()));
        t::ext::FTPARTSETROTY = Some(|_, p, r| log(format!("rot {p} {r}")));
    }

    // Enter: callbacks installed, motion state changed.
    t::ftCa_SpecialHi_Enter(gp);
    assert_eq!(take(), ["ms 353 0"]);
    assert!(f.x21EC.is_some() && f.grab_cb.is_some() && f.grabbed_cb.is_some());
    assert_eq!(f.x1A68, 2);

    // Phys: no stick, |vel.x| over the cap -> quick deaccel by the OOB friction.
    da.specialhi_horz_vel = 1.0;
    f.co_attrs.air_drift_max = 1.0;
    unsafe {
        f.mv.ca.specialhi.vel.x = 5.0;
        f.mv.ca.specialhi.vel.y = -2.0;
    }
    t::ftCa_SpecialHi_Phys(gp);
    unsafe {
        assert_eq!(f.mv.ca.specialhi.vel.x, 4.5);
        assert_eq!(f.mv.ca.specialhi.vel.y, -2.0);
    }
    assert_eq!((f.self_vel.x, f.self_vel.y, f.x74_self_accel.x), (9.5, -4.0, 0.0));

    // IASA: stick past the threshold turns the fighter and sets x2_b1.
    da.specialhi_input_var = 0.5;
    f.cmd_vars[0] = 1;
    f.facing_dir = 1.0;
    f.input.lstick[0].x = -0.9;
    t::ftCa_SpecialAirHi_IASA(gp);
    assert_eq!(f.cmd_vars[0], 0);
    assert_eq!(unsafe { f.mv.ca.specialhi.x2_b1() }, 1);
    assert_eq!(f.facing_dir, -1.0);
    assert_eq!(take(), [format!("rot 0 {}", (-core::f64::consts::FRAC_PI_2) as f32)]);

    // Anim end -> FallSpecial with the up-B attrs.
    da.specialhi_freefall_air_spd_mul = 0.75;
    da.specialhi_landing_lag = 30.0;
    t::ftCa_SpecialAirHi_Anim(gp);
    assert_eq!(take(), ["fallspecial 0.75 30"]);

    // Catch a grounded victim, then throw.
    v.ground_or_air = GA_Ground;
    v.cur_pos = Vec3 { x: 1.0, y: 2.0, z: 3.0 };
    t::ftCa_SpecialLw_800E5128(gp);
    assert_eq!(take(), ["ms 355 2", "DB368"]);
    assert_eq!(f.anon_221b.x221B_b7(), 1);
    assert!(f.accessory4_cb.is_some());
    t::ftCa_SpecialLw_800E550C(gp);
    assert_eq!((f.cur_pos.x, f.cur_pos.y, f.cur_pos.z), (1.0, 2.0, 3.0));
    t::ftCa_SpecialHiCatch_Coll(gp); // grounded catch: no ft_80083B68
    t::ftCa_SpecialHiCatch_Anim(gp);
    assert_eq!(take()[1..], ["DE2A8", "DE7C0"]);
    f.cmd_vars[0] = 1;
    t::ftCa_SpecialHiThrow0_Anim(gp);
    assert_eq!(take(), ["Fall"]);
    assert_eq!(unsafe { f.mv.ca.specialhi.x2_b0() }, 1);
}
