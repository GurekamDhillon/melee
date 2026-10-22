//! TU state-oracle runtime (Rust side): the twin of oracle_tu/tu_oracle.h. Value streams, FNV
//! mixing, pointer classes and the mock-call trace must stay identical to the C header.
#![allow(static_mut_refs, dead_code)]

pub static mut ST: u64 = 0; // fill / argument stream
pub static mut MST: u64 = 0; // mock-return stream
pub static mut TRACE: u64 = 0;
pub static mut H: u64 = 0;

pub const FNV0: u64 = 0xCBF29CE484222325;

pub fn next(s: &mut u64) -> u64 {
    *s = s.wrapping_add(0x9E3779B97F4A7C15);
    let mut z = *s;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
    z ^ (z >> 31)
}
pub fn mix(h: u64, v: u64) -> u64 {
    (h ^ v).wrapping_mul(0x100000001B3)
}

pub static mut CONSTS: Vec<i64> = Vec::new();

pub fn draw_int(s: &mut u64) -> i64 {
    let r = next(s);
    let v = (r >> 3) as i64;
    match r & 7 {
        0 | 1 => 0,
        2 => 1 + (v % 3),
        3 => v & 0xFF,
        4 => (v % 41) - 20,
        5 => (v % 2001) - 1000,
        6 => v & 0xFFFF,
        _ => unsafe {
            if !CONSTS.is_empty() && (v >> 40) & 1 != 0 {
                return CONSTS[((v >> 8) as u64 % CONSTS.len() as u64) as usize];
            }
            (v ^ (v >> 32)) as i32 as i64
        },
    }
}
pub fn draw_range(s: &mut u64, lo: i64, hi: i64) -> i64 {
    let r = next(s);
    lo + (r % ((hi - lo + 1) as u64)) as i64
}
pub fn draw_f32(s: &mut u64) -> f32 {
    let r = next(s);
    let v = (r >> 8) as i64;
    match r & 7 {
        0 | 1 => 0.0,
        2 => ((v % 4001) - 2000) as f32 / 100.0,
        3 => ((v % 41) - 20) as f32,
        4 => ((v % 6001) - 3000) as f32 / 2.0,
        5 => ((v % 201) - 100) as f32 / 1000.0,
        6 => ((v % 20001) - 10000) as f32 / 100.0,
        _ => (v % 3) as f32,
    }
}
pub fn draw_f64(s: &mut u64) -> f64 {
    let r = next(s);
    let v = (r >> 8) as i64;
    match r & 7 {
        0 | 1 => 0.0,
        2 => ((v % 4001) - 2000) as f64 / 100.0,
        3 => ((v % 41) - 20) as f64,
        4 => ((v % 6001) - 3000) as f64 / 2.0,
        5 => ((v % 201) - 100) as f64 / 1000.0,
        6 => ((v % 20001) - 10000) as f64 / 100.0,
        _ => (v % 3) as f64,
    }
}

// ---- regions ----
pub struct Reg {
    pub base: u64,
    pub size: u64,
    pub name: &'static str,
}
pub static mut REGS: Vec<Reg> = Vec::new();
pub static mut NSTATIC: usize = 0;
pub fn reg_add<T>(p: *const T, size: usize, name: &'static str) {
    unsafe { REGS.push(Reg { base: p as usize as u64, size: size as u64, name }) }
}
pub fn ptr_class<T>(p: *const T) -> u64 {
    let a = p as usize as u64;
    if a == 0 {
        return 0;
    }
    unsafe {
        for (i, r) in REGS.iter().enumerate() {
            if a >= r.base && a < r.base + r.size {
                return (((i + 1) as u64) << 32) | (a - r.base);
            }
        }
        for (i, r) in REGS.iter().enumerate() {
            if a == r.base + r.size {
                return (((i + 1) as u64) << 32) | (a - r.base);
            }
        }
    }
    1
}

/// Bytes readable from `p` as a string: to the end of its region, else 512.
pub fn str_limit<T>(p: *const T) -> u64 {
    let a = p as usize as u64;
    unsafe {
        for r in REGS.iter() {
            if a >= r.base && a < r.base + r.size {
                return r.base + r.size - a;
            }
        }
    }
    512
}

// ---- function pointers ----
pub static mut FNS: Vec<u64> = Vec::new();
pub fn fn_add(a: usize) {
    unsafe { FNS.push(a as u64) }
}
pub fn fn_class(a: usize) -> u64 {
    let a = a as u64;
    if a == 0 {
        return 0;
    }
    unsafe {
        for (i, f) in FNS.iter().enumerate() {
            if *f == a {
                return 0x1000 + i as u64;
            }
        }
    }
    1
}

// ---- trace ----
pub fn t_begin(id: i32) {
    unsafe { TRACE = mix(TRACE, 0xC0DE0000u64.wrapping_add(id as i64 as u64)) }
}
pub fn t_i(v: i64) {
    unsafe { TRACE = mix(TRACE, v as u64) }
}
pub fn t_f32(v: f32) {
    unsafe { TRACE = mix(TRACE, v.to_bits() as u64) }
}
pub fn t_f64(v: f64) {
    unsafe { TRACE = mix(TRACE, v.to_bits()) }
}
pub fn t_p<T>(p: *const T) {
    unsafe { TRACE = mix(TRACE, ptr_class(p)) }
}
pub fn t_fn(a: usize) {
    unsafe { TRACE = mix(TRACE, fn_class(a)) }
}
/// # Safety
/// `s` is NULL or NUL-terminated.
pub unsafe fn t_s(s: *const u8) {
    if s.is_null() {
        TRACE = mix(TRACE, 0xFFFF);
        return;
    }
    let mut i = 0;
    while i < 512 && (i as u64) < str_limit(s) && *s.add(i) != 0 {
        TRACE = mix(TRACE, *s.add(i) as u64);
        i += 1;
    }
    TRACE = mix(TRACE, 0xEEEE);
}

// ---- state hash ----
pub fn h_i(v: i64) {
    unsafe { H = mix(H, v as u64) }
}
pub fn h_u(v: u64) {
    unsafe { H = mix(H, v) }
}
pub fn h_f32(v: f32) {
    unsafe { H = mix(H, v.to_bits() as u64) }
}
pub fn h_f64(v: f64) {
    unsafe { H = mix(H, v.to_bits()) }
}
pub fn h_p<T>(p: *const T) {
    unsafe { H = mix(H, ptr_class(p)) }
}
pub fn h_fn(a: usize) {
    unsafe { H = mix(H, fn_class(a)) }
}
/// # Safety
/// `p` is valid for `n` bytes.
pub unsafe fn h_bytes(p: *const u8, n: usize) {
    for i in 0..n {
        H = mix(H, *p.add(i) as u64);
    }
}
pub static mut OBJ_HASH: Vec<u64> = Vec::new();

/// # Safety
/// `s` is NULL or NUL-terminated.
pub unsafe fn h_cstr(s: *const u8) {
    if s.is_null() {
        return;
    }
    let mut i = 0;
    while i < 512 && (i as u64) < str_limit(s) && *s.add(i) != 0 {
        H = mix(H, *s.add(i) as u64);
        i += 1;
    }
    H = mix(H, 0xEEEE);
}
pub static mut DYN: Vec<u64> = Vec::new();
