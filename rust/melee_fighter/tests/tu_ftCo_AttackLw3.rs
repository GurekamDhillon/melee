//! src/tu/ftCo_AttackLw3.rs vs the C (gen/cref/tu/ftCo_AttackLw3.c: the whole TU, shipping
//! contraction; regenerate with `python rust/melee_fighter/gen/cref/tu/run_tu.py ftCo_AttackLw3`).
#![allow(static_mut_refs, non_snake_case)]
mod tu_ftco_harness;
#[macro_use]
mod tu_ftco_attack;
use tu_ftco_harness::*;
use tu_ftco_attack::*;
use melee_fighter::tu::ftCo_AttackLw3 as m;
use melee_fighter::types::*;
use m::ext::hook as hk;

fn isf(fr: &Fighter) -> u32 { (fr as *const Fighter == unsafe { F } as *const Fighter) as u32 }
fn mv0() -> *mut u32 { &mut f().mv as *mut _ as *mut u32 }

#[test]
fn ftCo_AttackLw3_matches_c() {
    init();
    af_init();
    unsafe {
        hk::ftCo_GetLStickAngle = Some(|fr| { t(1); t(isf(fr)); angle_of(srv(11)) });
        hk::ftCo_80094E54 = Some(|fr| { t(2); t(isf(fr)); chk(2) });
        hk::ftCo_800957F4 = Some(|g, ms| { t(3); t(is_go(g)); t(ms as u32) });
        hk::ft_800892A0 = vrec!(4);
        hk::ft_80089824 = vrec!(5);
        hk::ftGw_AttackLw3_Enter = vrec!(6);
        hk::ftpickupitem_80094790 = chkrec!(7);
        hk::Fighter_ChangeMotionState = Some(|g, ms, fl, a, bb, c, o| {
            t(8); t(is_go(g)); t(ms as u32); t(fl); t(b(a)); t(b(bb)); t(b(c)); t(o.is_null() as u32)
        });
        hk::ftAnim_8006EBA4 = vrec!(9);
        hk::ftAnim_IsFramesRemaining = chkrec!(10);
        hk::ftCo_800D638C = vrec!(11);
        hk::ftCo_AttackS4_CheckInput = chkrec!(12);
        hk::ftCo_AttackHi4_CheckInput = chkrec!(13);
        hk::ftCo_AttackLw4_CheckInput = chkrec!(14);
        hk::ftCo_AttackS3_CheckInput = chkrec!(15);
        hk::ftCo_AttackHi3_CheckInput = chkrec!(16);
        hk::ftCo_Attack1_CheckInput = chkrec!(17);
        hk::ftCo_Jump_CheckInput = chkrec!(18);
        hk::ftCo_Dash_CheckInput = chkrec!(19);
        hk::ftCo_Squat_CheckInput = chkrec!(20);
        hk::ftCo_Turn_CheckInput = chkrec!(21);
        hk::ftCo_Walk_CheckInput = chkrec!(22);
        hk::ft_80084F3C = vrec!(23);
        hk::ft_80084104 = vrec!(24);
    }
    let mut c = Check::new(include_str!("data/tu_ftCo_AttackLw3.txt"));
    for op in 0..11u32 {
        for k in 0..2000u32 {
            case(op, k);
            af_fill();
            unsafe { *mv0() = rnd() & 1; }
            let g = go();
            let mut ret = match op {
                0 => m::ftCo_AttackLw3_CheckInput(g) as u32,
                1 => { unsafe { m::callUnk(g) }; 0 }
                2 => { m::decideFighter(g); 0 }
                3 => { m::doEnter(g); 0 }
                4 => { m::wrapper(g); 0 }
                5 => { m::ftCo_AttackLw3_Anim(g); 0 }
                6 => m::checkPadA(g) as u32,
                7 => m::checkItemThrowInput(g) as u32,
                8 => { m::ftCo_AttackLw3_IASA(g); 0 }
                9 => { m::ftCo_AttackLw3_Phys(g); 0 }
                _ => { m::ftCo_AttackLw3_Coll(g); 0 }
            };
            ret ^= unsafe { *mv0() };
            let is = f().x21EC.map(|p| p as usize) == Some(m::callUnk as usize);
            ret ^= (is as u32) << 31;
            c.line(op, k, ret, af_hash());
        }
    }
    assert_eq!(c.done(), 22000);
}
