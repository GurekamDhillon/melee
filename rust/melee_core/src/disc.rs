//! Discrimination bits (feature `disc`, the covtest build only; notes/coverage-testing.md). Each call site of a
//! Gekko float helper (fp.rs) or PowerPC integer helper (ppc.rs) gets a cell, keyed by its `#[track_caller]`
//! location: the operand classes seen there, and one bit per alternative semantics whose result differed from the
//! helper's at that call (fused vs unfused, flooring vs truncating division, wrapping vs saturating casts, masked vs
//! PowerPC shifts, the sign of an exact zero). Each bit keeps the first frame (`set_frame`) it was set on, so a run
//! checked against Dolphin up to frame F verifies the bits first set before F. Not compiled in any other build.
#![allow(static_mut_refs)]

use std::panic::Location;

/// Operation kinds (the cell's `kind`).
pub const K_FMA: u8 = 0; // fmadds / fmsubs / fmsubs_neg
pub const K_FNM: u8 = 1; // fnmsubs / fnmadds
pub const K_DIV: u8 = 2;
pub const K_REM: u8 = 3;
pub const K_SHL: u8 = 4;
pub const K_SHR: u8 = 5;
pub const K_CVT: u8 = 6; // fctiwz
pub const KIND_NAMES: [&str; 7] = ["fma", "fnm", "div", "rem", "shl", "shr", "cvt"];

/// Alternative bits.
pub const A_FUSED: u8 = 0; // one-rounding f32 fma (`mul_add`)
pub const A_UNFUSED: u8 = 1; // f32 multiply, then f32 add (two f32 roundings)
pub const A_F64: u8 = 2; // the f64 intermediate kept (result not rounded to f32 before the next use: compared in f64)
pub const A_ZSIGN: u8 = 3; // fnm: the +a*c form's sign of an exact zero (fmadds(-a, c, b))
pub const A_FLOOR: u8 = 4; // div/rem: flooring instead of truncating
pub const A_WRAP: u8 = 5; // shifts: the amount masked to the width (x86/Rust wrapping); cvt: a wrapping (i64 -> i32) cast
pub const ALT_NAMES: [&str; 6] = ["fused", "unfused", "f64-intermediate", "zero-sign", "flooring", "wrapping"];

#[derive(Clone, Copy)]
pub struct Cell {
    pub loc: usize, // &'static Location as an address; 0 = empty
    pub kind: u8,
    /// operand classes: float operands 9 bits each (fclass), int operands 5 bits each (iclass)
    pub classes: u64,
    pub alts: u8,
    pub first: [i32; 6],
    pub calls: u64,
}

const EMPTY: Cell = Cell { loc: 0, kind: 0, classes: 0, alts: 0, first: [i32::MAX; 6], calls: 0 };
const N: usize = 1 << 15;
static mut TABLE: [Cell; N] = [EMPTY; N];
static mut FRAME: i32 = i32::MIN;
static mut USED: usize = 0;

/// The frame later bits are stamped with (slpcheck: the NHV1 CSV frame).
pub fn set_frame(f: i32) {
    unsafe { FRAME = f };
}

#[inline]
fn cell(loc: &'static Location<'static>, kind: u8) -> &'static mut Cell {
    let a = loc as *const _ as usize;
    let mut h = (a >> 3).wrapping_mul(0x9E37_79B9_7F4A_7C15) >> 49;
    unsafe {
        loop {
            let c = &mut TABLE[h & (N - 1)];
            if c.loc == a {
                return c;
            }
            if c.loc == 0 {
                if USED >= N - 1 {
                    return &mut TABLE[N - 1];
                }
                USED += 1;
                c.loc = a;
                c.kind = kind;
                return c;
            }
            h += 1;
        }
    }
}

/// 0 +0, 1 -0, 2 +denormal, 3 -denormal, 4 +normal, 5 -normal, 6 +inf, 7 -inf, 8 NaN
#[inline]
pub fn fclass(x: f64, single: bool) -> u32 {
    let neg = x.is_sign_negative() as u32;
    if x.is_nan() {
        8
    } else if x == 0.0 {
        neg
    } else if x.is_infinite() {
        6 + neg
    } else if (single && (x.abs() as f32) < f32::MIN_POSITIVE) || (!single && x.abs() < f64::MIN_POSITIVE) {
        2 + neg
    } else {
        4 + neg
    }
}

/// bits: 0 negative, 1 zero, 2 MIN, 3 shift amount >= the width, 4 divisor 0
#[inline]
fn iclass(v: i64, min: i64) -> u64 {
    ((v < 0) as u64) | (((v == 0) as u64) << 1) | (((v == min) as u64) << 2)
}

#[inline]
fn set_alt(c: &mut Cell, bit: u8) {
    c.alts |= 1 << bit;
    let f = unsafe { FRAME };
    if f < c.first[bit as usize] {
        c.first[bit as usize] = f;
    }
}

#[inline]
fn bits_eq32(a: f32, b: f32) -> bool {
    a.to_bits() == b.to_bits() || (a.is_nan() && b.is_nan())
}

/// fmadds-family: `r` the helper's result, `neg_b` the subtracted forms (b enters as -b).
#[track_caller]
#[inline(never)]
pub fn fma(a: f32, c_: f32, b: f32, neg_b: bool, r: f32) {
    let c = cell(Location::caller(), K_FMA);
    c.calls += 1;
    c.classes |= (1u64 << fclass(a as f64, true)) | (1u64 << (9 + fclass(c_ as f64, true))) | (1u64 << (18 + fclass(b as f64, true)));
    let bb = if neg_b { -b } else { b };
    if !bits_eq32(a.mul_add(c_, bb), r) {
        set_alt(c, A_FUSED);
    }
    if !bits_eq32(a * c_ + bb, r) {
        set_alt(c, A_UNFUSED);
    }
    let d = a as f64 * c_ as f64 + bb as f64;
    if d != r as f64 && !(d.is_nan() && r.is_nan()) {
        set_alt(c, A_F64);
    }
}

/// fnmsubs / fnmadds: `r` = -(a*c -/+ b); the alternative writes it as fmadds(-a, c, ±b).
#[track_caller]
#[inline(never)]
pub fn fnm(a: f32, c_: f32, b: f32, sub: bool, r: f32) {
    let c = cell(Location::caller(), K_FNM);
    c.calls += 1;
    c.classes |= (1u64 << fclass(a as f64, true)) | (1u64 << (9 + fclass(c_ as f64, true))) | (1u64 << (18 + fclass(b as f64, true)));
    let bb = if sub { b } else { -b };
    let alt = ((-a) as f64 * c_ as f64 + bb as f64) as f32;
    if !bits_eq32(alt, r) {
        set_alt(c, A_ZSIGN);
    }
    if !bits_eq32((-a).mul_add(c_, bb), r) {
        set_alt(c, A_FUSED);
    }
    if !bits_eq32((-a) * c_ + bb, r) {
        set_alt(c, A_UNFUSED);
    }
}

/// div/rem: `a`, `b` the operands (as i64), `bits` the width, `r` the helper's result.
#[track_caller]
#[inline(never)]
pub fn divrem(rem: bool, a: i64, b: i64, min: i64, r: i64) {
    let c = cell(Location::caller(), if rem { K_REM } else { K_DIV });
    c.calls += 1;
    c.classes |= iclass(a, min) | (iclass(b, min) << 5) | (((b == 0) as u64) << 4);
    if b != 0 && !(a == min && b == -1) {
        // flooring division/modulo (Python's): differs when the signs differ and the remainder is not 0
        let (q, m) = (a / b, a % b);
        let adj = m != 0 && ((m < 0) != (b < 0));
        let alt = if rem { if adj { m + b } else { m } } else if adj { q - 1 } else { q };
        if alt != r {
            set_alt(c, A_FLOOR);
        }
    }
}

/// shifts: `v` the value, `n` the amount, `width` the bits, `r` the helper's result, `wrapped` the masked-amount result.
#[track_caller]
#[inline(never)]
pub fn shift(left: bool, v: i64, n: i64, width: u32, r: i64, wrapped: i64, min: i64) {
    let c = cell(Location::caller(), if left { K_SHL } else { K_SHR });
    c.calls += 1;
    c.classes |= iclass(v, min) | ((((n as u64) >= width as u64) as u64) << 3) | (iclass(n, i64::MIN) << 5);
    if wrapped != r {
        set_alt(c, A_WRAP);
    }
}

/// fctiwz: `x` the operand, `r` the saturating result.
#[track_caller]
#[inline(never)]
pub fn cvt(x: f64, r: i32) {
    let c = cell(Location::caller(), K_CVT);
    c.calls += 1;
    c.classes |= 1u64 << fclass(x, false);
    let wrapped = if x.is_nan() { 0 } else { (x as i64) as i32 };
    if wrapped != r {
        set_alt(c, A_WRAP);
    }
}

/// Write the table: `file:line:col kind calls classes alts first0..first5` (MELEE_DISC_OUT).
pub fn dump(path: &str) -> std::io::Result<()> {
    use std::io::Write;
    let mut f = std::io::BufWriter::new(std::fs::File::create(path)?);
    unsafe {
        for c in TABLE.iter().filter(|c| c.loc != 0) {
            let l = &*(c.loc as *const Location<'static>);
            write!(f, "{}:{}:{}\t{}\t{}\t{:x}\t{:x}", l.file(), l.line(), l.column(), KIND_NAMES[c.kind as usize], c.calls, c.classes, c.alts)?;
            for x in c.first {
                write!(f, "\t{}", if x == i32::MAX { "-".to_string() } else { x.to_string() })?;
            }
            writeln!(f)?;
        }
    }
    Ok(())
}
