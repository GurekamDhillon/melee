//! tu::ftfoxspeciallw vs the C (gen/turef/tu_ftfoxspeciallw_ref.c, shipping contraction; data in
//! tests/data/tu_ftfoxspeciallw.txt; regenerate with
//! `python rust/melee_chara/gen/turef/tu_shard289.py tu_ftfoxspeciallw`): ftFx_SpecialLw_Turn and
//! ftFox_SpecialLw_Turn_Inline (the fused `-(MTXDegToRad(180 / x9C) - rotz)`), plus a smoke test of
//! the stateful rest through recording hooks. One #[test]: the hooks are process globals.
#![allow(static_mut_refs)]

use melee_chara::stubs as st;
use melee_chara::tu::ftfoxspeciallw as t;
use melee_types::*;

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

static mut LOG: std::vec::Vec<String> = std::vec::Vec::new();
static mut ROTZ: f32 = 0.0;
static mut ROTY: f32 = 0.0;
fn log(s: &str) {
    unsafe { LOG.push(s.to_string()) }
}
fn take() -> std::vec::Vec<String> {
    unsafe { core::mem::take(&mut LOG) }
}

#[test]
fn matches_c_and_smoke() {
    let want = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/tu_ftfoxspeciallw.txt")).unwrap();
    let mut f: Box<Fighter> = Box::new(Fighter::zeroed());
    let mut g: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    let da: &'static mut ftFox_DatAttrs = Box::leak(Box::new(ftFox_DatAttrs::zeroed()));
    f.dat_attrs = da as *mut _ as *mut _;
    g.user_data = &mut *f as *mut Fighter as *mut _;
    let gp: *mut HSD_GObj = &mut *g;
    unsafe {
        t::ext::FTPARTGETROTZ = Some(|_, _| ROTZ);
        t::ext::FTPARTSETROTY = Some(|_, _, v| ROTY = v);
    }
    let mut r = Rng(0x70E1F2);
    let mut n = 0;
    for (i, line) in want.lines().enumerate() {
        da.x9C_FOX_REFLECTOR_TURN_FRAMES = if r.rnd() & 3 != 0 {
            let a = (r.rnd() % 30) as f32;
            a + r.rf()
        } else {
            r.rf()
        };
        f.facing_dir = if r.rnd() & 1 != 0 { 1.0 } else { -1.0 };
        f.cmd_vars[0] = r.rnd() % 2;
        unsafe {
            f.mv.fx.SpecialLw.turnFrames = (r.rnd() % 40) as i32 - 5;
            ROTZ = r.rf() * 3.0;
            ROTY = 0.0;
        }
        if i & 1 != 0 {
            t::ftFx_SpecialLw_Turn(gp);
        } else {
            t::ftFox_SpecialLw_Turn_Inline(gp);
        }
        let got = format!(
            "{} {:08x} {:08x} {} {}",
            i,
            unsafe { ROTY }.to_bits(),
            f.facing_dir.to_bits(),
            unsafe { f.mv.fx.SpecialLw.turnFrames },
            f.cmd_vars[0]
        );
        assert_eq!(got, line, "case {i}");
        n += 1;
    }
    assert_eq!(n, 4000);

    // ---- smoke: the stateful remainder through recording hooks ----
    let mut parts: std::vec::Vec<FighterBone> = (0..80).map(|_| unsafe { core::mem::zeroed() }).collect();
    f.parts = parts.as_mut_ptr();
    let cd: &'static ftCommonData = Box::leak(Box::new(ftCommonData::zeroed()));
    *da = ftFox_DatAttrs::zeroed();
    unsafe {
        melee_fighter::hooks::P_FT_COMMON_DATA = cd;
        st::FIGHTER_CHANGEMOTIONSTATE = Some(|_, ms, fl, _, _, _, _| log(&format!("ms {ms} {fl:x}")));
        st::FTANIM_8006EBA4 = Some(|_| {});
        st::FTANIM_ISFRAMESREMAINING = Some(|_| 0);
        st::FTCOMMON_8007DB24 = Some(|_| log("DB24"));
        st::FT_80082708 = Some(|_| 0);
        st::FT_80081D0C = Some(|_| 1);
        st::FTCOLL_8007AEF8 = Some(|_| log("AEF8"));
        t::ext::EFSYNC_SPAWN = Some(|id, _, a| {
            log(&format!("ef {id} {}", a.len()));
            core::ptr::null_mut()
        });
        t::ext::FIGHTER_SETEFFECTHITLAGCALLBACKS = Some(|_| log("hitlag cbs"));
        t::ext::FTPARTS_GETBONEINDEX = Some(|_, p| p);
        t::ext::FTCOLL_CREATEREFLECTHIT = Some(|_, d, cb| log(&format!("reflect {} {}", (*d).x0_bone_id, cb.is_some())));
        t::ext::FTCO_80099F1C = Some(|_| false);
        t::ext::FTCO_JUMP_CHECKINPUT = Some(|_| false);
        t::ext::FTCO_800CB870 = Some(|_| false);
        t::ext::FTCO_800C97A8 = Some(|_| false);
        t::ext::FTCO_8009A184 = Some(|_, ms, fl, _| log(&format!("pass {ms} {fl:x}")));
        t::ext::LB_8000B1CC = Some(|_, _, out| *out = Vec3 { x: 1.0, y: 2.0, z: 3.0 });
        t::ext::LB_800119DC = Some(|p, a, b, c, d| log(&format!("spark {} {a} {b} {c} {d}", (*p).y)));
    }
    da.xB0_FOX_REFLECTOR_REFLECTION.x0_bone_id = 77;

    // Enter: SetVars from the DAT, start GFX accessory.
    da.x98_FOX_REFLECTOR_RELEASE_LAG = 9.6;
    da.xA4_FOX_REFLECTOR_GRAVITY_DELAY = 4;
    da.xA8_FOX_REFLECTOR_MOMENTUM_PRESERVE_X = 2.0;
    f.self_vel = Vec { x: 8.0, y: 3.0, z: 0.0 };
    t::ftFx_SpecialAirLw_Enter(gp);
    assert_eq!(take(), [format!("ms {} 0", ftFx_MS_SpecialAirLwStart)]);
    assert_eq!((f.self_vel.x, f.self_vel.y, f.cmd_vars[1]), (4.0, 0.0, 4));
    unsafe {
        assert_eq!((f.mv.fx.SpecialLw.releaseLag, f.mv.fx.SpecialLw.isRelease, f.mv.fx.SpecialLw.gravityDelay), (9, 0, 4));
        (f.accessory4_cb.unwrap())(gp);
    }
    assert_eq!(take(), ["ef 1161 1", "hitlag cbs"]);

    // Start anim: B released sets isRelease; frames over -> aerial loop (reflect bubble), loop GFX accessory.
    f.input.held_buttons[0] = 0;
    f.ground_or_air = GA_Air;
    t::ftFx_SpecialAirLwStart_Anim(gp);
    assert_eq!(unsafe { f.mv.fx.SpecialLw.isRelease }, 1);
    assert_eq!(take(), [format!("ms {} 2", ftFx_MS_SpecialAirLwLoop), "reflect 77 true".to_string(), "DB24".to_string()]);
    assert!(f.accessory4_cb.is_some());

    // Loop anim with the lag run out and B released -> End.
    unsafe { f.mv.fx.SpecialLw.releaseLag = 1 };
    t::ftFx_SpecialAirLwLoop_Anim(gp);
    assert_eq!(unsafe { f.mv.fx.SpecialLw.releaseLag }, 0);
    assert_eq!(take(), [format!("ms {} 0", ftFx_MS_SpecialAirLwEnd)]);

    // Air phys: gravity delay counts down, then falls.
    f.co_attrs.terminal_velocity = 5.0;
    da.xAC_FOX_REFLECTOR_FALL_ACCEL = 0.25;
    unsafe { f.mv.fx.SpecialLw.gravityDelay = 1 };
    f.self_vel.y = 0.0;
    t::ftFx_SpecialAirLwEnd_Phys(gp);
    assert_eq!((unsafe { f.mv.fx.SpecialLw.gravityDelay }, f.self_vel.y), (0, 0.0));
    t::ftFx_SpecialAirLwEnd_Phys(gp);
    assert_eq!(f.self_vel.y, -0.25);

    // Turn check starts a turn when the stick flips.
    unsafe { t::ext::FTCO_800C97A8 = Some(|_| true) };
    da.x9C_FOX_REFLECTOR_TURN_FRAMES = 8.0;
    unsafe { ROTZ = 0.0 };
    assert!(t::ftFx_SpecialLwTurn_Check(gp));
    assert_eq!(take()[0], format!("ms {} 2", ftFx_MS_SpecialAirLwTurn));
    assert_eq!((f.reflecting(), unsafe { f.mv.fx.SpecialLw.turnFrames }), (1, 7));
    assert!(f.reflect_hit_cb.is_some() && f.accessory4_cb.is_some());
    unsafe { t::ext::FTCO_800C97A8 = Some(|_| false) };
    assert!(!t::ftFx_SpecialLwTurn_Check(gp));

    // Hit_Check: lag over + released ends it; otherwise back to the loop with a new bubble.
    unsafe { f.mv.fx.SpecialLw.releaseLag = 0; f.mv.fx.SpecialLw.isRelease = 0 };
    assert!(t::ftFx_SpecialLwHit_Check(gp));
    assert_eq!(take(), [format!("ms {} 2", ftFx_MS_SpecialAirLwLoop), "reflect 77 true".to_string()]);
    unsafe { f.mv.fx.SpecialLw.isRelease = 1 };
    assert!(!t::ftFx_SpecialLwHit_Check(gp));

    // Hit_Enter: facing from the reflect, spark at the hip, hit state by ground/air.
    take();
    f.ReflectAttr.x1A2C_reflectHitDirection = -1.0;
    f.ground_or_air = GA_Ground;
    t::ftFx_SpecialLwHit_Enter(gp);
    assert_eq!(f.facing_dir, -1.0);
    let l = take();
    assert_eq!(l[0], format!("spark 2 120 3 {} {}", 0.1_f64 as f32, (core::f64::consts::PI / 3.0) as f32));
    assert_eq!(l[1], format!("ms {} 0", ftFx_MS_SpecialLwHit));
    assert!(f.reflect_hit_cb.is_some());
    f.set_x2219_b0(0);
    unsafe { (f.accessory4_cb.unwrap())(gp) };
    assert_eq!(take(), ["ef 1162 1", "hitlag cbs"]);

    // Start IASA: pass through the platform when the check says so.
    unsafe { t::ext::FTCO_80099F1C = Some(|_| true) };
    f.cur_anim_frame = 3.0;
    t::ftFx_SpecialLwStart_IASA(gp);
    assert_eq!(take()[0], format!("pass {} {:x}", ftFx_MS_SpecialAirLwStart, 0xC4C5082u32));
    // Loop phys / coll
    t::ftFx_SpecialAirLwLoop_Phys(gp);
    assert_eq!(take(), ["AEF8"]);
    unsafe { st::FT_80081D0C = Some(|_| 0) }; // still airborne: nothing happens
    t::ftFx_SpecialAirLwLoop_Coll(gp);
    assert!(take().is_empty());
}
