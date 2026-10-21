//! src/tu/ftCo_SquatRv.rs vs the C (gen/cref/tu/ftCo_SquatRv.c: the whole TU, shipping
//! contraction; regenerate with `python rust/melee_fighter/gen/cref/tu/run_tu.py ftCo_SquatRv`).
#![allow(static_mut_refs, non_snake_case)]
mod tu_ftco_harness;
mod tu_s217;
use tu_ftco_harness::*;
use tu_s217::*;
use melee_fighter::tu::ftCo_SquatRv as m;
use m::ext::hook as hk;

static mut ALLNO: bool = false;
macro_rules! q { ($($n:ident = $id:expr),* $(,)?) => { unsafe { $( hk::$n = Some(|g| { t($id); t(pi(g)); if ALLNO { false } else { chk($id) } }); )* } } }

#[test]
fn ftCo_SquatRv_matches_c() {
    s_init();
    unsafe {
        hk::Fighter_ChangeMotionState = Some(cms);
        hk::Mex_Enabled = Some(|n| { t(40); t(n.as_bytes()[0] as i8 as i32 as u32); (SRV[3] & 1) as i32 });
    }
    sv!(hk; ft_8008A2BC = 13, ft_80084F3C = 32, ft_80083F88 = 33);
    sc!(hk; ftAnim_IsFramesRemaining = 12);
    q!(ftCo_800D68C0 = 19, ftCo_Attack100_CheckInput = 17, ftCo_AttackS4_CheckInput = 21,
       ftCo_AttackHi4_CheckInput = 22, ftCo_AttackLw4_CheckInput = 23, ftCo_AttackS3_CheckInput = 24,
       ftCo_AttackHi3_CheckInput = 25, ftCo_AttackLw3_CheckInput = 26, ftCo_Attack1_CheckInput = 27,
       ftCo_80091A4C = 28, ftCo_800DE9D8 = 29, ftCo_Jump_CheckInput = 30, ftCo_Walk_CheckInput = 31);
    let mut c = Check::new(include_str!("data/tu_ftCo_SquatRv.txt"));
    for op in 0..6u32 {
        for k in 0..3000u32 {
            case(op, k);
            s_fill();
            unsafe { ALLNO = SRV[2] & 1 != 0 };
            let g = go();
            let ret = match op {
                0 => m::ftCo_SquatRv_CheckInput(g) as u32,
                1 => { m::ftCo_SquatRv_Enter(g); 0 }
                2 => { m::ftCo_SquatRv_Anim(g); 0 }
                3 => { m::ftCo_SquatRv_IASA(g); 0 }
                4 => { m::ftCo_SquatRv_Phys(g); 0 }
                _ => { m::ftCo_SquatRv_Coll(g); 0 }
            };
            c.line(op, k, ret, state_hash());
        }
    }
    assert_eq!(c.done(), 18000);
}
