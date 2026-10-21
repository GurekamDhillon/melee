//! `src/melee/lb/lbtrigf.c`: the game's atanf / atan2f / acosf / asinf (the TARGET_PC atanf,
//! which reproduces the retail fused arithmetic). Contraction sites of the C build are explicit.

use crate::gekko::frsqrte;
use melee_core::fp::fmadds;

const M_PI: f64 = 3.14159265358979323846;
const M_PI_2: f64 = M_PI / 2.0;
/// MSL float.c `MSL_TrigF_80400770` / `..74`: NaN 0x7FFFFFFF, +inf.
const NAN: u32 = 0x7FFF_FFFF;
const INF: u32 = 0x7F80_0000;
const SIGN_BIT: u32 = 1 << 31;
const BITWISE_PI_2: u32 = 0x3FC9_0FDB;

/// C double literals in a float table: decimal -> f64 -> f32.
#[rustfmt::skip]
static ATANF_LOOKUP: [f32; 46] = [
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
fn lk(index: i32, k: i32) -> f32 {
    ATANF_LOOKUP[(index + k) as usize]
}

/// `lbTrigf_Fnmsubs`: `(float)-((double)a*(double)c - (double)b)` (an exact-product fma in the
/// C build, so the plain double expression is the same).
#[inline]
fn fnmsubs(a: f32, c: f32, b: f32) -> f32 {
    (-(a as f64 * c as f64 - b as f64)) as f32
}

/// `atanf` (TARGET_PC branch of lbtrigf.c).
pub fn atanf(x: f32) -> f32 {
    let silver_ratio = 2.4142136573791504f32;
    let silver_ratio_conjugate = 0.4142135679721832f32;
    let mut lookup_index: i32 = -1;
    let mut x_ge_ratio = false;
    let sign_bit_x = x.to_bits() & SIGN_BIT;
    let x = f32::from_bits(x.to_bits() & !SIGN_BIT);
    let xb = x.to_bits() as i32;

    let mut result;
    if x >= silver_ratio {
        x_ge_ratio = true;
        result = 1.0f32 / x;
    } else if silver_ratio_conjugate < x {
        lookup_index = 0;
        match x.to_bits() & 0x7F80_0000 {
            0x3F00_0000 => {
                if !(xb < 0x3F08_D5B9) {
                    lookup_index = 1;
                }
                if !(xb < 0x3F52_1801) {
                    lookup_index += 1;
                }
            }
            0x3F80_0000 => {
                lookup_index = 2;
                if !(xb < 0x3F9B_F7EC) {
                    lookup_index = 3;
                }
                if !(xb < 0x3FEF_789E) {
                    lookup_index += 1;
                }
            }
            0x4000_0000 => lookup_index = 4,
            _ => {}
        }
        let offset_39 = lk(lookup_index, 39);
        let offset_33 = lk(lookup_index, 33);
        result = 1.0f32 / (offset_33 + (x + offset_39));
        result = fnmsubs(result, lk(lookup_index, 7), offset_33)
            + fnmsubs(result, lk(lookup_index, 13), offset_39);
    } else {
        result = x;
    }

    let rs = result * result;
    let a = &ATANF_LOOKUP;
    let mut poly = fmadds(rs, a[6], a[5]);
    poly = fmadds(rs, poly, a[4]);
    poly = fmadds(rs, poly, a[3]);
    poly = fmadds(rs, poly, a[2]);
    poly = fmadds(rs, poly, a[1]);
    result = fmadds(result * rs, poly, result);
    result += lk(lookup_index, 27);
    result += lk(lookup_index, 20);

    if x_ge_ratio {
        result -= M_PI_2 as f32;
        return if sign_bit_x != 0 { result } else { -result };
    }
    f32::from_bits(result.to_bits() | sign_bit_x)
}

/// `atan2f`.
pub fn atan2f(y: f32, x: f32) -> f32 {
    let sx = x.to_bits() & SIGN_BIT;
    let sy = y.to_bits() & SIGN_BIT;
    if sx == sy {
        if sx != 0 {
            return if x == -0.0 { -M_PI_2 as f32 } else { atanf(y / x) - M_PI as f32 };
        }
        return if x != 0.0 { atanf(y / x) } else { M_PI_2 as f32 };
    }
    if x < 0.0 {
        return M_PI as f32 + atanf(y / x);
    }
    if x != 0.0 {
        return atanf(y / x);
    }
    f32::from_bits(sy.wrapping_add(BITWISE_PI_2))
}

/// The float Newton step of lbtrigf.c: `0.5f * g * (3.0f - g*g*v)`; the C build fuses
/// `3.0f - (g*g)*v` into one fmadds.
#[inline]
fn nr_step_f(g: f32, v: f32) -> f32 {
    (0.5f32 * g) * fmadds(-(g * g), v, 3.0)
}

/// `acosf`.
pub fn acosf(x: f32) -> f32 {
    let mut result = fmadds(-x, x, 1.0); // 1.0F - x * x, fused
    if result > 0.0 {
        let mut guess = frsqrte(result as f64) as f32;
        guess = nr_step_f(guess, result);
        guess = nr_step_f(guess, result);
        guess = nr_step_f(guess, result);
        result = guess;
    } else if result != 0.0 {
        result = f32::from_bits(NAN);
    } else {
        result = f32::from_bits(INF);
    }
    M_PI_2 as f32 - atanf(x * result)
}

/// `lb_sqrtf` (static): despite the name it returns the refined reciprocal square root.
fn lb_sqrtf(x: f32) -> f32 {
    if x > 0.0 {
        let mut guess = frsqrte(x as f64) as f32;
        guess = nr_step_f(guess, x);
        guess = nr_step_f(guess, x);
        guess = nr_step_f(guess, x);
        return guess;
    }
    if x != 0.0 {
        return f32::from_bits(NAN);
    }
    f32::from_bits(INF)
}

/// `asinf`: `atanf(x * lb_sqrtf(-(x * x - 1.0f)))`, the inner `x*x - 1` fused.
pub fn asinf(x: f32) -> f32 {
    atanf(x * lb_sqrtf(-fmadds(x, x, -1.0)))
}
