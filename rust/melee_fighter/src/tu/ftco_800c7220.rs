//! src/melee/ft/ftCo_800C7220.c: Kirby Wait/WalkSlow/WalkMiddle selectors and the
//! `ft_8008521C` thunk. Static inlines inlineB0..B2 are `ftCo_800C7070`.

use super::ftco_800c7070::{ext as e70, fp, ftco_800c7070, G};
use super::ftco_800c70d0::ext as e0d0;
use crate::types::*;

/// `FTKB_IS_KIRBY(k)` (ft/forward.h, TARGET_PC): Kirby, or an m-ex kind whose clone base is Kirby.
pub fn ftkb_is_kirby(k: FighterKind) -> bool {
    let k = k as i32;
    k == Ft_Kind_Kirby as i32
        || (k >= Ft_Kind_Mex0 as i32 && k < Ft_Kind_Max as i32 && ext::ftKb_MexKirbyLike(k) != 0)
}

fn enter(g: G, ms: ftCommon_MotionState) {
    if !ftkb_is_kirby(fp(g).kind) {
        ftco_800c7070(g);
    } else {
        e70::Fighter_ChangeMotionState(g, ms, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        let f = fp(g);
        f.set_x2219_b2(1);
        f.set_x2219_b1(1);
    }
}

/// `ftCo_800C7220(gobj)`: Kirby -> Wait.
pub fn ftco_800c7220(g: G) {
    enter(g, ftCo_MS_Wait)
}

/// `ftCo_800C7294(gobj)`: Kirby -> WalkSlow.
pub fn ftco_800c7294(g: G) {
    enter(g, ftCo_MS_WalkSlow)
}

/// `ftCo_800C7308(gobj)`: Kirby -> WalkMiddle.
pub fn ftco_800c7308(g: G) {
    enter(g, ftCo_MS_WalkMiddle)
}

/// `ftCo_800C737C(gobj)`: `ft_8008521C(gobj)`.
pub fn ftco_800c737c(g: G) {
    e0d0::ft_8008521C(g)
}

pub mod ext {
    #![allow(non_snake_case, static_mut_refs)]
    pub static mut FTKB_MEX_KIRBY_LIKE: Option<fn(i32) -> i32> = None;
    /// TODO(group4 ftkirby.c) `ftKb_MexKirbyLike` (PC m-ex: is this kind's clone base Kirby).
    pub fn ftKb_MexKirbyLike(kind: i32) -> i32 {
        match unsafe { FTKB_MEX_KIRBY_LIKE } {
            Some(f) => f(kind),
            None => panic!("TODO(group4 ftkirby.c) ftKb_MexKirbyLike is not ported"),
        }
    }
}
