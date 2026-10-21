//! src/tu/ftCo_SquatWait.rs vs the C (gen/cref/tu/ftCo_SquatWait.c: the whole TU, shipping
//! contraction; regenerate with `python rust/melee_fighter/gen/cref/tu/run_tu.py ftCo_SquatWait`).
#![allow(static_mut_refs, non_snake_case)]
mod tu_ftco_harness;
mod tu_s217;
use tu_ftco_harness::*;
use tu_s217::*;
use melee_fighter::tu::ftCo_SquatWait as m;
use m::ext::hook as hk;

static mut ALLNO: bool = false;
macro_rules! q { ($($n:ident = $id:expr),* $(,)?) => { unsafe { $( hk::$n = Some(|g| { t($id); t(pi(g)); if ALLNO { false } else { chk($id) } }); )* } } }

#[test]
fn ftCo_SquatWait_matches_c() {
    s_init();
    unsafe {
        hk::Fighter_ChangeMotionState = Some(cms);
        hk::ftCo_8008A698 = Some(|f| { t(10); t(pi(f)); chk(10) });
        hk::ftData_80085FD4 = Some(|f, ms| { t(11); t(pi(f)); t(ms as u32); (SRV[4] & 3) as usize });
        hk::ftCo_8008A6D8 = Some(|g, a| { t(12); t(pi(g)); t(a as u32) });
        hk::ftCommon_8007EFC0 = Some(|f, v| { t(13); t(pi(f)); t(v) });
        hk::ftCo_8008A7A8 = Some(|g, w| { t(16); t(pi(g)); t(pi(w)) });
    }
    sv!(hk; ftLk_AttackAir_800EB3BC = 14, ftCl_Init_8014919C = 15, ft_80084F3C = 32, ftColl_8007AEE0 = 36, ft_80083F88 = 33);
    q!(ftCo_800D68C0 = 19, ftCo_Attack100_CheckInput = 17, ftCo_AttackS4_CheckInput = 21,
       ftCo_AttackHi4_CheckInput = 22, ftCo_AttackLw4_CheckInput = 23, ftCo_AttackS3_CheckInput = 24,
       ftCo_AttackHi3_CheckInput = 25, ftCo_AttackLw3_CheckInput = 26, ftCo_Attack1_CheckInput = 27,
       ftCo_80091A4C = 28, ftCo_800DE9D8 = 29, ftCo_Jump_CheckInput = 30, ftCo_80099F9C = 31,
       ftCo_Dash_CheckInput = 34, ftCo_SquatRv_CheckInput = 35);
    let mut c = Check::new(include_str!("data/tu_ftCo_SquatWait.txt"));
    for op in 0..7u32 {
        for k in 0..2000u32 {
            case(op, k);
            s_fill();
            unsafe { ALLNO = SRV[2] & 1 != 0 };
            let g = go();
            let ret = match op {
                0 => m::ftCo_SquatWait_CheckInput(g) as u32,
                1 => { m::fn_800D62C4(g); 0 }
                2 => { m::ftCo_800D638C(g); 0 }
                3 => { m::ftCo_SquatWait_Anim(g); 0 }
                4 => { m::ftCo_SquatWait_IASA(g); 0 }
                5 => { m::ftCo_SquatWait_Phys(g); 0 }
                _ => { m::ftCo_SquatWait_Coll(g); 0 }
            };
            c.line(op, k, ret, state_hash());
        }
    }
    assert_eq!(c.done(), 14000);
}
