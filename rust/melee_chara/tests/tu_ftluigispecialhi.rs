//! tu::ftluigispecialhi vs the C (gen/turef/tu_ftluigispecialhi_ref.c, shipping contraction; data in
//! tests/data/tu_ftluigispecialhi.txt; regenerate with
//! `python rust/melee_chara/gen/turef/tu_shard345.py tu_ftluigispecialhi`): the IASA's momentum
//! angle, turn rotation and facing, the aerial Phys velocities, the aerial Enter velocity and the
//! freefall hand-over. Plus a smoke test of the collision dispatch.
#![allow(static_mut_refs)]

use melee_chara::stubs as st;
use melee_chara::tu::ftluigispecialhi as t;
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
            _ => ((self.rnd() % 4001) as i32 - 2000) as f32 / 997.0,
        }
    }
}

static mut ROT_N: i32 = 0;
static mut ROT: (i32, f32) = (-1, 0.0);
static mut FS: (f32, f32) = (0.0, 0.0);

#[test]
fn matches_c() {
    let _guard = LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let want = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/tu_ftluigispecialhi.txt")).unwrap();
    let mut lines = want.lines();
    let mut f: Box<Fighter> = Box::new(Fighter::zeroed());
    let mut g: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    let a: &'static mut ftLuigiAttributes = Box::leak(Box::new(ftLuigiAttributes::zeroed()));
    let cd: &'static mut ftCommonData = Box::leak(Box::new(unsafe { core::mem::zeroed() }));
    f.dat_attrs = a as *mut _ as *mut _;
    g.user_data = &mut *f as *mut Fighter as *mut _;
    let gp: *mut HSD_GObj = &mut *g;
    unsafe {
        melee_fighter::hooks::P_FT_COMMON_DATA = cd;
        t::ext::FTPARTSETROTY = Some(|_, part, v| {
            ROT_N += 1;
            ROT = (part, v);
        });
        st::FT_80085154 = Some(|_| {});
        st::FT_80084FA8 = Some(|_| {});
        st::FIGHTER_CHANGEMOTIONSTATE = Some(|_, _, _, _, _, _, _| {});
        st::FTANIM_8006EBA4 = Some(|_| {});
        st::FTANIM_ISFRAMESREMAINING = Some(|_| 0);
        st::FTCO_80096900 = Some(|_, _, _, _, mob, lag| FS = (mob, lag));
    }
    let mut r = Rng(0x1A9D0F3);
    for i in 0..4000 {
        // IASA
        f.input.lstick[0].x = r.rf();
        a.x5C_LUIGI_SUPERJUMP_MOMENTUM_STICK_RANGE = r.rf();
        a.x60_LUIGI_SUPERJUMP_ANGLE_DIFF = r.rf() * 60.0;
        a.x58_LUIGI_SUPERJUMP_REVERSE_STICK_RANGE = r.rf();
        f.cmd_vars[0] = r.rnd() & 1;
        f.lstick_angle = r.rf();
        f.facing_dir = if r.rnd() & 1 != 0 { 1.0 } else { -1.0 };
        unsafe { f.anon_2210.anon_0.set_throw_flags_b3((r.rnd() & 1) as u8) };
        unsafe {
            ROT_N = 0;
            ROT = (-1, 0.0);
        }
        t::ftLg_SpecialHi_IASA(gp);
        let got = unsafe {
            format!(
                "I {} {:08x} {} {:08x} {} {:08x} {}",
                i,
                f.lstick_angle.to_bits(),
                f.anon_2210.anon_0.throw_flags_b3(),
                f.facing_dir.to_bits(),
                ROT_N,
                ROT.1.to_bits(),
                ROT.0
            )
        };
        assert_eq!(got, lines.next().unwrap(), "case {i} I");
        // Phys
        a.x6C_LUIGI_SUPERJUMP_VEL_Y = r.rf();
        a.x68_LUIGI_SUPERJUMP_GRAVITY_START = r.rf();
        f.co_attrs.terminal_velocity = r.rf();
        f.co_attrs.aerial_friction = r.rf();
        cd.aerial_friction_oob = r.rf();
        f.co_attrs.air_drift_max = r.rf();
        f.cmd_vars[0] = r.rnd() & 1;
        f.self_vel.x = r.rf();
        f.self_vel.y = r.rf();
        f.self_vel.z = r.rf();
        t::ftLg_SpecialAirHi_Phys(gp);
        let got = format!(
            "P {} {:08x} {:08x} {:08x} {:08x}",
            i,
            f.self_vel.x.to_bits(),
            f.self_vel.y.to_bits(),
            f.self_vel.z.to_bits(),
            f.x74_self_accel.x.to_bits()
        );
        assert_eq!(got, lines.next().unwrap(), "case {i} P");
        // Enter
        a.x64_LUIGI_SUPERJUMP_VEL_X = r.rf();
        f.self_vel.x = r.rf();
        f.self_vel.y = r.rf();
        t::ftLg_SpecialAirHi_Enter(gp);
        let got = format!("E {} {:08x} {:08x}", i, f.self_vel.x.to_bits(), f.self_vel.y.to_bits());
        assert_eq!(got, lines.next().unwrap(), "case {i} E");
        // Anim
        a.x50_LUIGI_SUPERJUMP_FREEFALL_MOBILITY = r.rf();
        a.x54_LUIGI_SUPERJUMP_LANDING_LAG = r.rf();
        unsafe { FS = (123.0, 123.0) };
        t::ftLg_SpecialHi_Anim(gp);
        let got = unsafe { format!("N {} {:08x} {:08x}", i, FS.0.to_bits(), FS.1.to_bits()) };
        assert_eq!(got, lines.next().unwrap(), "case {i} N");
    }
    assert!(lines.next().is_none());
}

static mut LOG: std::vec::Vec<String> = std::vec::Vec::new();
fn log(s: String) {
    unsafe { LOG.push(s) }
}

#[test]
fn smoke_coll() {
    let _guard = LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let mut f: Box<Fighter> = Box::new(Fighter::zeroed());
    let mut g: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    let a: &'static mut ftLuigiAttributes = Box::leak(Box::new(ftLuigiAttributes::zeroed()));
    f.dat_attrs = a as *mut _ as *mut _;
    g.user_data = &mut *f as *mut Fighter as *mut _;
    let gp: *mut HSD_GObj = &mut *g;
    unsafe {
        t::ext::FT_80083B68 = Some(|_| log("83B68".into()));
        t::ext::FT_800831CC = Some(|_, _, _| log("831CC".into()));
        st::FT_80084104 = Some(|_| log("84104".into()));
    }
    // Air with the dive not started, or still rising: free air collision.
    f.ground_or_air = GA_Air;
    f.cmd_vars[0] = 0;
    t::ftLg_SpecialHi_Coll(gp);
    f.cmd_vars[0] = 1;
    f.self_vel.y = 1.0;
    t::ftLg_SpecialAirHi_Coll(gp);
    // Falling after the hit frame: the landing / cliff variant.
    f.self_vel.y = -1.0;
    t::ftLg_SpecialHi_Coll(gp);
    // On the ground.
    f.ground_or_air = GA_Ground;
    t::ftLg_SpecialHi_Coll(gp);
    assert_eq!(unsafe { LOG.join(",") }, "83B68,83B68,831CC,84104");
}
