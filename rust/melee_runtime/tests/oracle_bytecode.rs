//! Bit-for-bit check of baselib bytecode.c (`HSD_ByteCodeEval`) and lb_00CE.c `expf` / `powf` /
//! `powi` against the C. Vectors: `tests/data/oracle_bytecode.txt`, written by
//! `oracle/build_bytecode.sh` (C TUs compiled with the shipping build's contraction + gwtool
//! lowering; generated inputs only).
use melee_runtime::bytecode::byte_code_eval;
use melee_runtime::lb_00ce::{expf, powf, powi};
use melee_runtime::random::HsdRand;

const DATA: &str = include_str!("data/oracle_bytecode.txt");

fn hx(s: &str) -> u32 {
    u32::from_str_radix(s, 16).unwrap()
}
fn f(s: &str) -> f32 {
    f32::from_bits(hx(s))
}
/// NaN payloads are not compared (the C's x86 and Rust may pick different quiet NaNs).
fn same(a: u32, b: u32) -> bool {
    a == b || (f32::from_bits(a).is_nan() && f32::from_bits(b).is_nan())
}

#[test]
fn oracle_bytecode() {
    let (mut checks, mut failed) = (0usize, 0usize);
    let mut counts = [0usize; 4];
    for (ln, line) in DATA.lines().enumerate() {
        let t: Vec<&str> = line.split(' ').collect();
        let mut check = |got: u32, want: u32, what: &str| {
            checks += 1;
            if !same(got, want) {
                failed += 1;
                if failed < 20 {
                    eprintln!("line {}: {what}: got {got:08x} want {want:08x}: {line}", ln + 1);
                }
            }
        };
        match t[0] {
            "E" => {
                counts[0] += 1;
                check(expf(f(t[1])).to_bits(), hx(t[2]), "expf");
            }
            "P" => {
                counts[1] += 1;
                check(powf(f(t[1]), f(t[2])).to_bits(), hx(t[3]), "powf");
            }
            "I" => {
                counts[2] += 1;
                check(powi(hx(t[1]) as i32, hx(t[2]) as i32) as u32, hx(t[3]), "powi");
            }
            "B" => {
                counts[3] += 1;
                let code: Vec<u8> =
                    (0..t[2].len()).step_by(2).map(|i| u8::from_str_radix(&t[2][i..i + 2], 16).unwrap()).collect();
                let nargs: usize = t[3].parse().unwrap();
                let args: Vec<f32> = (0..nargs).map(|i| f(t[4 + i])).collect();
                let mut rand = HsdRand { seed: hx(t[1]), ..Default::default() };
                let r = byte_code_eval(Some(&code), &args, nargs as i32, &mut rand);
                check(r.to_bits(), hx(t[4 + nargs]), "result");
                check(rand.seed, hx(t[5 + nargs]), "seed");
            }
            _ => panic!("bad row {line}"),
        }
    }
    eprintln!("oracle_bytecode: {checks} checks ({counts:?} E/P/I/B rows), {failed} failed");
    assert_eq!(failed, 0);
    assert!(counts[3] >= 20000);
}

#[test]
fn null_bytecode_is_zero() {
    let mut rand = HsdRand::default();
    assert_eq!(byte_code_eval(None, &[], 0, &mut rand).to_bits(), 0);
}
