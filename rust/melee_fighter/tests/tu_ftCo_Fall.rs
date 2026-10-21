//! src/tu/ftCo_Fall.rs vs the C (gen/cref/tu/ftCo_Fall.c: the whole TU, shipping contraction;
//! regenerate with `python rust/melee_fighter/gen/cref/tu/run_tu.py ftCo_Fall`).
#![allow(static_mut_refs, non_snake_case)]
mod tu_ftco_harness;
#[macro_use]
mod tu_ftco_dfe;
use tu_ftco_harness::*;
use tu_ftco_dfe::*;
use melee_fighter::types::*;
use melee_fighter::tu::ftCo_Fall as m;
use m::ext::hook as hk;

#[test]
fn ftCo_Fall_matches_c() {
    init();
    df_init();
    unsafe {
        hk::ftCo_80090780 = dv!(1);
        hk::ftCo_800C5240 = dchk!(2);
        hk::ftCo_800C5D34 = dv!(3);
        hk::ftLib_GetKind = Some(|g| {
            t(4); t(is_go(g));
            match SRV[5] % 4 { 0 => Ft_Kind_MasterH, 1 => Ft_Kind_CrezyH, _ => f().kind }
        });
        hk::ftMh_MS_389_80151018 = dv!(5);
        hk::ftCh_GrabUnk1_8015BC88 = dv!(6);
        hk::Fighter_ChangeMotionState = Some(|g, ms, fl, a, bb, c, o| {
            t(7); t(is_go(g)); t(ms as u32); t(fl); t(b(a)); t(b(bb)); t(b(c)); t(o.is_null() as u32)
        });
        hk::ftCommon_ClampAirDrift = dfv!(8);
        hk::ftCommon_8007D5D4 = dfv!(9);
        hk::HSD_JObjAnimAll = Some(|j| { t(10); t((j == skel()) as u32); });
        hk::ftAnim_8006FE9C = Some(|p, pt, a, bb| { t(11); t(is_f(p)); t(pt); t(b(a)); t(b(bb)); });
        hk::ftAnim_8006FF74 = Some(|p, pt| { t(12); t(is_f(p)); t(pt); });
        hk::ftAnim_8006EDD0 = Some(|p, s, a, bb| { t(13); t(is_f(p)); t(s as u32); t(b(a)); t(b(bb)); });
        hk::ftCo_SpecialAir_CheckInput = dchk!(14);
        hk::ftCo_80095328 = Some(|g, bp| { t(15); t(is_go(g)); t(bp.is_null() as u32); chk(15) });
        hk::ftCo_800D7100 = dchk!(16);
        hk::ftCo_800C3B10 = dchk!(17);
        hk::ftCo_80099A58 = dchk!(18);
        hk::ftCo_AttackAir_CheckItemThrowInput = dchk!(19);
        hk::ftCo_800D705C = dchk!(20);
        hk::ftCo_800CEE70 = dchk!(21);
        hk::ftPe_8011BA54 = dchk!(22);
        hk::ftCo_800CB870 = dchk!(23);
        hk::ftPe_8011BAD8 = dchk!(24);
        hk::ft_80084DB0 = dv!(25);
        hk::ft_800831CC = Some(|g, a, bb| {
            t(26); t(is_go(g));
            t((a as usize == melee_fighter::tu::ftCo_FallSpecial::ftCo_80096CC8 as usize) as u32);
            t((bb.map(|x| x as usize) == Some(m::ext::ft_80082B1C as usize)) as u32);
        });
    }
    let mut c = Check::new(include_str!("data/tu_ftCo_Fall.txt"));
    for op in 0..11u32 {
        for k in 0..3000u32 {
            case(op, k);
            df_fill();
            let a = match rnd() % 3 { 0 => 0.0, 1 => 1.0, _ => rg() };
            let s0 = (rnd() % 64) as i32;
            let s1 = (rnd() % 64) as i32;
            let s2 = (rnd() % 64) as i32;
            let g = go();
            let ret = match op {
                0 => { m::ftCo_Fall_Enter(g); 0 }
                1 => { m::ftCo_Fall_Enter_YoshiEgg(g); 0 }
                2 => { m::ftCo_Fall_Enter_YoshiEgg_Kirby(g); 0 }
                3 => { m::ftCo_800CC988(g, a); 0 }
                4 => { m::ftCo_Fall_Anim(g); 0 }
                5 => m::ftCo_Fall_IASA_Inner(g) as u32,
                6 => {
                    let p = unsafe {
                        if rnd() & 1 != 0 { core::ptr::addr_of_mut!(f().mv.co.fall.x4) } else { core::ptr::addr_of_mut!(f().mv.co.fallspecial.x4) }
                    };
                    m::ftCo_Fall_Anim_Inner(g, p, s0, s1, s2);
                    0
                }
                7 => { m::ftCo_Fall_IASA(g); 0 }
                8 => { m::ftCo_Fall_Phys(g); 0 }
                9 => { m::ftCo_Fall_Coll(g); 0 }
                _ => m::ftCo_Fall_inline(g) as u32,
            };
            c.line(op, k, ret, df_hash().0);
        }
    }
    assert_eq!(c.done(), 33000);
}
