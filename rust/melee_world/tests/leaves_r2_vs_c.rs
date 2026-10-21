//! Night-3 round-2 leaves (gr::leaves, hud::leaves) against the C
//! (cref/gen_leaves_r2_ref.sh -> tests/data/leaves_r2_ref.txt), shipping-build contraction.
use melee_world::gr::leaves::*;
use melee_world::hud::leaves::*;
use melee_world::vec::Vec3;

const DATA: &str = include_str!("data/leaves_r2_ref.txt");

fn h(s: &str) -> u32 {
    u32::from_str_radix(s, 16).unwrap()
}
fn f(u: u32) -> f32 {
    f32::from_bits(u)
}

#[test]
fn leaves_r2_match_c() {
    let mut counts = std::collections::BTreeMap::new();
    for (ln, line) in DATA.lines().enumerate() {
        let tag = &line[..1];
        let (ins, outs) = line[2..].split_once('|').unwrap();
        let ins = ins.trim();
        let o: Vec<u32> = outs.split_whitespace().map(h).collect();
        let i: Vec<u32> = if ins == "-" { vec![] } else { ins.split_whitespace().map(h).collect() };
        let ctx = || format!("line {ln}: {line}");
        match tag {
            "G" => {
                let v = Vec3 { x: f(i[0]), y: f(i[1]), z: 0.0 };
                let a = [f(i[2]), f(i[3]), f(i[4]), f(i[5])];
                assert_eq!(grgreens_80213ab4(&v, a[0], a[1], a[2], a[3]) as u32, o[0], "{}", ctx());
                assert_eq!(groldpupupu_8021128c(&v, a[0], a[1], a[2], a[3]) as u32, o[1], "{}", ctx());
            }
            "S" => assert_eq!(grstadium_scale_color(i[0] as u8) as u32, o[0], "{}", ctx()),
            "D" => {
                let a = Vec3 { x: f(i[0]), y: f(i[1]), z: f(i[2]) };
                let b = Vec3 { x: f(i[3]), y: f(i[4]), z: f(i[5]) };
                assert_eq!(grmc_distance_squared(&a, &b).to_bits(), o[0], "{}", ctx());
            }
            "F" => {
                assert_eq!(grzebes_subtract(f(i[0]), f(i[1])).to_bits(), o[0], "{}", ctx());
                assert_eq!(grinishie1_fabsf_inline(f(i[0])).to_bits(), o[1], "{}", ctx());
            }
            "K" => {
                let k = i[0] as u16 as i16;
                let mut gp: melee_types::Ground = unsafe { core::mem::zeroed() };
                unsafe { gp.u.inishie2.xC6 = k };
                grkongo_801d8270_gp(&mut gp);
                assert_eq!(unsafe { gp.u.inishie2.xC6 } as u16 as u32, o[0], "{}", ctx());
                let mut gp: melee_types::Ground = unsafe { core::mem::zeroed() };
                unsafe { gp.u.taru.xC6 = k };
                groldkongo_802105ac_gp(&mut gp);
                assert_eq!(unsafe { gp.u.taru.xC6 } as u16 as u32, o[1], "{}", ctx());
            }
            "N" => {
                if i.is_empty() {
                    assert_eq!(devtext_strlen(None) as u32, o[0]);
                } else {
                    let mut s = [0x55u8; 16];
                    devtext_num_to_str(i[0] as i32, &mut s);
                    assert_eq!(devtext_strlen(Some(&s)) as u32, o[0], "{}", ctx());
                    let want: Vec<u8> = o[1..].iter().map(|&x| x as u8).collect();
                    assert_eq!(&s[..13], &want[..], "{}", ctx());
                }
            }
            "C" => assert_eq!(devtext_clamp(i[0] as i32, i[1] as i32) as u32, o[0], "{}", ctx()),
            "I" => {
                let (c, p) = (i[0] as i32, i[1] as i32);
                assert_eq!(ifstock_802f89f8_count(c) as u32, o[0], "{}", ctx());
                assert_eq!(ifstock_802f89f8_divisor(p) as u32, o[1], "{}", ctx());
                assert_eq!(ifstock_802f89f8_digit(c, p) as u32, o[2], "{}", ctx());
            }
            "Q" => assert_eq!(ifnametag_inline_a1(f(i[0])).to_bits(), o[0], "{}", ctx()),
            "O" => {
                let mut fl = i[0] as u8 as i8;
                let r = ifstatus_foo(&mut fl, f(i[1]), f(i[2]));
                assert_eq!((r.to_bits(), fl as u8 as u32), (o[0], o[1]), "{}", ctx());
            }
            _ => panic!("bad tag {tag}"),
        }
        *counts.entry(tag.to_string()).or_insert(0) += 1;
    }
    let got: Vec<(String, i32)> = counts.into_iter().collect();
    let want: Vec<(String, i32)> = [
        ("C", 5000), ("D", 20000), ("F", 5000), ("G", 20000), ("I", 5000), ("K", 12), ("N", 5001),
        ("O", 5000), ("Q", 5000), ("S", 256),
    ]
    .iter()
    .map(|&(a, b)| (a.to_string(), b))
    .collect();
    assert_eq!(got, want);
}

/// Mutation check: an unfused grMc_DistanceSquared must fail some rows (the sites are live).
#[test]
fn distance_squared_fusion_is_observable() {
    let mut diff = 0;
    for line in DATA.lines().filter(|l| l.starts_with("D ")) {
        let i: Vec<u32> = line[2..].split('|').next().unwrap().split_whitespace().map(h).collect();
        let (dx, dy, dz) = (f(i[0]) - f(i[3]), f(i[1]) - f(i[4]), f(i[2]) - f(i[5]));
        let unfused = dx * dx + dy * dy + dz * dz;
        let a = Vec3 { x: f(i[0]), y: f(i[1]), z: f(i[2]) };
        let b = Vec3 { x: f(i[3]), y: f(i[4]), z: f(i[5]) };
        if unfused.to_bits() != grmc_distance_squared(&a, &b).to_bits() {
            diff += 1;
        }
    }
    assert!(diff > 0);
}
