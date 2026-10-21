//! src/tu/ftCo_AttackDash.rs vs the C (gen/cref/tu/ftCo_AttackDash.c: the whole TU, shipping
//! contraction; regenerate with `python rust/melee_fighter/gen/cref/tu/run_tu.py ftCo_AttackDash`).
#![allow(static_mut_refs, non_snake_case)]
mod tu_ftco_harness;
#[macro_use]
mod tu_ftco_attack;
use tu_ftco_harness::*;
use tu_ftco_attack::*;
use melee_fighter::tu::ftCo_AttackDash as m;
use m::ext::hook as hk;

fn mv0() -> *mut u32 { &mut f().mv as *mut _ as *mut u32 }

#[test]
fn ftCo_AttackDash_matches_c() {
    init();
    af_init();
    unsafe {
        hk::it_8026B30C = Some(|i| { t(1); t(af_item(i)); (srv(0) % 5) as i32 });
        hk::ftCo_800957F4 = Some(|g, ms| { t(2); t(is_go(g)); t(ms as u32) });
        hk::ftCo_Attack_800CCF58 = Some(|g, a| { t(3); t(is_go(g)); t(a as u32) });
        hk::ftKb_MexKirbyLike = Some(|k| { t(4); t(k as u32); chk(4) as i32 });
        hk::ftKb_SpecialN_800F1F68 = vrec!(5);
        hk::Fighter_ChangeMotionState = Some(|g, ms, fl, a, bb, c, o| {
            t(6); t(is_go(g)); t(ms as u32); t(fl); t(b(a)); t(b(bb)); t(b(c)); t(o.is_null() as u32)
        });
        hk::ftAnim_8006EBA4 = vrec!(7);
        hk::ftAnim_IsFramesRemaining = chkrec!(8);
        hk::ft_8008A2BC = vrec!(9);
        hk::ftCo_800D8AE0 = chkrec!(10);
        hk::ftCo_Wait_IASA = vrec!(11);
        hk::ft_80085030 = Some(|g, a, bb| { t(12); t(is_go(g)); t(b(a)); t(b(bb)) });
        hk::ft_80084104 = vrec!(13);
    }
    let mut c = Check::new(include_str!("data/tu_ftCo_AttackDash.txt"));
    for op in 0..8u32 {
        for k in 0..2000u32 {
            case(op, k);
            af_fill();
            unsafe { *mv0() = rnd(); }
            let g = go();
            let ret = match op {
                0 => m::ftCo_AttackDash_CheckInput(g) as u32,
                1 => { m::decideFighter(g); 0 }
                2 => { m::doEnter(g); 0 }
                3 => { m::ftCo_AttackDash_Anim(g); 0 }
                4 => { m::ftCo_AttackDash_SetMv0(g); 0 }
                5 => { m::ftCo_AttackDash_IASA(g); 0 }
                6 => { m::ftCo_AttackDash_Phys(g); 0 }
                _ => { m::ftCo_AttackDash_Coll(g); 0 }
            } ^ unsafe { *mv0() };
            c.line(op, k, ret, af_hash());
        }
    }
    assert_eq!(c.done(), 16000);
}
