//! Bit-exact Gekko estimate instructions, ported from pc/gameworld/gekko_fp.c
//! (table and algorithm from Dolphin, GPL-2.0-or-later; see pc/LICENSE).
//!
//! TODO(move-to-core): belongs in melee_core::fp next to fmadds; kept here so group 2 does not
//! edit melee_core while other groups port concurrently.

const FRSQRTE_EXPECTED: [(i64, i64); 32] = [
    (0x1a7e800, -0x568), (0x17cb800, -0x4f3), (0x1552800, -0x48d),
    (0x130c000, -0x435), (0x10f2000, -0x3e7), (0x0eff000, -0x3a2),
    (0x0d2e000, -0x365), (0x0b7c000, -0x32e), (0x09e5000, -0x2fc),
    (0x0867000, -0x2d0), (0x06ff000, -0x2a8), (0x05ab800, -0x283),
    (0x046a000, -0x261), (0x0339800, -0x243), (0x0218800, -0x226),
    (0x0105800, -0x20b), (0x3ffa000, -0x7a4), (0x3c29000, -0x700),
    (0x38aa000, -0x670), (0x3572000, -0x5f2), (0x3279000, -0x584),
    (0x2fb7000, -0x524), (0x2d26000, -0x4cc), (0x2ac0000, -0x47e),
    (0x2881000, -0x43a), (0x2665000, -0x3fa), (0x2468000, -0x3c2),
    (0x2287000, -0x38e), (0x20c1000, -0x35e), (0x1f12000, -0x332),
    (0x1d79000, -0x30a), (0x1bf4000, -0x2e6),
];

const EXP_MASK: i64 = 0x7FF << 52;
const MANTISSA_MASK: i64 = (1 << 52) - 1;
const SIGN_BIT: u64 = 1 << 63;
const QUIET_BIT: u64 = 1 << 51;

/// `__frsqrte` (Gekko `frsqrte`).
pub fn frsqrte(val: f64) -> f64 {
    let mut integral = val.to_bits() as i64;
    let mut mantissa = integral & MANTISSA_MASK;
    let sign = integral & SIGN_BIT as i64;
    let mut exponent = integral & EXP_MASK;

    if mantissa == 0 && exponent == 0 {
        return f64::from_bits(if sign != 0 { SIGN_BIT | EXP_MASK as u64 } else { EXP_MASK as u64 });
    }
    if exponent == EXP_MASK {
        if mantissa == 0 {
            return if sign != 0 { f64::from_bits(EXP_MASK as u64 | QUIET_BIT) } else { 0.0 };
        }
        return f64::from_bits(val.to_bits() | QUIET_BIT);
    }
    if sign != 0 {
        return f64::from_bits(EXP_MASK as u64 | QUIET_BIT);
    }
    if exponent == 0 {
        loop {
            exponent -= 1 << 52;
            mantissa <<= 1;
            if mantissa & (1 << 52) != 0 {
                break;
            }
        }
        mantissa &= MANTISSA_MASK;
        exponent += 1 << 52;
    }
    let exponent_lsb = exponent & (1 << 52);
    exponent = ((0x3FF_i64 << 52) - ((exponent - (0x3FE_i64 << 52)) / 2)) & EXP_MASK;
    integral = sign | exponent;
    let i = ((exponent_lsb | mantissa) >> 37) as i32;
    let index = (i / 2048) as usize;
    let (base, dec) = FRSQRTE_EXPECTED[index];
    integral |= (base + dec * (i % 2048) as i64) << 26;
    f64::from_bits(integral as u64)
}

/// Dolphin's Force25Bit (pc/gameworld/mtx_pc.c `gekko_force25`).
pub fn force25(d: f64) -> f64 {
    let u = d.to_bits();
    f64::from_bits((u & 0xFFFF_FFFF_F800_0000) + (u & 0x800_0000))
}
