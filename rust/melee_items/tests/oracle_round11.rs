//! Round 11 (night 3, round 3): src/itkinds11.rs (6 hand ports: jobj.h getter, sqrtf_accurate and
//! HSD_Randi callers) against tests/c_oracle/round11.c (contraction on + gwtool lowering, as the
//! shipping build). Same harness as round 10 (the infrastructure below is a copy of
//! oracle_round10.rs's); each case builds a GObj whose user data is the case Item and whose
//! hsd_obj is a chain of 4 JObjs, as round11.c's `r11_jobjs`.
#![allow(non_upper_case_globals, dead_code)]

#[path = "c_oracle/expected.rs"]
#[allow(dead_code)]
mod expected;
#[path = "round2/fields.rs"]
mod fields;

use expected as e;
use melee_items::itkinds11::*;
use melee_runtime::random::HsdRand;
use melee_items::layout::*;
use std::ptr::null_mut;

pub const KINDS: [i32; 29] = [It_Kind_Heart, It_Kind_Tomato, It_Kind_Foods, It_Kind_Coin,
    It_Kind_Lucky_Egg, It_Kind_WhispyHealApple, It_Kind_L_Gun, It_Kind_S_Scope,
    It_Kind_StarRod, It_Kind_LipStick, It_Kind_F_Flower, It_Kind_Harisen, It_Kind_Freeze,
    It_Kind_MSBomb, It_Kind_Flipper, It_Kind_BombHei, It_Kind_Link_Bomb, It_Kind_Kuriboh,
    It_Kind_Octarock_Stone - 1, It_Kind_Octarock_Stone, It_Kind_Old_Kuri - 1,
    It_Kind_Old_Kuri, It_Kind_Arwing_Laser - 1, It_Kind_Arwing_Laser, It_Kind_Kyasarin_Egg,
    It_Kind_WhispyApple, 0, 100, 230];
const POOL: [u32; 10] = [0x00000000, 0x80000000, 0x3F800000, 0xBF800000, 0x3F000000,
    0xBF000000, 0x40000000, 0x00000001, 0x7F7FFFFF, 0x42C80000];

const DMG: [u32; 4] = [0x00000000, 0x80000000, 0x3F800000, 0x3F000000];

pub fn hmix(h: u32, v: u32) -> u32 {
    (h ^ v).wrapping_mul(0x01000193)
}

/// round2.c's LCG, `RF()` and `PTR()`/`PIDX()`.
pub struct Gen {
    s: u32,
    ga: *mut HSD_GObj,
    gb: *mut HSD_GObj,
}
impl Gen {
    pub fn r(&mut self) -> u32 {
        self.s = self.s.wrapping_mul(1103515245).wrapping_add(12345);
        self.s
    }
    pub fn rf(&mut self) -> f32 {
        let mut b = self.r();
        let sel = self.r() % 8;
        if sel == 0 {
            return f32::from_bits(POOL[(b % 10) as usize]);
        }
        if sel < 4 {
            return ((b >> 8) as i32 - 0x7FFFFF) as f32 / 65536.0;
        }
        if (b >> 23) & 0xFF == 0xFF {
            b &= !0x40000000;
        }
        f32::from_bits(b)
    }
    /// round2.c `RM()`: |x| < 128 with 16 fraction bits.
    pub fn rm(&mut self) -> f32 {
        ((self.r() >> 8) as i32 - 0x7FFFFF) as f32 / 65536.0
    }
    pub fn ptr(&mut self) -> *mut HSD_GObj {
        let r = self.r();
        match r % 3 {
            0 => null_mut(),
            1 => self.ga,
            _ => self.gb,
        }
    }
    pub fn pidx<T>(&self, p: *mut T) -> u32 {
        let p = p as *mut HSD_GObj;
        if p.is_null() {
            0
        } else if p == self.ga {
            1
        } else if p == self.gb {
            2
        } else {
            3
        }
    }
}

pub struct Ext {
    sa: [u32; 256],
    attr: ItemAttr,
    art: Article,
    unk: itUnkAttributes_legacy,
    common: ItemCommonData,
}

type Case = fn(&mut Item, &mut Gen, &mut Vec, &mut Ext) -> u32;

fn run(id: u32, name: &str, want: &[u32; 48], case: Case) {
    let mut ga = Box::new(HSD_GObj::zeroed());
    let mut gb = Box::new(HSD_GObj::zeroed());
    let mut ip = Box::new(Item::zeroed());
    let mut ext = Box::new(Ext {
        attr: ItemAttr::zeroed(),
        art: Article::zeroed(),
        unk: itUnkAttributes_legacy::zeroed(),
        common: ItemCommonData::zeroed(),
        sa: [0; 256],
    });
    for c in 0..48u32 {
        let mut g = Gen {
            s: 0x2545F491 ^ id.wrapping_mul(0x9E3779B9) ^ c.wrapping_mul(0x85EBCA6B),
            ga: &mut *ga,
            gb: &mut *gb,
        };
        fields::fill(&mut ip, &mut g);
        // round2.c fill_ext()
        let x = &mut *ext;
        x.attr = ItemAttr::zeroed();
        x.art = Article::zeroed();
        x.unk = itUnkAttributes_legacy::zeroed();
        x.common = ItemCommonData::zeroed();
        x.attr.set_x0_is_heavy(g.r() as u8);
        x.attr.set_x0_78(g.r() as u8);
        x.attr.set_x0_hold_kind(g.r() as u8);
        x.attr.x1C_damage_mul = g.rf();
        x.unk.x0_float = g.rf();
        x.unk.x4_float = g.rf();
        x.art.x4_specialAttributes = &mut x.unk as *mut itUnkAttributes_legacy as *mut _;
        x.art.x8_hurtbones = if g.r() & 1 != 0 { g.ga as *mut _ } else { null_mut() };
        x.common.xB8 = g.rf();
        x.common.xBC = g.rf();
        x.common.xDC = g.r() as i32;
        ip.xCC_item_attr = &mut x.attr;
        ip.xC4_article_data = &mut x.art;
        let mut ov = Vec { x: g.rf(), y: g.rf(), z: g.rf() };
        let ret = case(&mut ip, &mut g, &mut ov, x);
        let h = hmix(
            hmix(hmix(hmix(fields::snap(&ip, &g), ret), ov.x.to_bits()), ov.y.to_bits()),
            ov.z.to_bits(),
        );
        assert_eq!(h, want[c as usize], "{name} case {c}");
    }
}

fn var_words(ip: &mut Item) -> &mut [u32] {
    let n = std::mem::size_of::<Item_ItemVars>() / 4;
    // SAFETY: the item-var union is plain data (floats, ints, pointers, bytes), 4-aligned.
    unsafe { std::slice::from_raw_parts_mut(&mut ip.xDD4_itemVar as *mut Item_ItemVars as *mut u32, n) }
}

/// round4.c `r4_pre`.
fn r4_pre(ip: &mut Item, g: &mut Gen, x: &mut Ext) {
    ip.ground_or_air = g.r() % 3;
    ip.xDAC_itcmd_var0 = if g.r() & 1 != 0 { 0 } else { g.r() };
    ip.xDB0_itcmd_var1 = if g.r() & 1 != 0 { 0 } else { g.r() };
    ip.xDB4_itcmd_var2 = if g.r() & 1 != 0 { 0 } else { g.r() };
    ip.xDB8_itcmd_var3 = if g.r() & 1 != 0 { 0 } else { g.r() };
    ip.x5CC_currentAnimFrame = if g.r() & 3 != 0 { g.rf() } else { 16.0 };
    ip.x5D0_animFrameSpeed = g.rf();
    ip.xCCC_incDamageDirection = g.rf();
    ip.xCA8 = g.r() as i32;
    x.attr.x10_fall_speed = if g.r() & 3 != 0 { g.rm() } else { g.rf() };
    x.attr.x14_fall_speed_max = if g.r() & 3 != 0 { g.rm() } else { g.rf() };
    x.common.x4C_float = g.rf();
    x.common.x58_float = g.rm();
    x.common.x5C_float = g.rm();
    x.common.x60_float = g.rm();
    for k in 0..256 {
        x.sa[k] = if g.r() & 3 != 0 { g.rm().to_bits() } else { g.rf().to_bits() };
    }
    x.art.x4_specialAttributes = x.sa.as_mut_ptr() as *mut _;
    for w in var_words(ip).iter_mut() {
        let s = g.r() % 4;
        *w = match s {
            0 => 0,
            1 => g.r() % 4,
            2 => g.rf().to_bits(),
            _ => g.rm().to_bits(),
        };
    }
}

/// round4.c `r4_h`.
fn r4_h(ip: &mut Item) -> u32 {
    let mut h = hmix(hmix(hmix(hmix(hmix(0, ip.ground_or_air), ip.xDAC_itcmd_var0), ip.xDB0_itcmd_var1),
        ip.xDB4_itcmd_var2), ip.xDB8_itcmd_var3);
    h = hmix(hmix(hmix(hmix(h, ip.x5CC_currentAnimFrame.to_bits()), ip.x5D0_animFrameSpeed.to_bits()),
        ip.xCCC_incDamageDirection.to_bits()), ip.xCA8 as u32);
    for w in var_words(ip).iter() {
        h = hmix(h, *w);
    }
    h
}

/// Bit-reverse the item-var byte at `off` (x86 C bitfield order <-> GC order).
fn r4_rev(ip: &mut Item, off: usize) {
    // SAFETY: `off` is inside the 504-byte union.
    unsafe {
        let p = (&mut ip.xDD4_itemVar as *mut Item_ItemVars as *mut u8).add(off);
        *p = (*p).reverse_bits();
    }
}


macro_rules! cases {
    ($fname:ident, $count:literal; $($id:literal $want:ident |$ip:ident, $g:ident, $ov:ident, $x:ident| $body:expr;)*) => {
        #[test]
        fn $fname() {
            let mut n = 0;
            $(
                {
                    #[allow(unused_variables, unused_unsafe, unused_mut)]
                    fn case($ip: &mut Item, $g: &mut Gen, $ov: &mut Vec, $x: &mut Ext) -> u32 {
                        unsafe { $body }
                    }
                    run($id, stringify!($want), &e::$want, case);
                    n += 1;
                }
            )*
            assert_eq!(n, $count);
        }
    };
}

/// round7.c `r7_v`.
fn r7_v(v: &Vec) -> u32 {
    hmix(hmix(hmix(0, v.x.to_bits()), v.y.to_bits()), v.z.to_bits())
}
/// round7.c `r7_c`.
fn r7_c(g: &mut Gen) -> f32 {
    let s = (g.r() >> 16) % 8;
    if s == 0 {
        0.0
    } else if s < 3 {
        g.rf()
    } else {
        g.rm()
    }
}
/// round7.c `r7_vin`.
fn r7_vin(g: &mut Gen) -> Vec {
    let x = r7_c(g);
    let y = r7_c(g);
    let z = r7_c(g);
    Vec { x, y, z }
}
/// round9.c `r9_x`.
fn r9_x(g: &mut Gen) -> f32 {
    let s = (g.r() >> 16) % 4;
    let v = if s & 1 != 0 { g.rm() } else { g.rf() };
    if s >= 2 && v < 0.0 { -v } else { v }
}
/// round9.c `r9_i`.
fn r9_i(g: &mut Gen) -> i32 {
    if (g.r() >> 16) & 3 != 0 { ((g.r() >> 16) % 200) as i32 } else { g.r() as i32 }
}
/// round9.c `SEED()`.
fn seed(g: &mut Gen) -> HsdRand {
    HsdRand { seed: g.r(), ..HsdRand::default() }
}
/// round9.c `SEED_HI()`: a seed whose next draw is > 0.9.
fn seed_hi(g: &mut Gen) -> HsdRand {
    loop {
        let s = g.r();
        if (s.wrapping_mul(214013).wrapping_add(2531011) >> 16) >= 0xE667 {
            return HsdRand { seed: s, ..HsdRand::default() };
        }
    }
}
/// round9.c `r9_grav_pick`: a seed (next draw > 0.9) and `v` where the fused `0.6 r + |v|`
/// differs from two f32 roundings.
fn r9_grav_pick(g: &mut Gen) -> (HsdRand, f32) {
    loop {
        let s = g.r();
        let k = s.wrapping_mul(214013).wrapping_add(2531011) >> 16;
        let v = g.rm();
        let w = v.abs();
        let r = k as f32 / 65536.0;
        let q = 0.6f32 * r + w;
        let e = 0.6f32 as f64 * r as f64 + w as f64;
        if k >= 0xE667 && e as f32 != q {
            return (HsdRand { seed: s, ..HsdRand::default() }, v);
        }
    }
}
/// round9.c `SEED_H(x)`.
fn seed_h(x: u32, r: &HsdRand) -> u32 {
    hmix(x, r.seed)
}



struct World {
    g: Box<HSD_GObj>,
    j: Box<[HSD_JObj; 4]>,
    md: Box<ItemModelDesc>,
    bt: Box<DynamicBoneTable>,
    gl: Box<[HSD_GObj; 3]>,
    li: Box<[Item; 2]>,
    la: Box<[ItemAttr; 2]>,
    lj: Box<[HSD_JObj; 2]>,
    track: Box<Item_FtTrack>,
}

/// round11.c `r11_jobjs` (+ the T macro's `memset(g); g->user_data = ip`).
fn r11_jobjs(ip: &mut Item, g: &mut Gen) -> World {
    let mut w = World {
        g: Box::new(HSD_GObj::zeroed()),
        j: Box::new([HSD_JObj::zeroed(), HSD_JObj::zeroed(), HSD_JObj::zeroed(), HSD_JObj::zeroed()]),
        md: Box::new(ItemModelDesc::zeroed()),
        bt: Box::new(DynamicBoneTable::zeroed()),
        gl: Box::new([HSD_GObj::zeroed(), HSD_GObj::zeroed(), HSD_GObj::zeroed()]),
        li: Box::new([Item::zeroed(), Item::zeroed()]),
        la: Box::new([ItemAttr::zeroed(), ItemAttr::zeroed()]),
        lj: Box::new([HSD_JObj::zeroed(), HSD_JObj::zeroed()]),
        track: Box::new(Item_FtTrack::zeroed()),
    };
    w.g.user_data = ip as *mut Item as *mut _;
    let base = w.j.as_mut_ptr();
    for k in 0..4 {
        let j = &mut w.j[k];
        j.child = if k < 3 { unsafe { base.add(k + 1) } } else { null_mut() };
        j.rotate.x = r7_c(g);
        j.rotate.y = r7_c(g);
        j.rotate.z = r7_c(g);
        j.rotate.w = r7_c(g);
        j.translate.x = g.rm();
        j.translate.y = g.rm();
        j.translate.z = g.rm();
    }
    w.g.hsd_obj = base as *mut _;
    w
}
fn r11_jidx(w: &World, j: *mut HSD_JObj) -> u32 {
    if j.is_null() {
        return 4;
    }
    for k in 0..4 {
        if j as *const HSD_JObj == &w.j[k] as *const HSD_JObj {
            return k as u32;
        }
    }
    5
}
/// round11.c `r11_model`.
fn r11_model(w: &mut World, ip: &mut Item, g: &mut Gen, x: &mut Ext) {
    *w.md = ItemModelDesc::zeroed();
    w.md.x4_bone_count = if g.r() & 1 != 0 { 0 } else { (g.r() >> 16) % 8 + 1 };
    w.md.xC_bit_field = (g.r() >> 8) as u8;
    x.art.x10_modelDesc = &mut *w.md;
    let base = w.j.as_mut_ptr();
    for k in 0..8 {
        w.bt.bones[k] = if g.r() % 5 != 0 { unsafe { base.add((g.r() % 4) as usize) } } else { null_mut() };
    }
    ip.xBBC_dynamicBoneTable = &mut *w.bt;
}
/// round11.c `r11_ecb`.
fn r11_ecb(e: &mut itECB, g: &mut Gen) {
    e.top = g.rm().abs() / 8.0;
    e.bottom = -(g.rm().abs() / 8.0);
    e.right = -(g.rm().abs() / 8.0);
    e.left = g.rm().abs() / 8.0;
}
fn r11_h(ip: &Item) -> u32 {
    hmix(hmix(hmix(ip.x70_nudge.x.to_bits(), ip.xDC0), ip.x70_nudge.y.to_bits()), ip.x70_nudge.x.to_bits() >> 31)
}
/// round11.c `r11_list`: returns the list head.
fn r11_list(w: &mut World, ip: &mut Item, g: &mut Gen) -> *mut HSD_GObj {
    let n = g.r() % 4;
    for k in 0..2 {
        w.li[k] = Item::zeroed();
        w.la[k] = ItemAttr::zeroed();
        w.lj[k] = HSD_JObj::zeroed();
        let li = &mut w.li[k];
        li.xDC8_word.flags.bf_0 = g.r().to_be_bytes();
        li.ground_or_air = if g.r() & 3 != 0 { GA_Ground } else { GA_Air };
        unsafe { li.xDD1_flag.x0.set_b0((g.r() & 3 == 0) as u8) };
        w.la[k].set_x0_is_heavy((g.r() & 3 != 0) as u8);
        li.xCC_item_attr = &mut w.la[k];
        li.x70_nudge.x = r7_c(g);
        r11_ecb(&mut li.xBEC, g);
        w.lj[k].translate.x = g.rm() / 4.0;
        w.lj[k].translate.y = g.rm() / 4.0;
        w.lj[k].translate.z = g.rm();
    }
    for k in 0..3 {
        w.gl[k] = HSD_GObj::zeroed();
    }
    for k in 0..2 {
        w.gl[k + 1].user_data = &mut w.li[k] as *mut Item as *mut _;
        w.gl[k + 1].hsd_obj = &mut w.lj[k] as *mut HSD_JObj as *mut _;
    }
    r11_ecb(&mut ip.xBEC, g);
    w.j[0].translate.x = g.rm() / 4.0;
    w.j[0].translate.y = g.rm() / 4.0;
    if g.r() & 1 == 0 {
        w.lj[0].translate.x = w.j[0].translate.x;
        w.lj[0].translate.y = w.j[0].translate.y;
        while w.li[0].xDC8_word.flags.x13() != 0 {
            w.li[0].xDC8_word.flags.bf_0 = g.r().to_be_bytes();
        }
        w.li[0].ground_or_air = GA_Ground;
        unsafe { w.li[0].xDD1_flag.x0.set_b0(0) };
        w.la[0].set_x0_is_heavy(1);
    }
    w.gl[0] = unsafe { core::ptr::read(&*w.g) };
    let base = w.gl.as_mut_ptr();
    for k in 0..2 {
        w.gl[k].next = unsafe { base.add(k + 1) };
    }
    let gp: *mut HSD_GObj = &mut *w.g;
    let head = match n {
        0 => null_mut(),
        1 => unsafe { base.add(1) },
        2 => gp,
        _ => base,
    };
    if n == 2 {
        w.g.next = unsafe { base.add(1) };
    }
    head
}
/// round11.c `r11_trk`.
fn r11_trk(w: &mut World, ip: &mut Item, g: &mut Gen) {
    let t = &mut *w.track;
    *t = Item_FtTrack::zeroed();
    t.count = ((g.r() >> 16) % 12) as usize;
    unsafe { t.x154.x0.set_b0((g.r() & 3 == 0) as u8) };
    for k in 0..11 {
        r11_ecb(&mut t.ecb_offset_arr[k], g);
        t.ft_pos_arr[k].x = g.rm() / 4.0;
        t.ft_pos_arr[k].y = g.rm() / 4.0;
        t.ft_pos_arr[k].z = g.rm();
    }
    r11_ecb(&mut ip.xBEC, g);
    w.j[0].translate.x = g.rm() / 4.0;
    w.j[0].translate.y = g.rm() / 4.0;
    if g.r() & 1 == 0 {
        let i = if g.r() & 1 != 0 && t.count != 0 { t.count - 1 } else { 0 };
        t.ft_pos_arr[i].x = w.j[0].translate.x;
        t.ft_pos_arr[i].y = w.j[0].translate.y;
    }
}

cases! { hand_match_c, 7;
    3101 R11_80272CC0 |ip, g, ov, x| { let mut w = r11_jobjs(ip, g); r11_model(&mut w, ip, g, x); let idx = ((g.r() >> 16) % 6) as i32; let r = it_80272cc0(&mut *w.g, idx); r11_jidx(&w, r) };
    3102 R11_80274990 |ip, g, ov, x| { let mut w = r11_jobjs(ip, g); r11_model(&mut w, ip, g, x); it_80274990(&mut *w.g).to_bits() };
    3103 R11_80271B60 |ip, g, ov, x| { let mut w = r11_jobjs(ip, g); x.common.x7C_float = g.rm(); r11_trk(&mut w, ip, g); let mut r = seed(g); let c = x.common; it_80271b60(&mut r, &mut *w.g, &w.track, &c); seed_h(r11_h(ip), &r) };
    3104 R11_80271F78 |ip, g, ov, x| { let mut w = r11_jobjs(ip, g); x.common.x7C_float = g.rm(); let head = r11_list(&mut w, ip, g); let mut r = seed(g); let c = x.common; it_80271f78(&mut r, &mut *w.g, head, &c); seed_h(r11_h(ip), &r) };
    3105 R11_LINKARROW_802A850C_INLINE_2 |ip, g, ov, x| { r4_pre(ip, g, x); let mut w = r11_jobjs(ip, g); ip.xDD4_itemVar.linkarrow.x18 = r7_vin(g); let mut q = Quaternion { x: 0.0, y: 0.0, z: 0.0, w: 0.0 }; it_link_arrow_802a850c_inline_2(&mut *w.g, &mut q); hmix(hmix(hmix(hmix(r4_h(ip), q.x.to_bits()), q.y.to_bits()), q.z.to_bits()), q.w.to_bits()) };
    3106 R11_CALC_DIST_2D_ACCURATE |ip, g, ov, x| { let v = r7_vin(g); let u = calc_dist_2d_accurate(&v).to_bits(); hmix(hmix(0, u as u32), (u >> 32) as u32) };
    3107 R11_CALC_DIST_2D_ACCURATE_FUSED |ip, g, ov, x| { let mut v = r7_vin(g); let mut k = 0i32; while k < 4000 { if k > 0 { v = r7_vin(g); } let yy = v.y * v.y; let xx = v.x * v.x; let dx = v.x as f64 * v.x as f64; if (dx + yy as f64) as f32 != xx + yy { break; } k += 1; } let u = calc_dist_2d_accurate(&v).to_bits(); hmix(hmix(k as u32, u as u32), (u >> 32) as u32) };
}
