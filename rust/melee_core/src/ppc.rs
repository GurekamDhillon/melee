//! Gekko integer semantics where Rust's differ (notes/int-audit.md). The C build gets the same behaviour from gwtool's
//! `makeArithmeticSafe` (pc/tools/gwtool/gwtool.cpp); every Rust site the int_audit scanner lists goes through here.
//!
//! - `divw` / `divwu` never trap. Gekko (and Dolphin, which models the hardware): `divw` by 0 or `INT_MIN / -1`
//!   gives `-1` when the dividend is negative, else `0`; `divwu` by 0 gives `0`. MWCC lowers `%` to
//!   `divw q; mullw t,q,b; subf r,t,a`, so `x % 0 = x` and `INT_MIN % -1 = INT_MIN - 1 = INT_MAX` (and `x %u 0 = x`).
//! - `slw` / `srw` / `sraw` use only the low 6 bits of rB (Dolphin Interpreter_Integer.cpp: `amount & 0x20` selects
//!   the shifted-out result, `amount & 0x1f` the shift): 32..63 give 0 (or all sign bits for `sraw`), 64 shifts by 0,
//!   96 gives 0, and a negative amount is its two's-complement low 6 bits (-1 -> 63 -> 0, -33 -> 31).
//!   Rust masks the amount to 5 bits (and a debug build would panic). 64-bit operands are not slw/srw (MWCC calls
//!   its runtime shift helpers), so i64/u64/isize/usize keep "amount >= 64 gives 0 / sign fill".
//! - `fctiwz`: saturating, and NaN gives `0x8000_0000`; Rust `as` gives 0 for NaN. Narrow targets (`(s16) f`,
//!   `(u8) f`) are `fctiwz` then truncation, not a saturating narrow cast.
//! - `(u32) f` is MWCC's `__cvt_fp2unsigned`: saturating, negative and NaN give 0, the same as Rust `as u32`.

/// Gekko division, remainder and shift semantics on the integer widths the game uses.
pub trait Ppc: Copy {
    /// `a / b`, `divw`/`divwu` semantics (no trap).
    fn pdiv(self, b: Self) -> Self;
    /// `a % b` as divw+mullw+subf (`x % 0 = x`, `MIN % -1 = MAX`).
    fn prem(self, b: Self) -> Self;
    /// `a << n` with `slw` semantics: the low 6 bits of `n`; 32..63 give 0.
    fn pshl(self, n: i64) -> Self;
    /// `a >> n` with `srw`/`sraw` semantics: the low 6 bits of `n`; 32..63 give 0 (unsigned) or all sign bits (signed).
    fn pshr(self, n: i64) -> Self;
}

macro_rules! signed {
    ($($t:ty),*) => {$(
        impl Ppc for $t {
            #[inline]
            fn pdiv(self, b: Self) -> Self {
                if b == 0 || (self == <$t>::MIN && b == -1) { if self < 0 { -1 } else { 0 } } else { self / b }
            }
            #[inline]
            fn prem(self, b: Self) -> Self {
                if b == 0 { self } else if self == <$t>::MIN && b == -1 { <$t>::MAX } else { self % b }
            }
            #[inline]
            fn pshl(self, n: i64) -> Self {
                if (n as u64) >= <$t>::BITS as u64 { 0 } else { self << n }
            }
            #[inline]
            fn pshr(self, n: i64) -> Self {
                if (n as u64) >= <$t>::BITS as u64 { self >> (<$t>::BITS - 1) } else { self >> n }
            }
        }
    )*};
}
macro_rules! unsigned {
    ($($t:ty),*) => {$(
        impl Ppc for $t {
            #[inline]
            fn pdiv(self, b: Self) -> Self { if b == 0 { 0 } else { self / b } }
            #[inline]
            fn prem(self, b: Self) -> Self { if b == 0 { self } else { self % b } }
            #[inline]
            fn pshl(self, n: i64) -> Self {
                if (n as u64) >= <$t>::BITS as u64 { 0 } else { self << n }
            }
            #[inline]
            fn pshr(self, n: i64) -> Self {
                if (n as u64) >= <$t>::BITS as u64 { 0 } else { self >> n }
            }
        }
    )*};
}
signed!(i64, isize);
unsigned!(u64, usize);

impl Ppc for i32 {
    #[inline]
    fn pdiv(self, b: Self) -> Self {
        if b == 0 || (self == i32::MIN && b == -1) { if self < 0 { -1 } else { 0 } } else { self / b }
    }
    #[inline]
    fn prem(self, b: Self) -> Self {
        if b == 0 { self } else if self == i32::MIN && b == -1 { i32::MAX } else { self % b }
    }
    /// slw: `rB & 0x20` -> 0, else shift by `rB & 0x1f`.
    #[inline]
    fn pshl(self, n: i64) -> Self { (self as u32).pshl(n) as i32 }
    /// sraw: `rB & 0x20` -> all sign bits, else arithmetic shift by `rB & 0x1f`.
    #[inline]
    fn pshr(self, n: i64) -> Self {
        let rb = n as u32;
        if rb & 0x20 != 0 { self >> 31 } else { self >> (rb & 0x1f) }
    }
}
impl Ppc for u32 {
    #[inline]
    fn pdiv(self, b: Self) -> Self { if b == 0 { 0 } else { self / b } }
    #[inline]
    fn prem(self, b: Self) -> Self { if b == 0 { self } else { self % b } }
    #[inline]
    fn pshl(self, n: i64) -> Self {
        let rb = n as u32;
        if rb & 0x20 != 0 { 0 } else { self << (rb & 0x1f) }
    }
    /// srw.
    #[inline]
    fn pshr(self, n: i64) -> Self {
        let rb = n as u32;
        if rb & 0x20 != 0 { 0 } else { self >> (rb & 0x1f) }
    }
}

// i8/i16/u8/u16 operands are promoted to `int` in C: divide/shift as i32, then truncate.
macro_rules! narrow {
    ($($t:ty),*) => {$(
        impl Ppc for $t {
            #[inline]
            fn pdiv(self, b: Self) -> Self { (self as i32).pdiv(b as i32) as $t }
            #[inline]
            fn prem(self, b: Self) -> Self { (self as i32).prem(b as i32) as $t }
            #[inline]
            fn pshl(self, n: i64) -> Self { (self as i32).pshl(n) as $t }
            #[inline]
            fn pshr(self, n: i64) -> Self { (self as i32).pshr(n) as $t }
        }
    )*};
}
narrow!(i8, i16, u8, u16);

/// `fctiwz`: truncate toward zero, saturate, NaN -> `i32::MIN` (0x8000_0000).
#[inline]
pub fn fctiwz(x: f64) -> i32 {
    if x.is_nan() { i32::MIN } else { x as i32 }
}
/// `(s32) f` / `(int) f` for an f32.
#[inline]
pub fn fctiwz_f32(x: f32) -> i32 {
    fctiwz(x as f64)
}

/// `(u32) f`: MWCC's `__cvt_fp2unsigned` saturates, negative and NaN give 0, which is Rust's `as u32` (int-audit.md).
/// The helpers exist so every float->int conversion in game code is explicit (semantics_lint I3). They take f32 or f64:
/// widening an f32 to f64 is exact, so the result is the same either way.
#[inline]
pub fn fcvt_u32(x: impl Into<f64>) -> u32 {
    x.into() as u32
}
/// `(u64) f`: kept as Rust's saturating `as u64` (NaN 0). Not checked against MWCC's 64-bit runtime conversion.
#[inline]
pub fn fcvt_u64(x: impl Into<f64>) -> u64 {
    x.into() as u64
}
/// `(s64) f`: kept as Rust's saturating `as i64` (NaN 0). Not checked against MWCC's `__cvt_fp2signed` (int-audit.md).
#[inline]
pub fn fcvt_i64(x: impl Into<f64>) -> i64 {
    x.into() as i64
}
/// `(size_t) f`: 32-bit unsigned on the console (`__cvt_fp2unsigned`), as [`fcvt_u32`], widened.
#[inline]
pub fn fcvt_usize(x: impl Into<f64>) -> usize {
    x.into() as u32 as usize
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
    }
    /// slw/srw/sraw: low 6 bits of rB (Dolphin Interpreter::slwx/srwx/srawx).
    #[test]
    fn shift_six_bit_amount() {
        assert_eq!(1u32.pshl(64), 1);
        assert_eq!(1u32.pshl(65), 2);
        assert_eq!(1u32.pshl(96), 0);
        assert_eq!(5i32.pshl(63), 0);
        assert_eq!(0x80u32.pshr(64), 0x80);
        assert_eq!(0x80u32.pshr(71), 1);
        assert_eq!((-8i32).pshr(64), -8);
        assert_eq!((-8i32).pshr(65), -4);
        assert_eq!((-8i32).pshr(32), -1);
        assert_eq!(8i32.pshr(63), 0);
        // negative amounts: two's-complement low 6 bits
        assert_eq!(1u32.pshl(-1), 0); // 0x3f
        assert_eq!(1u32.pshl(-33), 1 << 31); // 0x1f
        assert_eq!(1u32.pshl(-64), 1); // 0x00
        assert_eq!((-8i32).pshr(-1), -1);
        assert_eq!(i32::MIN.pshr(-33), -1);
        assert_eq!((-128i8).pshr(64), -128);
        assert_eq!(1u16.pshl(64), 1);
        assert_eq!(1u64.pshl(64), 0); // 64-bit: runtime helper, unchanged
        assert_eq!(fcvt_u32(-1.5f32), 0);
        assert_eq!(fcvt_u32(f64::NAN), 0);
        assert_eq!(fcvt_u32(5e9f32), u32::MAX);
        assert_eq!(fcvt_u32(7.9f64), 7);
        assert_eq!(fcvt_usize(-3.0f32), 0);
        assert_eq!(fctiwz(1e20), i32::MAX);
        assert_eq!(fctiwz(-1e20), i32::MIN);
        assert_eq!(fctiwz_f32(-2.9), -2);
    }
}
