//! Smoke test of tu::ftpeachspecials (Peach Bomber) through recording hooks, including the two
//! fused position updates of `ftPe_SpecialSStart_Anim`.
#![allow(static_mut_refs)]

use melee_chara::stubs as st;
use melee_chara::tu::ftpeachspecials as t;
use melee_core::fp::fmadds;
use melee_types::*;

static mut LOG: std::vec::Vec<String> = std::vec::Vec::new();
fn log(s: String) {
    unsafe { LOG.push(s) }
}
fn take() -> std::vec::Vec<String> {
    unsafe { core::mem::take(&mut LOG) }
}
static mut FRAMES: i32 = 0;
static mut AIR: i32 = 0;

#[test]
fn smoke() {
    let mut f: Box<Fighter> = Box::new(Fighter::zeroed());
    let mut g: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    let da: &'static mut ftPe_DatAttrs = Box::leak(Box::new(ftPe_DatAttrs::zeroed()));
    let mut bones: std::vec::Vec<FighterBone> = (0..8).map(|_| unsafe { core::mem::zeroed() }).collect();
    f.parts = bones.as_mut_ptr();
    f.dat_attrs = da as *mut _ as *mut _;
    g.user_data = &mut *f as *mut Fighter as *mut _;
    let gp: *mut HSD_GObj = &mut *g;
    unsafe {
        st::FIGHTER_CHANGEMOTIONSTATE = Some(|_, ms, fl, a, s, b, _| log(format!("ms {ms} {fl:x} {a} {s} {b}")));
        st::FTANIM_8006EBA4 = Some(|_| {});
        st::FTANIM_ISFRAMESREMAINING = Some(|_| FRAMES);
        st::FT_80081D0C = Some(|_| AIR);
        st::FT_800827A0 = Some(|_| AIR);
        st::FTCOMMON_8007D7FC = Some(|_| log("d7fc".into()));
        t::ext::LB_8000B1CC = Some(|_, _, out| *out = Vec3 { x: 5.0, y: 6.0, z: 9.0 });
        t::ext::IT_802BD158 = Some(|_, pos, smash, fd| log(format!("bomber {} {} {} {smash} {fd}", (*pos).x, (*pos).y, (*pos).z)));
    }

    // Enter: x21EC reset hook + start motion; the reset reads the smash stick threshold.
    da.x34 = 3.0;
    da.x40 = 4.0;
    da.x30 = 50;
    f.facing_dir = -1.0;
    t::ftPe_SpecialS_Enter(gp);
    assert_eq!(take(), [format!("ms {ftPe_MS_SpecialSStart} 0 0 1 0")]);
    assert_eq!(f.gr_vel, -3.0);
    f.active_sticky.lstick.x = 100;
    unsafe { f.x21EC.unwrap()(gp) };
    assert_eq!(unsafe { f.mv.pe.specials.x0 }, 0);
    f.active_sticky.lstick.x = 10;
    unsafe { f.x21EC.unwrap()(gp) };
    assert_eq!(unsafe { f.mv.pe.specials.x0 }, 1);
    t::ftPe_SpecialAirS_Enter(gp);
    assert_eq!(take(), [format!("ms {ftPe_MS_SpecialAirSStart} 0 0 1 0")]);
    assert_eq!(f.self_vel.y, 4.0);

    // Start anim end on the ground, no wall hit: becomes airborne at the fused hop offset, then jumps.
    da.specials_vel_x = 2.0;
    da.specials_smash_vel_x = 7.0;
    da.specials_vel_y = 8.0;
    f.cmd_vars = [0; 4];
    f.x34_scale.y = 1.25;
    f.cur_pos = Vec3 { x: 10.0, y: 20.0, z: 0.0 };
    unsafe { FRAMES = 1 };
    t::ftPe_SpecialSStart_Anim(gp);
    assert!(take().is_empty());
    unsafe { FRAMES = 0 };
    t::ftPe_SpecialSStart_Anim(gp);
    assert_eq!(f.cur_pos.x, fmadds(-4.0 * -1.0, 1.25, 10.0));
    assert_eq!(f.cur_pos.y, fmadds(3.5, 1.25, 20.0));
    assert_eq!(f.self_vel.x, 7.0 * -1.0); // smash flag set above (stick 10 < 50)
    assert_eq!(f.self_vel.y, 8.0);
    assert_eq!(take(), [format!("ms {ftPe_MS_SpecialAirSJump} 0 0 1 0")]);
    assert!(f.x21F8.is_some() && f.hurtbox_detect_cb.is_some());

    // Wall ahead while the start runs: speed zeroed and the end is requested.
    f.facing_dir = 1.0;
    f.coll_data.env_flags = Collide_LeftWallMask & 0x1;
    f.gr_vel = 5.0;
    unsafe { AIR = 1 };
    t::ftPe_SpecialSStart_Coll(gp);
    assert_eq!(f.gr_vel, 0.0);
    assert_eq!(f.cmd_vars[0], 1);

    // Jump anim end -> air end (smash) motion; the wall request divides the velocity.
    da.specials_end_vel_x = 1.5;
    da.specials_end_vel_y = 2.5;
    da.x68 = 2.0;
    da.x6C = 4.0;
    f.self_vel = Vec3 { x: 8.0, y: 8.0, z: 0.0 };
    f.cmd_vars = [0, 0, 1, 0];
    unsafe { FRAMES = 0 };
    take();
    t::ftPe_SpecialAirSJump_Anim(gp);
    let l = take();
    assert_eq!(l[0], format!("ms {ftPe_MS_SpecialAirSEnd_1} 0 0 1 0"));
    // doPostEnd with the flag: the bomber burst at the hip and the end velocities.
    assert_eq!(l[1], "bomber 5 6 0 1 1");
    assert_eq!((f.self_vel.x, f.self_vel.y), (1.5, 2.5));
    assert!(f.x21F8.is_some());
}

