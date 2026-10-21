//! Round 2 (night 2): src/itlib.rs against tests/c_oracle/round2.c (contraction on + gwtool
//! lowering, as the shipping build). Both sides fill the same Item fields from the same LCG
//! (round2/fields.rs / c_oracle/round2_fields.h, generated from one list by
//! c_oracle/gen_round2_fields.py), run the function, and hash every filled field + the return
//! value + an output Vec3. One hash per case, 48 cases per table, 104 tables for 103 functions.
#![allow(non_upper_case_globals)]

#[path = "c_oracle/expected.rs"]
#[allow(dead_code)]
mod expected;
#[path = "round2/fields.rs"]
mod fields;

use expected as e;
use melee_items::itlib::*;
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

fn hurt_in(g: &mut Gen) -> HurtCapsule {
    let mut hc = HurtCapsule::zeroed();
    hc.a_offset = Vec { x: g.rf(), y: g.rf(), z: g.rf() };
    hc.b_offset = Vec { x: g.rf(), y: g.rf(), z: g.rf() };
    hc.scale = g.rf();
    hc
}
fn hurt_h(hc: &HurtCapsule) -> u32 {
    [hc.a_offset.x, hc.a_offset.y, hc.a_offset.z, hc.b_offset.x, hc.b_offset.y, hc.b_offset.z,
        hc.scale]
        .iter()
        .fold(0, |h, v| hmix(h, v.to_bits()))
}

macro_rules! cases {
    ($($id:literal $want:ident |$ip:ident, $g:ident, $ov:ident, $x:ident| $body:expr;)*) => {
        #[test]
        fn itlib_matches_c() {
            let mut n = 0;
            $(
                {
                    #[allow(unused_variables, unused_unsafe)]
                    fn case($ip: &mut Item, $g: &mut Gen, $ov: &mut Vec, $x: &mut Ext) -> u32 {
                        unsafe { $body }
                    }
                    run($id, stringify!($want), &e::$want, case);
                    n += 1;
                }
            )*
            assert_eq!(n, 104);
        }
    };
}

cases! {
    1 R2_8026B294 |ip, g, ov, x| { it_8026b294(ip, ov); 0 };
    2 R2_ISHEAVY |ip, g, ov, x| it_is_heavy(ip) as u32;
    3 R2_8026B2D8 |ip, g, ov, x| it_8026b2d8(ip) as u32;
    4 R2_GETKIND |ip, g, ov, x| it_get_kind(ip) as u32;
    5 R2_8026B30C |ip, g, ov, x| it_8026b30c(ip) as u32;
    6 R2_GETHOLDKIND |ip, g, ov, x| it_get_hold_kind(ip) as u32;
    7 R2_DAMAGEMUL |ip, g, ov, x| it_get_damage_multiplier(ip).to_bits();
    8 R2_8026B344 |ip, g, ov, x| {
        ip.facing_dir = g.rm();
        ip.xBCC_unk.x = g.rm();
        ip.pos.x = g.rm();
        it_8026b344(ip, ov);
        0
    };
    9 R2_GRABX |ip, g, ov, x| it_get_grab_range_x(ip).to_bits();
    10 R2_GRABY |ip, g, ov, x| it_get_grab_range_y(ip).to_bits();
    11 R2_8026B390 |ip, g, ov, x| { it_8026b390(ip); 0 };
    12 R2_8026B3A8 |ip, g, ov, x| { it_8026b3a8(ip); 0 };
    13 R2_8026B424 |ip, g, ov, x| {
        let d = if g.r() & 1 != 0 { (g.r() % 1000) as i32 - 100 } else { g.r() as i32 };
        it_8026b424(d, &x.common).to_bits()
    };
    14 R2_8026B47C |ip, g, ov, x| it_8026b47c(ip) as u32;
    15 R2_8026B4F0 |ip, g, ov, x| it_8026b4f0(ip) as u32;
    16 R2_8026B54C |ip, g, ov, x| it_8026b54c(ip).to_bits();
    17 R2_8026B560 |ip, g, ov, x| it_8026b560(ip).to_bits();
    18 R2_8026B574 |ip, g, ov, x| it_8026b574(ip).to_bits();
    19 R2_8026B588 |ip, g, ov, x| it_8026b588(&x.common) as u32;
    20 R2_8026B594 |ip, g, ov, x| it_8026b594(ip) as u32;
    21 R2_8026B6C8 |ip, g, ov, x| it_8026b6c8(ip) as u32;
    22 R2_8026B718 |ip, g, ov, x| { let v = g.rf(); it_8026b718(ip, v); 0 };
    23 R2_8026B724 |ip, g, ov, x| { it_8026b724(ip); 0 };
    24 R2_8026B73C |ip, g, ov, x| { it_8026b73c(ip); 0 };
    25 R2_8026B774 |ip, g, ov, x| { let a = (g.r() % 32) as u8; it_8026b774(ip, a) as u32 };
    26 R2_MOTIONID |ip, g, ov, x| it_get_motion_id(ip) as u32;
    27 R2_TEAMID |ip, g, ov, x| it_get_team_id(ip) as u32;
    28 R2_8026B7BC |ip, g, ov, x| it_8026b7bc(ip) as u32;
    29 R2_8026B7CC |ip, g, ov, x| it_8026b7cc(ip) as u32;
    30 R2_8026B7E8 |ip, g, ov, x| it_8026b7e8(ip) as u32;
    31 R2_8026B894 |ip, g, ov, x| { let p = g.ptr(); it_8026b894(ip, p) as u32 };
    32 R2_8026B924 |ip, g, ov, x| it_8026b924(ip) as u32;
    33 R2_8026B960 |ip, g, ov, x| it_8026b960(ip).to_bits();
    34 R2_8026BB88 |ip, g, ov, x| { it_8026bb88(ip, ov); 0 };
    35 R2_8026BBCC |ip, g, ov, x| { it_8026bbcc(ip, ov); 0 };
    36 R2_8026BC68 |ip, g, ov, x| it_8026bc68(ip) as u32;
    37 R2_ATTACKID |ip, g, ov, x| it_get_attack_id(ip) as u32;
    38 R2_8026BCF4 |ip, g, ov, x| { it_8026bcf4(ip); 0 };
    39 R2_8026BD0C |ip, g, ov, x| { it_8026bd0c(ip); 0 };
    40 R2_8026BD24 |ip, g, ov, x| { it_8026bd24(ip); 0 };
    41 R2_8026BD3C |ip, g, ov, x| { it_8026bd3c(ip); 0 };
    42 R2_8026BD54 |ip, g, ov, x| { it_8026bd54(ip); 0 };
    43 R2_8026BD6C |ip, g, ov, x| { it_8026bd6c(ip); 0 };
    44 R2_8026BD84 |ip, g, ov, x| { it_8026bd84(ip); 0 };
    45 R2_8026BD9C |ip, g, ov, x| { it_8026bd9c(ip); 0 };
    46 R2_8026BDCC |ip, g, ov, x| { it_8026bdcc(ip); 0 };
    47 R2_8026BE28 |ip, g, ov, x| { it_8026be28(ip); 0 };
    48 R2_8026C1E8 |ip, g, ov, x| it_8026c1e8(ip) as u32;
    49 R2_8026C334 |ip, g, ov, x| { it_8026c334(ip, ov); 0 };
    50 R2_80273030 |ip, g, ov, x| it_80273030(ip) as u32;
    51 R2_80273130 |ip, g, ov, x| it_80273130(ip) as u32;
    52 R2_8027346C |ip, g, ov, x| { it_8027346c(ip); 0 };
    53 R2_80273484 |ip, g, ov, x| { it_80273484(ip); 0 };
    54 R2_8027349C |ip, g, ov, x| { it_8027349c(ip); 0 };
    55 R2_802734B4 |ip, g, ov, x| { it_802734b4(ip); 0 };
    56 R2_80274C60 |ip, g, ov, x| { it_80274c60(ip); 0 };
    57 R2_80274C78 |ip, g, ov, x| it_80274c78(ip) as u32;
    58 R2_80274C88 |ip, g, ov, x| { it_80274c88(ip); 0 };
    59 R2_80274D6C |ip, g, ov, x| { it_80274d6c(ip); 0 };
    60 R2_80274DAC |ip, g, ov, x| { it_80274dac(ip); 0 };
    61 R2_80274DFC |ip, g, ov, x| { it_80274dfc(ip); 0 };
    62 R2_80274E44 |ip, g, ov, x| { it_80274e44(ip); 0 };
    63 R2_80274ECC |ip, g, ov, x| {
        let a = if g.r() % 3 == 0 { g.r() as i32 } else { (g.r() & 1) as i32 };
        it_80274ecc(ip, a);
        0
    };
    64 R2_80274EF8 |ip, g, ov, x| { it_80274ef8(ip); 0 };
    65 R2_80274F10 |ip, g, ov, x| { it_80274f10(ip); 0 };
    66 R2_802750E8 |ip, g, ov, x| { let m = g.r() as i32; it_802750e8(ip, m) as u32 };
    67 R2_802751D8 |ip, g, ov, x| it_802751d8(ip) as u32;
    68 R2_80275210 |ip, g, ov, x| { it_80275210(ip); 0 };
    69 R2_80275228 |ip, g, ov, x| { it_80275228(ip); 0 };
    70 R2_80275240 |ip, g, ov, x| { it_80275240(ip); 0 };
    71 R2_80275258 |ip, g, ov, x| { it_80275258(ip); 0 };
    72 R2_80275270 |ip, g, ov, x| { it_80275270(ip); 0 };
    73 R2_802753BC |ip, g, ov, x| { let a = g.r() as i16; it_802753bc(ip, a); 0 };
    74 R2_802753DC |ip, g, ov, x| { it_802753dc(ip); 0 };
    75 R2_80275414 |ip, g, ov, x| { it_80275414(ip); 0 };
    76 R2_8027542C |ip, g, ov, x| { it_8027542c(ip); 0 };
    77 R2_80275444 |ip, g, ov, x| { it_80275444(ip); 0 };
    78 R2_80275474 |ip, g, ov, x| { it_80275474(ip); 0 };
    79 R2_802754A4 |ip, g, ov, x| { it_802754a4(ip); 0 };
    80 R2_802754BC |ip, g, ov, x| { it_802754bc(ip); 0 };
    81 R2_802754D4 |ip, g, ov, x| { it_802754d4(ip); 0 };
    82 R2_80275504 |ip, g, ov, x| { it_80275504(ip); 0 };
    83 R2_80275534 |ip, g, ov, x| { let s = g.rf(); it_80275534(ip, s); 0 };
    84 R2_80275594 |ip, g, ov, x| {
        let idx = (g.r() % 4) as i32;
        let m = g.rf();
        it_80275594(ip, idx, m);
        0
    };
    85 R2_802755C0 |ip, g, ov, x| { let s = g.rf(); it_802755c0(ip, s); 0 };
    86 R2_802756D0 |ip, g, ov, x| { it_802756d0(ip); 0 };
    87 R2_8027570C |ip, g, ov, x| { let i = (g.r() % 4) as i32; it_8027570c(ip, i); 0 };
    88 R2_8027572C |ip, g, ov, x| { let i = (g.r() % 4) as i32; it_8027572c(ip, i); 0 };
    89 R2_80275788 |ip, g, ov, x| { it_80275788(ip); 0 };
    90 R2_80275820 |ip, g, ov, x| {
        let a = Vec { x: g.rf(), y: g.rf(), z: g.rf() };
        let b = Vec { x: g.rf(), y: g.rf(), z: g.rf() };
        let idx = (g.r() % 4) as i32;
        it_80275820(ip, &a, &b, idx);
        0
    };
    91 R2_80275870 |ip, g, ov, x| it_80275870(ip) as u32;
    92 R2_802758D4 |ip, g, ov, x| it_802758d4(ip).to_bits();
    104 R2_802758D4_ZEROS |ip, g, ov, x| {
        for k in 0..4 {
            ip.x5D4_hitboxes[k].hit.damage = f32::from_bits(DMG[(g.r() % 4) as usize]);
        }
        it_802758d4(ip).to_bits()
    };
    93 R2_802762D8 |ip, g, ov, x| it_802762d8(ip) as u32;
    94 R2_80276308 |ip, g, ov, x| it_80276308(ip) as u32;
    95 R2_80276348 |ip, g, ov, x| it_80276348(ip, ov) as u32;
    96 R2_802763B8 |ip, g, ov, x| it_802763b8(ip) as u32;
    97 R2_80276408 |ip, g, ov, x| { it_80276408(&ip.x378_itemColl, ov); 0 };
    98 R2_802714C0 |ip, g, ov, x| { it_802714c0(ip); 0 };
    99 R2_80271508 |ip, g, ov, x| { let s = g.r() % 3; it_80271508(ip, s as _); 0 };
    100 R2_80271534 |ip, g, ov, x| {
        let idx = (g.r() % 2) as i32;
        let hc = hurt_in(g);
        it_80271534(ip, idx, &hc);
        0
    };
    101 R2_80271590 |ip, g, ov, x| {
        let idx = (g.r() % 2) as i32;
        let mut hc = hurt_in(g);
        it_80271590(ip, idx, &mut hc);
        hurt_h(&hc)
    };
    102 R2_80272280 |ip, g, ov, x| { it_80272280(ip); 0 };
    103 R2_80272298 |ip, g, ov, x| { it_80272298(ip); 0 };
}
