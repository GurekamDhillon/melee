//! Test hook ABI of the generated `ext` modules in `crate::tu::*`.
//!
//! Every callee a ported TU takes from a TU that is not ported yet is an `ext` function named like
//! the C function. Unhooked it panics with a TODO (or forwards to a port, once one exists). The
//! state-oracle tests (tests/s497_*.rs) install [`Hook`] in each `ext::HOOK`: the hook gets the
//! callee's name and its arguments in the C-ABI-neutral form [`A`] and returns the value the C
//! stub of the oracle harness returns (bits, widened to u64).
use core::ffi::c_char;

/// One argument of a hooked call.
#[derive(Clone, Copy, Debug)]
pub enum A {
    /// integer, bool or enum, zero-extended from its own width
    I(u64),
    F(f32),
    D(f64),
    /// pointer or function pointer
    P(usize),
    /// `char*` / `const char*`
    S(*const c_char),
    /// pointer argument traced by the hash of the `n` bytes behind it (pointers to stack locals)
    M(usize, usize),
    /// out parameter: traced as `O`, not by address
    O(usize),
}

pub type Hook = unsafe fn(&'static str, &[A]) -> u64;
