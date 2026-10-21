//! src/tu/ftCo_StopCeil.rs vs the C (gen/cref/tu/ftCo_StopCeil.c: the whole TU, shipping
//! contraction; regenerate with `python rust/melee_fighter/gen/cref/tu/run_tu.py ftCo_StopCeil`).
#![allow(static_mut_refs, non_snake_case)]
mod tu_ftco_harness;
mod tu_s217;
use tu_ftco_harness::*;
use tu_s217::*;
use melee_fighter::tu::ftCo_StopCeil as m;
use m::ext::hook as hk;

#[test]
fn ftCo_StopCeil_matches_c() {
    s_init();
    unsafe {
        hk::Fighter_ChangeMotionState = Some(cms);
        hk::ftKb_SpecialN_800F1F1C = Some(|g, v| {
            t(10); t(pi(g)); t(b(v.x)); t(b(v.y)); t(b(v.z));
            if chk(10) { v.y = v.y + 1.0; }
        });
        hk::ft_80082D40 = Some(|g, y| { t(11); t(pi(g)); t(b(y)) });
        hk::ft_80083464 = Some(|g, _a, _b| { t(14); t(pi(g)); t(1); t(1) });
    }
    sv!(hk; ftCo_Fall_Enter = 13);
    sc!(hk; ftAnim_IsFramesRemaining = 12);
    let mut c = Check::new(include_str!("data/tu_ftCo_StopCeil.txt"));
    for op in 0..6u32 {
        for k in 0..3000u32 {
            case(op, k);
            s_fill();
            let g = go();
            let ret = match op {
                0 => m::ftCo_8009EF68(g) as u32,
                1 => { m::ftCo_8009EFA4(g); 0 }
                2 => { m::ftCo_StopCeil_Anim(g); 0 }
                3 => { m::ftCo_StopCeil_IASA(g); 0 }
                4 => { m::ftCo_StopCeil_Phys(g); 0 }
                _ => { m::ftCo_StopCeil_Coll(g); 0 }
            };
            c.line(op, k, ret, state_hash());
        }
    }
    assert_eq!(c.done(), 18000);
}
