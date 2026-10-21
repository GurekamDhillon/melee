//! src/tu/ftCo_Run.rs vs the C (gen/cref/tu/ftCo_Run.c: the whole TU, TARGET_PC, shipping
//! contraction; regenerate with `python rust/melee_fighter/gen/cref/tu/run_tu.py ftCo_Run`).
#![allow(static_mut_refs, non_snake_case)]
mod tu_ftco_harness;
mod tu_ftco_misc8;
use tu_ftco_harness::*;
use tu_ftco_misc8::*;
use melee_fighter::tu::ftCo_Run as m;
use m::ext::hook as hk;

static mut FR: f32 = 0.0;
static mut MV: [f32; 2] = [0.0; 2];
static mut NMV: usize = 0;

macro_rules! v { ($($n:ident = $id:expr),*) => { unsafe { $( hk::$n = Some(|g| { t($id); t(pi(g)) }); )* } } }
macro_rules! bs { ($($n:ident = $id:expr),*) => { unsafe { $( hk::$n = Some(|g| { t($id); t(pi(g)); sr($id) }); )* } } }

#[test]
fn ftCo_Run_matches_c() {
    m8_init();
    unsafe {
        hk::ScriptGame_CapsActionAllowed = Some(|fr, fl| { t(1); t(pf(fr)); t(fl as u32); sr(1) as i32 });
        hk::ScriptGame_ModValue = Some(|s, fl, bs| { t(2); t(s as u32); t(fl as u32); t(b(bs)); let v = MV[NMV & 1]; NMV += 1; v });
        hk::Fighter_ChangeMotionState = Some(|g, ms, fl, a, bb, c, o| { t(3); t(pi(g)); t(ms as u32); t(fl); t(b(a)); t(b(bb)); t(b(c)); t(pi(o)) });
        hk::ft_GetGroundFrictionMultiplier = Some(|fr| { t(4); t(pf(fr)); FR });
        hk::ftAnim_SetAnimRate = Some(|g, r| { t(5); t(pi(g)); t(b(r)) });
        hk::ftCo_80091B90 = Some(|g, x| { t(14); t(pi(g)); t(x as u32) });
        hk::ftCommon_CalcGroundAccel_DashRun = Some(|fr, a, tv, fc| { t(20); t(pf(fr)); t(b(a)); t(b(tv)); t(b(fc)) });
    }
    bs!(ftCo_SpecialS_CheckInput = 6, ftCo_Attack100_CheckInput = 7, ftCo_800D6824 = 8, ftCo_800D68C0 = 9,
        ftCo_800D8A38 = 10, ftCo_AttackDash_CheckInput = 11, ftCo_80091A4C = 13, ftCo_800DE9D8 = 16,
        fn_800CAF78 = 17, fn_800C9D40 = 18, ftCo_RunBrake_CheckInput = 19);
    v!(ftCo_AttackDash_SetMv0 = 12, ftCo_80091B9C = 15, ftCommon_SetSelfMovementFromGroundedMovement = 21, ft_800844EC = 22);
    let mut c = Check::new(include_str!("data/tu_ftCo_Run.txt"));
    for op in 0..10u32 {
        for k in 0..3000u32 {
            case(op, k);
            m8_fill();
            unsafe {
                NMV = 0;
                FR = rg();
                MV[0] = rw();
                MV[1] = rw();
                if rnd() & 1 != 0 { MV[1] = 0.0; }
            }
            let g = go();
            let ret = match op {
                0 => m::fn_800CA5F0(g) as u32,
                1 => m::fn_800CA644(g) as u32,
                2 => m::fn_800CA698(g) as u32,
                3 => { m::ftCo_Run_Enter(g, rw()); 0 }
                4 => { let a = rw(); let bb = rg(); let cc = rg(); m::ftCo_Run_Enter_Full(g, a, bb, cc); 0 }
                5 => { m::ftCo_Run_Anim(g); 0 }
                6 => { m::ftCo_Run_IASA(g); 0 }
                7 | 9 => { m::ftCo_Run_Phys(g); 0 }
                _ => { m::ftCo_Run_Coll(g); 0 }
            };
            c.line(op, k, ret, state_hash());
        }
    }
    assert_eq!(c.done(), 30000);
}
