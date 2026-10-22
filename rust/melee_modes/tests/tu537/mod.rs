//! Shared by tests/tu_gm_1a*.rs, tu_gmscmemcard.rs, tu_gm_1b03.rs: the xorshift32 stream of
//! gen/turef/tu_gm_common.h and a small hook log.
#![allow(dead_code)]

pub struct Rng(pub u32);
impl Rng {
    pub fn new() -> Rng {
        Rng(0x2468ACE)
    }
    pub fn rnd(&mut self) -> u32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 17;
        self.0 ^= self.0 << 5;
        self.0
    }
    pub fn rf(&mut self) -> f32 {
        match self.rnd() % 8 {
            0 => 0.0,
            1 => -0.0,
            2 => 1.0,
            3 => -1.0,
            _ => ((self.rnd() % 4001) as i32 - 2000) as f32 / 997.0,
        }
    }
}

pub fn data(name: &str) -> String {
    std::fs::read_to_string(format!("{}/tests/data/{}.txt", env!("CARGO_MANIFEST_DIR"), name)).unwrap()
}
