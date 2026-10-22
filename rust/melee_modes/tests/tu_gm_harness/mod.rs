//! Rust half of gen/cref/tu/tu_common.h: same xorshift, same fill order, same trace hash.
#![allow(dead_code, static_mut_refs)]

pub static mut TR: u32 = 0;
pub fn t(v: u32) {
    unsafe { TR = TR.wrapping_mul(31).wrapping_add(v) }
}
pub fn b(f: f32) -> u32 {
    f.to_bits()
}
pub fn hbuf(p: *const u8, n: usize) -> u32 {
    let mut h = 0x811C_9DC5u32;
    for i in 0..n {
        h ^= unsafe { *p.add(i) } as u32;
        h = h.wrapping_mul(0x0100_0193);
    }
    h
}
/// Hash of the bytes of a value, folded into the trace.
pub fn tb<T>(v: &T) {
    t(hbuf(v as *const T as *const u8, core::mem::size_of::<T>()))
}
pub fn hv<T>(v: &T) -> u32 {
    hbuf(v as *const T as *const u8, core::mem::size_of::<T>())
}
pub static mut SRV: [u32; 16] = [0; 16];
pub fn sr(i: usize) -> bool {
    unsafe { (SRV[i & 15] >> (i >> 4)) & 1 != 0 }
}
pub static mut RS: u32 = 0;
pub fn rnd() -> u32 {
    unsafe {
        RS ^= RS << 13;
        RS ^= RS >> 17;
        RS ^= RS << 5;
        RS
    }
}
pub fn rg() -> f32 {
    match rnd() % 16 {
        0 => 0.0,
        1 => -0.0,
        2 => f32::from_bits(0x7FC0_0000),
        3 => 1.0,
        4 => -1.0,
        _ => ((rnd() % 41) as i32 - 20) as f32 / 20.0,
    }
}
/// Every byte random, like `fillb` in tu_common.h.
pub fn fillb<T>(v: &mut T) {
    let p = v as *mut T as *mut u8;
    for i in 0..core::mem::size_of::<T>() {
        unsafe { *p.add(i) = (rnd() >> 11) as u8 };
    }
}
pub fn zero<T>(v: &mut T) {
    unsafe { core::ptr::write_bytes(v as *mut T as *mut u8, 0, 1 * core::mem::size_of::<T>()) }
}
pub fn case(op: u32, k: u32) {
    unsafe {
        TR = 0;
        RS = 0x9059u32.wrapping_add(op.wrapping_mul(7919)).wrapping_add(k.wrapping_mul(104729));
        for i in 0..16 {
            SRV[i] = rnd();
        }
    }
}
/// Compares the per-case lines with tests/data/tu_<stem>.txt.
pub struct Check {
    lines: std::str::Lines<'static>,
    pub n: u32,
}
impl Check {
    pub fn new(s: &'static str) -> Self {
        Check { lines: s.lines(), n: 0 }
    }
    pub fn line(&mut self, op: u32, k: u32, ret: u32, hash: u32) {
        let got = format!("{} {} {:08X} {:08X} {:08X}", op, k, ret, hash, unsafe { TR });
        let want = self.lines.next().expect("expected file too short");
        assert_eq!(got, want, "op {op} case {k}");
        self.n += 1;
    }
    /// A line that starts with a tag instead of an op number ("T 0 ...").
    pub fn line_tag(&mut self, tag: &str, ret: u32, hash: u32) {
        let got = format!("{} 0 {:08X} {:08X} {:08X}", tag, ret, hash, unsafe { TR });
        let want = self.lines.next().expect("expected file too short");
        assert_eq!(got, want, "line {tag}");
        self.n += 1;
    }
    pub fn done(mut self) -> u32 {
        assert!(self.lines.next().is_none(), "expected file has extra lines");
        self.n
    }
}
