//! Shared helpers of the lb* TU state oracles (rust/melee_modes/tests/tu_lb*.rs): mirrors of
//! gen/turef/tu_gm_common.h (xorshift32 stream, capped log) and the comparison with the data file the
//! C driver wrote (python rust/melee_modes/gen/turef/tu_shard833.py).
#![allow(dead_code, static_mut_refs)]

pub static mut S: u32 = 0x2468ACE;
pub fn rnd() -> u32 {
    unsafe {
        S ^= S << 13;
        S ^= S >> 17;
        S ^= S << 5;
        S
    }
}
/// tu_gm_common.h `rf()`.
pub fn rf() -> f32 {
    match rnd() % 8 {
        0 => 0.0,
        1 => -0.0,
        2 => 1.0,
        3 => -1.0,
        _ => ((rnd() % 4001) as i32 - 2000) as f32 / 997.0f32,
    }
}

/// The callee log (`g_log`, capped at 8191 bytes like the C one).
pub static mut LOG: String = String::new();
pub fn lg(s: &str) {
    unsafe {
        for c in s.chars() {
            if LOG.len() + 1 < 8192 {
                LOG.push(c);
            }
        }
    }
}
pub fn log_take() -> String {
    unsafe { core::mem::take(&mut LOG) }
}

pub static mut OUT: String = String::new();
pub fn out(s: &str) {
    unsafe { OUT.push_str(s) }
}
#[macro_export]
macro_rules! outf { ($($a:tt)*) => { crate::common_lb::out(&format!($($a)*)) } }

/// Compare the produced text with the C driver's output, reporting the first differing line.
pub fn check(stem: &str) {
    let path = format!("{}/tests/data/{}.txt", env!("CARGO_MANIFEST_DIR"), stem);
    let want = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"));
    let got = unsafe { OUT.clone() };
    let gl: Vec<&str> = got.lines().collect();
    let wl: Vec<&str> = want.lines().collect();
    for i in 0..gl.len().max(wl.len()) {
        if gl.get(i) != wl.get(i) {
            let ctx = |v: &Vec<&str>| v[i.saturating_sub(2)..(i + 2).min(v.len())].join(" | ");
            panic!("{stem}: first difference at line {}\n  rust: {}\n  c:    {}", i + 1, ctx(&gl), ctx(&wl));
        }
    }
    assert!(!gl.is_empty(), "{stem}: no output");
}

// ---- pointer registry (mirror of tu_lb_common.h)
use core::ffi::{c_char, c_void, CStr};
use melee_modes::tu::lbext::{self, OsArg};

static mut PREGS: Vec<(&'static str, u64, u64)> = Vec::new();
static mut PFNS: Vec<(&'static str, u64)> = Vec::new();
pub fn preg<T>(name: &'static str, p: *const T, n: usize) {
    unsafe { PREGS.push((name, p as u64, n as u64)) }
}
pub fn preg_clear() {
    unsafe { PREGS.clear() }
}
pub fn pfreg(name: &'static str, f: usize) {
    unsafe { PFNS.push((name, f as u64)) }
}
pub fn pidu(v: u64) -> String {
    pidx(v, true)
}
fn pidx(v: u64, allow_num: bool) -> String {
    unsafe {
        for (n, b, len) in PREGS.iter() {
            if v >= *b && v < *b + *len {
                return format!("{}+{}", n, v - *b);
            }
        }
        for (n, f) in PFNS.iter() {
            if *f == v {
                return format!("f:{}", n);
            }
        }
    }
    if v == 0 {
        "null".into()
    } else if !allow_num || v > 0xFFFF_FFFF {
        "?".into()
    } else {
        format!("n{}", v)
    }
}
pub fn pid(p: *const c_void) -> String {
    pidx(p as u64, false)
}
pub fn pidf(f: usize) -> String {
    pidx(f as u64, false)
}
pub fn pids(s: *const c_char) -> String {
    if s.is_null() {
        return "null".into();
    }
    let b = unsafe { CStr::from_ptr(s) }.to_bytes();
    let b = &b[..b.len().min(40)];
    format!("\"{}\"", String::from_utf8_lossy(b))
}

/// Mirror of `os_report_va`.
pub fn fmt_report(fmt: &str, args: &[OsArg]) -> String {
    let f = fmt.as_bytes();
    let mut i = 0;
    let mut k = 0;
    let mut o = String::new();
    let pad = |s: String, width: usize, zp: bool| -> String {
        let mut s = s;
        while s.len() < width {
            s.insert(0, if zp { '0' } else { ' ' });
        }
        s
    };
    while i < f.len() {
        if f[i] != b'%' {
            o.push(f[i] as char);
            i += 1;
            continue;
        }
        i += 1;
        let mut zp = false;
        let mut width = 0usize;
        if f[i] == b'0' {
            zp = true;
            i += 1;
        }
        while f[i].is_ascii_digit() {
            width = width * 10 + (f[i] - b'0') as usize;
            i += 1;
        }
        let c = f[i];
        i += 1;
        let a = args[k];
        k += 1;
        match (c, a) {
            (b'd', OsArg::I(v)) => {
                if v < 0 {
                    o.push('-');
                    o.push_str(&pad(format!("{}", (v as i64).unsigned_abs()), width.saturating_sub(1), zp));
                } else {
                    o.push_str(&pad(format!("{}", v), width, zp));
                }
            }
            (b'd', OsArg::U(v)) => o.push_str(&pad(format!("{}", v as i32), width, zp)),
            (b'u', OsArg::U(v)) => o.push_str(&pad(format!("{}", v), width, zp)),
            (b'u', OsArg::I(v)) => o.push_str(&pad(format!("{}", v as u32), width, zp)),
            (b'x', OsArg::U(v)) => o.push_str(&pad(format!("{:x}", v), width, zp)),
            (b'x', OsArg::I(v)) => o.push_str(&pad(format!("{:x}", v as u32), width, zp)),
            (b's', OsArg::S(p)) => o.push_str(unsafe { CStr::from_ptr(p) }.to_str().unwrap()),
            _ => panic!("fmt_report {fmt}"),
        }
    }
    o
}

/// Installs the shared `OSReport` / `__assert` hooks (they log like tu_lb_common.h).
pub fn install_common() {
    unsafe {
        lbext::H_OSReport = Some(|fmt, args| lg(&format!("OSReport({});", fmt_report(fmt, args))));
        lbext::H___assert = Some(|_f, l, c| lg(&format!("assert({},{});", l, c)));
    }
    std::panic::set_hook(Box::new(|i| unsafe {
        if !IN_TRY {
            eprintln!("{i}");
        }
    }));
}
/// TRY: runs `f`; true when it hit `__assert` (a panic).
static mut IN_TRY: bool = false;
pub fn try_run<F: FnOnce()>(f: F) -> bool {
    unsafe { IN_TRY = true };
    let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)).is_err();
    unsafe { IN_TRY = false };
    r
}

// ---- arena below 4 GB (mirror of tu_lb_common.h)
extern "system" {
    fn VirtualAlloc(addr: *mut u8, size: usize, ty: u32, prot: u32) -> *mut u8;
}
static mut ARENA_P: *mut u8 = core::ptr::null_mut();
static mut ARENA_USED: usize = 0;
pub fn arena_init() {
    unsafe {
        if ARENA_P.is_null() {
            ARENA_P = VirtualAlloc(0x2000_0000usize as *mut u8, 1 << 20, 0x3000, 4);
            assert!(!ARENA_P.is_null(), "arena_init failed");
        }
        core::ptr::write_bytes(ARENA_P, 0, 1 << 20);
        ARENA_USED = 64;
    }
}
pub fn az(n: usize) -> *mut u8 {
    unsafe {
        let p = ARENA_P.add(ARENA_USED);
        ARENA_USED += (n + 15) & !15;
        p
    }
}
pub fn arena_at(off: usize) -> *mut u8 {
    unsafe { ARENA_P.add(off) }
}
