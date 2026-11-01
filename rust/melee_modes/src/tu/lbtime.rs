//! lbtime.c: ports folded from the hand-written modules (consolidation steps 21/22).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use crate::lb::{same};

// ---- folded from lb.rs (consolidation steps 21/22) ----
/// lbtime.c `lbTime_8000AEC8_fp`: saturating u32 add.
pub fn lbTime_8000AEC8_fp(a: u32, b: u32) -> u32 {
    melee_core::ct!("lbTime_8000AEC8_fp");
    if u32::MAX - a > b {
        a.wrapping_add(b)
    } else {
        u32::MAX
    }
}

/// lbtime.c `lbTime_8000AEE4_fp`: u32 + signed, saturating at 0 and u32::MAX.
pub fn lbTime_8000AEE4_fp(a: u32, b: i32) -> u32 {
    melee_core::ct!("lbTime_8000AEE4_fp");
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

/// lbtime.c `lbTime_8000AF24_fp`: the same on the low 16 bits (`c - ab` is an unsigned compare).
pub fn lbTime_8000AF24_fp(a: u32, b: i32) -> u32 {
    melee_core::ct!("lbTime_8000AF24_fp");
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

/// lbtime.c `lbTime_8000AF74_fp`: the same on the low 8 bits.
pub fn lbTime_8000AF74_fp(a: u32, b: i32) -> u32 {
    melee_core::ct!("lbTime_8000AF74_fp");
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
