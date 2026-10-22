//! Smoke test for tu::ftkoopaspecialhi (stateful; no fused sites) through recording hooks.
//! One #[test]: the hooks are process globals.
#![allow(static_mut_refs, unused_unsafe)]

use melee_chara::stubs as st;
use melee_chara::tu::ftkoopaspecialhi as t;
use melee_chara::tu::ftkoopaspecialn as tn;
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
    unsafe {
    let mut f: Box<Fighter> = Box::new(Fighter::zeroed());
    let mut g: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    let da: &'static mut ftKoopaAttributes = Box::leak(Box::new(ftKoopaAttributes::zeroed()));
    let mut parts: Box<[FighterBone; 4]> = Box::new(unsafe { core::mem::zeroed() });
    f.dat_attrs = da as *mut _ as *mut _;
    f.parts = parts.as_mut_ptr();
    g.user_data = &mut *f as *mut Fighter as *mut _;
    let gp: *mut HSD_GObj = &mut *g;
    unsafe {
        st::FIGHTER_CHANGEMOTIONSTATE = Some(|_, ms, fl, _, _, _, _| log(format!("ms {ms} {fl:x}")));
        st::FTANIM_8006EBA4 = Some(|_| log("6EBA4".into()));
        st::FTANIM_ISFRAMESREMAINING = Some(|_| 1);
        st::FT_8008A2BC = Some(|_| log("Wait".into()));
        st::FTCO_FALL_ENTER = Some(|_| log("Fall".into()));
        st::FTCO_80096900 = Some(|_, _, _, _, m, lag| log(format!("fallspecial {m} {lag}")));
        st::FT_80084F3C = Some(|_| log("groundphys".into()));
        st::FT_80084DB0 = Some(|_| log("airphys".into()));
        st::FT_80082708 = Some(|_| 1);
        st::FT_80081D0C = Some(|_| 1);
        st::FT_CHECKGROUNDANDLEDGE = Some(|_, d| {
            log(format!("ledge {d}"));
            0
        });
        st::FTCLIFFCOMMON_80081298 = Some(|_| 1);
        st::EFLIB_DESTROYALL = Some(|_| log("destroy".into()));
        st::FTPARTS_80074B0C = Some(|_, a, b| log(format!("parts {a} {b}")));
        t::ext::EFSYNC_SPAWN = Some(|id, _, args| {
            log(format!("fx {id:x} {}", args.len()));
            core::ptr::null_mut()
        });
        t::ext::FTPARTSETROTX = Some(|_, p, v| log(format!("rotx {p} {v}")));
        t::ext::FTCLIFFCOMMON_80081370 = Some(|_| log("cliffcatch".into()));
        tn::ext::FTCOLL_8007B128 = Some(|_, _, _| {});
        tn::ext::FTCOLL_8007B0C0 = Some(|_, _| {});
        tn::ext::FTCOMMON_8007F5CC = Some(|_, _| {});
    }

    // Enter (ground): motion 0x167, damage callbacks, effect spawned, jumps used from caps.
    f.co_attrs.max_jumps = 2;
    f.self_vel.y = 5.0;
    t::ftKp_SpecialHi_Enter(gp);
    assert_eq!(take(), "ms 359 0,fx 4da 1,6EBA4");
    assert_eq!((f.self_vel.y, f.x1968_jumpsUsed), (0.0, 2));
    assert_eq!(f.x2219_b0(), 1);
    assert!(f.take_dmg_cb.is_some() && f.death2_cb.is_some() && f.pre_hitlag_cb.is_some());

    // Anim (ground): the rising counter ticks, then the Wait fall-through at the end.
    f.mv.co.walk.middle_anim_frame = 3.0;
    f.cmd_vars[2] = 0;
    f.ground_or_air = GA_Ground;
    unsafe { st::FTANIM_ISFRAMESREMAINING = Some(|_| 0) };
    t::ftKp_SpecialHi_Anim(gp);
    assert_eq!(f.mv.co.walk.middle_anim_frame, 4.0);
    assert_eq!(take(), "parts 0 0,Wait");

    // Air Anim: script ends the move (cmd_vars[2] == 2) while airborne: effects destroyed, Fall.
    f.cmd_vars[2] = 2;
    f.ground_or_air = GA_Air;
    da.x7C = 0.0;
    t::ftKp_SpecialAirHi_Anim(gp);
    assert_eq!(f.x2219_b0(), 0);
    assert!(f.pre_hitlag_cb.is_none() && f.post_hitlag_cb.is_none());
    let got = take();
    assert!(got.ends_with("destroy,Fall"), "{got}");
    da.x7C = 12.0;
    f.cmd_vars[2] = 0;
    t::ftKp_SpecialAirHi_Anim(gp);
    assert!(take().ends_with("fallspecial 1 12"));

    // Phys: scripted ground slide vs the free phys.
    da.x68 = 0.0;
    f.cmd_vars[0] = 1;
    t::ftKp_SpecialHi_Phys(gp);
    t::ftKp_SpecialAirHi_Phys(gp);
    assert_eq!(take(), "groundphys,airphys");

    // Coll: the shell tilts to the floor normal while grounded-on-slope flags are set.
    unsafe {
        f.mv.kp.specials.x10 = 1;
    }
    f.cmd_vars[3] = 1;
    f.facing_dir = -1.0;
    f.coll_data.floor.normal = Vec3 { x: 0.0, y: 1.0, z: 0.0 };
    f.mv.co.walk.middle_anim_frame = 0.0;
    f.cmd_vars[2] = 0;
    f.ground_or_air = GA_Air;
    t::ftKp_SpecialHi_Coll_inline(gp);
    assert_eq!(take(), "rotx 0 -0");
    f.cmd_vars[3] = 0;
    t::ftKp_SpecialHi_Coll_inline(gp);
    assert_eq!(take(), "rotx 0 0");
    // Coll (grounded -> landing): the ground Coll swaps to the aerial-flagged state.
    f.cur_anim_frame = 7.0;
    unsafe { st::FT_80082708 = Some(|_| 0) };
    t::ftKp_SpecialHi_Coll(gp);
    let got = take();
    assert!(got.starts_with("ms 360 c4c5292"), "{got}");
    unsafe { st::FT_80082708 = Some(|_| 1) };
    t::ftKp_SpecialHi_Coll(gp);
    assert_eq!(unsafe { f.mv.kp.specials.x10 }, 1);
    take();

    // AirHi Coll while rising: ground contact swaps motion; none -> x10 cleared and ledge check.
    f.self_vel.y = 1.0;
    unsafe { st::FT_80081D0C = Some(|_| 0) };
    t::ftKp_SpecialAirHi_Coll(gp);
    assert_eq!(unsafe { f.mv.kp.specials.x10 }, 0);
    take();
    f.self_vel.y = -1.0;
    t::ftKp_SpecialAirHi_Coll(gp);
    assert_eq!(unsafe { f.mv.kp.specials.x10 }, 0);
    assert!(take().contains("ledge 0,cliffcatch"));
    }
}
