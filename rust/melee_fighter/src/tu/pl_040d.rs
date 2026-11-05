//! Native port of src/melee/pl/pl_040D.c: per-player "who hit whom" bonus bookkeeping
//! (`pl_x5EC_t` at StaticPlayer +0x6A8 and the `pl_x8C0_t` / `pl_x8D8_t` rings in the stale-move
//! table extension).
//!
//! Every definition of the TU already has a port in `crate::plbonus` (the whole `pl_80040DDC ..
//! pl_80041BFC` range); they are re-exported here under the C names.
#![allow(non_snake_case, non_upper_case_globals, clippy::missing_safety_doc)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use crate::ftcommon::abs_c;
#[allow(unused_imports)]
use crate::hooks as h;
#[allow(unused_imports)]
use crate::player::{get_entity_at_index, get_team, get_unk6a8_ptr, player_8003221c};
#[allow(unused_imports)]
use crate::stale::action_stats;
#[allow(unused_imports)]
use crate::types::*;
#[allow(unused_imports)]
use crate::plbonus::{calculate_average, counts_hit, ext, k, x5ec};
pub use crate::plbonus::{fn_8004138c as fn_8004138C, fn_800415b0 as fn_800415B0, fn_80041bc8 as fn_80041BC8, pl_80040ddc as pl_80040DDC, pl_80040ed4 as pl_80040ED4, pl_80040fbc as pl_80040FBC, pl_800410f4 as pl_800410F4, pl_800411c4 as pl_800411C4, pl_80041280 as pl_80041280, pl_800412d0 as pl_800412D0, pl_8004134c as pl_8004134C, pl_800414c0 as pl_800414C0, pl_80041524 as pl_80041524, pl_80041720 as pl_80041720, pl_80041744 as pl_80041744, pl_8004182c as pl_8004182C, pl_800418f4 as pl_800418F4, pl_800419ac as pl_800419AC, pl_80041b08 as pl_80041B08, pl_80041bfc as pl_80041BFC};
// ---- folded from plbonus.rs (consolidation steps 21/22) ----
// ---- pl_040D.c ---------------------------------------------------------------------------

/// `pl_80040DDC`.
pub fn pl_80040ddc(slot: i32) {
    melee_core::ct!("pl_80040ddc");
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
    melee_core::ct!("pl_80040ed4");
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
    melee_core::ct!("pl_80040fbc");
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
    melee_core::ct!("pl_800410f4");
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
    melee_core::ct!("pl_800411c4");
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

/// `pl_80041280_fp`.
pub fn pl_80041280_fp(slot: i32, arg1: i32) {
    melee_core::ct!("pl_80041280_fp");
    if arg1 != 1 {
        for e in unsafe { (*x5ec(slot)).x10.iter_mut() } {
            e.x10 = 0;
        }
    }
}

/// `pl_800412D0` (float -> int, fctiwz via melee_core::ppc).
pub fn pl_800412d0(slot: i32) -> i32 {
    melee_core::ct!("pl_800412d0");
    unsafe { melee_core::ppc::fctiwz_f32((*x5ec(slot)).x4) }
}

/// `pl_80041300`.
pub fn pl_80041300(slot: i32) -> i32 {
    melee_core::ct!("pl_80041300");
    let t = unsafe { &*x5ec((*x5ec(slot)).x0 as i32) };
    melee_core::ppc::fctiwz_f32(t.x10[slot as usize].x0)
}

/// `pl_8004134C`.
pub fn pl_8004134c(slot: i32) -> i32 {
    melee_core::ct!("pl_8004134c");
    let t = unsafe { &*x5ec((*x5ec(slot)).x0 as i32) };
    t.x10[slot as usize].x4 as i32
}

/// `fn_8004138C`.
pub fn fn_8004138c(arg0: i32, arg1: i32) {
    melee_core::ct!("fn_8004138c");
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
    melee_core::ct!("pl_800414c0");
    let t = unsafe { &*x5ec(arg0) };
    if t.xC != 0 {
        return t.x8 as f32 / t.xC as f32;
    }
    0.0
}

/// `pl_80041524_fp`.
pub fn pl_80041524_fp(arg0: i32) {
    melee_core::ct!("pl_80041524_fp");
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
    melee_core::ct!("fn_800415b0");
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

/// `pl_80041720_fp`.
pub fn pl_80041720_fp(slot: i32, arg1: i32, arg2: u16) {
    melee_core::ct!("pl_80041720_fp");
    fn_800415b0(slot, arg1, arg2, 0);
}

/// `pl_80041744_fp`.
pub fn pl_80041744_fp(arg0: i32, arg1: i32, arg2: u16) {
    melee_core::ct!("pl_80041744_fp");
    if arg1 == 0 {
        fn_800415b0(arg0, arg1, arg2, 1);
    }
}

/// `fn_80041770`.
pub fn fn_80041770(slot: i32, arg1: u8, arg2: u32) {
    melee_core::ct!("fn_80041770");
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
    melee_core::ct!("pl_8004182c");
    for i in 0..5 {
        let e = &ext(slot).x0_staleMoveTable.x8C0.x4[i];
        let (a, b) = (e.x2, e.x3_b0() as u32);
        fn_80041770(slot, a, b);
    }
}

/// `pl_800418F4`.
pub fn pl_800418f4(slot: i32) {
    melee_core::ct!("pl_800418f4");
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
    melee_core::ct!("pl_800419ac");
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
    melee_core::ct!("pl_80041b08");
    for e in ext(arg0).x0_staleMoveTable.x8D8.x4.iter_mut() {
        if arg2 as u32 == e.x0 {
            e.set_x4_b0(1);
        }
    }
}

/// `fn_80041BC8`.
pub fn fn_80041bc8(arg0: i32, arg1: u32, arg2: u32) {
    melee_core::ct!("fn_80041bc8");
    if arg2 != 0 && arg1 == 0 {
        h::pl_80038824(arg0, 0xC8);
    }
}

/// `pl_80041BFC`.
pub fn pl_80041bfc(slot: i32) {
    melee_core::ct!("pl_80041bfc");
    for i in 0..5 {
        let e = &ext(slot).x0_staleMoveTable.x8D8.x4[i];
        let (a, b) = (e.x4_b0() as u32, e.x4_b1() as u32);
        fn_80041bc8(slot, a, b);
    }
}
