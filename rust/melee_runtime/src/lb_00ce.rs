//! Port of the series math in `src/melee/lb/lb_00CE.c`: `expf`, `powf`, `powi` (the game's own
//! definitions; the shipping build links these, not a CRT).
//!
//! Float rule: `fma_sites.py` finds no fused site in these three (each step is a separate
//! statement or a division). Both series loop until the partial sum stops changing; like the C,
//! `expf(NaN)` and inputs whose terms reach `inf / inf` never terminate.
//!
//! Not here: `lb_8000D008` / `lb_8000D148` (ported by group 6 in `melee_modes::lb`).

/// `expf`: Taylor series of e^|x|, inverted for negative x.
pub fn expf(arg8: f32) -> f32 {
    let mut x = arg8;
    let mut n: i32 = 2;
    let mut fact = 1.0f32;
    let neg = if x < 0.0 {
        x = -x;
        true
    } else {
        false
    };
    let mut term = x;
    let mut sum = 1.0f32 + x;
    loop {
        let k = n;
        term *= x;
        let prev = sum;
        n = n.wrapping_add(1);
        fact *= k as f32;
        sum += term / fact;
        if sum == prev {
            break;
        }
    }
    if neg {
        sum = 1.0 / sum;
    }
    sum
}

/// `powf`: `exp(y * 2*atanh((x-1)/(x+1)))`, 0 for x == 0.
pub fn powf(arg0: f32, arg1: f32) -> f32 {
    if arg0 == 0.0 {
        return 0.0;
    }
    let mut k: i32 = 1;
    let mut t = (arg0 - 1.0) / (1.0 + arg0);
    let mut s = t;
    let t2 = t * t;
    loop {
        k = k.wrapping_add(2);
        t *= t2;
        let prev = s;
        s += t / k as f32;
        if s == prev {
            break;
        }
    }
    expf(arg1 * (2.0 * s))
}

/// `powi`: integer power with wrapping multiply; 0 for base 0 or a negative exponent.
pub fn powi(base: i32, exponent: i32) -> i32 {
    if base == 0 {
        return 0;
    }
    if exponent >= 0 {
        let mut r: i32 = 1;
        for _ in 0..exponent {
            r = r.wrapping_mul(base);
        }
        return r;
    }
    0
}
