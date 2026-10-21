//! Native group-3 types, generated from the GameCube layout (gen/gen_types.py).
//!
//! Bitfield storage is `[u8; N]` holding the GameCube bytes in GC order, read MSB-first like the
//! PPC ABI allocates them; the accessors below hide that.

/// Read a bitfield of width `w` starting `bit` bits (MSB-first) into the storage bytes.
macro_rules! bf_get {
    ($st:expr, $bit:expr, $w:expr, bool) => { $crate::types::bf_read(&$st, $bit, $w) != 0 };
    ($st:expr, $bit:expr, $w:expr, $t:ident) => {{
        let raw = $crate::types::bf_read(&$st, $bit, $w);
        <$t as $crate::types::BfVal>::from_raw(raw, $w)
    }};
}
macro_rules! bf_set {
    ($st:expr, $bit:expr, $w:expr, $v:expr) => {
        $crate::types::bf_write(&mut $st, $bit, $w, $crate::types::BfVal::to_raw($v))
    };
}

/// Raw bits of a bitfield (unsigned, right-aligned).
#[inline]
pub fn bf_read(st: &[u8], bit: u32, w: u32) -> u64 {
    let mut v = 0u64;
    for i in 0..w {
        let b = bit + i;
        v = (v << 1) | ((st[(b / 8) as usize] >> (7 - b % 8)) & 1) as u64;
    }
    v
}
#[inline]
pub fn bf_write(st: &mut [u8], bit: u32, w: u32, v: u64) {
    for i in 0..w {
        let b = bit + i;
        let one = (v >> (w - 1 - i)) & 1;
        let m = 1u8 << (7 - b % 8);
        let p = &mut st[(b / 8) as usize];
        *p = if one != 0 { *p | m } else { *p & !m };
    }
}

/// Conversion between a field's Rust type and its raw bits (sign extension for signed C types).
pub trait BfVal: Copy {
    fn to_raw(self) -> u64;
    fn from_raw(raw: u64, w: u32) -> Self;
}
macro_rules! bfval_u { ($($t:ty),*) => {$(impl BfVal for $t {
    #[inline] fn to_raw(self) -> u64 { self as u64 }
    #[inline] fn from_raw(raw: u64, _w: u32) -> Self { raw as $t }
})*}; }
macro_rules! bfval_i { ($($t:ty),*) => {$(impl BfVal for $t {
    #[inline] fn to_raw(self) -> u64 { self as i64 as u64 }
    #[inline] fn from_raw(raw: u64, w: u32) -> Self { (((raw << (64 - w)) as i64) >> (64 - w)) as $t }
})*}; }
bfval_u!(u8, u16, u32, u64);
bfval_i!(i8, i16, i32, i64);
impl BfVal for bool {
    #[inline] fn to_raw(self) -> u64 { self as u64 }
    #[inline] fn from_raw(raw: u64, _w: u32) -> Self { raw != 0 }
}

mod generated;
pub use generated::*;
