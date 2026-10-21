//! src/tu/ftCo_CliffAttack.rs vs the C (gen/cref/tu/ftCo_CliffAttack.c: the whole TU, shipping contraction;
//! regenerate with `python rust/melee_fighter/gen/cref/tu/run_tu.py ftCo_CliffAttack`).
#![allow(static_mut_refs, non_snake_case)]
mod tu_ftco_harness;
mod tu_ftco_cliff;
use tu_ftco_harness::*;
use tu_ftco_cliff::*;
use melee_fighter::tu::ftCo_CliffAttack as m;
use m::ext::hook as hk;

#[test]
fn ftCo_CliffAttack_matches_c() {
    init();
    unsafe {
        hk::Fighter_ChangeMotionState = Some(s_change_motion);
        hk::ftAnim_8006EBA4 = Some(s_6eba4);
        hk::ftCommon_8007E2F4 = Some(s_7e2f4);
        hk::ftCo_CliffCatch_Phys = Some(s_catch_phys);
        hk::ftCommon_8007D7FC = Some(s_7d7fc);
        hk::ftCo_800DF6F8 = Some(s_df6f8);
        hk::ftCo_800DF72C = Some(s_df72c);
        hk::ftCo_CliffClimb_Anim = Some(s_climb_anim);
        hk::ftCo_CliffClimb_Phys = Some(s_climb_phys);
        hk::ftCo_CliffClimb_Coll = Some(s_climb_coll);
        hk::ftCo_8009B040 = Some(s_9b040);
    }
    assert_eq!(run_cliff(include_str!("data/tu_ftCo_CliffAttack.txt"), 8, |op| {
        let g = go();
        match op {
            0 => { m::ftCo_8009AE14(g); 0 },
            1 => m::ftCo_8009AE38(g) as u32,
            2 => { m::ftCo_8009AEA4(g); 0 },
            3 => { m::ftCo_CliffAttack_Anim(g); 0 },
            4 => { m::ftCo_CliffAttack_IASA(g); 0 },
            5 => { m::ftCo_CliffAttack_Phys(g); 0 },
            6 => { m::ftCo_CliffAttack_Coll(g); 0 },
            7 => m::ftCo_8009AFD4(g) as u32,
            _ => unreachable!(),
        }
    }), 8 * 2000);
}
