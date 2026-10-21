//! `src/melee/lb/` leaf functions group 1 did not port (lbtime, lb_00CE, lbsnap, lbaudio_ax,
//! lbcardnew, lbcollision, lbdvd, lb_00F9 helpers). Hand ports, same names as the C, checked
//! by `tests/oracle.rs`. TODO(group1): move into melee_runtime when group 1 ports these TUs.
#![allow(non_snake_case)]

use melee_core::vec::Vec3;
use melee_runtime::cabi;

/// lbaudio_ax.c `calcPan`: linear pan from left to right over `end` steps, clamped to 0..127.
/// All f32 operations (no fused site); `end == 0` gives NaN -> 0 via gwtool's saturation.
pub fn calcPan(mut current: i32, end: i32, left: i32, right: i32) -> i32 {
    if current > end {
        current = end;
    }
    if left != right {
        let mut result = if left <= right {
            left as f32 + current as f32 * (right as f32 - left as f32) / end as f32
        } else {
            right as f32 + current as f32 * (left as f32 - right as f32) / end as f32
        };
        if result < 0.0 {
            result = 0.0;
        }
        if result > 127.0 {
            result = 127.0;
        }
        return cabi::f2i32(result as f64);
    }
    64
}

/// lbcardnew.c `convertSdkResult` (CARD_RESULT_* -> lbCardResult).
pub fn convertSdkResult(sdk_result: i32) -> i32 {
    const READY: i32 = 0;
    const NO_FILE: i32 = 4;
    const MALFORMED: i32 = 9;
    const INVALID: i32 = 13;
    const FATAL: i32 = 14;
    const DEVICE: i32 = 15;
    match sdk_result {
        -1 | -2 | -3 => DEVICE,  // BUSY, WRONGDEVICE, NOCARD
        -4 => NO_FILE,           // NOFILE
        0 => READY,              // READY
        -5 | -128 => FATAL,      // IOERROR, FATAL_ERROR
        -6 | -13 => MALFORMED,   // BROKEN, ENCODING
        _ => INVALID,            // UNLOCKED, CANCELED, default
    }
}

/// lbcollision.c `isSmall`.
pub fn isSmall(x: f32) -> bool {
    x < 0.00001 && x > -0.00001
}

/// lbdvd.c `same`.
pub fn same(a: i32, b: i32) -> i32 {
    (a == b) as i32
}

/// lbsnap.c `RGB565_TO_RGB5A3`.
pub fn RGB565_TO_RGB5A3(pixel: u16) -> u16 {
    let mut result = (pixel >> 1) & (0x7C00 | 0x03E0);
    result |= pixel & 0x001F;
    result | 0x8000
}

/// lbsnap.c `lbSnap_8001E204`.
pub fn lbSnap_8001E204() -> i32 {
    0x3E840
}

/// lbsnap.c `lbSnap_8001E210`.
pub fn lbSnap_8001E210() -> i32 {
    0x840
}

/// lbtime.c `lbTime_8000AEC8`: saturating u32 add.
pub fn lbTime_8000AEC8(a: u32, b: u32) -> u32 {
    if u32::MAX - a > b {
        a.wrapping_add(b)
    } else {
        u32::MAX
    }
}

/// lbtime.c `lbTime_8000AEE4`: u32 + signed, saturating at 0 and u32::MAX.
pub fn lbTime_8000AEE4(a: u32, b: i32) -> u32 {
    if b > 0 {
        if u32::MAX - a > b as u32 {
            a.wrapping_add(b as u32)
        } else {
            u32::MAX
        }
    } else if a > b.wrapping_neg() as u32 {
        a.wrapping_add(b as u32)
    } else {
        0
    }
}

/// lbtime.c `lbTime_8000AF24`: the same on the low 16 bits (`c - ab` is an unsigned compare).
pub fn lbTime_8000AF24(a: u32, b: i32) -> u32 {
    if b > 0 {
        let ab = a & 0xFFFF;
        let c: i32 = 0xFFFF;
        if (c as u32).wrapping_sub(ab) > b as u32 {
            a.wrapping_add(b as u32)
        } else {
            c as u32
        }
    } else {
        let c = (a & 0xFFFF) as i32;
        if c > b.wrapping_neg() {
            a.wrapping_add(b as u32)
        } else {
            0
        }
    }
}

/// lbtime.c `lbTime_8000AF74`: the same on the low 8 bits.
pub fn lbTime_8000AF74(a: u32, b: i32) -> u32 {
    if b > 0 {
        let lo = 0xFF - (a & 0xFF);
        return if lo > b as u32 { a.wrapping_add(b as u32) } else { 0xFF };
    }
    let c = (a & 0xFF) as i32;
    if c > b.wrapping_neg() {
        a.wrapping_add(b as u32)
    } else {
        0
    }
}

/// lb_00CE.c `expf`: Taylor series until the f32 sum stops changing, all f32 (no fused site).
/// Like the C, it never returns for some inputs (68 multiples of 1/16 in 12.625..14.6875 in
/// magnitude spin forever, the sum oscillating in the last bit); and NaN never terminates.
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
        n += 1;
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

/// lb_00CE.c `powi`: integer power by repeated (wrapping) multiplication; 0 for a negative
/// exponent or base 0.
pub fn powi(base: i32, exponent: i32) -> i32 {
    if base == 0 {
        return 0;
    }
    if exponent >= 0 {
        let mut result: i32 = 1;
        for _ in 0..exponent {
            result = result.wrapping_mul(base);
        }
        return result;
    }
    0
}

/// lb_00F9.c `approximatelyZeroVec3`.
pub fn approximatelyZeroVec3(vec: Vec3) -> bool {
    !(vec.x >= 0.00001 || vec.x <= -0.00001 || vec.y >= 0.00001 || vec.y <= -0.00001
        || vec.z >= 0.00001 || vec.z <= -0.00001)
}

/// lb_00F9.c `absf` (`-0.0` stays `-0.0`; NaN unchanged).
pub fn absf(x: f32) -> f32 {
    if x < 0.0 {
        -x
    } else {
        x
    }
}
