//! Round 12 (night 3, round 4): src/itkinds12.rs (15 hand ports) against tests/c_oracle/round12.c
//! (contraction on + gwtool lowering, as the shipping build). Same harness as round 11 (the
//! infrastructure below is a copy of oracle_round11.rs's, plus round 10's r10_h / r10_coll).
#![allow(non_upper_case_globals, dead_code)]

#[path = "c_oracle/expected.rs"]
#[allow(dead_code)]
mod expected;
#[path = "round2/fields.rs"]
mod fields;

use expected as e;
use melee_items::itkinds11::*;
use melee_items::itkinds12::*;
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

cases! { hand_match_c, 15;
    3201 R12_8026B1D4 |ip, g, ov, x| { let mut gg = HSD_GObj::zeroed(); gg.user_data = ip as *mut Item as *mut _; let mut hc = HitCapsule::zeroed(); hc.damage = if g.r() & 3 != 0 { g.rm() / 8.0 } else { g.rf() }; ip.x40_vel = r7_vin(g); x.common.x80_float[5] = g.rm() / 64.0; x.common.x80_float[6] = g.rm() / 64.0; let c = x.common; it_8026b1d4(&mut gg, &hc, &c).to_bits() };
    3202 R12_8026C258 |ip, g, ov, x| { let (l, head) = r12_list(g); let mut p = Vec { x: g.rm() / 4.0, y: g.rm() / 4.0, z: g.rm() }; if g.r() & 3 == 0 { p.x = l.li[(g.r() % 3) as usize].pos.x; } let fd = r12_face(g); let r = it_8026c258(head, &p, fd); r12_gidx(&l, r) };
    3203 R12_ITEM_8026784C |ip, g, ov, x| { let d = (g.r() >> 16) % 14; let mut u = HSD_ObjAllocUnk::zeroed(); for w in unk_words(&mut u).iter_mut() { *w = ((g.r() >> 16) % 4) as i32 - 1; } let r = item_8026784c(d as i32, g.r() as i32, &mut u) as u32; let mut h = 0; for w in unk_words(&mut u).iter() { h = hmix(h, *w as u32); } hmix(r, h) };
    3204 R12_8026C1D4 |ip, g, ov, x| { let mut arts: [*mut Article; 20] = [null_mut(); 20]; let mut art = Article::zeroed(); let mut lg = itLGun_ItemVars { timer: g.r() as i32 }; art.x4_specialAttributes = &mut lg as *mut itLGun_ItemVars as *mut _; arts[It_Kind_L_Gun as usize] = &mut art; it_8026c1d4(arts.as_ptr()) };
    3205 R12_8026B7D8 |ip, g, ov, x| { let c = g.r() as i32; it_8026b7d8(&c) as u32 };
    3206 R12_8026B7E0 |ip, g, ov, x| { let c = g.r() as i32; it_8026b7e0(&c) as u32 };
    3207 R12_80274ED8 |ip, g, ov, x| { let mut c = if g.r() & 1 != 0 { 0x7FFFFFFF } else { g.r() as i32 }; it_80274ed8(&mut c); c as u32 };
    3208 R12_80274EE8 |ip, g, ov, x| { let mut c = if g.r() & 1 != 0 { 0x7FFFFFFF } else { g.r() as i32 }; it_80274ee8(&mut c); c as u32 };
    3209 R12_8026F9A0 |ip, g, ov, x| { let mut c = g.r(); it_8026f9a0(&mut c); c };
    3210 R12_80272C90 |ip, g, ov, x| { let mut w = r11_jobjs(ip, g); r11_model(&mut w, ip, g, x); w.md.x8_bone_attach_id = ((g.r() >> 16) % 6) as i32; let r = it_80272c90(&mut *w.g); r11_jidx(&w, r) };
    3211 R12_802746F8 |ip, g, ov, x| { let k = g.r() % 4; let mut w = r11_jobjs(ip, g); r11_model(&mut w, ip, g, x); if k == 0 { w.g.hsd_obj = null_mut(); } else if k == 1 { w.j[1].child = null_mut(); } let r = it_802746f8(&mut *w.g); r11_jidx(&w, r) };
    3212 R12_80274F28 |ip, g, ov, x| { let a = g.r() % 3; let b = g.r() % 3; let e: HSD_GObjEvent = match a { 0 => None, 1 => Some(r12_ev), _ => Some(r12_ev2) }; let i: HSD_GObjInteraction = match b { 0 => None, 1 => Some(r12_in), _ => Some(r12_in2) }; ip.xD08 = g.r() as u8;
        // xDD0_flag is filled and hashed by value in GC bit order on both sides (round2 ufs_set / ufs_get)
        it_80274f28(ip, g.r() as i8, e, i); let dd0 = ip.xDD0_flag.byte as u32;
        hmix(hmix(hmix(hmix(0, dd0), ip.xD08 as u32), r12_evi(ip.grab_dealt)), r12_ini(ip.grabbed_for_victim)) };
    3213 R12_80277544 |ip, g, ov, x| { r4_pre(ip, g, x); r10_coll(ip, g); x.attr.x50 = if (g.r() >> 16) % 4 != 0 { r7_c(g) } else { g.rm() / 16777216.0 }; ip.x88 = r7_vin(g); ip.xAC_unk = r7_vin(g); ip.xC2C = ((g.r() >> 16) % 3) as i32; ip.xC30 = ((g.r() >> 16) % 3) as i32; ip.xD58 = (g.r() >> 16) % 4; ip.xD5C = (g.r() >> 16) % 3; x.common.xC0 = g.rm() / 64.0; x.common.xC4 = g.rm() / 64.0; x.common.x38_float = (g.r() >> 16) % 4; ip.x40_vel = r7_vin(g); ip.x70_nudge = r7_vin(g); x.common.xCC = if g.r() & 1 != 0 { g.rm() } else { r7_c(g) }; let c = x.common; let r = it_80277544(ip, &c) as u32; hmix(r, r10_h(ip)) };
    3214 R12_802775F0 |ip, g, ov, x| { ip.spin_spd = if g.r() & 3 != 0 { g.rm() } else { 0.0 }; let a = r7_vin(g); ip.xC1C.bottom = if g.r() & 7 != 0 { g.rm() / 8.0 } else { 0.0 }; ip.facing_dir = r12_face(g); it_802775f0(ip, &a); ip.xD3C_spinSpeed.to_bits() };
    3215 R12_8027B070 |ip, g, ov, x| { let mut fp: Box<Fighter> = Box::new(Fighter::zeroed()); let mut fg = HSD_GObj::zeroed(); fp.x2068_attackID = g.r() as i32; fp.x206C_attack_instance = g.r() as u16; fp.x2070.x2070_int = g.r() as i32; fp.x2074.x2074_vec.x = g.rf(); fp.x2074.x2074_vec.y = g.rf(); fp.x2074.x207C.x = g.r() as i32; fp.x2074.x207C.y = g.r() as i32; fp.x2074.anon_10.x2084 = g.r(); fp.x2074.x2088 = g.r() as u16; fg.user_data = &mut *fp as *mut Fighter as *mut _; it_8027b070(ip, &mut fg); r12_atk(ip) };
}

// ---- fused-site / boundary tables (round12.c 3216..3222) ----------------------------------
// One f32 rounding of an exact f64 product + addend, and plain f32 ops: round12.c's selectors.
fn r12_fm(a: f32, b: f32, c: f32) -> f32 {
    (a as f64 * b as f64 + c as f64) as f32
}
fn r12_b1d4(v: &Vec, k5: f32, k6: f32, dmg: f32, var: i32) -> f32 {
    let (xx, yy, zz) = (v.x * v.x, v.y * v.y, v.z * v.z);
    let inn = if var == 2 { r12_fm(v.y, v.y, xx) } else { r12_fm(v.x, v.x, yy) };
    let sq = if var == 1 { zz + inn } else { r12_fm(v.z, v.z, inn) };
    let r = r12_fm(melee_runtime::gekko::sqrtf(sq), k5, dmg) + k6;
    if r as f64 <= 1.0 { 1.0 } else { r }
}
fn r12_b1d4_pick(ip: &mut Item, hc: &mut HitCapsule, x: &mut Ext, g: &mut Gen, var: i32) -> i32 {
    let mut k = 0;
    while k < 4000 {
        ip.x40_vel = r7_vin(g);
        x.common.x80_float[5] = g.rm() / 64.0;
        x.common.x80_float[6] = g.rm() / 64.0;
        hc.damage = g.rm() / 8.0;
        let (k5, k6) = (x.common.x80_float[5], x.common.x80_float[6]);
        if r12_b1d4(&ip.x40_vel, k5, k6, hc.damage, 0).to_bits() != r12_b1d4(&ip.x40_vel, k5, k6, hc.damage, var).to_bits() {
            break;
        }
        k += 1;
    }
    k
}
fn r12_544_draw(ip: &mut Item, g: &mut Gen, x: &mut Ext) {
    r4_pre(ip, g, x); r10_coll(ip, g); x.attr.x50 = if (g.r() >> 16) % 4 != 0 { r7_c(g) } else { g.rm() / 16777216.0 }; ip.x88 = r7_vin(g); ip.xAC_unk = r7_vin(g); ip.xC2C = ((g.r() >> 16) % 3) as i32; ip.xC30 = ((g.r() >> 16) % 3) as i32; ip.xD58 = (g.r() >> 16) % 4; ip.xD5C = 1 + (g.r() >> 16) % 2; x.common.xC0 = g.rm() / 64.0; x.common.xC4 = g.rm() / 64.0; x.common.x38_float = (g.r() >> 16) % 4; ip.x40_vel = r7_vin(g); ip.x70_nudge = r7_vin(g);
}
/// round12.c `r12_544_pick`.
unsafe fn r12_544_pick(ip: &mut Item, g: &mut Gen, x: &mut Ext, mode: i32) -> i32 {
    let mut k = 0;
    while k < 2000 {
        r12_544_draw(ip, g, x);
        let mut tmp: Box<Item> = Box::new(core::ptr::read(ip));
        let c = x.common;
        if !melee_items::itkinds10::it_80277040(&mut tmp, &c) {
            k += 1;
            continue;
        }
        if mode == 1 {
            x.common.xCC = f32::from_bits(0x7F7FFFFF);
            break;
        }
        let sx = tmp.x40_vel.x + tmp.x70_nudge.x;
        let sy = tmp.x40_vel.y + tmp.x70_nudge.y;
        x.common.xCC = r12_fm(sx, sx, sy * sy);
        if sx * sx + sy * sy < x.common.xCC {
            break;
        }
        k += 1;
    }
    assert!(k < 2000, "r12_544_pick found no draw (mode {mode})");
    k
}

cases! { fused_match_c, 6;
    3216 R12_8026B1D4_FUSED_OUTER |ip, g, ov, x| { let mut gg = HSD_GObj::zeroed(); gg.user_data = ip as *mut Item as *mut _; let mut hc = HitCapsule::zeroed(); ip.xDC8_word.flags.set_x14(1); let k = r12_b1d4_pick(ip, &mut hc, x, g, 1); assert!(k < 4000); let c = x.common; hmix(k as u32, it_8026b1d4(&mut gg, &hc, &c).to_bits()) };
    3217 R12_8026B1D4_FUSED_INNER |ip, g, ov, x| { let mut gg = HSD_GObj::zeroed(); gg.user_data = ip as *mut Item as *mut _; let mut hc = HitCapsule::zeroed(); ip.xDC8_word.flags.set_x14(1); let k = r12_b1d4_pick(ip, &mut hc, x, g, 2); assert!(k < 4000); let c = x.common; hmix(k as u32, it_8026b1d4(&mut gg, &hc, &c).to_bits()) };
    3218 R12_8026C258_TIE |ip, g, ov, x| { let (mut l, _) = r12_list(g); for i in 0..3 { l.li[i].hold_kind = if i < 2 { 4 } else { 3 }; l.li[i].grab_victim = null_mut(); l.li[i].owner = null_mut(); } let head = l.gl.as_mut_ptr(); let p = Vec { x: 0.0, y: 0.0, z: g.rm() }; let m = (g.r() >> 16) & 1; let mut k = 0i32;
        if m == 0 { l.li[1].pos = l.li[0].pos; } else { while k < 4000 { let a = g.rm() / 4.0; let b = g.rm() / 4.0; l.li[0].pos.x = a; l.li[0].pos.y = b; l.li[1].pos.x = b; l.li[1].pos.y = a; if r12_fm(b, b, a * a) < r12_fm(a, a, b * b) { break; } k += 1; } assert!(k < 4000); }
        let r = it_8026c258(head, &p, 0.5); hmix(hmix(m, k as u32), r12_gidx(&l, r)) };
    3220 R12_80277544_FUSED |ip, g, ov, x| { let k = r12_544_pick(ip, g, x, 0); let c = x.common; let r = it_80277544(ip, &c) as u32; hmix(hmix(k as u32, r), r10_h(ip)) };
    3221 R12_80277544_D5C |ip, g, ov, x| { let k = r12_544_pick(ip, g, x, 1); let c = x.common; let r = it_80277544(ip, &c) as u32; hmix(hmix(k as u32, r), r10_h(ip)) };
    3222 R12_ITEM_8026784C_ALL |ip, g, ov, x| { let mut h = 0u32; for d in 0..14 { for _rep in 0..4 { let mut u = HSD_ObjAllocUnk::zeroed(); for w in unk_words(&mut u).iter_mut() { *w = ((g.r() >> 16) % 4) as i32 - 1; } let r = item_8026784c(d, 0, &mut u) as u32; let mut uh = 0; for w in unk_words(&mut u).iter() { uh = hmix(uh, *w as u32); } h = hmix(hmix(h, r), uh); } } h };
}
