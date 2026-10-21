//! src/tu/ftCo_Squat.rs vs the C (gen/cref/tu/ftCo_Squat.c: the whole TU, shipping
//! contraction; regenerate with `python rust/melee_fighter/gen/cref/tu/run_tu.py ftCo_Squat`).
#![allow(static_mut_refs, non_snake_case)]
mod tu_ftco_harness;
mod tu_s217;
use tu_ftco_harness::*;
use tu_s217::*;
use melee_fighter::tu::ftCo_Squat as m;
use m::ext::hook as hk;

static mut ALLNO: bool = false;
macro_rules! q { ($($n:ident = $id:expr),* $(,)?) => { unsafe { $( hk::$n = Some(|g| { t($id); t(pi(g)); if ALLNO { false } else { chk($id) } }); )* } } }

#[test]
fn ftCo_Squat_matches_c() {
    s_init();
    unsafe {
        hk::Fighter_ChangeMotionState = Some(cms);
        hk::ftCommon_8007EFC0 = Some(|f, v| { t(11); t(pi(f)); t(v) });
        hk::mpColl_IsOnPlatform = Some(|c| { t(14); t(pi(c)); chk(14) });
    }
    sv!(hk; ftAnim_8006EBA4 = 10, ftCo_800D638C = 13, ftCo_8009A228 = 15, ft_80084F3C = 32, ft_80083F88 = 33);
    sc!(hk; ftAnim_IsFramesRemaining = 12);
    q!(ftCo_SpecialS_CheckInput = 16, ftCo_Attack100_CheckInput = 17, ftCo_800D6824 = 18, ftCo_800D68C0 = 19,
       ftCo_Catch_CheckInput = 20, ftCo_AttackS4_CheckInput = 21, ftCo_AttackHi4_CheckInput = 22,
       ftCo_AttackLw4_CheckInput = 23, ftCo_AttackS3_CheckInput = 24, ftCo_AttackHi3_CheckInput = 25,
       ftCo_AttackLw3_CheckInput = 26, ftCo_Attack1_CheckInput = 27, ftCo_80091A4C = 28, ftCo_800DE9D8 = 29,
       ftCo_Jump_CheckInput = 30, ftCo_80099F9C = 31);
    let mut c = Check::new(include_str!("data/tu_ftCo_Squat.txt"));
    for op in 0..8u32 {
        for k in 0..2000u32 {
            case(op, k);
            s_fill();
            unsafe {
                ALLNO = SRV[2] & 1 != 0;
                f().mv.co.squat.x0 = (rnd() & 1) as i32;
                f().mv.co.squat.x4 = if rnd() & 1 != 0 { 1.0 } else { rg() };
            }
            let g = go();
            let ret = match op {
                0 => m::ftCo_Squat_CheckInput(g) as u32,
                1 => m::fn_800D5F84(g) as u32,
                2 => m::ftCo_800D5FB0(g) as u32,
                3 => { m::ftCo_Squat_Enter(g); 0 }
                4 => { m::ftCo_Squat_Anim(g); 0 }
                5 => { m::ftCo_Squat_IASA(g); 0 }
                6 => { m::ftCo_Squat_Phys(g); 0 }
                _ => { m::ftCo_Squat_Coll(g); 0 }
            };
            c.line(op, k, ret, state_hash());
        }
    }
    assert_eq!(c.done(), 16000);
}
