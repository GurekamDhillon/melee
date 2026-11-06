
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

use melee_core::fp::Fmadd as _;
use crate::itkinds7::bisect_value;

use crate::layout::{
    itEgg_ItemVars, itLizardonAttributes, itPeachTurnipAttributes, itTaruAttributes,
    it_2E5A_Attrs, Item, ItemCommonData, ItemPickTable, Vec as GcVec,
};

use melee_core::fp::{fmadds, frsqrte};

use melee_runtime::random::HsdRand;

/// One MSL Newton step: `0.5 * g * (3.0 - g * g * x)`, the subtraction fused (f64).
#[inline(always)]
pub(crate) fn nr(g: f64, x: f64) -> f64 {
    (0.5 * g) * (-(g * g)).fmadd(x, 3.0)
}

#[inline(always)]
pub(crate) fn sqrt_n(x: f32, steps: u32) -> f32 {
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

/// `my_sqrtf` (it_26B1.c): 3 steps.
pub fn my_sqrtf(x: f32) -> f32 {
    sqrt_n(x, 3)
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

/// `itKyasarin_Randi` (itkyasarin.c).
pub fn it_kyasarin_randi(rand: &mut HsdRand, range: i32) -> i32 {
    rand.randi(range)
}

/// `attrRand` (itegg.c).
pub fn attr_rand(rand: &mut HsdRand, attrs: &itEgg_ItemVars) -> i32 {
    rand.randi(attrs.rand_max)
}

/// `itTaru_RandCheck` (ittaru.c): `HSD_Randi(x0 + x4) < x0`.
pub fn it_taru_rand_check(rand: &mut HsdRand, attr: &itTaruAttributes) -> bool {
    rand.randi(attr.x0.wrapping_add(attr.x4)) < attr.x0
}

#[inline(always)]
pub(crate) unsafe fn lizardon_x70(rand: &mut HsdRand, ip: &mut Item) {
    let a = &*((*ip.xC4_article_data).x4_specialAttributes as *const itLizardonAttributes);
    let (lo, hi) = (a.x28, a.x2C);
    ip.xDD4_itemVar.lizardon.x70 = lo.wrapping_add(rand.randi(hi.wrapping_sub(lo)));
}

#[inline(always)]
pub(crate) fn box_pick(rand: &mut HsdRand, arg1: i32, arg2: i32, arg3: i32, arg4: i32) -> bool {
    let sum1 = arg1.wrapping_add(arg2);
    let sum2 = arg3.wrapping_add(sum1);
    let r = rand.randi(arg4.wrapping_add(sum2));
    r < arg1 || r < sum1 || r < sum2
}

pub use crate::tu::itspawn::it_8026c65c;
pub use crate::tu::itdrop::it_8026f53c;
pub use crate::tu::iteffect::it_80278800_rand_vec_fp as it_80278800_rand_vec;
pub use crate::tu::itbox::it_80286340_fp as it_80286340;
pub use crate::tu::itkusudama::it_80289be8_inline;
pub use crate::tu::itkusudama::it_8028a114;
pub use crate::tu::itsamusgrapple::it_802b9328_grav;
pub use crate::tu::itpeachturnip::it_802bd32c;
pub use crate::tu::itlizardon::it_802cbaa8_inline;
pub use crate::tu::itlizardon::it_802cbd24_inline;
pub use crate::tu::it_2E5A::it_802e61c4;
pub use crate::tu::itzako::itzako_sqrtf_fp as itzako_sqrtf;
pub use crate::tu::itlugia::my_sqrtf_accurate_fp as my_sqrtf_accurate;
pub use crate::tu::itunknown::randi_perm_fp as randi_perm;
pub use crate::tu::itunknown::randi_perm_int_fp as randi_perm_int;
pub use crate::tu::itsamusgrapple::samus_grapple_calc_grav_fp as samus_grapple_calc_grav;
pub use crate::tu::itmaplib::sqrtf_accurate_local_fp as sqrtf_accurate_local;
pub use crate::tu::itmaplib::sqrtf_accurate_sp18_fp as sqrtf_accurate_sp18;
