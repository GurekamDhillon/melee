//! ftCo_GetLStickAngle (src/ftcommon_c.rs) vs the C (gen/cref/tu/ftcommon_stick.c; regenerate with
//! `python rust/melee_fighter/gen/cref/tu/run_tu.py ftcommon_stick`).
#![allow(static_mut_refs, non_snake_case)]
mod tu_ftco_harness;
use melee_fighter::ftcommon_c as m;
use tu_ftco_harness::*;

#[test]
fn ft_co_get_l_stick_angle_matches_c() {
    init();
    let mut lines = include_str!("data/tu_ftcommon_stick.txt").lines();
    for k in 0..20000u32 {
        case(0, k);
        let f = f();
        f.input.lstick[0].x = if rnd() & 1 != 0 { rw() } else { rg() };
        f.input.lstick[0].y = if rnd() & 1 != 0 { rw() } else { rg() };
        let r = m::ftCo_GetLStickAngle(f);
        assert_eq!(format!("{} {:08X} {:08X}", k, b(r), unsafe { TR }), lines.next().unwrap(), "case {k}");
    }
    assert!(lines.next().is_none());
}
