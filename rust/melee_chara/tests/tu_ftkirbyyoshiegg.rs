//! tu::ftkirbyyoshiegg vs the C (gen/turef/tu_ftkirbyyoshiegg_ref.c: fn_8010B16C cut verbatim,
//! shipping contraction, 1 fused site; data in tests/data/tu_ftkirbyyoshiegg.txt; regenerate with
//! `python rust/melee_chara/gen/turef/tu_shard329.py tu_ftkirbyyoshiegg`), plus a smoke test of
//! the rest through recording hooks. One #[test]: hooks are process globals.
#![allow(static_mut_refs, unused_unsafe)]

use melee_chara::stubs as st;
use melee_chara::tu::ftkirbyspecialyoshi as ys;
use melee_chara::tu::ftkirbyyoshiegg as t;
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
fn log(s: String) {
    unsafe { LOG.push(s) }
}
fn take() -> String {
    unsafe { core::mem::take(&mut LOG).join(",") }
}

#[test]
fn matches_c_and_smoke() {
    let want = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/tu_ftkirbyyoshiegg.txt")).unwrap();
    let mut f: Box<Fighter> = Box::new(Fighter::zeroed());
    let mut g: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    let mut jobj: Box<HSD_JObj> = Box::new(unsafe { core::mem::zeroed() });
    let mut acc: Box<HSD_JObj> = Box::new(unsafe { core::mem::zeroed() });
    let fd: &'static mut ftData = Box::leak(Box::new(unsafe { core::mem::zeroed() }));
    let ea: &'static mut ftKb_DatAttrs = Box::leak(Box::new(ftKb_DatAttrs::zeroed()));
    fd.ext_attr = ea as *mut _ as *mut _;
    g.user_data = &mut *f as *mut Fighter as *mut _;
    g.hsd_obj = &mut *jobj as *mut HSD_JObj as *mut _;
    f.x20A0_accessory = &mut *acc;
    let gp: *mut HSD_GObj = &mut *g;
    static mut FD: *mut ftData = core::ptr::null_mut();
    unsafe {
        FD = fd;
        ys::ext::FTKB_KIRBYFTDATA = Some(|| FD);
        t::ext::FTCO_800C0C88 = Some(|v| v & 1 != 0);
    }
    let mut r = Rng(0xE66);
    let mut n = 0;
    for (i, line) in want.lines().enumerate() {
        f.grab_timer = r.rf() * 60.0;
        f.dmg.x1838_percentTemp = r.rf() * 30.0;
        f.dmg.x18CC = if r.rnd() & 1 != 0 { 3 } else { (r.rnd() % 5) as i32 };
        f.dmg.x18D0 = (r.rnd() % 8) as i32;
        ea.specialn_ys_unk3 = r.rf();
        ea.specialn_ys_damage_multiplier = r.rf() + 1.5;
        f.x1828 = 0;
        t::fn_8010B16C(gp);
        assert_eq!(format!("{} {:08x} {}", i, f.grab_timer.to_bits(), f.x1828), line, "case {i}");
        n += 1;
    }
    assert_eq!(n, 3000);

    // ---- smoke: the rest through recording hooks ----
    unsafe {
        LOG.clear();
        st::FIGHTER_CHANGEMOTIONSTATE = Some(|_, ms, fl, _, _, _, a3| log(format!("ms {ms} {fl:x} {}", !a3.is_null())));
        st::FT_80084F3C = Some(|_| log("ground phys".into()));
        st::FT_80084DB0 = Some(|_| log("air phys".into()));
        st::FT_8008403C = Some(|_, cb| log(format!("coll ground cb={}", cb.is_some())));
        st::FT_80082C74 = Some(|_, cb| log(format!("coll air cb={}", cb.is_some())));
        st::FTANIM_SETANIMRATE = Some(|_, rate| log(format!("rate {rate}")));
        t::ext::FIGHTER_UPDATEMODELSCALE = Some(|_| log("scale".into()));
        t::ext::HSD_JOBJSETSCALE = Some(|_, s| log(format!("setscale {} {} {}", (*s).x, (*s).y, (*s).z)));
    }
    // Egg shrink: x14 counts down; scale = ((1 - xC) + xC * (x10 - x14) / x10) * saved scale.
    f.mv.co.yoshiegg.x10 = 4.0;
    f.mv.co.yoshiegg.x14 = 2.0;
    f.mv.co.yoshiegg.xC = 0.5;
    f.mv.co.yoshiegg.x18 = Vec3 { x: 2.0, y: 4.0, z: 8.0 };
    f.mv.co.yoshiegg.scale = Vec3 { x: 1.0, y: 3.0, z: 5.0 };
    f.accessory4_cb = Some(t::cb_fn_8010AA64);
    t::fn_8010AA64(gp);
    // x14 -> 1: t = (4 - 1) / 4 * 0.5 = 0.375; s = 0.5 + 0.375 = 0.875
    assert_eq!(take(), "setscale 1.75 3.5 7,setscale 0.875 2.625 4.375");
    assert_eq!(unsafe { f.mv.co.yoshiegg.x14 }, 1.0);
    assert!(f.accessory4_cb.is_some());
    f.mv.co.yoshiegg.x14 = 0.0;
    t::fn_8010AA64(gp);
    assert_eq!(take(), "scale");
    assert!(f.accessory4_cb.is_none());

    // Phys / Coll dispatch on the ground-or-air state.
    f.ground_or_air = GA_Ground;
    t::ftCo_KirbyYoshiEgg_Phys(gp);
    t::ftCo_KirbyYoshiEgg_Coll(gp);
    f.ground_or_air = GA_Air;
    t::ftCo_KirbyYoshiEgg_Phys(gp);
    t::ftCo_KirbyYoshiEgg_Coll(gp);
    assert_eq!(take(), "ground phys,coll ground cb=true,air phys,coll air cb=true");
    t::fn_8010B1D4(gp);
    assert_eq!(take(), "scale");

    // Anim: the escape timer ticks down by the Kirby ftData value; mashing reduces it further.
    ea.specialn_ys_egg_breakout_resistance = 1.0;
    ea.specialn_ys_frames_reduced_per_input = 5.0;
    ea.specialn_ys_unk1 = 8.0;
    ea.specialn_ys_unk2 = 2.0;
    f.grab_timer = 10.0;
    f.mv.kb.specialhi.x8.f = 0.0;
    unsafe {
        t::ext::FTCOMMON_GRABMASH = Some(|_, dec| {
            log(format!("mash {dec}"));
            1
        });
    }
    t::ftCo_KirbyYoshiEgg_Anim(gp);
    assert_eq!(f.grab_timer, 9.0);
    assert_eq!(take(), "mash 5,rate 2");
    assert_eq!(unsafe { f.mv.kb.specialhi.x8.f }, 8.0);
    assert_eq!(unsafe { f.mv.kb.specialhi.x4 }, 1);
}
