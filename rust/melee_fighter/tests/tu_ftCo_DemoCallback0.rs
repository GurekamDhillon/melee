//! src/tu/ftCo_DemoCallback0.rs vs the C (gen/cref/tu/ftCo_DemoCallback0.c, shipping contraction;
//! regenerate with `python rust/melee_fighter/gen/cref/tu/run_tu.py ftCo_DemoCallback0`).
#![allow(static_mut_refs, non_snake_case)]
mod tu_ftco_harness;
use tu_ftco_harness::*;
use melee_fighter::tu::ftCo_DemoCallback0 as m;
use m::ext::hook as hk;

#[test]
fn ftCo_DemoCallback0_matches_c() {
    init();
    unsafe {
        hk::Fighter_ChangeMotionState = Some(|g, ms, fl, a, bb, c, o| {
            t(7); t(is_go(g)); t(ms as u32); t(fl); t(b(a)); t(b(bb)); t(b(c)); t(o.is_null() as u32)
        });
    }
    let mut c = Check::new(include_str!("data/tu_ftCo_DemoCallback0.txt"));
    for k in 0..200u32 {
        case(0, k);
        let fr = f();
        fr.set_x2219_b1((rnd() & 1) as u8);
        fr.set_x2219_b2((rnd() & 1) as u8);
        m::ftCo_800C6150(go());
        let mut h = Hs::new();
        h.m(f().x2219_b1() as u32);
        h.m(f().x2219_b2() as u32);
        c.line(0, k, 0, h.0);
    }
    assert_eq!(c.done(), 200);
}
