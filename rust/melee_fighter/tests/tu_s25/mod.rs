//! Rust mirror of gen/cref/tu/tu_common.h (same xorshift, same fill order) for the shard-25
//! TU tests (ftcamera, ftchangeparam, ftcliffcommon, ftcmdscript, ftcolanim, ftcoll, ftcommon).
#![allow(dead_code, static_mut_refs)]
use melee_fighter::types::*;

pub struct H {
    pub rs: u32,
    pub tr: u32,
    pub h: u32,
    pub srv: [u32; 16],
}

impl H {
    pub fn case(op: u32, k: u32) -> H {
        let mut s = H { rs: 0x9059u32.wrapping_add(op.wrapping_mul(7919)).wrapping_add(k.wrapping_mul(104729)), tr: 0, h: 0, srv: [0; 16] };
        for i in 0..16 {
            s.srv[i] = s.rnd();
        }
        s
    }
    pub fn rnd(&mut self) -> u32 {
        self.rs ^= self.rs << 13;
        self.rs ^= self.rs >> 17;
        self.rs ^= self.rs << 5;
        self.rs
    }
    pub fn rg(&mut self) -> f32 {
        match self.rnd() % 16 {
            0 => 0.0,
            1 => -0.0,
            2 => f32::from_bits(0x7FC00000),
            3 => 1.0,
            4 => -1.0,
            _ => ((self.rnd() % 41) as i32 - 20) as f32 / 20.0,
        }
    }
    pub fn rw(&mut self) -> f32 {
        const K: [f32; 4] = [1.0, 13.0, 1000.0, 100000.0];
        let a = self.rg();
        a * K[(self.rnd() % 4) as usize]
    }
    pub fn sr(&self, i: usize) -> u32 { self.srv[i] & 1 }
    pub fn t(&mut self, v: u32) { self.tr = self.tr.wrapping_mul(31).wrapping_add(v); }
    pub fn hm(&mut self, v: u32) { self.h = self.h.wrapping_mul(0x01000193) ^ v; }
    pub fn hf(&mut self, f: f32) { self.hm(f.to_bits()); }
    pub fn hv(&mut self, v: &Vec3) { self.hf(v.x); self.hf(v.y); self.hf(v.z); }
}

pub fn zeroed<T>() -> Box<T> {
    unsafe { Box::new(core::mem::zeroed()) }
}

/// Lines of tests/data/<name>.txt.
pub fn expected(name: &str) -> std::vec::Vec<String> {
    let p = format!("{}/tests/data/{}.txt", env!("CARGO_MANIFEST_DIR"), name);
    std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("{p}: {e}")).lines().map(String::from).collect()
}

/// Thread-local trace sink for hooks (fn pointers cannot capture).
pub static mut TR: u32 = 0;
pub fn t(v: u32) { unsafe { TR = TR.wrapping_mul(31).wrapping_add(v) } }
