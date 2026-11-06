//! Gekko integer semantics where Rust's differ (notes/int-audit.md). The C build gets the same behaviour from gwtool's
//! `makeArithmeticSafe` (pc/tools/gwtool/gwtool.cpp); every Rust site the int_audit scanner lists goes through here.
//!
//! - `divw` / `divwu` never trap. Gekko (and Dolphin, which models the hardware): `divw` by 0 or `INT_MIN / -1`
//!   gives `-1` when the dividend is negative, else `0`; `divwu` by 0 gives `0`. MWCC lowers `%` to
//!   `divw q; mullw t,q,b; subf r,t,a`, so `x % 0 = x` and `INT_MIN % -1 = INT_MIN - 1 = INT_MAX` (and `x %u 0 = x`).
//! - `slw` / `srw` / `sraw` take a 6-bit amount: 32..63 shifts everything out (0, or all sign bits for `sraw`).
//!   Rust masks the amount to 5 bits (and a debug build would panic).
//! - `fctiwz`: saturating, and NaN gives `0x8000_0000`; Rust `as` gives 0 for NaN. Narrow targets (`(s16) f`,
//!   `(u8) f`) are `fctiwz` then truncation, not a saturating narrow cast.
//! - `(u32) f` is MWCC's `__cvt_fp2unsigned`: saturating, negative and NaN give 0, the same as Rust `as u32`.

/// Gekko division, remainder and shift semantics on the integer widths the game uses.
pub trait Ppc: Copy {
    /// `a / b`, `divw`/`divwu` semantics (no trap).
    fn pdiv(self, b: Self) -> Self;
    /// `a % b` as divw+mullw+subf (`x % 0 = x`, `MIN % -1 = MAX`).
    fn prem(self, b: Self) -> Self;
    /// `a << n` with `slw` semantics: amounts >= the width give 0.
    fn pshl(self, n: i64) -> Self;
    /// `a >> n` with `srw`/`sraw` semantics: amounts >= the width give 0 (unsigned) or all sign bits (signed).
    fn pshr(self, n: i64) -> Self;
}

macro_rules! signed {
    ($($t:ty),*) => {$(
        impl Ppc for $t {
            #[inline]
            #[cfg_attr(feature = "disc", track_caller)]
            fn pdiv(self, b: Self) -> Self {
                let r = if b == 0 || (self == <$t>::MIN && b == -1) { if self < 0 { -1 } else { 0 } } else { self / b };
                #[cfg(feature = "disc")]
                crate::disc::divrem(false, self as i64, b as i64, <$t>::MIN as i64, r as i64);
                r
            }
            #[inline]
            #[cfg_attr(feature = "disc", track_caller)]
            fn prem(self, b: Self) -> Self {
                let r = if b == 0 { self } else if self == <$t>::MIN && b == -1 { <$t>::MAX } else { self % b };
                #[cfg(feature = "disc")]
                crate::disc::divrem(true, self as i64, b as i64, <$t>::MIN as i64, r as i64);
                r
            }
            #[inline]
            #[cfg_attr(feature = "disc", track_caller)]
            fn pshl(self, n: i64) -> Self {
                let r = if (n as u64) >= <$t>::BITS as u64 { 0 } else { self << n };
                #[cfg(feature = "disc")]
                crate::disc::shift(true, self as i64, n, <$t>::BITS, r as i64, self.wrapping_shl(n as u32) as i64, <$t>::MIN as i64);
                r
            }
            #[inline]
            #[cfg_attr(feature = "disc", track_caller)]
            fn pshr(self, n: i64) -> Self {
                let r = if (n as u64) >= <$t>::BITS as u64 { self >> (<$t>::BITS - 1) } else { self >> n };
                #[cfg(feature = "disc")]
                crate::disc::shift(false, self as i64, n, <$t>::BITS, r as i64, self.wrapping_shr(n as u32) as i64, <$t>::MIN as i64);
                r
            }
        }
    )*};
}
macro_rules! unsigned {
    ($($t:ty),*) => {$(
        impl Ppc for $t {
            #[inline]
            #[cfg_attr(feature = "disc", track_caller)]
            fn pdiv(self, b: Self) -> Self {
                let r = if b == 0 { 0 } else { self / b };
                #[cfg(feature = "disc")]
                crate::disc::divrem(false, self as i64, b as i64, 0, r as i64);
                r
            }
            #[inline]
            #[cfg_attr(feature = "disc", track_caller)]
            fn prem(self, b: Self) -> Self {
                let r = if b == 0 { self } else { self % b };
                #[cfg(feature = "disc")]
                crate::disc::divrem(true, self as i64, b as i64, 0, r as i64);
                r
            }
            #[inline]
            #[cfg_attr(feature = "disc", track_caller)]
            fn pshl(self, n: i64) -> Self {
                let r = if (n as u64) >= <$t>::BITS as u64 { 0 } else { self << n };
                #[cfg(feature = "disc")]
                crate::disc::shift(true, self as i64, n, <$t>::BITS, r as i64, self.wrapping_shl(n as u32) as i64, 0);
                r
            }
            #[inline]
            #[cfg_attr(feature = "disc", track_caller)]
            fn pshr(self, n: i64) -> Self {
                let r = if (n as u64) >= <$t>::BITS as u64 { 0 } else { self >> n };
                #[cfg(feature = "disc")]
                crate::disc::shift(false, self as i64, n, <$t>::BITS, r as i64, self.wrapping_shr(n as u32) as i64, 0);
                r
            }
        }
    )*};
}
signed!(i32, i64, isize);
unsigned!(u32, u64, usize);

// i8/i16/u8/u16 operands are promoted to `int` in C: divide/shift as i32, then truncate.
macro_rules! narrow {
    ($($t:ty),*) => {$(
        impl Ppc for $t {
            #[inline]
            #[cfg_attr(feature = "disc", track_caller)]
            fn pdiv(self, b: Self) -> Self { (self as i32).pdiv(b as i32) as $t }
            #[inline]
            #[cfg_attr(feature = "disc", track_caller)]
            fn prem(self, b: Self) -> Self { (self as i32).prem(b as i32) as $t }
            #[inline]
            #[cfg_attr(feature = "disc", track_caller)]
            fn pshl(self, n: i64) -> Self { (self as i32).pshl(n) as $t }
            #[inline]
            #[cfg_attr(feature = "disc", track_caller)]
            fn pshr(self, n: i64) -> Self { (self as i32).pshr(n) as $t }
        }
    )*};
}
narrow!(i8, i16, u8, u16);

/// `fctiwz`: truncate toward zero, saturate, NaN -> `i32::MIN` (0x8000_0000).
#[inline]
#[cfg_attr(feature = "disc", track_caller)]
pub fn fctiwz(x: f64) -> i32 {
    let r = if x.is_nan() { i32::MIN } else { x as i32 };
    #[cfg(feature = "disc")]
    crate::disc::cvt(x, r);
    r
}
/// `(s32) f` / `(int) f` for an f32.
#[inline]
#[cfg_attr(feature = "disc", track_caller)]
pub fn fctiwz_f32(x: f32) -> i32 {
    fctiwz(x as f64)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn gekko() {
        assert_eq!(5i32.pdiv(0), 0);
        assert_eq!((-5i32).pdiv(0), -1);
        assert_eq!(i32::MIN.pdiv(-1), -1);
        assert_eq!(7u32.pdiv(0), 0);
        assert_eq!(7i32.prem(0), 7);
        assert_eq!(i32::MIN.prem(-1), i32::MAX);
        assert_eq!(1u32.pshl(32), 0);
        assert_eq!(1u32.pshl(31), 1 << 31);
        assert_eq!((-8i32).pshr(40), -1);
        assert_eq!(8i32.pshr(40), 0);
        assert_eq!(8u32.pshr(-1), 0);
        assert_eq!((-128i8).pdiv(-1), -128);
        assert_eq!(0x80u8.pshr(9), 0);
        assert_eq!((-128i8).pshr(9), -1);
        assert_eq!(fctiwz(f64::NAN), i32::MIN);
        assert_eq!(fctiwz(1e20), i32::MAX);
        assert_eq!(fctiwz(-1e20), i32::MIN);
        assert_eq!(fctiwz_f32(-2.9), -2);
    }
}
