//! src/tu/ftCo_CliffClimb.rs vs the C (gen/cref/tu/ftCo_CliffClimb.c: the whole TU, shipping contraction;
//! regenerate with `python rust/melee_fighter/gen/cref/tu/run_tu.py ftCo_CliffClimb`).
#![allow(static_mut_refs, non_snake_case)]
mod tu_ftco_harness;
mod tu_ftco_cliff;
use tu_ftco_harness::*;
use tu_ftco_cliff::*;
use melee_fighter::tu::ftCo_CliffClimb as m;
use m::ext::hook as hk;

#[test]
fn ftCo_CliffClimb_matches_c() {
    init();
    unsafe {
        hk::ftCo_GetLStickAngle = Some(s_lstick);
        hk::ftCo_GetCStickAngle = Some(s_cstick);
        hk::ftCo_800DF79C = Some(s_df79c);
        hk::ftCo_Fall_Enter = Some(s_fall_enter);
        hk::Fighter_ChangeMotionState = Some(s_change_motion);
        hk::ftAnim_8006EBA4 = Some(s_6eba4);
        hk::ftCommon_8007E2F4 = Some(s_7e2f4);
        hk::ftCo_CliffCatch_Phys = Some(s_catch_phys);
        hk::ftAnim_IsFramesRemaining = Some(s_frames_remaining);
        hk::ftCommon_8007D92C = Some(s_7d92c);
        hk::mpLib_80054ED8 = Some(s_54ed8);
        hk::mpLib_80053ECC_Floor = Some(s_53ecc);
        hk::mpLib_80053DA4_Floor = Some(s_53da4);
        hk::ftCommon_8007D7FC = Some(s_7d7fc);
        hk::ft_80084FA8 = Some(s_84fa8);
        hk::ft_800821DC = Some(s_821dc);
        hk::ftCo_8009AE14 = Some(s_9ae14);
        hk::ftCo_8009EF68 = Some(s_9ef68);
        hk::ft_80084104 = Some(s_84104);
    }
    assert_eq!(run_cliff(include_str!("data/tu_ftCo_CliffClimb.txt"), 7, |op| {
        let g = go();
        match op {
            0 => m::ftCo_8009AA0C(g) as u32,
            1 => m::ftCo_8009AAFC(g, sr(10), f().input.cstick[0].x, unsafe { ANG[0] }) as u32,
            2 => { m::ftCo_8009AB9C(g); 0 },
            3 => { m::ftCo_CliffClimb_Anim(g); 0 },
            4 => { m::ftCo_CliffClimb_IASA(g); 0 },
            5 => { m::ftCo_CliffClimb_Phys(g); 0 },
            6 => { m::ftCo_CliffClimb_Coll(g); 0 },
            _ => unreachable!(),
        }
    }), 7 * 2000);
}
