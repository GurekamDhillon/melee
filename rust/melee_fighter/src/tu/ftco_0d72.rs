//! src/melee/ft/kinds/ftCommon/ftCo_0D72.c: is the current motion inside one of the two
//! `x2D0` motion ranges (`x2C` / `x30` start, `x28` length; -1 = unused).
//! No fused sites. Checked against the C by tests/tu_ftco_0d72.rs.

use crate::types::*;

/// `ftCo_800D72A0`.
pub fn ftco_800d72a0(fp: &Fighter) -> bool {
    melee_core::ct!("ftco_800d72a0");
    let p = unsafe { &*fp.x2D0 };
    let starts = [p.x2C as i32, p.x30 as i32];
    let m = fp.motion_id as i32;
    for s in starts {
        if s != -1 && s <= m && m < s.wrapping_add(p.x28) {
            return true;
        }
    }
    false
}
