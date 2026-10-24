//! `libs/dolphin/src/dolphin/os/OSError.c`: `OSReport`, the debug console printf (`vprintf`).
//!
//! The C is `void OSReport(char* msg, ...)`; here the variadic arguments are a slice of
//! register-width values (what the C-variadic callers of the TUs already pass): `%s` takes a
//! pointer, the integer conversions (`%d %i %u %x %X %o %c`) the low 32 bits, `%f` the bits of a `double`
//! (`%e` / `%g` are not in the formatter's subset: the game does not use them). The text is produced by
//! [`crate::cformat::c_format`] and written to stdout, or to [`OSREPORT_SINK`] when the host
//! installs one (the oracle tests do). Checked against the C `vprintf` of the same formats and
//! arguments by `tests/pure_os_error.rs` (`python rust/melee_runtime/gen/pure/run_pure.py os_error`).
#![allow(non_snake_case, non_upper_case_globals, static_mut_refs, clippy::missing_safety_doc)]

use crate::cformat::{c_format, PrintfArg};
use core::ffi::c_char;

/// Where `OSReport` writes. Unset: stdout.
pub static mut OSREPORT_SINK: Option<fn(&[u8])> = None;

/// Pair the format's conversions with the slice values.
pub unsafe fn os_report_args(fmt: *const c_char, va: &[usize]) -> std::vec::Vec<PrintfArg> {
    let mut out = std::vec::Vec::new();
    let mut p = fmt as *const u8;
    let mut k = 0usize;
    while !p.is_null() && *p != 0 {
        if *p != b'%' {
            p = p.add(1);
            continue;
        }
        p = p.add(1);
        while matches!(*p, b'-' | b'+' | b' ' | b'#' | b'0') {
            p = p.add(1);
        }
        while (*p as char).is_ascii_digit() {
            p = p.add(1);
        }
        if *p == b'.' {
            p = p.add(1);
            while (*p as char).is_ascii_digit() {
                p = p.add(1);
            }
        }
        while matches!(*p, b'h' | b'l' | b'L' | b'z' | b't' | b'j') {
            p = p.add(1);
        }
        let conv = *p;
        if conv == 0 {
            break;
        }
        p = p.add(1);
        let v = va.get(k).copied().unwrap_or(0);
        match conv {
            b'%' => {}
            b's' => {
                out.push(PrintfArg::Str(v as *const i8));
                k += 1;
            }
            b'f' | b'F' | b'e' | b'E' | b'g' | b'G' => {
                out.push(PrintfArg::Dbl(f64::from_bits(v as u64)));
                k += 1;
            }
            _ => {
                out.push(PrintfArg::Int(v as u32 as i32));
                k += 1;
            }
        }
    }
    out
}

/// The text `OSReport(fmt, ...)` prints.
pub unsafe fn os_report_text(fmt: *const c_char, va: &[usize]) -> std::vec::Vec<u8> {
    let args = os_report_args(fmt, va);
    c_format(fmt as *const i8, &args)
}

fn emit(bytes: &[u8]) {
    use std::io::Write;
    unsafe {
        if let Some(f) = OSREPORT_SINK {
            f(bytes);
            return;
        }
    }
    let _ = std::io::stdout().write_all(bytes);
}

/// `OSReport(msg, ...)`.
pub unsafe fn OSReport(msg: *const c_char, va: &[usize]) {
    emit(&os_report_text(msg, va));
}

/// `OSReport` of an already formatted message (callers that format in Rust).
pub fn OSReport_str(msg: &str) {
    emit(msg.as_bytes());
}
