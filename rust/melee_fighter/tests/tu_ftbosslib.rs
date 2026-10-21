//! tu/ftbosslib.rs vs the C (gen/cref/tu_ftbosslib_ref.c, data in tests/data/tu_ftbosslib.txt;
//! regenerate with `python rust/melee_fighter/gen/cref/tu_run.py tu_ftbosslib`), plus smoke
//! tests of the fighter-state helpers.
use melee_fighter::tu::ftbosslib as m;
use melee_fighter::types::{Fighter, HSD_GObj, Vec3};

fn f(s: &str) -> f32 { f32::from_bits(u32::from_str_radix(s, 16).unwrap()) }

#[test]
fn be40_bit_exact_vs_c() {
    let data = include_str!("data/tu_ftbosslib.txt");
    let mut n = 0;
    for line in data.lines() {
        let w: Vec<&str> = line.split_whitespace().collect();
        assert_eq!(w[0], "BE40");
        let t = Vec3 { x: f(w[1]), y: f(w[2]), z: f(w[3]) };
        let c = Vec3 { x: f(w[4]), y: f(w[5]), z: f(w[6]) };
        let (vx, vy, o) = m::boss_8015be40_math(&t, &c, f(w[7]), f(w[8]));
        assert_eq!((vx.to_bits(), vy.to_bits(), o.to_bits()), (f(w[9]).to_bits(), f(w[10]).to_bits(), f(w[11]).to_bits()), "{line}");
        n += 1;
    }
    assert_eq!(n, 4000);
}

#[test]
fn be40_writes_fighter() {
    let mut fi: Fighter = unsafe { core::mem::zeroed() };
    let mut g: HSD_GObj = unsafe { core::mem::zeroed() };
    g.user_data = &mut fi as *mut Fighter as *mut _;
    let mut out = 9.0;
    m::ftBossLib_8015BE40(&mut g, &Vec3 { x: 3.0, y: 4.0, z: 0.0 }, &mut out, 1.0, 0.5);
    assert_eq!(out, 5.0);
    assert_eq!((fi.self_vel.x, fi.self_vel.y), (1.5, 2.0));
    m::ftBossLib_8015BE40(&mut g, &Vec3 { x: 0.1, y: 0.0, z: 0.0 }, &mut out, 1.0, 0.5);
    assert_eq!(out, 0.0);
}
