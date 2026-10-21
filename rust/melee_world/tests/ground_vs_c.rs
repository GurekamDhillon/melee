//! melee_world::gr::ground against the C (cref/gen_ground_ref.sh -> tests/data/ground_ref.txt),
//! built with the shipping build's FMA contraction. Bit-exact on every output.
use melee_world::gr::ground::{ground_4b50_angles, ground_5630_volume, ground_5694_pan, ground_801c5aec};
use melee_world::vec::Vec3;

const DATA: &str = include_str!("data/ground_ref.txt");

fn h(s: &str) -> u32 {
    u32::from_str_radix(s, 16).unwrap()
}
fn v(a: &[u32]) -> Vec3 {
    Vec3 { x: f32::from_bits(a[0]), y: f32::from_bits(a[1]), z: f32::from_bits(a[2]) }
}
fn bits(v: Vec3) -> [u32; 3] {
    [v.x.to_bits(), v.y.to_bits(), v.z.to_bits()]
}

#[test]
fn ground_matches_c() {
    let (mut nb, mut ne, mut nv) = (0, 0, 0);
    for (ln, line) in DATA.lines().enumerate() {
        let tag = &line[..1];
        let (ins, outs) = line[2..].split_once('|').unwrap_or((&line[2..], ""));
        let i: Vec<u32> = ins.split_whitespace().map(h).collect();
        let o: Vec<u32> = outs.split_whitespace().map(h).collect();
        match tag {
            "B" => {
                assert_eq!(bits(ground_4b50_angles(v(&i))), [o[0], o[1], o[2]], "line {ln}: {line}");
                nb += 1;
            }
            "E" => {
                let r = ground_801c5aec(&v(&i[0..3]), &v(&i[3..6]), &v(&i[6..9]));
                assert_eq!(bits(r), [o[0], o[1], o[2]], "line {ln}: {line}");
                ne += 1;
            }
            "V" => {
                let val = f32::from_bits(i[0]);
                assert_eq!(ground_5630_volume(val) as u32, i[1], "line {ln}");
                assert_eq!(ground_5694_pan(val) as u32, i[2], "line {ln}");
                nv += 1;
            }
            _ => panic!("bad tag {tag}"),
        }
    }
    assert_eq!((nb, ne, nv), (20000, 20000, 20000));
}
