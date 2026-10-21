//! src/tu/ftCo_PassiveStand.rs vs the C (gen/cref/tu/ftCo_PassiveStand.c: the whole TU, shipping
//! contraction; regenerate with `python rust/melee_fighter/gen/cref/tu/run_tu.py ftCo_PassiveStand`).
#![allow(static_mut_refs, non_snake_case)]
mod tu_ftco_harness;
mod tu_ftco_misc8;
use tu_ftco_harness::*;
use tu_ftco_misc8::*;
use melee_fighter::tu::ftCo_PassiveStand as m;
use m::ext::hook as hk;

macro_rules! v { ($($n:ident = $id:expr),*) => { unsafe { $( hk::$n = Some(|g| { t($id); t(pi(g)) }); )* } } }
macro_rules! bs { ($($n:ident = $id:expr),*) => { unsafe { $( hk::$n = Some(|g| { t($id); t(pi(g)); sr($id) }); )* } } }

#[test]
fn ftCo_PassiveStand_matches_c() {
    m8_init();
    unsafe {
        hk::ftCommon_8007D7FC = Some(|fr| { t(2); t(pf(fr)) });
        hk::Fighter_ChangeMotionState = Some(|g, ms, fl, a, bb, c, o| { t(3); t(pi(g)); t(ms as u32); t(fl); t(b(a)); t(b(bb)); t(b(c)); t(pi(o)) });
        hk::ft_800881D8 = Some(|fr, s, vo, pa| { t(4); t(pf(fr)); t(s as u32); t(vo as u32); t(pa as u32) });
        hk::ft_PlaySFX = Some(|fr, s, vo, pa| { t(5); t(pf(fr)); t(s as u32); t(vo as u32); t(pa as u32) });
        hk::efAsync_Spawn = Some(|g, q, kd, gfx, j| { t(6); t(pi(g)); t(pi(q)); t(kd); t(gfx); t(pi(j)) });
    }
    bs!(ftCo_800986B0 = 1, ftAnim_IsFramesRemaining = 7);
    v!(ft_8008A2BC = 8, ft_80084FA8 = 9, ft_80084104 = 10);
    let mut c = Check::new(include_str!("data/tu_ftCo_PassiveStand.txt"));
    for op in 0..6u32 {
        for k in 0..3000u32 {
            case(op, k);
            m8_fill();
            let g = go();
            let ret = match op {
                0 => m::ftCo_80098928(g) as u32,
                1 => { m::ftCo_800989D4(g, (rnd() % 400) as i32); 0 }
                2 => { m::ftCo_PassiveStand_Anim(g); 0 }
                3 => { m::ftCo_PassiveStand_IASA(g); 0 }
                4 => { m::ftCo_PassiveStand_Phys(g); 0 }
                _ => { m::ftCo_PassiveStand_Coll(g); 0 }
            };
            c.line(op, k, ret, state_hash());
        }
    }
    assert_eq!(c.done(), 18000);
}
