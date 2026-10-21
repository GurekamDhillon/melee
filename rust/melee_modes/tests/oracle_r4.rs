//! Group 6 round 4: callers of rounds 1-3 against the C, bit for bit.
//! tests/data/vectors_r4.txt is written by oracle/build_r4.sh (gen_oracle_r4.py): each C
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

use melee_modes::{gm_r4::*, lb_r4::*, mn_r4::*};
use melee_types as mt;
use mt::HSD_GObj;
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

fn f(b: u64) -> f32 {
    f32::from_bits(b as u32)
}
fn fb(x: f32) -> u64 {
    x.to_bits() as u64
}
fn i(x: i32) -> u64 {
    x as u32 as u64
}
fn b(x: bool) -> u64 {
    x as u64
}

fn eval(name: &str, s: &[u64], p: &[P]) -> u64 {
    let s32 = |k: usize| s[k] as u32 as i32;
    let u8_ = |k: usize| s[k] as u8;
    match name {
        // ---- gm, textlib
        "bitset_set" => { bitset_set(p[0].s(), s[0] as u32); 0 }
        "bitset_clear" => { bitset_clear(p[0].s(), s[0] as u32); 0 }
        "bitset_test" => bitset_test(p[0].s(), s[0] as u32) as u64,
        "bitset_test_word" => bitset_test_word(p[0].s(), s[0] as u32) as u64,
        "gm_SetupHumanPlayer" => { gm_SetupHumanPlayer(p[0].r(), u8_(0), u8_(1), u8_(2), u8_(3)); 0 }
        "gm_SetupCpuPlayer" => { gm_SetupCpuPlayer(p[0].r(), u8_(0), u8_(1), u8_(2), u8_(3)); 0 }
        "gm_MatchHasMultipleWinners" => b(gm_MatchHasMultipleWinners(p[0].r())),
        "gm_801B07E8_layer" => {
            gm_801B07E8_layer(p[0].r(), p[1].o(), p[2].o(), p[3].o(), p[4].o(), p[5].o());
            0
        }
        "gmVsMelee_UpdateKOCounts" => { gmVsMelee_UpdateKOCounts(p[0].s(), p[1].r()); 0 }
        "fn_801771C0" => {
            if p.len() > 1 { p[1].check::<mt::MatchEnd>(); }
            fn_801771C0(p[0].r());
            0
        }
        "DevText_SetCursorX" => { DevText_SetCursorX(p[0].r(), s32(0)); 0 }
        // ---- lb
        "lb_8000D148" => i(lb_8000D148(f(s[0]), f(s[1]), f(s[2]), f(s[3]), f(s[4]), f(s[5]), f(s[6]))),
        "groundHeight" => fb(groundHeight(p[0].r(), p[1].r())),
        "calc_acos" => fb(calc_acos(f(s[0]))),
        "lbShadow_8000E9F0" => {
            if p.len() > 2 { p[2].check::<mt::Vec>(); }
            lbShadow_8000E9F0(p[0].r(), p[1].r(), f(s[0]));
            0
        }
        "fn_800253D8" => b(fn_800253D8(ro(p))),
        "fn_800256BC" => b(fn_800256BC(ro(p))),
        "fn_80025B44" => b(fn_80025B44(ro(p))),
        "fn_80025CBC" => b(fn_80025CBC(ro(p))),
        "fn_800259EC" => b(fn_800259EC(ro(p))),
        "fn_80025A98" => b(fn_80025A98(ro(p))),
        "lbAudioAx_800264E4" => i(lbAudioAx_800264E4(ro(p))),
        "lb_800138CC" => {
            p[1].check::<mt::CameraBlurData>();
            // SAFETY: an Option<extern fn> is a nullable pointer; the value is only stored.
            let cb: mt::HSD_GObjEvent = unsafe { std::mem::transmute::<usize, mt::HSD_GObjEvent>(s[0] as usize) };
            lb_800138CC(p[0].r(), cb);
            0
        }
        "lb_800138D8" => { p[1].check::<mt::CameraBlurData>(); lb_800138D8(p[0].r(), s[0] as u8 as i8); 0 }
        "lb_8001204C" => {
            for q in &p[3..] { q.check::<mt::HSD_JObj>(); }
            let root = p[0].o::<mt::HSD_JObj>().map(|j| &*j);
            let ind = p[2].os::<u16>().map(|x| &*x);
            i(lb_8001204C(root, p[1].os(), ind, s32(0)))
        }
        // ---- mn
        "mnDiagram3_GetPopupSpacing" => {
            for q in &p[2..] { q.check::<mt::HSD_JObj>(); }
            fb(mnDiagram3_GetPopupSpacing(p[0].r(), p[1].r()))
        }
        "mnDiagram3_GetPopupY" => fb(mnDiagram3_GetPopupY(p[0].r(), f(s[0]), u8_(1))),
        "mnDiagram3_GetRowSpacing" => {
            for q in &p[1..] { q.check::<mt::HSD_JObj>(); }
            fb(mnDiagram3_GetRowSpacing(p[0].r()))
        }
        "mnName_TextWidth" => fb(mnName_TextWidth(f(s[0]), p[0].r())),
        "mnVibration_GetCursorYSpacing" => fb(mnVibration_GetCursorYSpacing(f(s[0]), p[0].r())),
        _ => panic!("no port for {name}"),
    }
}

/// The first buffer as a sound gobj (NULL -> None); the second, if any, is its user data.
fn ro<'a>(p: &[P]) -> Option<&'a HSD_GObj> {
    if p.len() > 1 {
        p[1].check::<lbAudioAx_UserData>();
    }
    p[0].o::<HSD_GObj>().map(|g| &*g)
}

#[test]
fn group6_round4_callers_match_c() {
    let text = include_str!("data/vectors_r4.txt");
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
    assert_eq!(per.len(), 30, "functions covered (round 4)");
}
