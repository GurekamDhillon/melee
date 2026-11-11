//! Fixed-point scalars in the file's own unit: 1/60000. A script scalar is stored as `raw` with value `raw / 60000`,
//! so keeping that raw integer is exact. Floats from the file's data blocks (f32 bits) are converted once, at decode,
//! by integer arithmetic (round half away from zero) so nothing in the runner ever touches a float.

use core::fmt;
use core::ops::{Add, Neg, Sub};

/// One unit of value is `ONE` raw.
pub const ONE: i64 = 60_000;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Fx(pub i64);

impl Fx {
    pub const ZERO: Fx = Fx(0);
    pub const fn raw(self) -> i64 {
        self.0
    }
    pub const fn from_raw(raw: i64) -> Fx {
        Fx(raw)
    }
    pub const fn from_int(i: i64) -> Fx {
        Fx(i.saturating_mul(ONE))
    }
    /// value = num / den (exact when 60000·num/den is an integer; truncated toward zero otherwise).
    pub const fn ratio(num: i64, den: i64) -> Fx {
        Fx(num * ONE / den)
    }
    /// Truncate toward zero to an integer (the file's "integer fields are truncated").
    pub const fn trunc(self) -> i64 {
        self.0 / ONE
    }
    pub const fn trunc_fx(self) -> Fx {
        Fx(self.0 / ONE * ONE)
    }
    /// Product, truncated toward zero in the last raw unit.
    pub fn mul(self, o: Fx) -> Fx {
        Fx(((self.0 as i128 * o.0 as i128) / ONE as i128).clamp(i64::MIN as i128, i64::MAX as i128) as i64)
    }
    pub fn abs(self) -> Fx {
        Fx(self.0.saturating_abs())
    }
    /// Exact conversion of an IEEE single (bits) into 1/60000 units, round half away from zero. NaN gives 0, and
    /// out-of-range values saturate. Integer-only, so it is identical on every machine.
    pub fn from_f32_bits(bits: u32) -> Fx {
        let neg = bits >> 31 != 0;
        let exp = ((bits >> 23) & 0xff) as i32;
        let frac = (bits & 0x7f_ffff) as u128;
        if exp == 0xff {
            return if frac != 0 { Fx(0) } else if neg { Fx(i64::MIN) } else { Fx(i64::MAX) };
        }
        let (m, e) = if exp == 0 { (frac, -149) } else { (frac | 0x80_0000, exp - 150) };
        // value = m * 2^e ; raw = round(m * 60000 * 2^e)
        let p = m * ONE as u128;
        let mag: u128 = if e >= 0 {
            if e > 60 {
                u128::MAX
            } else {
                p.checked_shl(e as u32).unwrap_or(u128::MAX)
            }
        } else {
            let s = (-e) as u32;
            if s >= 127 {
                0
            } else {
                (p + (1u128 << (s - 1))) >> s
            }
        };
        let mag = mag.min(i64::MAX as u128) as i64;
        Fx(if neg { -mag } else { mag })
    }
}

impl Add for Fx {
    type Output = Fx;
    fn add(self, o: Fx) -> Fx {
        Fx(self.0.saturating_add(o.0))
    }
}
impl Sub for Fx {
    type Output = Fx;
    fn sub(self, o: Fx) -> Fx {
        Fx(self.0.saturating_sub(o.0))
    }
}
impl Neg for Fx {
    type Output = Fx;
    fn neg(self) -> Fx {
        Fx(self.0.saturating_neg())
    }
}

impl fmt::Debug for Fx {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}
/// Decimal, exact (up to 5 fractional digits, trailing zeros dropped).
impl fmt::Display for Fx {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let neg = self.0 < 0;
        let a = self.0.unsigned_abs();
        let ip = a / ONE as u64;
        let fp = a % ONE as u64; // in 1/60000
        // print fp/60000 with 5 decimals (exact for multiples of 0.6e-? ; rounded otherwise)
        let dec = (fp * 100_000 + 30_000) / 60_000;
        let (ip, dec) = if dec >= 100_000 { (ip + 1, dec - 100_000) } else { (ip, dec) };
        if neg && (ip != 0 || dec != 0) {
            f.write_str("-")?;
        }
        write!(f, "{ip}")?;
        if dec != 0 {
            let mut d = dec;
            let mut w = 5;
            while d % 10 == 0 {
                d /= 10;
                w -= 1;
            }
            write!(f, ".{d:0w$}")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn f32_conversion() {
        assert_eq!(Fx::from_f32_bits(0.3f32.to_bits()), Fx(18_000));
        assert_eq!(Fx::from_f32_bits((-0.18f32).to_bits()), Fx(-10_800));
        assert_eq!(Fx::from_f32_bits(99999.0f32.to_bits()), Fx::from_int(99999));
        assert_eq!(Fx::from_f32_bits(0.0f32.to_bits()), Fx(0));
        assert_eq!(Fx::from_f32_bits(f32::NAN.to_bits()), Fx(0));
        assert_eq!(Fx::from_f32_bits(1.4f32.to_bits()), Fx(84_000));
    }
    #[test]
    fn arith() {
        assert_eq!(Fx::from_int(20).mul(Fx(18_000)), Fx::from_int(6));
        assert_eq!(Fx::ratio(154, 10).trunc(), 15);
        assert_eq!(Fx::ratio(-154, 10).trunc(), -15);
        assert_eq!(alloc::format!("{}", Fx::ratio(-27, 10)), "-2.7");
    }
}
