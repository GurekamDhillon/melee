//! Round 14 (night 3, round 6): src/itkinds14.rs (26 hand ports) against tests/c_oracle/round14.c
//! (contraction on + gwtool lowering, as the shipping build). Same harness as round 13 (the
//! infrastructure below is a copy of oracle_round13.rs's, without round 13's own cases).
#![allow(non_upper_case_globals, dead_code)]

#[path = "c_oracle/expected.rs"]
#[allow(dead_code)]
mod expected;
#[path = "round2/fields.rs"]
mod fields;

use expected as e;
use melee_items::itkinds13::*;
use melee_items::itkinds14::*;
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
    unk: itUnkAttributes,
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
        unk: itUnkAttributes::zeroed(),
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
        x.unk = itUnkAttributes::zeroed();
        x.common = ItemCommonData::zeroed();
        x.attr.set_x0_is_heavy(g.r() as u8);
        x.attr.set_x0_78(g.r() as u8);
        x.attr.set_x0_hold_kind(g.r() as u8);
        x.attr.x1C_damage_mul = g.rf();
        x.unk.x0_float = g.rf();
        x.unk.x4_float = g.rf();
        x.art.x4_specialAttributes = &mut x.unk as *mut itUnkAttributes as *mut _;
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
        unsafe { li.xDD1_flag.anon_0.set_b0((g.r() & 3 == 0) as u8) };
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
        unsafe { w.li[0].xDD1_flag.anon_0.set_b0(0) };
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
    unsafe { t.x154.anon_0.set_b0((g.r() & 3 == 0) as u8) };
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

fn r10_h(ip: &mut Item) -> u32 {
    let mut h = hmix(hmix(hmix(0, r7_v(&ip.x7C)), r7_v(&ip.x88)), r7_v(&ip.x94));
    h = hmix(hmix(h, r7_v(&ip.xA0)), r7_v(&ip.x40_vel));
    h = hmix(hmix(hmix(hmix(h, ip.xBEC.top.to_bits()), ip.xBEC.bottom.to_bits()), ip.xBEC.right.to_bits()),
        ip.xBEC.left.to_bits());
    h = hmix(hmix(hmix(hmix(h, ip.xC30 as u32), ip.xD58), ip.xD5C), ip.facing_dir.to_bits());
    hmix(h, r4_h(ip))
}
/// round10.c `r10_coll`.
fn r10_coll(ip: &mut Item, g: &mut Gen) {
    let c = &mut ip.x378_itemColl;
    c.env_flags = (g.r() >> 8) as i32;
    c.floor.normal = r7_vin(g);
    c.ceiling.normal = r7_vin(g);
    c.left_facing_wall.normal = r7_vin(g);
    c.right_facing_wall.normal = r7_vin(g);
    c.left_facing_wall.index = ((g.r() >> 16) % 8) as i32;
    c.right_facing_wall.index = ((g.r() >> 16) % 8) as i32;
}

struct L12 {
    gl: Box<[HSD_GObj; 3]>,
    li: Box<[Item; 3]>,
}
/// round12.c `r12_list`: returns the list (kept alive) and the head.
fn r12_list(g: &mut Gen) -> (L12, *mut HSD_GObj) {
    let mut l = L12 {
        gl: Box::new([HSD_GObj::zeroed(), HSD_GObj::zeroed(), HSD_GObj::zeroed()]),
        li: Box::new([Item::zeroed(), Item::zeroed(), Item::zeroed()]),
    };
    let gb = l.gl.as_mut_ptr();
    for k in 0..3 {
        let li = &mut l.li[k];
        li.hold_kind = (3 + (g.r() >> 16) % 6) as i32;
        li.grab_victim = if g.r() & 3 != 0 { null_mut() } else { gb };
        li.xDC8_word.flags.bf_0 = g.r().to_be_bytes();
        li.owner = if g.r() & 1 != 0 { null_mut() } else { unsafe { gb.add(1) } };
        li.pos.x = g.rm() / 4.0;
        li.pos.y = g.rm() / 4.0;
        li.pos.z = g.rm();
        let lp: *mut Item = li;
        l.gl[k].user_data = lp as *mut _;
        l.gl[k].next = if k < 2 { unsafe { gb.add(k + 1) } } else { null_mut() };
    }
    let head = if g.r() % 5 != 0 { gb } else { null_mut() };
    (l, head)
}
fn r12_gidx(l: &L12, p: *mut HSD_GObj) -> u32 {
    if p.is_null() {
        return 3;
    }
    for k in 0..3 {
        if p as *const HSD_GObj == &l.gl[k] as *const HSD_GObj {
            return k as u32;
        }
    }
    4
}
/// round12.c `r12_face`.
fn r12_face(g: &mut Gen) -> f32 {
    let s = (g.r() >> 16) % 4;
    match s {
        0 => -1.0,
        1 => 1.0,
        2 => g.rf(),
        _ => 0.5,
    }
}
fn unk_words(d: &mut HSD_ObjAllocUnk) -> &mut [i32] {
    let n = std::mem::size_of::<HSD_ObjAllocUnk>() / 4;
    // SAFETY: HSD_ObjAllocUnk is all 4-byte integers.
    unsafe { std::slice::from_raw_parts_mut(d as *mut HSD_ObjAllocUnk as *mut i32, n) }
}
// four distinct callbacks (distinct bodies, so they cannot be merged)
static EV_HIT: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
unsafe extern "C" fn r12_ev(_g: *mut HSD_GObj) { EV_HIT.fetch_add(1, std::sync::atomic::Ordering::Relaxed); }
unsafe extern "C" fn r12_ev2(_g: *mut HSD_GObj) { EV_HIT.fetch_add(2, std::sync::atomic::Ordering::Relaxed); }
unsafe extern "C" fn r12_in(_a: *mut HSD_GObj, _b: *mut HSD_GObj) { EV_HIT.fetch_add(3, std::sync::atomic::Ordering::Relaxed); }
unsafe extern "C" fn r12_in2(_a: *mut HSD_GObj, _b: *mut HSD_GObj) { EV_HIT.fetch_add(4, std::sync::atomic::Ordering::Relaxed); }
fn r12_evi(e: HSD_GObjEvent) -> u32 {
    match e {
        None => 0,
        Some(f) if f as usize == r12_ev as usize => 1,
        Some(_) => 2,
    }
}
fn r12_ini(e: HSD_GObjInteraction) -> u32 {
    match e {
        None => 0,
        Some(f) if f as usize == r12_in as usize => 1,
        Some(_) => 2,
    }
}
/// round12.c `r12_atk`.
fn r12_atk(ip: &Item) -> u32 {
    let mut h = hmix(hmix(0, ip.xD88_attackID as u32), ip.xD8C_attack_instance as u32);
    let w = unsafe { ip.xD90.x2070_int } as u32;
    h = hmix(hmix(hmix(h, w), ip.xD94.x.to_bits()), ip.xD94.y.to_bits());
    h = hmix(hmix(hmix(h, ip.xD9C.x as u32), ip.xD9C.y as u32), ip.xDA4_word);
    hmix(h, ip.xDA8_short as u32)
}

// ---- round 13 (round13.c 3301..3329) ---------------------------------------------------------

/// round13.c `r13_gm_fill` / `r13_gm_h`: the 12 bytes from `x90` on.
fn r13_gm_fill(gm: &mut UnkAllstarData, g: &mut Gen) {
    let p = unsafe { (gm as *mut UnkAllstarData as *mut u8).add(std::mem::offset_of!(UnkAllstarData, x90)) };
    for k in 0..12 {
        unsafe { *p.add(k) = (1 + (g.r() >> 16) % 255) as u8 };
    }
}
fn r13_gm_h(gm: &UnkAllstarData) -> u32 {
    let p = unsafe { (gm as *const UnkAllstarData as *const u8).add(std::mem::offset_of!(UnkAllstarData, x90)) };
    let mut h = 0;
    for k in 0..12 {
        h = hmix(h, unsafe { *p.add(k) } as u32);
    }
    h
}
struct Link {
    li2: Box<Item>,
    lg: Box<HSD_GObj>,
}
/// round13.c `r13_link`.
fn r13_link(ip: &mut Item, g: &mut Gen) -> Link {
    let mut l = Link { li2: Box::new(Item::zeroed()), lg: Box::new(HSD_GObj::zeroed()) };
    let lgp: *mut HSD_GObj = &mut *l.lg;
    unsafe { l.li2.xDD4_itemVar.freeze.x20 = lgp as *mut _ };
    l.lg.user_data = &mut *l.li2 as *mut Item as *mut _;
    unsafe { ip.xDD4_itemVar.freeze.unk_1C = if (g.r() >> 16) & 1 != 0 { lgp } else { null_mut() } };
    l
}
fn r13_link_h(ip: &Item, l: &Link) -> u32 {
    unsafe {
        hmix(hmix(0, ip.xDD4_itemVar.freeze.unk_1C.is_null() as u32), l.li2.xDD4_itemVar.freeze.x20.is_null() as u32)
    }
}
/// round13.c `r13_ll`: the Like-Like attrs' `x0.x0_f32` points at a drawn vector.
fn r13_ll(ip: &mut Item, g: &mut Gen, x: &mut Ext) -> Box<Vec> {
    r4_pre(ip, g, x);
    let mut v = Box::new(Vec { x: 0.0, y: 0.0, z: 0.0 });
    v.x = g.rf();
    v.y = g.rf();
    v.z = g.rf();
    unsafe { (*(x.sa.as_mut_ptr() as *mut itLikelikeAttributes)).x0.x0_f32 = &mut *v };
    ip.facing_dir = r12_face(g);
    v
}
/// round13.c `r13_sh`: r10_h plus the sign bits of facing_dir and vel.x as bit 0.
fn r13_sh(ip: &mut Item) -> u32 {
    let (f, v) = (ip.facing_dir.to_bits() >> 31, ip.x40_vel.x.to_bits() >> 31);
    hmix(hmix(r10_h(ip), f), v)
}
/// round13.c `r13_sc`.
fn r13_sc(g: &mut Gen) -> i32 {
    let s = (g.r() >> 16) % 4;
    if s == 0 { g.r() as i32 } else { ((g.r() >> 16) % 100) as i32 - 10 }
}
fn sa_ptr(x: &mut Ext) -> *mut u8 {
    x.sa.as_mut_ptr() as *mut u8
}

// ---- round 14 (round14.c 3401..3426) ---------------------------------------------------------

struct L14 {
    li: Box<[Item; 3]>,
    lg: Box<[HSD_GObj; 3]>,
}
/// round14.c `r14_lists`: three zeroed items, GObj k -> item k.
fn r14_lists() -> L14 {
    let mut l = L14 {
        li: Box::new([Item::zeroed(), Item::zeroed(), Item::zeroed()]),
        lg: Box::new([HSD_GObj::zeroed(), HSD_GObj::zeroed(), HSD_GObj::zeroed()]),
    };
    for k in 0..3 {
        l.lg[k].user_data = &mut l.li[k] as *mut Item as *mut _;
    }
    l
}
fn lgp(l: &mut L14, k: usize) -> *mut HSD_GObj {
    &mut l.lg[k]
}
fn r14_gidx(l: &L14, p: *const u8) -> u32 {
    if p.is_null() {
        return 3;
    }
    for k in 0..3 {
        if p == &l.lg[k] as *const HSD_GObj as *const u8 || p == &l.li[k] as *const Item as *const u8 {
            return k as u32;
        }
    }
    4
}
/// round14.c `r14_plist`: returns the lists and the head.
fn r14_plist(g: &mut Gen) -> (L14, *mut HSD_GObj) {
    let mut l = r14_lists();
    let b = l.lg.as_mut_ptr();
    for k in 0..3 {
        l.li[k].kind = ((g.r() >> 16) % 240) as _;
        l.lg[k].next = if k < 2 { unsafe { b.add(k + 1) } } else { null_mut() };
    }
    let head = if g.r() % 5 != 0 { b } else { null_mut() };
    (l, head)
}
fn r14_ptr<T>(p: *const T) -> u32 {
    let v = p as usize as u64;
    hmix(v as u32, (v >> 32) as u32)
}
fn self_gobj(ip: &mut Item) -> Box<HSD_GObj> {
    let mut gg = Box::new(HSD_GObj::zeroed());
    gg.user_data = ip as *mut Item as *mut _;
    gg
}
fn cmd_new(cu: &mut [CmdUnion; 2]) -> CommandInfo {
    let mut c: CommandInfo = unsafe { core::mem::zeroed() };
    c.anon_8.u = cu.as_mut_ptr();
    c
}

cases! { hand_match_c, 28;
    3401 R14_80285314 |ip, g, ov, x| { let mut w = r11_jobjs(ip, g); r11_model(&mut w, ip, g, x); let r = it_80285314(&mut *w.g); r11_jidx(&w, r) };
    3402 R14_GET_BONE_BY_ID |ip, g, ov, x| { let mut w = r11_jobjs(ip, g); r11_model(&mut w, ip, g, x); let idx = ((g.r() >> 16) % 6) as i32; let r = get_bone_by_id(&mut *w.g, idx); r11_jidx(&w, r) };
    3403 R14_80279768 |ip, g, ov, x| { let mut cu: [CmdUnion; 2] = core::mem::zeroed(); let mut cmd = cmd_new(&mut cu); ip.anon_dbc.xDBC_itcmd_var4.flags.set_x0((g.r() >> 16) & 1); ip.anon_dbc.xDBC_itcmd_var4.flags.set_x1((g.r() >> 16) & 1); it_80279768(ip, &mut cmd); let f = &ip.anon_dbc.xDBC_itcmd_var4.flags; hmix(hmix(f.x0(), f.x1()), cmd.anon_8.u.offset_from(cu.as_ptr()) as u32) };
    3404 R14_80279720 |ip, g, ov, x| { let mut cu: [CmdUnion; 2] = core::mem::zeroed(); let mut cmd = cmd_new(&mut cu); cu[0].set_throw_flags.set_opcode(g.r() & 63); cu[0].set_throw_flags.set_hit_idx(g.r() & 0x3FFFFFF); it_80279720(ip, &mut cmd); hmix(ip.xDB0_itcmd_var1, cmd.anon_8.u.offset_from(cu.as_ptr()) as u32) };
    3405 R14_80279744 |ip, g, ov, x| { let mut cu: [CmdUnion; 2] = core::mem::zeroed(); let mut cmd = cmd_new(&mut cu); cu[0].set_throw_flags.set_opcode(g.r() & 63); cu[0].set_throw_flags.set_hit_idx(g.r() & 0x3FFFFFF); it_80279744(ip, &mut cmd); hmix(ip.xDB4_itcmd_var2, cmd.anon_8.u.offset_from(cu.as_ptr()) as u32) };
    3406 R14_8028B08C |ip, g, ov, x| { let mut st: [ItemStateTable; 11] = core::mem::zeroed(); for k in 0..11 { st[k].anim_id = ((g.r() >> 16) % 8) as _; } let s = ((g.r() >> 16) % 11) as i32; it_8028b08c(&st, s) as u32 };
    3407 R14_YOSHISTAR_M0_PHYS |ip, g, ov, x| { r4_pre(ip, g, x); ip.facing_dir = r12_face(g); it_yoshistar_unk_motion0_phys(ip); r10_h(ip) };
    3408 R14_80291D38 |ip, g, ov, x| { r4_pre(ip, g, x); let n = ((g.r() >> 16) % 10) as i32; it_80291d38(ip, n) as u32 };
    3409 R14_802AB9C0 |ip, g, ov, x| { let mut l = r14_lists(); for k in 0..3 { l.li[k].owner = lgp(&mut l, 0); l.li[k].xDC8_word.flags.set_x13(1); } for k in 0..6 { ip.xDD4_itemVar.pkthunder.xDD4[k] = if (g.r() >> 16) % 3 != 0 { let j = ((g.r() >> 16) % 3) as usize; lgp(&mut l, j) } else { null_mut() }; } ip.xDD4_itemVar.pkthunder.xF00 = if (g.r() >> 16) & 1 != 0 { 0 } else { g.r() as i32 }; ip.owner = lgp(&mut l, 1); let mut gg = self_gobj(ip); let arg: *mut HSD_GObj = if (g.r() >> 16) & 7 != 0 { &mut *gg } else { null_mut() }; it_802ab9c0(arg); ip.xDD4_itemVar.pkthunder.xDD4 = [null_mut(); 6]; let mut h = hmix(0, ip.owner.is_null() as u32); for k in 0..3 { h = hmix(hmix(h, l.li[k].owner.is_null() as u32), l.li[k].xDC8_word.flags.x13()); } h };
    3410 R14_LIKELIKEVELOCITY |ip, g, ov, x| { let mut v = r13_ll(ip, g, x); let a = &*(x.sa.as_ptr() as *const itLikelikeAttributes); ip.xDD4_itemVar.likelike.x20.x = ip.pos.x + g.rm() / 64.0; ip.xDD4_itemVar.likelike.x20.y = if (g.r() >> 16) & 3 != 0 { ip.pos.y + g.rm() / 64.0 } else { g.rf() }; ip.xDD4_itemVar.likelike.x48 = ((g.r() >> 16) % 7) as i32; let mut m = if (g.r() >> 16) & 3 != 0 { g.rm() / 16.0 } else { g.rf() }; if (g.r() >> 16) & 1 != 0 { v.y = 4.0; m = 1.0; } likelike_velocity(ip, a, m); let _keep = v; hmix(r13_sh(ip), ip.xDD4_itemVar.likelike.x48 as u32) };
    3411 R14_SAMUSBOMB_M2_PHYS |ip, g, ov, x| { r4_pre(ip, g, x); ip.x7C = r7_vin(g); let a = &mut *(x.sa.as_mut_ptr() as *mut itSamusBombAttributes); a.xC = if (g.r() >> 16) & 3 != 0 { g.rm().abs() / 8.0 } else { g.rf() }; if (g.r() >> 16) & 3 == 0 { a.xC = melee_items::itkinds9::my_sqrtf_accurate(melee_core::fp::fmadds(ip.x7C.x, ip.x7C.x, ip.x7C.y * ip.x7C.y)); } it_samusbomb_unk_motion2_phys(ip); r10_h(ip) };
    3412 R14_PKFIREPILLAR_L24 |ip, g, ov, x| { r4_pre(ip, g, x); ip.xC9C = ((g.r() >> 16) % 300) as i32 - 20; ip.xD44_lifeTimer = if (g.r() >> 16) & 3 != 0 { g.rm() } else { g.rf() }; if (g.r() >> 16) & 7 == 0 { ip.xC9C = 0; ip.xD44_lifeTimer = 0.0; } let r = it_nesspkfirepillar_logic24_dmg_received(ip) as u32; hmix(r, ip.xD44_lifeTimer.to_bits()) };
    3413 R14_802B22B8_INLINE |ip, g, ov, x| { let mut l = r14_lists(); ip.xDD4_itemVar.pikachuthunder.x34 = if (g.r() >> 16) % 3 != 0 { lgp(&mut l, 0) } else { null_mut() }; l.lg[0].user_data = if (g.r() >> 16) & 3 != 0 { &mut l.li[0] as *mut Item as *mut _ } else { null_mut() }; l.li[0].xDD4_itemVar.pikachuthunder.x4 = g.r() as i32; it_802b22b8_inline(ip); ip.xDD4_itemVar.pikachuthunder.x34 = null_mut(); let p = l.li[0].xDD4_itemVar.pikachuthunder; hmix(hmix(hmix(hmix(0, p.x28.x.to_bits()), p.x28.y.to_bits()), p.x28.z.to_bits()), p.x4 as u32) };
    3414 R14_JOLTAIR_M0_ANIM_INLINE |ip, g, ov, x| { let mut l = r14_lists(); let mut gg = self_gobj(ip); let gp: *mut HSD_GObj = &mut *gg; ip.xDD4_itemVar.pikachujoltair.xDD8 = if (g.r() >> 16) % 3 != 0 { lgp(&mut l, 0) } else { null_mut() }; l.lg[0].user_data = if (g.r() >> 16) & 3 != 0 { &mut l.li[0] as *mut Item as *mut _ } else { null_mut() }; let s = (g.r() >> 16) % 3; l.li[0].xDD4_itemVar.pikachujoltground.xDDC = if s == 0 { gp } else if s == 1 { null_mut() } else { lgp(&mut l, 1) }; let r = it_pikachutjoltair_unk_motion0_anim_inline(gp, ip) as u32; ip.xDD4_itemVar.pikachujoltair.xDD8 = null_mut(); r };
    3415 R14_PIKATHUNDER_M2_SCALECALL |ip, g, ov, x| { r4_pre(ip, g, x); ip.x5D4_hitboxes[0].hit.state = if (g.r() >> 16) & 1 != 0 { HitCapsule_Disabled } else { 1 as _ }; ip.x5D4_hitboxes[0].hit.scale = g.rf(); ip.xDD4_itemVar.pikachuthunder.x10 = g.rm(); ip.xDD4_itemVar.pikachuthunder.xC = if (g.r() >> 16) & 3 != 0 { g.rm() } else { g.rf() }; it_pikachuthunder_unk_motion2_scale_call(ip); hmix(ip.x5D4_hitboxes[0].hit.scale.to_bits(), ip.x5D4_hitboxes[0].hit.state as u32) };
    3416 R14_BOMBHEI_80280974_INLINE |ip, g, ov, x| { r4_pre(ip, g, x); { let f = &mut ip.xDCD_flag.anon_0; f.set_b5(0); f.set_b6(0); f.set_b7(0); } ip.xD5C = g.r(); fn_80280974_inline(ip); let f = ip.xDCD_flag.anon_0; hmix(hmix(hmix(r10_h(ip), f.b5() as u32), f.b6() as u32), f.b7() as u32) };
    3417 R14_8026C47C |ip, g, ov, x| { let (_l, head) = r14_plist(g); let mut a = it_8026C47C_arg0_t::zeroed(); { let w = core::slice::from_raw_parts_mut(&mut a as *mut _ as *mut u32, 8); for k in 0..8 { w[k] = g.r(); } } it_8026c47c(head, &mut a); let w = core::slice::from_raw_parts(&a as *const _ as *const u32, 8); let mut h = 0; for k in 0..8 { h = hmix(h, w[k]); } h };
    3418 R14_802D14D0 |ip, g, ov, x| { it_802d14d0(); 0 };
    3419 R14_802F1340 |ip, g, ov, x| { it_802f1340(); 0 };
    3420 R14_802D472C_INLINE |ip, g, ov, x| { (it_802d472c_inline(ip) == ip.xC4_article_data) as u32 };
    3421 R14_GETFIGHTER |ip, g, ov, x| { let k = ((g.r() >> 16) % 3) as usize; let mut l = r14_lists(); ip.owner = lgp(&mut l, k); r14_gidx(&l, fn_802a33a0_get_fighter(ip) as *const u8) };
    3422 R14_ZGSHELL_GET_ATTRS |ip, g, ov, x| { r4_pre(ip, g, x); (zgshell_get_attrs(ip) as *mut u8 == sa_ptr(x)) as u32 };
    3423 R14_DOSEI_GETJOBJ |ip, g, ov, x| { let mut w = r11_jobjs(ip, g); let r = it_dosei_get_jobj(&mut *w.g); r11_jidx(&w, r) };
    3424 R14_JOBJ_CHILD |ip, g, ov, x| { let mut w = r11_jobjs(ip, g); let k = ((g.r() >> 16) % 5) as usize; let j: *mut HSD_JObj = if k < 4 { &mut w.j[k] } else { null_mut() }; let r = jobj_child(j); r11_jidx(&w, r) };
    3425 R14_CLIMBERSSTRING_JOINT24 |ip, g, ov, x| { r4_pre(ip, g, x); r14_ptr(it_802c248c_joint24(ip)) };
    3426 R14_CLIMBERSSTRING_JOINT28 |ip, g, ov, x| { r4_pre(ip, g, x); r14_ptr(it_802c248c_joint28(ip)) };
    3427 R14_SAMUSBOMB_M2_PHYS_EDGE |ip, g, ov, x| { r4_pre(ip, g, x); ip.x7C.x = g.rm(); ip.x7C.y = g.rm(); ip.x7C.z = g.rm(); let a = &mut *(x.sa.as_mut_ptr() as *mut itSamusBombAttributes); let l = melee_items::itkinds9::my_sqrtf_accurate(melee_core::fp::fmadds(ip.x7C.x, ip.x7C.x, ip.x7C.y * ip.x7C.y)); a.xC = if (g.r() >> 16) & 1 != 0 { l } else { f32::from_bits(l.to_bits().wrapping_sub(1)) }; it_samusbomb_unk_motion2_phys(ip); r10_h(ip) };
    3428 R14_LIKELIKEVELOCITY_EDGE |ip, g, ov, x| { const D: [f32; 4] = [4.0, -4.0, 3.5, -4.5]; let mut v = r13_ll(ip, g, x); let a = &*(x.sa.as_ptr() as *const itLikelikeAttributes); v.y = 4.0; ip.pos.x = g.rm() / 64.0; ip.pos.y = g.rm() / 64.0; ip.xDD4_itemVar.likelike.x20.x = ip.pos.x + D[((g.r() >> 16) % 4) as usize]; ip.xDD4_itemVar.likelike.x20.y = ip.pos.y + D[((g.r() >> 16) % 4) as usize]; ip.xDD4_itemVar.likelike.x48 = ((g.r() >> 16) % 7) as i32; likelike_velocity(ip, a, 1.0); let _keep = v; hmix(r13_sh(ip), ip.xDD4_itemVar.likelike.x48 as u32) };
}
