//! tu::ftkirbyspecialseak vs the C (gen/turef/tu_ftkirbyspecialseak_ref.c: fn_80106DB0 and
//! ftKb_SpecialNSk_8010603C cut verbatim, shipping contraction, 6 fused sites; data in
//! tests/data/tu_ftkirbyspecialseak.txt; regenerate with `python
//! rust/melee_chara/gen/turef/tu_shard329.py tu_ftkirbyspecialseak`), plus a smoke test of the
//! rest through recording hooks. One #[test]: hooks are process globals.
#![allow(static_mut_refs, unused_unsafe)]

use melee_chara::stubs as st;
use melee_chara::tu::ftkirbyspecialseak as t;
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
static mut RR: u32 = 777;
static mut NL: u32 = 99;
static mut NEEDLE: Option<Box<HSD_GObj>> = None;
static mut LOG: std::vec::Vec<String> = std::vec::Vec::new();
fn log(s: String) {
    unsafe { LOG.push(s) }
}
fn take() -> String {
    unsafe { core::mem::take(&mut LOG).join(",") }
}

#[test]
fn matches_c_and_smoke() {
    let want = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/tu_ftkirbyspecialseak.txt")).unwrap();
    let mut f: Box<Fighter> = Box::new(Fighter::zeroed());
    let mut g: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    let da: &'static mut ftKb_DatAttrs = Box::leak(Box::new(ftKb_DatAttrs::zeroed()));
    g.user_data = &mut *f as *mut Fighter as *mut _;
    f.dat_attrs = da as *mut _ as *mut _;
    let gp: *mut HSD_GObj = &mut *g;
    unsafe {
        NEEDLE = Some(Box::new(core::mem::zeroed()));
        fh::HSD_RANDI = Some(|n| {
            RR = RR.wrapping_mul(1664525).wrapping_add(1013904223);
            ((RR >> 8) % n as u32) as i32
        });
        fh::PLAY_SFX = Some(|_, id, vol, pan| OUT.push_str(&format!(" sfx {:x} {} {}", id, vol as u8, pan as u8)));
        t::ext::IT_802AFD8C = Some(|_, pos, kind, facing| {
            NL = NL.wrapping_mul(1103515245).wrapping_add(12345);
            OUT.push_str(&format!(
                " throw {:08x} {:08x} {:08x} {} {:08x}",
                (*pos).x.to_bits(),
                (*pos).y.to_bits(),
                (*pos).z.to_bits(),
                kind,
                facing.to_bits()
            ));
            if (NL >> 16) & 3 != 0 {
                &mut **NEEDLE.as_mut().unwrap() as *mut HSD_GObj
            } else {
                core::ptr::null_mut()
            }
        });
        t::ext::IT_802AFEA8 = Some(|_, _, flag| OUT.push_str(&format!(" arm {flag}")));
        t::ext::EFSYNC_SPAWN = Some(|id, _, args| {
            let p = args[0] as *const Vec3;
            OUT.push_str(&format!(" fx {:x} {:08x} {:08x} {:08x}", id, (*p).x.to_bits(), (*p).y.to_bits(), (*p).z.to_bits()));
            core::ptr::null_mut()
        });
    }
    let needle: *mut HSD_GObj = unsafe { &mut **NEEDLE.as_mut().unwrap() };
    let mut r = Rng(0x5EA6);
    let mut n = 0;
    for (i, line) in want.lines().enumerate() {
        unsafe {
            da.specialn_sk_graphic_x_offset_ground = r.rf();
            da.specialn_sk_graphic_y_offset_ground = r.rf();
            da.specialn_sk_graphic_x_offset_air = r.rf();
            da.specialn_sk_graphic_y_offset_air = r.rf();
            f.x34_scale.y = r.rf();
            f.facing_dir = if i & 2 != 0 { 1.0 } else { -1.0 };
            f.cur_pos.x = r.rf();
            f.cur_pos.y = r.rf();
            f.cur_pos.z = r.rf();
            f.ground_or_air = if r.rnd() & 1 != 0 { GA_Ground } else { GA_Air };
            f.u.kb.xB4 = (r.rnd() % 7) as i32;
            f.u.kb.xB8 = if r.rnd() & 3 != 0 { needle } else { core::ptr::null_mut() };
            f.mv.kb.specialhi.x4 = (r.rnd() % 4 != 0) as i32;
            OUT = format!("{i}");
        }
        if i & 1 != 0 {
            t::ftKb_SpecialNSk_8010603C(gp);
        } else {
            t::fn_80106DB0(gp);
        }
        let got = unsafe { format!("{} | {} {} {}", OUT, f.u.kb.xB4, !f.u.kb.xB8.is_null() as i32, f.mv.kb.specialhi.x4) };
        assert_eq!(got, line, "case {i}");
        n += 1;
    }
    assert_eq!(n, 3000);

    // ---- smoke: the rest through recording hooks ----
    unsafe {
    unsafe {
        LOG.clear();
        OUT.clear();
        st::FIGHTER_CHANGEMOTIONSTATE = Some(|_, ms, fl, _, _, _, _| log(format!("ms {ms} {fl:x}")));
        st::FTANIM_ISFRAMESREMAINING = Some(|_| 1);
        st::FTANIM_8006EBA4 = Some(|_| log("6EBA4".into()));
        st::FTCO_FALL_ENTER = Some(|_| log("Fall".into()));
        st::FT_8008A2BC = Some(|_| log("Wait".into()));
        st::FTCO_80096900 = Some(|_, _, _, _, m, lag| log(format!("fallspecial {m} {lag}")));
        st::FT_80082708 = Some(|_| 0);
        st::FT_80081D0C = Some(|_| 0);
        t::ext::IT_802B19AC = Some(|_, _, bone, kind, facing| {
            log(format!("held {bone} {kind} {facing}"));
            &mut **NEEDLE.as_mut().unwrap() as *mut HSD_GObj
        });
        t::ext::FTCO_800BFFD0 = Some(|_, a, b| {
            log(format!("BFFD0 {a} {b}"));
            true
        });
        t::ext::FTCO_LANDING_ENTER_BASIC = Some(|_| log("LandBasic".into()));
    }
    f.facing_dir = 1.0;
    // Enter: flag and vars reset, one charge stored, damage callbacks installed.
    f.u.kb.xB4 = 0;
    t::ftKb_SpecialNSk_8010612C(gp);
    assert_eq!(take(), format!("ms {} 0,6EBA4", ftKb_MS_SkSpecialNStart));
    assert_eq!(f.u.kb.xB4, 1);
    assert!(f.death2_cb.is_some() && f.take_dmg_cb.is_some());
    t::ftKb_SpecialNSk_801061E4(gp);
    assert_eq!(take(), format!("ms {} 0,6EBA4", ftKb_MS_SkSpecialAirNStart));

    // Start Anim: pick up the held needle, enter the loop.
    st_set_frames(0);
    t::ftKb_SkSpecialNStart_Anim(gp);
    assert_eq!(take(), format!("held 39 {} 1,ms {} 0", It_Kind_Kirby_SeakNeedleHeld, ftKb_MS_SkSpecialNLoop));
    assert!(!f.u.kb.xB8.is_null());
    t::ftKb_SkSpecialAirNStart_Anim(gp);
    assert_eq!(take(), format!("held 39 {} 1,ms {} 0", It_Kind_Kirby_SeakNeedleHeld, ftKb_MS_SkSpecialAirNLoop));
    st_set_frames(1);
    t::ftKb_SkSpecialNStart_Anim(gp);
    assert_eq!(take(), "");

    // Loop Anim: charge sfx on step 0; a stored needle per loop, capped at 6 (then the flash).
    f.mv.kb.specialhi.x8.i = 0;
    f.cur_anim_frame = 0.0;
    f.u.kb.xB4 = 5;
    unsafe { OUT.clear() };
    t::ftKb_SkSpecialNLoop_Anim(gp);
    assert_eq!(unsafe { OUT.clone() }, " sfx 41f36 127 64");
    assert_eq!((f.u.kb.xB4, unsafe { f.mv.kb.specialhi.x8.i }), (6, 0));
    f.u.kb.xB4 = 6;
    t::ftKb_SkSpecialAirNLoop_Anim(gp);
    assert_eq!(take(), "BFFD0 87 0");
    assert_eq!((f.u.kb.xB4, unsafe { f.mv.kb.specialhi.x8.i }), (6, 100));

    // End Anim: a needle is released on frames 2, 5, 8, 11, 14, 17 of the counter.
    for x0 in 0..19 {
        f.mv.kb.specialhi.x0 = x0;
        f.mv.kb.specialhi.x4 = 0;
        f.u.kb.xB8 = needle;
        t::ftKb_SkSpecialNEnd_Anim(gp);
        let released = matches!(x0, 2 | 5 | 8 | 11 | 14 | 17);
        assert_eq!(f.mv.kb.specialhi.x4, released as i32, "x0={x0}");
        assert_eq!(f.u.kb.xB8.is_null(), released, "x0={x0}");
        assert_eq!(f.mv.kb.specialhi.x0, x0 + 1);
    }
    take();
    // Air End / Cancel Anim: Fall without the freefall toggle, FallSpecial with it.
    st_set_frames(0);
    da.specialn_sk_freefall_toggle = 0.0;
    f.mv.kb.specialhi.x0 = 0;
    t::ftKb_SkSpecialAirNEnd_Anim(gp);
    t::ftKb_SkSpecialAirNCancel_Anim(gp);
    assert_eq!(take(), "Fall,Fall");
    da.specialn_sk_freefall_toggle = 0.75;
    t::ftKb_SkSpecialAirNEnd_Anim(gp);
    assert_eq!(take(), "fallspecial 1 0.75");
    st_set_frames(0);
    f.u.kb.xB8 = needle;
    t::ftKb_SkSpecialNCancel_Anim(gp);
    assert!(f.u.kb.xB8.is_null());
    assert_eq!(take(), "Wait");

    // Coll: landing out of Cancel / End goes through Fall or the basic landing.
    da.specialn_sk_freefall_toggle = 0.0;
    t::ftKb_SkSpecialNCancel_Coll(gp);
    assert_eq!(take(), "Fall");
    f.u.kb.xB4 = 4;
    f.mv.kb.specialhi.x4 = 1;
    t::ftKb_SkSpecialNEnd_Coll(gp);
    assert_eq!((f.u.kb.xB4, f.mv.kb.specialhi.x4), (0, 0));
    assert_eq!(take(), "Fall");
    t::ftKb_SkSpecialAirNCancel_Coll(gp);
    assert_eq!(take(), "");
    unsafe { st::FT_80081D0C = Some(|_| 1) };
    t::ftKb_SkSpecialAirNCancel_Coll(gp);
    f.u.kb.xB4 = 3;
    t::ftKb_SkSpecialAirNEnd_Coll(gp);
    assert_eq!(take(), "LandBasic,LandBasic");
    assert_eq!(f.u.kb.xB4, 0);

    // The empty reflect callback and the held-needle accessors.
    f.u.kb.xB8 = core::ptr::null_mut();
    assert!(t::ftKb_SpecialNSk_80105FF0(gp));
    f.u.kb.xB8 = needle;
    assert!(!t::ftKb_SpecialNSk_80105FF0(gp));
    assert!(t::ftKb_SpecialNSk_80105FF0(core::ptr::null_mut()));
    f.u.kb.xB4 = 4;
    assert_eq!(t::ftKb_SpecialNSk_80106020(gp), 4);
    assert_eq!(t::ftKb_SpecialNSk_80106020(core::ptr::null_mut()), 0);
    }
}

fn st_set_frames(v: i32) {
    unsafe {
        if v == 0 {
            st::FTANIM_ISFRAMESREMAINING = Some(|_| 0);
        } else {
            st::FTANIM_ISFRAMESREMAINING = Some(|_| 1);
        }
    }
}
