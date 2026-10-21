//! src/tu/ftCo_LandingAir.rs vs the C (gen/cref/tu/ftCo_LandingAir.c: the whole TU, shipping
//! contraction; regenerate with `python rust/melee_fighter/gen/cref/tu/run_tu.py ftCo_LandingAir`).
//! Everything observable goes through the callees, so the trace carries it (state hash 0).
#![allow(static_mut_refs, non_snake_case)]
mod tu_ftco_harness;
use tu_ftco_harness::*;
use melee_fighter::tu::ftCo_LandingAir as m;
use m::ext::hook as hk;
use melee_fighter::types::*;

macro_rules! v { ($($n:ident = $id:expr),*) => { unsafe { $( hk::$n = Some(|g| { t($id); t(is_go(g)) }); )* } } }

#[test]
fn ftCo_LandingAir_matches_c() {
    init();
    v!(ftCo_Landing_Enter_Basic = 6, ftCo_Landing_Anim = 7, ftCo_Landing_Phys = 8, ftCo_Landing_Coll = 9);
    unsafe {
        hk::ScriptGame_SkillLanding = Some(|fr, mode, lag, lf| {
            t(1); t((fr as *mut Fighter == f() as *mut Fighter) as u32); t(mode as u32);
            if mode != -1 { t(b(lag)); }
            t(b(lf))
        });
        hk::ftCommon_8007D7FC = Some(|fr| { t(2); t((fr as *mut Fighter == f() as *mut Fighter) as u32) });
        hk::Fighter_ChangeMotionState = Some(|g, ms, fl, a, bb, c, o| {
            t(3); t(is_go(g)); t(ms as u32); t(fl); t(b(a)); t(b(bb)); t(b(c)); t(o.is_null() as u32)
        });
        hk::ftAnim_SetAnimRate = Some(|g, r| { t(4); t(is_go(g)); t(b(r)) });
        hk::ftAnim_8006F484 = Some(|g| { t(5); t(is_go(g)); rw() });
    }
    let mut c = Check::new(include_str!("data/tu_ftCo_LandingAir.txt"));
    for op in 0..6u32 {
        for k in 0..4000u32 {
            case(op, k);
            let fr = f();
            fr.cmd_vars[0] = if rnd() % 3 != 0 { 1 } else { 0 };
            fr.motion_id = (63 + rnd() % 9) as _;
            fr.co_attrs.landingairn_lag = rw(); fr.co_attrs.landingairf_lag = rw(); fr.co_attrs.landingairb_lag = rw();
            fr.co_attrs.landingairhi_lag = rw(); fr.co_attrs.landingairlw_lag = rw();
            fr.x67F = (rnd() % 12) as u8;
            cd().xE4 = (rnd() % 12) as i32;
            cd().xE8 = rw();
            let ms = (rnd() % 400) as i32;
            let lag = rw();
            let g = go();
            match op {
                0 => m::ftCo_LandingAir_EnterWithLag(g),
                1 => m::ftCo_LandingAir_EnterWithMsidLag(g, ms, lag),
                2 => m::ftCo_LandingAir_Anim(g),
                3 => m::ftCo_LandingAir_IASA(g),
                4 => m::ftCo_LandingAir_Phys(g),
                _ => m::ftCo_LandingAir_Coll(g),
            }
            c.line(op, k, 0, 0);
        }
    }
    assert_eq!(c.done(), 24000);
}
