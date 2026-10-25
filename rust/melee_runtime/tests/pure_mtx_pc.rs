//! `tu::mtx_pc::raw` vs the C (`pc/gameworld/mtx_pc.c`, shipping contraction), bit for bit on
//! generated inputs. Regenerate the expected data with
//! `python rust/melee_runtime/gen/pure/run_pure.py mtx_pc`.
mod pure_common;
use melee_runtime::tu::mtx_pc::raw::*;
use melee_types::Vec;
use pure_common::*;

fn rv(r: &mut Rng) -> Vec {
    Vec { x: r.f(), y: r.f(), z: r.f() }
}
fn rm(r: &mut Rng) -> [[f32; 4]; 3] {
    let mut m = [[0.0; 4]; 3];
    for i in 0..3 {
        for j in 0..4 {
            m[i][j] = r.f();
        }
    }
    m
}
fn bytes<T>(v: &T) -> &[u8] {
    unsafe { core::slice::from_raw_parts(v as *const T as *const u8, core::mem::size_of::<T>()) }
}

#[test]
fn mtx_pc_matches_c() {
    let mut r = Rng::new();
    let mut out = String::new();
    for k in 0..400 {
        let (mut a, mut b, mut c) = (rv(&mut r), rv(&mut r), rv(&mut r));
        unsafe { PSVECAdd(&mut a, &mut b, &mut c) };
        out += &line("add", k, fnv(bytes(&c)));
        let (mut a, mut b, mut c) = (rv(&mut r), rv(&mut r), rv(&mut r));
        unsafe { PSVECCrossProduct(&mut a, &mut b, &mut c) };
        out += &line("cross", k, fnv(bytes(&c)));
        let (mut a, mut b) = (rv(&mut r), rv(&mut r));
        unsafe { PSVECCrossProduct(&mut a, &mut b, &mut a) };
        out += &line("crossa", k, fnv(bytes(&a)));
        let mut m1 = rm(&mut r);
        unsafe { PSMTXIdentity(m1.as_mut_ptr()) };
        out += &line("ident", k, fnv(bytes(&m1)));
        let (mut m1, mut m2, mut m3) = (rm(&mut r), rm(&mut r), rm(&mut r));
        unsafe { PSMTXConcat(m1.as_mut_ptr(), m2.as_mut_ptr(), m3.as_mut_ptr()) };
        out += &line("concat", k, fnv(bytes(&m3)));
        let (mut m1, mut m2) = (rm(&mut r), rm(&mut r));
        unsafe { PSMTXConcat(m1.as_mut_ptr(), m2.as_mut_ptr(), m1.as_mut_ptr()) };
        out += &line("concat_a", k, fnv(bytes(&m1)));
        let (fov, asp, ss, st, ts, tt) = (r.f() * 0.1, r.f(), r.f(), r.f(), r.f(), r.f());
        let mut m1 = rm(&mut r);
        unsafe { MTXLightPerspective(m1.as_mut_ptr(), fov, asp, ss, st, ts, tt) };
        out += &line("lightp", k, fnv(bytes(&m1)));
        {
            let (mut v1, _v2, mut v3) = (rv(&mut r), rv(&mut r), rv(&mut r));
            unsafe { PSVECNormalize(&mut v1, &mut v3) };
            out += &line("vnorm", k, fnv(bytes(&v3)));
            let mut v1 = rv(&mut r);
            let p = &mut v1 as *mut Vec;
            unsafe { PSVECNormalize(p, p) };
            out += &line("vnorma", k, fnv(bytes(&v1)));
            let mut v1 = rv(&mut r);
            let s1 = unsafe { PSVECMag(&mut v1) };
            out += &line("vmag", k, fnv(bytes(&s1)));
            let (mut v1, mut v2) = (rv(&mut r), rv(&mut r));
            let s1 = unsafe { PSVECDotProduct(&mut v1, &mut v2) };
            out += &line("vdot", k, fnv(bytes(&s1)));
            let mut m1 = rm(&mut r);
            let (mut v1, mut v2, mut v3) = (rv(&mut r), rv(&mut r), rv(&mut r));
            unsafe { C_MTXLookAt(m1.as_mut_ptr(), &mut v1, &mut v2, &mut v3) };
            out += &line("lookat", k, fnv(bytes(&m1)));
            let (tx, ty, tz) = (r.f(), r.f(), r.f());
            let mut m1 = rm(&mut r);
            unsafe { PSMTXTrans(m1.as_mut_ptr(), tx, ty, tz) };
            out += &line("trans", k, fnv(bytes(&m1)));
            let (mut m1, mut m2) = (rm(&mut r), rm(&mut r));
            unsafe { PSMTXTranspose(m1.as_mut_ptr(), m2.as_mut_ptr()) };
            out += &line("transp", k, fnv(bytes(&m2)));
            let mut m1 = rm(&mut r);
            unsafe { PSMTXTranspose(m1.as_mut_ptr(), m1.as_mut_ptr()) };
            out += &line("transpa", k, fnv(bytes(&m1)));
            let mut m1 = rm(&mut r);
            let mut q = melee_types::Quaternion { x: r.f(), y: r.f(), z: r.f(), w: r.f() };
            unsafe { PSMTXQuat(m1.as_mut_ptr(), &mut q) };
            out += &line("quat", k, fnv(bytes(&m1)));
            {
                const AX: [i8; 7] = [b'x' as i8, b'X' as i8, b'y' as i8, b'Y' as i8, b'z' as i8, b'Z' as i8, b'q' as i8];
                let a = AX[(r.u() % 7) as usize];
                let (sn, cs) = (r.f(), r.f());
                let mut m1 = rm(&mut r);
                unsafe { mtx_rot_trig(m1.as_mut_ptr(), a, sn, cs) };
                out += &line("rottrig", k, fnv(bytes(&m1)));
            }
            {
                let (t, b, l, rr, ss, st, ts, tt) = (r.f(), r.f(), r.f(), r.f(), r.f(), r.f(), r.f(), r.f());
                let mut m1 = rm(&mut r);
                unsafe { MTXLightOrtho(m1.as_mut_ptr(), t, b, l, rr, ss, st, ts, tt) };
                out += &line("lighto", k, fnv(bytes(&m1)));
            }
        }
    }
    compare("mtx_pc", &out);
}
