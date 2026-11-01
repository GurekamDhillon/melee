//! lbcardnew.c: ports folded from the hand-written modules (consolidation steps 21/22).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use melee_types as mt;
// ---- folded from lb.rs (consolidation steps 21/22) ----
/// lbcardnew.c `convertSdkResult_fp` (CARD_RESULT_* -> lbCardResult).
pub fn convertSdkResult_fp(sdk_result: i32) -> i32 {
    melee_core::ct!("convertSdkResult_fp");
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

// ---- Round 2 ----

/// lbcardnew.c `convertHsdResult_fp`: HSD memcard result -> LbCardResult, else the SDK mapping.
pub fn convertHsdResult_fp(hsd_result: i32) -> i32 {
    melee_core::ct!("convertHsdResult_fp");
    match hsd_result {
        0 | 1 => 0,
        -261..=-258 => 2,
        -263 | -262 | -257 => 3,
        -267..=-264 => 10,
        _ => convertSdkResult_fp(hsd_result),
    }
}

// ---- folded from lb_ptr.rs (consolidation steps 21/22) ----
/// lbcardnew.c `readCardFileSize_fp` (a `volatile` read).
pub fn readCardFileSize_fp(file_size: &i32) -> i32 {
    melee_core::ct!("readCardFileSize_fp");
    // SAFETY: a reference is valid for reads.
    unsafe { core::ptr::read_volatile(file_size) }
}
