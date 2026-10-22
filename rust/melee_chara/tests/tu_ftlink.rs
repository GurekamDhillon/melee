//! tu::ftlink vs the C (gen/turef/tu_ftlink_ref.c, shipping contraction; data in
//! tests/data/tu_ftlink.txt; regenerate with `python rust/melee_chara/gen/turef/tu_shard337.py
//! tu_ftlink`), plus a smoke test of the init/damage callbacks. One #[test]: hooks are globals.
#![allow(static_mut_refs)]

use melee_chara::stubs as st;
use melee_chara::tu::ftlink as t;
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

static mut LOG: std::vec::Vec<String> = std::vec::Vec::new();
static mut SFX: u32 = 0;
fn log(s: String) {
    unsafe { LOG.push(s) }
}

#[test]
fn matches_c_and_smoke() {
    let want = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/tu_ftlink.txt")).unwrap();
    let mut f: Box<Fighter> = Box::new(Fighter::zeroed());
    let mut g: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    let fd: &'static mut ftData = Box::leak(Box::new(unsafe { core::mem::zeroed() }));
    let ea: &'static mut ftLk_DatAttrs = Box::leak(Box::new(ftLk_DatAttrs::zeroed()));
    let cd: &'static mut ftCommonData = Box::leak(Box::new(ftCommonData::zeroed()));
    fd.ext_attr = ea as *mut _ as *mut _;
    f.ft_data = fd;
    g.user_data = &mut *f as *mut Fighter as *mut _;
    let gp: *mut HSD_GObj = &mut *g;
    unsafe {
        fh::P_FT_COMMON_DATA = cd as *const _;
        fh::PLAY_SFX = Some(|_, id, vol, pan| SFX = (id + vol + pan) as u32);
    }
    let cd = unsafe { &mut *(fh::P_FT_COMMON_DATA as *mut ftCommonData) };
    let ea = unsafe { &mut *(fd.ext_attr as *mut ftLk_DatAttrs) };
    let mut r = Rng(0x1E4B0D);
    let mut n = 0;
    for (i, line) in want.lines().enumerate() {
        f.x19A4 = (r.rnd() % 61) as i32 - 30;
        ea.xD8 = r.rf();
        cd.x294 = r.rf();
        cd.x28C = r.rf();
        cd.x290 = r.rf();
        cd.x2E4 = r.rf();
        cd.x2E8 = r.rf();
        f.specialn_facing_dir = r.rf();
        unsafe { SFX = 0 };
        t::ftLk_800EB334(gp);
        assert_eq!(format!("{} {:08x} {}", i, f.gr_vel.to_bits(), unsafe { SFX }), line, "case {i}");
        n += 1;
    }
    assert_eq!(n, 4000);

    // smoke: boomerang exists, OnDeath-adjacent state, scale adjustment, item callbacks.
    f.dat_attrs = ea as *mut _ as *mut _;
    assert!(!t::ftLk_Init_BoomerangExists(gp));
    let mut boom: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    f.u.lk.boomerang_gobj = &mut *boom;
    assert!(t::ftLk_Init_BoomerangExists(gp));

    *ea = ftLk_DatAttrs::zeroed();
    ea.specialhi_pos_y_offset = 3.0;
    ea.x2C = 11;
    let backing: &'static mut ftLk_DatAttrs = Box::leak(Box::new(ftLk_DatAttrs::zeroed()));
    f.dat_attrs = backing as *mut _ as *mut _;
    f.x34_scale.y = 1.0;
    t::ftLk_Init_LoadSpecialAttrs(gp);
    assert_eq!(backing.specialhi_pos_y_offset, 3.0);
    assert_eq!(backing.x2C, 11);
    f.x34_scale.y = 2.0;
    t::ftLk_Init_LoadSpecialAttrs(gp);
    assert_eq!(backing.specialhi_pos_y_offset, 6.0);

    // PUSH_ATTRS: dat_attrs becomes the backup block holding a copy of ext_attr.
    let backup: &'static mut ftLk_DatAttrs = Box::leak(Box::new(ftLk_DatAttrs::zeroed()));
    f.dat_attrs_backup = backup as *mut _ as *mut _;
    t::ftLk_Init_OnLoadForCLink(&mut *f);
    assert_eq!(f.dat_attrs, f.dat_attrs_backup);
    assert_eq!(backup.x2C, 11);

    unsafe {
        st::ITISHEAVY = Some(|_| 1);
        st::FTPARTS_80074A4C = Some(|_, m, v| log(format!("parts {m} {v}")));
        st::FTANIM_80070CC4 = Some(|_, fl| log(format!("CC4 {fl}")));
    }
    t::ftLk_Init_OnItemDropExt(gp, 1);
    assert_eq!(f.x8B0[1].x10, -1);
    let got = unsafe { LOG.join(",") };
    assert_eq!(got, "parts 1 0,parts 2 0,CC4 1");
}
