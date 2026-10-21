//! melee_world::gr::grlib::point_inside_box (grlib.c PointInsideColl) against the C
//! (cref/gen_grlib_ref.sh -> tests/data/grlib_ref.txt), built with the shipping build's FMA
//! contraction. Bit-exact on every row.
use melee_world::gr::grlib::point_inside_box;
use melee_world::vec::Vec3;

const DATA: &str = include_str!("data/grlib_ref.txt");

fn rows() -> impl Iterator<Item = ([f32; 9], bool)> {
    DATA.lines().map(|l| {
        let t: Vec<&str> = l.split_whitespace().collect();
        assert_eq!(t[0], "P");
        let mut a = [0f32; 9];
        for i in 0..9 {
            a[i] = f32::from_bits(u32::from_str_radix(t[1 + i], 16).unwrap());
        }
        (a, t[10] == "1")
    })
}

#[test]
fn grlib_point_inside_matches_c() {
    let mut n = 0;
    for (a, want) in rows() {
        let p = Vec3 { x: a[6], y: a[7], z: 0.0 };
        assert_eq!(point_inside_box(a[0], a[1], a[2], a[3], a[4], a[5], &p, a[8]), want, "row {n}: {a:?}");
        n += 1;
    }
    assert_eq!(n, 40000);
}

// No fused-vs-unfused mutation check: every fused site multiplies by 0.5f, which is exact for
// normal values, so fma(0.5, a, b) == 0.5*a + b and no input separates them (tried: 40000 rows,
// a third on the exact edge). The port still fuses where the shipping build does.
