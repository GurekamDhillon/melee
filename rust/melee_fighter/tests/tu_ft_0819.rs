//! src/tu/ft_0819.rs vs the C of src/melee/ft/ft_0819.c (gen/cref/tu_ft_0819_ref.c, expected
//! log in tests/data/tu_ft_0819.txt; regenerate with
//! `python rust/melee_fighter/gen/cref/tu_run_s57.py tu_ft_0819`). Fill order matches the C.
#![allow(static_mut_refs)]

use melee_fighter::hooks;
use melee_fighter::tu::ft_0819::{self as m, ext};
use melee_fighter::types::*;
use std::vec::Vec;

static mut LOG: String = String::new();
static mut FLAG: [bool; 4] = [false; 4];
static mut HIT: [bool; 4] = [false; 4];
static mut ITEMS: *mut Item = core::ptr::null_mut();
static mut HEAD: *mut HSD_GObj = core::ptr::null_mut();

fn log(s: String) {
    unsafe {
        LOG.push_str(&s);
        LOG.push('\n');
    }
}

struct Rng(u32);
impl Rng {
    fn rnd(&mut self) -> u32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 17;
        self.0 ^= self.0 << 5;
        self.0
    }
    fn rf(&mut self) -> f32 {
        let k = self.rnd() % 16;
        if k == 0 {
            return 0.0;
        }
        if k == 1 {
            return -0.0;
        }
        ((self.rnd() % 2001) as i32 - 1000) as f32 / 7.0
    }
}

fn fnv<T>(v: &T) -> u32 {
    let b = unsafe { core::slice::from_raw_parts(v as *const T as *const u8, core::mem::size_of::<T>()) };
    b.iter().fold(2166136261u32, |h, &x| (h ^ x as u32).wrapping_mul(16777619))
}

fn item_index(p: *const u8) -> usize {
    (p as usize - unsafe { ITEMS } as usize) / core::mem::size_of::<Item>()
}

#[test]
fn ft_0819_matches_c() {
    let mut items: Vec<Item> = (0..4).map(|_| Item::zeroed()).collect();
    let mut igo: Vec<HSD_GObj> = (0..4).map(|_| HSD_GObj::zeroed()).collect();
    unsafe {
        ITEMS = items.as_mut_ptr();
        hooks::PLINK_HEAD = Some(|l| if l == HSD_GOBJ_PLINK_ITEM { HEAD } else { core::ptr::null_mut() });
        ext::IT_8026BC68 = Some(|g| {
            let i = item_index((*g).user_data as *const u8);
            log(format!("BC68 {i}"));
            FLAG[i]
        });
        ext::LB_80014638 = Some(|a, b| {
            let i = item_index(b as *const _ as *const u8);
            log(format!(
                "L {} {:08X} {:08X} {:08X} {:08X} {:08X} {:08X}",
                i, a.x0.x.to_bits(), a.x0.y.to_bits(), a.x0.z.to_bits(), a.xC.x.to_bits(), a.xC.y.to_bits(), a.xC.z.to_bits()
            ));
            HIT[i]
        });
        ext::FTCO_800C703C = Some(|_| log("C703C".into()));
    }
    for op in 0..3u32 {
        for k in 0..2000u32 {
            let mut fp: Box<Fighter> = Box::new(Fighter::zeroed());
            let mut g: Box<HSD_GObj> = Box::new(HSD_GObj::zeroed());
            g.user_data = &mut *fp as *mut Fighter as *mut core::ffi::c_void;
            let gp: *mut HSD_GObj = &mut *g;
            for i in 0..4 {
                items[i] = Item::zeroed();
                igo[i] = HSD_GObj::zeroed();
            }
            let mut r = Rng(0x81938u32.wrapping_add(op.wrapping_mul(7919)).wrapping_add(k.wrapping_mul(104729)));
            fp.coll_data.cur_pos.x = r.rf();
            fp.coll_data.cur_pos.y = r.rf();
            fp.coll_data.ecb.left.x = r.rf();
            fp.coll_data.ecb.right.x = r.rf();
            fp.cur_pos.x = r.rf();
            fp.cur_pos.y = r.rf();
            fp.co_attrs.x17C = r.rf();
            fp.dmg.x1930.x0.x = r.rf();
            fp.dmg.x1930.xC.z = r.rf();
            fp.set_x2219_b1((r.rnd() % 4 == 0) as u8);
            fp.set_x2223_b4((r.rnd() % 4 == 0) as u8);
            fp.player_id = (r.rnd() % 6) as u8;
            fp.dmg.x1950 = (r.rnd() % 8 == 0) as i32;
            let n = (r.rnd() % 5) as usize;
            let igp = igo.as_mut_ptr();
            for i in 0..4 {
                igo[i].user_data = &mut items[i] as *mut Item as *mut core::ffi::c_void;
                igo[i].next = if i + 1 < n { unsafe { igp.add(i + 1) } } else { core::ptr::null_mut() };
                items[i].pos.x = r.rf();
                items[i].xCD0 = r.rf();
                items[i].xCB0_source_ply = (r.rnd() % 9) as i32;
                unsafe {
                    FLAG[i] = r.rnd() & 1 != 0;
                    HIT[i] = r.rnd() & 1 != 0;
                }
            }
            unsafe { HEAD = if n != 0 { igp } else { core::ptr::null_mut() } };
            log(format!("OP {op} {k}"));
            let mut ret = 0;
            match op {
                0 => m::ft_80081938(gp),
                1 => m::ft_800819a8(gp),
                _ => ret = m::ft_80081a00(gp) as i32,
            }
            let (b1, b4, b5) = (fp.x2219_b1(), fp.x2223_b4(), fp.x2223_b5());
            fp.set_x2219_b1(0);
            fp.set_x2223_b4(0);
            fp.set_x2223_b5(0);
            log(format!("R {} {} {} {} {:08X}", ret, b1, b4, b5, fnv(&*fp)));
            for i in 0..4 {
                let own = (items[i].xCFC == gp) as i32;
                items[i].xCFC = core::ptr::null_mut();
                log(format!("I {} {} {:08X}", i, own, fnv(&items[i])));
            }
        }
    }
    let got = unsafe { LOG.clone() };
    let mut el = include_str!("data/tu_ft_0819.txt").lines();
    let mut n = 0;
    for gl in got.lines() {
        assert_eq!(gl, el.next().expect("C log shorter"), "line {n}");
        n += 1;
    }
    assert!(el.next().is_none(), "C log longer");
}
