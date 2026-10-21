//! Rust half of gen/cref/tu/tu_common.h: same xorshift, same fill order, same trace hash.
#![allow(dead_code, static_mut_refs)]
use melee_fighter::types::*;

pub static mut TR: u32 = 0;
pub fn t(v: u32) { unsafe { TR = TR.wrapping_mul(31).wrapping_add(v) } }
pub fn b(f: f32) -> u32 { f.to_bits() }
pub static mut SRV: [u32; 16] = [0; 16];
pub fn sr(i: usize) -> bool { unsafe { (SRV[i & 15] >> (i >> 4)) & 1 != 0 } }
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
pub fn rw() -> f32 {
    const K: [f32; 4] = [1.0, 13.0, 1000.0, 100000.0];
    let a = rg();
    a * K[(rnd() % 4) as usize]
}
pub struct Hs(pub u32);
impl Hs {
    pub fn new() -> Self { Hs(0x811C_9DC5) }
    pub fn m(&mut self, v: u32) { self.0 = self.0.wrapping_mul(0x0100_0193) ^ v; }
}

pub static mut GO: *mut HSD_GObj = core::ptr::null_mut();
pub static mut GO2: *mut HSD_GObj = core::ptr::null_mut();
pub static mut IGO: *mut HSD_GObj = core::ptr::null_mut();
pub static mut F: *mut Fighter = core::ptr::null_mut();
pub static mut F2: *mut Fighter = core::ptr::null_mut();
pub static mut CD: *mut ftCommonData = core::ptr::null_mut();
pub fn go() -> *mut HSD_GObj { unsafe { GO } }
pub fn go2() -> *mut HSD_GObj { unsafe { GO2 } }
pub fn igo() -> *mut HSD_GObj { unsafe { IGO } }
pub fn f() -> &'static mut Fighter { unsafe { &mut *F } }
pub fn f2() -> &'static mut Fighter { unsafe { &mut *F2 } }
pub fn cd() -> &'static mut ftCommonData { unsafe { &mut *CD } }
pub fn is_go(g: *mut HSD_GObj) -> u32 { (g == go()) as u32 }

pub fn init() {
    unsafe {
        F = Box::leak(Box::new(Fighter::zeroed()));
        F2 = Box::leak(Box::new(Fighter::zeroed()));
        GO = Box::leak(Box::new(HSD_GObj::zeroed()));
        GO2 = Box::leak(Box::new(HSD_GObj::zeroed()));
        IGO = Box::leak(Box::new(HSD_GObj::zeroed()));
        CD = Box::leak(Box::new(ftCommonData::zeroed()));
        (*GO).user_data = F as *mut core::ffi::c_void;
        (*GO2).user_data = F2 as *mut core::ffi::c_void;
        melee_fighter::hooks::P_FT_COMMON_DATA = CD;
    }
}
pub fn case(op: u32, k: u32) {
    unsafe {
        *F = Fighter::zeroed();
        *F2 = Fighter::zeroed();
        *CD = ftCommonData::zeroed();
        TR = 0;
        RS = 0x9059u32.wrapping_add(op.wrapping_mul(7919)).wrapping_add(k.wrapping_mul(104729));
        for i in 0..16 { SRV[i] = rnd(); }
    }
}
/// Compares the per-case lines with tests/data/tu_<stem>.txt.
pub struct Check { lines: std::str::Lines<'static>, pub n: u32 }
impl Check {
    pub fn new(s: &'static str) -> Self { Check { lines: s.lines(), n: 0 } }
    pub fn line(&mut self, op: u32, k: u32, ret: u32, hash: u32) {
        let got = format!("{} {} {:08X} {:08X} {:08X}", op, k, ret, hash, unsafe { TR });
        let want = self.lines.next().expect("expected file too short");
        assert_eq!(got, want, "op {op} case {k}");
        self.n += 1;
    }
    pub fn done(mut self) -> u32 { assert!(self.lines.next().is_none(), "expected file has extra lines"); self.n }
}
