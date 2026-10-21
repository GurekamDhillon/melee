//! The Rust PPC interpreter vs pc/platform/gw_ppc.c itself: tests/data/ppc_oracle.txt is written
//! by oracle/ppc_oracle.sh (the C execute core cut out of gw_ppc.c, clang i686-msvc -O2). Every
//! case's initial state is rebuilt here with the same generator, the program is run through
//! `mex_import::ppc`, and exit kind, final pc and the FNV hash of the whole machine (registers,
//! the 64 KiB test memory, the bridge call log) must match.

use mex_import::ppc::{Bridge, Cpu, Fault, FlatMem, Machine, Mem};

const MEM_BASE: u32 = 0x8030_0000;
const MEM_SIZE: usize = 0x10000;

struct Rng(u64);
impl Rng {
    fn rnd(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
    fn r32(&mut self) -> u32 {
        self.rnd() as u32
    }
    fn below(&mut self, n: u32) -> u32 {
        (self.rnd() % n as u64) as u32
    }
}

const FP_SPECIAL: [u64; 20] = [
    0x0000000000000000, 0x8000000000000000, 0x3FF0000000000000, 0xBFF0000000000000,
    0x7FF0000000000000, 0xFFF0000000000000, 0x7FF8000000000000, 0x7FF8000012345678,
    0xFFF4000000000001, 0x0000000000000001, 0x41E0000000000000, 0xC1E0000000200000,
    0xC1E0000000000000, 0x41DFFFFFFFC00000, 0x3FE0000000000000, 0xBFE0000000000000,
    0x4004000000000000, 0x47EFFFFFE0000000, 0x36A0000000000000, 0x4330000080000000,
];

/// ppc_oracle.c init_state.
fn init_state(seed: u64) -> (Cpu, Vec<u8>) {
    let mut g = Rng(seed.wrapping_mul(0x2545_F491_4F6C_DD1D).wrapping_add(1));
    let mut c = Cpu::default();
    for i in 0..32 {
        let k = g.below(4);
        c.gpr[i] = if k == 0 { g.below(17).wrapping_sub(8) } else if k == 1 { g.r32() | 0x8000_0000 } else { g.r32() };
    }
    c.gpr[1] = MEM_BASE + 0xF000;
    c.gpr[27] = g.below(0x40) * 4;
    for i in 28..32 {
        c.gpr[i] = MEM_BASE + 0x8000 + (i as u32 - 28) * 0x800 + g.below(0x40) * 4;
    }
    for i in 0..32 {
        let k = g.below(5);
        c.fpr[i] = if k == 0 {
            FP_SPECIAL[g.below(20) as usize]
        } else if k == 1 {
            (f32::from_bits(g.r32()) as f64).to_bits()
        } else if k == 2 {
            let num = g.r32() as i32 as f64;
            let sh = g.below(24);
            (num / (1u32 << sh) as f64).to_bits()
        } else {
            g.rnd()
        };
    }
    c.cr = g.r32();
    c.xer = g.r32() & 0xE000_007F;
    c.ctr = g.below(4);
    c.lr = 0;
    c.pc = MEM_BASE;
    let mem = (0..MEM_SIZE as u32).map(|i| ((i.wrapping_mul(131).wrapping_add(7)) ^ (i >> 8)) as u8).collect();
    (c, mem)
}

fn fnv(mut h: u64, v: u64, bytes: u32) -> u64 {
    for i in 0..bytes {
        h ^= (v >> (8 * i)) as u8 as u64;
        h = h.wrapping_mul(0x100000001B3);
    }
    h
}

/// ppc_oracle.c's recording gw_ppc_bridge_call.
struct TestBridge {
    code_hi: u32,
    log: u64,
}
impl<M: Mem> Bridge<M> for TestBridge {
    fn in_blob(&self, a: u32) -> bool {
        (MEM_BASE..self.code_hi).contains(&a)
    }
    fn call(&mut self, c: &mut Cpu, _m: &mut M, t: u32) -> Result<(), Fault> {
        self.log = fnv(self.log, t as u64, 4);
        self.log = fnv(self.log, c.gpr[3] as u64, 4);
        c.gpr[3] = t.wrapping_mul(2654435761) ^ c.gpr[4];
        c.set_f(1, c.gpr[5] as i32 as f64 * 0.25);
        Ok(())
    }
}

#[test]
fn interpreter_matches_gw_ppc_c() {
    let data = include_str!("data/ppc_oracle.txt");
    let mut n = 0;
    let mut bad = vec![];
    let mut kinds = [0u32; 6];
    for line in data.lines() {
        let (lhs, rhs) = line.split_once(" = ").unwrap();
        let mut it = lhs.split_whitespace();
        let seed: u64 = it.next().unwrap().parse().unwrap();
        let cnt: usize = it.next().unwrap().parse().unwrap();
        let words: Vec<u32> = it.map(|w| u32::from_str_radix(w, 16).unwrap()).collect();
        assert_eq!(words.len(), cnt);
        let mut r = rhs.split_whitespace();
        let want_exit: u32 = r.next().unwrap().parse().unwrap();
        let want_pc = u32::from_str_radix(r.next().unwrap(), 16).unwrap();
        let want_h = u64::from_str_radix(r.next().unwrap(), 16).unwrap();

        let (cpu, mut bytes) = init_state(seed);
        for (i, w) in words.iter().enumerate() {
            bytes[4 * i..4 * i + 4].copy_from_slice(&w.to_be_bytes());
        }
        let mut mem = FlatMem { base: MEM_BASE, bytes };
        let mut br = TestBridge { code_hi: MEM_BASE + 4 * cnt as u32, log: 0xCBF29CE484222325 };
        let mut m = Machine::new(cpu);
        let exit = match m.run(&mut mem, &mut br, 4000) {
            Ok(_) => 0,
            Err(Fault::Mem(_)) => 1,
            Err(Fault::BadOpcode(..)) => 2,
            Err(Fault::Panic(_)) => 3,
            Err(Fault::Fetch(_)) => 4,
            Err(Fault::Budget) => 5,
            Err(Fault::Unresolved(_)) => 9,
        };
        let c = &m.cpu;
        let mut h = 0xCBF29CE484222325u64;
        for g in c.gpr {
            h = fnv(h, g as u64, 4);
        }
        for f in c.fpr {
            h = fnv(h, f, 8);
        }
        for v in [c.lr, c.ctr, c.xer, c.cr, c.pc] {
            h = fnv(h, v as u64, 4);
        }
        for &b in &mem.bytes {
            h = fnv(h, b as u64, 1);
        }
        h = fnv(h, br.log, 8);
        kinds[want_exit as usize] += 1;
        if (exit, c.pc, h) != (want_exit, want_pc, want_h) {
            bad.push(format!("seed {seed}: rust exit {exit} pc {:08X} hash {h:016X}; C {want_exit} {want_pc:08X} {want_h:016X}", c.pc));
        }
        n += 1;
    }
    assert!(n >= 6000, "{n} cases");
    // the generator must actually exercise every exit kind
    assert!(kinds.iter().all(|&k| k > 0), "exit kinds {kinds:?}");
    assert!(bad.is_empty(), "{} of {n} cases differ:\n{}", bad.len(), bad[..bad.len().min(20)].join("\n"));
}
