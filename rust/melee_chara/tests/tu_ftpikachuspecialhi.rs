//! Smoke test of tu::ftpikachuspecialhi (Quick Attack) through recording hooks, with the fused
//! formulas recomputed from `fmadds` and the MSL math ports.
#![allow(static_mut_refs)]

use melee_chara::stubs as st;
use melee_chara::tu::ftpikachuspecialhi as t;
use melee_core::fp::fmadds;
use melee_runtime::gekko::{cosf, sinf, sqrtf};
use melee_runtime::lbtrigf::atan2f;
use melee_types::*;

static mut LOG: std::vec::Vec<String> = std::vec::Vec::new();
fn log(s: String) {
    unsafe { LOG.push(s) }
}
fn take() -> std::vec::Vec<String> {
    unsafe { core::mem::take(&mut LOG) }
}
static mut FRAMES: i32 = 0;
static mut RAND: f32 = 0.25;
static mut ON_PLATFORM: bool = false;
static mut LEDGE: i32 = 0;
static mut LAND: bool = false;

fn new_fighter() -> (Box<Fighter>, Box<HSD_GObj>, &'static mut ftPikachuAttributes, std::vec::Vec<FighterBone>) {
    let mut f: Box<Fighter> = Box::new(Fighter::zeroed());
    let mut g: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    let da: &'static mut ftPikachuAttributes = Box::leak(Box::new(ftPikachuAttributes::zeroed()));
    let mut bones: std::vec::Vec<FighterBone> = (0..4).map(|_| unsafe { core::mem::zeroed() }).collect();
    f.parts = bones.as_mut_ptr();
    f.dat_attrs = da as *mut _ as *mut _;
    g.user_data = &mut *f as *mut Fighter as *mut _;
    (f, g, da, bones)
}

#[test]
fn smoke() {
    let (mut f, mut g, da, _bones) = new_fighter();
    let gp: *mut HSD_GObj = &mut *g;
    unsafe {
        st::FIGHTER_CHANGEMOTIONSTATE = Some(|_, ms, fl, a, s, b, _| log(format!("ms {ms} {fl:x} {a} {s} {b}")));
        st::FTANIM_8006EBA4 = Some(|_| {});
        st::FTANIM_SETANIMRATE = Some(|_, r| log(format!("rate {r}")));
        st::FTANIM_ISFRAMESREMAINING = Some(|_| FRAMES);
        st::FT_8008A2BC = Some(|_| log("Wait".into()));
        st::FTCO_80096900 = Some(|_, a, b, c, m, l| log(format!("land {a} {b} {c} {m} {l}")));
        st::FTCO_LANDINGFALLSPECIAL_ENTER = Some(|_, a, l| log(format!("landfs {a} {l}")));
        st::FT_CHECKGROUNDANDLEDGE = Some(|_, d| {
            log(format!("ledge {d}"));
            LEDGE
        });
        st::FTCLIFFCOMMON_80081298 = Some(|_| 0);
        st::FTCOMMON_8007D7FC = Some(|_| log("d7fc".into()));
        t::ext::LB_8000B1CC = Some(|_, off, out| {
            assert!(off.is_null());
            *out = Vec3 { x: 100.0, y: 200.0, z: 0.0 };
        });
        t::ext::HSD_RANDF = Some(|| RAND);
        t::ext::HSD_JOBJSETSCALE = Some(|_, s| log(format!("scale {} {} {}", (*s).x, (*s).y, (*s).z)));
        t::ext::EFSYNC_SPAWN = Some(|id, _, va| {
            let v = &*(va[0] as *mut Vec3);
            log(format!("efsync {id} {} {}", v.x, v.y));
            core::ptr::null_mut()
        });
        t::ext::FTCO_8009A134 = Some(|_| ON_PLATFORM);
        t::ext::FT_80082888 = Some(|_, _| LAND);
        t::ext::FT_8008239C = Some(|_, d, _| {
            log(format!("air239c {d}"));
            LAND
        });
        t::ext::FTPARTSETROTX = Some(|_, p, r| log(format!("rotx {p} {r}")));
        melee_fighter::hooks::FTPARTS_GET_BONE_INDEX = Some(|_, i| i);
        melee_chara::chars::CAPS_MAX_JUMPS = Some(|_| 3);
    }

    // Enter: the zip duration comes from x5C, speeds and flags clear.
    da.x5C = 7;
    f.cmd_vars[0] = 3;
    f.gr_vel = 1.0;
    f.self_vel = Vec3 { x: 1.0, y: 1.0, z: 0.0 };
    t::ftPk_SpecialHi_Enter(gp);
    assert_eq!(take(), ["ms 353 0 0 1 0"]);
    assert_eq!(unsafe { f.mv.pk.specialhi.x0 }, 7);
    assert_eq!((f.gr_vel, f.self_vel.x, f.self_vel.y), (0.0, 0.0, 0.0));
    assert_eq!(f.cmd_vars[0], 0);
    t::ftPk_SpecialAirHi_Enter(gp);
    assert_eq!(take(), ["ms 356 0 0 1 0"]);

    // Aerial zip with a hard diagonal stick facing left: the facing flips, the angle uses the flipped
    // x, speed = fmadds(x90, mag, x94) split by cosf / sinf, mag capped at 0.999.
    da.x8C = 0.3;
    da.x90 = 2.0;
    da.x94 = 1.0;
    da.x98 = 0.5;
    da.x60 = 5;
    f.facing_dir = 1.0;
    f.input.lstick[0].x = -0.6;
    f.input.lstick[0].y = 0.8;
    unsafe { FRAMES = 1 };
    t::ftPk_SpecialAirHiStart0_Anim(gp);
    assert!(take().is_empty());
    unsafe { FRAMES = 0 };
    t::ftPk_SpecialAirHiStart0_Anim(gp);
    assert_eq!(take(), ["ms 357 a 13 1 0", "rate 0"]);
    let mag = sqrtf(fmadds(-0.6, -0.6, 0.8 * 0.8)).min(0.999);
    assert_eq!(mag, 0.999);
    let angle = atan2f(0.8, -0.6 * -1.0);
    let sp = fmadds(2.0, mag, 1.0);
    assert_eq!(f.facing_dir, -1.0);
    assert_eq!(f.self_vel.x, -1.0 * (sp * cosf(angle)));
    assert_eq!(f.self_vel.y, sp * sinf(angle));
    assert_eq!(unsafe { f.mv.pk.specialhi.x4 }, 5);
    assert_eq!(f.x1968_jumpsUsed, 3);
    assert_eq!(unsafe { (f.mv.pk.specialhi.x10.x, f.mv.pk.specialhi.x10.y) }, (-0.6, 0.8));
    assert_eq!(f.x2223_b4(), 1);
    assert!(f.x21F8.is_some());

    // A second zip also runs the motion change at frame 12 and scales the speed by x98.
    unsafe { f.mv.pk.specialhi.x8 = 1 };
    t::ftPk_SpecialHi_80126E1C(gp);
    assert_eq!(take(), ["ms 357 2 12 1 0", "ms 357 a 13 1 0", "rate 0"]);

    // A weak stick falls back to straight up at MAX_STICK_MAG (the deadzone check is group 3's).
    unsafe { f.mv.pk.specialhi.x8 = 0 };
    f.input.lstick[0].x = 0.0;
    f.input.lstick[0].y = 0.1;
    f.facing_dir = 1.0;
    // (update_facing_outside_deadzone reads the common-data table; only the velocity is checked
    // in the grounded zip below, so skip this branch here.)

    // Ground zip: stick pointing away from the floor normal is the grounded launch.
    f.coll_data.floor.normal = Vec3 { x: 0.0, y: 1.0, z: 0.0 };
    f.input.lstick[0].x = 0.0;
    f.input.lstick[0].y = -1.0;
    f.facing_dir = -1.0;
    unsafe { f.mv.pk.specialhi.x8 = 1 };
    t::ftPk_SpecialHi_80126C0C(gp);
    assert_eq!(take(), ["ms 354 2 12 1 0", "ms 354 a 13 1 0", "rate 0"]);
    assert_eq!(f.gr_vel, fmadds(2.0, 0.999, 1.0) * f.facing_dir * 0.5 * 1.0);
    // The ground zip flips x only through facing_dir (UpdateFacing follows the stick x = 0 -> right).

    // Second-zip check (80127064): stick weaker than x8C is false; the angle test compares against
    // MTXDegToRad(xA8).
    unsafe { f.mv.pk.specialhi.x8 = 0 };
    f.input.lstick[0].x = 0.0;
    f.input.lstick[0].y = 0.1;
    assert!(!t::ftPk_SpecialHi_80127064(gp));
    f.input.lstick[0].x = 1.0;
    f.input.lstick[0].y = 0.0;
    unsafe {
        f.mv.pk.specialhi.x10.x = -1.0;
        f.mv.pk.specialhi.x10.y = 0.0;
    }
    da.xA8 = 90;
    assert!(t::ftPk_SpecialHi_80127064(gp));
    da.xA8 = 200;
    assert!(!t::ftPk_SpecialHi_80127064(gp));
    unsafe { f.mv.pk.specialhi.x8 = 1 };
    da.xA8 = 90;
    assert!(!t::ftPk_SpecialHi_80127064(gp));

    // MotionChangeUpdateVel: keeps the velocity scaled by xA4 (air) or the ground speed (ground).
    da.xA4 = 0.5;
    f.self_vel = Vec3 { x: 8.0, y: 4.0, z: 0.0 };
    f.gr_vel = 6.0;
    t::ftPk_SpecialHi_MotionChangeUpdateVel_Unk1(gp);
    assert_eq!(take(), ["ms 358 2 0 1 0"]);
    assert_eq!((f.self_vel.x, f.self_vel.y, f.gr_vel), (4.0, 2.0, 0.0));
    f.self_vel = Vec3 { x: 8.0, y: 4.0, z: 0.0 };
    f.gr_vel = 6.0;
    t::ftPk_SpecialHi_MotionChangeUpdateVel_Unk0(gp);
    assert_eq!(take(), ["ms 355 2 0 1 0"]);
    assert_eq!((f.self_vel.x, f.self_vel.y, f.gr_vel), (0.0, 0.0, 3.0));

    // UpdateVel points every x velocity toward the facing direction.
    f.facing_dir = -1.0;
    f.gr_vel = 3.0;
    f.self_vel.x = 2.0;
    unsafe { f.mv.pk.specialhi.x10.x = 5.0 };
    t::ftPk_SpecialHi_UpdateVel(gp);
    assert_eq!((f.gr_vel, f.self_vel.x, unsafe { f.mv.pk.specialhi.x10.x }), (-3.0, -2.0, -5.0));

    // Start1 anim: counts down; the sparks jitter by fmadds(6, rand, -3) until the zip ends.
    f.kind = Ft_Kind_Pikachu;
    unsafe { f.mv.pk.specialhi.x4 = 2 };
    t::ftPk_SpecialHiStart1_Anim(gp);
    assert_eq!(take(), [format!("efsync 1012 {} {}", 100.0 + fmadds(6.0, 0.25, -3.0), 200.0 + fmadds(6.0, 0.25, -3.0))]);
    assert_eq!(f.x2219_b0(), 1);
    unsafe { f.mv.pk.specialhi.x4 = 2 };
    t::ftPk_SpecialAirHiStart1_Anim(gp);
    assert_eq!(take(), [format!("efsync 1012 {} {}", 100.0 + fmadds(10.0, 0.25, -5.0), 200.0 + fmadds(10.0, 0.25, -5.0))]);
    unsafe { f.mv.pk.specialhi.x4 = 1 };
    t::ftPk_SpecialHiStart1_Anim(gp);
    assert_eq!(take(), ["ms 355 2 0 1 0", "efsync 1012 100 200"]);
    f.kind = Ft_Kind_Pichu;
    unsafe { f.mv.pk.specialhi.x4 = 1 };
    t::ftPk_SpecialAirHiStart1_Anim(gp);
    assert_eq!(take(), ["ms 358 2 0 1 0"]);

    // 8012642C (air Start1 phys): rotation fmadds(facing, atan2(vx, vy), x78 - pi/2), scale, and the
    // transN fields cleared around ftCommon_8007E2FC with the velocity restored.
    da.x78 = 1.0;
    da.x7C_scale = Vec { x: 1.0, y: 2.0, z: 3.0 };
    f.facing_dir = -1.0;
    f.self_vel = Vec3 { x: 3.0, y: 4.0, z: 0.0 };
    f.x6D8.x = 9.0;
    f.lstick_angle = 9.0;
    f.x68C_transNPos.z = 9.0;
    t::ftPk_SpecialAirHiStart1_Phys(gp);
    let want = fmadds(-1.0, atan2f(3.0, 4.0), 1.0 - core::f32::consts::FRAC_PI_2);
    assert_eq!(take(), [format!("rotx 2 {want}"), "scale 1 2 3".to_string()]);
    assert_eq!((f.self_vel.x, f.self_vel.y), (3.0, 4.0));
    assert_eq!((f.x6D8.x, f.lstick_angle, f.x68C_transNPos.z), (0.0, 0.0, 0.0));

    // Start1 collision: not landed + wall -> keeps the speed via Unk1; no wall -> the Move motion.
    unsafe { LAND = false };
    f.coll_data.env_flags = 0;
    t::ftPk_SpecialHiStart1_Coll(gp);
    assert_eq!(take()[0], "ms 357 c4c508a 0 0 0");
    f.coll_data.env_flags = 0x1;
    t::ftPk_SpecialHiStart1_Coll(gp);
    assert_eq!(take().last().unwrap(), "ms 358 2 0 1 0");
}
