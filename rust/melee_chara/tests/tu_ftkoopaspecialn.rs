//! tu::ftkoopaspecialn vs the C (gen/turef/tu_ftkoopaspecialn_ref.c: ftKp_SpecialLw_80134ACC with
//! its table and inline cut verbatim, shipping contraction; data in tests/data/tu_ftkoopaspecialn.txt;
//! regenerate with `python rust/melee_chara/gen/turef/tu_shard329.py tu_ftkoopaspecialn`), plus a
//! smoke test of the rest through recording hooks. One #[test]: hooks are process globals.
#![allow(static_mut_refs, unused_unsafe)]

use melee_chara::stubs as st;
use melee_chara::tu::ftkoopaspecialn as t;
use melee_fighter::hooks as fh;
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

static mut OUT: String = String::new();
static mut LBV: Vec3 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
static mut COUNTER: u32 = 0;
static mut RR: u32 = 12345;
static mut LOG: std::vec::Vec<String> = std::vec::Vec::new();
fn log(s: String) {
    unsafe { LOG.push(s) }
}
fn take() -> String {
    unsafe { core::mem::take(&mut LOG).join(",") }
}

#[test]
fn matches_c_and_smoke() {
    let want = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/tu_ftkoopaspecialn.txt")).unwrap();
    let mut f: Box<Fighter> = Box::new(Fighter::zeroed());
    let mut g: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    let da: &'static mut ftKoopaAttributes = Box::leak(Box::new(ftKoopaAttributes::zeroed()));
    let mut parts: Box<[FighterBone; 64]> = Box::new(unsafe { core::mem::zeroed() });
    g.user_data = &mut *f as *mut Fighter as *mut _;
    f.dat_attrs = da as *mut _ as *mut _;
    f.parts = parts.as_mut_ptr();
    let gp: *mut HSD_GObj = &mut *g;
    unsafe {
        fh::HSD_RANDI = Some(|n| {
            RR = RR.wrapping_mul(1664525).wrapping_add(1013904223);
            ((RR >> 8) % n as u32) as i32
        });
        fh::PLAY_SFX = Some(|_, id, vol, pan| OUT.push_str(&format!(" sfx {:x} {} {}", id, vol as u8, pan as u8)));
        t::ext::LB_8000B1CC = Some(|_, _, out| *out = LBV);
        t::ext::ITEM_8026AE60 = Some(|| {
            COUNTER += 1;
            COUNTER - 1
        });
        t::ext::FT_80089824 = Some(|_| OUT.push_str(" A"));
        t::ext::FT_800892A0 = Some(|_| OUT.push_str(" B"));
        t::ext::ITKOOPAFLAME_SPAWN = Some(|_, pos, facing, unk, gfx, speed, scale, kind| {
            OUT.push_str(&format!(
                " spawn {:08x} {:08x} {:08x} {} {} {} {} {}",
                (*pos).x.to_bits(),
                (*pos).y.to_bits(),
                facing.to_bits(),
                unk,
                gfx,
                speed,
                scale,
                kind
            ));
            core::ptr::null_mut()
        });
    }
    let mut r = Rng(0x329A11);
    let mut n = 0;
    for (i, line) in want.lines().enumerate() {
        unsafe {
            LBV = Vec3 { x: r.rf(), y: r.rf(), z: r.rf() };
            f.x34_scale.y = r.rf();
            f.facing_dir = if i & 1 != 0 { 1.0 } else { -1.0 };
            if i % 7 == 0 {
                f.facing_dir = r.rf();
            }
            da.x24 = r.rf();
            da.x28 = r.rf();
            da.x1C = r.rf() * 3.0;
            da.x18 = r.rf() * 5.0 + 7.0;
            f.u.kp.x222C = r.rf() * 40.0;
            f.u.kp.x2230 = r.rf() * 6.0 + 3.0;
            f.mv.kp.specialn.facing_dir = (r.rnd() % 4) as i32;
            f.mv.kp.specialn.x14 = (r.rnd() % 12) as i32;
            f.mv.kp.specialn.x4 = r.rnd() % 5;
            f.kind = if r.rnd() & 1 != 0 { Ft_Kind_Koopa } else { Ft_Kind_Mario } as _;
            OUT = format!("{i}");
        }
        t::ftKp_SpecialLw_80134ACC(gp);
        let got = unsafe { format!("{} | {} {}", OUT, f.mv.kp.specialn.x14, f.mv.kp.specialn.x4) };
        assert_eq!(got, line, "case {i}");
        n += 1;
    }
    assert_eq!(n, 3000);

    // ---- smoke: the rest through recording hooks ----
    unsafe {
        LOG.clear();
        st::FIGHTER_CHANGEMOTIONSTATE = Some(|_, ms, fl, _, _, _, _| log(format!("ms {ms} {fl:x}")));
        st::FTANIM_ISFRAMESREMAINING = Some(|_| 1);
        st::FTPARTS_80074B0C = Some(|_, a, b| log(format!("parts {a} {b}")));
        t::ext::FTCOLL_8007B128 = Some(|_, bone, state| log(format!("hurt {bone:x} {state}")));
        t::ext::FTCOLL_8007B0C0 = Some(|_, state| log(format!("hurtall {state}")));
        t::ext::FTCOMMON_8007F5CC = Some(|_, v| log(format!("5CC {v}")));
        t::ext::CAMERA_REQUESTQUAKE = Some(|k, _| log(format!("quake {k}")));
        COUNTER = 40;
    }
    // Enter: vars initialised, item counter taken.
    t::ftKp_SpecialN_Enter(gp);
    assert_eq!(take(), "ms 341 0");
    unsafe {
        assert_eq!((f.mv.kp.specialn.x0, f.mv.kp.specialn.x4, f.mv.kp.specialn.x10, f.mv.kp.specialn.x14), (0, 40, 1, 0));
    }
    t::ftKp_SpecialAirN_Enter(gp);
    assert_eq!(take(), "ms 344 0");

    // Anim loop: quake every da.x20 frames, x10 counts down on frame 0.
    da.x20 = 3;
    f.cur_anim_frame = 0.0;
    unsafe { f.mv.kp.specialn.x10 = 1 };
    unsafe { f.mv.kp.specialn.x18 = 0 };
    t::ftKp_SpecialN_Anim(gp);
    assert_eq!(take(), format!("quake {}", QuakeKind_Small));
    unsafe {
        assert_eq!((f.mv.kp.specialn.x10, f.mv.kp.specialn.x18), (0, 1));
    }
    t::ftKp_SpecialAirN_Anim(gp);
    t::ftKp_SpecialN_Anim(gp);
    assert_eq!(unsafe { f.mv.kp.specialn.x18 }, 0);
    assert_eq!(take(), "");
    t::ftKp_SpecialN_Anim(gp);
    assert_eq!(take(), format!("quake {}", QuakeKind_Small));

    // IASA: B held below the flame limit spawns every third call and drains the meters.
    da.flame_limit = 5;
    da.x14 = 0.5;
    da.x1C = 0.25;
    f.u.kp.x222C = 10.0;
    f.u.kp.x2230 = 0.3;
    unsafe {
        f.mv.kp.specialn.flame_timer = 0;
        f.mv.kp.specialn.x0 = 0;
        f.mv.kp.specialn.x14 = 1;
        f.mv.kp.specialn.x10 = 1;
        da.x18 = 4.0;
        f.input.held_buttons[0] = HSD_PAD_B as _;
        OUT.clear();
    }
    t::ftKp_SpecialN_IASA(gp);
    unsafe {
        assert!(OUT.contains("spawn"), "{}", OUT);
        assert_eq!(f.mv.kp.specialn.x0, 1);
        assert_eq!(f.mv.kp.specialn.flame_timer, 1);
        assert_eq!(f.u.kp.x222C, 9.0);
        assert_eq!(f.u.kp.x2230, 0.25);
    }
    // At the limit with B released and no spare charge: End state.
    unsafe {
        f.mv.kp.specialn.flame_timer = 5;
        f.mv.kp.specialn.x10 = 0;
        f.input.held_buttons[0] = 0;
    }
    t::ftKp_SpecialN_IASA(gp);
    assert_eq!(take(), "ms 343 0");
    t::ftKp_SpecialAirN_IASA(gp);
    assert_eq!(take(), "ms 346 0");
    assert_eq!(unsafe { f.mv.kp.specialn.flame_timer }, 5);

    // The attribute getters and the meter refill.
    da.x10 = 7.9;
    da.x18 = 9.5;
    assert_eq!((t::ftKp_SpecialLw_80134DE0(gp), t::ftKp_SpecialLw_80134E1C(gp)), (7, 9));
    assert_eq!((t::ftKp_SpecialLw_80134DE0(core::ptr::null_mut()), t::ftKp_SpecialLw_80134E1C(core::ptr::null_mut())), (1, 2));
    da.x8 = 2.0;
    da.xC = 3.0;
    f.u.kp.x222C = 6.5;
    f.u.kp.x2230 = 1.0;
    f.motion_id = 0x154;
    t::ftKp_SpecialLw_80134D78(gp);
    assert_eq!(unsafe { (f.u.kp.x222C, f.u.kp.x2230) }, (7.9, 4.0));
    f.motion_id = 0x157;
    t::ftKp_SpecialLw_80134D78(gp);
    assert_eq!(unsafe { (f.u.kp.x222C, f.u.kp.x2230) }, (7.9, 4.0));

    // 80135780: the script bit toggles intangibility of the twelve bones, grounded or airborne.
    f.ground_or_air = GA_Ground;
    f.cmd_vars[2] = 1;
    t::ftKp_SpecialN_80135780(gp);
    let got = take();
    assert!(got.starts_with("parts 0 1,hurt 37 2,hurt 30 2,hurt 3c 2,hurt 21 2,hurt 3d 2,hurt 22 2,hurt f 2,hurt 6 2,hurt 10 2,hurt 7 2,hurt 13 2,hurt a 2,5CC 0"), "{got}");
    assert_eq!(f.x221E_b4(), 0);
    f.ground_or_air = GA_Air;
    f.cmd_vars[1] = 2;
    t::ftKp_SpecialN_80135780(gp);
    assert_eq!(take(), "parts 0 0,hurtall 0,5CC 1");
    assert_eq!(f.x221E_b4(), 1);
    let _ = st::ftAnim_IsFramesRemaining;
}
