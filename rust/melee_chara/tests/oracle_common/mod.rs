//! Rust side of the state oracles (rust/melee_chara/gen/oracle). Every function here repeats the
//! same-named piece of gen/oracle/oracle.h line for line: the seeded generator, the region and
//! function-pointer names, the call log text, the struct fill by field kind. A generated
//! tests/oracle_<stem>.rs drives it; the expected hashes come from the game's own C.
#![allow(dead_code, static_mut_refs, clippy::all)]

use core::ffi::c_void;
use melee_types::{Vec2, Vec3};

pub static mut RS: u32 = 0;
pub fn rnd() -> u32 {
    unsafe {
        RS ^= RS << 13;
        RS ^= RS >> 17;
        RS ^= RS << 5;
        RS
    }
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
        return 1.0;
    }
    if k == 3 {
        return -1.0;
    }
    if k == 4 {
        const BV: [f32; 12] = [0.0005, 0.001, 0.0015, 0.002, 0.01, 0.05, 0.1, 0.2, 0.25, 0.5, 0.75, 0.9];
        let bi = (rnd() % 12) as usize;
        let sg = rnd() & 1;
        return if sg != 0 { -BV[bi] } else { BV[bi] };
    }
    let v = (rnd() % 2001) as i32 - 1000;
    if k < 10 {
        return v as f32 / 1000.0;
    }
    v as f32 / 7.0
}
pub fn iv(sgn: bool) -> u32 {
    let k = rnd() % 8;
    let mut v = if k < 2 {
        0
    } else if k < 5 {
        rnd() % 4
    } else if k < 7 {
        rnd() % 400
    } else {
        rnd() & 0xFFFF
    };
    if sgn {
        let n = rnd() % 6;
        if n == 0 {
            v = (v as i32).wrapping_neg() as u32;
        }
    }
    v
}
pub fn gen_i() -> i32 {
    match rnd() % 4 {
        0 => 0,
        1 => 1,
        2 => 2,
        _ => -1,
    }
}
pub fn gen_b() -> i32 {
    (rnd() & 1) as i32
}

/// NaN payload and sign depend on operand order, not on the game: every NaN is compared as the
/// default quiet NaN (oracle.h `cf32`).
pub fn cf32(b: u32) -> u32 {
    if (b & 0x7F80_0000) == 0x7F80_0000 && (b & 0x7F_FFFF) != 0 {
        0x7FC0_0000
    } else {
        b
    }
}
pub fn cf64(b: u64) -> u64 {
    if (b & 0x7FF0_0000_0000_0000) == 0x7FF0_0000_0000_0000 && (b & 0xF_FFFF_FFFF_FFFF) != 0 {
        0x7FF8_0000_0000_0000
    } else {
        b
    }
}
pub fn bc(f: f32) -> u32 {
    cf32(f.to_bits())
}
pub fn dc(f: f64) -> u64 {
    cf64(f.to_bits())
}
/// Raw bytes of a float array with every NaN canonicalised (the C `hash_f`).
pub fn canon_f32_bytes(v: &[f32]) -> Vec<u8> {
    v.iter().flat_map(|f| bc(*f).to_le_bytes()).collect()
}

pub const FNV0: u64 = 0xCBF2_9CE4_8422_2325;
pub fn fnv(mut h: u64, b: &[u8]) -> u64 {
    for &x in b {
        h ^= x as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01B3);
    }
    h
}
pub fn bytes<T>(v: &T) -> &[u8] {
    unsafe { core::slice::from_raw_parts(v as *const T as *const u8, core::mem::size_of::<T>()) }
}
pub fn bytes_mut<T>(v: &mut T) -> &mut [u8] {
    unsafe { core::slice::from_raw_parts_mut(v as *mut T as *mut u8, core::mem::size_of::<T>()) }
}

// ------------------------------------------------------------- regions, pointer names
pub static mut OREGS: Vec<(usize, usize)> = Vec::new();
pub static mut OFNS: Vec<usize> = Vec::new();
pub fn oreg<T>(b: *const T, n: usize) {
    unsafe { OREGS.push((b as usize, n)) }
}
pub fn ofn(p: usize) {
    unsafe { OFNS.push(p) }
}
pub fn normptr(p: usize) -> u64 {
    if p == 0 {
        return 0;
    }
    unsafe {
        for (i, &(b, n)) in OREGS.iter().enumerate() {
            if p >= b && p < b + n {
                return ((i as u64 + 1) << 32) | (p - b) as u64;
            }
        }
        for (i, &f) in OFNS.iter().enumerate() {
            if f == p {
                return (0x1000 + i as u64) << 32;
            }
        }
    }
    (0xFFFFu64 << 32) | 1
}
pub static mut OSCR: [[u8; 64]; 4] = [[0; 64]; 4];
pub fn gen_p() -> *mut c_void {
    let k = rnd() % 5;
    if k == 0 {
        return core::ptr::null_mut();
    }
    unsafe { OSCR[(k - 1) as usize].as_mut_ptr() as *mut c_void }
}

// ------------------------------------------------------------- the call log
pub static mut LG: String = String::new();
pub fn lg_s(s: &str) {
    unsafe { LG.push_str(s) }
}
pub fn lg_c(c: char) {
    unsafe { LG.push(c) }
}
pub fn lg_u(v: u64) {
    unsafe { LG.push_str(&v.to_string()) }
}
pub fn lg_hx(v: u64, digits: usize) {
    unsafe { LG.push_str(&format!("{:0w$x}", v, w = digits)) }
}
pub fn lg_hmin(v: u64) {
    unsafe { LG.push_str(&format!("{:x}", v)) }
}
pub fn lg_int(v: u32) {
    lg_u(v as u64);
    lg_c(',');
}
pub fn lg_f(f: f32) {
    lg_c('f');
    lg_hx(bc(f) as u64, 8);
    lg_c(',');
}
pub fn lg_d(f: f64) {
    lg_c('d');
    lg_hx(dc(f), 16);
    lg_c(',');
}
pub fn lg_ptr(p: usize) {
    lg_c('@');
    lg_hmin(normptr(p));
    lg_c(',');
}
pub fn lg_v3(p: *const Vec3) {
    lg_c('@');
    lg_hmin(normptr(p as usize));
    if !p.is_null() {
        let v = unsafe { &*p };
        lg_c('[');
        lg_hx(bc(v.x) as u64, 8);
        lg_c(' ');
        lg_hx(bc(v.y) as u64, 8);
        lg_c(' ');
        lg_hx(bc(v.z) as u64, 8);
        lg_c(']');
    }
    lg_c(',');
}
pub fn lg_v2(p: *const Vec2) {
    lg_c('@');
    lg_hmin(normptr(p as usize));
    if !p.is_null() {
        let v = unsafe { &*p };
        lg_c('[');
        lg_hx(bc(v.x) as u64, 8);
        lg_c(' ');
        lg_hx(bc(v.y) as u64, 8);
        lg_c(']');
    }
    lg_c(',');
}
pub fn lg_pf(p: *const f32) {
    lg_c('@');
    lg_hmin(normptr(p as usize));
    if !p.is_null() {
        lg_c('[');
        lg_hx(bc(unsafe { *p }) as u64, 8);
        lg_c(']');
    }
    lg_c(',');
}
pub fn lg_pi(p: *const i32) {
    lg_c('@');
    lg_hmin(normptr(p as usize));
    if !p.is_null() {
        lg_c('[');
        lg_u(unsafe { *p } as u32 as u64);
        lg_c(']');
    }
    lg_c(',');
}
pub fn lg_ov3(p: *mut Vec3) {
    if !p.is_null() {
        let v = unsafe { &mut *p };
        v.x = rf();
        v.y = rf();
        v.z = rf();
        lg_c('<');
        lg_hx(bc(v.x) as u64, 8);
        lg_c(' ');
        lg_hx(bc(v.y) as u64, 8);
        lg_c(' ');
        lg_hx(bc(v.z) as u64, 8);
        lg_c('>');
    }
}
pub fn lg_ov2(p: *mut Vec2) {
    if !p.is_null() {
        let v = unsafe { &mut *p };
        v.x = rf();
        v.y = rf();
        lg_c('<');
        lg_hx(bc(v.x) as u64, 8);
        lg_c(' ');
        lg_hx(bc(v.y) as u64, 8);
        lg_c('>');
    }
}
pub fn lg_opf(p: *mut f32) {
    if !p.is_null() {
        unsafe { *p = rf() };
        lg_c('<');
        lg_hx(bc(unsafe { *p }) as u64, 8);
        lg_c('>');
    }
}
pub fn lg_opi(p: *mut i32) {
    if !p.is_null() {
        unsafe { *p = gen_i() };
        lg_c('<');
        lg_u(unsafe { *p } as u32 as u64);
        lg_c('>');
    }
}
pub fn lg_retu(v: u32) {
    lg_c('=');
    lg_u(v as u64);
}
pub fn lg_retf(f: f32) {
    lg_c('=');
    lg_c('f');
    lg_hx(bc(f) as u64, 8);
}
pub fn lg_retd(f: f64) {
    lg_c('=');
    lg_c('d');
    lg_hx(dc(f), 16);
}
pub fn lg_retp(p: usize) {
    lg_c('=');
    lg_c('@');
    lg_hmin(normptr(p));
}

// ------------------------------------------------------------- struct fill by field kind
pub type ODesc = (u32, u32, u8);
fn put32(p: &mut [u8], v: u32) {
    p[..4].copy_from_slice(&v.to_le_bytes());
}
pub unsafe fn ofill(base: *mut u8, d: &[ODesc]) {
    for &(off, size, kind) in d {
        let p = core::slice::from_raw_parts_mut(base.add(off as usize), 8);
        match kind {
            b'f' => put32(p, rf().to_bits()),
            b'd' => {
                let a = rf();
                let b = (a as f64).to_bits();
                put32(p, b as u32);
                put32(&mut p[4..], (b >> 32) as u32);
            }
            b'i' => put32(p, iv(true)),
            b'u' => put32(p, iv(false)),
            b's' => {
                let v = iv(true);
                p[0] = v as u8;
                p[1] = (v >> 8) as u8;
            }
            b'S' => {
                let v = iv(false);
                p[0] = v as u8;
                p[1] = (v >> 8) as u8;
            }
            b'c' => p[0] = iv(true) as u8,
            b'C' => p[0] = iv(false) as u8,
            b'q' => {
                let v = iv(false);
                let w = iv(false);
                put32(p, v);
                put32(&mut p[4..], w & 0xF);
            }
            b'p' => p[..8].fill(0),
            b'b' => {
                for j in 0..size as usize {
                    *base.add(off as usize + j) = rnd() as u8;
                }
            }
            _ => {}
        }
    }
}
pub unsafe fn onorm(img: &mut [u8], d: &[ODesc]) {
    for &(off, _, kind) in d {
        let o = off as usize;
        if kind == b'p' {
            let v = usize::from_le_bytes(img[o..o + 8].try_into().unwrap());
            img[o..o + 8].copy_from_slice(&normptr(v).to_le_bytes());
        } else if kind == b'f' {
            let b = u32::from_le_bytes(img[o..o + 4].try_into().unwrap());
            img[o..o + 4].copy_from_slice(&cf32(b).to_le_bytes());
        } else if kind == b'd' {
            let b = u64::from_le_bytes(img[o..o + 8].try_into().unwrap());
            img[o..o + 8].copy_from_slice(&cf64(b).to_le_bytes());
        }
    }
}

/// A pointer to a small argument struct: id + FNV of its bytes (oracle.h `lg_ps`).
pub fn lg_ps(p: *const u8, n: usize, d: &[ODesc]) {
    lg_c('@');
    lg_hmin(normptr(p as usize));
    if !p.is_null() {
        let mut tmp = unsafe { core::slice::from_raw_parts(p, n) }.to_vec();
        unsafe { onorm(&mut tmp, d) };
        lg_c('{');
        lg_hx(fnv(FNV0, &tmp), 16);
        lg_c('}');
    }
    lg_c(',');
}

// ------------------------------------------------------------- efSync_Spawn shapes
pub fn efsync_shape(gfx: i32) -> &'static str {
    match gfx {
        0x406 => "Vf",
        0x4CF => "Vf",
        0x3FF => "Vff",
        1285 => "V",
        _ => "pppp",
    }
}
pub fn oracle_efsync(gfx: i32, gobj: *mut melee_types::HSD_GObj, args: &[*mut c_void]) -> *mut c_void {
    let sh = efsync_shape(gfx).as_bytes();
    lg_s("efSync_Spawn(");
    lg_int(gfx as u32);
    lg_ptr(gobj as usize);
    for (i, &a) in args.iter().enumerate() {
        match sh[i] {
            b'V' => lg_v3(a as *const Vec3),
            b'f' => lg_pf(a as *const f32),
            _ => lg_ptr(a as usize),
        }
    }
    lg_s(")");
    let r = gen_p();
    lg_retp(r as usize);
    lg_c(';');
    r
}

/// Fn-pointer argument as an address (0 for None).
pub fn fnaddr<T: Copy>(f: Option<T>) -> usize {
    match f {
        None => 0,
        Some(f) => unsafe { core::mem::transmute_copy::<T, usize>(&f) },
    }
}
