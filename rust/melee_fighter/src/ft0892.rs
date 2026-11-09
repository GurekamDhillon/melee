
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
pub(crate) fn fp<'a>(g: G) -> &'a mut Fighter {
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
pub(crate) fn x2073(s: &Struct2070) -> u8 {
    unsafe { s.x0.x2073 }
}

#[inline(always)]
pub(crate) fn bits2074(f: &mut Fighter) -> &mut Struct2074_x10_x0 {
    unsafe { &mut f.x2074.x10.x0 }
}

/// The 2074 reset shared by 800892D4 / 80089460 / 80089768 (same stores, same order).
pub(crate) fn reset_2074(f: &mut Fighter) {
    f.x2074.x2074_vec.x = 0;
    f.x2074.x2074_vec.y = 0;
    f.x2074.x207C.x = 0.0;
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

pub(crate) fn snapshot_flags(f: &mut Fighter) {
    f.x2074.x2074_vec.x = melee_core::ppc::fctiwz_f32(f.dmg.x1830_percent);
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

/// `ftGetTemplateKind` (ft/inlines.h, TARGET_PC).
#[inline]
pub fn template_kind(f: &Fighter) -> i32 {
    if f.kind >= Ft_Kind_Mex0 && h::geno_define_template_skeleton(f.kind as i32) != 0 {
        return h::geno_define_base_kind(f.kind as i32);
    }
    f.kind as i32
}

pub use crate::tu::ft_0892::ft_800892d4;
pub use crate::tu::ft_0892::ft_80089460;
pub use crate::tu::ft_0892::ft_800895e0;
pub use crate::tu::ft_0892::ft_80089768_fp as ft_80089768;
pub use crate::tu::ft_0892::ft_80089824_fp as ft_80089824;
pub use crate::tu::ft_0892::ft_80089884_fp as ft_80089884;
pub use crate::tu::ft_0892::ft_80089890_fp as ft_80089890;
pub use crate::tu::ft_0892::ft_8008989c;
pub use crate::tu::ft_0892::ft_800898a8;
pub use crate::tu::ft_0892::ft_800898b4;
pub use crate::tu::ft_0892::ft_800898c0;
pub use crate::tu::ft_0892::ft_80089914;
