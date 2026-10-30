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
fn ext(slot: i32) -> &'static mut pl_StaleMoveTableExt_t {
    unsafe { &mut *crate::player::get_stale_move_table_index_ptr2(slot) }
}
#[inline]
fn x5ec(slot: i32) -> *mut pl_x5EC_t {
    get_unk6a8_ptr(slot) as *mut pl_x5EC_t
}
#[inline]
fn k() -> &'static pl_804D6470_t {
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

use crate::tu::plbonuslib::unk_cond;

#[inline]
fn counts_hit(arg4: i32) -> bool {
    !matches!(arg4, 0x4C | 0x5A | 0x5B | 0x5C | 0x60)
}

// ---- pl_040D.c ---------------------------------------------------------------------------

/// `pl_80040DDC`.
pub fn pl_80040ddc(slot: i32) {
    let t = unsafe { &mut *x5ec(slot) };
    t.x0 = 0;
    t.x4 = 0.0;
    t.x8 = 0;
    t.xC = 0;
    for e in t.x10.iter_mut() {
        e.x0 = 0.0;
        e.x4 = 0;
        e.x8 = 0;
        e.xC = 0;
        e.x10 = 0;
        e.set_x12_b0(0);
    }
}

/// `pl_80040ED4`.
pub fn pl_80040ed4(slot: i32, _arg1: i32, arg2: i32, arg3: i32, arg4: i32, arg5: f32) {
    let r31 = x5ec(slot);
    let r3 = x5ec(arg2);
    if arg3 != 1 {
        unsafe {
            if counts_hit(arg4) {
                (*r31).x8 = (*r31).x8.wrapping_add(1);
                if (*r3).x10[slot as usize].x4 == 0 {
                    (*r31).xC = (*r31).xC.wrapping_add(1);
                }
                (*r3).x10[slot as usize].x4 = (*r3).x10[slot as usize].x4.wrapping_add(1);
            }
            (*r31).x0 = arg2 as u8;
            (*r31).x4 = arg5;
            (*r3).x10[slot as usize].x0 += arg5;
        }
    }
}

/// `pl_80040FBC`.
pub fn pl_80040fbc(slot: i32, _unused: i32, arg2: i32, arg3: i32, arg4: i32) {
    let r29 = unsafe { &mut *x5ec(arg2) };
    let r3 = ext(arg2);
    if arg3 != 1 {
        let e = &mut r29.x10[slot as usize];
        if arg4 == 1 && r3.xD50 != 0 {
            e.set_x12_b0(1);
        }
        if counts_hit(arg4) {
            e.x10 = e.x10.wrapping_add(1);
        }
        if e.x10 as i32 >= k().x4 {
            let same = h::gm_8016b168() && get_team(slot) == get_team(arg2);
            if !same {
                h::pl_80038788(slot, 2, 1);
            }
        }
    }
}

/// `pl_800410F4`.
pub fn pl_800410f4(slot: i32, _unused: i32, arg2: i32, arg3: i32, arg4: i32) {
    let r3 = unsafe { &mut *x5ec(arg2) };
    if arg3 != 1 {
        let e = &mut r3.x10[slot as usize];
        if e.xC != 0 && e.xC == arg4 {
            e.x8 = e.x8.wrapping_add(1);
            if e.xC == 0x5A && e.x8 as i32 >= k().x17C {
                h::pl_80038788(slot, 0xC3, 1);
            }
        } else {
            e.x8 = 1;
            e.xC = arg4;
        }
    }
}

/// `pl_800411C4`.
pub fn pl_800411c4(slot: i32, arg1: i32) {
    if arg1 != 1 {
        fn_8004138c(slot, arg1);
        let t = x5ec(slot);
        for i in 0..6 {
            let e = unsafe { &mut (*t).x10[i] };
            if e.x12_b0() != 0 && e.x4 == 1 {
                h::pl_80038824(i as i32, 0x4C);
            }
            e.x0 = 0.0;
            e.x4 = 0;
            e.x8 = 0;
            e.xC = 0;
            e.set_x12_b0(0);
        }
    }
}

/// `pl_80041280`.
pub fn pl_80041280(slot: i32, arg1: i32) {
    if arg1 != 1 {
        for e in unsafe { (*x5ec(slot)).x10.iter_mut() } {
            e.x10 = 0;
        }
    }
}

/// `pl_800412D0` (float -> int, saturating like fctiwz except NaN).
pub fn pl_800412d0(slot: i32) -> i32 {
    unsafe { (*x5ec(slot)).x4 as i32 }
}

/// `pl_80041300`.
pub fn pl_80041300(slot: i32) -> i32 {
    let t = unsafe { &*x5ec((*x5ec(slot)).x0 as i32) };
    t.x10[slot as usize].x0 as i32
}

/// `pl_8004134C`.
pub fn pl_8004134c(slot: i32) -> i32 {
    let t = unsafe { &*x5ec((*x5ec(slot)).x0 as i32) };
    t.x10[slot as usize].x4 as i32
}

/// `fn_8004138C`.
pub fn fn_8004138c(arg0: i32, arg1: i32) {
    let t = unsafe { &*x5ec(arg0) };
    let mut total = 0.0f32;
    let mut best = 0.0f32;
    if arg1 != 1 {
        for e in t.x10.iter() {
            if best < e.x0 {
                best = e.x0;
            }
            total += e.x0;
        }
        if total != 0.0 && calculate_average(total - best, total) >= k().xD4 {
            h::pl_80038788(arg0, 0x58, 1);
        }
    }
}

/// `pl_800414C0`.
pub fn pl_800414c0(arg0: i32) -> f32 {
    let t = unsafe { &*x5ec(arg0) };
    if t.xC != 0 {
        return t.x8 as f32 / t.xC as f32;
    }
    0.0
}

/// `pl_80041524`.
pub fn pl_80041524(arg0: i32) {
    let t = &mut ext(arg0).x0_staleMoveTable.x8C0;
    t.x0 = 0;
    for e in t.x4.iter_mut() {
        e.x0 = 0;
        e.x2 = 0;
        e.set_x3_b0(0);
    }
}

/// `fn_800415B0`.
pub fn fn_800415b0(slot: i32, _unused: i32, arg2: u16, arg3: u32) {
    let t = &mut ext(slot).x0_staleMoveTable.x8C0;
    if arg2 != 0 {
        for e in t.x4.iter_mut() {
            if arg2 == e.x0 {
                if arg3 != 0 {
                    e.set_x3_b0(1);
                } else {
                    e.x2 = e.x2.wrapping_add(1);
                }
                return;
            }
        }
        let c = t.x0 as usize;
        fn_80041770(slot, t.x4[c].x2, t.x4[c].x3_b0() as u32);
        let e = &mut t.x4[c];
        e.x0 = arg2;
        e.x2 = 0;
        e.set_x3_b0(0);
        if arg3 != 0 {
            e.set_x3_b0(arg3 as u8);
        } else {
            e.x2 = 1;
        }
        t.x0 = if t.x0 == 4 { 0 } else { t.x0 + 1 };
    }
}

/// `pl_80041720`.
pub fn pl_80041720(slot: i32, arg1: i32, arg2: u16) {
    fn_800415b0(slot, arg1, arg2, 0);
}

/// `pl_80041744`.
pub fn pl_80041744(arg0: i32, arg1: i32, arg2: u16) {
    if arg1 == 0 {
        fn_800415b0(arg0, arg1, arg2, 1);
    }
}

/// `fn_80041770`.
pub fn fn_80041770(slot: i32, arg1: u8, arg2: u32) {
    if arg1 >= 5 {
        h::pl_80038824(slot, 0x7D);
    } else if arg1 >= 4 {
        h::pl_80038824(slot, 0x7C);
    } else if arg1 >= 3 {
        h::pl_80038824(slot, 0x7B);
    } else if arg1 >= 2 {
        h::pl_80038824(slot, 0x7A);
    }
    if arg2 != 0 && arg1 != 0 {
        h::pl_80038824(slot, 0x78);
    }
}

/// `pl_8004182C`.
pub fn pl_8004182c(slot: i32) {
    for i in 0..5 {
        let e = &ext(slot).x0_staleMoveTable.x8C0.x4[i];
        let (a, b) = (e.x2, e.x3_b0() as u32);
        fn_80041770(slot, a, b);
    }
}

/// `pl_800418F4`.
pub fn pl_800418f4(slot: i32) {
    let t = &mut ext(slot).x0_staleMoveTable.x8D8;
    t.x0 = 0;
    for e in t.x4.iter_mut() {
        e.x0 = 0;
        e.set_x4_b0(0);
        e.set_x4_b1(0);
    }
}

/// `pl_800419AC`.
pub fn pl_800419ac(arg0: i32, _arg1: i32, arg2: u16) {
    let t = &mut ext(arg0).x0_staleMoveTable.x8D8;
    if t.x4.iter().any(|e| e.x4_b1() != 0 && arg2 as u32 == e.x0) {
        return;
    }
    let c = t.x0 as usize;
    fn_80041bc8(arg0, t.x4[c].x4_b0() as u32, t.x4[c].x4_b1() as u32);
    let e = &mut t.x4[c];
    e.x0 = arg2 as u32;
    e.set_x4_b0(0);
    e.set_x4_b1(1);
    t.x0 = if t.x0 == 4 { 0 } else { t.x0 + 1 };
}

/// `pl_80041B08`.
pub fn pl_80041b08(arg0: i32, _unused: *mut core::ffi::c_void, arg2: u16) {
    for e in ext(arg0).x0_staleMoveTable.x8D8.x4.iter_mut() {
        if arg2 as u32 == e.x0 {
            e.set_x4_b0(1);
        }
    }
}

/// `fn_80041BC8`.
pub fn fn_80041bc8(arg0: i32, arg1: u32, arg2: u32) {
    if arg2 != 0 && arg1 == 0 {
        h::pl_80038824(arg0, 0xC8);
    }
}

/// `pl_80041BFC`.
pub fn pl_80041bfc(slot: i32) {
    for i in 0..5 {
        let e = &ext(slot).x0_staleMoveTable.x8D8.x4[i];
        let (a, b) = (e.x4_b0() as u32, e.x4_b1() as u32);
        fn_80041bc8(slot, a, b);
    }
}

// ---- plbonuslib.c (pl_8003FBFC .. end of file) -------------------------------------------

pub use crate::tu::plbonuslib::pl_8003FBFC as pl_8003fbfc;
pub use crate::tu::plbonuslib::pl_8003FC20 as pl_8003fc20;
pub use crate::tu::plbonuslib::pl_8003FC44 as pl_8003fc44;

pub use crate::tu::plbonuslib::pl_8003FC88 as pl_8003fc88;

pub use crate::tu::plbonuslib::pl_8003FDA0 as pl_8003fda0;
pub use crate::tu::plbonuslib::pl_8003FDC8 as pl_8003fdc8;
pub use crate::tu::plbonuslib::pl_8003FDF4 as pl_8003fdf4;
pub use crate::tu::plbonuslib::pl_8003FE1C as pl_8003fe1c;
pub use crate::tu::plbonuslib::pl_8003FE40 as pl_8003fe40;

fn none_has(kind: i32) -> bool {
    for i in 0..6 {
        if h::pl_80039418(i, kind) != 0 {
            return false;
        }
    }
    true
}
pub use crate::tu::plbonuslib::pl_8003FE64 as pl_8003fe64;
pub use crate::tu::plbonuslib::pl_8003FED0 as pl_8003fed0;

pub use crate::tu::plbonuslib::pl_8003FF44 as pl_8003ff44;

pub use crate::tu::plbonuslib::pl_8003FFDC as pl_8003ffdc;

pub use crate::tu::plbonuslib::pl_80040048;

pub use crate::tu::plbonuslib::pl_80040120;

pub use crate::tu::plbonuslib::pl_800401F0 as pl_800401f0;

pub use crate::tu::plbonuslib::pl_80040270;

pub use crate::tu::plbonuslib::pl_800402D0 as pl_800402d0;

pub use crate::tu::plbonuslib::pl_80040330;

pub use crate::tu::plbonuslib::pl_80040374;

pub use crate::tu::plbonuslib::pl_800403C0 as pl_800403c0;

pub use crate::tu::plbonuslib::pl_800403FC as pl_800403fc;

pub use crate::tu::plbonuslib::pl_80040460;

pub use crate::tu::plbonuslib::pl_8004049C as pl_8004049c;

pub use crate::tu::plbonuslib::pl_80040614;

pub use crate::tu::plbonuslib::pl_8004065C as pl_8004065c;

pub use crate::tu::plbonuslib::pl_80040688;

pub use crate::tu::plbonuslib::pl_800407C8 as pl_800407c8;

pub use crate::tu::plbonuslib::pl_80040870;
pub use crate::tu::plbonuslib::pl_80040894;
pub use crate::tu::plbonuslib::pl_800408B8 as pl_800408b8;
pub use crate::tu::plbonuslib::pl_800408DC as pl_800408dc;
pub use crate::tu::plbonuslib::pl_80040900;
pub use crate::tu::plbonuslib::pl_80040924;
pub use crate::tu::plbonuslib::pl_80040948;
pub use crate::tu::plbonuslib::pl_80040A04 as pl_80040a04;
pub use crate::tu::plbonuslib::pl_80040A30 as pl_80040a30;
pub use crate::tu::plbonuslib::pl_80040A54 as pl_80040a54;
pub use crate::tu::plbonuslib::pl_80040A78 as pl_80040a78;
pub use crate::tu::plbonuslib::pl_80040A9C as pl_80040a9c;
pub use crate::tu::plbonuslib::pl_80040AF0 as pl_80040af0;
pub use crate::tu::plbonuslib::pl_80040B18 as pl_80040b18;
pub use crate::tu::plbonuslib::pl_80040B3C as pl_80040b3c;
pub use crate::tu::plbonuslib::pl_80040B64 as pl_80040b64;
pub use crate::tu::plbonuslib::pl_80040B8C as pl_80040b8c;
pub use crate::tu::plbonuslib::pl_80040BD8 as pl_80040bd8;
pub use crate::tu::plbonuslib::pl_80040BFC as pl_80040bfc;
pub use crate::tu::plbonuslib::pl_80040C24 as pl_80040c24;
pub use crate::tu::plbonuslib::pl_80040C48 as pl_80040c48;
pub use crate::tu::plbonuslib::pl_80040C6C as pl_80040c6c;
pub use crate::tu::plbonuslib::pl_80040C90 as pl_80040c90;
pub use crate::tu::plbonuslib::pl_80040CB4 as pl_80040cb4;
pub use crate::tu::plbonuslib::pl_80040CD8 as pl_80040cd8;
pub use crate::tu::plbonuslib::pl_80040CFC as pl_80040cfc;
pub use crate::tu::plbonuslib::pl_80040D20 as pl_80040d20;
pub use crate::tu::plbonuslib::pl_80040D44 as pl_80040d44;
pub use crate::tu::plbonuslib::pl_80040D68 as pl_80040d68;
pub use crate::tu::plbonuslib::pl_80040D8C as pl_80040d8c;
pub use crate::tu::plbonuslib::pl_80040DB8 as pl_80040db8;
