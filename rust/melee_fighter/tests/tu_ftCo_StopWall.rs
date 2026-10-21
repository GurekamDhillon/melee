//! src/tu/ftCo_StopWall.rs vs the C (gen/cref/tu/ftCo_StopWall.c: the whole TU, shipping
//! contraction; regenerate with `python rust/melee_fighter/gen/cref/tu/run_tu.py ftCo_StopWall`).
#![allow(static_mut_refs, non_snake_case)]
mod tu_ftco_harness;
mod tu_s217;
use tu_ftco_harness::*;
use tu_s217::*;
use melee_fighter::tu::ftCo_StopWall as m;
use m::ext::hook as hk;

#[test]
fn ftCo_StopWall_matches_c() {
    s_init();
    unsafe {
        hk::Fighter_ChangeMotionState = Some(cms);
        hk::ftKb_SpecialN_800F1F1C = Some(|g, v| {
            t(10); t(pi(g)); t(b(v.x)); t(b(v.y)); t(b(v.z));
            if chk(10) { v.x = v.x + 1.0; }
        });
    }
    sv!(hk; ft_800843FC = 11, ftCommon_8007E2FC = 12, ft_8008A2BC = 14);
    sc!(hk; ftAnim_IsFramesRemaining = 13);
    let mut c = Check::new(include_str!("data/tu_ftCo_StopWall.txt"));
    for op in 0..6u32 {
        for k in 0..3000u32 {
            case(op, k);
            s_fill();
            let g = go();
            let ret = match op {
                0 => m::ftCo_8009EDA4(g) as u32,
                1 => { m::ftCo_8009EE30(g); 0 }
                2 => { m::ftCo_StopWall_Anim(g); 0 }
                3 => { m::ftCo_StopWall_IASA(g); 0 }
                4 => { m::ftCo_StopWall_Phys(g); 0 }
                _ => { m::ftCo_StopWall_Coll(g); 0 }
            };
            c.line(op, k, ret, state_hash());
        }
    }
    assert_eq!(c.done(), 18000);
}
