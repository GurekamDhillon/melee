//! src/tu/ftCo_Rebound.rs vs the C (gen/cref/tu/ftCo_Rebound.c: the whole TU, shipping contraction;
//! regenerate with `python rust/melee_fighter/gen/cref/tu/run_tu.py ftCo_Rebound`).
#![allow(static_mut_refs, non_snake_case)]
mod tu_ftco_harness;
mod tu_ftco_misc8;
use tu_ftco_harness::*;
use tu_ftco_misc8::*;
use melee_fighter::tu::ftCo_Rebound as m;
use m::ext::hook as hk;

macro_rules! v { ($($n:ident = $id:expr),*) => { unsafe { $( hk::$n = Some(|g| { t($id); t(pi(g)) }); )* } } }

#[test]
fn ftCo_Rebound_matches_c() {
    m8_init();
    unsafe {
        hk::Fighter_ChangeMotionState = Some(|g, ms, fl, a, bb, c, o| { t(1); t(pi(g)); t(ms as u32); t(fl); t(b(a)); t(b(bb)); t(b(c)); t(pi(o)) });
        hk::ftCommon_800804A0 = Some(|fr, x| { t(2); t(pf(fr)); t(b(x)) });
        hk::ftAnim_IsFramesRemaining = Some(|g| { t(3); t(pi(g)); sr(3) });
    }
    v!(ft_8008A2BC = 4, ft_80084F3C = 5, ft_80083F88 = 6);
    let mut c = Check::new(include_str!("data/tu_ftCo_Rebound.txt"));
    for op in 0..7u32 {
        for k in 0..3000u32 {
            case(op, k);
            m8_fill();
            let g = go();
            match op {
                0 => m::ftCo_80099D9C(g),
                1 => m::ftCo_ReboundStop_Anim(g),
                2 => m::ftCo_80099E44(g),
                3 => m::ftCo_Rebound_Anim(g),
                4 => m::ftCo_Rebound_IASA(g),
                5 => m::ftCo_Rebound_Phys(g),
                _ => m::ftCo_Rebound_Coll(g),
            }
            c.line(op, k, 0, state_hash());
        }
    }
    assert_eq!(c.done(), 21000);
}
