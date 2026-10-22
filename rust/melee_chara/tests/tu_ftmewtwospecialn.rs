//! Smoke test of tu::ftmewtwospecialn (Shadow Ball): entries, the charge query, the remove /
//! cancel predicates, recoil and release, through recording hooks.
#![allow(static_mut_refs)]

use melee_chara::stubs as st;
use melee_chara::tu::ftmewtwospecialn as t;
use melee_types::*;

static mut LOG: std::vec::Vec<String> = std::vec::Vec::new();
fn log(s: String) {
    unsafe { LOG.push(s) }
}
fn take() -> String {
    unsafe { std::mem::take(&mut LOG).join(",") }
}

#[test]
fn smoke() {
    let mut attrs: Box<ftMewtwoAttributes> = Box::new(unsafe { core::mem::zeroed() });
    let mut f: Box<Fighter> = Box::new(Fighter::zeroed());
    f.dat_attrs = &mut *attrs as *mut _ as *mut _;
    let mut parts: std::vec::Vec<FighterBone> = (0..64).map(|_| unsafe { core::mem::zeroed() }).collect();
    f.parts = parts.as_mut_ptr();
    let mut g: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    g.user_data = &mut *f as *mut Fighter as *mut _;
    let gp: *mut HSD_GObj = &mut *g;
    let mut ball: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    let bp: *mut HSD_GObj = &mut *ball;
    unsafe {
        st::FIGHTER_CHANGEMOTIONSTATE = Some(|_, ms, _, _, _, _, _| log(format!("M{ms}")));
        st::FTANIM_8006EBA4 = Some(|_| log("8006EBA4".into()));
        st::EFLIB_DESTROYALL = Some(|_| log("efdestroy".into()));
        st::FTCOMMON_8007D7FC = Some(|_| log("7fc".into()));
        t::ext::IT_802C573C = Some(|_| log("destroy_ball".into()));
        t::ext::FTCO_800BFFAC = Some(|_| log("ffac".into()));
        t::ext::FT_PLAYSFX = Some(|_, id, _, _| log(format!("sfx {id}")));
        t::ext::LB_8000B1CC = Some(|_, off, out| {
            assert_eq!((*off).z, 2.0);
            *out = Vec3 { x: 5.0, y: 6.0, z: 7.0 }
        });
        t::ext::IT_802C53F0 = Some(|_, pos, angle, charge, cycles| {
            log(format!("launch {} {} {} {} {}", (*pos).x, (*pos).y, (*pos).z, angle, charge));
            assert_eq!(cycles, 7.0);
        });
    }
    attrs.x0_MEWTWO_SHADOWBALL_CHARGE_CYCLES = 7.0;
    attrs.x10_MEWTWO_SHADOWBALL_RELEASE_LAG = 9;

    // Entries: release lag only when no charge is carried; air entry halves the vertical speed.
    f.u.mt.x2234_shadowBallCharge = 0;
    f.self_vel.y = 3.0;
    t::ftMt_SpecialN_Enter(gp);
    assert_eq!(unsafe { f.mv.mt.SpecialN.releaseLag }, 9);
    assert_eq!(f.self_vel.y, 0.0);
    assert!(take().starts_with(&format!("M{}", ftMt_MS_SpecialNStart)));
    f.u.mt.x2234_shadowBallCharge = 3;
    f.self_vel.y = 3.0;
    t::ftMt_SpecialAirN_Enter(gp);
    assert_eq!(unsafe { f.mv.mt.SpecialN.releaseLag }, 0);
    assert_eq!(f.self_vel.y, 1.5);
    take();

    // GetChargeLevel: -1 without a held ball, else charge + cycles.
    let (mut lvl, mut cyc) = (0, 0);
    assert_eq!(t::ftMt_SpecialN_GetChargeLevel(gp, &mut lvl, &mut cyc), -1);
    f.u.mt.x2230_shadowHeldGObj = bp;
    assert_eq!(t::ftMt_SpecialN_GetChargeLevel(gp, &mut lvl, &mut cyc), 0);
    assert_eq!((lvl, cyc), (3, 7));
    assert_eq!(t::ftMt_SpecialN_GetChargeLevel(core::ptr::null_mut(), &mut lvl, &mut cyc), -1);

    // Predicates by motion id.
    f.motion_id = ftMt_MS_SpecialNLoop as _;
    assert!(!t::ftMt_SpecialN_CheckShadowBallCancel(gp));
    f.motion_id = ftMt_MS_SpecialNCancel as _;
    assert!(t::ftMt_SpecialN_CheckShadowBallCancel(gp));
    f.motion_id = 1;
    assert!(t::ftMt_SpecialN_CheckShadowBallRemove(gp));
    assert!(t::ftMt_SpecialN_CheckShadowBallCancel(core::ptr::null_mut()));

    // Recoil: air recoil on air, ground recoil on ground.
    attrs.x8_MEWTWO_SHADOWBALL_AIR_RECOIL_X = 0.5;
    attrs.x4_MEWTWO_SHADOWBALL_GROUND_RECOIL_X = 0.25;
    f.facing_dir = -1.0;
    f.gr_vel = 0.0;
    f.self_vel.x = 0.0;
    f.ground_or_air = GA_Air as _;
    t::ftMewtwo_SpecialN_SetRecoil(gp);
    assert_eq!((f.self_vel.x, f.gr_vel), (-1.5, 0.0));
    f.ground_or_air = GA_Ground as _;
    t::ftMewtwo_SpecialN_SetRecoil(gp);
    assert_eq!(f.gr_vel, -0.75);

    // Release: needs cmd_vars[1] == 1 and a held ball; launches from the shoulder with z cleared.
    f.facing_dir = 1.0;
    f.cmd_vars[1] = 0;
    t::ftMt_SpecialN_ReleaseShadowBall(gp);
    assert_eq!(take(), "");
    f.cmd_vars[1] = 1;
    t::ftMt_SpecialN_ReleaseShadowBall(gp);
    assert_eq!(take(), "launch 5 6 0 0 3,sfx 200115");
    assert_eq!(f.cmd_vars[1], 2);
    assert_eq!(unsafe { f.u.mt.x2234_shadowBallCharge }, 0);
    assert!(unsafe { f.u.mt.x2230_shadowHeldGObj.is_null() });

    // OnTakeDamage drops a held ball and resets the charge.
    f.u.mt.x2230_shadowHeldGObj = bp;
    f.u.mt.x2234_shadowBallCharge = 2;
    t::ftMt_SpecialN_OnTakeDamage(gp);
    assert_eq!(take(), "destroy_ball,ffac,efdestroy");
    assert_eq!(unsafe { f.u.mt.x2234_shadowBallCharge }, 0);
}
