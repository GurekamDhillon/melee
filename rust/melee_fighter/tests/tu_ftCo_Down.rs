//! src/tu/ftCo_Down.rs vs the C (gen/cref/tu/ftCo_Down.c, shipping contraction; regenerate with
//! `python rust/melee_fighter/gen/cref/tu/run_tu.py ftCo_Down`).
#![allow(static_mut_refs, non_snake_case)]
mod tu_ftco_harness;
use tu_ftco_harness::*;
use melee_fighter::tu::ftCo_Down as m;
use m::ext::hook as hk;
use melee_fighter::types::*;

static mut ANG: f32 = 0.0;
fn isf(fr: &Fighter) -> u32 { (fr as *const Fighter == unsafe { F } as *const Fighter) as u32 }

#[test]
fn ftCo_Down_matches_c() {
    init();
    unsafe {
        hk::ftCo_GetLStickAngle = Some(|fr| { t(1); t(isf(fr)); ANG });
        hk::ftCo_800DF678 = Some(|fr| { t(2); t(isf(fr)); sr(1) });
        hk::ftCo_800DF644 = Some(|fr| { t(3); t(isf(fr)); sr(2) });
        hk::Fighter_ChangeMotionState = Some(|g, ms, fl, a, bb, c, o| {
            t(4); t(is_go(g)); t(ms as u32); t(fl); t(b(a)); t(b(bb)); t(b(c)); t(o.is_null() as u32)
        });
        hk::ftAnim_8006EBA4 = Some(|g| { t(5); t(is_go(g)) });
        hk::ftCommon_SetGroundedKnockbackIfLanded = Some(|fr| { t(6); t(isf(fr)) });
        hk::ftAnim_IsFramesRemaining = Some(|_| { t(7); sr(3) });
        hk::ft_8008A2BC = Some(|g| { t(8); t(is_go(g)) });
        hk::ft_80084FA8 = Some(|g| { t(9); t(is_go(g)) });
        hk::ft_80084104 = Some(|g| { t(10); t(is_go(g)) });
        hk::ftCo_8009856C = Some(|g, ms| { t(11); t(is_go(g)); t(ms as u32) });
    }
    let ms = [ftCo_MS_DownWaitU, ftCo_MS_DownBoundU, ftCo_MS_DownWaitD, 0];
    let mut c = Check::new(include_str!("data/tu_ftCo_Down.txt"));
    for op in 0..7u32 {
        for k in 0..2000u32 {
            case(op, k);
            unsafe { ANG = rg() };
            let d = cd();
            d.x248 = rg(); d.x20_radians = rg(); d.x24C = rw();
            let fr = f();
            fr.input.lstick[0].x = rg();
            fr.input.cstick[0].x = rg();
            fr.facing_dir = rg();
            fr.motion_id = ms[(rnd() % 4) as usize] as _;
            fr.x67C = (rnd() % 20) as u8; fr.x67D = (rnd() % 20) as u8;
            let g = go();
            let ret = match op {
                0 => m::ftCo_Down_CheckInput(g) as u32,
                1 => { m::ftCo_80098324(g, (rnd() % 400) as i32); 0 }
                2 => { m::ftCo_Down_Anim(g); 0 }
                3 => { m::ftCo_Down_IASA(g); 0 }
                4 => { m::ftCo_Down_Phys(g); 0 }
                5 => { m::ftCo_Down_Coll(g); 0 }
                _ => m::ftCo_80098400(g) as u32,
            };
            let mut h = Hs::new();
            h.m(f().motion_id as u32);
            c.line(op, k, ret, h.0);
        }
    }
    assert_eq!(c.done(), 14000);
}
