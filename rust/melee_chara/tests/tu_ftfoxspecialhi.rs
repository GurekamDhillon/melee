//! tu::ftfoxspecialhi vs the C (gen/turef/tu_ftfoxspecialhi_ref.c, shipping contraction; data in
//! tests/data/tu_ftfoxspecialhi.txt; regenerate with
//! `python rust/melee_chara/gen/turef/tu_shard289.py tu_ftfoxspecialhi`): ftFx_SpecialAirHi_Phys
//! (both fused sites) and ftFx_SpecialAirHi_Enter (trig, stick tests, f32 -> int), plus a smoke
//! test of the stateful rest through recording hooks. One #[test]: the hooks are process globals.
#![allow(static_mut_refs)]

use melee_chara::stubs as st;
use melee_chara::tu::ftfoxspecialhi as t;
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
static mut ROTX: f32 = 0.0;
static mut MS: i32 = 0;
fn log(s: &str) {
    unsafe { LOG.push(s.to_string()) }
}
fn take() -> std::vec::Vec<String> {
    unsafe { core::mem::take(&mut LOG) }
}

#[test]
fn matches_c_and_smoke() {
    let want = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/tu_ftfoxspecialhi.txt")).unwrap();
    let mut f: Box<Fighter> = Box::new(Fighter::zeroed());
    let mut g: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    let da: &'static mut ftFox_DatAttrs = Box::leak(Box::new(ftFox_DatAttrs::zeroed()));
    f.dat_attrs = da as *mut _ as *mut _;
    g.user_data = &mut *f as *mut Fighter as *mut _;
    let gp: *mut HSD_GObj = &mut *g;
    unsafe {
        st::FIGHTER_CHANGEMOTIONSTATE = Some(|_, ms, _, _, _, _, _| MS = ms);
        st::FTANIM_8006EBA4 = Some(|_| {});
        t::ext::FTPARTS_GETBONEINDEX = Some(|_, p| p);
        t::ext::FTPARTSETROTX = Some(|_, _, v| ROTX = v);
    }
    let mut r = Rng(0xF0F1F2);
    let mut lines = want.lines();
    let mut n = 0;
    for i in 0..4000 {
        da.x64_FOX_FIREFOX_DIRECTION_STICK_RANGE_MIN = r.rf() * 0.5;
        da.x68_FOX_FIREFOX_DURATION = r.rf() * 20.0;
        da.x70_FOX_FIREFOX_DURATION_END = (r.rnd() % 6) as f32;
        da.x74_FOX_FIREFOX_SPEED = r.rf();
        da.x78_FOX_FIREFOX_REVERSE_ACCEL = r.rf();
        da.x88_FOX_FIREFOX_FACING_STICK_RANGE_MIN = r.rf() * 0.5;
        f.facing_dir = if r.rnd() & 1 != 0 { 1.0 } else { -1.0 };
        f.self_vel.x = r.rf();
        f.self_vel.y = r.rf();
        f.input.lstick[0].x = r.rf();
        f.input.lstick[0].y = r.rf();
        f.co_attrs.max_jumps = (r.rnd() % 6) as _;
        unsafe {
            f.mv.fx.SpecialHi.rotateModel = r.rf() * 3.0;
            f.mv.fx.SpecialHi.unk = (r.rnd() % 6) as i32;
            ROTX = 0.0;
            if i & 1 != 0 {
                t::ftFx_SpecialAirHi_Phys(gp);
                let got = format!("{} p {:08x} {:08x} {}", i, f.self_vel.x.to_bits(), f.self_vel.y.to_bits(), f.mv.fx.SpecialHi.unk);
                assert_eq!(got, lines.next().unwrap(), "case {i}");
            } else {
                t::ftFx_SpecialAirHi_Enter(gp);
                let got = format!(
                    "{} e {:08x} {:08x} {:08x} {:08x} {} {} {} {} {}",
                    i,
                    f.self_vel.x.to_bits(),
                    f.self_vel.y.to_bits(),
                    f.facing_dir.to_bits(),
                    f.mv.fx.SpecialHi.rotateModel.to_bits(),
                    f.mv.fx.SpecialHi.travelFrames,
                    f.mv.fx.SpecialHi.unk,
                    f.mv.fx.SpecialHi.unk2,
                    f.x1968_jumpsUsed,
                    MS
                );
                assert_eq!(got, lines.next().unwrap(), "case {i}");
                assert_eq!(format!("{} r {:08x}", i, ROTX.to_bits()), lines.next().unwrap(), "case {i} rot");
            }
        }
        n += 1;
    }
    assert_eq!(n, 4000);
    assert!(lines.next().is_none());

    // ---- smoke: the stateful remainder through recording hooks ----
    let mut parts: std::vec::Vec<FighterBone> = (0..80).map(|_| unsafe { core::mem::zeroed() }).collect();
    f.parts = parts.as_mut_ptr();
    *da = ftFox_DatAttrs::zeroed();
    f.facing_dir = 1.0;
    let cd: &'static ftCommonData = Box::leak(Box::new(ftCommonData::zeroed()));
    unsafe {
        melee_fighter::hooks::P_FT_COMMON_DATA = cd;
        st::FIGHTER_CHANGEMOTIONSTATE = Some(|_, ms, fl, _, _, _, _| log(&format!("ms {ms} {fl:x}")));
        st::FT_80082708 = Some(|_| 1);
        st::FT_CHECKGROUNDANDLEDGE = Some(|_, _| 0);
        st::FTCLIFFCOMMON_80081298 = Some(|_| 0);
        st::FTANIM_ISFRAMESREMAINING = Some(|_| 0);
        st::FT_8008A2BC = Some(|_| log("wait"));
        st::FTCO_80096900 = Some(|_, a, b, c, mob, lag| log(&format!("fallspecial {a} {b} {c} {mob} {lag}")));
        st::FTCOMMON_8007D7FC = Some(|_| log("7D7FC"));
        st::FTCOMMON_8007DB24 = Some(|_| log("DB24"));
        st::FT_800851C0 = Some(|_| log("851C0"));
        st::FT_80084F3C = Some(|_| log("84F3C"));
        t::ext::EFSYNC_SPAWN = Some(|id, _, a| {
            log(&format!("ef {id} {}", a.len()));
            core::ptr::null_mut()
        });
        t::ext::FIGHTER_SETEFFECTHITLAGCALLBACKS = Some(|_| log("hitlag cbs"));
        t::ext::FTCO_8009A134 = Some(|_| false);
    }
    // Enter / charge GFX.
    da.x54_FOX_FIREFOX_GRAVITY_DELAY = 3.7;
    da.x58_FOX_FIREFOX_VEL_X = 2.0;
    f.gr_vel = 8.0;
    t::ftFx_SpecialHi_Enter(gp);
    assert_eq!(take(), [format!("ms {} 0", ftFx_MS_SpecialHiHold)]);
    assert_eq!(unsafe { f.mv.fx.SpecialHi.gravityDelay }, 3);
    assert_eq!(f.gr_vel, 4.0);
    unsafe { (f.accessory4_cb.unwrap())(gp) };
    assert_eq!(take(), ["ef 1163 1", "hitlag cbs"]);
    assert!(f.x2219_b0() == 1 && f.accessory4_cb.is_none());
    f.set_x2219_b0(0);
    t::ftFx_SpecialHi_CreateLaunchGFX(gp);
    assert_eq!(take(), ["ef 1164 1", "hitlag cbs"]);

    // Hold air start: self_vel.x /= vel_x, y = 0.
    f.self_vel = Vec { x: 6.0, y: 5.0, z: 0.0 };
    t::ftFx_SpecialAirHiStart_Enter(gp);
    assert_eq!((f.self_vel.x, f.self_vel.y), (3.0, 0.0));
    take();

    // Hold anims: frames over -> aerial enters Air Hi, grounded goes via AirToGround.
    f.ground_or_air = GA_Air;
    f.input.lstick[0].x = 0.0;
    f.input.lstick[0].y = 0.0;
    t::ftFx_SpecialHiHold_Anim(gp);
    assert_eq!(take(), [format!("ms {} 0", ftFx_MS_SpecialAirHi)]);

    // Travel: counts frames down; at the end a landing/fall state is entered.
    unsafe { f.mv.fx.SpecialHi.travelFrames = 2 };
    t::ftFx_SpecialAirHi_Anim(gp);
    assert_eq!(unsafe { f.mv.fx.SpecialHi.travelFrames }, 1);
    assert!(take().is_empty());
    t::ftFx_SpecialHi_Anim(gp);
    assert_eq!(take(), ["DB24".to_string(), format!("ms {} 0", ftFx_MS_SpecialHiFall)]);

    // Fall end: landing state at frame 13; bound enter.
    t::ftFx_SpecialHiFall_Enter(gp);
    assert_eq!(take(), ["7D7FC".to_string(), format!("ms {} {:x}", ftFx_MS_SpecialHiLanding, Ft_MF_SkipColAnim | Ft_MF_UpdateCmd)]);
    da.x84_FOX_FIREFOX_BOUND_VEL_X = 0.5;
    f.self_vel.x = 4.0;
    f.cmd_vars[0] = 5;
    t::ftFx_SpecialHiBound_Enter(gp);
    assert_eq!(take(), [format!("ms {} 0", ftFx_MS_SpecialHiBound), "ef 1030 2".to_string(), "hitlag cbs".to_string()]);
    assert_eq!((f.self_vel.x, f.cmd_vars[0]), (2.0, 0));

    // Bound anim: script flag in the air -> freefall with the DAT mobility / landing lag.
    da.x8C_FOX_FIREFOX_FREEFALL_MOBILITY = 0.25;
    da.x90_FOX_FIREFOX_LANDING_LAG = 30.0;
    f.cmd_vars[0] = 1;
    f.co_attrs.max_jumps = 2;
    f.x1968_jumpsUsed = 0;
    t::ftFx_SpecialHiBound_Anim(gp);
    assert_eq!(take(), ["fallspecial 1 0 1 0.25 30"]);
    assert_eq!(f.x1968_jumpsUsed, 2);

    // Bound phys: air runs 851C0 + quick deaccel; ground 84F3C.
    f.ground_or_air = GA_Air;
    t::ftFx_SpecialHiBound_Phys(gp);
    assert_eq!(take(), ["851C0"]);
    f.ground_or_air = GA_Ground;
    t::ftFx_SpecialHiBound_Phys(gp);
    assert_eq!(take(), ["84F3C"]);

    // IsBound: bounce count reached -> true without asking the pass check.
    da.x6C_FOX_FIREFOX_BOUNCE_VAR = 2;
    unsafe { f.mv.fx.SpecialHi.unk2 = 2 };
    assert!(t::ftFox_SpecialHi_IsBound(gp));
    unsafe { f.mv.fx.SpecialHi.unk2 = 0 };
    assert!(t::ftFox_SpecialHi_IsBound(gp)); // ftCo_8009A134 false -> still bound

    // Coll (ground): a floor under Fox aims the model along its normal.
    f.coll_data.env_flags = Collide_FloorPush;
    f.coll_data.floor.normal = Vec { x: 0.0, y: 1.0, z: 0.0 };
    unsafe { t::ext::FTPARTS_GETBONEINDEX = Some(|_, p| p) };
    t::ftFx_SpecialHi_Coll(gp);
    assert_eq!(unsafe { f.mv.fx.SpecialHi.unk2 }, 1);
    assert_eq!(unsafe { f.mv.fx.SpecialHi.rotateModel }, 0.0);
    // Hi_Phys: the duration counter reaches DURATION_END -> ground deaccel path runs (no hooks needed).
    da.x70_FOX_FIREFOX_DURATION_END = 1.0;
    da.x78_FOX_FIREFOX_REVERSE_ACCEL = 0.0;
    unsafe { f.mv.fx.SpecialHi.unk = 0 };
    t::ftFx_SpecialHi_Phys(gp);
    assert_eq!(unsafe { f.mv.fx.SpecialHi.unk }, 1);
    assert_eq!(unsafe { ROTX }, 2.0 * (core::f64::consts::PI as f32));
}
