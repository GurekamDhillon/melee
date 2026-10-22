//! tu::ftlinkspecials vs the C (gen/turef/tu_ftlinkspecials_ref.c, shipping contraction; data in
//! tests/data/tu_ftlinkspecials.txt; regenerate with `python rust/melee_chara/gen/turef/tu_shard337.py
//! tu_ftlinkspecials`): `calcAnglePos` bit-exact, plus a smoke test of the Boomerang state
//! machine through recording hooks. One #[test]: hooks are globals.
#![allow(static_mut_refs, unused_unsafe)]

use melee_chara::stubs as st;
use melee_chara::tu::ftlinkspecials as t;
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
fn take_log() -> String {
    unsafe { std::mem::take(&mut LOG).join(",") }
}

#[test]
fn matches_c_and_smoke() {
    unsafe { body() }
}

unsafe fn body() {
    let want = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/tu_ftlinkspecials.txt")).unwrap();
    let mut f: Box<Fighter> = Box::new(Fighter::zeroed());
    let mut g: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    let da: &'static mut ftLk_DatAttrs = Box::leak(Box::new(ftLk_DatAttrs::zeroed()));
    f.dat_attrs = da as *mut _ as *mut _;
    g.user_data = &mut *f as *mut Fighter as *mut _;
    let gp: *mut HSD_GObj = &mut *g;

    let mut r = Rng(0x7A11E5);
    let mut n = 0;
    for (i, line) in want.lines().enumerate() {
        f.input.lstick[0].x = r.rf();
        f.input.lstick[0].y = r.rf();
        f.facing_dir = if r.rnd() & 1 != 0 { 1.0 } else { -1.0 };
        da.x14 = if r.rnd() % 4 != 0 { (r.rnd() % 100) as f32 / 150.0 } else { r.rf() };
        da.x18 = r.rf();
        da.x1C = if r.rnd() % 3 != 0 { 1.0 } else { r.rf() };
        let dist = r.rf() * 10.0;
        let mut pos = Vec3 { x: 9.0, y: 9.0, z: 9.0 };
        let a = t::calcAnglePos(&f, da, &mut pos, dist);
        let got = format!("{} {:08x} {:08x} {:08x} {:08x}", i, a.to_bits(), pos.x.to_bits(), pos.y.to_bits(), pos.z.to_bits());
        assert_eq!(got, line, "case {i}");
        n += 1;
    }
    assert_eq!(n, 6000);

    // smoke: Enter / Anim / IASA / Coll / onAccessory4 through recording hooks.
    let mut parts: std::vec::Vec<FighterBone> = (0..80).map(|_| unsafe { core::mem::zeroed() }).collect();
    f.parts = parts.as_mut_ptr();
    let mut boom: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    let bp: *mut HSD_GObj = &mut *boom;
    unsafe {
        st::FIGHTER_CHANGEMOTIONSTATE = Some(|_, ms, fl, _, _, _, _| log(format!("ms {ms} {fl:x}")));
        st::FTANIM_8006EBA4 = Some(|_| {});
        st::FTANIM_ISFRAMESREMAINING = Some(|_| 0);
        st::FT_8008A2BC = Some(|_| log("Wait".into()));
        st::FTCO_FALL_ENTER = Some(|_| log("Fall".into()));
        st::FT_80082708 = Some(|_| 0);
        st::FT_80081D0C = Some(|_| 1);
        st::FTCOMMON_8007D7FC = Some(|_| log("7D7FC".into()));
        t::ext::IT_802A07B4 = Some(|_| log("remove".into()));
        t::ext::IT_802A013C = Some(|_, _, _, part, x2c| {
            log(format!("spawn {part} {x2c}"));
            core::ptr::null_mut()
        });
    }
    // Enter: boomerang available / used.
    f.u.lk.used_boomerang = 0;
    t::ftLk_SpecialS_Enter(gp);
    f.u.lk.used_boomerang = 1;
    t::ftLk_SpecialAirS_Enter(gp);
    assert_eq!(take_log(), "ms 350 0,ms 355 0");
    assert!(f.accessory4_cb.is_some() && f.x21EC.is_some());
    // S2 anim: the cmd_vars[1] pulse removes the boomerang, the end of the animation waits.
    f.u.lk.boomerang_gobj = bp;
    f.cmd_vars[1] = 1;
    t::ftLk_SpecialS2_Anim(gp);
    assert_eq!(take_log(), "remove,Wait");
    assert_eq!(f.cmd_vars[1], 0);
    // 2071b0 ranges.
    for v in 0..16u8 {
        unsafe { f.x2070.anon_0.set_x2071_b0_3(v) };
        assert_eq!(t::ftLk_SpecialS_Is2071b0_5to13(gp), (5..=13).contains(&v), "{v}");
        assert_eq!(t::ftLk_SpecialS_Is2071b0_1to13(gp), (1..=13).contains(&v), "{v}");
    }
    // Coll: ground S2 leaves the boomerang and falls; air S1 is the shared state change.
    t::ftLk_SpecialS2_Coll(gp);
    assert_eq!(take_log(), "remove,Fall");
    // RemoveBoomerang0 / 1.
    f.u.lk.boomerang_gobj = bp;
    f.u.lk.used_boomerang = 1;
    assert!(!t::ftLk_SpecialS_RemoveBoomerang0(gp));
    assert_eq!((f.u.lk.used_boomerang, f.u.lk.boomerang_gobj.is_null()), (0, true));
    f.u.lk.boomerang_gobj = bp;
    f.u.lk.used_boomerang = 1;
    t::ftLk_SpecialS_RemoveBoomerang1(gp);
    assert_eq!((f.u.lk.used_boomerang, f.u.lk.boomerang_gobj.is_null()), (0, true));
    assert_eq!(take_log(), "remove");
    // onAccessory4: the throw pulse spawns a boomerang (null here: nothing recorded after).
    unsafe { melee_fighter::hooks::FTPARTS_GET_BONE_INDEX = Some(|_, i| i) };
    t::ext::LB_8000B1CC = Some(|_, off, out| {
        assert!(off.is_none());
        *out = Vec3 { x: 1.0, y: 2.0, z: 3.0 };
    });
    unsafe { f.anon_2210.anon_0.set_throw_flags_b0(1) };
    da.x2C = 77;
    t::onAccessory4(gp);
    assert_eq!(take_log(), "spawn 31 77");
    assert_eq!(unsafe { f.anon_2210.anon_0.throw_flags_b0() }, 0);
    assert!(f.u.lk.boomerang_gobj.is_null());
    // on21EC: the dash-smash window decides x4.
    unsafe { melee_fighter::hooks::P_FT_COMMON_DATA = Box::leak(Box::new(ftCommonData::zeroed())) };
    let cd = unsafe { &mut *(melee_fighter::hooks::P_FT_COMMON_DATA as *mut ftCommonData) };
    cd.dash_smash_stick_threshold = 0.5;
    cd.dash_smash_window = 3;
    cd.x44 = 1.5;
    f.input.lstick[0].x = -0.75;
    f.active_sticky.lstick.x = 4;
    t::on21EC(gp);
    assert_eq!(f.u.lk.x4, 1);
    assert_eq!(unsafe { f.x2070.anon_0.count_thrown_items() }, 1);
    f.active_sticky.lstick.x = 5;
    t::on21EC(gp);
    assert_eq!(f.u.lk.x4, 0);
}
