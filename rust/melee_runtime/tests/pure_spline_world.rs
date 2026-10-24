//! `tu::spline_world` vs baselib/spline.c (shipping contraction), bit for bit over random splines of
//! every type. Regenerate the expected data with
//! `python rust/melee_runtime/gen/pure/run_pure.py spline_world`.
mod pure_common;
use melee_runtime::tu::spline_world::*;
use melee_types::*;
use pure_common::*;

#[test]
fn spline_world_matches_c() {
    let mut r = Rng::new();
    let mut out = String::new();
    for op in 0..2 {
        for k in 0..600 {
            let mut cv = [Vec { x: 0.0, y: 0.0, z: 0.0 }; 24];
            let mut lens = [0.0f32; 8];
            let mut poly = [[0.0f32; 5]; 8];
            let mut s: HSD_Spline = HSD_Spline::zeroed();
            for c in cv.iter_mut() {
                c.x = r.f();
                c.y = r.f();
                c.z = r.f();
            }
            for i in 0..8 {
                lens[i] = (r.u() % 1000) as f32 / 100.0;
                for j in 0..5 {
                    poly[i][j] = (r.u() % 10000) as f32 / 8192.0;
                }
            }
            s.r#type = (r.u() % 5) as u8;
            s.numcv = (4 + r.u() % 4) as i16;
            s.tension = r.f();
            s.cv = cv.as_mut_ptr();
            s.totalLength = (1 + r.u() % 10) as f32;
            s.segLength = lens.as_mut_ptr();
            s.segPoly = poly.as_mut_ptr();
            let u = match r.u() % 6 {
                0 => 0.0,
                1 => 1.0,
                2 => -0.25,
                3 => 1.25,
                _ => (r.u() % 1025) as f32 / 1024.0,
            };
            let mut o = Vec { x: 99.0, y: 98.0, z: 97.0 };
            unsafe {
                if op == 0 {
                    splGetSplinePoint(&mut o, &mut s, u)
                } else {
                    splArcLengthPoint(&mut o, &mut s, u * s.totalLength)
                }
            }
            let mut b = [0u8; 12];
            b[0..4].copy_from_slice(&o.x.to_ne_bytes());
            b[4..8].copy_from_slice(&o.y.to_ne_bytes());
            b[8..12].copy_from_slice(&o.z.to_ne_bytes());
            out += &line(&format!("op{op}"), k, fnv(&b));
        }
    }
    compare("spline_world", &out);
}
