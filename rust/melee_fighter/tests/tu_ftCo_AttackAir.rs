//! src/tu/ftCo_AttackAir.rs vs the C (gen/cref/tu/ftCo_AttackAir.c: the whole TU, shipping
//! contraction; regenerate with `python rust/melee_fighter/gen/cref/tu/run_tu.py ftCo_AttackAir`).
#![allow(static_mut_refs, non_snake_case)]
mod tu_ftco_harness;
#[macro_use]
mod tu_ftco_attack;
use tu_ftco_harness::*;
use tu_ftco_attack::*;
use melee_fighter::tu::ftCo_AttackAir as m;
use melee_fighter::types::*;
use m::ext::hook as hk;

fn isf(fr: &Fighter) -> u32 { (fr as *const Fighter == unsafe { F } as *const Fighter) as u32 }

#[test]
fn ftCo_AttackAir_matches_c() {
    init();
    af_init();
    unsafe {
        hk::ftCo_800DF478 = Some(|fr| { t(1); t(isf(fr)); chk(1) });
        hk::it_8026B30C = Some(|i| { t(2); t(af_item(i)); (srv(0) % 5) as i32 });
        hk::ftCo_800CDDA0 = vrec!(3);
        hk::ftLk_AttackAir_Enter = vrec!(4);
        hk::ftGw_AttackAirN_DecideAction = vrec!(5);
        hk::ftCo_GetCStickAngle = Some(|fr| { t(6); t(isf(fr)); angle_of(srv(3)) });
        hk::ftCo_GetLStickAngle = Some(|fr| { t(7); t(isf(fr)); angle_of(srv(11)) });
        hk::Fighter_ChangeMotionState = Some(|g, ms, fl, a, bb, c, o| {
            t(8); t(is_go(g)); t(ms as u32); t(fl); t(b(a)); t(b(bb)); t(b(c)); t(o.is_null() as u32)
        });
        hk::ftAnim_8006EBA4 = vrec!(9);
        hk::ftAnim_IsFramesRemaining = chkrec!(10);
        hk::ftCo_Fall_Enter = vrec!(11);
        hk::ftCo_80095328 = Some(|g, o| { t(12); t(is_go(g)); t(o.is_null() as u32); chk(12) });
        hk::ftCo_800D7100 = chkrec!(13);
        hk::ftCo_800C3B10 = chkrec!(14);
        hk::ftCo_800D705C = chkrec!(15);
        hk::ftCo_800CB870 = chkrec!(16);
        hk::ft_80084DB0 = vrec!(17);
        hk::ft_80082C74 = Some(|g, cb| {
            t(18); t(is_go(g));
            t((cb.map(|p| p as usize) == Some(m::ext::ftCo_LandingAir_EnterWithLag as usize)) as u32)
        });
    }
    let mut c = Check::new(include_str!("data/tu_ftCo_AttackAir.txt"));
    for op in 0..16u32 {
        for k in 0..2000u32 {
            case(op, k);
            af_fill();
            let ms = (rnd() % 400) as i32;
            let g = go();
            let ret = match op {
                0 | 15 => m::checkItemThrowInput(g) as u32,
                1 => m::ftCo_AttackAir_CheckItemThrowInput(g) as u32,
                2 => { m::decideFighter(g); 0 }
                3 | 14 => m::ftCo_AttackAir_GetMsidFromCStick(f()) as u32,
                4 => { m::ftCo_AttackAir_EnterFromCStick(g); 0 }
                5 => { m::ftCo_AttackAir_EnterFromMsid(g, ms); 0 }
                6 => { m::ftCo_AttackAir_Anim(g); 0 }
                7 => { m::ftCo_AttackAirN_IASA(g); 0 }
                8 => { m::ftCo_AttackAirF_IASA(g); 0 }
                9 => { m::ftCo_AttackAirB_IASA(g); 0 }
                10 => { m::ftCo_AttackAirHi_IASA(g); 0 }
                11 => { m::ftCo_AttackAirLw_IASA(g); 0 }
                12 => { m::ftCo_AttackAir_Phys(g); 0 }
                _ => { m::ftCo_AttackAir_Coll(g); 0 }
            };
            c.line(op, k, ret, af_hash());
        }
    }
    assert_eq!(c.done(), 32000);
}
