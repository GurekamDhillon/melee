//! tu::ftco_0d67 vs the C (gen/cref/tu_ftco_0d67_ref.c, shipping contraction; data in
//! tests/data/tu_ftco_0d67.txt; regenerate with `python rust/melee_fighter/gen/cref/tu_shard97.py`).
#![allow(static_mut_refs)]

use melee_fighter::hooks as h;
use melee_fighter::tu::ftco_0d67::ftco_800d67c4;
use melee_fighter::types::*;

struct Rng(u32);
impl Rng {
    fn rnd(&mut self) -> u32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 17;
        self.0 ^= self.0 << 5;
        self.0
    }
    fn rf(&mut self) -> f32 {
        match self.rnd() % 8 {
            0 => 0.0,
            1 => -0.0,
            2 => 0.5,
            3 => -0.5,
            _ => ((self.rnd() % 2001) as i32 - 1000) as f32 / 997.0,
        }
    }
}

#[test]
fn matches_c() {
    let want = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/tu_ftco_0d67.txt")).unwrap();
    let mut f: Box<Fighter> = Box::new(Fighter::zeroed());
    let cd: &'static mut ftCommonData = Box::leak(Box::new(ftCommonData::zeroed()));
    unsafe { h::P_FT_COMMON_DATA = cd as *const _ };
    let cd = unsafe { &mut *(h::P_FT_COMMON_DATA as *mut ftCommonData) };
    let mut r = Rng(0x1234567);
    let mut n = 0;
    for (i, line) in want.lines().enumerate() {
        f.input.pressed_buttons = r.rnd() & 0x300;
        f.input.lstick[0].x = r.rf();
        f.input.lstick[0].y = r.rf();
        cd.x218 = r.rf();
        cd.x21C = r.rf();
        assert_eq!(format!("{} {}", i, ftco_800d67c4(&f) as i32), line, "case {i}");
        n += 1;
    }
    assert_eq!(n, 4000);
}
