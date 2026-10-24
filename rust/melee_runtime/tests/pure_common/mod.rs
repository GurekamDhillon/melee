//! Shared by the pure-function oracle tests: the same xorshift64 / FNV-1a64 as
//! `gen/pure/common.h`, and the comparison with `tests/data/pure_<stem>.txt`.
#![allow(dead_code)]
pub struct Rng(u64);
impl Rng {
    pub fn new() -> Self {
        Rng(0x9E3779B97F4A7C15)
    }
    pub fn u(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    /// `rndf()` of common.h (same draws in the same order).
    pub fn f(&mut self) -> f32 {
        let r = self.u();
        match (r % 16) as i32 {
            0 => 0.0,
            1 => -0.0,
            2 => 1.0,
            3 => -1.0,
            4 => ((self.u() % 2001) as i32 - 1000) as f32 * 0.001,
            5 => ((self.u() % 20001) as i32 - 10000) as f32,
            6 => ((self.u() % 2001) as i32 - 1000) as f32 * 1e-20,
            7 => ((self.u() % 2001) as i32 - 1000) as f32 * 1e12,
            _ => ((self.u() % 2000001) as i32 - 1000000) as f32 * 0.0001,
        }
    }
}
pub fn fnv(b: &[u8]) -> u64 {
    let mut h = 0xCBF29CE484222325u64;
    for &x in b {
        h ^= x as u64;
        h = h.wrapping_mul(0x100000001B3);
    }
    h
}
pub fn line(op: &str, k: i32, h: u64) -> String {
    format!("{op} {k} {h:016x}\n")
}
pub fn compare(stem: &str, got: &str) {
    let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/data").join(format!("pure_{stem}.txt"));
    let want = std::fs::read_to_string(&p).unwrap().replace("\r\n", "\n");
    let (g, w): (Vec<&str>, Vec<&str>) = (got.lines().collect(), want.lines().collect());
    assert_eq!(g.len(), w.len(), "line count");
    let bad: Vec<_> = g.iter().zip(&w).filter(|(a, b)| a != b).take(5).collect();
    assert!(bad.is_empty(), "{} differ, first: {:?}", g.iter().zip(&w).filter(|(a, b)| a != b).count(), bad);
}
