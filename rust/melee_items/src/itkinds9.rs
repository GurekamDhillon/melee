//! Port round 9: hand ports of 26 item / toy functions whose one unported callee was
//! `HSD_Randi` / `HSD_Randf` (now `melee_runtime::random::HsdRand`) or `__frsqrte`
//! (`melee_core::fp::frsqrte`, wrapped in the MSL Newton steps as `melee_runtime::gekko`).
//! Verified against verbatim C copies (tests/c_oracle/round9_bodies.h, extracted by
//! gen_round9.py) in tests/oracle_round9.rs.
//!
//! The C RNG is a global (`HSD_RandSeedPtr`); here the caller passes the `HsdRand` that owns it
//! (the `melee` bin keeps one per match, `vs_match.rs`). Draw order is the C's.
//!
//! Fused sites (shipping-build contraction, checked in the IR of each TU):
//! - every Newton step `0.5 * g * (3.0 - g*g*x)`: one f64 fma `fma(-(g*g), x, 3.0)` (as
//!   `gekko::sqrtf`), 3 or 4 per sqrt;
//! - `_Toy_80309338`: 2 f32 (`dx*dx + dy*dy`, `dz*dz + ..`);
//! - `it_80278800_rand_vec`: 3 f32 (compound `+=`);
//! - `samus_grapple_calc_grav` 2, `it_802B9328_grav` 2 (`k*r - v` / `k*r + v`);
//! - `it_80289BE8_inline` 2 (`a*r - b*c`: the left product is fused);
//! - `it_802E61C4` 1 (`0.01F * t + 1.0F`).
//!
//! Same-named file-local inlines: the counter (and the extractor) take the first definition by
//! path. `my_sqrtf` is it_26B1.c's (3 steps; itdrmariopill/itlugia/itnessyoyo/itsamusbomb have
//! their own, itsamusbomb's with 4 steps), `sqrtf_store` is it_2E5A.c's, `sqrtf_accurate_store`
//! itmaplib.c's.
#![allow(non_snake_case)]

use crate::itkinds7::bisect_value;
use crate::layout::{
    itEgg_ItemVars, itLizardonAttributes, itPeachTurnipAttributes, itTaruAttributes,
    it_2E5A_Attrs, Item, ItemCommonData, ItemPickTable, Vec as GcVec,
};
use melee_core::fp::{fmadds, frsqrte};
use melee_runtime::random::HsdRand;

/// One MSL Newton step: `0.5 * g * (3.0 - g * g * x)`, the subtraction fused (f64).
#[inline(always)]
fn nr(g: f64, x: f64) -> f64 {
    (0.5 * g) * (-(g * g)).mul_add(x, 3.0)
}

#[inline(always)]
fn sqrt_n(x: f32, steps: u32) -> f32 {
    if x > 0.0 {
        let xd = x as f64;
        let mut g = frsqrte(xd);
        for _ in 0..steps {
            g = nr(g, xd);
        }
        return (xd * g) as f32;
    }
    x
}

/// `sqrtf_store` (it_2E5A.c): 3 steps, the result also stored through `y` (volatile in C).
/// `y` is left untouched when `x <= 0` or NaN.
pub fn sqrtf_store(x: f32, y: &mut f32) -> f32 {
    if x > 0.0 {
        *y = sqrt_n(x, 3);
        return *y;
    }
    x
}

/// `itzako_sqrtf` (itzako.c): 3 steps.
pub fn itzako_sqrtf(x: f32) -> f32 {
    sqrt_n(x, 3)
}

/// `_tyDisplay_80318CB4_sqrt_store` (tydisplay.c): 3 steps (`0.5F` / `3.0F` promote to the same
/// doubles), result stored through `rounded`.
pub fn ty_display_80318cb4_sqrt_store(x: f32, rounded: &mut f32) -> f32 {
    if x > 0.0 {
        *rounded = sqrt_n(x, 3);
        return *rounded;
    }
    x
}

/// `sqrtf_accurate_store` (itmaplib.c): 4 steps, stored through `y`.
pub fn sqrtf_accurate_store(x: f32, y: &mut f32) -> f32 {
    if x > 0.0 {
        *y = sqrt_n(x, 4);
        return *y;
    }
    x
}

/// `sqrtf_accurate_local` (itmaplib.c): 4 steps.
pub fn sqrtf_accurate_local(x: f32) -> f32 {
    sqrt_n(x, 4)
}

/// `sqrtf_accurate_sp18` (itmaplib.c): 4 steps.
pub fn sqrtf_accurate_sp18(x: f32) -> f32 {
    sqrt_n(x, 4)
}

/// `my_sqrtf` (it_26B1.c): 3 steps.
pub fn my_sqrtf(x: f32) -> f32 {
    sqrt_n(x, 3)
}

/// `my_sqrtf_accurate` (itlugia.c): 4 steps.
pub fn my_sqrtf_accurate(x: f32) -> f32 {
    sqrt_n(x, 4)
}

/// `_Toy_80309338` (toy.c): distance between two points; a `None` point is the origin
/// (`_Toy_803B88D4 = { 0 }`). `dy*dy`, then `dx*dx + _` and `dz*dz + _` fused (f32).
pub fn toy_80309338(arg0: Option<&GcVec>, arg1: Option<&GcVec>) -> f32 {
    let zero = GcVec { x: 0.0, y: 0.0, z: 0.0 };
    let a = arg0.unwrap_or(&zero);
    let b = arg1.unwrap_or(&zero);
    let mut v = a.y - b.y;
    let dx = a.x - b.x;
    let dz = a.z - b.z;
    v = v * v;
    v = fmadds(dx, dx, v);
    v = fmadds(dz, dz, v);
    sqrt_n(v, 3)
}

/// `randi_perm` (itunknown.c): `HSD_Randi(f)`; `f` converts to `s32` by truncation (the C is
/// undefined outside the `s32` range; Rust saturates there).
pub fn randi_perm(rand: &mut HsdRand, f: f32) -> i32 {
    rand.randi(f as i32)
}

/// `randi_perm_int` (itunknown.c).
pub fn randi_perm_int(rand: &mut HsdRand, i: i32) -> i32 {
    rand.randi(i)
}

/// `itKyasarin_Randi` (itkyasarin.c).
pub fn it_kyasarin_randi(rand: &mut HsdRand, range: i32) -> i32 {
    rand.randi(range)
}

/// `attrRand` (itegg.c).
pub fn attr_rand(rand: &mut HsdRand, attrs: &itEgg_ItemVars) -> i32 {
    rand.randi(attrs.rand_max)
}

/// `it_8026C65C` (itspawn.c): weighted kind pick from an `ItemPickTable`.
/// # Safety
/// `table.x4` holds `table.size` kinds and `table.xC` the `size + 1` cumulative weights.
pub unsafe fn it_8026c65c(rand: &mut HsdRand, table: &ItemPickTable) -> i32 {
    let total = table.x8 as i32;
    let v = rand.randi(total);
    *table.x4.offset(bisect_value(v, table, 0, table.size as i32) as isize) as i32
}

/// `itTaru_RandCheck` (ittaru.c): `HSD_Randi(x0 + x4) < x0`.
pub fn it_taru_rand_check(rand: &mut HsdRand, attr: &itTaruAttributes) -> bool {
    rand.randi(attr.x0.wrapping_add(attr.x4)) < attr.x0
}

/// `it_80278800_rand_vec` (iteffect.c): `out += 2 v (r - 0.5)` per axis, one draw each.
pub fn it_80278800_rand_vec(rand: &mut HsdRand, out: &mut GcVec, v: &GcVec) {
    let f = rand.randf();
    out.x = fmadds(2.0 * v.x, f - 0.5, out.x);
    let f = rand.randf();
    out.y = fmadds(2.0 * v.y, f - 0.5, out.y);
    let f = rand.randf();
    out.z = fmadds(2.0 * v.z, f - 0.5, out.z);
}

#[inline(always)]
unsafe fn lizardon_x70(rand: &mut HsdRand, ip: &mut Item) {
    let a = &*((*ip.xC4_article_data).x4_specialAttributes as *const itLizardonAttributes);
    let (lo, hi) = (a.x28, a.x2C);
    ip.xDD4_itemVar.lizardon.x70 = lo.wrapping_add(rand.randi(hi.wrapping_sub(lo)));
}

/// `it_802CBAA8_inline` (itlizardon.c): `x70 = x28 + HSD_Randi(x2C - x28)`.
/// # Safety
/// The item's article data and special attributes are valid.
pub unsafe fn it_802cbaa8_inline(rand: &mut HsdRand, ip: &mut Item) {
    lizardon_x70(rand, ip)
}

/// `it_802CBD24_inline` (itlizardon.c): same body as `it_802CBAA8_inline`.
/// # Safety
/// As `it_802cbaa8_inline`.
pub unsafe fn it_802cbd24_inline(rand: &mut HsdRand, ip: &mut Item) {
    lizardon_x70(rand, ip)
}

/// `samus_grapple_calc_grav` (itsamusgrapple.c). Both comparisons are in f64 (`0.9`, `0.0`).
pub fn samus_grapple_calc_grav(rand: &mut HsdRand, vel_y: f32) -> f32 {
    if rand.randf() as f64 > 0.9 {
        if (vel_y as f64) < 0.0 {
            -fmadds(0.6, rand.randf(), -vel_y)
        } else {
            fmadds(0.6, rand.randf(), vel_y)
        }
    } else {
        0.0
    }
}

/// `it_802B9328_grav` (itsamusgrapple.c): as `samus_grapple_calc_grav` with factor 1.
pub fn it_802b9328_grav(rand: &mut HsdRand, vely: f32) -> f32 {
    let one = 1.0f32;
    if rand.randf() as f64 > 0.9 {
        if (vely as f64) < 0.0 {
            -fmadds(one, rand.randf(), -vely)
        } else {
            fmadds(one, rand.randf(), vely)
        }
    } else {
        0.0
    }
}

/// `it_8026F53C` (itdrop.c): unless `chk`, draws the drop velocity; then copies it to the item.
/// `common` is the C global `it_804D6D28`.
/// # Safety
/// `ip.xCC_item_attr` is valid when `chk` is false.
pub unsafe fn it_8026f53c(rand: &mut HsdRand, ip: &mut Item, vel: &mut GcVec, chk: bool, common: &ItemCommonData) {
    if !chk {
        vel.x = common.x54_float * (2.0 * (rand.randf() - 0.5));
        vel.y = (*ip.xCC_item_attr).x18;
        vel.z = 0.0;
    }
    ip.x40_vel = *vel;
}

#[inline(always)]
fn box_pick(rand: &mut HsdRand, arg1: i32, arg2: i32, arg3: i32, arg4: i32) -> bool {
    let sum1 = arg1.wrapping_add(arg2);
    let sum2 = arg3.wrapping_add(sum1);
    let r = rand.randi(arg4.wrapping_add(sum2));
    r < arg1 || r < sum1 || r < sum2
}

/// `it_80286340` (itbox.c). The GObj argument is unused.
pub fn it_80286340(rand: &mut HsdRand, arg1: i32, arg2: i32, arg3: i32, arg4: i32) -> bool {
    box_pick(rand, arg1, arg2, arg3, arg4)
}

/// `it_8028A114` (itkusudama.c): same body as `it_80286340`.
pub fn it_8028a114(rand: &mut HsdRand, arg1: i32, arg2: i32, arg3: i32, arg4: i32) -> bool {
    box_pick(rand, arg1, arg2, arg3, arg4)
}

/// `it_80289BE8_inline` (itkusudama.c): spawn position 5 below the item and a random velocity.
pub fn it_80289be8_inline(rand: &mut HsdRand, ip: &Item, vel_scale: f32, pos: &mut GcVec, vel: &mut GcVec) {
    *pos = ip.pos;
    pos.y -= 5.0;
    let r = rand.randf();
    vel.x = fmadds(vel_scale, r, -(vel_scale * 0.5));
    let r = rand.randf();
    vel.y = fmadds(vel_scale * 0.5, r, -(vel_scale * 0.25));
    vel.z = 0.0;
}

/// `it_802E61C4` (it_2E5A.c): `arg1 * x2C[arg2]` scaled by `1 + 0.01 * x24 * r`, at least 1.
/// # Safety
/// The item's special attributes are an `it_2E5A_Attrs`, `arg2` in 0..4; the products fit `s32`
/// (the C float-to-int conversion is undefined otherwise).
pub unsafe fn it_802e61c4(rand: &mut HsdRand, ip: &Item, arg1: i32, arg2: i32) -> i32 {
    let attr = &*((*ip.xC4_article_data).x4_specialAttributes as *const it_2E5A_Attrs);
    let mut v = (arg1 as f32 * attr.x2C[arg2 as usize]) as i32;
    let r = rand.randf();
    let t = attr.x24 * r;
    v = (v as f32 * fmadds(0.01, t, 1.0)) as i32;
    if v < 1 {
        v = 1;
    }
    v
}

/// `it_802BD32C` (itpeachturnip.c): weighted turnip pick; -1 if none.
/// # Safety
/// The item's special attributes are an `itPeachTurnipAttributes` with `x4_length <= 7` (the C
/// reads `x8[length]`).
pub unsafe fn it_802bd32c(rand: &mut HsdRand, ip: &Item) -> i32 {
    let attr = &*((*ip.xC4_article_data).x4_specialAttributes as *const itPeachTurnipAttributes);
    let n = attr.x4_length;
    let mut sum = 0i32;
    let mut i = 0;
    while i < n {
        sum = sum.wrapping_add(attr.x8[i as usize].x0_odds);
        i += 1;
    }
    let r = rand.randi(sum);
    let mut cum = attr.x8[0].x0_odds;
    let mut ret = -1;
    i = 0;
    while i < n && ret == -1 {
        if r < cum {
            ret = i;
        }
        cum = cum.wrapping_add(attr.x8[(i + 1) as usize].x0_odds);
        i += 1;
    }
    ret
}
