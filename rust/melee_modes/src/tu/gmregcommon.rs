//! Port of src/melee/gm/gmregcommon.c: helpers shared by the regular-match result screens.
//! `gm_8017BE84` / `gm_8017BE8C` were ported earlier (gm.rs, gm_ptr.rs; oracle-checked there) and
//! are re-exported here.
#![allow(non_snake_case, non_upper_case_globals)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use melee_types as mt;
#[allow(unused_imports)]
use crate::gm_ptr::{CHKIND_NONE};
use melee_types::ChKind_None;

pub use crate::gm::gm_8017BE84;

pub mod ext {
    crate::oext_fns! {
        /// gm_1601.c
        fn gm_GetNumCostumesForCKind(ckind: u8) -> u8 = crate::tu::adapt::gm_GetNumCostumesForCKind__a594a2;
        /// debug.c
        fn __assert(file: *const i8, line: u32, msg: *const i8) -> ();
        /// os (pc/platform): variadic in C; the only call here has no arguments after the format.
        fn OSReport(fmt: *const i8) -> ();
    }
}

crate::globals! {
    /// `u8 lbl_803D79F0[]`: the 24 character ids the result screens cycle through.
    ro pub lbl_803D79F0: [u8; 24] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 20, 21, 22, 24, 25, 33];
}

const NUM_OPPONENTS: usize = 3;

/// gm_8017BE8C: how many of the first three opponents are not ChKind_None.
pub unsafe fn gm_8017BE8C(opps: *const i8) -> i32 {
    melee_core::ct!("gm_8017BE8C");
    crate::gm_ptr::gm_8017BE8C(core::slice::from_raw_parts(opps, NUM_OPPONENTS))
}

/// gmRegSetupEnemyColorTable: give every opponent a costume colour that differs from `color` and
/// from the other opponents' (0xFF marks "needs a colour").
pub unsafe fn gmRegSetupEnemyColorTable(ckind: i8, color: u8, ckinds: *const i8, colors: *mut u8) {
    melee_core::ct!("gmRegSetupEnemyColorTable");
    let ncolors = ext::gm_GetNumCostumesForCKind(ckind as u8);
    let none = ChKind_None as i8;
    for i in 0..NUM_OPPONENTS {
        if *ckinds.add(i) != none && ckind == *ckinds.add(i) && color == *colors.add(i) {
            *colors.add(i) = 0xFF;
        }
    }
    for i in 0..NUM_OPPONENTS {
        if *ckinds.add(i) == none {
            continue;
        }
        if *colors.add(i) != 0xFF {
            continue;
        }
        for j in 0..ncolors as i32 {
            if color as i32 == j {
                continue;
            }
            let mut result = true;
            for k in 0..NUM_OPPONENTS {
                result = true;
                if i == k || (*ckinds.add(k) as u8) == (none as u8) {
                    continue;
                }
                if j == *colors.add(k) as i32 {
                    result = false;
                    break;
                }
            }
            if result {
                *colors.add(i) = j as u8;
                break;
            }
        }
        if *colors.add(i) == 0xFF {
            // HSD_ASSERTREPORT(0xDA, 0, "Error : not find color in gmRegSetupEnemyColorTable()\n")
            ext::OSReport(c"Error : not find color in gmRegSetupEnemyColorTable()\n".as_ptr());
            ext::__assert(c"gmregcommon.c".as_ptr(), 0xDA, c"0".as_ptr());
        }
    }
}

// ---- folded from gm.rs (consolidation steps 21/22) ----
/// gmregcommon.c `gm_8017BE84_fp`.
pub fn gm_8017BE84_fp(arg0: u32) -> i32 {
    melee_core::ct!("gm_8017BE84_fp");
    ((arg0 >> 3) & 0x1F) as i32
}

// ---- folded from gm_ptr.rs (consolidation steps 21/22) ----
/// gmregcommon.c `gm_8017BE8C_fp`: opponents that are not ChKind_None (NUM_OPPONENTS = 3).
pub fn gm_8017BE8C_fp(opps: &[i8]) -> i32 {
    melee_core::ct!("gm_8017BE8C_fp");
    opps[..3].iter().filter(|&&o| o != CHKIND_NONE).count() as i32
}
