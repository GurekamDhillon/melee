
//! `src/melee/lb/lbtrigf.c`: the game's atanf / atan2f / acosf / asinf (the TARGET_PC atanf,
//! which reproduces the retail fused arithmetic). Contraction sites of the C build are explicit.

use crate::gekko::frsqrte;

use melee_core::fp::fmadds;

pub(crate) const M_PI: f64 = 3.14159265358979323846;

pub(crate) const M_PI_2: f64 = M_PI / 2.0;

/// MSL float.c `MSL_TrigF_80400770` / `..74`: NaN 0x7FFFFFFF, +inf.
pub(crate) const NAN: u32 = crate::tu::float::MSL_TrigF_80400770[0] as u32;

pub(crate) const INF: u32 = crate::tu::float::MSL_TrigF_80400774[0] as u32;

pub(crate) const SIGN_BIT: u32 = 1 << 31;

pub(crate) const BITWISE_PI_2: u32 = 0x3FC9_0FDB;

/// C double literals in a float table: decimal -> f64 -> f32.
#[rustfmt::skip]
pub(crate) static ATANF_LOOKUP: [f32; 46] = [
    1.0, -0.3333333134651184_f64 as f32, 0.1999988704919815_f64 as f32,
    -0.14281649887561798_f64 as f32, 0.11041180044412613_f64 as f32,
    -0.08459755778312683_f64 as f32, 0.04714243486523628_f64 as f32,
    6.828420162200928_f64 as f32, 3.239828109741211_f64 as f32, 2.0,
    1.4464620351791382_f64 as f32, 1.1715729236602783_f64 as f32, 1.039566159248352_f64 as f32,
    7.1350000325764995e-06_f64 as f32, 8.200000252145401e-07_f64 as f32, 0.0,
    6.299999881775875e-07_f64 as f32, 0.0, 0.0, 0.0,
    0.3926900029182434_f64 as f32, 0.5890486240386963_f64 as f32, 0.7853981256484985_f64 as f32,
    0.9817469716072083_f64 as f32, 1.1780970096588135_f64 as f32, 1.3744460344314575_f64 as f32,
    0.0, 9.081698408408556e-06_f64 as f32, 2.3000000126671694e-08_f64 as f32,
    6.30000016599297e-08_f64 as f32, 7.040000014058023e-07_f64 as f32,
    2.499999993688107e-07_f64 as f32, 7.900000014160469e-07_f64 as f32,
    2.414212942123413_f64 as f32, 1.4966057538986206_f64 as f32, 1.0,
    0.6681786179542542_f64 as f32, 0.4142135679721832_f64 as f32, 0.1989123672246933_f64 as f32,
    5.620000251838064e-07_f64 as f32, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
];

/// `lookup_ptr[k]` with `lookup_ptr = &atanf_lookup[lookup_index]` (index may be -1).
#[inline]
pub(crate) fn lk(index: i32, k: i32) -> f32 {
    ATANF_LOOKUP[(index + k) as usize]
}

/// `lbTrigf_Fnmsubs`: `(float)-((double)a*(double)c - (double)b)` (an exact-product fma in the
/// C build, so the plain double expression is the same).
#[inline]
pub(crate) fn fnmsubs(a: f32, c: f32, b: f32) -> f32 {
    (-(a as f64 * c as f64 - b as f64)) as f32
}

/// The float Newton step of lbtrigf.c: `0.5f * g * (3.0f - g*g*v)`; the C build fuses
/// `3.0f - (g*g)*v` into one fmadds.
#[inline]
pub(crate) fn nr_step_f(g: f32, v: f32) -> f32 {
    (0.5f32 * g) * fmadds(-(g * g), v, 3.0)
}

pub use crate::tu::lbtrigf::acosf;
pub use crate::tu::lbtrigf::asinf_fp as asinf;
pub use crate::tu::lbtrigf::atan2f_fp as atan2f;
pub use crate::tu::lbtrigf::atanf_fp as atanf;
pub use crate::tu::lbtrigf::lb_sqrtf_fp as lb_sqrtf;
