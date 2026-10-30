//! `src/melee/lb/` leaf functions group 1 did not port (lbtime, lb_00CE, lbsnap, lbaudio_ax,
//! lbcardnew, lbcollision, lbdvd, lb_00F9 helpers). Hand ports, same names as the C, checked
//! by `tests/oracle.rs`. TODO(group1): move into melee_runtime when group 1 ports these TUs.
#![allow(non_snake_case)]

use melee_core::vec::Vec3;
use melee_runtime::cabi;

pub use melee_runtime::tu::lbaudio_ax::calcPan;

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

pub use melee_runtime::tu::lbcollision::isSmall;

/// lbdvd.c `same`.
pub fn same(a: i32, b: i32) -> i32 {
    (a == b) as i32
}

pub use melee_runtime::tu::lbsnap::RGB565_TO_RGB5A3;

pub use melee_runtime::tu::lbsnap::lbSnap_8001E204;

pub use melee_runtime::tu::lbsnap::lbSnap_8001E210;

pub use melee_runtime::tu::lbtime::lbTime_8000AEC8;

pub use melee_runtime::tu::lbtime::lbTime_8000AEE4;

pub use melee_runtime::tu::lbtime::lbTime_8000AF24;

pub use melee_runtime::tu::lbtime::lbTime_8000AF74;

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

pub use melee_runtime::tu::lb_00F9::absf;

// ---- Round 2 ----

/// lbcardnew.c `convertHsdResult`: HSD memcard result -> LbCardResult, else the SDK mapping.
pub fn convertHsdResult(hsd_result: i32) -> i32 {
    match hsd_result {
        0 | 1 => 0,
        -261..=-258 => 2,
        -263 | -262 | -257 => 3,
        -267..=-264 => 10,
        _ => convertSdkResult(hsd_result),
    }
}

/// lb_00CE.c `powf`: `expf(y * 2 * atanh-series((x-1)/(x+1)))`, all f32 (no fused site). Like
/// `expf` it need not terminate; the oracle keeps x > 0 and |y ln x| <= 12.
pub fn powf(arg0: f32, arg1: f32) -> f32 {
    if arg0 == 0.0 {
        return 0.0;
    }
    let mut k: i32 = 1;
    let mut term = (arg0 - 1.0) / (1.0 + arg0);
    let mut sum = term;
    let sq = term * term;
    loop {
        k += 2;
        term *= sq;
        let prev = sum;
        sum += term / k as f32;
        if sum == prev {
            break;
        }
    }
    expf(arg1 * (2.0 * sum))
}

pub use melee_runtime::tu::lb_00CE::lb_8000D008;
