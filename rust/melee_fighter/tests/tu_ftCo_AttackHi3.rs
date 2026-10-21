//! src/tu/ftCo_AttackHi3.rs vs the C (gen/cref/tu/ftCo_AttackHi3.c: the whole TU, shipping
//! contraction; regenerate with `python rust/melee_fighter/gen/cref/tu/run_tu.py ftCo_AttackHi3`).
#![allow(static_mut_refs, non_snake_case)]
mod tu_ftco_harness;
#[macro_use]
mod tu_ftco_attack;
use tu_ftco_harness::*;
use tu_ftco_attack::*;
use melee_fighter::tu::ftCo_AttackHi3 as m;
use melee_fighter::types::*;
use m::ext::hook as hk;

fn isf(fr: &Fighter) -> u32 { (fr as *const Fighter == unsafe { F } as *const Fighter) as u32 }

#[test]
fn ftCo_AttackHi3_matches_c() {
    init();
    af_init();
    unsafe {
        hk::ftCo_GetLStickAngle = Some(|fr| { t(1); t(isf(fr)); angle_of(srv(11)) });
        hk::ftCo_80094E54 = Some(|fr| { t(2); t(isf(fr)); chk(2) });
        hk::ftCo_800957F4 = Some(|g, ms| { t(3); t(is_go(g)); t(ms as u32) });
        hk::Fighter_ChangeMotionState = Some(|g, ms, fl, a, bb, c, o| {
            t(4); t(is_go(g)); t(ms as u32); t(fl); t(b(a)); t(b(bb)); t(b(c)); t(o.is_null() as u32)
        });
        hk::ftAnim_8006EBA4 = vrec!(5);
        hk::ftAnim_IsFramesRemaining = chkrec!(6);
        hk::ft_8008A2BC = vrec!(7);
        hk::ftCo_Wait_IASA = vrec!(8);
        hk::ft_80084F3C = vrec!(9);
        hk::ft_80084104 = vrec!(10);
    }
    let mut c = Check::new(include_str!("data/tu_ftCo_AttackHi3.txt"));
    for op in 0..6u32 {
        for k in 0..2000u32 {
            case(op, k);
            af_fill();
            let g = go();
            let ret = match op {
                0 => m::ftCo_AttackHi3_CheckInput(g) as u32,
                1 => { m::doEnter(g); 0 }
                2 => { m::ftCo_AttackHi3_Anim(g); 0 }
                3 => { m::ftCo_AttackHi3_IASA(g); 0 }
                4 => { m::ftCo_AttackHi3_Phys(g); 0 }
                _ => { m::ftCo_AttackHi3_Coll(g); 0 }
            };
            c.line(op, k, ret, af_hash());
        }
    }
    assert_eq!(c.done(), 12000);
}
