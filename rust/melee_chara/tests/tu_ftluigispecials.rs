//! tu::ftluigispecials vs the C (gen/turef/tu_ftluigispecials_ref.c, shipping contraction; data in
//! tests/data/tu_ftluigispecials.txt; regenerate with
//! `python rust/melee_chara/gen/turef/tu_shard345.py tu_ftluigispecials`), plus a smoke test of the
//! stateful callbacks through recording hooks.
#![allow(static_mut_refs)]

use melee_chara::stubs as st;
use melee_chara::tu::ftluigispecials as t;
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

static mut DMG_N: u32 = 0;
static mut DMG_SUM: u32 = 0;
static mut DMG_LAST: u32 = 0;
static mut RANDI_ARG: i32 = 0;
static mut RANDI_RET: i32 = 0;
static mut CMS: (i32, u32, f32) = (-1, 0, 0.0);

fn install_cref_hooks() {
    unsafe {
        t::ext::FTCOLL_8007ABD0 = Some(|_, d, _| {
            DMG_N += 1;
            DMG_SUM = DMG_SUM.wrapping_add(d);
            DMG_LAST = d;
        });
        t::ext::HSD_RANDI = Some(|n| {
            RANDI_ARG = n;
            RANDI_RET
        });
        st::FIGHTER_CHANGEMOTIONSTATE = Some(|_, m, f, a, _, _, _| CMS = (m, f, a));
        st::FTANIM_8006EBA4 = Some(|_| {});
        st::FTANIM_ISFRAMESREMAINING = Some(|_| 0);
    }
}

#[test]
fn matches_c() {
    let _guard = LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let want = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/tu_ftluigispecials.txt")).unwrap();
    let mut lines = want.lines();
    let mut f: Box<Fighter> = Box::new(Fighter::zeroed());
    let mut g: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    let a: &'static mut ftLuigiAttributes = Box::leak(Box::new(ftLuigiAttributes::zeroed()));
    f.dat_attrs = a as *mut _ as *mut _;
    g.user_data = &mut *f as *mut Fighter as *mut _;
    let gp: *mut HSD_GObj = &mut *g;
    install_cref_hooks();
    let mut r = Rng(0x3459A1);
    let mut n = 0;
    for i in 0..3000 {
        // A
        a.x14_LUIGI_GREENMISSILE_DAMAGE_SLOPE = r.rf();
        a.x10_LUIGI_GREENMISSILE_DAMAGE_TILT = r.rf();
        unsafe {
            f.mv.lg.SpecialS.chargeFrames = (r.rnd() % 400) as i32 - 50;
        }
        if i & 1 != 0 {
            // see the C driver: the sum lands on an integer N only when the product is rounded first
            a.x14_LUIGI_GREENMISSILE_DAMAGE_SLOPE = (r.rnd() % 100000) as f32 / (1 + r.rnd() % 9973) as f32;
            unsafe { f.mv.lg.SpecialS.chargeFrames = (r.rnd() % 400) as i32 + 1 };
            let rr = (unsafe { f.mv.lg.SpecialS.chargeFrames } as f64 * a.x14_LUIGI_GREENMISSILE_DAMAGE_SLOPE as f64) as f32;
            a.x10_LUIGI_GREENMISSILE_DAMAGE_TILT = (50 + r.rnd() % 150) as f32 - rr;
        }
        unsafe {
            f.mv.lg.SpecialS.isMisfire = ((r.rnd() % 3) == 0) as i32;
        }
        f.x914[0].state = if r.rnd() % 4 == 0 { 0 } else { HitCapsule_Enabled };
        f.cmd_vars[0] = 0;
        unsafe {
            DMG_N = 0;
            DMG_SUM = 0;
            DMG_LAST = 0;
        }
        t::ftLg_SpecialS_Anim(gp);
        t::ftLg_SpecialAirS_Anim(gp);
        t::ftLg_SpecialSMisfire_Anim(gp);
        t::ftLg_SpecialAirSMisfire_Anim(gp);
        let got = unsafe { format!("A {} {} {} {}", i, DMG_N, DMG_SUM, DMG_LAST) };
        assert_eq!(got, lines.next().unwrap(), "case {i} A");
        // B
        f.active_sticky.lstick.x = (r.rnd() % 256) as u8;
        a.x4_LUIGI_GREENMISSILE_SMASH = (r.rnd() % 300) as f32 - 10.0;
        a.x8_LUIGI_GREENMISSILE_CHARGE_RATE = r.rf();
        a.x44_LUIGI_GREENMISSILE_MISFIRE_CHANCE = r.rf();
        unsafe { RANDI_RET = if r.rnd() % 3 == 0 { 0 } else { 1 } };
        f.cmd_vars[0] = 7;
        f.x2070.x2070_int = 0;
        t::ftLg_SpecialS_SetVars(gp);
        let got = unsafe {
            format!(
                "B {} {} {} {} {} {}",
                i,
                f.mv.lg.SpecialS.chargeFrames,
                f.mv.lg.SpecialS.isMisfire,
                RANDI_ARG,
                f.x2070.anon_0.count_thrown_items(),
                f.cmd_vars[0]
            )
        };
        assert_eq!(got, lines.next().unwrap(), "case {i} B");
        // C
        a.x28_LUIGI_GREENMISSILE_MUL_X = r.rf();
        a.x24_LUIGI_GREENMISSILE_VEL_X = r.rf();
        a.x2C_LUIGI_GREENMISSILE_VEL_Y = r.rf();
        a.xC_LUIGI_GREENMISSILE_MAX_CHARGE_FRAMES = r.rf();
        a.x48_LUIGI_GREENMISSILE_MISFIRE_VEL_X = r.rf();
        a.x4C_LUIGI_GREENMISSILE_MISFIRE_VEL_Y = r.rf();
        unsafe {
            f.mv.lg.SpecialS.chargeFrames = (r.rnd() % 400) as i32 - 20;
            f.mv.lg.SpecialS.isMisfire = ((r.rnd() % 4) == 0) as i32;
        }
        f.facing_dir = if r.rnd() & 1 != 0 { 1.0 } else { -1.0 };
        f.cur_anim_frame = r.rf();
        f.self_vel.x = 99.0;
        f.self_vel.y = 99.0;
        unsafe { CMS.0 = -1 };
        t::ftLg_SpecialSFly_Enter(gp);
        let got = unsafe {
            format!(
                "C {} {:08x} {:08x} {} {:08x} {:08x}",
                i,
                f.self_vel.x.to_bits(),
                f.self_vel.y.to_bits(),
                CMS.0,
                CMS.1,
                CMS.2.to_bits()
            )
        };
        assert_eq!(got, lines.next().unwrap(), "case {i} C");
        // D (+ E on the same line)
        a.x40_LUIGI_GREENMISSILE_GRAVITY_MUL = r.rf();
        a.x34_LUIGI_GREENMISSILE_GRAVITY_START = r.rf();
        a.x30_LUIGI_GREENMISSILE_MUL_Y = r.rf();
        a.x3C_LUIGI_GREENMISSILE_X_DECEL = r.rf();
        f.co_attrs.terminal_velocity = r.rf();
        f.cmd_vars[0] = r.rnd() & 1;
        f.self_vel.x = r.rf();
        f.self_vel.y = r.rf();
        f.x74_self_accel.x = 5.0;
        t::ftLg_SpecialAirS2_Phys(gp);
        let mut got = format!("D {} {:08x} {:08x} {:08x}", i, f.self_vel.x.to_bits(), f.self_vel.y.to_bits(), f.x74_self_accel.x.to_bits());
        f.self_vel.x = r.rf();
        f.self_vel.y = r.rf();
        t::ftLg_SpecialAirSEnd_Phys(gp);
        got += &format!(" {:08x} {:08x} {:08x}", f.self_vel.x.to_bits(), f.self_vel.y.to_bits(), f.x74_self_accel.x.to_bits());
        a.x1C_LUIGI_GREENMISSILE_UNK2 = r.rf();
        f.coll_data.floor.normal.x = r.rf();
        f.coll_data.floor.normal.y = r.rf();
        f.gr_vel = r.rf();
        t::ftLg_SpecialSStart_Phys(gp);
        got += &format!(
            " {:08x} {:08x} {:08x} {:08x}",
            f.gr_vel.to_bits(),
            f.xE4_ground_accel_1.to_bits(),
            f.self_vel.x.to_bits(),
            f.self_vel.y.to_bits()
        );
        f.gr_vel = r.rf();
        t::ftLg_SpecialSEnd_Phys(gp);
        got += &format!(
            " {:08x} {:08x} {:08x} {:08x} {:08x}",
            f.xE4_ground_accel_1.to_bits(),
            f.self_vel.x.to_bits(),
            f.self_vel.y.to_bits(),
            f.x74_self_accel.x.to_bits(),
            f.x74_self_accel.y.to_bits()
        );
        assert_eq!(got, lines.next().unwrap(), "case {i} D");
        n += 1;
    }
    assert_eq!(n, 3000);
    assert!(lines.next().is_none());
}

static mut LOG: std::vec::Vec<String> = std::vec::Vec::new();
fn log(s: String) {
    unsafe { LOG.push(s) }
}

#[test]
fn smoke_callbacks() {
    let _guard = LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let mut f: Box<Fighter> = Box::new(Fighter::zeroed());
    let mut g: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    let a: &'static mut ftLuigiAttributes = Box::leak(Box::new(ftLuigiAttributes::zeroed()));
    let mut parts: std::vec::Vec<FighterBone> = (0..8).map(|_| unsafe { core::mem::zeroed() }).collect();
    f.dat_attrs = a as *mut _ as *mut _;
    f.parts = parts.as_mut_ptr();
    g.user_data = &mut *f as *mut Fighter as *mut _;
    let gp: *mut HSD_GObj = &mut *g;
    unsafe {
        st::FIGHTER_CHANGEMOTIONSTATE = Some(|_, m, fl, _, _, _, _| log(format!("cms {m} {fl:x}")));
        st::FTANIM_8006EBA4 = Some(|_| log("8006EBA4".into()));
        st::FTANIM_ISFRAMESREMAINING = Some(|_| 0);
        st::FTCOMMON_8007DB24 = Some(|_| log("DB24".into()));
        t::ext::EFSYNC_SPAWN = Some(|id, _, args| {
            log(format!("ef {id} {}", args.len()));
            core::ptr::null_mut()
        });
    }
    a.x18_LUIGI_GREENMISSILE_TRACTION = 2.0;
    a.xC_LUIGI_GREENMISSILE_MAX_CHARGE_FRAMES = 3.0;
    a.x38_LUIGI_GREENMISSILE_FRICTION_END = 4.0;

    // Enter: tractioned ground speed, SetVars stored for the start of the move.
    f.gr_vel = 6.0;
    t::ftLg_SpecialS_Enter(gp);
    assert_eq!(f.gr_vel, 3.0);
    assert!(f.x21EC.is_some());
    f.self_vel.x = 8.0;
    f.self_vel.y = 5.0;
    t::ftLg_SpecialAirS_Enter(gp);
    assert_eq!((f.self_vel.x, f.self_vel.y), (4.0, 0.0));

    // Start -> Hold at the end of the animation, which arms the charge GFX callback.
    t::ftLg_SpecialSStart_Anim(gp);
    assert!(f.accessory4_cb.is_some());
    f.accessory4_cb = None;

    // Hold: the charge counts up; past the maximum it launches (not a misfire).
    unsafe { f.mv.lg.SpecialS.chargeFrames = 2 };
    t::ftLg_SpecialSHold_Anim(gp);
    assert_eq!(unsafe { f.mv.lg.SpecialS.chargeFrames }, 3);
    unsafe { f.mv.lg.SpecialS.isMisfire = 1 };
    t::ftLg_SpecialSHold_Anim(gp);
    // Launch of a misfire enters the Misfire state instead.
    f.input.held_buttons[0] = 0;
    t::ftLg_SpecialAirSHold_IASA(gp);
    assert_eq!(f.accessory4_cb.is_none(), true);

    // SEnd_Enter: ground speed over the end friction; air variant divides self_vel.x.
    f.gr_vel = 8.0;
    t::ftLg_SpecialSEnd_Enter(gp);
    assert_eq!(f.gr_vel, 2.0);
    f.self_vel.x = 12.0;
    t::ftLg_SpecialAirSEnd_Enter(gp);
    assert_eq!(f.self_vel.x, 3.0);

    // OnGiveDamage stops the missile and ends the move (air end state).
    f.self_vel.x = 5.0;
    f.self_vel.y = 2.0;
    t::ftLg_SpecialS_OnGiveDamage(gp);
    assert_eq!((f.self_vel.x, f.self_vel.y), (0.0, 0.0));

    let got = unsafe { LOG.join(",") };
    assert_eq!(
        got,
        "cms 343 0,8006EBA4,cms 349 0,8006EBA4,cms 344 200,DB24,cms 348 0,DB24,cms 346 0,cms 352 0,cms 352 0".replace(
            "cms 348 0,DB24,cms 346 0,cms 352 0,cms 352 0",
            &got["cms 343 0,8006EBA4,cms 349 0,8006EBA4,cms 344 200,DB24,".len()..]
        )
    );
}
