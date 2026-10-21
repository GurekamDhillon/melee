//! src/tu/ftCo_PassiveCeil.rs vs the C (gen/cref/tu/ftCo_PassiveCeil.c: the whole TU, shipping
//! contraction; regenerate with `python rust/melee_fighter/gen/cref/tu/run_tu.py ftCo_PassiveCeil`).
#![allow(static_mut_refs, non_snake_case)]
mod tu_ftco_harness;
mod tu_ftco_misc8;
use tu_ftco_harness::*;
use tu_ftco_misc8::*;
use melee_fighter::tu::ftCo_PassiveCeil as m;
use m::ext::hook as hk;

macro_rules! v { ($($n:ident = $id:expr),*) => { unsafe { $( hk::$n = Some(|g| { t($id); t(pi(g)) }); )* } } }
macro_rules! bs { ($($n:ident = $id:expr),*) => { unsafe { $( hk::$n = Some(|g| { t($id); t(pi(g)); sr($id) }); )* } } }

#[test]
fn ftCo_PassiveCeil_matches_c() {
    m8_init();
    unsafe {
        hk::ftKb_SpecialN_800F1F1C = Some(|g, p| { t(3); t(pi(g)); t(b(p.x)); t(b(p.y)); t(b(p.z)); p.y = rw(); p.x = rg() });
        hk::Fighter_ChangeMotionState = Some(|g, ms, fl, a, bb, c, o| { t(4); t(pi(g)); t(ms as u32); t(fl); t(b(a)); t(b(bb)); t(b(c)); t(pi(o)) });
        hk::efAsync_Spawn = Some(|g, q, kd, gfx, j, p| { t(5); t(pi(g)); t(pi(q)); t(kd); t(gfx); t(pi(j)); let p = &*p; t(b(p.x)); t(b(p.y)); t(b(p.z)) });
        hk::ft_800881D8 = Some(|fr, s, vo, pa| { t(7); t(pf(fr)); t(s as u32); t(vo as u32); t(pa as u32) });
        hk::ft_PlaySFX = Some(|fr, s, vo, pa| { t(8); t(pf(fr)); t(s as u32); t(vo as u32); t(pa as u32) });
        hk::ftCommon_8007EBAC = Some(|fr, a, bb| { t(9); t(pf(fr)); t(a); t(bb) });
        hk::ftCo_800BFFD0 = Some(|fr, a, bb| { t(10); t(pf(fr)); t(a as u32); t(bb as u32); sr(10) });
        hk::ft_800831CC = Some(|g, a, bb| { t(14); t(pi(g)); t(a(g, 3) as u32); bb(g) });
        hk::ftCo_80096CC8 = Some(|g, l| { t(15); t(pi(g)); t(l as u32); sr(15) });
    }
    bs!(ftCo_800986B0 = 1, ftAnim_IsFramesRemaining = 11);
    v!(ftCommon_8007E2FC = 2, ftCo_80090574 = 6, ftCo_Fall_Enter = 12, ft_80084DB0 = 13, ft_80082B1C = 16);
    let mut c = Check::new(include_str!("data/tu_ftCo_PassiveCeil.txt"));
    for op in 0..6u32 {
        for k in 0..3000u32 {
            case(op, k);
            m8_fill();
            let g = go();
            let ret = match op {
                0 => m::ftCo_800C23A0(g) as u32,
                1 => { m::ftCo_800C23FC(g); 0 }
                2 => { m::ftCo_PassiveCeil_Anim(g); 0 }
                3 => { m::ftCo_PassiveCeil_IASA(g); 0 }
                4 => { m::ftCo_PassiveCeil_Phys(g); 0 }
                _ => { m::ftCo_PassiveCeil_Coll(g); 0 }
            };
            c.line(op, k, ret, state_hash());
        }
    }
    assert_eq!(c.done(), 18000);
}
