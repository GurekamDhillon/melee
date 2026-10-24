//! `tu::mtx_world` vs baselib/mtx.c (shipping contraction), bit for bit. Regenerate the expected data
//! with `python rust/melee_runtime/gen/pure/run_pure.py mtx_world`.
mod pure_common;
use melee_runtime::tu::mtx_world::*;
use melee_types::Vec;
use pure_common::*;

fn rm(r: &mut Rng, singular: bool) -> [[f32; 4]; 3] {
    let mut m = [[0.0f32; 4]; 3];
    for i in 0..3 {
        for j in 0..4 {
            m[i][j] = r.f();
        }
    }
    if singular {
        for j in 0..4 {
            m[1][j] = m[0][j] * 2.0;
        }
    }
    m
}
fn bytes<T>(v: &T) -> &[u8] {
    unsafe { core::slice::from_raw_parts(v as *const T as *const u8, core::mem::size_of::<T>()) }
}

#[test]
fn mtx_world_matches_c() {
    let mut r = Rng::new();
    let mut out = String::new();
    for op in 0..4 {
        for k in 0..400 {
            let sing = r.u() % 5 == 0;
            let mut inv = rm(&mut r, sing);
            let mut src = rm(&mut r, false);
            let mut dst = rm(&mut r, false);
            let mut sc = Vec { x: 77.0, y: 78.0, z: 79.0 };
            let h = unsafe {
                match op {
                    0 => {
                        HSD_MtxGetScale(src.as_mut_ptr(), &mut sc);
                        fnv(bytes(&sc))
                    }
                    1 => {
                        HSD_MtxInverseConcat(inv.as_mut_ptr(), src.as_mut_ptr(), dst.as_mut_ptr());
                        fnv(bytes(&dst))
                    }
                    2 => {
                        HSD_MtxInverseConcat(inv.as_mut_ptr(), src.as_mut_ptr(), src.as_mut_ptr());
                        fnv(bytes(&src))
                    }
                    _ => {
                        HSD_MtxInverseConcat(inv.as_mut_ptr(), inv.as_mut_ptr(), inv.as_mut_ptr());
                        fnv(bytes(&inv))
                    }
                }
            };
            out += &line(&format!("op{op}"), k, h);
        }
    }
    compare("mtx_world", &out);
}
