//! src/tu/ftCo_Pass.rs vs the C (gen/cref/tu/ftCo_Pass.c: the whole TU, TARGET_PC, shipping
//! contraction; regenerate with `python rust/melee_fighter/gen/cref/tu/run_tu.py ftCo_Pass`).
#![allow(static_mut_refs, non_snake_case)]
mod tu_ftco_harness;
mod tu_ftco_misc8;
use tu_ftco_harness::*;
use tu_ftco_misc8::*;
use melee_fighter::tu::ftCo_Pass as m;
use m::ext::hook as hk;

macro_rules! v { ($($n:ident = $id:expr),*) => { unsafe { $( hk::$n = Some(|g| { t($id); t(pi(g)) }); )* } } }
macro_rules! bs { ($($n:ident = $id:expr),*) => { unsafe { $( hk::$n = Some(|g| { t($id); t(pi(g)); sr($id) }); )* } } }

fn isc(c: &melee_fighter::types::CollData) -> u32 { (c as *const _ == &f().coll_data as *const _) as u32 }

#[test]
fn ftCo_Pass_matches_c() {
    m8_init();
    unsafe {
        hk::mpColl_IsOnPlatform = Some(|c| { t(1); t(isc(c)); sr(1) });
        hk::mpUpdateFloorSkip = Some(|c| { t(2); t(isc(c)) });
        hk::Replay_UcfVersion = Some(|p| { t(5); t(p as u32); if SRV[5] & 1 != 0 { 84 } else { (rnd() % 90) as i32 } });
        hk::ftUcf_FlickCounter = Some(|p| { t(6); t(p as u32); (rnd() % 4) as i32 });
        hk::ftCommon_8007D5D4 = Some(|fr| { t(7); t(pf(fr)) });
        hk::ftCommon_ClampAirDrift = Some(|fr| { t(8); t(pf(fr)) });
        hk::Fighter_ChangeMotionState = Some(|g, ms, fl, a, bb, c, o| { t(9); t(pi(g)); t(ms as u32); t(fl); t(b(a)); t(b(bb)); t(b(c)); t(pi(o)) });
        hk::ftCo_80095328 = Some(|g, p| { t(13); t(pi(g)); t(p.is_null() as u32); sr(13) });
    }
    bs!(ftCo_800C5240 = 3, ftCo_800C60C8 = 4, ftAnim_IsFramesRemaining = 10, ftCo_SpecialAir_CheckInput = 12,
        ftCo_800D7100 = 14, ftCo_800C3B10 = 15, ftCo_80099A58 = 16, ftCo_AttackAir_CheckItemThrowInput = 17,
        ftCo_800D705C = 18, ftCo_800CB870 = 19);
    v!(ftCo_Fall_Enter = 11, ft_80084DB0 = 20, ft_80082F28 = 21);
    let mut c = Check::new(include_str!("data/tu_ftCo_Pass.txt"));
    for op in 0..11u32 {
        for k in 0..3000u32 {
            case(op, k);
            m8_fill();
            let g = go();
            let ret = match op {
                0 => m::ftCo_80099F1C(g) as u32,
                1 => m::ftCo_80099F9C(g) as u32,
                2 | 10 => m::ftCo_8009A080(g) as u32,
                3 => m::ftCo_8009A134(g) as u32,
                4 => {
                    let ms = (rnd() % 400) as i32;
                    let mf = rnd();
                    let st = rg();
                    m::ftCo_8009A184(g, ms, mf, st);
                    0
                }
                5 => { m::ftCo_8009A228(g); 0 }
                6 => { m::ftCo_Pass_Anim(g); 0 }
                7 => { m::ftCo_Pass_IASA(g); 0 }
                8 => { m::ftCo_Pass_Phys(g); 0 }
                _ => { m::ftCo_Pass_Coll(g); 0 }
            };
            c.line(op, k, ret, state_hash());
        }
    }
    assert_eq!(c.done(), 33000);
}
