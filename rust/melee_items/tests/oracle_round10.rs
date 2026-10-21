//! Round 10: src/itkinds10.rs (23 hand ports: lbVector_* / atan2f / sinf / cosf callers, and
//! more HSD_Randi / HSD_Randf callers) against tests/c_oracle/round10.c (contraction on + gwtool
//! lowering, as the shipping build), which compiles the game's lbtrigf.c and verbatim copies of
//! the lbvector.c / mtx_pc.c callees. Same harness as round 9 (the infrastructure below is a copy
//! of oracle_round9.rs's); RNG cases seed `HsdRand` like the C seeds `*HSD_RandSeedPtr` and hash
//! the seed after the call.
#![allow(non_upper_case_globals, dead_code)]

#[path = "c_oracle/expected.rs"]
#[allow(dead_code)]
mod expected;
#[path = "round2/fields.rs"]
mod fields;

use expected as e;
use melee_items::itkinds10::*;
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


/// round10.c `r10_h`.
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
/// round10.c `r10_a`.
fn r10_a(g: &mut Gen) -> f32 {
    g.rm() / 8.0
}
fn sa<T>(x: &mut Ext) -> &'static mut T {
    // SAFETY: the 256-word block outlives the case; every attribute struct used fits in it.
    unsafe { &mut *(x.sa.as_mut_ptr() as *mut T) }
}

/// round10.c `r10_dot_pick`.
fn r10_dot_pick(g: &mut Gen) -> (Vec, Vec) {
    loop {
        let v = Vec { x: g.rm(), y: g.rm(), z: g.rm() };
        let mut n = Vec { x: g.rm(), y: g.rm(), z: 0.0 };
        if v.z == 0.0 {
            continue;
        }
        let px = v.x * n.x;
        let py = v.y * n.y;
        n.z = -(px + py) / v.z;
        let inner = (v.x as f64 * n.x as f64 + py as f64) as f32;
        let fused = (v.z as f64 * n.z as f64 + inner as f64) as f32;
        let a = v.z * n.z + inner;
        let b = (v.z as f64 * n.z as f64 + (px + py) as f64) as f32;
        if (fused < 0.0) != (a < 0.0) || (fused < 0.0) != (b < 0.0) {
            return (v, n);
        }
    }
}

cases! { hand_match_c, 25;
    3001 R10_80271830 |ip, g, ov, x| { r4_pre(ip, g, x); ip.xBEC.top = r7_c(g); ip.xBEC.bottom = r7_c(g); ip.xBEC.right = r7_c(g); ip.xBEC.left = r7_c(g); let a = r10_a(g); it_80271830(ip, a); r10_h(ip) };
    3002 R10_8027649C |ip, g, ov, x| { r4_pre(ip, g, x); r10_coll(ip, g); ip.facing_dir = r7_c(g); let r = it_8027649c(ip).to_bits(); hmix(r, r10_h(ip)) };
    3003 R10_8027649C_REF |ip, g, ov, x| { r4_pre(ip, g, x); r10_coll(ip, g); ip.facing_dir = r7_c(g); let mut a = r7_vin(g); let mut b = r7_vin(g); let r = it_8027649c_ref(ip, &mut a, &mut b); hmix(hmix(hmix(r.to_bits(), r7_v(&a)), r7_v(&b)), r10_h(ip)) };
    3004 R10_80277040 |ip, g, ov, x| { r4_pre(ip, g, x); r10_coll(ip, g); x.attr.x50 = if (g.r() >> 16) % 4 != 0 { r7_c(g) } else { g.rm() / 16777216.0 }; ip.x88 = r7_vin(g); ip.xAC_unk = r7_vin(g); ip.xC2C = ((g.r() >> 16) % 3) as i32; ip.xC30 = ((g.r() >> 16) % 3) as i32; ip.xD58 = (g.r() >> 16) % 4; ip.xD5C = 0; x.common.xC0 = g.rm() / 64.0; x.common.xC4 = g.rm() / 64.0; x.common.x38_float = (g.r() >> 16) % 4; let c = x.common; let r = it_80277040(ip, &c) as u32; hmix(r, r10_h(ip)) };
    3005 R10_8027770C |ip, g, ov, x| { r4_pre(ip, g, x); r10_coll(ip, g); ip.x40_vel = r7_vin(g); x.attr.x58 = r7_c(g); let r = it_8027770c(ip) as u32; hmix(r, r10_h(ip)) };
    3006 R10_8027B798 |ip, g, ov, x| { r4_pre(ip, g, x); r10_coll(ip, g); let mut cd = Box::new(ftCommonData::zeroed()); cd.x100 = g.rm() / 64.0; cd.x144_radians = r10_a(g); cd.x148 = g.rm(); cd.x14C = g.rm(); cd.x150 = g.rm(); let s = (g.r() >> 16) % 4; ip.xCAC_angle = if s == 0 { 0x169 } else if s == 1 { ((g.r() >> 16) % 400) as i32 } else { 0x169 }; ip.xCC8_knockback = r7_c(g); let r = it_8027b798(ip, ov, &cd) as u32; hmix(r, r10_h(ip)) };
    3007 R10_8027BB1C |ip, g, ov, x| { let mut v = r7_vin(g); let a = r7_vin(g); it_8027bb1c(&mut v, &a); r7_v(&v) };
    3024 R10_8027770C_DOT_SIGN |ip, g, ov, x| { r4_pre(ip, g, x); r10_coll(ip, g); ip.x378_itemColl.env_flags = Collide_LeftWallMask; let (v, n) = r10_dot_pick(g); ip.x40_vel = v; ip.x378_itemColl.left_facing_wall.normal = n; x.attr.x58 = 1.0; let r = it_8027770c(ip) as u32; hmix(r, r10_h(ip)) };
    3025 R10_802E628C_UNIT_FACING |ip, g, ov, x| { r4_pre(ip, g, x); ip.facing_dir = if g.r() & 1 != 0 { 1.0 } else { -1.0 }; let mut a = Box::new(it_2E5A_Attrs::zeroed()); a.xC = g.rm(); a.x10 = g.rm(); a.x18 = g.rm(); a.x20 = g.rm(); x.art.x4_specialAttributes = &mut *a as *mut it_2E5A_Attrs as *mut _; let a8 = if (g.r() >> 16) % 4 != 0 { ((g.r() >> 16) % 7000) as f32 / 1000.0 } else { r10_a(g) }; let a9 = r7_c(g); let mut r = seed(g); it_802e628c(&mut r, ip, a8, a9); seed_h(r10_h(ip), &r) };
    3008 R10_802E628C |ip, g, ov, x| { r4_pre(ip, g, x); let mut a = Box::new(it_2E5A_Attrs::zeroed()); a.xC = g.rm(); a.x10 = g.rm(); a.x18 = g.rm(); a.x20 = g.rm(); x.art.x4_specialAttributes = &mut *a as *mut it_2E5A_Attrs as *mut _; let a8 = if (g.r() >> 16) % 4 != 0 { ((g.r() >> 16) % 7000) as f32 / 1000.0 } else { r10_a(g) }; let a9 = r7_c(g); let mut r = seed(g); it_802e628c(&mut r, ip, a8, a9); seed_h(r10_h(ip), &r) };
    3009 R10_802863BC |ip, g, ov, x| { r4_pre(ip, g, x); r10_coll(ip, g); sa::<itBoxAttributes>(x).bounce_angle_threshold = g.rm() / 32.0; ip.x7C = r7_vin(g); ip.x88 = r7_vin(g); ip.x94 = r7_vin(g); ip.xA0 = r7_vin(g); let r = it_802863bc(ip) as u32; hmix(r, r10_h(ip)) };
    3010 R10_FLIPPER_REPEL_INLINE |ip, g, ov, x| { r4_pre(ip, g, x); let k = if g.r() & 1 != 0 { It_Kind_Flipper as i32 } else { ((g.r() >> 16) % 40) as i32 }; let p = r7_vin(g); it_flipper_repel_inline(ip, k, &p, ov); r10_h(ip) };
    3011 R10_FOXBLASTER_SETSHOTANGLE |ip, g, ov, x| { r4_pre(ip, g, x); let a = r7_vin(g); let b = r7_vin(g); it_fox_blaster_set_shot_angle(ip, &a, &b); r10_h(ip) };
    3012 R10_80293DCC |ip, g, ov, x| { r4_pre(ip, g, x); let mut hd = Box::new(itHammerData::default()); hd.x0 = r9_i(g) as u32; hd.x4 = r9_i(g) as u32; hd.x8 = g.rm(); x.art.x4_specialAttributes = &mut *hd as *mut itHammerData as *mut _; let mut r = seed(g); let v = it_80293dcc(&mut r, ip) as u32; seed_h(v, &r) };
    3013 R10_802D61C8 |ip, g, ov, x| { r4_pre(ip, g, x); r10_coll(ip, g); ip.xDD4_itemVar.hinoarashi.x78 = r10_a(g); let c = ((g.r() >> 16) % 16) as i32; it_802d61c8(ip, c); r10_h(ip) };
    3014 R10_802D4494 |ip, g, ov, x| { r4_pre(ip, g, x); let ha = sa::<itHitodemanAttributes>(x); ha.x14 = g.rm(); ha.x18 = g.rm(); let mut r = seed(g); it_802d4494(&mut r, ip); seed_h(r10_h(ip), &r) };
    3015 R10_802D4510 |ip, g, ov, x| { r4_pre(ip, g, x); let ha = sa::<itHitodemanAttributes>(x); ha.x40 = r9_i(g); ha.x44 = r9_i(g); let mut r = seed(g); it_802d4510(&mut r, ip); seed_h(r10_h(ip), &r) };
    3016 R10_KOOPAFLAME_UNKMOTION0_PHYS |ip, g, ov, x| { r4_pre(ip, g, x); it_koopa_flame_unk_motion0_phys(ip); r10_h(ip) };
    3017 R10_LGUNBEAM_UNKMOTION0_PHYS |ip, g, ov, x| { r4_pre(ip, g, x); it_lgunbeam_unk_motion0_phys(ip); r10_h(ip) };
    3018 R10_802A2320 |ip, g, ov, x| { r4_pre(ip, g, x); ip.x40_vel = r7_vin(g); ip.xC58 = r7_vin(g); ip.facing_dir = if g.r() & 1 != 0 { 1.0 } else { r7_c(g) }; let r = it_802a2320(ip) as u32; hmix(r, r10_h(ip)) };
    3019 R10_SHADOWBALL_SHIELDBOUNCED |ip, g, ov, x| { r4_pre(ip, g, x); ip.x40_vel = r7_vin(g); ip.xC58 = r7_vin(g); let r = it_2725_logic101_shield_bounced(ip) as u32; hmix(r, r10_h(ip)) };
    3020 R10_802D6740 |ip, g, ov, x| { r4_pre(ip, g, x); it_802d6740(ip); r10_h(ip) };
    3021 R10_PIKACHUTJOLTGROUND_UNKMOTION0_PHYS |ip, g, ov, x| { r4_pre(ip, g, x); it_pikachutjoltground_unk_motion0_phys(ip); r10_h(ip) };
    3022 R10_SAMUSCHARGESHOT_UNKMOTION8_PHYS |ip, g, ov, x| { r4_pre(ip, g, x); it_samuschargeshot_unk_motion8_phys(ip); r10_h(ip) };
    3023 R10_802EC870 |ip, g, ov, x| { r4_pre(ip, g, x); sa::<itTincleAttributes>(x).x3C = r9_i(g); let a1 = if (g.r() >> 16) % 4 != 0 { 0 } else { 1 }; let mut r = seed(g); it_802ec870(&mut r, ip, a1); seed_h(r10_h(ip), &r) };
}
