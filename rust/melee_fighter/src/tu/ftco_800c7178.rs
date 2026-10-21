//! src/melee/ft/ftCo_800C7178.c: Mario/Luigi WalkSlow selector (`ftCo_800C7178`) and its
//! `ft_8008521C` thunk (`ftCo_800C7200`). Static inlines inlineB0..B2 are `ftCo_800C7070`.

use super::ftco_800c7070::{ext as e70, fp, ftco_800c7070, G};
use super::ftco_800c70d0::ext as e0d0;
use crate::types::*;

/// `ftCo_800C7178(gobj)`.
pub fn ftco_800c7178(g: G) {
    let k = fp(g).kind;
    let ms = if k == Ft_Kind_Mario || k == Ft_Kind_Luigi {
        ftCo_MS_WalkSlow
    } else {
        ftco_800c7070(g);
        return;
    };
    e70::Fighter_ChangeMotionState(g, ms, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
    let f = fp(g);
    f.set_x2219_b2(1);
    f.set_x2219_b1(1);
}

/// `ftCo_800C7200(gobj)`: `ft_8008521C(gobj)`.
pub fn ftco_800c7200(g: G) {
    e0d0::ft_8008521C(g)
}
