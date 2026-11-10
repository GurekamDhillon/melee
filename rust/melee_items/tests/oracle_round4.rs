//! Round 4 (night 3): src/itkinds.rs against tests/c_oracle/round4.c (contraction on + gwtool
//! lowering, as the shipping build). Same harness as rounds 2-3 (the infrastructure below is a
//! copy of oracle_round3.rs's): both sides fill the round-2 Item field list from the same LCG,
//! then `r4_pre` draws the extra fields the item-kind callbacks read (ground/air, command vars,
//! anim frame, fall speeds, a 256-word special-attributes block, every item-var word), run the
//! function, and hash the round-2 fields + `ret` + the output Vec3 + `r4_h` (those extra fields
//! and every item-var word). 48 cases per table, one table per function.
//!
//! Bitfields inside the item-var union are stored LSB-first by the x86 C compiler but in GC
//! (MSB-first) numbering by melee_types, so for the functions that read one (`r4_rev` offsets)
//! the Rust side bit-reverses that byte after the fill and again before hashing.
#![allow(non_upper_case_globals, dead_code)]

#[path = "c_oracle/expected.rs"]
#[allow(dead_code)]
mod expected;
#[path = "round2/fields.rs"]
mod fields;

use expected as e;
use melee_items::itkinds::*;
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

#[test]
fn itemvar_size_matches_c() {
    assert_eq!(std::mem::size_of::<Item_ItemVars>(), e::R4_ITEMVAR_BYTES);
}

macro_rules! cases {
    ($($id:literal $want:ident |$ip:ident, $g:ident, $ov:ident, $x:ident| $body:expr;)*) => {
        #[test]
        fn itkinds_match_c() {
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
            assert_eq!(n, 249);
        }
    };
}

cases! {
    301 R4_ITBOMBHEI_UNKMOTION1_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_bombhei_unk_motion1_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    302 R4_ITCLIMBERSICE_UNKMOTION1_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_climbersice_unk_motion1_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    303 R4_ITDOSEI_UNKMOTION3_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_dosei_unk_motion3_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    304 R4_ITDOSEI_UNKMOTION8_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_dosei_unk_motion8_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    305 R4_ITDOSEI_UNKMOTION11_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_dosei_unk_motion11_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    306 R4_ITEVYOSHIEGG_UNKMOTION1_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_evyoshiegg_unk_motion1_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    307 R4_ITFFLOWER_UNKMOTION1_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_fflower_unk_motion1_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    308 R4_ITFFLOWER_UNKMOTION6_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_fflower_unk_motion6_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    309 R4_ITFFLOWERFLAME_UNKMOTION3_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_fflowerflame_unk_motion3_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    310 R4_ITFLIPPER_FALLING_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_flipper_falling_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    311 R4_ITFREEZE_UNKMOTION1_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_freeze_unk_motion1_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    312 R4_ITFUSHIGIBANA_UNKMOTION2_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_fushigibana_unk_motion2_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    313 R4_ITGSHELL_UNKMOTION1_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_gshell_unk_motion1_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    314 R4_ITGSHELL_UNKMOTION3_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_gshell_unk_motion3_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    315 R4_ITGSHELL_UNKMOTION4_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_gshell_unk_motion4_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    316 R4_ITGSHELL_UNKMOTION8_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_gshell_unk_motion8_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    317 R4_ITHARISEN_UNKMOTION1_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_harisen_unk_motion1_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    318 R4_ITHEART_UNKMOTION3_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_heart_unk_motion3_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    319 R4_ITHINOARASHI_UNKMOTION2_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_hinoarashi_unk_motion2_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    320 R4_ITKUSUDAMA_UNKMOTION2_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_kusudama_unk_motion2_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    321 R4_ITLEADEAD_UNKMOTION10_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_leadead_unk_motion10_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    322 R4_ITLEADEAD_UNKMOTION14_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_leadead_unk_motion14_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    323 R4_ITLIKELIKE_UNKMOTION19_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_likelike_unk_motion19_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    324 R4_ITLIPSTICK_UNKMOTION1_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_lipstick_unk_motion1_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    325 R4_ITLUCKY_UNKMOTION5_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_lucky_unk_motion5_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    326 R4_ITLUCKY_UNKMOTION6_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_lucky_unk_motion6_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    327 R4_ITLUCKY_UNKMOTION3_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_lucky_unk_motion3_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    328 R4_ITMARIL_UNKMOTION2_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_maril_unk_motion2_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    329 R4_ITMARUMINE_UNKMOTION2_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_marumine_unk_motion2_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    330 R4_ITMARUMINE_UNKMOTION4_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_marumine_unk_motion4_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    331 R4_ITMSBOMB_UNKMOTION1_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_msbomb_unk_motion1_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    332 R4_ITOCTAROCK_UNKMOTION2_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_octarock_unk_motion2_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    333 R4_ITOCTAROCK_UNKMOTION6_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_octarock_unk_motion6_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    334 R4_ITOLDKURI_UNKMOTION3_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_oldkuri_unk_motion3_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    335 R4_ITOLDKURI_UNKMOTION4_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_oldkuri_unk_motion4_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    336 R4_ITOLDKURI_UNKMOTION6_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_oldkuri_unk_motion6_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    337 R4_ITOLDKURI_UNKMOTION8_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_oldkuri_unk_motion8_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    338 R4_ITOLDOTTOSEA_UNKMOTION4_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_oldottosea_unk_motion4_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    339 R4_ITPATAPATA_UNKMOTION6_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_patapata_unk_motion6_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    340 R4_ITRSHELL_UNKMOTION1_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_rshell_unk_motion1_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    341 R4_ITRSHELL_UNKMOTION3_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_rshell_unk_motion3_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    342 R4_ITRSHELL_UNKMOTION4_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_rshell_unk_motion4_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    343 R4_ITRSHELL_UNKMOTION6_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_rshell_unk_motion6_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    344 R4_ITSAMUSBOMB_UNKMOTION1_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_samusbomb_unk_motion1_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    345 R4_ITSCBALL_UNKMOTION1_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_scball_unk_motion1_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    346 R4_ITSPYCLOAK_UNKMOTION1_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_spycloak_unk_motion1_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    347 R4_ITSPYCLOAK_UNKMOTION3_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_spycloak_unk_motion3_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    348 R4_ITSTAR_UNKMOTION0_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_star_unk_motion0_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    349 R4_ITTARU_UNKMOTION1_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_taru_unk_motion1_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    350 R4_ITTARUCANN_UNKMOTION1_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_tarucann_unk_motion1_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    351 R4_ITTINCLE_UNKMOTION8_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_tincle_unk_motion8_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    352 R4_ITTOMATO_UNKMOTION4_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_tomato_unk_motion4_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    353 R4_ITTOSAKINTO_UNKMOTION3_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_tosakinto_unk_motion3_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    354 R4_ITWHISPYAPPLE_UNKMOTION5_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_whispyapple_unk_motion5_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    355 R4_ITOLDOTTOSEA_UNKMOTION9_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_oldottosea_unk_motion9_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    356 R4_ITWHITEBEA_UNKMOTION3_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_whitebea_unk_motion3_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    357 R4_ITWHITEBEA_UNKMOTION6_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_whitebea_unk_motion6_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    358 R4_ITWHITEBEA_UNKMOTION9_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_whitebea_unk_motion9_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    359 R4_ITWSTAR_UNKMOTION1_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_wstar_unk_motion1_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    360 R4_ITYOSHIEGGTHROW_UNKMOTION1_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_yoshieggthrow_unk_motion1_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    361 R4_ITZRSHELL_UNKMOTION1_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_zrshell_unk_motion1_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    362 R4_ITZRSHELL_UNKMOTION3_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_zrshell_unk_motion3_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    363 R4_ITZRSHELL_UNKMOTION4_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_zrshell_unk_motion4_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    364 R4_ITZRSHELL_UNKMOTION8_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_zrshell_unk_motion8_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    365 R4_ITBAT_LOGIC11_CLANKED |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_bat_logic11_clanked(ip, &x.common) as u32 }; hmix(ret, r4_h(ip)) };
    366 R4_ITBAT_LOGIC11_HITSHIELD |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_bat_logic11_hit_shield(ip, &x.common) as u32 }; hmix(ret, r4_h(ip)) };
    367 R4_ITDOSEI_LOGIC7_CLANKED |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_dosei_logic7_clanked(ip, &x.common) as u32 }; hmix(ret, r4_h(ip)) };
    368 R4_ITDOSEI_LOGIC7_HITSHIELD |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_dosei_logic7_hit_shield(ip, &x.common) as u32 }; hmix(ret, r4_h(ip)) };
    369 R4_ITFFLOWER_LOGIC25_DMGDEALT |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_fflower_logic25_dmg_dealt(ip, &x.common) as u32 }; hmix(ret, r4_h(ip)) };
    370 R4_ITFFLOWER_LOGIC25_CLANKED |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_fflower_logic25_clanked(ip, &x.common) as u32 }; hmix(ret, r4_h(ip)) };
    371 R4_ITFFLOWER_LOGIC25_HITSHIELD |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_fflower_logic25_hit_shield(ip, &x.common) as u32 }; hmix(ret, r4_h(ip)) };
    372 R4_ITLGUN_LOGIC16_DMGDEALT |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_lgun_logic16_dmg_dealt(ip, &x.common) as u32 }; hmix(ret, r4_h(ip)) };
    373 R4_ITLGUN_LOGIC16_CLANKED |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_lgun_logic16_clanked(ip, &x.common) as u32 }; hmix(ret, r4_h(ip)) };
    374 R4_ITLGUN_LOGIC16_HITSHIELD |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_lgun_logic16_hit_shield(ip, &x.common) as u32 }; hmix(ret, r4_h(ip)) };
    375 R4_ITPARASOL_LOGIC13_DMGDEALT |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_parasol_logic13_dmg_dealt(ip, &x.common) as u32 }; hmix(ret, r4_h(ip)) };
    376 R4_ITPARASOL_LOGIC13_CLANKED |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_parasol_logic13_clanked(ip, &x.common) as u32 }; hmix(ret, r4_h(ip)) };
    377 R4_ITPARASOL_LOGIC13_HITSHIELD |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_parasol_logic13_hit_shield(ip, &x.common) as u32 }; hmix(ret, r4_h(ip)) };
    378 R4_ITPEACHTURNIP_LOGIC56_CLANKED |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_peach_turnip_logic56_clanked(ip, &x.common) as u32 }; hmix(ret, r4_h(ip)) };
    379 R4_ITPEACHTURNIP_LOGIC56_HITSHIELD |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_peach_turnip_logic56_hit_shield(ip, &x.common) as u32 }; hmix(ret, r4_h(ip)) };
    380 R4_ITRSHELL_LOGIC15_CLANKED |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_rshell_logic15_clanked(ip, &x.common) as u32 }; hmix(ret, r4_h(ip)) };
    381 R4_ITSCBALL_LOGIC30_HITSHIELD |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_scball_logic30_hit_shield(ip, &x.common) as u32 }; hmix(ret, r4_h(ip)) };
    382 R4_ITSSCOPE_LOGIC21_DMGDEALT |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_sscope_logic21_dmg_dealt(ip, &x.common) as u32 }; hmix(ret, r4_h(ip)) };
    383 R4_ITSSCOPE_LOGIC21_CLANKED |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_sscope_logic21_clanked(ip, &x.common) as u32 }; hmix(ret, r4_h(ip)) };
    384 R4_ITSSCOPE_LOGIC21_HITSHIELD |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_sscope_logic21_hit_shield(ip, &x.common) as u32 }; hmix(ret, r4_h(ip)) };
    385 R4_ITSWORD_LOGIC12_CLANKED |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_sword_logic12_clanked(ip, &x.common) as u32 }; hmix(ret, r4_h(ip)) };
    386 R4_ITSWORD_LOGIC12_HITSHIELD |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_sword_logic12_hit_shield(ip, &x.common) as u32 }; hmix(ret, r4_h(ip)) };
    387 R4_ITKYASARIN_UNKMOTION9_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_kyasarin_unk_motion9_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    388 R4_ITLEADEAD_UNKMOTION16_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_leadead_unk_motion16_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    389 R4_ITLEADEAD_UNKMOTION17_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_leadead_unk_motion17_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    390 R4_ITLIKELIKE_UNKMOTION21_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_likelike_unk_motion21_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    391 R4_ITOCTAROCK_UNKMOTION8_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_octarock_unk_motion8_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    392 R4_ITOCTAROCKSTONE_UNKMOTION1_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_octarockstone_unk_motion1_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    393 R4_ITOLDKURI_UNKMOTION11_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_oldkuri_unk_motion11_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    394 R4_ITOLDOTTOSEA_UNKMOTION11_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_oldottosea_unk_motion11_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    395 R4_ITWHITEBEA_UNKMOTION11_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_whitebea_unk_motion11_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    396 R4_ITZRSHELL_UNKMOTION10_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_zrshell_unk_motion10_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    397 R4_ITENTEI_UNKMOTION0_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_entei_unk_motion0_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    398 R4_IT_802CA618 |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_802ca618(ip); 0 }; hmix(ret, r4_h(ip)) };
    399 R4_ITKAMEX_UNKMOTION2_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_kamex_unk_motion2_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    400 R4_ITPIPPI_UNKMOTION5_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_pippi_unk_motion5_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    401 R4_ITRAIKOU_UNKMOTION0_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_raikou_unk_motion0_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    402 R4_ITSUIKUN_UNKMOTION0_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_suikun_unk_motion0_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    403 R4_ITLEADEAD_UNKMOTION12_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_leadead_unk_motion12_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    404 R4_ITNOKONOKO_UNKMOTION5_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_nokonoko_unk_motion5_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    405 R4_ITOCTAROCK_UNKMOTION3_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_octarock_unk_motion3_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    406 R4_ITOLDKURI_UNKMOTION9_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_oldkuri_unk_motion9_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    407 R4_ITOLDOTTOSEA_UNKMOTION5_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_oldottosea_unk_motion5_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    408 R4_ITWHITEBEA_UNKMOTION4_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_whitebea_unk_motion4_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    409 R4_IT_2ADA_UNKMOTION0_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_2ada_unk_motion0_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    410 R4_IT_2E5A_UNKMOTION1_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_2e5a_unk_motion1_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    411 R4_ITCOIN_UNKMOTION4_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_coin_unk_motion4_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    412 R4_ITEGG_UNKMOTION1_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_egg_unk_motion1_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    413 R4_ITLGUN_UNKMOTION1_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_lgun_unk_motion1_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    414 R4_ITFOODS_UNKMOTION3_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_foods_unk_motion3_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    415 R4_ITHAMMER_UNKMOTION1_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_hammer_unk_motion1_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    416 R4_ITHAMMER_UNKMOTION4_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_hammer_unk_motion4_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    417 R4_ITHAMMERHEAD_LOGIC40_REFLECTED |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_hammer_head_logic40_reflected(ip) as u32 }; hmix(ret, r4_h(ip)) };
    418 R4_ITOCTAROCKSTONE_LOGIC4_REFLECTED |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_octarockstone_logic4_reflected(ip) as u32 }; hmix(ret, r4_h(ip)) };
    419 R4_ITPEACHTURNIP_LOGIC56_REFLECTED |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_peach_turnip_logic56_reflected(ip) as u32 }; hmix(ret, r4_h(ip)) };
    420 R4_ITSTARROD_LOGIC22_DMGDEALT |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_star_rod_logic22_dmg_dealt(ip, &x.common) as u32 }; hmix(ret, r4_h(ip)) };
    421 R4_ITSTARROD_LOGIC22_CLANKED |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_star_rod_logic22_clanked(ip, &x.common) as u32 }; hmix(ret, r4_h(ip)) };
    422 R4_ITSTARROD_LOGIC22_HITSHIELD |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_star_rod_logic22_hit_shield(ip, &x.common) as u32 }; hmix(ret, r4_h(ip)) };
    423 R4_ITDKINOKO_UNKMOTION0_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_dkinoko_unk_motion0_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    424 R4_ITKINOKO_UNKMOTION0_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_kinoko_unk_motion0_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    425 R4_ITDKINOKO_UNKMOTION1_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_dkinoko_unk_motion1_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    426 R4_ITKINOKO_UNKMOTION1_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_kinoko_unk_motion1_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    427 R4_ITDOSEI_UNKMOTION6_ANIM |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_dosei_unk_motion6_anim(ip) as u32 }; hmix(ret, r4_h(ip)) };
    428 R4_ITDOSEI_UNKMOTION8_ANIM |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_dosei_unk_motion8_anim(ip) as u32 }; hmix(ret, r4_h(ip)) };
    429 R4_IT_802CAE60 |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_802cae60(ip) as u32 }; hmix(ret, r4_h(ip)) };
    430 R4_IT_802CF120 |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_802cf120(ip) as u32 }; hmix(ret, r4_h(ip)) };
    431 R4_ITLINKBOMB_UNKMOTION1_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_linkbomb_unk_motion1_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    432 R4_ITPEACHTURNIP_UNKMOTION1_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_peachturnip_unk_motion1_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    433 R4_IT_802D5884 |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_802d5884(ip) as u32 }; hmix(ret, r4_h(ip)) };
    434 R4_ITMARIL_UNKMOTION2_ANIM_INLINE |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_maril_unk_motion2_anim_inline(ip) as u32 }; hmix(ret, r4_h(ip)) };
    435 R4_ITLUIGIFIREBALL_UNKMOTION0_ANIM |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_luigifireball_unk_motion0_anim(ip) as u32 }; hmix(ret, r4_h(ip)) };
    436 R4_ITMARIOFIREBALL_UNKMOTION0_ANIM |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_mariofireball_unk_motion0_anim(ip) as u32 }; hmix(ret, r4_h(ip)) };
    437 R4_ITMBALL_MOTION1_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_mball_motion1_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    438 R4_ITMBALL_MOTION6_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_mball_motion6_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    439 R4_ITMETALB_UNKMOTION1_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_metalb_unk_motion1_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    440 R4_ITMETALB_UNKMOTION3_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_metalb_unk_motion3_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    441 R4_ITNESSPKFIRE_UNKMOTION0_ANIM |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_nesspkfire_unk_motion0_anim(ip) as u32 }; hmix(ret, r4_h(ip)) };
    442 R4_ITPIKACHUTHUNDER_UNKMOTION1_ANIM |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_pikachuthunder_unk_motion1_anim(ip) as u32 }; hmix(ret, r4_h(ip)) };
    443 R4_IT_802AA7E4 |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { g.pidx(it_802aa7e4(ip)) }; hmix(ret, r4_h(ip)) };
    444 R4_ITZELDADINFIRE_GETOWNER |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { g.pidx(it_zelda_din_fire_get_owner(ip)) }; hmix(ret, r4_h(ip)) };
    445 R4_ITNOKONOKO_UNKMOTION4_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_nokonoko_unk_motion4_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    446 R4_ITNOKONOKO_UNKMOTION10_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_nokonoko_unk_motion10_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    447 R4_ITPIPPI_UNKMOTION1_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_pippi_unk_motion1_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    448 R4_ITTOGEPY_UNKMOTION1_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_togepy_unk_motion1_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    449 R4_ITRABBITC_UNKMOTION1_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_rabbitc_unk_motion1_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    450 R4_ITRABBITC_UNKMOTION3_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_rabbitc_unk_motion3_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    451 R4_ITRSHELL_UNKMOTION3_ANIM |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_rshell_unk_motion3_anim(ip) as u32 }; hmix(ret, r4_h(ip)) };
    452 R4_ITRSHELL_UNKMOTION4_ANIM |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_rshell_unk_motion4_anim(ip) as u32 }; hmix(ret, r4_h(ip)) };
    453 R4_ITSEAKNEEDLETHROWN_UNKMOTION1_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_seakneedlethrown_unk_motion1_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    454 R4_ITSEAKNEEDLETHROWN_UNKMOTION4_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_seakneedlethrown_unk_motion4_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    455 R4_ITCAPSULE_UNKMOTION4_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_capsule_unk_motion4_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    456 R4_ITFFLOWERFLAME_UNKMOTION1_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_fflowerflame_unk_motion1_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    457 R4_ITFREEZE_UNKMOTION3_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_freeze_unk_motion3_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    458 R4_ITSSCOPE_UNKMOTION1_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_sscope_unk_motion1_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    459 R4_ITOCTAROCKSTONE_UNKMOTION0_ANIM |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_octarockstone_unk_motion0_anim(ip) as u32 }; hmix(ret, r4_h(ip)) };
    460 R4_ITLIZARDON_UNKMOTION1_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_lizardon_unk_motion1_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    461 R4_ITMARUMINE_UNKMOTION5_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_marumine_unk_motion5_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    462 R4_ITKABIGON_UNKMOTION0_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_kabigon_unk_motion0_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    463 R4_ITNESSPKFIREPILLAR_UNKMOTION0_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_nesspkfirepillar_unk_motion0_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    464 R4_ITBAT_LOGIC11_DMGDEALT |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_bat_logic11_dmg_dealt(ip, &x.common) as u32 }; hmix(ret, r4_h(ip)) };
    465 R4_ITHARISEN_LOGIC24_DMGDEALT |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_harisen_logic24_dmg_dealt(ip, &x.common) as u32 }; hmix(ret, r4_h(ip)) };
    466 R4_ITLIPSTICK_LOGIC23_DMGDEALT |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_lipstick_logic23_dmg_dealt(ip, &x.common) as u32 }; hmix(ret, r4_h(ip)) };
    467 R4_ITLINKBOMB_LOGIC16_HITSHIELD |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_link_bomb_logic16_hit_shield(ip, &x.common) as u32 }; hmix(ret, r4_h(ip)) };
    468 R4_ITNOKONOKO_UNKMOTION7_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_nokonoko_unk_motion7_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    469 R4_ITLIKELIKE_UNKMOTION3_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_likelike_unk_motion3_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    470 R4_ITMEWTWODISABLE_LOGIC67_REFLECTED |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_mewtwo_disable_logic67_reflected(ip) as u32 }; hmix(ret, r4_h(ip)) };
    471 R4_ITDRMARIOPILL_REFLECTED |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_dr_mario_pill_reflected(ip) as u32 }; hmix(ret, r4_h(ip)) };
    472 R4_ITMBALL_REFLECTED |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_mball_reflected(ip) as u32 }; hmix(ret, r4_h(ip)) };
    473 R4_ITGSHELL_LOGIC14_REFLECTED |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_gshell_logic14_reflected(ip) as u32 }; hmix(ret, r4_h(ip)) };
    474 R4_ITRSHELL_LOGIC15_REFLECTED |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_rshell_logic15_reflected(ip) as u32 }; hmix(ret, r4_h(ip)) };
    475 R4_ITPEACHTOADSPORE_LOGIC68_REFLECTED |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_peach_toad_spore_logic68_reflected(ip) as u32 }; hmix(ret, r4_h(ip)) };
    476 R4_IT_2725_LOGIC50_REFLECTED |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_2725_logic50_reflected(ip) as u32 }; hmix(ret, r4_h(ip)) };
    477 R4_IT_2ADA_UNKMOTION0_ANIM |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_2ada_unk_motion0_anim(ip) as u32 }; hmix(ret, r4_h(ip)) };
    478 R4_IT_2F28_UNKMOTION0_ANIM |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_2f28_unk_motion0_anim(ip) as u32 }; hmix(ret, r4_h(ip)) };
    479 R4_ITCHICORITALEAF_UNKMOTION0_ANIM |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_chicoritaleaf_unk_motion0_anim(ip) as u32 }; hmix(ret, r4_h(ip)) };
    480 R4_ITFUSHIGIBANA_UNKMOTION1_ANIM_INLINE2 |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_fushigibana_unk_motion1_anim_inline2(ip) as u32 }; hmix(ret, r4_h(ip)) };
    481 R4_ITDRMARIOPILL_UNKMOTION0_ANIM |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_dr_mario_pill_unk_motion0_anim(ip) as u32 }; hmix(ret, r4_h(ip)) };
    482 R4_ITPEACHTOADSPORE_UNKMOTION0_ANIM |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_peachtoadspore_unk_motion0_anim(ip) as u32 }; hmix(ret, r4_h(ip)) };
    483 R4_ITSONANS_UNKMOTION1_ANIM |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_sonans_unk_motion1_anim(ip) as u32 }; hmix(ret, r4_h(ip)) };
    484 R4_IT_802D4FFC |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_802d4ffc(ip) as u32 }; hmix(ret, r4_h(ip)) };
    485 R4_ITBOX_UNKMOTION7_ANIM |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_box_unk_motion7_anim(ip) as u32 }; hmix(ret, r4_h(ip)) };
    486 R4_ITEGG_UNKMOTION6_ANIM |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_egg_unk_motion6_anim(ip) as u32 }; hmix(ret, r4_h(ip)) };
    487 R4_ITFOXLASER_UNKMOTION1_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_foxlaser_unk_motion1_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    488 R4_ITLGUNRAY_UNKMOTION0_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_lgunray_unk_motion0_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    489 R4_ITCEREBI_UNKMOTION2_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_cerebi_unk_motion2_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    490 R4_ITFIRE_UNKMOTION2_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_fire_unk_motion2_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    491 R4_ITMEW_UNKMOTION2_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_mew_unk_motion2_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    492 R4_ITCLIMBERSICE_SUB_X4 |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_climbers_ice_sub_x4(ip); 0 }; hmix(ret, r4_h(ip)) };
    493 R4_ITCOIN_UNKMOTION0_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_coin_unk_motion0_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    494 R4_ITCOIN_UNKMOTION3_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_coin_unk_motion3_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    495 R4_ITDOSEI_UNKMOTION1_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_dosei_unk_motion1_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    496 R4_ITDRMARIOPILL_MOTION2_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_dr_mario_pill_motion2_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    497 R4_ITDRMARIOPILL_MOTION2_COLL |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_dr_mariopill_motion2_coll(ip) as u32 }; hmix(ret, r4_h(ip)) };
    498 R4_ITDRMARIOPILL_MOTION6_COLL |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_dr_mario_pill_motion6_coll(ip) as u32 }; hmix(ret, r4_h(ip)) };
    499 R4_ITFOXILLUSION_LOGIC14_DMGDEALT |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_fox_illusion_logic14_dmg_dealt(ip) as u32 }; hmix(ret, r4_h(ip)) };
    500 R4_IT_802D4EF4 |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_802d4ef4(ip) as u32 }; hmix(ret, r4_h(ip)) };
    501 R4_ITLIPSTICK_UNKMOTION2_ANIM |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_lipstick_unk_motion2_anim(ip) as u32 }; hmix(ret, r4_h(ip)) };
    502 R4_ITSONANS_LOGIC9_DMGDEALT |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_sonans_logic9_dmg_dealt(ip) as u32 }; hmix(ret, r4_h(ip)) };
    503 R4_ITHITODEMAN_UNKMOTION1_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_hitodeman_unk_motion1_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    504 R4_ITLEADEAD_UNKMOTION3_ANIM_INLINE |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_leadead_unk_motion3_anim_inline(ip); 0 }; hmix(ret, r4_h(ip)) };
    505 R4_ITLEADEAD_UNKMOTION4_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_leadead_unk_motion4_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    506 R4_IT_802CC684 |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_802cc684(ip); 0 }; hmix(ret, r4_h(ip)) };
    507 R4_ITPEACHTOADSPORE_UNKMOTION0_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_peachtoadspore_unk_motion0_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    508 R4_ITPARASOL_UNKMOTION1_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_parasol_unk_motion1_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    509 R4_ITPARASOL_UNKMOTION2_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_parasol_unk_motion2_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    510 R4_ITSEAKNEEDLETHROWN_UNKMOTION0_ANIM |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_seakneedlethrown_unk_motion0_anim(ip) as u32 }; hmix(ret, r4_h(ip)) };
    511 R4_ITPIKACHUTJOLTGROUND_UNKMOTION0_ANIM |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_pikachutjoltground_unk_motion0_anim(ip) as u32 }; hmix(ret, r4_h(ip)) };
    512 R4_ITARWINGLASER_UNKMOTION3_ANIM |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_arwinglaser_unk_motion3_anim(ip); 0 }; hmix(ret, r4_h(ip)) };
    513 R4_ITNESSPKFLASHEXPLODE_UNKMOTION0_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_ness_pk_flash_explode_unk_motion0_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    514 R4_ITLIPSTICKSPORE_UNKMOTION0_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_lipstickspore_unk_motion0_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    515 R4_ITOCTAROCK_UNKMOTION1_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_octarock_unk_motion1_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    516 R4_ITOCTAROCK_UNKMOTION4_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_octarock_unk_motion4_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    517 R4_IT_802E5944 |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_802e5944(ip); 0 }; hmix(ret, r4_h(ip)) };
    518 R4_ITNOKONOKO_UNKMOTION8_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_nokonoko_unk_motion8_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    519 R4_ITPATAPATA_UNKMOTION4_COLL |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_patapata_unk_motion4_coll(ip) as u32 }; hmix(ret, r4_h(ip)) };
    520 R4_IT_802BDC38 |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_802bdc38(ip) as u32 }; hmix(ret, r4_h(ip)) };
    521 R4_IT_8029FDBC |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_8029fdbc(ip) as u32 }; hmix(ret, r4_h(ip)) };
    522 R4_IT_802AA7F0 |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_802aa7f0(ip) as u32 }; hmix(ret, r4_h(ip)) };
    523 R4_IT_8028D390 |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_8028d390(ip); 0 }; hmix(ret, r4_h(ip)) };
    524 R4_ITGAMEWATCHCHEF_UNKMOTION0_PHYS |ip, g, ov, x| { r4_pre(ip, g, x); ip.xDD4_itemVar.gamewatchchef.x4 = (g.r() % 4) as i32; let ret = { it_gamewatchchef_unk_motion0_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    525 R4_ITHAMMERHEAD_LOGIC40_SPAWNED |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_hammer_head_logic40_spawned(ip, &x.common); 0 }; hmix(ret, r4_h(ip)) };
    526 R4_ITHASSAM_UNKMOTION0_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_hassam_unk_motion0_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    527 R4_ITHASSAM_UNKMOTION2_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_hassam_unk_motion2_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    528 R4_ITMARIL_UNKMOTION1_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_maril_unk_motion1_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    529 R4_ITLINKARROW_UNKMOTION1_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_linkarrow_unk_motion1_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    530 R4_IT_80283BD4 |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_80283bd4(ip); 0 }; hmix(ret, r4_h(ip)) };
    531 R4_IT_802D98AC |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_802d98ac(ip); 0 }; hmix(ret, r4_h(ip)) };
    532 R4_IT_802F2020 |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_802f2020(ip) as u32 }; hmix(ret, r4_h(ip)) };
    533 R4_ITFREEZE_UNKMOTION4_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_freeze_unk_motion4_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    534 R4_ITFREEZER_UNKMOTION2_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_freezer_unk_motion2_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    535 R4_ITHOUOU_UNKMOTION4_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_houou_unk_motion4_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    536 R4_ITKIREIHANA_UNKMOTION4_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_kireihana_unk_motion4_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    537 R4_ITKAMEX_UNKMOTION1_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_kamex_unk_motion1_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    538 R4_ITLINKBOMB_UNKMOTION4_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  r4_rev(ip, 0); let ret = { it_linkbomb_unk_motion4_phys(ip); 0 }; r4_rev(ip, 0); hmix(ret, r4_h(ip)) };
    539 R4_ITSAMUSMISSILE_UNKMOTION1_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_samusmissile_unk_motion1_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    540 R4_IT_802CD7D4 |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_802cd7d4(ip) as u32 }; hmix(ret, r4_h(ip)) };
    541 R4_IT_8028D4E4 |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_8028d4e4(ip); 0 }; hmix(ret, r4_h(ip)) };
    542 R4_ITGSHELL_UNKMOTION3_ANIM |ip, g, ov, x| { r4_pre(ip, g, x);  r4_rev(ip, 24); let ret = { it_gshell_unk_motion3_anim(ip) as u32 }; r4_rev(ip, 24); hmix(ret, r4_h(ip)) };
    543 R4_IT_8028B988 |ip, g, ov, x| { r4_pre(ip, g, x);  r4_rev(ip, 24); let ret = { it_8028b988(ip); 0 }; r4_rev(ip, 24); hmix(ret, r4_h(ip)) };
    544 R4_IT_802D24A0 |ip, g, ov, x| { r4_pre(ip, g, x); ip.kind = if g.r() % 4 == 0 { (g.r() % 240) as i32 } else { 193 + (g.r() % 11) as i32 }; let ret = { it_802d24a0(ip); 0 }; hmix(ret, r4_h(ip)) };
    545 R4_IT_802CB844 |ip, g, ov, x| { r4_pre(ip, g, x); ip.kind = if g.r() % 4 == 0 { (g.r() % 240) as i32 } else { 193 + (g.r() % 11) as i32 }; let ret = { it_802cb844(ip); 0 }; hmix(ret, r4_h(ip)) };
    546 R4_ITFLIPPER_ADDSPINIMPULSE |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { let p = Vec { x: g.rf(), y: g.rf(), z: g.rf() }; let v = g.rf(); it_flipper_add_spin_impulse(ip, &p, v); 0 }; hmix(ret, r4_h(ip)) };
    547 R4_ITCHICORITA_PHYS |ip, g, ov, x| { r4_pre(ip, g, x);  let ret = { it_chicorita_phys(ip); 0 }; hmix(ret, r4_h(ip)) };
    548 R4_IT_802E657C |ip, g, ov, x| { r4_pre(ip, g, x);  r4_rev(ip, 24); let ret = { it_802e657c(ip) as u32 }; r4_rev(ip, 24); hmix(ret, r4_h(ip)) };
    600 R4_SAMUSMISSILE_FMA |ip, g, ov, x| {
        r4_pre(ip, g, x);
        ip.xDB0_itcmd_var1 = 1;
        ip.facing_dir = g.rm();
        ip.x40_vel.x = g.rm();
        x.sa[13] = 1e30f32.to_bits();
        it_samusmissile_unk_motion1_phys(ip);
        hmix(0, r4_h(ip))
    };
}
