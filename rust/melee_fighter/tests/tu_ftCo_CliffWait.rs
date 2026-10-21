//! src/tu/ftCo_CliffWait.rs vs the C (gen/cref/tu/ftCo_CliffWait.c: the whole TU, shipping contraction;
//! regenerate with `python rust/melee_fighter/gen/cref/tu/run_tu.py ftCo_CliffWait`).
#![allow(static_mut_refs, non_snake_case)]
mod tu_ftco_harness;
mod tu_ftco_cliff;
use tu_ftco_harness::*;
use tu_ftco_cliff::*;
use melee_fighter::tu::ftCo_CliffWait as m;
use m::ext::hook as hk;

#[test]
fn ftCo_CliffWait_matches_c() {
    init();
    unsafe {
        hk::Fighter_ChangeMotionState = Some(s_change_motion);
        hk::ftCommon_8007E2F4 = Some(s_7e2f4);
        hk::ftColl_8007B760 = Some(s_7b760);
        hk::ftCo_8009AE38 = Some(s_9ae38);
        hk::ftCo_8009AFD4 = Some(s_9afd4);
        hk::ftCo_8009B170 = Some(s_9b170);
        hk::ftCo_8009AA0C = Some(s_9aa0c);
        hk::ftCo_CliffCatch_Phys = Some(s_catch_phys);
        hk::ftCo_CliffCatch_Coll = Some(s_catch_coll);
        hk::ftCo_80090780 = Some(s_90780);
    }
    assert_eq!(run_cliff(include_str!("data/tu_ftCo_CliffWait.txt"), 6, |op| {
        let g = go();
        match op {
            0 => { m::ftCo_8009A804(g); 0 },
            1 => { m::ftCo_CliffWait_Anim(g); 0 },
            2 => { m::ftCo_CliffWait_IASA(g); 0 },
            3 => { m::ftCo_CliffWait_Phys(g); 0 },
            4 => { m::ftCo_CliffWait_Coll(g); 0 },
            5 => m::ftCo_8009A9AC(g) as u32,
            _ => unreachable!(),
        }
    }), 6 * 2000);
}
