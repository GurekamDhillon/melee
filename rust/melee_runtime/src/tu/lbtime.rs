//! Port of src/melee/lb/lbtime.c: saturating add helpers for u32 / u16 / u8 timers and the OS clock readers.
//! Callees of unported TUs go through `ext` (unset: panics "TODO"). Checked against the C (whole TU,
//! shipping contraction) by tests/lbtu_lbtime.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, unused_variables, unused_mut, dead_code)]

use core::ffi::c_void;
use melee_types::*;

// BEGIN GENERATED EXT (rust/melee_runtime/gen/lbtu/gen_tu.py; the callees of the C TU)
pub mod ext {
    #![allow(non_upper_case_globals, non_snake_case, unused_imports, dead_code, unreachable_code)]
    use super::*;
    use core::ffi::c_void;
    pub mod hook {
        use super::*;
        pub static mut OSGetTime: Option<fn() -> i64> = None;
        pub static mut OSTicksToCalendarTime: Option<fn(i64, *mut OSCalendarTime)> = None;
    }
    pub fn OSGetTime() -> i64 {
        match unsafe { hook::OSGetTime } { Some(f) => f(), None => { melee_core::unported!("TODO: OSGetTime is not ported") } }
    }
    pub fn OSTicksToCalendarTime(a0: i64, a1: *mut OSCalendarTime) {
        match unsafe { hook::OSTicksToCalendarTime } { Some(f) => f(a0, a1), None => { melee_core::unported!("TODO: OSTicksToCalendarTime is not ported") } }
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
        pub unsafe extern "C" fn OSGetTime() -> i64 { super::OSGetTime() }
    }
    // Data defined in other TUs (the tests set them).
    pub use crate::tu::perf::__OSBusClock;
}
// END GENERATED EXT

/// `__OSBusClock / 4` (`OS_TIMER_CLOCK`), a u32.
#[inline]
unsafe fn os_timer_clock() -> u32 {
    ext::__OSBusClock / 4
}

pub fn lbTime_8000AEC8(a: u32, b: u32) -> u32 {
    melee_core::ct!("lbTime_8000AEC8");
    if (u32::MAX.wrapping_sub(a)) > b { a.wrapping_add(b) } else { u32::MAX }
}

pub fn lbTime_8000AEE4(a: u32, b: i32) -> u32 {
    melee_core::ct!("lbTime_8000AEE4");
    if b > 0 {
        if u32::MAX.wrapping_sub(a) > b as u32 { a.wrapping_add(b as u32) } else { u32::MAX }
    } else if a > b.wrapping_neg() as u32 {
        a.wrapping_add(b as u32)
    } else {
        0
    }
}

pub fn lbTime_8000AF24(a: u32, b: i32) -> u32 {
    melee_core::ct!("lbTime_8000AF24");
    if b > 0 {
        let ab = a & 0xFFFF;
        let c: i32 = 0xFFFF;
        if (c as u32).wrapping_sub(ab) > b as u32 { a.wrapping_add(b as u32) } else { c as u32 }
    } else {
        let c = (a & 0xFFFF) as i32;
        if c > b.wrapping_neg() { a.wrapping_add(b as u32) } else { 0 }
    }
}

pub fn lbTime_8000AF74(a: u32, b: i32) -> u32 {
    melee_core::ct!("lbTime_8000AF74");
    if b > 0 {
        let lo = 0xFFu32 - (a & 0xFF);
        // `lo > b`: u32 vs int compares unsigned
        return if lo > b as u32 { a.wrapping_add(b as u32) } else { 0xFF };
    }
    let c = (a & 0xFF) as i32;
    if c > b.wrapping_neg() { a.wrapping_add(b as u32) } else { 0 }
}

pub unsafe fn lbTime_GetTimeInSeconds() -> u32 {
    melee_core::ct!("lbTime_GetTimeInSeconds");
    let ticks: i64 = ext::OSGetTime();
    // OSTime / u32 clock: the u32 converts to s64, so a negative tick count stays negative, then u64
    let mut secs = ticks.wrapping_div(os_timer_clock() as i64) as u64;
    if secs > u32::MAX as u64 {
        secs = u32::MAX as u64;
    }
    secs as u32
}

pub unsafe fn lbTime_8000B028(td: *mut OSCalendarTime, secs: u32) {
    melee_core::ct!("lbTime_8000B028");
    ext::OSTicksToCalendarTime((secs as u64).wrapping_mul(os_timer_clock() as u64) as i64, td)
}
