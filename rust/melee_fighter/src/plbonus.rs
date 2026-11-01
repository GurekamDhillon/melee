
//! Bonus / stat bookkeeping over the player table (port round 4): all of src/melee/pl/pl_040D.c
//! and the tail of src/melee/pl/plbonuslib.c (`pl_8003FBFC` to the end of the file, plus the
//! static inline `unk_cond` and plbonusinline.h / pl/inlines.h helpers they use).
//!
//! Everything here is field logic over `StaleMoveTable` / `pl_StaleMoveTableExt_t` /
//! `pl_x5EC_t` (in `StaticPlayer::stale_moves`). Calls out go through `crate::hooks`:
//! plbonus.c (`pl_80038788`, `pl_80038824`, `pl_80038898`, `pl_80039418`, `pl_800386D8`,
//! TODO(group3 plbonus)), ft_0877.c / ft_0892.c accessors (TODO(group3 ft)), gm (TODO(group6)),
//! and the `pl_804D6470` constants table (`hooks::pl_804d6470`, loaded by plbonus.c).
//! No fused (a*b+c) site in any of these functions (`tools/rust-port/fma_sites.py`).
//!
//! C quirks kept: `x8C0.x0` / `x8D8.x0` ring indices and the `x5EC.x0` slot are used unchecked in
//! C; here an out-of-range value panics instead of reading the neighbouring field.
//! Checked against the C by tests/plbonus_ref.rs (gen/cref/plbonus_ref.c).

use crate::ftcommon::abs_c;

use crate::hooks as h;

use crate::player::{get_entity_at_index, get_team, get_unk6a8_ptr, player_8003221c};

use crate::stale::action_stats;

use crate::types::*;

#[inline]
pub(crate) fn ext(slot: i32) -> &'static mut pl_StaleMoveTableExt_t {
    unsafe { &mut *crate::player::get_stale_move_table_index_ptr2(slot) }
}

#[inline]
pub(crate) fn x5ec(slot: i32) -> *mut pl_x5EC_t {
    get_unk6a8_ptr(slot) as *mut pl_x5EC_t
}

#[inline]
pub(crate) fn k() -> &'static pl_804D6470_t {
    h::pl_804d6470()
}

/// plbonusinline.h `pl_CalculateAverage` (HSD_ASSERT total != 0 -> panic).
#[inline]
pub fn calculate_average(val: f32, total: f32) -> f32 {
    assert!(total != 0.0, "plbonusinline.h:16 total != 0.0F");
    val / total
}

/// pl/inlines.h `pl_CheckIfSameTeam`.
#[inline]
pub fn check_if_same_team(a: i32, b: i32) -> bool {
    h::gm_8016b168() && get_team(b) == get_team(a)
}

#[inline]
pub(crate) fn counts_hit(arg4: i32) -> bool {
    !matches!(arg4, 0x4C | 0x5A | 0x5B | 0x5C | 0x60)
}

pub(crate) fn none_has(kind: i32) -> bool {
    for i in 0..6 {
        if h::pl_80039418(i, kind) != 0 {
            return false;
        }
    }
    true
}

pub use crate::tu::pl_040d::fn_8004138c;
pub use crate::tu::pl_040d::fn_800415b0;
pub use crate::tu::pl_040d::fn_80041770;
pub use crate::tu::pl_040d::fn_80041bc8;
pub use crate::tu::plbonuslib::pl_8003fbfc;
pub use crate::tu::plbonuslib::pl_8003fc20;
pub use crate::tu::plbonuslib::pl_8003fc44;
pub use crate::tu::plbonuslib::pl_8003fc88;
pub use crate::tu::plbonuslib::pl_8003fda0;
pub use crate::tu::plbonuslib::pl_8003fdc8;
pub use crate::tu::plbonuslib::pl_8003fdf4;
pub use crate::tu::plbonuslib::pl_8003fe1c;
pub use crate::tu::plbonuslib::pl_8003fe40;
pub use crate::tu::plbonuslib::pl_8003fe64;
pub use crate::tu::plbonuslib::pl_8003fed0;
pub use crate::tu::plbonuslib::pl_8003ff44;
pub use crate::tu::plbonuslib::pl_8003ffdc;
pub use crate::tu::plbonuslib::pl_80040048_fp as pl_80040048;
pub use crate::tu::plbonuslib::pl_80040120_fp as pl_80040120;
pub use crate::tu::plbonuslib::pl_800401f0;
pub use crate::tu::plbonuslib::pl_80040270_fp as pl_80040270;
pub use crate::tu::plbonuslib::pl_800402d0;
pub use crate::tu::plbonuslib::pl_80040330_fp as pl_80040330;
pub use crate::tu::plbonuslib::pl_80040374_fp as pl_80040374;
pub use crate::tu::plbonuslib::pl_800403c0;
pub use crate::tu::plbonuslib::pl_800403fc;
pub use crate::tu::plbonuslib::pl_80040460_fp as pl_80040460;
pub use crate::tu::plbonuslib::pl_8004049c;
pub use crate::tu::plbonuslib::pl_80040614_fp as pl_80040614;
pub use crate::tu::plbonuslib::pl_8004065c;
pub use crate::tu::plbonuslib::pl_80040688_fp as pl_80040688;
pub use crate::tu::plbonuslib::pl_800407c8;
pub use crate::tu::plbonuslib::pl_80040870_fp as pl_80040870;
pub use crate::tu::plbonuslib::pl_80040894_fp as pl_80040894;
pub use crate::tu::plbonuslib::pl_800408b8;
pub use crate::tu::plbonuslib::pl_800408dc;
pub use crate::tu::plbonuslib::pl_80040900_fp as pl_80040900;
pub use crate::tu::plbonuslib::pl_80040924_fp as pl_80040924;
pub use crate::tu::plbonuslib::pl_80040948_fp as pl_80040948;
pub use crate::tu::plbonuslib::pl_80040a04;
pub use crate::tu::plbonuslib::pl_80040a30;
pub use crate::tu::plbonuslib::pl_80040a54;
pub use crate::tu::plbonuslib::pl_80040a78;
pub use crate::tu::plbonuslib::pl_80040a9c;
pub use crate::tu::plbonuslib::pl_80040af0;
pub use crate::tu::plbonuslib::pl_80040b18;
pub use crate::tu::plbonuslib::pl_80040b3c;
pub use crate::tu::plbonuslib::pl_80040b64;
pub use crate::tu::plbonuslib::pl_80040b8c;
pub use crate::tu::plbonuslib::pl_80040bd8;
pub use crate::tu::plbonuslib::pl_80040bfc;
pub use crate::tu::plbonuslib::pl_80040c24;
pub use crate::tu::plbonuslib::pl_80040c48;
pub use crate::tu::plbonuslib::pl_80040c6c;
pub use crate::tu::plbonuslib::pl_80040c90;
pub use crate::tu::plbonuslib::pl_80040cb4;
pub use crate::tu::plbonuslib::pl_80040cd8;
pub use crate::tu::plbonuslib::pl_80040cfc;
pub use crate::tu::plbonuslib::pl_80040d20;
pub use crate::tu::plbonuslib::pl_80040d44;
pub use crate::tu::plbonuslib::pl_80040d68;
pub use crate::tu::plbonuslib::pl_80040d8c;
pub use crate::tu::plbonuslib::pl_80040db8;
pub use crate::tu::pl_040d::pl_80040ddc;
pub use crate::tu::pl_040d::pl_80040ed4;
pub use crate::tu::pl_040d::pl_80040fbc;
pub use crate::tu::pl_040d::pl_800410f4;
pub use crate::tu::pl_040d::pl_800411c4;
pub use crate::tu::pl_040d::pl_80041280_fp as pl_80041280;
pub use crate::tu::pl_040d::pl_800412d0;
pub use crate::tu::pl_040d::pl_80041300;
pub use crate::tu::pl_040d::pl_8004134c;
pub use crate::tu::pl_040d::pl_800414c0;
pub use crate::tu::pl_040d::pl_80041524_fp as pl_80041524;
pub use crate::tu::pl_040d::pl_80041720_fp as pl_80041720;
pub use crate::tu::pl_040d::pl_80041744_fp as pl_80041744;
pub use crate::tu::pl_040d::pl_8004182c;
pub use crate::tu::pl_040d::pl_800418f4;
pub use crate::tu::pl_040d::pl_800419ac;
pub use crate::tu::pl_040d::pl_80041b08;
pub use crate::tu::pl_040d::pl_80041bfc;
pub use crate::tu::plbonuslib::unk_cond_fp as unk_cond;
