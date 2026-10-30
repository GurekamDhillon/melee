//! Shared plumbing of the lb* TU ports (lbfile, lbgx, lbheap, lbmemory, lbmthp, lbrefract, lbshadow):
//! the settable-hook macro behind every TU's `ext` module and the two callees every lb TU reaches
//! (`OSReport`, `__assert`) as hooks.
#![allow(non_snake_case, non_upper_case_globals, dead_code, static_mut_refs)]

use core::ffi::c_char;

/// `__OSBusClock` (the GameCube bus clock, read from 0x800000F8 by the C macros).
pub const OS_BUS_CLOCK: u32 = 162_000_000;

/// One variadic argument of `OSReport` (`%d`/`%u`/`%x`, `%s`).
#[derive(Clone, Copy, Debug)]
pub enum OsArg {
    I(i32),
    U(u32),
    S(*const c_char),
}

/// Declares a settable hook `$h` for the unported callee `$name` and the function `$name` that calls it
/// (or, with `= default`, the default when the hook is unset: the port of that callee). An unset
/// hook without default panics "TODO: <name> is not ported".
macro_rules! lb_hook {
    ($h:ident, $name:ident, ($($a:ident: $t:ty),*) $(-> $r:ty)? = $def:expr) => {
        pub static mut $h: Option<fn($($t),*) $(-> $r)?> = None;
        #[allow(non_snake_case)]
        pub fn $name($($a: $t),*) $(-> $r)? {
            match unsafe { $h } {
                Some(f) => f($($a),*),
                None => { let d: fn($($t),*) $(-> $r)? = $def; d($($a),*) }
            }
        }
    };
    ($h:ident, $name:ident, ($($a:ident: $t:ty),*) $(-> $r:ty)?) => {
        pub static mut $h: Option<fn($($t),*) $(-> $r)?> = None;
        #[allow(non_snake_case)]
        pub fn $name($($a: $t),*) $(-> $r)? {
            match unsafe { $h } {
                Some(f) => f($($a),*),
                None => panic!(concat!("TODO: ", stringify!($name), " is not ported")),
            }
        }
    };
}
pub(crate) use lb_hook;

lb_hook!(H_OSReport, OSReport, (fmt: &str, args: &[OsArg]));

pub static mut H___assert: Option<fn(&str, u32, &str)> = None;
/// `__assert(file, line, cond)` (sysdolphin/baselib/debug.h, `ATTRIBUTE_NORETURN`): the hook may
/// log; it (or this function) then panics, the game would hang in OSPanic.
pub fn __assert(file: &str, line: u32, cond: &str) -> ! {
    melee_core::ct!("__assert");
    if let Some(f) = unsafe { H___assert } {
        f(file, line, cond);
    }
    panic!("__assert: {file}:{line}: {cond}")
}
