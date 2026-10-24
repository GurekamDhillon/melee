//! Glue check for the math adapters in `tu/adapt.rs`: argument order, destination, aliasing and
//! return values of the `PSVEC*` / `PSMTX*` / `MTX*` stub bridges against the verified
//! `melee_runtime::mtx` routines (those are checked against the C by melee_runtime's oracles).
use melee_runtime::mtx::{self, Mtx, Vec3};
use melee_modes::tu::adapt as ad;

fn rng(s: &mut u64) -> f32 {
    *s = s.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
    (((*s >> 33) % 20001) as i32 - 10000) as f32 * 0.01
}
fn v3(s: &mut u64) -> Vec3 { Vec3 { x: rng(s), y: rng(s), z: rng(s) } }
fn mt(s: &mut u64) -> Mtx { let mut m = [[0.0; 4]; 3]; for r in &mut m { for c in r.iter_mut() { *c = rng(s); } } m }
fn bits(v: &Vec3) -> [u32; 3] { [v.x.to_bits(), v.y.to_bits(), v.z.to_bits()] }

#[test]
fn math_adapters_match_runtime() {
    let mut s = 0x1234_5678u64;
    for _ in 0..2000 {
        let (mut a, mut b, mut o) = (v3(&mut s), v3(&mut s), v3(&mut s));
        let k = rng(&mut s);
        ad::PSVECSubtract__594d9f(&mut a as *mut _ as _, &mut b as *mut _ as _, &mut o as *mut _ as _);
        assert_eq!(bits(&o), bits(&mtx::vec_sub(&a, &b)));
        ad::PSVECCrossProduct__0fba16(&mut a as *mut _ as _, &mut b as *mut _ as _, &mut o as *mut _ as _);
        assert_eq!(bits(&o), bits(&mtx::vec_cross(&a, &b)));
        let mut n = a;
        ad::PSVECNormalize__01cbbc(&mut n as *mut _ as _, &mut n as *mut _ as _);
        assert_eq!(bits(&n), bits(&mtx::vec_normalize(&a)));
        assert_eq!(ad::PSVECMag__307672(&mut a as *mut _ as _).to_bits(), mtx::vec_mag(&a).to_bits());
        let _ = k;
        let (mut m1, mut m2, mut mo) = (mt(&mut s), mt(&mut s), [[0.0f32; 4]; 3]);
        ad::PSMTXConcat__e4df5b(m1.as_mut_ptr().cast(), m2.as_mut_ptr().cast(), mo.as_mut_ptr().cast());
        assert_eq!(mo, mtx::concat(&m1, &m2));
        let mut inv = [[7.0f32; 4]; 3];
        let r = ad::PSMTXInverse__c61344(m1.as_mut_ptr().cast(), inv.as_mut_ptr().cast());
        match mtx::inverse(&m1) {
            Some(e) => { assert_eq!(r, 1); assert_eq!(inv, e); }
            None => { assert_eq!(r, 0); assert_eq!(inv, [[7.0f32; 4]; 3]); }
        }
        let mut t = [[0.0f32; 4]; 3];
        ad::PSMTXTranspose__cc6675(m1.as_mut_ptr().cast(), t.as_mut_ptr().cast());
        assert_eq!(t, mtx::transpose(&m1));
        let mut dst = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        ad::PSMTXMultVec__aeae10(m1.as_mut_ptr().cast(), &mut a as *mut _ as _, &mut dst as *mut _ as _);
        assert_eq!(bits(&dst), bits(&mtx::mult_vec(&m1, &a)));
        let mut rm = [[0.0f32; 4]; 3];
        ad::MTXRotRad__bc3d39(rm.as_mut_ptr().cast(), b'y' as i8, k);
        let mut er = [[0.0f32; 4]; 3];
        mtx::rot_rad(&mut er, b'y', k);
        assert_eq!(rm, er);
    }
}
