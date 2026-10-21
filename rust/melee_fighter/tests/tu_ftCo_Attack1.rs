//! src/tu/ftCo_Attack1.rs vs the C (gen/cref/tu/ftCo_Attack1.c: the whole TU, shipping
//! contraction; regenerate with `python rust/melee_fighter/gen/cref/tu/run_tu.py ftCo_Attack1`).
#![allow(static_mut_refs, non_snake_case)]
mod tu_ftco_harness;
#[macro_use]
mod tu_ftco_attack;
use tu_ftco_harness::*;
use tu_ftco_attack::*;
use melee_fighter::tu::ftCo_Attack1 as m;
use melee_fighter::types::*;
use m::ext::hook as hk;

#[test]
fn ftCo_Attack1_matches_c() {
    init();
    af_init();
    unsafe {
        hk::it_8026B30C = Some(|i| { t(1); t(af_item(i)); (srv(0) % 5) as i32 });
        hk::ftCo_800957F4 = Some(|g, ms| { t(2); t(is_go(g)); t(ms as u32) });
        hk::ftCo_Attack_800CCF58 = Some(|g, a| { t(3); t(is_go(g)); t(a as u32) });
        hk::ftCo_Attack_800CDD14 = vrec!(4);
        hk::ft_800892A0 = vrec!(5);
        hk::ft_80089824 = vrec!(6);
        hk::ftGw_Attack11_Enter = vrec!(7);
        hk::ftpickupitem_80094790 = chkrec!(8);
        hk::Fighter_ChangeMotionState = Some(|g, ms, fl, a, bb, c, o| {
            t(9); t(is_go(g)); t(ms as u32); t(fl); t(b(a)); t(b(bb)); t(b(c)); t(o.is_null() as u32)
        });
        hk::ftAnim_8006EBA4 = vrec!(10);
        hk::ftAnim_IsFramesRemaining = chkrec!(11);
        hk::ft_8008A2BC = vrec!(12);
        hk::ftCo_AttackS4_CheckInput = chkrec!(13);
        hk::ftCo_AttackHi4_CheckInput = chkrec!(14);
        hk::ftCo_AttackLw4_CheckInput = chkrec!(15);
        hk::ftCo_AttackS3_CheckInput = chkrec!(16);
        hk::ftCo_AttackHi3_CheckInput = chkrec!(17);
        hk::ftCo_AttackLw3_CheckInput = chkrec!(18);
        hk::ftCo_Attack_800D6A50 = chkrec!(19);
        hk::ftCo_Jump_CheckInput = chkrec!(20);
        hk::ftCo_Dash_CheckInput = chkrec!(21);
        hk::ftCo_800D5FB0 = chkrec!(22);
        hk::ftCo_Turn_CheckInput = chkrec!(23);
        hk::ftCo_Walk_CheckInput = chkrec!(24);
        hk::ftCo_Wait_IASA = vrec!(25);
        hk::ft_80084FA8 = vrec!(26);
        hk::ft_80084104 = vrec!(27);
        // inlines.h ftGetTemplateKind over the Geno_DefineTemplateSkeleton/BaseKind stubs.
        hk::ftGetTemplateKind = Some(|fr| {
            let k = fr.kind as i32;
            if k >= Ft_Kind_Mex0 as i32 {
                t(28); t(k as u32);
                if chk(28) { t(29); t(k as u32); return if sr(2) { Ft_Kind_Mars as i32 } else { k }; }
            }
            k
        });
    }
    let mut c = Check::new(include_str!("data/tu_ftCo_Attack1.txt"));
    for op in 0..19u32 {
        for k in 0..2000u32 {
            case(op, k);
            af_fill();
            let g = go();
            let ret = match op {
                0 => m::ftCo_Attack1_CheckInput(g) as u32,
                1 => { unsafe { m::onPkPc21EC(g) }; 0 }
                2 => { m::decideAttack11(g); 0 }
                3 => {
                    let r = m::getMotionFlags(f());
                    let is = f().x21EC.map(|p| p as usize) == Some(m::onPkPc21EC as usize);
                    r | ((is as u32) << 8)
                }
                4 => { m::checkAttack11(g); 0 }
                5 => { m::ftCo_Attack11_Anim(g); 0 }
                6 => { m::ftCo_Attack11_IASA(g); 0 }
                7 => { m::ftCo_Attack11_Phys(g); 0 }
                8 => { m::ftCo_Attack11_Coll(g); 0 }
                9 => { m::doAttack12Rapid(g); 0 }
                10 => { m::doAttack12Normal(g); 0 }
                11 => { m::doAttack12(g); 0 }
                12 => m::checkAttack12(g) as u32,
                13 => { m::ftCo_Attack12_Anim(g); 0 }
                14 => { m::ftCo_Attack12_IASA(g); 0 }
                15 => { m::doAttack13(g); 0 }
                16 => m::checkAttack13(g) as u32,
                17 => { m::ftCo_Attack13_Anim(g); 0 }
                _ => { m::ftCo_Attack13_IASA(g); 0 }
            };
            c.line(op, k, ret, af_hash());
        }
    }
    assert_eq!(c.done(), 38000);
}
