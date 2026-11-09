//! tu::ftclink vs the C (gen/turef/tu_ftclink_ref.c, shipping contraction; data in
//! tests/data/tu_ftclink.txt; regenerate with `python rust/melee_chara/gen/turef/tu_shard89.py
//! tu_ftclink`), plus a smoke test of the hooked callbacks. One #[test]: hooks are globals.
#![allow(static_mut_refs)]

use melee_chara::stubs as st;
use melee_chara::tu::ftclink as t;
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
    let want = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/tu_ftclink.txt")).unwrap();
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
        t::ext::FTCO_80092ED8 = Some(|a, b| b - a as f32);
        fh::PLAY_SFX = Some(|_, id, vol, pan| SFX = (id + vol + pan) as u32);
    }
    let cd = unsafe { &mut *(fh::P_FT_COMMON_DATA as *mut ftCommonData) };
    let ea = unsafe { &mut *(fd.ext_attr as *mut ftLk_DatAttrs) };
    let mut r = Rng(0xC11C11);
    let mut n = 0;
    for (i, line) in want.lines().enumerate() {
        f.x19A4 = (r.rnd() % 61) as i32 - 30;
        ea.xD8 = r.rf();
        cd.x294 = r.rf();
        f.specialn_facing_dir = r.rf();
        unsafe { SFX = 0 };
        t::ftCl_Init_80149114(gp);
        assert_eq!(format!("{} {:08x} {}", i, f.gr_vel.to_bits(), unsafe { SFX }), line, "case {i}");
        n += 1;
    }
    assert_eq!(n, 4000);

    // smoke
    f.dat_attrs = ea as *mut _ as *mut _;
    unsafe {
        st::ITISHEAVY = Some(|_| 1);
        st::FTPARTS_80074A4C = Some(|_, m, v| log(format!("parts {m} {v}")));
        st::FTANIM_80070CC4 = Some(|_, fl| log(format!("CC4 {fl}")));
        t::ext::FTCOLL_8007B1B8 = Some(|_, _, cb| log(format!("shield {}", cb.is_some())));
        t::ext::FTCO_800DEAE8 = Some(|_, a, b| log(format!("DEAE8 {a} {b}")));
        t::ext::IT_802C8C34 = Some(|_| log("drop milk".into()));
    }
    t::ftCl_Init_OnItemDropExt(gp, 1);
    assert_eq!(f.x8B0[1].x10, -1);
    t::ftCl_Init_8014919C(gp);
    assert_eq!((f.x221B.x221B_b2(), f.x221B.x221B_b3(), f.x221B.x221B_b4()), (1, 1, 1));
    assert!(t::ftCl_Init_8014920C(core::ptr::null_mut()));
    f.motion_id = 342;
    assert!(t::ftCl_Init_8014920C(gp));
    let mut milk: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    unsafe { f.u.lk.x18 = &mut *milk as *mut HSD_GObj as *mut Item_GObj };
    assert!(!t::ftCl_Init_8014920C(gp));
    t::ftCl_Init_80149268(gp);
    assert!(unsafe { f.u.lk.x18 }.is_null());
    f.cmd_vars[1] = 7;
    assert_eq!(t::ftCl_Init_801492F4(gp), 7);
    t::ftCl_Init_80149318(gp);
    assert_eq!(f.cmd_vars[1], 0);
    let got = unsafe { LOG.join(",") };
    assert_eq!(got, "parts 1 0,parts 2 0,CC4 1,shield true,drop milk,DEAE8 342 343");
}
