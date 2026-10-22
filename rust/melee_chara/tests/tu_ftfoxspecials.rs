//! Smoke test for tu::ftfoxspecials (stateful; no fused sites) through recording hooks.
//! One #[test]: the hooks are process globals.
#![allow(static_mut_refs)]

use melee_chara::stubs as st;
use melee_chara::tu::ftfoxspecials as t;
use melee_types::*;

static mut LOG: std::vec::Vec<String> = std::vec::Vec::new();
fn log(s: &str) {
    unsafe { LOG.push(s.to_string()) }
}
fn take() -> std::vec::Vec<String> {
    unsafe { core::mem::take(&mut LOG) }
}

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
    let cd: &'static ftCommonData = Box::leak(Box::new(ftCommonData::zeroed()));

    unsafe {
        melee_fighter::hooks::P_FT_COMMON_DATA = cd;
        st::FIGHTER_CHANGEMOTIONSTATE = Some(|_, ms, fl, _, _, _, _| log(&format!("ms {ms} {fl:x}")));
        st::FTANIM_8006EBA4 = Some(|_| {});
        st::FTANIM_ISFRAMESREMAINING = Some(|_| 1);
        st::FT_80085088 = Some(|_| log("85088"));
        st::FT_80085134 = Some(|_| log("85134"));
        t::ext::EFSYNC_SPAWN = Some(|id, _, a| {
            log(&format!("ef {id} {}", a.len()));
            core::ptr::null_mut()
        });
        t::ext::FIGHTER_SETEFFECTHITLAGCALLBACKS = Some(|_| log("hitlag cbs"));
        t::ext::FTPARTGETROTX = Some(|_, _| 0.5);
        t::ext::IT_8029CEB4 = Some(|_, _, k, d| {
            log(&format!("ghost {k} {d}"));
            0x77usize as *mut HSD_GObj
        });
    }

    // Start enters: DAT-driven delay, ghost cleared.
    da.x24_FOX_ILLUSION_GRAVITY_DELAY = 4.9;
    da.x28_FOX_ILLUSION_GROUND_VEL_X = 2.0;
    f.gr_vel = 6.0;
    f.cmd_vars[2] = 9;
    t::ftFx_SpecialSStart_Enter(gp);
    assert_eq!(take(), [format!("ms {} 0", ftFx_MS_SpecialSStart)]);
    assert_eq!((f.gr_vel, f.cmd_vars[2], unsafe { f.mv.fx.SpecialS.gravityDelay }), (3.0, 0, 4));
    f.self_vel = Vec { x: 8.0, y: 5.0, z: 0.0 };
    f.co_attrs.max_jumps = 3;
    t::ftFx_SpecialAirSStart_Enter(gp);
    assert_eq!((f.self_vel.x, f.self_vel.y, f.x1968_jumpsUsed), (4.0, 0.0, 3));
    take();

    // Start anim over -> Dash enter seeds the afterimage history from cur_pos / rot x and installs the GFX accessory.
    unsafe { st::FTANIM_ISFRAMESREMAINING = Some(|_| 0) };
    f.cur_pos = Vec { x: 1.0, y: 2.0, z: 3.0 };
    t::ftFx_SpecialSStart_Anim(gp);
    assert_eq!(take(), [format!("ms {} {:x}", ftFx_MS_SpecialS, Ft_MF_SkipRumble)]);
    unsafe {
        assert_eq!(f.mv.fx.SpecialS.ghostEffectPos[3].z, 3.0);
        assert_eq!(f.mv.fx.SpecialS.blendFrames, [0.5; 4]);
        (f.accessory4_cb.unwrap())(gp);
    }
    assert_eq!(take(), ["ef 1165 2", "hitlag cbs"]);
    assert_eq!(f.x2219_b0(), 1);

    // Dash phys shifts the history by one and records this frame.
    f.cur_pos = Vec { x: 10.0, y: 20.0, z: 30.0 };
    t::ftFx_SpecialS_Phys(gp);
    assert_eq!(take(), ["85088"]);
    unsafe {
        assert_eq!(f.mv.fx.SpecialS.ghostEffectPos[0].x, 10.0);
        assert_eq!(f.mv.fx.SpecialS.ghostEffectPos[1].x, 1.0);
        assert_eq!(f.mv.fx.SpecialS.ghostEffectPos[3].x, 1.0);
    }
    t::ftFx_SpecialAirS_Phys(gp);
    assert_eq!(take(), ["85134"]);
    let mut p = Vec { x: 0.0, y: 0.0, z: 0.0 };
    t::ftFx_SpecialS_CopyGhostPosIndexed(gp, 1, &mut p);
    assert_eq!(p.x, 10.0);
    assert_eq!(t::ftFx_SpecialS_ReturnFloatVarIndexed(gp, 2), 0.5);
    f.cmd_vars[2] = 5;
    assert_eq!(t::ftFx_SpecialS_GetCmdVar2(gp), 5);

    // Dash anim: script spawns the ghost (Fox -> Illusion), still dashing while frames remain.
    unsafe { st::FTANIM_ISFRAMESREMAINING = Some(|_| 1) };
    f.cmd_vars[2] = 1;
    f.facing_dir = -1.0;
    t::ftFx_SpecialS_Anim(gp);
    assert_eq!(take(), [format!("ghost {} -1", It_Kind_Fox_Illusion)]);
    assert_eq!(unsafe { f.mv.fx.SpecialS.ghostGObj } as usize, 0x77);
    assert_eq!((f.cmd_vars[2], f.x2222_b2()), (0, 1));
    f.kind = Ft_Kind_Falco;
    f.cmd_vars[2] = 1;
    t::ftFx_SpecialAirS_Anim(gp);
    assert_eq!(take(), [format!("ghost {} -1", It_Kind_Falco_Phantasm)]);
    unsafe { st::FTANIM_ISFRAMESREMAINING = Some(|_| 0) };
    f.cmd_vars[2] = 0;
    t::ftFx_SpecialS_Anim(gp);
    assert_eq!(take(), [format!("ms {} {:x}", ftFx_MS_SpecialSEnd, Ft_MF_SkipRumble), ]);

    // IASA: B ends the dash, picking the grounded / aerial end by ground_or_air.
    f.input.pressed_buttons = 0;
    t::ftFx_SpecialS_IASA(gp);
    assert!(take().is_empty());
    f.input.pressed_buttons = HSD_PAD_B as _;
    f.ground_or_air = GA_Air;
    t::ftFx_SpecialS_IASA(gp);
    assert_eq!(take(), [format!("ms {} {:x}", ftFx_MS_SpecialAirSEnd, Ft_MF_SkipRumble)]);
    f.ground_or_air = GA_Ground;
    t::ftFx_SpecialAirS_IASA(gp);
    assert_eq!(take(), [format!("ms {} {:x}", ftFx_MS_SpecialSEnd, Ft_MF_SkipRumble)]);

    // End enters set velocities from the DAT and arm the fall accel counter.
    da.x34_FOX_ILLUSION_GROUND_END_VEL_X = 1.5;
    da.x3C_FOX_ILLUSION_AIR_END_VEL_X = 2.5;
    da.x44_FOX_ILLUSION_FALL_ACCEL = 6.9;
    f.facing_dir = -1.0;
    t::ftFx_SpecialSEnd_Enter(gp);
    assert_eq!((f.gr_vel, unsafe { f.mv.fx.SpecialS.gravityDelay }), (-1.5, 6));
    t::ftFx_SpecialAirSEnd_Enter(gp);
    assert_eq!((f.self_vel.x, f.self_vel.y), (-2.5, 0.0));
    take();

    // Air end phys: delay counts down then falls with the DAT terminal velocity; ghost history keeps moving.
    da.x48_FOX_ILLUSION_TERMINAL_VELOCITY = 0.5;
    f.co_attrs.terminal_velocity = 9.0;
    unsafe { f.mv.fx.SpecialS.gravityDelay = 1 };
    f.self_vel.y = 0.0;
    t::ftFx_SpecialAirSEnd_Phys(gp);
    assert_eq!((unsafe { f.mv.fx.SpecialS.gravityDelay }, f.self_vel.y), (0, 0.0));
    t::ftFx_SpecialAirSEnd_Phys(gp);
    assert_eq!(f.self_vel.y, -0.5);

    // Motion range test.
    f.motion_id = ftFx_MS_SpecialAirS as FtMotionId;
    assert!(!t::ftFx_SpecialS_CheckGhostRemove(gp));
    f.motion_id = ftFx_MS_SpecialHi as FtMotionId;
    assert!(t::ftFx_SpecialS_CheckGhostRemove(gp));
}
