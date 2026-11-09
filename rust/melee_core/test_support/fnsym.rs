//! Function identity by symbol, for the C-oracle test harnesses (included with `#[path]`; needs the `backtrace`
//! dev-dependency). rustc may give a small leaf fn a second, crate-local copy (cross-crate inlining: the `.llvm.<n>`
//! twin in the symbol table), so a pointer the library stored (a state table, a callback it installed) and the same
//! fn named in the test (`m::f as usize`) can be different addresses. Two addresses are the same function when their
//! symbols, demangled without the hash and the `.llvm.<n>` suffix, are equal. Resolves through the debuginfo or the
//! symbol table (release keeps both), on Linux and Windows alike. Function merging must be off (`require_unmerged`).
#![allow(dead_code)]

use std::collections::HashMap;
use std::sync::Mutex;

static NAMES: Mutex<Option<HashMap<usize, Option<String>>>> = Mutex::new(None);

/// The symbol of the function at `addr` (its entry), `None` when nothing resolves.
pub fn name(addr: usize) -> Option<String> {
    if addr == 0 {
        return None;
    }
    let mut g = NAMES.lock().unwrap_or_else(|e| e.into_inner());
    let map = g.get_or_insert_with(HashMap::new);
    if let Some(n) = map.get(&addr) {
        return n.clone();
    }
    let mut out = None;
    // resolve() looks up `addr - 1` (it takes a return address), so ask for the entry's second byte; with debuginfo the
    // callback lists the frames inlined there first, innermost to outermost: the fn itself is the last one
    backtrace::resolve((addr + 1) as *mut core::ffi::c_void, |s| {
        if let Some(n) = s.name() {
            // `{:#}`: demangled without the trailing hash
            let n = format!("{n:#}");
            out = Some(match n.find(".llvm.") {
                Some(i) => n[..i].to_string(),
                None => n,
            });
        }
    });
    map.insert(addr, out.clone());
    out
}

/// `a` and `b` are the same function: the same address, or two copies of one symbol.
pub fn same(a: usize, b: usize) -> bool {
    static CHECKED: std::sync::Once = std::sync::Once::new();
    CHECKED.call_once(require_unmerged);
    a == b || (a != 0 && b != 0 && matches!((name(a), name(b)), (Some(x), Some(y)) if x == y))
}

// Two distinct functions with one body. LLVM's MergeFunctions (on in optimized rustc builds) folds such twins into
// one address, so a pointer to either names both: distinct C callbacks with identical bodies (empty UnkMotion
// procs, ...) then hash as one id and the C oracle's state-table / callback hashes cannot match. The oracle tests
// run with it off (tools/rust-port/oracle_tests.sh: RUSTC_BOOTSTRAP=1 RUSTFLAGS=-Zmerge-functions=disabled).
#[inline(never)]
extern "C" fn twin_a(p: *mut u32) {
    unsafe { core::ptr::write_volatile(p, core::ptr::read_volatile(p).wrapping_mul(3) ^ 0x5A) }
}
#[inline(never)]
extern "C" fn twin_b(p: *mut u32) {
    unsafe { core::ptr::write_volatile(p, core::ptr::read_volatile(p).wrapping_mul(3) ^ 0x5A) }
}

/// Panics unless the build keeps body-identical functions apart (see `twin_a`).
pub fn require_unmerged() {
    let (a, b) = (core::hint::black_box(twin_a as usize), core::hint::black_box(twin_b as usize));
    assert!(
        a != b,
        "this build merges body-identical functions (rustc MergeFunctions): fn ids cannot match the C oracle; run the \
         oracle tests through tools/rust-port/oracle_tests.sh (RUSTC_BOOTSTRAP=1 RUSTFLAGS=-Zmerge-functions=disabled)"
    );
}

/// The index in `regs` of the function at `addr`: by address, else by symbol.
pub fn find(regs: &[usize], addr: usize) -> Option<usize> {
    static CHECKED: std::sync::Once = std::sync::Once::new();
    CHECKED.call_once(require_unmerged);
    regs.iter().position(|&x| x == addr).or_else(|| {
        let n = name(addr)?;
        regs.iter().position(|&x| name(x).as_deref() == Some(n.as_str()))
    })
}
