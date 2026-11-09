//! tu::ftkirbyspecialfox (src/melee/ft/kinds/ftKirby/ftkirbyspecialfox.c): no fused sites. The one
//! float step, `M_PI - launch_angle` in double, is checked by hand; the motion-id mappings and the
//! state callbacks are exercised through recording hooks.
#![allow(static_mut_refs)]

use melee_chara::stubs as st;
use melee_chara::tu::ftkirbyspecialfox as t;
use melee_types::*;

static mut LOG: std::vec::Vec<String> = std::vec::Vec::new();
fn log(s: String) {
    unsafe { LOG.push(s) }
}
fn take() -> String {
    unsafe { core::mem::take(&mut LOG).join(",") }
}

#[test]
fn smoke() {
    let mut f: Box<Fighter> = Box::new(Fighter::zeroed());
    let mut g: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    let da: &'static mut ftKb_DatAttrs = Box::leak(Box::new(ftKb_DatAttrs::zeroed()));
    let mut parts: std::vec::Vec<FighterBone> = (0..60).map(|_| unsafe { core::mem::zeroed() }).collect();
    let mut item: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    f.dat_attrs = da as *mut _ as *mut _;
    f.parts = parts.as_mut_ptr();
    g.user_data = &mut *f as *mut Fighter as *mut _;
    let gp: *mut HSD_GObj = &mut *g;
    let ip: *mut HSD_GObj = &mut *item;
    unsafe {
        st::FIGHTER_CHANGEMOTIONSTATE = Some(|_, msid, flags, a, b, c, _| log(format!("cms {msid} {flags:x} {a} {b} {c}")));
        st::FTANIM_8006EBA4 = Some(|_| log("8006EBA4".into()));
        st::FTANIM_ISFRAMESREMAINING = Some(|_| 0);
        st::FT_8008A2BC = Some(|_| log("8A2BC".into()));
        st::FTCO_FALL_ENTER = Some(|_| log("fall".into()));
        st::FTCO_80096900 = Some(|_, a, b, c, m, l| log(format!("96900 {a} {b} {c} {m} {l}")));
        st::FTCOMMON_8007D7FC = Some(|_| {});
        melee_fighter::hooks::PLAY_SFX = Some(|_, id, vol, pan| log(format!("sfx {id:x} {vol} {pan}")));
        t::ext::LB_8000B1CC = Some(|_, off, out| unsafe {
            log(format!("lb {} {} {}", (*off).x, (*off).y, (*off).z));
            *out = Vec3 { x: 1.0, y: 2.0, z: 3.0 };
        });
        t::ext::IT_802AEAB4 = Some(|_| log("destroy".into()));
        t::ext::IT_8029C6A4 = Some(|ang, speed, _, pos, id| log(format!("shot {ang} {speed} {} {} {} {id}", unsafe { (*pos).x }, unsafe { (*pos).y }, unsafe { (*pos).z })));
        t::ext::IT_802AE1D0 = Some(|_| log("1D0".into()));
        t::ext::IT_802AE8A8 = Some(|dir, _, _, part, id| {
            log(format!("spawn {dir} {part} {id}"));
            core::ptr::null_mut::<HSD_GObj>().wrapping_add(0x1000)
        });
        t::ext::IT_802ADDD0 = Some(|_, v| log(format!("DDD0 {v}")));
        t::ext::IT_802AE538 = Some(|_| log("E538".into()));
        t::ext::IT_802AE608 = Some(|_| log("E608".into()));
        t::ext::IT_802AAA50 = Some(|_| log("AA50".into()));
        t::ext::FTFOX_SPECIALN_CHECKLOOPINPUT = Some(|_| log("loopinput".into()));
        t::ext::FTPICKUPITEM_80094818 = Some(|_, b| log(format!("pickup {b}")));
    }

    // Motion-id mappings (800FDD4C): Fox / Falco block motions map to 0..5, throws to 0xD6 below, else 9.
    f.u.kb.hat.kind = Ft_Kind_Fox;
    f.motion_id = ftKb_MS_FxSpecialAirNEnd as _;
    assert_eq!(t::ftKb_SpecialNFx_800FDD4C(gp), ftKb_MS_FxSpecialAirNEnd - 0x1A7);
    f.motion_id = ftCo_MS_ThrowHi as _;
    assert_eq!(t::ftKb_SpecialNFx_800FDD4C(gp), (ftCo_MS_ThrowHi as u32) - 0xD6);
    f.motion_id = 5 as _;
    assert_eq!(t::ftKb_SpecialNFx_800FDD4C(gp), 9);
    f.u.kb.hat.kind = Ft_Kind_Falco;
    f.motion_id = ftKb_MS_FcSpecialNLoop as _;
    assert_eq!(t::ftKb_SpecialNFx_800FDD4C(gp), ftKb_MS_FcSpecialNLoop - 0x208);
    f.motion_id = ftKb_MS_FxSpecialNLoop as _;
    assert_eq!(t::ftKb_SpecialNFx_800FDD4C(gp), 9);
    assert_eq!(t::ftKb_SpecialNFx_800FDD4C(core::ptr::null_mut()), 9);
    // 800FDDF4: x2071_b6 inside the block motions, true elsewhere / without a fighter.
    f.motion_id = ftKb_MS_FcSpecialAirNStart as _;
    unsafe { f.x2070.x0.set_x2071_b6(0) };
    assert!(!t::ftKb_SpecialNFx_800FDDF4(gp));
    unsafe { f.x2070.x0.set_x2071_b6(1) };
    assert!(t::ftKb_SpecialNFx_800FDDF4(gp));
    f.motion_id = 1 as _;
    unsafe { f.x2070.x0.set_x2071_b6(0) };
    assert!(t::ftKb_SpecialNFx_800FDDF4(gp));
    assert!(t::ftKb_SpecialNFx_800FDDF4(core::ptr::null_mut()));

    // Muzzle positions: lb_8000B1CC with the fixed offsets, or zero without a fighter.
    let mut v = Vec3 { x: 9.0, y: 9.0, z: 9.0 };
    t::ftKb_SpecialNFx_800FDC00(gp, &mut v);
    assert_eq!((v.x, v.y, v.z), (1.0, 2.0, 3.0));
    t::ftKb_SpecialNFx_800FDC70(gp, &mut v);
    t::ftKb_SpecialNFx_800FDC00(core::ptr::null_mut(), &mut v);
    assert_eq!((v.x, v.y, v.z), (0.0, 0.0, 0.0));
    assert_eq!(take(), "lb 0 1.45 5.016,lb 0 1.45 0.016");

    // Blaster item: 800FDD14 / DEB4 / DEE0.
    assert!(t::ftKb_SpecialNFx_800FDD14(gp));
    f.u.kb.xB0 = ip;
    assert!(!t::ftKb_SpecialNFx_800FDD14(gp));
    t::ftKb_SpecialNFx_800FDEB4(gp);
    assert!(unsafe { f.u.kb.xB0 }.is_null());
    f.u.kb.xB0 = ip;
    t::ftKb_SpecialNFx_800FDEE0(gp);
    assert!(unsafe { f.u.kb.xB0 }.is_null());
    assert_eq!(take(), "destroy");

    // Shot: facing right uses the launch angle, facing left PI - angle (double); voice by facing.
    f.u.kb.hat.kind = Ft_Kind_Fox;
    da.specialn_fx_launch_angle = 0.5;
    da.specialn_fx_launch_speed = 4.0;
    da.specialn_fx_blaster_projectile_id = 7;
    da.specialn_fc_launch_angle = 0.25;
    da.specialn_fc_launch_speed = 6.0;
    da.specialn_fc_blaster_projectile_id = 9;
    f.u.kb.xB0 = ip;
    f.cmd_vars[2] = 1;
    f.facing_dir = 1.0;
    t::ftKb_SpecialNFx_CreateBlasterShot(gp);
    assert_eq!(f.cmd_vars[2], 0);
    f.cmd_vars[2] = 1;
    f.facing_dir = -1.0;
    t::ftKb_SpecialNFx_CreateBlasterShot(gp);
    f.u.kb.hat.kind = Ft_Kind_Falco;
    f.cmd_vars[2] = 1;
    t::ftKb_SpecialNFx_CreateBlasterShot(gp);
    let pi_minus = core::f64::consts::PI - 0.5f32 as f64;
    let pi_minus_fc = core::f64::consts::PI - 0.25f32 as f64;
    assert_eq!(
        take(),
        format!(
            "lb 0 1.45 5.016,shot 0.5 4 1 2 0 7,1D0,sfx 1ae17 127 64,lb 0 1.45 5.016,shot {pi_minus} 4 1 2 0 7,1D0,sfx 1ae1a 127 64,\
lb 0 1.45 5.016,shot {pi_minus_fc} 6 1 2 0 9,1D0,sfx 18706 127 64"
        )
    );

    // Entry: motion by hat kind, blaster item spawned with the kind's id, death/take-damage cbs set.
    f.u.kb.hat.kind = Ft_Kind_Falco;
    da.specialn_fc_blaster_item_id = 33;
    f.facing_dir = 1.0;
    f.self_vel.x = 4.0;
    t::ftKb_SpecialNFx_800FE100(gp);
    assert!(!unsafe { f.u.kb.xB0 }.is_null());
    assert_eq!((f.self_vel.x, f.gr_vel), (0.0, 0.0));
    assert_eq!(unsafe { f.mv.fx.SpecialN.isBlasterLoop }, 0);
    assert!(f.death2_cb.is_some() && f.take_dmg_cb.is_some());
    assert_eq!(take(), format!("cms {} 0 0 1 0,8006EBA4,spawn 1 {} 33", ftKb_MS_FcSpecialNStart, FtPart_R3rdNa));
    f.u.kb.hat.kind = Ft_Kind_Fox;
    da.specialn_fx_blaster_item_id = 31;
    t::ftKb_SpecialNFx_800FE240(gp);
    assert_eq!(take(), format!("cms {} 0 0 1 0,8006EBA4,spawn 1 {} 31", ftKb_MS_FxSpecialAirNStart, FtPart_R3rdNa));

    // Loop anim: out of frames with isBlasterLoop set re-enters the loop, otherwise ends; Shot at the end.
    f.u.kb.hat.kind = Ft_Kind_Fox;
    f.cmd_vars = [0, 0, 0, 1];
    unsafe { f.mv.fx.SpecialN.isBlasterLoop = 1 };
    t::ftKb_FxSpecialNLoop_Anim(gp);
    assert_eq!(unsafe { f.mv.fx.SpecialN.isBlasterLoop }, 0);
    assert!(f.x21EC.is_some() && f.accessory4_cb.is_some());
    let flags = Ft_MF_SkipAttackCount | Ft_MF_SkipModel | Ft_MF_KeepGfx;
    assert_eq!(take(), format!("DDD0 1,E538,cms {} {flags:x} 0 1 0,DDD0 1", ftKb_MS_FxSpecialNLoop));
    t::ftKb_FxSpecialNLoop_Anim(gp);
    assert_eq!(f.cmd_vars[1], 1);
    assert_eq!(
        take(),
        format!("DDD0 1,cms {} {:x} 0 1 0,DDD0 1", ftKb_MS_FxSpecialNEnd, Ft_MF_SkipModel | Ft_MF_KeepGfx)
    );
    // End anim: pose by cmd_vars[1], pick up on 2, release on cmd_vars[3] == 2, then Wait.
    f.cmd_vars = [0, 2, 0, 2];
    f.u.kb.xB0 = ip;
    t::ftKb_FxSpecialNEnd_Anim(gp);
    assert!(unsafe { f.u.kb.xB0 }.is_null());
    assert_eq!(take(), "DDD0 2,pickup false,E608,8A2BC");
    // Air end anim: freefall by the hat's toggle (Falco 20), or Fall at 0.
    f.u.kb.hat.kind = Ft_Kind_Falco;
    da.specialn_fc_freefall_toggle = 20.0;
    f.cmd_vars = [0, 0, 0, 0];
    t::ftKb_FxSpecialAirNEnd_Anim(gp);
    f.u.kb.hat.kind = Ft_Kind_Fox;
    da.specialn_fx_freefall_toggle = 0.0;
    t::ftKb_FxSpecialAirNEnd_Anim(gp);
    f.u.kb.hat.kind = Ft_Kind_Kirby;
    t::ftKb_FxSpecialAirNEnd_Anim(gp);
    assert_eq!(take(), "DDD0 0,96900 1 0 1 1 20,DDD0 0,fall,DDD0 0,fall");
    // IASA forwards to ftFox_SpecialN_CheckLoopInput except the End states.
    t::ftKb_FxSpecialNStart_IASA(gp);
    t::ftKb_FxSpecialNEnd_IASA(gp);
    t::ftKb_FxSpecialAirNLoop_IASA(gp);
    assert_eq!(take(), "loopinput,loopinput");

    // Ness PK Flash helpers.
    f.u.kb.ns_flash_gobj = ip as *mut Item_GObj;
    f.motion_id = ftKb_MS_NsSpecialAirNHold0 as _;
    assert!(t::ftKb_SpecialNNs_800FEC78(gp));
    f.motion_id = ftKb_MS_NsSpecialNHold1 as _;
    assert!(!t::ftKb_SpecialNNs_800FEC78(gp));
    t::ftKb_SpecialNNs_800FECBC(gp);
    assert!(unsafe { f.u.kb.ns_flash_gobj }.is_null());
    f.u.kb.ns_flash_gobj = ip as *mut Item_GObj;
    t::ftKb_SpecialNNs_800FECE8(gp);
    assert_eq!(take(), "AA50");
    drop(parts);
}
