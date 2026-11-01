//! ft_0892.c: ports folded from the hand-written modules (consolidation steps 21/22).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use crate::hooks as h;
#[allow(unused_imports)]
use crate::types::*;
#[allow(unused_imports)]
use crate::ft0892::{G, fp, reset_18d8, reset_2074, s2070_from_gc_int, snapshot_flags, template_kind, x2073};

// ---- folded from ft0892.rs (consolidation steps 21/22) ----
/// `ft_800892D4`: clear every hit/attack record.
pub fn ft_800892d4(f: &mut Fighter) {
    melee_core::ct!("ft_800892d4");
    f.dmg.x18d4 = UnkPlBonusBits::zeroed();
    // x2070_int = *(s32*)&x18d4: a 4-byte copy (both are GC-ordered bytes)
    f.x2070 = unsafe { core::mem::transmute::<UnkPlBonusBits, Struct2070>(f.dmg.x18d4) };
    reset_2074(f);
    reset_18d8(&mut f.dmg.x18d8);
    f.dmg.x18ec_instancehitby = 0;
    f.x2074.x2088 = 0;
}

/// `ft_80089460`.
pub fn ft_80089460(f: &mut Fighter) {
    melee_core::ct!("ft_80089460");
    reset_2074(f);
    snapshot_flags(f);
}

/// `ft_800895E0(fp, arg1)`: `arg1` is the `Struct2070` the C puns out of an int.
pub fn ft_800895e0(f: &mut Fighter, arg: Struct2070) {
    melee_core::ct!("ft_800895e0");
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
    ft_80089768_fp(&mut f.x2074);
    snapshot_flags(f);
}

/// `ft_80089768_fp(Vec2*)`: the C casts `&x2074.x2074_vec` to a local mirror of `Struct2074`
/// (x10_b7..x10_b0 = x2084_b0..b7, x11_b7..x11_b3 = x2085_b0..b4); the only caller passes that.
pub fn ft_80089768_fp(s: &mut Struct2074) {
    melee_core::ct!("ft_80089768_fp");
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

/// `ft_80089824_fp`.
pub fn ft_80089824_fp(g: G) {
    melee_core::ct!("ft_80089824_fp");
    let f = fp(g);
    // fp->x2070.x2070_int = val.x2070_int: a self-copy
    ft_80089460(f);
    f.x2074.x2088 = crate::stale::attack_counter_next();
    h::pl_80037c60(g, Struct2070::zeroed());
}

pub fn ft_80089884_fp(g: G) -> *mut Struct2070 {
    melee_core::ct!("ft_80089884_fp");
    &mut fp(g).x2070
}

pub fn ft_80089890_fp(g: G) -> *mut UnkPlBonusBits {
    melee_core::ct!("ft_80089890_fp");
    &mut fp(g).dmg.x18d4
}

pub fn ft_8008989c(g: G) -> u16 {
    melee_core::ct!("ft_8008989c");
    fp(g).dmg.x18ec_instancehitby
}

pub fn ft_800898a8(g: G) -> *mut Struct2074 {
    melee_core::ct!("ft_800898a8");
    &mut fp(g).x2074
}

pub fn ft_800898b4(g: G) -> *mut ft_800898B4_t {
    melee_core::ct!("ft_800898b4");
    &mut fp(g).dmg.x18d8
}

/// `ft_800898C0`.
pub fn ft_800898c0(g: G) -> bool {
    melee_core::ct!("ft_800898c0");
    let f = fp(g);
    let k = x2073(&f.x2070);
    k == 113 || k == 114 || k == 99 || h::ftfx_appeals_check_if_used(f)
}

/// `ft_80089914`: is `msid` a side tilt/smash that counts (Fox/Falco low f-tilts and Mewtwo's
/// AttackS3S excluded).
pub fn ft_80089914(g: G, msid: i32) -> bool {
    melee_core::ct!("ft_80089914");
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
