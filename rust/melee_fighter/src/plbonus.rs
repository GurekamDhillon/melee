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

/// plbonuslib.c static inline `unk_cond`.
#[inline]
fn unk_cond(arg0: i32, r23: i32) -> bool {
    r23 == 6 || r23 == arg0 || check_if_same_team(arg0, r23)
}

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

/// `pl_8003FBFC`.
pub fn pl_8003fbfc(a: i32) -> i32 {
    ext(a).xCFC
}
/// `pl_8003FC20`.
pub fn pl_8003fc20(a: i32) -> i32 {
    ext(a).xD00
}
/// `pl_8003FC44`.
pub fn pl_8003fc44(slot: i32, arg1: i32) {
    let t = ext(slot);
    if arg1 != 0 {
        return;
    }
    unsafe { t.xDD0.anon_0.set_bit0(1) };
}

/// `pl_8003FC88`.
pub fn pl_8003fc88(arg0: i32, arg1: i32, arg2: i32) {
    let s = &mut ext(arg0).x0_staleMoveTable;
    if arg1 == 1 {
        return;
    }
    let r0 = s.xC90;
    let hit = match r0 {
        0 => (1..=3).contains(&arg2),
        1 => (6..=8).contains(&arg2),
        2 => (9..=11).contains(&arg2),
        3 => (17..=48).contains(&arg2),
        _ => false,
    };
    if hit {
        if r0 == 3 {
            s.xC90 = 0;
            h::pl_80038824(arg0, 0x56);
        } else {
            s.xC90 = s.xC90.wrapping_add(1);
        }
    } else {
        s.xC90 = 0;
    }
}

/// `pl_8003FDA0`.
pub fn pl_8003fda0(arg0: i32, _arg1: i32) {
    h::pl_80038788(arg0, 0x5B, 1);
}
/// `pl_8003FDC8`.
pub fn pl_8003fdc8(arg0: i32, _arg1: i32) {
    let t = ext(arg0);
    t.xD70 = t.xD70.wrapping_add(1);
}
/// `pl_8003FDF4`.
pub fn pl_8003fdf4(arg0: i32, _arg1: i32) {
    h::pl_80038788(arg0, 0xC4, 1);
}
/// `pl_8003FE1C`.
pub fn pl_8003fe1c(slot: i32, _arg1: i32) {
    h::pl_80038824(slot, 0xA);
}
/// `pl_8003FE40`.
pub fn pl_8003fe40(arg0: i32, _arg1: i32) {
    h::pl_80038824(arg0, 0x4B);
}

fn none_has(kind: i32) -> bool {
    for i in 0..6 {
        if h::pl_80039418(i, kind) != 0 {
            return false;
        }
    }
    true
}
/// `pl_8003FE64`.
pub fn pl_8003fe64(arg0: i32, _arg1: i32) {
    if none_has(0x1A) {
        h::pl_80038788(arg0, 0x1A, 1);
    }
}
/// `pl_8003FED0`.
pub fn pl_8003fed0(arg0: i32, arg1: i32) {
    if arg1 != 0 {
        return;
    }
    if none_has(0x82) {
        h::pl_80038788(arg0, 0x82, 1);
    }
}

/// `pl_8003FF44` (the C evaluates ft_80087A8C only when the earlier tests pass).
pub fn pl_8003ff44(arg0: i32, arg1: i32, arg2: i32) {
    let t = ext(arg0);
    let g = get_entity_at_index(arg0, arg1);
    if arg1 == 0 && (1..=0x10).contains(&arg2) && h::ft_80087a8c(g) > k().x38 {
        unsafe { t.xDD1.anon_0.set_bit4(1) };
    }
}

/// `pl_8003FFDC`.
pub fn pl_8003ffdc(arg0: i32, _arg1: i32, arg2: i32, arg3: i32, arg4: i32) {
    let g = get_entity_at_index(arg2, arg3);
    if arg0 != arg2 && arg4 != h::ft_80087a98(g) {
        h::pl_80038824(arg0, 0x46);
    }
}

/// `pl_80040048`.
pub fn pl_80040048(arg0: i32, arg1: i32) {
    let g = get_entity_at_index(arg0, arg1);
    let r3 = h::ftlib_80087300(g);
    if arg1 != 0 || r3 == 6 {
        return;
    }
    let f31 = unsafe { (*h::ft_800898b4(g)).kb_applied1 };
    let t = ext(arg0);
    if f31 >= k().x98 {
        t.x0_staleMoveTable.xCE8 = t.x0_staleMoveTable.xCE8.wrapping_add(1);
    }
    if f31 >= k().xA0 && !h::ft_80087aa4(g) && h::ft_80087ab4(g) <= 1 {
        h::pl_80038824(arg0, 0x48);
    }
}

/// `pl_80040120`.
pub fn pl_80040120(arg0: i32, arg1: i32) {
    if arg1 == 1 {
        return;
    }
    if ext(arg0).xD4C != 0 {
        h::pl_80038824(arg0, 0x4A);
    }
    if ext(arg0).xD54 != 0 {
        h::pl_80038824(arg0, 0x4D);
    }
    for i in 0..6 {
        if !player_8003221c(i) {
            continue;
        }
        if i == arg0 {
            ext(arg0).xD50 = k().xB0;
        } else {
            ext(i).xD4C = k().xA4;
        }
    }
}

/// `pl_800401F0`.
pub fn pl_800401f0(arg0: i32, _arg1: i32, arg2: f32, arg3: f32) {
    let s = &mut ext(arg0).x0_staleMoveTable;
    s.xCDC += abs_c(arg2);
    s.xCE0 += abs_c(arg3);
}

/// `pl_80040270`.
pub fn pl_80040270(arg0: i32, arg1: i32, arg2: f32) {
    let s = &mut ext(arg0).x0_staleMoveTable;
    if arg1 != 0 {
        return;
    }
    if arg2 >= k().x98 {
        s.xCE4 = s.xCE4.wrapping_add(1);
    }
}

/// `pl_800402D0`.
pub fn pl_800402d0(slot: i32, arg1: i32, arg2: i32) {
    let s = &mut ext(slot).x0_staleMoveTable;
    if arg1 != 0 {
        return;
    }
    s.xCEC = s.xCEC.wrapping_add(1);
    if arg2 == 0 {
        return;
    }
    s.xCF0 = s.xCF0.wrapping_add(1);
}

/// `pl_80040330`.
pub fn pl_80040330(slot: i32, arg1: i32, arg2: f32) {
    if arg1 != 0 {
        return;
    }
    let t = ext(slot);
    if arg2 < t.xCF8 {
        t.xCF8 = arg2;
    }
}

/// `pl_80040374`.
pub fn pl_80040374(arg0: i32, arg1: i32) {
    let t = ext(arg0);
    if arg1 == 0 && t.xD64 != 6 {
        t.xD60 = k().x124;
    }
}

/// `pl_800403C0`.
pub fn pl_800403c0(arg0: i32, _arg1: i32) {
    h::pl_80038788(arg0, 0xC5, 1);
}

/// `pl_800403FC`.
pub fn pl_800403fc(arg0: i32, _arg1: i32, slot: i32, arg3: i32, arg4: i32) {
    let t = ext(slot);
    if arg3 == 0 && (0x54..=0x57).contains(&arg4) {
        t.x0_staleMoveTable.xCAC = arg0 as u32;
    }
}

/// `pl_80040460`.
pub fn pl_80040460(slot: i32, arg1: i32) {
    let t = ext(slot);
    if arg1 != 0 {
        return;
    }
    t.x0_staleMoveTable.xCAC = 6;
}

/// `pl_8004049C(player, item kind)`: KO of a stage enemy.
#[allow(non_upper_case_globals)]
pub fn pl_8004049c(player: i32, arg1: ItemKind) {
    if player == 6 {
        return;
    }
    assert!((0..6).contains(&player), "zako ko player illegal ! :{player}");
    let v: i32 = match arg1 {
        It_Kind_Old_Kuri | It_Kind_Kuriboh => 0xCE,
        It_Kind_Heiho => 0xD6,
        It_Kind_Nokonoko => 0xCF,
        It_Kind_Patapata => 0xD0,
        It_Kind_Likelike => 0xD2,
        It_Kind_Old_Lead | It_Kind_Leadead => 0xD1,
        It_Kind_Old_Octa | It_Kind_Octarock => 0xD3,
        It_Kind_Old_Otto | It_Kind_Ottosea => 0xD4,
        It_Kind_Whitebea => 0xD5,
        _ => return,
    };
    let r = h::gm_8016f2f8(v, player as u8);
    h::gm_8016b6e8(player, r);
    h::pl_80038824(player, v);
}

/// `pl_80040614` (`!arg8` is `arg8 == 0.0`).
pub fn pl_80040614(arg0: i32, _arg1: bool, arg8: f32) {
    if arg8 == 0.0 {
        return;
    }
    if arg8 <= k().x4C {
        h::pl_80038788(arg0, 0x2D, 1);
    }
}

/// `pl_8004065C`.
pub fn pl_8004065c(arg0: i32, arg1: i32) {
    if arg1 != 0 {
        return;
    }
    h::pl_80038824(arg0, 0x78);
}

/// `pl_80040688`.
pub fn pl_80040688(arg0: i32, arg1: i32, arg2: i32) {
    if h::gm_get_frame_count() <= k().xEC as u32 {
        h::pl_80038788(arg0, 0x5F, 1);
    }
    if arg2 != 0 && arg1 != 6 {
        let s = &ext(arg1).x0_staleMoveTable;
        let r0 = s.xCB8;
        let x3 = s.xCBC.x3;
        let b0 = s.xCC0.x10_b0();
        if !unk_cond(arg1, r0) {
            if (0x33..=0x3D).contains(&x3) {
                h::pl_80038788(r0, 0x2A, 1);
            }
            if b0 != 0 {
                h::pl_80038788(r0, 7, 1);
                h::pl_80038898(r0, 6);
            }
        }
    }
}

/// `pl_800407C8`.
pub fn pl_800407c8(arg0: i32) -> f32 {
    let s = &ext(arg0).x0_staleMoveTable;
    if s.xCEC != 0 {
        let tmp = s.xCEC as u32 as f32;
        let f31 = s.xCF0 as u32 as f32;
        return calculate_average(f31, tmp);
    }
    0.0
}

/// `pl_80040870`.
pub fn pl_80040870(a: i32) -> f32 {
    ext(a).xCF8
}
/// `pl_80040894`.
pub fn pl_80040894(a: i32) -> u32 {
    ext(a).xD40
}
/// `pl_800408B8`.
pub fn pl_800408b8(a: i32) -> u32 {
    ext(a).xD48
}
/// `pl_800408DC`.
pub fn pl_800408dc(a: i32) -> i32 {
    ext(a).x0_staleMoveTable.xCB4
}
/// `pl_80040900`.
pub fn pl_80040900(a: i32) -> i32 {
    unsafe { action_stats(a).anon_358.x358_hits.total as i32 }
}
/// `pl_80040924`.
pub fn pl_80040924(a: i32) -> i32 {
    action_stats(a).attacks.total as i32
}
/// `pl_80040948`.
pub fn pl_80040948(a: i32) -> f32 {
    let total = action_stats(a).attacks.total;
    let r30 = unsafe { action_stats(a).anon_358.x358_hits.total };
    if total != 0 {
        return calculate_average(r30 as f32, total as f32);
    }
    0.0
}
/// `pl_80040A04`.
pub fn pl_80040a04(a: i32) -> i32 {
    let s = action_stats(a);
    s.attacks.total.wrapping_sub(s.attacks.aerials_count) as i32
}
/// `pl_80040A30`.
pub fn pl_80040a30(a: i32) -> i32 {
    action_stats(a).attacks.aerials_count as i32
}
/// `pl_80040A54`.
pub fn pl_80040a54(a: i32) -> i32 {
    action_stats(a).attacks.thrown_item_count as i32
}
/// `pl_80040A78`.
pub fn pl_80040a78(a: i32) -> i32 {
    action_stats(a).attacks.by_attack_counts[StatsAttack_Catch as usize] as i32
}
/// `pl_80040A9C`: the C loop sums `by_attack_counts[DownAttackD + 3 ..= DownAttackD + 11]`.
pub fn pl_80040a9c(a: i32) -> i32 {
    let c = &action_stats(a).attacks.by_attack_counts;
    let base = StatsAttack_DownAttackD as usize;
    let mut sum = 0i32;
    let mut cursor: u32 = 51;
    for k in 0..11usize {
        if cursor.wrapping_sub(51) > 1 {
            sum = sum.wrapping_add(c[base + k + 1] as i32);
        }
        cursor += 1;
    }
    sum
}
/// `pl_80040AF0`.
pub fn pl_80040af0(a: i32) -> u32 {
    h::pl_800386d8(action_stats(a), 0x70)
}
/// `pl_80040B18`.
pub fn pl_80040b18(a: i32) -> i32 {
    action_stats(a).attacks.specials_count as i32
}
/// `pl_80040B3C` (float -> u32, saturating).
pub fn pl_80040b3c(a: i32) -> u32 {
    ext(a).x0_staleMoveTable.xC6C as u32
}
/// `pl_80040B64` (float -> u32, saturating).
pub fn pl_80040b64(a: i32) -> u32 {
    ext(a).x0_staleMoveTable.xC60 as u32
}
/// `pl_80040B8C`.
pub fn pl_80040b8c(slot: i32, arg1: i32, arg2: i32) {
    let s = &mut ext(slot).x0_staleMoveTable;
    if arg1 != 0 {
        return;
    }
    s.xC68 = s.xC68.wrapping_add(arg2);
}
/// `pl_80040BD8`.
pub fn pl_80040bd8(a: i32) -> i32 {
    ext(a).x0_staleMoveTable.xC68
}
/// `pl_80040BFC` (float -> u32, saturating).
pub fn pl_80040bfc(a: i32) -> u32 {
    ext(a).x0_staleMoveTable.xC64 as u32
}
/// `pl_80040C24`.
pub fn pl_80040c24(a: i32) -> f32 {
    ext(a).xD74
}
/// `pl_80040C48`.
pub fn pl_80040c48(a: i32) -> f32 {
    ext(a).xD78
}
/// `pl_80040C6C`.
pub fn pl_80040c6c(a: i32) -> f32 {
    ext(a).xD7C
}
/// `pl_80040C90`.
pub fn pl_80040c90(a: i32) -> f32 {
    ext(a).xD80
}
/// `pl_80040CB4`.
pub fn pl_80040cb4(a: i32) -> i32 {
    ext(a).xD24 as i32
}
/// `pl_80040CD8`.
pub fn pl_80040cd8(a: i32) -> i32 {
    ext(a).xD20 as i32
}
/// `pl_80040CFC`.
pub fn pl_80040cfc(a: i32) -> u32 {
    ext(a).xD04
}
/// `pl_80040D20`.
pub fn pl_80040d20(a: i32) -> i32 {
    ext(a).xD68
}
/// `pl_80040D44`.
pub fn pl_80040d44(a: i32) -> f32 {
    ext(a).xD84
}
/// `pl_80040D68`.
pub fn pl_80040d68(a: i32) -> f32 {
    ext(a).xD88
}
/// `pl_80040D8C`.
pub fn pl_80040d8c(slot: i32, _arg1: i32) {
    let s = &mut ext(slot).x0_staleMoveTable;
    s.xCB0 = s.xCB0.wrapping_add(1);
}
/// `pl_80040DB8`.
pub fn pl_80040db8(slot: i32) -> i32 {
    ext(slot).x0_staleMoveTable.xCB0
}
