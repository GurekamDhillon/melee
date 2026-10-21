//! tu::ftco_0d72 vs the C (gen/cref/tu_ftco_0d72_ref.c; data tests/data/tu_ftco_0d72.txt;
//! regenerate with `python rust/melee_fighter/gen/cref/tu_shard97.py`).

use melee_fighter::tu::ftco_0d72::ftco_800d72a0;
use melee_fighter::types::*;

struct Rng(u32);
impl Rng {
    fn rnd(&mut self) -> u32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 17;
        self.0 ^= self.0 << 5;
        self.0
    }
    fn ri(&mut self) -> i32 {
        match self.rnd() % 8 {
            0 => -1,
            1 => 0x7FFFFFF0,
            2 => i32::MIN,
            _ => (self.rnd() % 400) as i32,
        }
    }
}

#[test]
fn matches_c() {
    let want = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/tu_ftco_0d72.txt")).unwrap();
    let mut f: Box<Fighter> = Box::new(Fighter::zeroed());
    let mut x: Box<Fighter_x2D0_t> = Box::new(Fighter_x2D0_t::zeroed());
    f.x2D0 = &mut *x;
    let mut r = Rng(0x2468ACE);
    let mut n = 0;
    for (i, line) in want.lines().enumerate() {
        let (a, b) = (r.ri(), r.ri());
        let c = if r.rnd() % 4 == 0 { r.ri() } else { (r.rnd() % 40) as i32 };
        let m = (r.rnd() % 400) as i32;
        unsafe {
            (*f.x2D0).x2C = a;
            (*f.x2D0).x30 = b;
            (*f.x2D0).x28 = c;
        }
        f.motion_id = m;
        assert_eq!(format!("{} {}", i, ftco_800d72a0(&f) as i32), line, "case {i}");
        n += 1;
    }
    assert_eq!(n, 4000);
}
