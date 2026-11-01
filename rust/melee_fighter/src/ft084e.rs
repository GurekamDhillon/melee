
//! Hand port, round 12: all 14 functions of src/melee/ft/ft_084E.c (shared physics callbacks:
//! air fall + drift, ground friction, animation-driven `x6A4_transNOffset` velocities) and
//! two more from the tail of ft_081B.c (`ft_80084CB0` ECB box copy, `ft_80084DB0` fall + drift),
//! and ft_08A1.c's stick test (`ft_8008A1FC`, `ft_8008A244` + its static inline).
//!
//! Fused sites (`tools/rust-port/fma_sites.py src/melee/ft/ft_084E.c`): ft_80085030 one f32
//! fmuladd (`z * dir - gr_vel`), ft_80085154 two (`a*b - c*d`, `a*b + c*d`: clang contracts the
//! left product, the right one is rounded to f32 first); all -> `fmadds`. The rest have none.
//! Checked against the C (cut verbatim, shipping contraction) by tests/ft084e_ref.rs.

use crate::ftcommon as fc;

use crate::hooks as h;

use crate::types::*;

use melee_core::fp::fmadds;

use melee_runtime::gekko::{cosf, sinf};

pub type G = *mut HSD_GObj;

#[inline(always)]
pub(crate) fn fp<'a>(g: G) -> &'a mut Fighter {
    unsafe { &mut *((*g).user_data as *mut Fighter) }
}

#[inline(always)]
pub(crate) fn x594_b0(f: &Fighter) -> u8 {
    unsafe { f.anon_594.anon_0.x594_b0() }
}

/// fall-fast check, then fast fall or `ftCommon_Fall(gravity, terminal_velocity)`.
pub(crate) fn fall_or_fast(f: &mut Fighter) {
    fc::check_fall_fast(f);
    if f.fall_fast() != 0 {
        fc::fall_fast(f);
    } else {
        let (g, t) = (f.co_attrs.gravity, f.co_attrs.terminal_velocity);
        fc::fall(f, g, t);
    }
}

pub use crate::tu::ft_081b::ft_80084cb0;
pub use crate::tu::ft_081b::ft_80084db0;
pub use crate::tu::ft_084e::ft_80084e1c;
pub use crate::tu::ft_084e::ft_80084eec;
pub use crate::tu::ft_084e::ft_80084f3c;
pub use crate::tu::ft_084e::ft_80084fa8;
pub use crate::tu::ft_084e::ft_80085004;
pub use crate::tu::ft_084e::ft_80085030;
pub use crate::tu::ft_084e::ft_80085088;
pub use crate::tu::ft_084e::ft_800850b4;
pub use crate::tu::ft_084e::ft_800850e0;
pub use crate::tu::ft_084e::ft_80085134;
pub use crate::tu::ft_084e::ft_80085154;
pub use crate::tu::ft_084e::ft_800851c0;
pub use crate::tu::ft_084e::ft_800851d0;
pub use crate::tu::ft_084e::ft_80085204;
pub use crate::tu::ft_08a1::ft_8008a1fc;
pub use crate::tu::ft_08a1::ft_8008a244;
pub use crate::tu::ft_08a1::ft_8008a244_inline;
