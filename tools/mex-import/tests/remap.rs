//! Field rewriting (`remap::GuestSpace`) vs the GameCube bytes, with melee_types' own GcDecode as
//! the reference: random GC bytes of a record are decoded into the native struct, the struct is
//! registered at a guest address, and every guest load the rewrite can serve must return the
//! big-endian value at that GC offset. Stores then go to both (guest -> native struct, and the
//! flat GC bytes) and the loads must still agree. Finally a PPC program run by the interpreter
//! reads and writes Fighter fields through the rewrite.

use mex_import::ppc::{Bridge, Cpu, Fault, Machine, Mem};
use mex_import::remap::{GuestSpace, Loc};
use mex_import::translate::{judge, Verdict};
use melee_core::gcdecode::{GcDecode, GcDecoder};
use std::collections::HashSet;

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    fn below(&mut self, n: u32) -> u32 {
        (self.next() % n as u64) as u32
    }
}

const BASE: u32 = 0x8100_0000;

fn be(b: &[u8], o: u32, n: u32) -> u64 {
    b[o as usize..(o + n) as usize].iter().fold(0, |v, &x| v << 8 | x as u64)
}

/// Every byte of [o, o+n) is a plain native scalar/array byte (not bool, padding or pointer).
fn plain(gs: &GuestSpace, rec: &str, o: u32, n: u32) -> bool {
    (0..n).all(|k| matches!(gs.layouts.locate(rec, o + k), Loc::Byte(_)))
}

fn check_record<T: GcDecode>(rec: &'static str, seed: u64) -> (u32, u32) {
    let mut g = Rng(seed);
    let gc_size = T::GC_BYTES;
    let mut gc: Vec<u8> = (0..gc_size).map(|_| g.next() as u8).collect();
    let relocs = HashSet::new();
    let mut d = GcDecoder::new(&gc, &relocs);
    let mut native: Box<T> = Box::new(T::gc_decode(&mut d, 0));
    let (_arena, _issues) = d.finish();
    let mut gs = GuestSpace::new();
    // SAFETY: `native` lives for the whole test; its pointers are null (no relocs).
    unsafe { gs.add_native(Some(BASE), rec, &mut *native as *mut T as *mut u8) }.unwrap();

    let mut checked = 0;
    let mut stores = 0;
    for round in 0..2 {
        for _ in 0..20000 {
            let n = [1u32, 2, 4, 8][g.below(4) as usize];
            let o = g.below(gc_size - n + 1);
            if !plain(&gs, rec, o, n) {
                continue;
            }
            if round == 1 && g.below(2) == 0 {
                let v = g.next() & if n == 8 { u64::MAX } else { (1u64 << (8 * n)) - 1 };
                match n {
                    1 => gs.st8(BASE + o, v as u8),
                    2 => gs.st16(BASE + o, v as u16),
                    4 => gs.st32(BASE + o, v as u32),
                    _ => gs.st64(BASE + o, v),
                }
                .unwrap();
                for k in 0..n {
                    gc[(o + k) as usize] = (v >> (8 * (n - 1 - k))) as u8;
                }
                stores += 1;
                continue;
            }
            let got = match n {
                1 => gs.ld8(BASE + o).unwrap() as u64,
                2 => gs.ld16(BASE + o).unwrap() as u64,
                4 => gs.ld32(BASE + o).unwrap() as u64,
                _ => gs.ld64(BASE + o).unwrap(),
            };
            assert_eq!(got, be(&gc, o, n), "{rec} +{o:#x} size {n} (round {round})");
            checked += 1;
        }
    }
    // bools read 0/1 and pointers (all null here) read handle 0 as a whole word
    for o in 0..gc_size {
        match gs.layouts.locate(rec, o) {
            Loc::Bool(_) => assert_eq!(gs.ld8(BASE + o).unwrap() as u64, (gc[o as usize] != 0) as u64),
            Loc::Ptr(_, 0, _) => {
                assert_eq!(gs.ld32(BASE + o).unwrap(), 0);
                assert!(gs.ld16(BASE + o).is_err(), "partial pointer read must fault");
            }
            _ => {}
        }
    }
    (checked, stores)
}

#[test]
fn rewrite_matches_gc_bytes() {
    use melee_types::*;
    let (c, s) = check_record::<Fighter>("Fighter", 0x1234_5678_9ABC_DEF1);
    assert!(c > 10000 && s > 3000, "{c} loads {s} stores");
    check_record::<Item>("Item", 7);
    check_record::<HSD_JObj>("HSD_JObj", 99);
    check_record::<HSD_GObj>("HSD_GObj", 5);
    check_record::<ftCo_DatAttrs>("ftCo_DatAttrs", 11);
}

struct NoBridge;
impl<M: Mem> Bridge<M> for NoBridge {
    fn in_blob(&self, a: u32) -> bool {
        (0x8000_0000..0x8000_0100).contains(&a)
    }
    fn call(&mut self, _c: &mut Cpu, _m: &mut M, t: u32) -> Result<(), Fault> {
        Err(Fault::Unresolved(t))
    }
}

/// `lfs f1, X(r3); fadds f1, f1, f1; stfs f1, X(r3); lwz r4, Y(r3); addi r4, r4, 1; stw r4, Y(r3); blr`
/// on a native Fighter through the rewrite: the native f32 doubles and the native u32 increments.
#[test]
fn interpreter_on_native_fighter() {
    use melee_types::*;
    let mut fp: Box<Fighter> = {
        let gc = vec![0u8; Fighter::GC_BYTES as usize];
        let relocs = HashSet::new();
        let mut d = GcDecoder::new(&gc, &relocs);
        Box::new(Fighter::gc_decode(&mut d, 0))
    };
    let mut gs = GuestSpace::new();
    gs.trace = Some(vec![]);
    let lay = &gs.layouts;
    // pick a 4-byte plain scalar field for each of the two accesses from the tables
    let mut f32_off = None;
    let mut u32_off = None;
    for o in (0x40..Fighter::GC_BYTES - 4).step_by(4) {
        if judge(lay, "Fighter", o, 4) == Verdict::Rewritable && (0..4).all(|k| matches!(lay.locate("Fighter", o + k), Loc::Byte(_))) {
            if f32_off.is_none() {
                f32_off = Some(o);
            } else if u32_off.is_none() && o > f32_off.unwrap() + 4 {
                u32_off = Some(o);
                break;
            }
        }
    }
    let (fo, uo) = (f32_off.unwrap(), u32_off.unwrap());
    let nat = |o: u32| match lay.locate("Fighter", o + 3) {
        Loc::Byte(b) => b,
        l => panic!("{l:?}"),
    };
    let (fn_, un) = (nat(fo), nat(uo));
    let p = &mut *fp as *mut Fighter as *mut u8;
    unsafe {
        (p.add(fn_) as *mut f32).write_unaligned(1.25);
        (p.add(un) as *mut u32).write_unaligned(41);
    }
    let code: std::vec::Vec<u32> = vec![
        0xC023_0000 | fo,          // lfs f1, fo(r3)
        0xEC21_082A,               // fadds f1, f1, f1
        0xD023_0000 | fo,          // stfs f1, fo(r3)
        0x8083_0000 | uo,          // lwz r4, uo(r3)
        0x3884_0001,               // addi r4, r4, 1
        0x9083_0000 | uo,          // stw r4, uo(r3)
        0x4E80_0020,               // blr
    ];
    gs.add_bytes(0x8000_0000, code.iter().flat_map(|w| w.to_be_bytes()).chain([0; 0x100 - 28]).collect());
    let h = unsafe { gs.add_native(None, "Fighter", p) }.unwrap();
    let mut cpu = Cpu::default();
    cpu.gpr[3] = h;
    cpu.pc = 0x8000_0000;
    let mut m = Machine::new(cpu);
    m.run(&mut gs, &mut NoBridge, 100).unwrap();
    unsafe {
        assert_eq!((p.add(fn_) as *const f32).read_unaligned(), 2.5);
        assert_eq!((p.add(un) as *const u32).read_unaligned(), 42);
    }
    assert_eq!(gs.trace.as_ref().unwrap().len(), 4);
}
