//! src/tu/ftCo_CliffJump.rs vs the C (gen/cref/tu/ftCo_CliffJump.c: the whole TU, shipping contraction;
//! regenerate with `python rust/melee_fighter/gen/cref/tu/run_tu.py ftCo_CliffJump`).
#![allow(static_mut_refs, non_snake_case)]
#[path = "../../melee_core/test_support/fnsym.rs"]
mod fnsym;
mod tu_ftco_harness;
mod tu_ftco_cliff;
use tu_ftco_harness::*;
use tu_ftco_cliff::*;
use melee_fighter::tu::ftCo_CliffJump as m;
use m::ext::hook as hk;

#[test]
fn ftCo_CliffJump_matches_c() {
    init();
    unsafe {
        hk::Fighter_ChangeMotionState = Some(s_change_motion);
        hk::ftAnim_8006EBA4 = Some(s_6eba4);
        hk::ftCommon_8007E2F4 = Some(s_7e2f4);
        hk::ftCo_CliffCatch_Phys = Some(s_catch_phys);
        hk::ftCo_Jump_GetInput = Some(s_jump_input);
        hk::ftAnim_IsFramesRemaining = Some(s_frames_remaining);
        hk::ftCo_CliffClimb_Coll = Some(s_climb_coll);
        hk::ftCo_Fall_Enter = Some(s_fall_enter);
        hk::ft_80084DB0 = Some(s_84db0);
        hk::ft_800835B0 = Some(|g, a, bb| { t(24); t(is_go(g)); t((fnsym::same(a as usize, m::ext::ftCo_80096CC8 as usize)) as u32); t((fnsym::same(bb as usize, m::ext::ft_80082B1C as usize)) as u32) });
    }
    assert_eq!(run_cliff(include_str!("data/tu_ftCo_CliffJump.txt"), 12, |op| {
        let g = go();
        match op {
            0 => m::ftCo_8009B170(g) as u32,
            1 => { m::ftCo_8009B1B8(g); 0 },
            2 => { m::ftCo_CliffJump1_Anim(g); 0 },
            3 => { m::ftCo_CliffJump1_IASA(g); 0 },
            4 => { m::ftCo_CliffJump1_Phys(g); 0 },
            5 => { m::ftCo_CliffJump1_Coll(g); 0 },
            6 => { m::ftCo_8009B2F8(g); 0 },
            7 => { m::ftCo_8009B390(g, unsafe { ANG[1] }); 0 },
            8 => { m::ftCo_CliffJump2_Anim(g); 0 },
            9 => { m::ftCo_CliffJump2_IASA(g); 0 },
            10 => { m::ftCo_CliffJump2_Phys(g); 0 },
            11 => { m::ftCo_CliffJump2_Coll(g); 0 },
            _ => unreachable!(),
        }
    }), 12 * 2000);
}
