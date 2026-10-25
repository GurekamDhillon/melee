//! `tu::mtx44::raw` vs the C (`libs/dolphin/src/dolphin/mtx/mtx44.c`, shipping contraction), bit for bit on
//! generated inputs. Regenerate the expected data with `python rust/melee_runtime/gen/pure/run_pure.py mtx44`.
mod pure_common;
use melee_runtime::tu::mtx44::raw::*;
use pure_common::*;

fn bytes<T>(v: &T) -> &[u8] {
    unsafe { core::slice::from_raw_parts(v as *const T as *const u8, core::mem::size_of::<T>()) }
}
fn rm4(r: &mut Rng) -> [[f32; 4]; 4] {
    let mut m = [[0.0; 4]; 4];
    for i in 0..4 {
        for j in 0..4 {
            m[i][j] = r.f();
        }
    }
    m
}

#[test]
fn mtx44_matches_c() {
    let mut r = Rng::new();
    let mut out = String::new();
    for k in 0..400 {
        let (mut a, b, c, d, e, f) = (r.f(), r.f(), r.f(), r.f(), r.f(), r.f());
        let mut m = rm4(&mut r);
        unsafe { MTXFrustum(m.as_mut_ptr(), a, b, c, d, e, f) };
        out += &line("frustum", k, fnv(bytes(&m)));
        let mut m = rm4(&mut r);
        a = if k & 1 != 0 { (r.u() % 17900) as f32 / 100.0 + 0.01 } else { r.f() };
        unsafe { MTXPerspective(m.as_mut_ptr(), a, b, c, d) };
        out += &line("persp", k, fnv(bytes(&m)));
        let mut m = rm4(&mut r);
        unsafe { MTXOrtho(m.as_mut_ptr(), b, c, d, e, f, a) };
        out += &line("ortho", k, fnv(bytes(&m)));
    }
    compare("mtx44", &out);
}
