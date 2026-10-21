//! C-build arithmetic semantics that differ from plain Rust. The C build is clang + gwtool, and
//! gwtool's `makeArithmeticSafe` (pc/tools/gwtool/gwtool.cpp) redefines the cases C leaves
//! undefined so they behave like the Gekko. A port must use these where an operand can hit them.
//! TODO(move-to-core): every group needs these.

/// `(s32)f` / `(int)f`: `llvm.fptosi.sat.i32` (fctiwz): saturating, NaN -> 0. Same as Rust `as`.
#[inline(always)]
pub fn f2i32(f: f64) -> i32 {
    f as i32
}
/// `(u32)f`: `llvm.fptoui.sat.i32`: saturating, negative and NaN -> 0. Same as Rust `as`.
#[inline(always)]
pub fn f2u32(f: f64) -> u32 {
    f as u32
}
/// `(s16)f`, `(u16)f`, `(s8)f`, `(u8)f`: saturate to **i32**, then truncate. NOT Rust's `as i16`
/// (which saturates to the narrow type): `(u8)300.0f` is 44 in the C build.
#[inline(always)]
pub fn f2i16(f: f64) -> i16 {
    (f as i32) as i16
}
#[inline(always)]
pub fn f2u16(f: f64) -> u16 {
    (f as i32) as u16
}
#[inline(always)]
pub fn f2i8(f: f64) -> i8 {
    (f as i32) as i8
}
#[inline(always)]
pub fn f2u8(f: f64) -> u8 {
    (f as i32) as u8
}

/// Signed divide: `x / 0` = (x < 0 ? -1 : 0), `INT_MIN / -1` = (INT_MIN < 0 ? -1 : 0) = -1.
#[inline(always)]
pub fn sdiv(a: i32, d: i32) -> i32 {
    if d == 0 || (a == i32::MIN && d == -1) {
        if a < 0 {
            -1
        } else {
            0
        }
    } else {
        a / d
    }
}
/// Signed remainder: `x % 0` = x, `INT_MIN % -1` = 0.
#[inline(always)]
pub fn srem(a: i32, d: i32) -> i32 {
    if d == 0 {
        a
    } else if a == i32::MIN && d == -1 {
        0
    } else {
        a % d
    }
}
/// Unsigned divide: `x / 0` = 0.
#[inline(always)]
pub fn udiv(a: u32, d: u32) -> u32 {
    if d == 0 {
        0
    } else {
        a / d
    }
}
/// Unsigned remainder: `x % 0` = x.
#[inline(always)]
pub fn urem(a: u32, d: u32) -> u32 {
    if d == 0 {
        a
    } else {
        a % d
    }
}
/// Variable `<<` on 32 bits: amount >= 32 gives 0 (slw).
#[inline(always)]
pub fn shl(a: u32, n: u32) -> u32 {
    if n >= 32 {
        0
    } else {
        a << n
    }
}
/// Variable logical `>>`: amount >= 32 gives 0 (srw).
#[inline(always)]
pub fn lshr(a: u32, n: u32) -> u32 {
    if n >= 32 {
        0
    } else {
        a >> n
    }
}
/// Variable arithmetic `>>`: amount >= 32 fills with the sign (sraw).
#[inline(always)]
pub fn ashr(a: i32, n: u32) -> i32 {
    if n >= 32 {
        a >> 31
    } else {
        a >> n
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn narrow_float_casts_truncate_after_i32_saturation() {
        assert_eq!(f2u8(300.0), 44);
        assert_eq!(f2i16(40000.0), -25536);
        assert_eq!(f2i32(f64::NAN), 0);
        assert_eq!(f2i32(1e20), i32::MAX);
        assert_eq!(f2u32(-5.0), 0);
    }
    #[test]
    fn division_and_shifts_do_not_trap() {
        assert_eq!(sdiv(-7, 0), -1);
        assert_eq!(sdiv(7, 0), 0);
        assert_eq!(sdiv(i32::MIN, -1), -1);
        assert_eq!(srem(i32::MIN, -1), 0);
        assert_eq!(srem(9, 0), 9);
        assert_eq!(udiv(9, 0), 0);
        assert_eq!(urem(9, 0), 9);
        assert_eq!(shl(1, 32), 0);
        assert_eq!(ashr(-8, 40), -1);
    }
}
