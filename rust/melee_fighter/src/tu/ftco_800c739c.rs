//! src/melee/ft/ftCo_800C739C.c: Kirby WalkFast selector (`ftCo_800C739C`) and the
//! `ft_8008521C` thunk (`ftCo_800C7414`). Static inlines inlineB0..B2 are `ftCo_800C7070`.

use super::ftco_800c7070::{ext as e70, fp, ftco_800c7070, G};
use super::ftco_800c70d0::ext as e0d0;
use super::ftco_800c7220::ftkb_is_kirby;
use crate::types::*;

/// `ftCo_800C739C(gobj)`.
pub fn ftco_800c739c(g: G) {
    melee_core::ct!("ftco_800c739c");
    let ms = if ftkb_is_kirby(fp(g).kind) {
        ftCo_MS_WalkFast
    } else {
        ftco_800c7070(g);
        return;
    };
    e70::Fighter_ChangeMotionState(g, ms, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
    let f = fp(g);
    f.set_x2219_b2(1);
    f.set_x2219_b1(1);
}

/// `ftCo_800C7414(gobj)`: `ft_8008521C(gobj)`.
pub fn ftco_800c7414(g: G) {
    melee_core::ct!("ftco_800c7414");
    e0d0::ft_8008521C(g)
}
