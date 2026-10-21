//! src/tu/ftCo_Passive.rs vs the C (gen/cref/tu/ftCo_Passive.c: the whole TU, shipping contraction;
//! regenerate with `python rust/melee_fighter/gen/cref/tu/run_tu.py ftCo_Passive`).
#![allow(static_mut_refs, non_snake_case)]
mod tu_ftco_harness;
mod tu_ftco_misc8;
use tu_ftco_harness::*;
use tu_ftco_misc8::*;
use melee_fighter::tu::ftCo_Passive as m;
use m::ext::hook as hk;

macro_rules! v { ($($n:ident = $id:expr),*) => { unsafe { $( hk::$n = Some(|g| { t($id); t(pi(g)) }); )* } } }

#[test]
fn ftCo_Passive_matches_c() {
    m8_init();
    unsafe {
        hk::ftCommon_8007D7FC = Some(|fr| { t(1); t(pf(fr)) });
        hk::Fighter_ChangeMotionState = Some(|g, ms, fl, a, bb, c, o| { t(2); t(pi(g)); t(ms as u32); t(fl); t(b(a)); t(b(bb)); t(b(c)); t(pi(o)) });
        hk::ft_800881D8 = Some(|fr, s, vo, pa| { t(3); t(pf(fr)); t(s as u32); t(vo as u32); t(pa as u32) });
        hk::ft_PlaySFX = Some(|fr, s, vo, pa| { t(4); t(pf(fr)); t(s as u32); t(vo as u32); t(pa as u32) });
        hk::ftCommon_SetGroundedKnockbackIfLanded = Some(|fr| { t(5); t(pf(fr)) });
        hk::efAsync_Spawn = Some(|g, q, kd, gfx, j| { t(6); t(pi(g)); t(pi(q)); t(kd); t(gfx); t(pi(j)) });
        hk::ftCo_800BFFD0 = Some(|fr, a, bb| { t(7); t(pf(fr)); t(a as u32); t(bb as u32); sr(7) });
        hk::ftAnim_IsFramesRemaining = Some(|g| { t(8); t(pi(g)); sr(8) });
    }
    v!(ft_8008A2BC = 9, ft_80084F3C = 10, ft_80083F88 = 11);
    let mut c = Check::new(include_str!("data/tu_ftCo_Passive.txt"));
    for op in 0..5u32 {
        for k in 0..2000u32 {
            case(op, k);
            m8_fill();
            let g = go();
            match op {
                0 => m::ftCo_800987D0(g),
                1 => m::ftCo_Passive_Anim(g),
                2 => m::ftCo_Passive_IASA(g),
                3 => m::ftCo_Passive_Phys(g),
                _ => m::ftCo_Passive_Coll(g),
            }
            c.line(op, k, 0, state_hash());
        }
    }
    assert_eq!(c.done(), 10000);
}
