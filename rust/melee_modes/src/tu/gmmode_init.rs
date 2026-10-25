//! `gm_Mode_Adventure_OnInit` (src/melee/gm/gmadventure.c) and `gm_Mode_AllStar_OnInit`
//! (src/melee/gm/gmallstar.c): reset the mode's character-select record. Checked against the C text
//! itself (`gen/cref/cut/mode_init.c`, `tests/cut_mode_init.rs`).
#![allow(non_snake_case)]

use melee_types::*;

use crate::tu::gmmain_lib::{gmMainLib_8015CDD4, gmMainLib_8015CDE0};

/// The two modes differ only in which settings record they reset and the stock count.
unsafe fn reset(s: *mut gmm_x0_528_t, stocks: u8) {
    (*s).c_kind = ChKind_None as i8;
    (*s).color = 0;
    (*s).stocks = stocks;
    (*s).cpu_level = 0;
    (*s).nametag = GM_NAMETAG_COUNT as u8;
    (*s).x5 = 0;
}

pub unsafe extern "C" fn gm_Mode_Adventure_OnInit() {
    reset(gmMainLib_8015CDD4(), 3);
}

pub unsafe extern "C" fn gm_Mode_AllStar_OnInit() {
    reset(gmMainLib_8015CDE0(), 1);
}
