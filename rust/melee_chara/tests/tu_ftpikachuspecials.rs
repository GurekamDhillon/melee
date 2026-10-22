//! Smoke test of tu::ftpikachuspecials (Skull Bash) through recording hooks, with the fused
//! damage / launch-velocity formulas checked against `fmadds`.
#![allow(static_mut_refs)]

use melee_chara::stubs as st;
use melee_chara::tu::ftpikachuspecials as t;
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
    let da: &'static mut ftPikachuAttributes = Box::leak(Box::new(ftPikachuAttributes::zeroed()));
    f.dat_attrs = da as *mut _ as *mut _;
    g.user_data = &mut *f as *mut Fighter as *mut _;
    let gp: *mut HSD_GObj = &mut *g;
    unsafe {
        st::FIGHTER_CHANGEMOTIONSTATE = Some(|_, ms, fl, a, s, b, _| log(format!("ms {ms} {fl:x} {a} {s} {b}")));
        st::FTANIM_8006EBA4 = Some(|_| {});
        st::FTANIM_ISFRAMESREMAINING = Some(|_| FRAMES);
        st::FT_8008A2BC = Some(|_| log("Wait".into()));
        st::FTCO_FALL_ENTER = Some(|_| log("Fall".into()));
        st::FT_80081D0C = Some(|_| AIR);
        st::FT_80082708 = Some(|_| AIR);
        st::FTCOMMON_8007DB24 = Some(|_| log("db24".into()));
        st::FTCOMMON_8007D7FC = Some(|_| log("d7fc".into()));
        t::ext::FTCOLL_8007ABD0 = Some(|_, d, _| log(format!("hit {d}")));
    }

    // Enter: divides the speed by x30, stores the stick check, zero air y velocity.
    da.x30 = 2.0;
    f.gr_vel = 8.0;
    t::ftPk_SpecialS_Enter(gp);
    assert_eq!(take(), ["ms 343 0 0 1 0"]);
    assert_eq!(f.gr_vel, 4.0);
    assert!(f.x21EC.is_some());
    f.self_vel = Vec3 { x: 6.0, y: 3.0, z: 0.0 };
    t::ftPk_SpecialAirS_Enter(gp);
    assert_eq!(take(), ["ms 348 0 0 1 0"]);
    assert_eq!((f.self_vel.x, f.self_vel.y), (3.0, 0.0));

    // Start anim ends -> Hold motion (344) with the effect callback; air -> 349.
    unsafe { FRAMES = 1 };
    t::ftPk_SpecialSStart_Anim(gp);
    assert!(take().is_empty());
    unsafe { FRAMES = 0 };
    t::ftPk_SpecialSStart_Anim(gp);
    assert_eq!(take(), ["ms 344 200 0 1 0"]);
    assert!(f.accessory4_cb.is_some());
    t::ftPk_SpecialAirSStart_Anim(gp);
    assert_eq!(take(), ["ms 349 200 0 1 0"]);

    // Hold: counts up each frame; past x24 releases into the S0 motion (347 ground / 352 air).
    da.x24 = 2.0;
    unsafe { f.mv.pk.unk3.x0 = 0 };
    f.cmd_vars[0] = 9;
    t::ftPk_SpecialSHold_Anim(gp);
    assert_eq!(take(), ["db24"]);
    assert_eq!(unsafe { f.mv.pk.unk3.x0 }, 1);
    unsafe { FRAMES = 1 };
    t::ftPk_SpecialSHold_Anim(gp);
    t::ftPk_SpecialSHold_Anim(gp);
    assert_eq!(take(), ["ms 347 0 0 1 0", "db24"]);
    assert_eq!(f.cmd_vars[0], 0);
    t::ftPk_SpecialAirSHold_Anim(gp);
    assert_eq!(take(), ["ms 352 0 0 1 0", "db24"]);

    // IASA: releasing B ends the hold.
    f.input.held_buttons[0] = HSD_PAD_B as u32;
    t::ftPk_SpecialSHold_IASA(gp);
    assert!(take().is_empty());
    f.input.held_buttons[0] = 0;
    t::ftPk_SpecialSHold_IASA(gp);
    assert_eq!(take()[0], "ms 347 0 0 1 0");

    // S0 anim: damage = fmadds(x0, x2C, x28) while the hit capsule is enabled.
    da.x28 = 3.0;
    da.x2C = 0.5;
    unsafe { f.mv.pk.unk3.x0 = 7 };
    f.x914[0].state = HitCapsule_Enabled;
    f.cmd_vars[0] = 0;
    t::ftPk_SpecialS0_Anim(gp);
    assert_eq!(take(), [format!("hit {}", fmadds(7.0, 0.5, 3.0) as u32)]);
    // The script flag launches (Unk10): fused velocity, the facing flip, the callbacks.
    da.x3C = 2.0;
    da.x40 = 0.25;
    da.x44 = 6.0;
    da.x24 = 4.0;
    f.facing_dir = -1.0;
    f.cmd_vars[0] = 1;
    t::ftPk_SpecialS0_Anim(gp);
    let l = take();
    assert_eq!(l[..2], ["hit 6".to_string(), "ms 350 a 0 1 0".to_string()]);
    assert_eq!(f.self_vel.x, fmadds(0.25, 7.0, 2.0) * -1.0);
    let temp = 0.5f32 * 7.0 / 4.0;
    assert_eq!(f.self_vel.y, fmadds(0.5, 6.0, 6.0 * temp));
    assert!(f.x21F8.is_some() && f.deal_dmg_cb.is_some());
    assert_eq!(f.cmd_vars[0], 0);

    // ZeroVelocity (the deal-damage callback): x zeroed, upward y kept zero, then Unk12.
    da.x50 = 1.0;
    f.self_vel = Vec3 { x: 5.0, y: 2.0, z: 0.0 };
    t::ftPk_SpecialS_ZeroVelocity(gp);
    assert_eq!((f.self_vel.x, f.self_vel.y), (0.0, 0.0));
    assert_eq!(take(), ["ms 351 0 0 1 0"]);

    // Air S1 collision: landing goes to the End state with the speed divided by x50; wall -> Unk12.
    da.x50 = 2.0;
    f.gr_vel = 6.0;
    unsafe { AIR = 1 };
    f.coll_data.env_flags = 0;
    t::ftPk_SpecialAirS1_Coll(gp);
    assert_eq!(take(), ["d7fc", "ms 346 0 0 1 0"]);
    assert_eq!(f.gr_vel, 3.0);
    unsafe { AIR = 0 };
    f.coll_data.env_flags = Collide_RightWallMask & 0x40;
    f.self_vel.x = 8.0;
    t::ftPk_SpecialAirS1_Coll(gp);
    assert_eq!(take(), ["ms 351 0 0 1 0"]);
    assert_eq!(f.self_vel.x, 4.0);
    t::ftPk_SpecialAirSEnd_Coll(gp);
    assert!(take().is_empty());

    // S1 anim (air): frames over -> Unk12; the ground variants are empty.
    unsafe { FRAMES = 0 };
    t::ftPk_SpecialS1_Anim(gp);
    t::ftPk_SpecialAirS1_Anim(gp);
    assert_eq!(take(), ["ms 351 0 0 1 0"]);
}
