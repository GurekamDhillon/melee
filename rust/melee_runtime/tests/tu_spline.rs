//! All eleven source definitions directly checked, plus every synthetic input field.
#[path = "../src/tu/spline.rs"]
mod m;
use melee_runtime::mtx::Vec3;
pub use melee_runtime::spline;
fn next(r: &mut u32) -> u32 {
    *r ^= *r << 13;
    *r ^= *r >> 17;
    *r ^= *r << 5;
    *r
}
fn draw(r: &mut u32) -> f32 {
    ((next(r) % 200001) as i32 - 100000) as f32 / 8192.0
}
fn pv(o: &mut Vec<u32>, v: Vec3) {
    o.extend([v.x.to_bits(), v.y.to_bits(), v.z.to_bits()]);
}
#[test]
fn all_spline_definitions_match_native_c() {
    for (line, row) in include_str!("data/oracle_spline_direct.txt")
        .lines()
        .filter(|s| !s.trim().is_empty())
        .enumerate()
    {
        let seed = line as u32 + 1;
        let mut rng = seed;
        let cv: [Vec3; 16] = core::array::from_fn(|_| Vec3 {
            x: draw(&mut rng),
            y: draw(&mut rng),
            z: draw(&mut rng),
        });
        let poly: [[f32; 5]; 4] = core::array::from_fn(|_| {
            core::array::from_fn(|_| (next(&mut rng) % 10000) as f32 / 8192.0)
        });
        let x: [f32; 6] = core::array::from_fn(|_| draw(&mut rng));
        let mut u = (seed % 1025) as f32 / 1024.0;
        if seed % 23 == 0 {
            u = -0.25;
        }
        if seed % 29 == 0 {
            u = 1.25;
        }
        let lengths = [0., 0.25, 0.5, 0.75, 1.];
        let s = m::Spline {
            ty: (seed % 5) as u8,
            numcv: 5,
            tension: x[0],
            cv: &cv,
            total_length: 4.,
            seg_length: &lengths,
            seg_poly: &poly,
        };
        let mut o = vec![
            seed,
            m::splGetHelmite(x[0], x[1], x[2], x[3], x[4], x[5]).to_bits(),
        ];
        pv(&mut o, m::splGetCardinalPoint(&cv, x[0], u));
        pv(&mut o, m::splGetBSplinePoint(&cv, u));
        pv(&mut o, m::splGetBezierPoint(&cv, u));
        let unchanged = Vec3 {
            x: 99.,
            y: 99.,
            z: 99.,
        };
        pv(&mut o, m::splGetSplinePoint(&s, u).unwrap_or(unchanged));
        o.push(m::splArcLengthPolynomial(&poly[0], u).to_bits());
        let c = m::spl_GetCoeffs(&s, (seed % 4) as i32);
        o.push(((c.as_ptr() as usize - poly.as_ptr() as usize) / 4) as u32);
        let dx = m::spl_GetArcLengthDx(x[1], x[2]);
        o.push(dx.to_bits());
        o.push(
            m::spl_IterateSimpsonsMiddle(&poly[(seed % 4) as usize], dx, x[3])
                .to_bits(),
        );
        o.push(m::splArcLengthGetParameter(&s, u).to_bits());
        pv(&mut o, m::splArcLengthPoint(&s, u).unwrap_or(unchanged));
        o.extend([
            s.ty as u32,
            s.numcv as u32,
            s.tension.to_bits(),
            s.total_length.to_bits(),
        ]);
        for p in cv {
            pv(&mut o, p);
        }
        o.extend(lengths.map(f32::to_bits));
        for p in poly {
            o.extend(p.map(f32::to_bits));
        }
        let expected: Vec<u32> = row
            .split_whitespace()
            .map(|v| u32::from_str_radix(v, 16).unwrap())
            .collect();
        assert_eq!(o.len(), expected.len());
        for (field, (a, b)) in o.iter().zip(expected.iter()).enumerate() {
            assert_eq!(a, b, "seed={seed}, field={field}");
        }
    }
}
