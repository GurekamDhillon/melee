//! src/tu/ftCo_Ottotto.rs vs the C (gen/cref/tu/ftCo_Ottotto.c: the whole TU, shipping contraction;
//! regenerate with `python rust/melee_fighter/gen/cref/tu/run_tu.py ftCo_Ottotto`).
#![allow(static_mut_refs, non_snake_case)]
mod tu_ftco_harness;
mod tu_ftco_misc8;
use tu_ftco_harness::*;
use tu_ftco_misc8::*;
use melee_fighter::tu::ftCo_Ottotto as m;
use m::ext::hook as hk;

macro_rules! v { ($($n:ident = $id:expr),*) => { unsafe { $( hk::$n = Some(|g| { t($id); t(pi(g)) }); )* } } }
macro_rules! bs { ($($n:ident = $id:expr),*) => { unsafe { $( hk::$n = Some(|g| { t($id); t(pi(g)); sr($id) }); )* } } }

#[test]
fn ftCo_Ottotto_matches_c() {
    m8_init();
    unsafe {
        hk::Fighter_ChangeMotionState = Some(|g, ms, fl, a, bb, c, o| { t(1); t(pi(g)); t(ms as u32); t(fl); t(b(a)); t(b(bb)); t(b(c)); t(pi(o)) });
        hk::mpFloorGetRight = Some(|l, p| { t(5); t(l as u32); p.x = rw(); p.y = rg(); p.z = rg() });
        hk::mpFloorGetLeft = Some(|l, p| { t(6); t(l as u32); p.x = rw(); p.y = rg(); p.z = rg() });
        hk::ft_80088770 = Some(|fr| { t(8); t(pf(fr)) });
        hk::ft_800887CC = Some(|fr| { t(9); t(pf(fr)) });
        hk::ft_80088328 = Some(|fr, s, vo, pa| { t(10); t(pf(fr)); t(s as u32); t(vo as u32); t(pa as u32) });
    }
    bs!(ftAnim_IsFramesRemaining = 2, ft_800827A0 = 3);
    v!(ftCo_Fall_Enter = 4, ft_8008A2BC = 7);
    bs!(ftCo_SpecialS_CheckInput = 11, ftCo_Attack100_CheckInput = 12, ftCo_800D6824 = 13, ftCo_800D68C0 = 14,
        ftCo_Catch_CheckInput = 15, ftCo_AttackS4_CheckInput = 16, ftCo_AttackHi4_CheckInput = 17,
        ftCo_AttackLw4_CheckInput = 18, ftCo_AttackS3_CheckInput = 19, ftCo_AttackHi3_CheckInput = 20,
        ftCo_AttackLw3_CheckInput = 21, ftCo_Attack1_CheckInput = 22, ftCo_80091A4C = 23, ftCo_800DE9D8 = 24,
        ftCo_Jump_CheckInput = 25, ftCo_Dash_CheckInput = 26, ftCo_800D5FB0 = 27, ftCo_Turn_CheckInput = 28,
        ftCo_Walk_CheckInput_Ottotto = 29);
    let mut c = Check::new(include_str!("data/tu_ftCo_Ottotto.txt"));
    for op in 0..12u32 {
        for k in 0..2000u32 {
            case(op, k);
            m8_fill();
            let g = go();
            let ret = match op {
                0 => m::ftCo_8009A3C8(g) as u32,
                1 => { m::ftCo_8009A410(g); 0 }
                2 => { m::ftCo_Ottotto_Anim(g); 0 }
                3 => { m::ftCo_Ottotto_IASA(g); 0 }
                4 => { m::ftCo_Ottotto_Phys(g); 0 }
                5 | 11 => { m::ftCo_Ottotto_Coll(g); 0 }
                6 => { m::ftCo_8009A6B8(g); 0 }
                7 => { m::ftCo_OttottoWait_Anim(g); 0 }
                8 => { m::ftCo_OttottoWait_IASA(g); 0 }
                9 => { m::ftCo_OttottoWait_Phys(g); 0 }
                _ => { m::ftCo_OttottoWait_Coll(g); 0 }
            };
            c.line(op, k, ret, state_hash());
        }
    }
    assert_eq!(c.done(), 24000);
}
