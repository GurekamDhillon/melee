//! tu::ftmariospeciallw vs the C (gen/turef/tu_ftmariospeciallw_ref.c, shipping contraction; data in
//! tests/data/tu_ftmariospeciallw.txt; regenerate with
//! `python rust/melee_chara/gen/turef/tu_shard345.py tu_ftmariospeciallw`): the Tornado's ground
//! and air physics, collision (floor-normal tilt, tap hand-over), Enter and the aerial Anim.
//! Plus a smoke test of the damage-callback handling.
#![allow(static_mut_refs)]

use melee_chara::stubs as st;
use melee_chara::tu::ftmariospeciallw as t;
use melee_types::*;

/// The hooks are process globals: the tests of this file run one at a time.
static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

struct Rng(u32);
impl Rng {
    fn rnd(&mut self) -> u32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 17;
        self.0 ^= self.0 << 5;
        self.0
    }
    fn rf(&mut self) -> f32 {
        match self.rnd() % 8 {
            0 => 0.0,
            1 => -0.0,
            2 => 1.0,
            3 => -1.0,
            _ => ((self.rnd() % 4001) as i32 - 2000) as f32 / 97.0,
        }
    }
}

static mut CMS_N: i32 = 0;
static mut CMS: (i32, u32, f32) = (0, 0, 0.0);
static mut ROT_N: i32 = 0;
static mut ROT_VAL: f32 = 0.0;
static mut FS_N: i32 = 0;
static mut FS: (f32, f32) = (0.0, 0.0);
static mut FALL_N: i32 = 0;
static mut FRAMES_LEFT: i32 = 0;
static mut R_GROUND: bool = false;
static mut R_AIR: bool = false;

fn install_cref_hooks() {
    unsafe {
        st::FIGHTER_CHANGEMOTIONSTATE = Some(|_, m, f, a, _, _, _| {
            CMS_N += 1;
            CMS = (m, f, a);
        });
        st::FTANIM_8006EBA4 = Some(|_| {});
        st::FTANIM_ISFRAMESREMAINING = Some(|_| FRAMES_LEFT);
        st::FTCO_FALL_ENTER = Some(|_| FALL_N += 1);
        st::FTCO_80096900 = Some(|_, _, _, _, mob, lag| {
            FS_N += 1;
            FS = (mob, lag);
        });
        st::FTCOMMON_8007D7FC = Some(|_| {});
        st::FT_8008A2BC = Some(|_| {});
        t::ext::FTPARTSETROTX = Some(|_, _, v| {
            ROT_N += 1;
            ROT_VAL = v;
        });
        t::ext::FT_80082888 = Some(|_, _| R_GROUND);
        t::ext::FT_800824A0 = Some(|_, _| R_AIR);
        t::ext::EFSYNC_SPAWN = Some(|_, _, _| core::ptr::null_mut());
    }
}

#[test]
fn matches_c() {
    let _guard = LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let want = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/tu_ftmariospeciallw.txt")).unwrap();
    let mut lines = want.lines();
    let mut f: Box<Fighter> = Box::new(Fighter::zeroed());
    let mut g: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    let a: &'static mut ftMario_DatAttrs = Box::leak(Box::new(ftMario_DatAttrs::zeroed()));
    f.dat_attrs = a as *mut _ as *mut _;
    g.user_data = &mut *f as *mut Fighter as *mut _;
    let gp: *mut HSD_GObj = &mut *g;
    install_cref_hooks();
    let mut r = Rng(0x4B1D77);
    for i in 0..4000 {
        // ground Phys
        a.speciallw.momentum_x = r.rf();
        a.speciallw.momentum_x_mul = r.rf();
        a.speciallw.friction_end = r.rf();
        a.speciallw.tap_y_vel_max = r.rf();
        a.speciallw.tap_grav = r.rf();
        a.speciallw.air_momentum_x = r.rf();
        unsafe { f.mv.mr.SpecialLw.groundVelX = r.rf() };
        f.cmd_vars[0] = r.rnd() & 1;
        f.cmd_vars[2] = r.rnd() & 1;
        f.input.lstick[0].x = r.rf();
        f.input.pressed_buttons = if r.rnd() & 1 != 0 { HSD_PAD_B as _ } else { 0 };
        f.gr_vel = r.rf();
        f.self_vel.y = r.rf();
        f.coll_data.floor.normal.x = r.rf();
        f.coll_data.floor.normal.y = r.rf();
        f.cur_anim_frame = r.rf();
        unsafe { CMS_N = 0 };
        t::ftMr_SpecialLw_Phys(gp);
        let got = unsafe {
            format!(
                "G {} {:08x} {:08x} {:08x} {:08x} {:08x} {:08x} {} {} {:08x} {}",
                i,
                f.gr_vel.to_bits(),
                f.xE4_ground_accel_1.to_bits(),
                f.self_vel.x.to_bits(),
                f.self_vel.y.to_bits(),
                f.x74_self_accel.x.to_bits(),
                f.mv.mr.SpecialLw.groundVelX.to_bits(),
                CMS_N,
                if CMS_N != 0 { CMS.0 } else { -1 },
                CMS.2.to_bits(),
                if CMS_N != 0 { CMS.1 } else { 0 }
            )
        };
        assert_eq!(got, lines.next().unwrap(), "case {i} G");
        // aerial Phys
        a.speciallw.air_momentum_x_mul = r.rf();
        f.co_attrs.gravity = r.rf();
        f.co_attrs.terminal_velocity = r.rf();
        unsafe { f.u.mr.x2234_tornadoCharge = (r.rnd() & 1) as i32 };
        f.cmd_vars[0] = r.rnd() & 1;
        f.cmd_vars[2] = r.rnd() & 1;
        f.self_vel.x = r.rf();
        f.self_vel.y = r.rf();
        unsafe { f.mv.mr.SpecialLw.groundVelX = r.rf() };
        t::ftMr_SpecialAirLw_Phys(gp);
        let got = unsafe {
            format!(
                "A {} {:08x} {:08x} {:08x} {:08x}",
                i,
                f.self_vel.x.to_bits(),
                f.self_vel.y.to_bits(),
                f.x74_self_accel.x.to_bits(),
                f.mv.mr.SpecialLw.groundVelX.to_bits()
            )
        };
        assert_eq!(got, lines.next().unwrap(), "case {i} A");
        // Coll
        unsafe {
            R_GROUND = r.rnd() & 1 != 0;
            R_AIR = r.rnd() & 1 != 0;
        }
        f.ground_or_air = if r.rnd() & 1 != 0 { GA_Ground } else { GA_Air };
        f.cmd_vars[3] = r.rnd() & 1;
        unsafe { f.mv.mr.SpecialLw.isUnkColl = (r.rnd() & 1) as i32 };
        f.facing_dir = if r.rnd() & 1 != 0 { 1.0 } else { -1.0 };
        f.coll_data.floor.normal.x = r.rf();
        f.coll_data.floor.normal.y = r.rf();
        f.self_vel.y = r.rf();
        unsafe {
            CMS_N = 0;
            ROT_N = 0;
        }
        t::ftMr_SpecialLw_Coll(gp);
        let got = unsafe {
            format!(
                "C {} {} {} {:08x} {} {:08x} {} {:08x}",
                i,
                CMS_N,
                f.mv.mr.SpecialLw.isUnkColl,
                f.self_vel.y.to_bits(),
                ROT_N,
                ROT_VAL.to_bits(),
                if CMS_N != 0 { CMS.0 } else { -1 },
                if CMS_N != 0 { CMS.1 } else { 0 }
            )
        };
        assert_eq!(got, lines.next().unwrap(), "case {i} C");
        unsafe {
            R_AIR = r.rnd() & 1 != 0;
            CMS_N = 0;
            ROT_N = 0;
            f.u.mr.x2234_tornadoCharge = 7;
        }
        t::ftMr_SpecialAirLw_Coll(gp);
        let got = unsafe {
            format!(
                "D {} {} {} {:08x} {} {:08x} {} {:08x} {}",
                i,
                CMS_N,
                f.mv.mr.SpecialLw.isUnkColl,
                f.self_vel.y.to_bits(),
                ROT_N,
                ROT_VAL.to_bits(),
                if CMS_N != 0 { CMS.0 } else { -1 },
                if CMS_N != 0 { CMS.1 } else { 0 },
                f.u.mr.x2234_tornadoCharge
            )
        };
        assert_eq!(got, lines.next().unwrap(), "case {i} D");
        // Enter
        a.speciallw.vel_y = r.rf();
        a.speciallw.unk0 = (r.rnd() % 20) as i32 - 5;
        unsafe { f.u.mr.x2234_tornadoCharge = (r.rnd() & 1) as i32 };
        f.self_vel.x = r.rf();
        f.self_vel.y = r.rf();
        t::ftMr_SpecialAirLw_Enter(gp);
        let got = unsafe {
            format!(
                "E {} {:08x} {:08x} {} {} {}",
                i,
                f.self_vel.x.to_bits(),
                f.self_vel.y.to_bits(),
                f.mv.mr.SpecialLw.unk,
                f.cmd_vars[2],
                f.x2219_b0()
            )
        };
        assert_eq!(got, lines.next().unwrap(), "case {i} E");
        f.self_vel.x = r.rf();
        f.self_vel.y = r.rf();
        t::ftMr_SpecialLw_Enter(gp);
        let got = format!("N {} {:08x} {:08x}", i, f.self_vel.x.to_bits(), f.self_vel.y.to_bits());
        assert_eq!(got, lines.next().unwrap(), "case {i} N");
        // AirLw_Anim
        a.speciallw.landing_lag = if r.rnd() % 3 == 0 { 0 } else { (r.rnd() % 30) as i32 };
        f.cmd_vars[1] = r.rnd() & 1;
        unsafe {
            FRAMES_LEFT = (r.rnd() & 1) as i32;
            FS_N = 0;
            FALL_N = 0;
            f.u.mr.x2234_tornadoCharge = 0;
        }
        t::ftMr_SpecialAirLw_Anim(gp);
        let got = unsafe {
            format!("L {} {} {} {} {:08x} {:08x} {}", i, FS_N, FALL_N, f.u.mr.x2234_tornadoCharge, FS.1.to_bits(), FS.0.to_bits(), f.cmd_vars[1])
        };
        assert_eq!(got, lines.next().unwrap(), "case {i} L");
    }
    assert!(lines.next().is_none());
}

#[test]
fn smoke_damage_callbacks() {
    let _guard = LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let mut f: Box<Fighter> = Box::new(Fighter::zeroed());
    let mut g: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    let a: &'static mut ftMario_DatAttrs = Box::leak(Box::new(ftMario_DatAttrs::zeroed()));
    f.dat_attrs = a as *mut _ as *mut _;
    g.user_data = &mut *f as *mut Fighter as *mut _;
    let gp: *mut HSD_GObj = &mut *g;
    install_cref_hooks();
    unsafe { FRAMES_LEFT = 1 };
    // Enter arms the rotation reset on damage and death; the callback zeroes the part rotation.
    t::ftMr_SpecialLw_Enter(gp);
    assert!(f.take_dmg_cb.is_some() && f.death2_cb.is_some());
    assert_eq!(f.x2219_b0(), 1);
    unsafe {
        ROT_N = 0;
        ROT_VAL = 1.0;
        f.take_dmg_cb.unwrap()(gp);
        assert_eq!((ROT_N, ROT_VAL), (1, 0.0));
    }
    // The animation running out clears them again and leaves the move.
    unsafe { FRAMES_LEFT = 0 };
    t::ftMr_SpecialLw_Anim(gp);
    assert!(f.take_dmg_cb.is_none() && f.death2_cb.is_none());
}
