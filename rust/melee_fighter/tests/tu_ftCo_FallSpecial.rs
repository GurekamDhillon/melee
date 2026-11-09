//! src/tu/ftCo_FallSpecial.rs vs the C (gen/cref/tu/ftCo_FallSpecial.c: the whole TU, shipping
//! contraction; regenerate with `python rust/melee_fighter/gen/cref/tu/run_tu.py ftCo_FallSpecial`).
#![allow(static_mut_refs, non_snake_case)]
#[path = "../../melee_core/test_support/fnsym.rs"]
mod fnsym;
mod tu_ftco_harness;
#[macro_use]
mod tu_ftco_dfe;
use tu_ftco_harness::*;
use tu_ftco_dfe::*;
use melee_fighter::tu::ftCo_FallSpecial as m;
use m::ext::hook as hk;

#[test]
fn ftCo_FallSpecial_matches_c() {
    init();
    df_init();
    unsafe {
        hk::ftCo_80090780 = dv!(1);
        hk::Fighter_ChangeMotionState = Some(|g, ms, fl, a, bb, c, o| {
            t(2); t(is_go(g)); t(ms as u32); t(fl); t(b(a)); t(b(bb)); t(b(c)); t(o.is_null() as u32)
        });
        hk::ftCommon_8007D60C = dfv!(3);
        hk::ftCommon_UseAllJumps = dfv!(4);
        hk::un_80322598 = Some(|a, y| { t(5); t(a as u32); t(b(y)); 0 });
        hk::ftCo_Fall_Anim_Inner = Some(|g, p, a, bb, c| {
            t(6); t(is_go(g));
            t((p == core::ptr::addr_of_mut!(f().mv.co.fallspecial.x4)) as u32);
            t(a as u32); t(bb as u32); t(c as u32);
            *p = rg();
        });
        hk::ftCo_800CC988 = Some(|g, a| { t(7); t(is_go(g)); t(b(a)); });
        hk::ftCo_800CEE70 = dchk!(8);
        hk::ftCo_800D705C = dchk!(9);
        hk::ftCo_800CB870 = dchk!(10);
        hk::ftCommon_CheckFallFast = Some(|p| { t(11); t(is_f(p)); chk(11) });
        hk::ftCommon_FallFast = dfv!(12);
        hk::ftCommon_Fall = Some(|p, a, bb| { t(13); t(is_f(p)); t(b(a)); t(b(bb)); });
        hk::ftCommon_CalcSelfAccel_AccelToVelClamped = Some(|p, a, bb, c| { t(14); t(is_f(p)); t(bn(a)); t(bn(bb)); t(bn(c)); });
        hk::Geno_FallSpecialColl = Some(|g| { t(15); t(is_go(g)); chk(15) as i32 });
        hk::ft_80083090 = Some(|g, a, bb| {
            t(16); t(is_go(g));
            t((fnsym::same(a as usize, m::ftCo_80096CC8 as usize)) as u32);
            t((bb.map(|x| x as usize) == Some(m::ftCo_80096D28 as usize)) as u32);
        });
        hk::mpLineGetFlags = Some(|l| { t(17); t(l as u32); SRV[6] & 0x1FF });
        hk::ftCo_800D0EC8 = Some(|p| { t(18); t(is_f(p)); rw() });
        hk::ftCo_LandingFallSpecial_Enter = Some(|g, a, bb| { t(19); t(is_go(g)); t(a as u32); t(b(bb)); });
        hk::ft_8008A2BC = dv!(20);
    }
    let mut c = Check::new(include_str!("data/tu_ftCo_FallSpecial.txt"));
    for op in 0..9u32 {
        for k in 0..3000u32 {
            case(op, k);
            df_fill();
            let a1 = (rnd() % 3) as i32;
            let a2 = (rnd() % 3) as i32;
            let ai = (rnd() % 3) as i32;
            let mob = rg();
            let ll = rw();
            let a6 = rg();
            let line = if rnd() & 1 != 0 { -1 } else { (rnd() % 8) as i32 };
            let g = go();
            let ret = match op {
                0 => { m::ftCo_800968C8(g); 0 }
                1 => { m::ftCo_80096900(g, a1, a2, ai, mob, ll); 0 }
                2 => { m::ftCo_800969D8(g, a1, a2, ai, mob, ll, a6); 0 }
                3 => { m::ftCo_FallSpecial_Anim(g); 0 }
                4 => { m::ftCo_FallSpecial_IASA(g); 0 }
                5 => { m::ftCo_FallSpecial_Phys(g); 0 }
                6 => { m::ftCo_FallSpecial_Coll(g); 0 }
                7 => m::ftCo_80096CC8(g, line) as u32,
                _ => { m::ftCo_80096D28(g); 0 }
            };
            c.line(op, k, ret, df_hash().0);
        }
    }
    assert_eq!(c.done(), 27000);
}
