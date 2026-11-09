//! Smoke test for tu::ftfoxspecialn (stateful; no fused sites) through recording hooks.
//! One #[test]: the hooks are process globals.
#![allow(static_mut_refs)]

use melee_chara::stubs as st;
use melee_chara::tu::ftfoxspecialn as t;
use melee_types::*;

static mut LOG: std::vec::Vec<String> = std::vec::Vec::new();
fn log(s: &str) {
    unsafe { LOG.push(s.to_string()) }
}
fn take() -> std::vec::Vec<String> {
    unsafe { core::mem::take(&mut LOG) }
}

const BLASTER: usize = 0x5150;

#[test]
fn smoke() {
    let mut f: Box<Fighter> = Box::new(Fighter::zeroed());
    let mut g: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    let da: &'static mut ftFox_DatAttrs = Box::leak(Box::new(ftFox_DatAttrs::zeroed()));
    let mut parts: std::vec::Vec<FighterBone> = (0..80).map(|_| unsafe { core::mem::zeroed() }).collect();
    f.parts = parts.as_mut_ptr();
    f.dat_attrs = da as *mut _ as *mut _;
    f.kind = Ft_Kind_Fox;
    g.user_data = &mut *f as *mut Fighter as *mut _;
    let gp: *mut HSD_GObj = &mut *g;
    let blaster = BLASTER as *mut HSD_GObj;

    unsafe {
        st::FIGHTER_CHANGEMOTIONSTATE = Some(|_, ms, fl, _, _, _, _| log(&format!("ms {ms} {fl:x}")));
        st::FTANIM_8006EBA4 = Some(|_| {});
        st::FTANIM_ISFRAMESREMAINING = Some(|_| 1);
        st::FTCOMMON_8007D7FC = Some(|_| log("7D7FC"));
        st::FT_8008A2BC = Some(|_| log("wait"));
        st::FTCO_FALL_ENTER = Some(|_| log("Fall"));
        st::FTCO_80096900 = Some(|_, a, b, c, mob, lag| log(&format!("fallspecial {a} {b} {c} {mob} {lag}")));
        t::ext::FTPARTS_GETBONEINDEX = Some(|_, p| p);
        t::ext::LB_8000B1CC = Some(|_, off, out| {
            // the hold joint: (0, 1.2325, z_offset) in, world position out = (z_offset, 5, 9)
            *out = Vec3 { x: (*off).z, y: 5.0, z: 9.0 };
        });
        t::ext::FT_800892A0 = Some(|_| log("892A0"));
        t::ext::FT_80089824 = Some(|_| log("89824"));
        t::ext::FTFX_INIT_800E5534 = Some(|_| false);
        t::ext::FTPICKUPITEM_80094818 = Some(|_, b| log(&format!("pickup {b}")));
        t::ext::FTLIB_GETKIND = Some(|_| Ft_Kind_Fox);
        t::ext::FT_PLAYSFX = Some(|_, id, v, p| log(&format!("sfx {id} {v} {p}")));
        t::ext::OSREPORT = Some(|_| log("osreport"));
        t::ext::IT_802AE8A8 = Some(|d, _, _, bone, k| {
            log(&format!("spawn {d} {bone} {k}"));
            BLASTER as *mut HSD_GObj
        });
        t::ext::IT_8026BAE8 = Some(|_, s| log(&format!("scale {s}")));
        t::ext::IT_802ADDD0 = Some(|_, v| log(&format!("vis {v}")));
        t::ext::IT_802AE538 = Some(|_| log("538"));
        t::ext::IT_802AE608 = Some(|_| log("608"));
        t::ext::IT_802AE1D0 = Some(|_| log("1D0"));
        t::ext::IT_802AEAB4 = Some(|_| log("remove"));
        t::ext::IT_8029C6A4 = Some(|a, v, _, p, k| log(&format!("shot {a} {v} {} {k}", (*p).x)));
        t::ext::IT_8029C6CC = Some(|a, v, _, p, k| log(&format!("throwshot {a} {v} {} {k}", (*p).x)));
    }

    da.x10_FOX_BLASTER_ANGLE = 0.5;
    da.x14_FOX_BLASTER_VEL = 3.0;
    da.x18_FOX_BLASTER_LANDING_LAG = 20.0;
    da.x1C_FOX_BLASTER_SHOT_ITKIND = 11;
    da.x20_FOX_BLASTER_GUN_ITKIND = 12;

    // Enter: grounded stop, spawn the held blaster.
    f.facing_dir = 1.0;
    f.gr_vel = 3.0;
    f.self_vel = Vec { x: 1.0, y: 2.0, z: 3.0 };
    f.cmd_vars = [1, 2, 3, 4];
    t::ftFx_SpecialN_Enter(gp);
    assert_eq!(
        take(),
        ["7D7FC".to_string(), format!("ms {} 0", ftFx_MS_SpecialNStart), format!("spawn 1 {} 12", FtPart_RThumbNb), "scale 0.85".to_string()]
    );
    assert_eq!((f.gr_vel, f.self_vel.x, f.self_vel.y, f.self_vel.z, f.cmd_vars), (0.0, 0.0, 0.0, 0.0, [0, 0, 0, 0]));
    assert_eq!(unsafe { f.u.fx.x222C_blasterGObj }, blaster);
    assert!(f.take_dmg_cb.is_some() && f.death2_cb.is_some());
    // spawn failure asserts
    unsafe { t::ext::IT_802AE8A8 = Some(|_, _, _, _, _| core::ptr::null_mut()) };
    let r = std::panic::catch_unwind(|| t::ftFx_SpecialAirN_Enter(gp as *mut HSD_GObj));
    assert!(r.is_err());
    take();
    unsafe {
        t::ext::IT_802AE8A8 = Some(|d, _, _, bone, k| {
            log(&format!("spawn {d} {bone} {k}"));
            BLASTER as *mut HSD_GObj
        })
    };
    f.u.fx.x222C_blasterGObj = blaster;

    // Start anim keeps the item visible; when over -> loop with the shot accessory.
    f.cmd_vars = [0, 0, 0, 1];
    t::ftFx_SpecialNStart_Anim(gp);
    assert_eq!(take(), ["vis 1", "538"]);
    unsafe { st::FTANIM_ISFRAMESREMAINING = Some(|_| 0) };
    t::ftFx_SpecialNStart_Anim(gp);
    assert_eq!(take(), ["vis 1".to_string(), format!("ms {} {:x}", ftFx_MS_SpecialNLoop, Ft_MF_SkipModel | Ft_MF_KeepGfx), "vis 1".to_string()]);
    assert!(f.accessory4_cb.is_some());

    // The accessory fires the shot: angle 0.5 rad facing right, hold z offset 4.2636 at the thumb.
    f.cmd_vars[2] = 1;
    unsafe { (f.accessory4_cb.unwrap())(gp) };
    assert_eq!(take(), [format!("shot 0.5 3 {} 11", 4.263599872589111_f32), "1D0".to_string(), "sfx 110103 127 64".to_string()]);
    assert_eq!(f.cmd_vars[2], 0);
    // Facing left: angle is PI - x (a C double narrowed to float), sfx index 1.
    f.facing_dir = -1.0;
    f.cmd_vars[2] = 1;
    unsafe { (f.accessory4_cb.unwrap())(gp) };
    let want_angle = (core::f64::consts::PI - 0.5_f32 as f64) as f32;
    assert_eq!(take(), [format!("shot {want_angle} 3 {} 11", 4.263599872589111_f32), "1D0".to_string(), "sfx 110106 127 64".to_string()]);
    f.facing_dir = 1.0;

    // Loop anim: B was pressed -> stay in the loop (OnChangeAction accessory); else -> End.
    f.mv.fx.SpecialN.isBlasterLoop = 1;
    f.cmd_vars = [0, 0, 0, 0];
    t::ftFx_SpecialNLoop_Anim(gp);
    assert_eq!(
        take(),
        ["vis 1".to_string(), format!("ms {} {:x}", ftFx_MS_SpecialNLoop, Ft_MF_SkipAttackCount | Ft_MF_SkipModel | Ft_MF_KeepGfx), "vis 1".to_string()]
    );
    assert_eq!(unsafe { f.mv.fx.SpecialN.isBlasterLoop }, 0);
    unsafe { (f.x21EC.unwrap())(gp) };
    assert_eq!(take(), ["892A0", "89824"]);
    t::ftFx_SpecialNLoop_Anim(gp);
    assert_eq!(take()[1], format!("ms {} {:x}", ftFx_MS_SpecialNEnd, Ft_MF_SkipModel | Ft_MF_KeepGfx));
    assert_eq!(f.cmd_vars[1], 1);

    // IASA: B press inside the window requests another loop.
    f.cmd_vars[0] = 1;
    f.input.pressed_buttons = HSD_PAD_B as _;
    t::ftFx_SpecialAirNLoop_IASA(gp);
    assert_eq!(unsafe { f.mv.fx.SpecialN.isBlasterLoop }, 1);

    // End anim: ends -> remove (null), wait; air end with landing lag -> freefall, without -> Fall.
    unsafe { st::FTANIM_ISFRAMESREMAINING = Some(|_| 1) };
    f.cmd_vars = [0, 2, 0, 2];
    t::ftFx_SpecialNEnd_Anim(gp); // frames remaining: only the visibility / pickup / 608 housekeeping
    assert_eq!(take(), ["vis 2", "pickup false", "608"]);
    unsafe { st::FTANIM_ISFRAMESREMAINING = Some(|_| 0) };
    f.u.fx.x222C_blasterGObj = blaster;
    f.cmd_vars = [0, 0, 0, 0];
    t::ftFx_SpecialNEnd_Anim(gp);
    assert_eq!(take(), ["vis 0", "wait"]);
    assert!(unsafe { f.u.fx.x222C_blasterGObj }.is_null());
    t::ftFx_SpecialAirNEnd_Anim(gp);
    assert_eq!(take(), ["vis 0", "fallspecial 1 0 1 1 20"]);
    da.x18_FOX_BLASTER_LANDING_LAG = 0.0;
    t::ftFx_SpecialAirNEnd_Anim(gp);
    assert_eq!(take(), ["vis 0", "Fall"]);

    // Blaster bookkeeping helpers.
    f.u.fx.x222C_blasterGObj = blaster;
    assert!(!t::ftFx_SpecialN_CheckRemoveBlaster(gp));
    t::ftFx_SpecialN_RemoveBlaster(gp);
    assert_eq!(take(), ["remove"]);
    assert!(t::ftFx_SpecialN_CheckRemoveBlaster(gp));
    assert!(f.take_dmg_cb.is_none() && f.death2_cb.is_none());
    f.motion_id = ftFx_MS_SpecialAirNLoop as FtMotionId;
    assert_eq!(t::ftFx_SpecialN_GetBlasterAction(gp), (ftFx_MS_SpecialAirNLoop - ftFx_MS_SpecialNStart) as i32);
    f.motion_id = ftCo_MS_ThrowHi as FtMotionId;
    assert_eq!(t::ftFx_SpecialN_GetBlasterAction(gp), (ftCo_MS_ThrowHi - ftCo_MS_CatchDash) as i32);
    f.motion_id = 0;
    assert_eq!(t::ftFx_SpecialN_GetBlasterAction(gp), 9);
    assert_eq!(t::ftFx_SpecialN_GetBlasterAction(core::ptr::null_mut()), 9);
    f.motion_id = ftFx_MS_SpecialNEnd as FtMotionId;
    unsafe { f.x2070.x0.set_x2071_b6(1) };
    assert!(t::ftFx_SpecialN_CheckBlasterAction(gp));
    unsafe { f.x2070.x0.set_x2071_b6(0) };
    assert!(!t::ftFx_SpecialN_CheckBlasterAction(gp));
    f.motion_id = 0;
    assert!(t::ftFx_SpecialN_CheckBlasterAction(gp));
    assert!(t::ftFx_SpecialN_CheckBlasterAction(core::ptr::null_mut()));

    // Throw anim: spawn on cmd 1, throw on throw flag with the move-specific sfx, drop on cmd 2, clear on cmd 0.
    unsafe { st::FTANIM_ISFRAMESREMAINING = Some(|_| 1) };
    f.cmd_vars = [0, 1, 0, 0];
    t::ftFx_Throw_Anim(gp);
    assert_eq!(take(), [format!("spawn 1 {} 12", FtPart_RThumbNb), "scale 0".to_string()]);
    f.x34_scale.y = 1.0;
    f.co_attrs.model_scaling = 2.0;
    f.u.fx.x222C_blasterGObj = core::ptr::null_mut();
    t::ftFx_Throw_Anim(gp);
    assert_eq!(take()[1], "scale 1.7");
    f.motion_id = ftCo_MS_ThrowHi as FtMotionId;
    unsafe { f.x2210.x0.set_throw_flags_b0(1) };
    t::ftFx_Throw_Anim(gp);
    let l = take();
    assert_eq!(l[0], "vis 1");
    assert!(l[1].starts_with("throwshot "), "{l:?}");
    assert_eq!(&l[2..], ["1D0", "sfx 110109 127 64"]);
    assert_eq!(unsafe { f.x2210.x0.throw_flags_b0() }, 0);
    f.motion_id = ftCo_MS_ThrowB as FtMotionId;
    unsafe { f.x2210.x0.set_throw_flags_b0(1) };
    t::ftFx_Throw_Anim(gp);
    let l = take();
    assert!(l[1].starts_with("throwshot "));
    assert_eq!(l[3], "sfx 110106 127 64"); // foxSFX[1 == facing_dir]
    f.motion_id = 0;
    unsafe { f.x2210.x0.set_throw_flags_b0(1) };
    t::ftFx_Throw_Anim(gp);
    assert!(take()[1].starts_with("shot "));
    f.cmd_vars[1] = 2;
    t::ftFx_Throw_Anim(gp);
    assert_eq!(take(), ["sfx 110100 127 64"]);
    f.cmd_vars[1] = 0;
    t::ftFx_Throw_Anim(gp);
    assert_eq!(take(), ["pickup false"]);
    assert!(f.take_dmg_cb.is_none());
}
