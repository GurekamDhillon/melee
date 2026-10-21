//! Hand port, round 7: all 12 C definitions of src/melee/ft/ft_0892.c (ft_800892D4 .. ft_80089914):
//! hit/attack event records `Fighter.x2070` (Struct2070), `x2074` (Struct2074), `dmg.x18d4`,
//! `dmg.x18d8` (ft_800898B4_t), `dmg.x18ec_instancehitby`.
//!
//! Fused sites (`tools/rust-port/fma_sites.py src/melee/ft/ft_0892.c`): none.
//! Checked against the C (cut verbatim, shipping contraction) by tests/ft0892_ref.rs.
//!
//! Byte-order note: `Struct2070` is a union of `{s8 x2070; bits; u8 x2073}` and `int x2070_int`.
//! The game is big-endian, so an int literal stored into `x2070_int` puts its low byte in
//! `x2073`. The native layout keeps the GameCube byte order of the struct view, so the int view is
//! [`s2070_from_gc_int`] / [`s2070_gc_int`] (big-endian over the four bytes), not the native
//! union member. `ft_800895E0`'s int argument is taken as the `Struct2070` it puns (the C's own
//! TODO says so). `*(s32*)&float_field = n` stores are `f32::from_bits`.

use crate::hooks as h;
use crate::types::*;

pub type G = *mut HSD_GObj;

#[inline(always)]
fn fp<'a>(g: G) -> &'a mut Fighter {
    unsafe { &mut *((*g).user_data as *mut Fighter) }
}

/// The four bytes of a `Struct2070` (GameCube order).
#[inline]
pub fn s2070_bytes(s: &Struct2070) -> [u8; 4] {
    unsafe { core::mem::transmute::<Struct2070, [u8; 4]>(*s) }
}
/// The big-endian `x2070_int` of the game (what the PPC reads through the int member).
#[inline]
pub fn s2070_gc_int(s: &Struct2070) -> i32 {
    i32::from_be_bytes(s2070_bytes(s))
}
/// `union Struct2070 u; u.x2070_int = v;` as the game (big-endian) lays it out.
#[inline]
pub fn s2070_from_gc_int(v: i32) -> Struct2070 {
    unsafe { core::mem::transmute::<[u8; 4], Struct2070>(v.to_be_bytes()) }
}
#[inline(always)]
fn x2073(s: &Struct2070) -> u8 {
    unsafe { s.anon_0.x2073 }
}

#[inline(always)]
fn bits2074(f: &mut Fighter) -> &mut Struct2074_anon_1061_anon_1063 {
    unsafe { &mut f.x2074.anon_10.anon_0 }
}

/// The 2074 reset shared by 800892D4 / 80089460 / 80089768 (same stores, same order).
fn reset_2074(f: &mut Fighter) {
    f.x2074.x2074_vec.x = f32::from_bits(0);
    f.x2074.x2074_vec.y = f32::from_bits(0);
    f.x2074.x207C.x = 0.0f32.to_bits() as i32;
    f.x2074.x207C.y = 6;
    let b = bits2074(f);
    b.set_x2084_b0(0);
    b.set_x2085_b3(0);
    b.set_x2084_b1(0);
    b.set_x2084_b2(0);
    b.set_x2084_b3(0);
    b.set_x2084_b4(0);
    b.set_x2084_b5(0);
    b.set_x2084_b6(0);
    b.set_x2084_b7(0);
    b.set_x2085_b0(0);
    b.set_x2085_b1(0);
    b.set_x2085_b2(0);
    b.set_x2085_b4(0);
}

/// The ft_800898B4_t reset (also fn_80037F00's NULL path).
pub fn reset_18d8(e: &mut ft_800898B4_t) {
    e.x0 = 0;
    e.x4 = 0;
    e.kb_applied1 = 0.0;
    e.xC = 6;
    e.set_x10_b0(0);
    e.set_x11_b3(0);
    e.set_x10_b1(0);
    e.set_x10_b2(0);
    e.set_x10_b3(0);
    e.set_x10_b4(0);
    e.set_x10_b5(0);
    e.set_x10_b6(0);
    e.set_x10_b7(0);
    e.set_x11_b0(0);
    e.set_x11_b1(0);
    e.set_x11_b2(0);
    e.set_x11_b4(0);
}

/// `ft_800892D4`: clear every hit/attack record.
pub fn ft_800892d4(f: &mut Fighter) {
    f.dmg.x18d4 = UnkPlBonusBits::zeroed();
    // x2070_int = *(s32*)&x18d4: a 4-byte copy (both are GC-ordered bytes)
    f.x2070 = unsafe { core::mem::transmute::<UnkPlBonusBits, Struct2070>(f.dmg.x18d4) };
    reset_2074(f);
    reset_18d8(&mut f.dmg.x18d8);
    f.dmg.x18ec_instancehitby = 0;
    f.x2074.x2088 = 0;
}

fn snapshot_flags(f: &mut Fighter) {
    f.x2074.x2074_vec.x = f32::from_bits(f.dmg.x1830_percent as i32 as u32);
    let metal = f.is_metal() != 0;
    let x197c = !f.x197C.is_null();
    let (b3, b4, b5, b6) = (f.x221D_b6() != 0, f.x2226_b4() != 0, f.x2220_b5() != 0, f.x2220_b6() != 0);
    let b = bits2074(f);
    if metal {
        b.set_x2084_b1(1);
    }
    if x197c {
        b.set_x2084_b2(1);
    }
    if b3 {
        b.set_x2084_b3(1);
    }
    if b4 {
        b.set_x2084_b4(1);
    }
    if b5 {
        b.set_x2084_b5(1);
    }
    if b6 {
        b.set_x2084_b6(1);
    }
}

/// `ft_80089460`.
pub fn ft_80089460(f: &mut Fighter) {
    reset_2074(f);
    snapshot_flags(f);
}

/// `ft_800895E0(fp, arg1)`: `arg1` is the `Struct2070` the C puns out of an int.
pub fn ft_800895e0(f: &mut Fighter, arg: Struct2070) {
    let mut val = arg;
    if x2073(&val) == 0 || x2073(&val) != x2073(&f.x2070) {
        f.x2074.x2088 = crate::stale::attack_counter_next();
    }
    if f.kind == Ft_Kind_Luigi && x2073(&val) == 0x71 {
        val = s2070_from_gc_int(0x240063);
    }
    if x2073(&val) == 0x62 && !f.item_gobj.is_null() && h::it_8026b6c8(f.item_gobj) != 0 {
        val = s2070_from_gc_int(0x44003D);
    }
    f.x2070 = val;
    ft_80089768(&mut f.x2074);
    snapshot_flags(f);
}

/// `ft_80089768(Vec2*)`: the C casts `&x2074.x2074_vec` to a local mirror of `Struct2074`
/// (x10_b7..x10_b0 = x2084_b0..b7, x11_b7..x11_b3 = x2085_b0..b4); the only caller passes that.
pub fn ft_80089768(s: &mut Struct2074) {
    s.x2074_vec.x = f32::from_bits(0);
    s.x2074_vec.y = f32::from_bits(0);
    s.x207C.x = 0.0f32.to_bits() as i32;
    s.x207C.y = 6;
    let b = unsafe { &mut s.anon_10.anon_0 };
    b.set_x2084_b0(0);
    b.set_x2085_b3(0);
    b.set_x2084_b1(0);
    b.set_x2084_b2(0);
    b.set_x2084_b3(0);
    b.set_x2084_b4(0);
    b.set_x2084_b5(0);
    b.set_x2084_b6(0);
    b.set_x2084_b7(0);
    b.set_x2085_b0(0);
    b.set_x2085_b1(0);
    b.set_x2085_b2(0);
    b.set_x2085_b4(0);
}

/// `ft_80089824`.
pub fn ft_80089824(g: G) {
    let f = fp(g);
    // fp->x2070.x2070_int = val.x2070_int: a self-copy
    ft_80089460(f);
    f.x2074.x2088 = crate::stale::attack_counter_next();
    h::pl_80037c60(g, Struct2070::zeroed());
}

pub fn ft_80089884(g: G) -> *mut Struct2070 {
    &mut fp(g).x2070
}
pub fn ft_80089890(g: G) -> *mut UnkPlBonusBits {
    &mut fp(g).dmg.x18d4
}
pub fn ft_8008989c(g: G) -> u16 {
    fp(g).dmg.x18ec_instancehitby
}
pub fn ft_800898a8(g: G) -> *mut Struct2074 {
    &mut fp(g).x2074
}
pub fn ft_800898b4(g: G) -> *mut ft_800898B4_t {
    &mut fp(g).dmg.x18d8
}

/// `ft_800898C0`.
pub fn ft_800898c0(g: G) -> bool {
    let f = fp(g);
    let k = x2073(&f.x2070);
    k == 113 || k == 114 || k == 99 || h::ftfx_appeals_check_if_used(f)
}

/// `ftGetTemplateKind` (ft/inlines.h, TARGET_PC).
#[inline]
pub fn template_kind(f: &Fighter) -> i32 {
    if f.kind >= Ft_Kind_Mex0 && h::geno_define_template_skeleton(f.kind as i32) != 0 {
        return h::geno_define_base_kind(f.kind as i32);
    }
    f.kind as i32
}

/// `ft_80089914`: is `msid` a side tilt/smash that counts (Fox/Falco low f-tilts and Mewtwo's
/// AttackS3S excluded).
pub fn ft_80089914(g: G, msid: i32) -> bool {
    let f = fp(g);
    if msid < ftCo_MS_AttackS3Hi as i32 || msid > ftCo_MS_AttackS4LwS as i32 {
        return false;
    }
    let low3 = (msid.wrapping_sub(ftCo_MS_AttackS3LwS as i32) as u32) <= 2;
    if template_kind(f) == Ft_Kind_Fox as i32 && low3 {
        return false;
    }
    if template_kind(f) == Ft_Kind_Falco as i32 && low3 {
        return false;
    }
    if f.kind == Ft_Kind_Mewtwo && msid == ftCo_MS_AttackS3S as i32 {
        return false;
    }
    true
}
