//! tu::ftco_0d8e vs the C (gen/cref/tu_ftco_0d8e_ref.c, shipping contraction; data in
//! tests/data/tu_ftco_0d8e.txt; regenerate with `python rust/melee_fighter/gen/cref/tu_shard97.py`).
//! TR folds every stubbed call and its arguments in the C's order (incl. the fused probe end).
#![allow(static_mut_refs)]

use melee_fighter::hooks as h;
use melee_fighter::tu::ftco_0d8e::{self as t, ext};
use melee_fighter::types::*;

struct W {
    f: Fighter,
    go: HSD_GObj,
    ig: [HSD_GObj; 2],
    it: [Item; 2],
    art: Article,
    sa: [f32; 64],
    da: [i32; 64],
    j: HSD_JObj,
    pb: [FighterBone; 8],
}
static mut WP: *mut W = core::ptr::null_mut();
static mut TR: u32 = 0;
static mut BONE: i32 = 0;
static mut SPAWN: i32 = 0;
static mut REMAP: bool = false;
static mut LBV: Vec3 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };

fn w() -> &'static mut W {
    unsafe { &mut *WP }
}
fn tt(v: u32) {
    unsafe { TR = TR.wrapping_mul(31).wrapping_add(v) }
}
fn e<T>(p: *const T) -> u32 {
    let p = p as *const u8;
    let w = w();
    if p.is_null() {
        0
    } else if p == &w.ig[0] as *const _ as *const u8 {
        1
    } else if p == &w.ig[1] as *const _ as *const u8 {
        2
    } else if p == &w.go as *const _ as *const u8 {
        3
    } else if p == &w.j as *const _ as *const u8 {
        4
    } else {
        99
    }
}
fn isf(fp: &Fighter) -> u32 {
    (fp as *const Fighter == &w().f as *const Fighter) as u32
}
fn tv(v: &Vec3) {
    tt(v.x.to_bits());
    tt(v.y.to_bits());
    tt(v.z.to_bits());
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
        match self.rnd() % 10 {
            0 => 0.0,
            1 => -0.0,
            2 => 1.0,
            3 => -1.0,
            _ => ((self.rnd() % 20001) as i32 - 10000) as f32 / 1013.0,
        }
    }
}

fn same(a: HSD_GObjEvent, b: unsafe extern "C" fn(*mut HSD_GObj)) -> u32 {
    matches!(a, Some(f) if f as usize == b as usize) as u32
}

#[test]
fn matches_c() {
    let want = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/tu_ftco_0d8e.txt")).unwrap();
    unsafe {
        WP = Box::leak(Box::new(core::mem::zeroed::<W>()));
        h::FTPARTS_GET_BONE_INDEX = Some(|fp, p| { tt(20); tt(isf(fp)); tt(p as u32); BONE });
        ext::LB_8000B1CC = Some(|j, off, out| { tt(1); tt(e(j)); tt(off.is_none() as u32); *out = LBV });
        ext::IT_802A2BA4 = Some(|g, pos, f, x| {
            tt(2); tt(e(g)); tv(pos); tt(f.to_bits()); tt(x as u32);
            if SPAWN < 2 { &mut w().ig[SPAWN as usize] } else { core::ptr::null_mut() }
        });
        h::FT_8008A2BC = Some(|g| { tt(3); tt(e(g)) });
        ext::HSD_JOBJ_SETUP_MATRIX_SUB = Some(|j| { tt(4); tt(e(j)) });
        ext::MP_CHECK_ALL_REMAP = Some(|skip, only, x0, y0, x1, y1| {
            tt(5); tt(1); tt(skip as u32); tt(only as u32);
            tt(x0.to_bits()); tt(y0.to_bits()); tt(x1.to_bits()); tt(y1.to_bits());
            REMAP
        });
        ext::IT_802A2B10 = Some(|g| { tt(6); tt(e(g)) });
        ext::IT_802A78B8 = Some(|g, v| { tt(7); tt(e(g)); tv(v) });
        h::PLAY_SFX = Some(|fp, id, vol, pan| { tt(8); tt(isf(fp)); tt(id as u32); tt(vol as u8 as u32); tt(pan as u8 as u32) });
        ext::IT_802A77DC = Some(|g| { tt(9); tt(e(g)) });
        ext::FTLK_INIT_BOOMERANG_EXISTS = Some(|g| { tt(10); tt(e(g)); false });
    }
    let w = w();
    w.go.user_data = (&mut w.f as *mut Fighter).cast();
    w.f.parts = w.pb.as_mut_ptr();
    w.f.dat_attrs = w.da.as_mut_ptr().cast();
    w.art.x4_specialAttributes = w.sa.as_mut_ptr().cast();
    for k in 0..2 {
        w.ig[k].user_data = (&mut w.it[k] as *mut Item).cast();
        w.it[k].xC4_article_data = &mut w.art;
    }
    let kinds = [Ft_Kind_Link, Ft_Kind_CLink, Ft_Kind_Samus, Ft_Kind_Mario];
    let mut r = Rng(0x0D8E0D8E);
    let mut n = 0;
    for (i, line) in want.lines().enumerate() {
        let op = r.rnd() % 5;
        let mut ret = 0u32;
        unsafe { TR = 0 };
        for k in 0x84 / 4..=0xBC / 4 {
            w.da[k] = (r.rnd() % 8) as i32;
        }
        for k in 0..64 {
            w.sa[k] = r.rf();
        }
        let bone = (r.rnd() % 8) as i32;
        unsafe {
            BONE = bone;
            SPAWN = (r.rnd() % 3) as i32;
            REMAP = r.rnd() % 2 != 0;
            LBV.x = r.rf();
            LBV.y = r.rf();
            LBV.z = r.rf();
        }
        w.f.kind = kinds[(r.rnd() % 4) as usize];
        w.f.mv.co.catch_.x0 = (r.rnd() % 9) as i32 as f32 - 1.0;
        w.f.facing_dir = r.rf();
        w.f.x34_scale.y = r.rf();
        w.f.coll_data.cur_pos.x = r.rf();
        let xc: *mut HSD_GObj = if r.rnd() % 3 == 0 { core::ptr::null_mut() } else { &mut w.ig[(r.rnd() % 2) as usize] };
        w.f.u.lk.xC = xc;
        w.f.accessory2_cb = None;
        w.f.death1_cb = None;
        w.f.accessory3_cb = None;
        w.j.flags = r.rnd() & (JOBJ_MTX_DIRTY | JOBJ_USER_DEF_MTX) as u32;
        for k in 0..4 {
            w.j.mtx[0][k] = r.rf();
            w.j.mtx[1][k] = r.rf();
            w.j.mtx[2][k] = r.rf();
        }
        w.pb[bone as usize].joint = &mut w.j;
        if op <= 1 && xc.is_null() {
            w.f.u.lk.xC = &mut w.ig[0];
        }
        let g: *mut HSD_GObj = &mut w.go;
        match op {
            0 => ret = t::fn_800d8ec8(g) as u32,
            1 => ret = t::fn_800d9228(g) as u32,
            2 => t::fn_800d949c(g),
            3 => t::ftco_800d94d8(g),
            _ => ret = t::fn_800d952c(g) as u32,
        }
        let got = format!(
            "{} {} {} {:08X} {:08X} {} {}{}{}",
            i, op, ret, unsafe { TR }, unsafe { w.f.mv.co.catch_.x0 }.to_bits(), e(unsafe { w.f.u.lk.xC }),
            same(w.f.accessory2_cb, ext::it_802A7AF0), same(w.f.death1_cb, ext::it_802A7AAC), same(w.f.accessory3_cb, ext::it_802A7B34)
        );
        assert_eq!(got, line, "case {i}");
        n += 1;
    }
    assert_eq!(n, 8000);
}
