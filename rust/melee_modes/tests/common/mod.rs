//! Rust mirror of rust/melee_modes/gen/turef/oracle.h: the seeded generator, the arena, the
//! trace/dump printers. A TU test replays the C driver's cases on the Rust port and compares
//! the whole output text with tests/data/tu_<stem>.txt (produced by the C driver).
#![allow(dead_code, static_mut_refs)]

use core::ffi::{c_char, CStr};

pub static mut ORS: u32 = 1;
pub fn rnd() -> u32 {
    unsafe {
        ORS ^= ORS << 13;
        ORS ^= ORS >> 17;
        ORS ^= ORS << 5;
        ORS
    }
}
pub fn rseed(s: u32) {
    unsafe { ORS = if s != 0 { s } else { 1 } }
}
pub fn rf() -> f32 {
    let k = rnd() % 16;
    if k == 0 {
        return 0.0;
    }
    if k == 1 {
        return -0.0;
    }
    if k == 2 {
        return (rnd() % 7) as f32 * 1e-4f32;
    }
    ((rnd() % 200001) as i32 - 100000) as f32 / 37.0f32
}

pub const ARENA_SZ: usize = 1 << 22;
extern "system" {
    fn VirtualAlloc(addr: *mut u8, size: usize, ty: u32, prot: u32) -> *mut u8;
}
pub struct Arena(pub *mut u8);
/// The arena lives below 4 GB at a fixed address: the game casts pointers through int/u32.
pub static mut ARENA: Arena = Arena(core::ptr::null_mut());
pub fn arena_init() {
    unsafe {
        if ARENA.0.is_null() {
            ARENA.0 = VirtualAlloc(0x2000_0000usize as *mut u8, ARENA_SZ, 0x3000, 4);
            assert!(!ARENA.0.is_null(), "arena_init failed");
        }
    }
}
pub static mut ARENA_USED: u64 = 64;
pub fn arena_reset() {
    unsafe {
        let n = ARENA_USED as usize;
        core::ptr::write_bytes(ARENA.0, 0, n);
        ARENA_USED = 64;
    }
}
pub fn az(n: usize) -> *mut u8 {
    unsafe {
        let p = ARENA.0.add(ARENA_USED as usize);
        ARENA_USED += (n as u64 + 15) & !15u64;
        p
    }
}
pub fn azr(n: usize) -> *mut u8 {
    let p = az(n);
    rfill(p, n);
    p
}
pub fn dump_arena() {
    unsafe { dump("arena", ARENA.0.add(64), ARENA_USED as usize - 64) }
}
pub fn az_t<T>() -> *mut T {
    az(core::mem::size_of::<T>()) as *mut T
}
pub fn arena_base() -> u64 {
    unsafe { ARENA.0 as u64 }
}

static mut REGS: Vec<(&'static str, u64, u64)> = Vec::new();
static mut FNS: Vec<(&'static str, u64)> = Vec::new();
pub static mut OUT: String = String::new();

pub fn reset_all() {
    unsafe {
        REGS.clear();
        FNS.clear();
        OUT.clear();
        ORS = 1;
    }
    arena_init();
    arena_reset();
}
pub fn reg<T>(name: &'static str, p: *const T, n: usize) {
    unsafe { REGS.push((name, p as u64, n as u64)) }
}
pub fn fnreg(name: &'static str, f: usize) {
    unsafe { FNS.push((name, f as u64)) }
}
pub fn out(s: &str) {
    unsafe { OUT.push_str(s) }
}

pub fn trp(p: *const u8) {
    let v = p as u64;
    if v == 0 {
        return out(" null");
    }
    let base = arena_base();
    if v >= base && v < base + ARENA_SZ as u64 {
        return out(&format!(" a{}", v - base));
    }
    unsafe {
        for (name, b, n) in REGS.iter() {
            if v >= *b && v < *b + *n {
                return out(&format!(" {}+{}", name, v - *b));
            }
        }
        for (name, f) in FNS.iter() {
            if *f == v {
                return out(&format!(" f:{}", name));
            }
        }
    }
    out(" x")
}
pub fn tri(x: i64) {
    out(&format!(" {}", x))
}
pub fn tru(x: u64) {
    out(&format!(" {}", x))
}
pub fn trf(x: f32) {
    out(&format!(" {:08x}", x.to_bits()))
}
pub fn trd(x: f64) {
    out(&format!(" {:016x}", x.to_bits()))
}
pub fn trs(s: *const c_char) {
    if s.is_null() {
        return out(" null");
    }
    let v = s as u64;
    let base = arena_base();
    if v >= base && v < base + ARENA_SZ as u64 {
        return trp(s as *const u8);
    }
    let mut st = String::new();
    for &b in unsafe { CStr::from_ptr(s) }.to_bytes() {
        if b < 0x20 || b >= 0x7f {
            st.push_str(&format!("\\x{:02x}", b));
        } else {
            st.push(b as char);
        }
    }
    out(&format!(" \"{}\"", st))
}

pub fn dump(tag: &str, p: *const u8, n: usize) {
    let b = unsafe { core::slice::from_raw_parts(p, n) };
    let mut h: u64 = 0xcbf29ce484222325;
    let base = arena_base();
    let mut i = 0usize;
    while i < n {
        if i & 7 == 0 && i + 8 <= n {
            let mut w = u64::from_le_bytes(b[i..i + 8].try_into().unwrap());
            if w >= base && w < base + ARENA_SZ as u64 {
                w = 0xA0A0000000000000 | (w - base);
            } else {
                let mut done = false;
                unsafe {
                    for (k, (_, rb, rn)) in REGS.iter().enumerate() {
                        if w >= *rb && w < *rb + *rn {
                            w = 0xB0B0000000000000 | ((k as u64) << 32) | (w - *rb);
                            done = true;
                            break;
                        }
                    }
                    if !done {
                        for (k, (_, f)) in FNS.iter().enumerate() {
                            if *f == w {
                                w = 0xC0C0000000000000 | k as u64;
                                break;
                            }
                        }
                    }
                }
            }
            for j in 0..8 {
                h ^= (w >> (8 * j)) & 0xff;
                h = h.wrapping_mul(0x100000001b3);
            }
            i += 8;
            continue;
        }
        h ^= b[i] as u64;
        h = h.wrapping_mul(0x100000001b3);
        i += 1;
    }
    if std::env::var_os("ORACLE_VERBOSE").is_some() {
        out(&format!("dump {} {} {:016x}\n", tag, n, h));
        if std::env::var_os("ORACLE_HEX").is_some() {
            out("hex");
            for q in 0..n {
                out(&format!(" {:02x}", b[q]));
            }
            out("\n");
        }
    } else {
        unsafe {
            HACC = (HACC ^ h).wrapping_mul(0x100000001b3);
            HACC ^= n as u64;
        }
    }
}
static mut HACC: u64 = 0xcbf29ce484222325;
/// End of a case's state: the folded hash of every dump since the last one.
pub fn state_end() {
    unsafe {
        if std::env::var_os("ORACLE_VERBOSE").is_none() {
            out(&format!("state {:016x}\n", HACC));
        }
        HACC = 0xcbf29ce484222325;
    }
}
pub fn dump_t<T>(tag: &str, v: *const T) {
    dump(tag, v as *const u8, core::mem::size_of::<T>())
}
pub fn size_t<T>(name: &str) {
    out(&format!("sizeof {} {}\n", name, core::mem::size_of::<T>()));
}
pub fn case(i: usize) {
    out(&format!("case {}\n", i));
}
pub fn rfill(p: *mut u8, n: usize) {
    for i in 0..n {
        unsafe { *p.add(i) = (rnd() >> 11) as u8 };
    }
}
pub fn rfill_t<T>(p: *mut T) {
    rfill(p as *mut u8, core::mem::size_of::<T>())
}

/// Compare the produced text with the C driver's output, reporting the first differing line.
pub fn check(stem: &str) {
    let path = std::env::var("ORACLE_DATA").unwrap_or_else(|_| format!("{}/tests/data/{}.txt", env!("CARGO_MANIFEST_DIR"), stem));
    let want = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"));
    let got = unsafe { OUT.clone() };
    let gl: Vec<&str> = got.lines().collect();
    let wl: Vec<&str> = want.lines().collect();
    for i in 0..gl.len().max(wl.len()) {
        if gl.get(i) != wl.get(i) {
            let ctx = |v: &Vec<&str>| v[i.saturating_sub(3)..(i + 3).min(v.len())].join(" | ");
            panic!("{stem}: first difference at line {}
  rust: {}
  c:    {}", i + 1, ctx(&gl), ctx(&wl));
        }
    }
}
