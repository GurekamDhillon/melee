//! gw_rollback.c's pure functions vs the C itself: tests/data/rb_oracle.txt is written by
//! oracle/rb_oracle.sh, which cuts rb_hash / rb_neutral_raw / rb_gen / rb_arrival out of
//! pc/platform/gw_rollback.c and builds them as shipping shims are (clang i686-pc-windows-msvc -O2).

use melee_net::session::{rb_arrival, rb_gen, rb_hash, RbConfig};

#[test]
fn rb_pure_functions_match_the_c() {
    let text = include_str!("data/rb_oracle.txt");
    let (mut nh, mut ng, mut na) = (0, 0, 0);
    for line in text.lines() {
        let w: Vec<&str> = line.split(' ').collect();
        let n = |i: usize| w[i].parse::<i64>().unwrap();
        match w[0] {
            "hash" => {
                assert_eq!(rb_hash(n(1) as u32, n(2) as u32, n(3) as u32), n(4) as u32, "{line}");
                nh += 1;
            }
            "gen" => {
                let o = rb_gen(n(1) as u32, n(2) as usize, n(3) as i32);
                let got = format!(
                    "gen {} {} {} {} {} {} {} {} {} {} {} {} {}",
                    w[1], w[2], w[3], o.buttons, o.raw[0], o.raw[1], o.raw[2], o.raw[3], o.pad_l, o.pad_r, o.present, o.is_raw, o.confirmed
                );
                assert_eq!(got, line);
                ng += 1;
            }
            "arr" => {
                let c = RbConfig { fake_lat: n(1) as i32, fake_jit: n(2) as i32, fake_loss: n(3) as i32, delay: n(4) as i32, seed: n(5) as u32, ..RbConfig::default() };
                assert_eq!(rb_arrival(&c, n(6) as usize, n(7) as i32), n(8), "{line}");
                na += 1;
            }
            _ => panic!("{line}"),
        }
    }
    assert!(nh == 2000 && ng == 5 * 4 * 623 && na == 5 * 8 * 323, "{nh} {ng} {na}");
}
