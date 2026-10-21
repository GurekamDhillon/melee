//! melee_runtime::tu::{dbcpu, dbcamera} pure helpers vs the C compiled natively with shipping
//! contraction (oracle/tu_db_ref.c via oracle/gen_tu_db_ref.sh -> tests/data/tu_db_ref.txt).

use melee_runtime::tu::dbcamera::*;
use melee_runtime::tu::dbcpu::{format_cpu_row, CpuHandicapRow};

const REF: &str = include_str!("data/tu_db_ref.txt");

fn hx(s: &str) -> f32 {
    f32::from_bits(u32::from_str_radix(s, 16).unwrap())
}
fn bits(f: f32) -> String {
    format!("{:08x}", f.to_bits())
}

#[test]
fn cpu_row_format_matches_c_printf() {
    let mut n = 0;
    for line in REF.lines().filter(|l| l.starts_with("CPU ")) {
        let (inp, want) = line[4..].split_once('|').unwrap();
        let t: Vec<&str> = inp.split(' ').collect();
        let row = CpuHandicapRow {
            player_state: t[0].parse().unwrap(),
            cpu_level: t[1].parse().unwrap(),
            cpu_type: t[2].parse().unwrap(),
            handicap: t[3].parse().unwrap(),
            unk50: hx(t[4]),
            attack_ratio: hx(t[5]),
            defense_ratio: hx(t[6]),
        };
        assert_eq!(format_cpu_row(&row), format!("\n{want}"), "{line}");
        n += 1;
    }
    assert_eq!(n, 400);
}

#[test]
fn camera_math_bit_exact() {
    let mut n = 0;
    for line in REF.lines().filter(|l| l.starts_with("CAM ")) {
        let t: Vec<&str> = line[4..].split(' ').collect();
        let (x, th, d, fov, big) = (hx(t[0]), hx(t[1]), hx(t[2]), hx(t[3]), hx(t[4]));
        assert_eq!(bits(cstick_threshold(x, th)), t[5], "threshold {line}");
        assert_eq!(info_int(big).to_string(), t[6], "info_int {line}");
        assert_eq!(bits(pan_scale(d, fov)), t[7], "pan_scale {line}");
        assert_eq!(bits(pan_scale_follow(d, fov)), t[8], "pan_scale_follow {line}");
        assert_eq!(bits(dolly_follow(d, x)), t[9], "dolly_follow {line}");
        assert_eq!(bits(dolly_free(d, x)), t[10], "dolly_free {line}");
        n += 1;
    }
    assert_eq!(n, 4000);
}
