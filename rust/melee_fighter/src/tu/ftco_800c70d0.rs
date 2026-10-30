//! src/melee/ft/ftCo_800C70D0.c: Mario/Luigi Wait selector (`ftCo_800C70D0`) and its
//! `ft_8008521C` thunk (`ftCo_800C7158`). Static inlines inlineB0..B2 are `ftCo_800C7070`.

use super::ftco_800c7070::{ext as e70, fp, ftco_800c7070, G};
use crate::types::*;

/// `ftCo_800C70D0(gobj)`.
pub fn ftco_800c70d0(g: G) {
    melee_core::ct!("ftco_800c70d0");
    let k = fp(g).kind;
    let ms = if k == Ft_Kind_Mario || k == Ft_Kind_Luigi {
        ftCo_MS_Wait
    } else {
        ftco_800c7070(g);
        return;
    };
    e70::Fighter_ChangeMotionState(g, ms, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
    let f = fp(g);
    f.set_x2219_b2(1);
    f.set_x2219_b1(1);
}

/// `ftCo_800C7158(gobj)`: `ft_8008521C(gobj)`.
pub fn ftco_800c7158(g: G) {
    melee_core::ct!("ftco_800c7158");
    ext::ft_8008521C(g)
}

pub mod ext {
    #![allow(non_snake_case, static_mut_refs)]
    use crate::types::*;
    pub static mut FT_8008521C: Option<fn(*mut HSD_GObj)> = None;
    /// TODO(group3 ft_0852) `ft_8008521C`.
    pub fn ft_8008521C(g: *mut HSD_GObj) {
        match unsafe { FT_8008521C } {
            Some(f) => f(g),
            None => { unsafe { let _ = crate::tu::ftdata::ft_8008521C(g as _); } return; },
        }
    }
}
