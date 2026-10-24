//! `OSReport` (Dolphin `os`): formatted text to the debug console. Natively the console is the
//! process's stderr; a host that wants the text (the PC build's log) installs [`SINK`].
#![allow(non_snake_case, non_upper_case_globals, static_mut_refs)]

/// Where the text goes instead of stderr.
pub static mut SINK: Option<fn(&str)> = None;

/// `OSReport(msg)` for an already-formatted message.
pub fn OSReport(msg: &str) {
    match unsafe { SINK } {
        Some(f) => f(msg),
        None => eprint!("{msg}"),
    }
}
