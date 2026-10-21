//! Round 9: src/itkinds9.rs (26 hand ports: HSD_Randi / HSD_Randf and __frsqrte callers)
//! against tests/c_oracle/round9.c (contraction on + gwtool lowering, as the shipping build),
//! which compiles the game's random.c. Same harness as rounds 4-8 (the infrastructure below is a
//! copy of oracle_round8.rs's); each RNG case seeds `HsdRand` from the case LCG like the C seeds
//! `*HSD_RandSeedPtr`, and hashes the seed after the call (draw count and order).
#![allow(non_upper_case_globals, dead_code)]

#[path = "c_oracle/expected.rs"]
#[allow(dead_code)]
mod expected;
#[path = "round2/fields.rs"]
mod fields;

use expected as e;
use melee_items::itkinds9::*;
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

cases! { hand_match_c, 29;
    2901 R9_SQRTF_STORE |ip, g, ov, x| { let mut y = 7.0f32; let v = r9_x(g); let r = sqrtf_store(v, &mut y); hmix(r.to_bits(), y.to_bits()) };
    2902 R9_ITZAKO_SQRTF |ip, g, ov, x| itzako_sqrtf(r9_x(g)).to_bits();
    2903 R9_TYDISPLAY_SQRT_STORE |ip, g, ov, x| { let mut y = 7.0f32; let v = r9_x(g); let r = ty_display_80318cb4_sqrt_store(v, &mut y); hmix(r.to_bits(), y.to_bits()) };
    2904 R9_SQRTF_ACCURATE_STORE |ip, g, ov, x| { let mut y = 7.0f32; let v = r9_x(g); let r = sqrtf_accurate_store(v, &mut y); hmix(r.to_bits(), y.to_bits()) };
    2905 R9_SQRTF_ACCURATE_LOCAL |ip, g, ov, x| sqrtf_accurate_local(r9_x(g)).to_bits();
    2906 R9_SQRTF_ACCURATE_SP18 |ip, g, ov, x| sqrtf_accurate_sp18(r9_x(g)).to_bits();
    2907 R9_MY_SQRTF |ip, g, ov, x| my_sqrtf(r9_x(g)).to_bits();
    2908 R9_MY_SQRTF_ACCURATE |ip, g, ov, x| my_sqrtf_accurate(r9_x(g)).to_bits();
    2909 R9_TOY_80309338 |ip, g, ov, x| { let a = r7_vin(g); let b = r7_vin(g); let s = (g.r() >> 16) % 8; toy_80309338(if s == 0 { None } else { Some(&a) }, if s == 1 { None } else { Some(&b) }).to_bits() };
    2910 R9_RANDI_PERM |ip, g, ov, x| { let f = g.rm() * 256.0; let mut r = seed(g); let v = randi_perm(&mut r, f) as u32; seed_h(v, &r) };
    2911 R9_RANDI_PERM_INT |ip, g, ov, x| { let i = r9_i(g); let mut r = seed(g); let v = randi_perm_int(&mut r, i) as u32; seed_h(v, &r) };
    2912 R9_KYASARIN_RANDI |ip, g, ov, x| { let i = r9_i(g); let mut r = seed(g); let v = it_kyasarin_randi(&mut r, i) as u32; seed_h(v, &r) };
    2913 R9_ATTRRAND |ip, g, ov, x| { let mut a = itEgg_ItemVars::zeroed(); a.rand_max = r9_i(g); let mut r = seed(g); let v = attr_rand(&mut r, &a) as u32; seed_h(v, &r) };
    2914 R9_8026C65C |ip, g, ov, x| { let n = 1 + ((g.r() >> 16) % 32) as i32; let mut tab = [0u16; 33]; let mut kinds = [0u8; 33]; let mut acc = 0i32; for k in 0..=n as usize { tab[k] = acc as u16; kinds[k] = (g.r() >> 16) as u8; acc += 1 + ((g.r() >> 16) % 50) as i32; } let mut t = ItemPickTable::zeroed(); t.size = n as u8; t.x4 = kinds.as_mut_ptr(); t.x8 = tab[n as usize]; t.xC = tab.as_mut_ptr(); let mut r = seed(g); let v = it_8026c65c(&mut r, &t) as u32; seed_h(v, &r) };
    2915 R9_TARU_RANDCHECK |ip, g, ov, x| { let mut a = itTaruAttributes::zeroed(); a.x0 = r9_i(g); a.x4 = r9_i(g); let mut r = seed(g); let v = it_taru_rand_check(&mut r, &a) as u32; seed_h(v, &r) };
    2916 R9_80278800_RAND_VEC |ip, g, ov, x| { let v = r7_vin(g); let mut r = seed(g); it_80278800_rand_vec(&mut r, ov, &v); seed_h(0, &r) };
    2917 R9_802CBAA8_INLINE |ip, g, ov, x| { r4_pre(ip, g, x); let mut r = seed(g); it_802cbaa8_inline(&mut r, ip); seed_h(r4_h(ip), &r) };
    2918 R9_802CBD24_INLINE |ip, g, ov, x| { r4_pre(ip, g, x); let mut r = seed(g); it_802cbd24_inline(&mut r, ip); seed_h(r4_h(ip), &r) };
    2919 R9_SAMUS_GRAPPLE_CALC_GRAV |ip, g, ov, x| { let v = r7_c(g); let mut r = seed(g); let o = samus_grapple_calc_grav(&mut r, v).to_bits(); seed_h(o, &r) };
    2920 R9_802B9328_GRAV |ip, g, ov, x| { let v = r7_c(g); let mut r = seed(g); let o = it_802b9328_grav(&mut r, v).to_bits(); seed_h(o, &r) };
    2927 R9_SAMUS_GRAPPLE_CALC_GRAV_HI |ip, g, ov, x| { let v = g.rm(); let mut r = seed_hi(g); let o = samus_grapple_calc_grav(&mut r, v).to_bits(); seed_h(o, &r) };
    2928 R9_802B9328_GRAV_HI |ip, g, ov, x| { let v = g.rm(); let mut r = seed_hi(g); let o = it_802b9328_grav(&mut r, v).to_bits(); seed_h(o, &r) };
    2929 R9_SAMUS_GRAPPLE_CALC_GRAV_FUSED |ip, g, ov, x| { let (mut r, v) = r9_grav_pick(g); let o = samus_grapple_calc_grav(&mut r, v).to_bits(); seed_h(o, &r) };
    2921 R9_8026F53C |ip, g, ov, x| { let mut v = r7_vin(g); let chk = (g.r() >> 16) & 1 != 0; x.common.x54_float = r7_c(g); x.attr.x18 = g.rf(); let mut r = seed(g); let c = x.common; it_8026f53c(&mut r, ip, &mut v, chk, &c); seed_h(r7_v(&v), &r) };
    2922 R9_80286340 |ip, g, ov, x| { let a = r9_i(g); let b = r9_i(g); let c = r9_i(g); let d = r9_i(g); let mut r = seed(g); let v = it_80286340(&mut r, a, b, c, d) as u32; seed_h(v, &r) };
    2923 R9_8028A114 |ip, g, ov, x| { let a = r9_i(g); let b = r9_i(g); let c = r9_i(g); let d = r9_i(g); let mut r = seed(g); let v = it_8028a114(&mut r, a, b, c, d) as u32; seed_h(v, &r) };
    2924 R9_80289BE8_INLINE |ip, g, ov, x| { let mut p = Vec { x: 0.0, y: 0.0, z: 0.0 }; let mut v = p; let s = r7_c(g); let mut r = seed(g); it_80289be8_inline(&mut r, ip, s, &mut p, &mut v); seed_h(hmix(r7_v(&p), r7_v(&v)), &r) };
    2925 R9_802E61C4 |ip, g, ov, x| { let mut a = Box::new(it_2E5A_Attrs::zeroed()); a.x24 = g.rm(); for k in 0..4 { a.x2C[k] = g.rm(); } x.art.x4_specialAttributes = &mut *a as *mut it_2E5A_Attrs as *mut _; let a1 = ((g.r() >> 16) % 2000) as i32 - 1000; let a2 = ((g.r() >> 16) % 4) as i32; let mut r = seed(g); let v = it_802e61c4(&mut r, ip, a1, a2) as u32; seed_h(v, &r) };
    2926 R9_802BD32C |ip, g, ov, x| { let mut a = Box::new(itPeachTurnipAttributes::zeroed()); a.x4_length = ((g.r() >> 16) % 8) as i32; for k in 0..8 { a.x8[k].x0_odds = ((g.r() >> 16) % 100) as i32; } x.art.x4_specialAttributes = &mut *a as *mut itPeachTurnipAttributes as *mut _; let mut r = seed(g); let v = it_802bd32c(&mut r, ip) as u32; seed_h(v, &r) };
}
