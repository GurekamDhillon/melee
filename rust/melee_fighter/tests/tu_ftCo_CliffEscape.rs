//! src/tu/ftCo_CliffEscape.rs vs the C (gen/cref/tu/ftCo_CliffEscape.c: the whole TU, shipping contraction;
//! regenerate with `python rust/melee_fighter/gen/cref/tu/run_tu.py ftCo_CliffEscape`).
#![allow(static_mut_refs, non_snake_case)]
mod tu_ftco_harness;
mod tu_ftco_cliff;
use tu_ftco_harness::*;
use tu_ftco_cliff::*;
use melee_fighter::tu::ftCo_CliffEscape as m;
use m::ext::hook as hk;

#[test]
fn ftCo_CliffEscape_matches_c() {
    init();
    unsafe {
        hk::Fighter_ChangeMotionState = Some(s_change_motion);
        hk::ftAnim_8006EBA4 = Some(s_6eba4);
        hk::ftCommon_8007E2F4 = Some(s_7e2f4);
        hk::ftCo_CliffCatch_Phys = Some(s_catch_phys);
        hk::ftCo_CliffClimb_Anim = Some(s_climb_anim);
        hk::ftCo_CliffClimb_Phys = Some(s_climb_phys);
        hk::ftCo_CliffClimb_Coll = Some(s_climb_coll);
    }
    assert_eq!(run_cliff(include_str!("data/tu_ftCo_CliffEscape.txt"), 5, |op| {
        let g = go();
        match op {
            0 => { m::ftCo_8009B040(g); 0 },
            1 => { m::ftCo_CliffEscape_Anim(g); 0 },
            2 => { m::ftCo_CliffEscape_IASA(g); 0 },
            3 => { m::ftCo_CliffEscape_Phys(g); 0 },
            4 => { m::ftCo_CliffEscape_Coll(g); 0 },
            _ => unreachable!(),
        }
    }), 5 * 2000);
}
