//! src/tu/jobj.rs (native pointer-level jobj.c / jobj.h subset) vs the C bodies in gen/native/jobj_ref.c
//! (clang, shipping contraction): 300 random trees x 40 random operations, state hashed after each.
//! Regenerate tests/data/native_jobj.txt by building and running gen/native/jobj_ref.c.
#![allow(non_snake_case)]
use melee_items::tu::jobj::*;
use melee_types::*;

struct Rng(u32);
impl Rng {
    fn next(&mut self) -> u32 {
        self.0 = self.0.wrapping_mul(214013).wrapping_add(2531011);
        self.0 >> 8
    }
    fn flags(&mut self) -> u32 {
        const BITS: [u32; 13] = [8, 64, 4096, 8388608, 16777216, 33554432, 1 << 28, 1 << 29, 1 << 30, 0x20000, 1, 2, 4];
        let mut f = 0;
        for b in BITS {
            if self.next() & 1 != 0 {
                f |= b;
            }
        }
        f
    }
}

const N: usize = 12;

fn hash_all(h: &mut u64, n: &[HSD_JObj; N]) {
    let mut hm = |x: u32| *h = (*h ^ x as u64).wrapping_mul(1099511628211);
    let idx = |p: *mut HSD_JObj| -> u32 {
        if p.is_null() { 99 } else { ((p as usize - n.as_ptr() as usize) / core::mem::size_of::<HSD_JObj>()) as u32 }
    };
    for j in n.iter() {
        hm(j.flags);
        for f in [j.rotate.y, j.scale.x, j.scale.y, j.scale.z, j.translate.x, j.translate.y, j.translate.z] {
            hm(f.to_bits());
        }
        hm(idx(j.parent));
        hm(idx(j.child));
        hm(idx(j.next));
    }
}

#[test]
fn native_jobj_matches_c() {
    let expected: std::vec::Vec<u64> = include_str!("data/native_jobj.txt").lines().map(|l| u64::from_str_radix(l.trim(), 16).unwrap()).collect();
    assert_eq!(expected.len(), 300);
    for round in 0..300u32 {
        let mut r = Rng(1000 + round);
        let mut h = 14695981039346656037u64;
        let mut n: [HSD_JObj; N] = unsafe { core::mem::zeroed() };
        let p = n.as_mut_ptr();
        unsafe {
            for i in 0..N {
                n[i].flags = r.flags() & !(JOBJ_INSTANCE as u32);
            }
            for i in 1..N {
                let par = (r.next() % i as u32) as usize;
                HSD_JObjAddChild(p.add(par), p.add(i));
            }
            for i in 0..N {
                if r.next() % 4 == 0 {
                    n[i].flags |= JOBJ_INSTANCE as u32;
                }
            }
            hash_all(&mut h, &n);
            for _ in 0..40 {
                let op = r.next() % 11;
                let k = (r.next() % N as u32) as usize;
                let f = r.flags();
                let v = (r.next() % 2000) as i32 - 1000;
                let v = v as f32 / 8.0;
                let q = p.add(k);
                match op {
                    0 => HSD_JObjSetMtxDirtySub(q),
                    1 => HSD_JObjSetFlags(q, f),
                    2 => HSD_JObjSetFlagsAll(q, f),
                    3 => HSD_JObjClearFlags(q, f),
                    4 => HSD_JObjClearFlagsAll(q, f),
                    5 => {
                        (*q).flags &= !(JOBJ_USE_QUATERNION as u32);
                        HSD_JObjSetRotationYWithMtxDirty(q, v)
                    }
                    6 => HSD_JObjSetScaleXWithMtxDirty(q, v),
                    7 => HSD_JObjSetScaleYWithMtxDirty(q, v),
                    8 => HSD_JObjSetScaleZWithMtxDirty(q, v),
                    9 => {
                        HSD_JObjSetTranslateXWithMtxDirty(q, v);
                        HSD_JObjSetTranslateYWithMtxDirty(q, v + 1.0);
                    }
                    _ => HSD_JObjSetTranslateZWithMtxDirty(q, v),
                }
                hash_all(&mut h, &n);
            }
        }
        assert_eq!(h, expected[round as usize], "round {round}");
    }
}
