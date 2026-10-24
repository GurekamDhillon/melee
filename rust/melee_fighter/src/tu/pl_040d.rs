//! Native port of src/melee/pl/pl_040D.c: per-player "who hit whom" bonus bookkeeping
//! (`pl_x5EC_t` at StaticPlayer +0x6A8 and the `pl_x8C0_t` / `pl_x8D8_t` rings in the stale-move
//! table extension).
//!
//! Every definition of the TU already has a port in `crate::plbonus` (the whole `pl_80040DDC ..
//! pl_80041BFC` range); they are re-exported here under the C names.
#![allow(non_snake_case, non_upper_case_globals, clippy::missing_safety_doc)]

pub use crate::plbonus::{
    fn_80041770 as fn_80041770, fn_8004138c as fn_8004138C, fn_800415b0 as fn_800415B0, fn_80041bc8 as fn_80041BC8,
    pl_80040ddc as pl_80040DDC, pl_80040ed4 as pl_80040ED4, pl_80040fbc as pl_80040FBC, pl_800410f4 as pl_800410F4,
    pl_800411c4 as pl_800411C4, pl_80041280 as pl_80041280, pl_800412d0 as pl_800412D0, pl_80041300 as pl_80041300,
    pl_8004134c as pl_8004134C, pl_800414c0 as pl_800414C0, pl_80041524 as pl_80041524, pl_80041720 as pl_80041720,
    pl_80041744 as pl_80041744, pl_8004182c as pl_8004182C, pl_800418f4 as pl_800418F4, pl_800419ac as pl_800419AC,
    pl_80041b08 as pl_80041B08, pl_80041bfc as pl_80041BFC,
};
