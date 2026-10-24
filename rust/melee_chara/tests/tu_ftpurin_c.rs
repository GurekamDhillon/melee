//! tu::ftpurin::ftPr_Init_8013C2F8 vs the C (gen/turef/tu_ftpurin_ref.c, the function cut verbatim from
//! ftpurin.c; data in tests/data/tu_ftpurin.txt, regenerate with
//! `python rust/melee_chara/gen/turef/tu_pre.py tu_ftpurin`). Slots 0..4 are cleared, the extra slot 5 is kept.
#![allow(static_mut_refs)]

use melee_chara::tu::ftpurin as t;
use melee_types::*;

struct Rng(u32);
impl Rng {
    fn rnd(&mut self) -> u32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 17;
        self.0 ^= self.0 << 5;
        self.0
    }
}

#[test]
fn matches_c() {
    let want = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/tu_ftpurin.txt")).unwrap();
    let mut r = Rng(0x8013C2);
    let mut n = 0;
    for (i, line) in want.lines().enumerate() {
        unsafe {
            for k in 0..6 {
                t::ft_8045A1E0[k] = if r.rnd() & 1 != 0 { (0x1000 + (r.rnd() % 100) as usize) as *mut HSD_Joint } else { core::ptr::null_mut() };
            }
        }
        t::ftPr_Init_8013C2F8();
        let mut out = format!("{}", i);
        unsafe {
            for k in 0..6 {
                out.push_str(&format!(" {}", t::ft_8045A1E0[k] as usize as u32));
            }
        }
        assert_eq!(out, line, "case {i}");
        n += 1;
    }
    assert_eq!(n, 200);
}
