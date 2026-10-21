//! src/tu/ftCo_PassiveWall.rs vs the C (gen/cref/tu/ftCo_PassiveWall.c: the whole TU, shipping
//! contraction; regenerate with `python rust/melee_fighter/gen/cref/tu/run_tu.py ftCo_PassiveWall`).
#![allow(static_mut_refs, non_snake_case)]
mod tu_ftco_harness;
mod tu_ftco_misc8;
use tu_ftco_harness::*;
use tu_ftco_misc8::*;
use melee_fighter::tu::ftCo_PassiveWall as m;
use m::ext::hook as hk;

macro_rules! v { ($($n:ident = $id:expr),*) => { unsafe { $( hk::$n = Some(|g| { t($id); t(pi(g)) }); )* } } }
macro_rules! bs { ($($n:ident = $id:expr),*) => { unsafe { $( hk::$n = Some(|g| { t($id); t(pi(g)); sr($id) }); )* } } }

#[test]
fn ftCo_PassiveWall_matches_c() {
    m8_init();
    unsafe {
        hk::ftKb_SpecialN_800F1F1C = Some(|g, p| { t(3); t(pi(g)); t(b(p.x)); t(b(p.y)); t(b(p.z)); p.x = rw(); p.y = rg() });
        hk::Fighter_ChangeMotionState = Some(|g, ms, fl, a, bb, c, o| { t(4); t(pi(g)); t(ms as u32); t(fl); t(b(a)); t(b(bb)); t(b(c)); t(pi(o)) });
        hk::efAsync_Spawn = Some(|g, q, kd, gfx, j, p| { t(5); t(pi(g)); t(pi(q)); t(kd); t(gfx); t(pi(j)); let p = &*p; t(b(p.x)); t(b(p.y)); t(b(p.z)) });
        hk::ft_800881D8 = Some(|fr, s, vo, pa| { t(7); t(pf(fr)); t(s as u32); t(vo as u32); t(pa as u32) });
        hk::ftCommon_8007EBAC = Some(|fr, a, bb| { t(8); t(pf(fr)); t(a); t(bb) });
        hk::ft_PlaySFX = Some(|fr, s, vo, pa| { t(9); t(pf(fr)); t(s as u32); t(vo as u32); t(pa as u32) });
        hk::ftCo_800BFFD0 = Some(|fr, a, bb| { t(10); t(pf(fr)); t(a as u32); t(bb as u32); sr(10) });
        hk::ftColl_8007B760 = Some(|g, x| { t(11); t(pi(g)); t(x as u32) });
        hk::ftAnim_SetAnimRate = Some(|g, r| { t(12); t(pi(g)); t(b(r)) });
        hk::powf = Some(|x, y| { t(13); t(b(x)); t(b(y)); rw() });
        hk::ftCo_80095328 = Some(|g, p| { t(17); t(pi(g)); t(p.is_null() as u32); sr(17) });
        hk::ftCommon_CheckFallFast = Some(|fr| { t(25); t(pf(fr)); fr.set_fall_fast(sr(25) as u8); sr(41) });
        hk::ftCommon_FallFast = Some(|fr| { t(26); t(pf(fr)) });
        hk::ftCommon_Fall = Some(|fr, a, bb| { t(27); t(pf(fr)); t(b(a)); t(b(bb)) });
        hk::ftCommon_CalcSelfAccel_AccelToVelClamped = Some(|fr, a, bb, c| { t(28); t(pf(fr)); t(b(a)); t(b(bb)); t(b(c)) });
        hk::ft_80083318 = Some(|g, a, bb| { t(29); t(pi(g)); t(a(g, 5) as u32); bb(g) });
        hk::ft_800831CC = Some(|g, a, bb| { t(30); t(pi(g)); t(a(g, 3) as u32); bb(g) });
        hk::ftCo_80096CC8 = Some(|g, l| { t(31); t(pi(g)); t(l as u32); sr(31) });
    }
    bs!(ftCo_800986B0 = 1, ft_80081F2C = 6, ftAnim_IsFramesRemaining = 14, ftCo_SpecialAir_CheckInput = 16,
        ftCo_800D7100 = 18, ftCo_800C3B10 = 19, ftCo_80099A58 = 20, ftCo_AttackAir_CheckItemThrowInput = 21,
        ftCo_800D705C = 22, ftCo_800CB870 = 23, ftCo_800CEE70 = 24);
    v!(ftCommon_8007E2FC = 2, ftCo_Fall_Enter = 15, ft_80082B1C = 32);
    let mut c = Check::new(include_str!("data/tu_ftCo_PassiveWall.txt"));
    for op in 0..8u32 {
        for k in 0..4000u32 {
            case(op, k);
            m8_fill();
            let g = go();
            let ret = match op {
                0 => m::ftCo_800C1D38(g) as u32,
                1 => m::ftCo_800C1E0C(f()) as u32,
                2 => {
                    let ms = (rnd() % 400) as i32;
                    let tm = (rnd() % 3) as i32;
                    let ve = (rnd() % 4) as i32;
                    let fd = rg();
                    m::ftCo_800C1E64(g, ms, tm, ve, fd);
                    0
                }
                3 | 7 => { m::ftCo_PassiveWall_Anim(g); 0 }
                4 => { m::ftCo_PassiveWall_IASA(g); 0 }
                5 => { m::ftCo_PassiveWall_Phys(g); 0 }
                _ => { m::ftCo_PassiveWall_Coll(g); 0 }
            };
            c.line(op, k, ret, state_hash());
        }
    }
    assert_eq!(c.done(), 32000);
}
