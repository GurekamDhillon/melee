//! Group 6 round 8 (night 3 round 4): lb leaves over nested pointers against the C, bit for bit.
//! Same harness as oracle_r4.rs. tests/data/vectors_r8.txt is written by oracle/build_r8.sh
//! (gen_oracle_r8.py over gen_oracle_r4.py): each C
//! function cut verbatim from its TU together with every game callee it reaches (so the C
//! side runs the game's own callees), compiled with contraction on + gwtool's lowering.
//! New since round 3: `l:<from>:<off>:<to>` stores buffer <to>'s address (ffff = NULL) at
//! byte <off> of buffer <from>; buffers past the argument ones are reachable only by links;
//! and before hashing, every 8-byte word that points into a buffer k is rewritten to
//! 0x5EED000000000000 | k<<24 | offset on both sides, so stored pointers compare by target.
//!
//! A line is `name tok... = ret hash`. `s:<hex>` is a scalar (or a pointer the function only
//! stores/compares); `b:<seed>:<size>:<sizeof>` a buffer filled by `fill` (seed 0 = NULL);
//! `f:<buf>:<off>:<size>:<hex>` a little-endian store into a buffer after filling. `hash` is
//! FNV-1a 64 over every non-NULL buffer after the call. The C views a buffer as its own struct
//! type, the Rust as the melee_types record: their native layouts are identical (gen_types.py
//! checks every offset against clang), so equal hashes mean equal field values.
#![allow(non_snake_case)]

use melee_modes::lb_r8::*;
use melee_types as mt;
use std::collections::BTreeMap;

fn sm(x: &mut u64) -> u64 {
    *x = x.wrapping_add(0x9E3779B97F4A7C15);
    let mut z = *x;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
    z ^ (z >> 31)
}

/// One 4-byte word of buffer fill; the same generator as main.c's `word`. Floats never
/// come out NaN or infinite (bit 30 is cleared in the raw-bits case).
fn word(st: &mut u64) -> u32 {
    let r = sm(st);
    let v = (r >> 32) as u32;
    match r & 7 {
        0 | 1 => 0,
        2 => v % 3 + 1,
        3 => v & 0xFF,
        4 => (((v % 1000001) as i32 - 500000) as f32 / 1000.0).to_bits(),
        5 => (((v % 4001) as i32 - 2000) as f32 / 1000.0).to_bits(),
        _ => v & 0xBFFF_FFFF,
    }
}

struct Buf {
    w: Vec<u64>,
    sz: usize,
    null: bool,
    /// sizeof(*arg) in the C, checked against the Rust type each view uses
    csize: usize,
}

impl Buf {
    fn new(seed: u64, sz: usize, csize: usize) -> Buf {
        let mut w = vec![0u64; sz / 8 + 2];
        if seed != 0 {
            let mut st = seed;
            let bytes = unsafe { std::slice::from_raw_parts_mut(w.as_mut_ptr() as *mut u8, sz) };
            for i in 0..sz / 4 {
                bytes[4 * i..4 * i + 4].copy_from_slice(&word(&mut st).to_le_bytes());
            }
        }
        Buf { w, sz, null: seed == 0, csize }
    }
    fn bytes(&mut self) -> &mut [u8] {
        unsafe { std::slice::from_raw_parts_mut(self.w.as_mut_ptr() as *mut u8, self.sz) }
    }
}

/// A buffer as the Rust function sees it: pointer, byte size, C sizeof of the pointee.
#[derive(Clone, Copy)]
struct P(*mut u8, usize, usize);

#[allow(dead_code)]
impl P {
    fn check<T>(self) {
        // the Rust record must have the C struct's size (layouts are checked by gen_types.py)
        assert_eq!(std::mem::size_of::<T>(), self.2, "sizeof {} vs C", std::any::type_name::<T>());
    }
    fn r<'a, T>(self) -> &'a mut T {
        self.check::<T>();
        assert!(!self.0.is_null() && std::mem::size_of::<T>() <= self.1);
        unsafe { &mut *(self.0 as *mut T) }
    }
    fn o<'a, T>(self) -> Option<&'a mut T> {
        self.check::<T>();
        if self.0.is_null() { None } else { Some(self.r()) }
    }
    fn os<'a, T>(self) -> Option<&'a mut [T]> {
        if self.0.is_null() { None } else { Some(self.s()) }
    }
    fn s<'a, T>(self) -> &'a mut [T] {
        self.check::<T>();
        assert!(!self.0.is_null());
        unsafe { std::slice::from_raw_parts_mut(self.0 as *mut T, self.1 / std::mem::size_of::<T>()) }
    }
}

#[allow(dead_code)]
fn f(b: u64) -> f32 {
    f32::from_bits(b as u32)
}
#[allow(dead_code)]
fn fb(x: f32) -> u64 {
    x.to_bits() as u64
}
fn i(x: i32) -> u64 {
    x as u32 as u64
}

fn eval(name: &str, s: &[u64], p: &[P]) -> u64 {
    match name {
        "lbColl_CopyHitCapsule" => { lbColl_CopyHitCapsule(&*p[0].r::<mt::HitCapsule>(), p[1].r()); 0 }
        "lbColl_8000ACFC" => lbColl_8000ACFC(s[0] as *mut std::ffi::c_void, p[0].r()) as u64,
        "lb_8000B09C" => (unsafe { lb_8000B09C(p[0].o::<mt::HSD_JObj>().map_or(std::ptr::null(), |j| j as *const _)) }) as u64,
        "lb_8000B134" => (unsafe { lb_8000B134(p[0].o::<mt::HSD_JObj>().map_or(std::ptr::null(), |j| j as *const _)) }) as u64,
        "lb_80011B74" => { unsafe { lb_80011B74(p[0].r(), s[0] as u32) }; 0 }
        _ => panic!("no port for {name}"),
    }
}

#[test]
fn group6_round8_match_c() {
    let text = include_str!("data/vectors_r8.txt");
    let mut per: BTreeMap<&str, (usize, usize)> = BTreeMap::new();
    let mut first_bad = Vec::new();
    for line in text.lines().filter(|l| !l.is_empty()) {
        let (lhs, rhs) = line.split_once(" = ").expect(line);
        let (want_r, want_h) = rhs.split_once(' ').expect(line);
        let (want_r, want_h) = (u64::from_str_radix(want_r, 16).unwrap(), u64::from_str_radix(want_h, 16).unwrap());
        let mut it = lhs.split(' ');
        let name = it.next().unwrap();
        let (mut s, mut bufs, mut links) = (Vec::new(), Vec::new(), Vec::new());
        for t in it {
            let v: Vec<u64> = t[2..].split(':').map(|x| u64::from_str_radix(x, 16).unwrap()).collect();
            match &t[..2] {
                "s:" => s.push(v[0]),
                "b:" => bufs.push(Buf::new(v[0], v[1] as usize, v[2] as usize)),
                "f:" => {
                    let bytes = bufs[v[0] as usize].bytes();
                    for k in 0..v[2] as usize {
                        bytes[v[1] as usize + k] = (v[3] >> (8 * k)) as u8;
                    }
                }
                "l:" => links.push((v[0] as usize, v[1] as usize, v[2] as usize)),
                _ => panic!("{t}"),
            }
        }
        let ps: Vec<P> = bufs
            .iter_mut()
            .map(|b| P(if b.null { std::ptr::null_mut() } else { b.w.as_mut_ptr() as *mut u8 }, b.sz, b.csize))
            .collect();
        for &(from, off, to) in &links {
            let a = if to == 0xFFFF { 0u64 } else { ps[to].0 as u64 };
            bufs[from].bytes()[off..off + 8].copy_from_slice(&a.to_le_bytes());
        }
        let got_r = eval(name, &s, &ps);
        let mut h: u64 = 0xCBF29CE484222325;
        let ranges: Vec<(u64, u64)> = ps.iter().map(|q| (q.0 as u64, q.1 as u64)).collect();
        for b in bufs.iter_mut().filter(|b| !b.null) {
            let bytes = b.bytes();
            for o in (0..bytes.len() / 8 * 8).step_by(8) {
                let w = u64::from_le_bytes(bytes[o..o + 8].try_into().unwrap());
                if let Some(q) = ranges.iter().position(|&(base, sz)| base != 0 && w >= base && w < base + sz) {
                    let n = 0x5EED_0000_0000_0000u64 | ((q as u64) << 24) | (w - ranges[q].0);
                    bytes[o..o + 8].copy_from_slice(&n.to_le_bytes());
                }
            }
            for &x in bytes.iter() {
                h ^= x as u64;
                h = h.wrapping_mul(0x100000001B3);
            }
        }
        let e = per.entry(name).or_default();
        e.0 += 1;
        if got_r != want_r || h != want_h {
            e.1 += 1;
            if first_bad.len() < 20 {
                first_bad.push(format!("{line}  (rust {got_r:x} {h:x})"));
            }
        }
    }
    let bad: Vec<_> = per.iter().filter(|(_, v)| v.1 > 0).collect();
    assert!(bad.is_empty(), "mismatches {bad:?}\n{}", first_bad.join("\n"));
    assert_eq!(per.len(), 5, "functions covered (round 8)");
}
