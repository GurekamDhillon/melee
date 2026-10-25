//! `tu::gekko_fp::{__frsqrte, __fres}` vs the C (`pc/gameworld/gekko_fp.c`), bit for bit on random bit
//! patterns, widened floats and subnormals. Regenerate with `python rust/melee_runtime/gen/pure/run_pure.py gekko_fp`.
mod pure_common;
use melee_runtime::tu::gekko_fp::{__fres, __frsqrte};
use pure_common::*;

#[test]
fn gekko_fp_matches_c() {
    let mut r = Rng::new();
    let mut out = String::new();
    for k in 0..3000 {
        let x = match k % 5 {
            0 => f64::from_bits(r.u()),
            1 => r.f() as f64,
            2 => {
                let a = r.u() & 0x800F_FFFF_FFFF_FFFF;
                f64::from_bits(a | ((r.u() % 2048) << 52))
            }
            3 => f64::from_bits(r.u() & 0x000F_FFFF_FFFF_FFFF),
            _ => (r.u() % 100000) as f64 / 7.0,
        };
        out += &format!("frsqrte {k} {:016x} {:016x}\n", x.to_bits(), __frsqrte(x).to_bits());
        out += &format!("fres {k} {:016x} {:016x}\n", x.to_bits(), __fres(x).to_bits());
    }
    compare("gekko_fp", &out);
}
