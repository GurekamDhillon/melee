//! src/tu/ftCo_Landing.rs vs the C (gen/cref/tu/ftCo_Landing.c: the whole TU, shipping
//! contraction; regenerate with `python rust/melee_fighter/gen/cref/tu/run_tu.py ftCo_Landing`).
#![allow(static_mut_refs, non_snake_case)]
mod tu_ftco_harness;
use tu_ftco_harness::*;
use melee_fighter::tu::ftCo_Landing as m;
use m::ext::hook as hk;
use melee_fighter::types::*;

macro_rules! v { ($($n:ident = $id:expr),*) => { unsafe { $( hk::$n = Some(|g| { t($id); t(is_go(g)) }); )* } } }
macro_rules! chk { ($($n:ident = $id:expr),*) => { unsafe { $( hk::$n = Some(|g| { t($id); t(is_go(g)); sr($id) & sr($id + 16) & sr($id + 32) }); )* } } }

#[test]
fn ftCo_Landing_matches_c() {
    init();
    v!(ftPe_8011D598 = 6, ftCo_HammerLanding_Enter = 8, ft_8008A2BC = 10, ft_80084F3C = 30, ft_80084280 = 31);
    chk!(ftCo_800C5240 = 7, ftCo_SpecialS_CheckInput = 11, ftCo_Attack100_CheckInput = 12, ftCo_800D6824 = 13,
        ftCo_800D68C0 = 14, ftCo_Catch_CheckInput = 15, ftCo_AttackS4_CheckInput = 16, ftCo_AttackHi4_CheckInput = 17,
        ftCo_AttackLw4_CheckInput = 18, ftCo_AttackS3_CheckInput = 19, ftCo_AttackHi3_CheckInput = 20,
        ftCo_AttackLw3_CheckInput = 21, ftCo_Attack1_CheckInput = 22, ftCo_80091A4C = 23, ftCo_800DE9D8 = 24,
        ftCo_Jump_CheckInput = 25, ftCo_Dash_CheckInput = 26, ftCo_SquatWait_CheckInput = 27,
        ftCo_Turn_CheckInput = 28, ftCo_Walk_CheckInput = 29);
    unsafe {
        hk::ftCommon_8007D7FC = Some(|fr| { t(1); t((fr as *mut Fighter == f() as *mut Fighter) as u32) });
        hk::Fighter_ChangeMotionState = Some(|g, ms, fl, a, bb, c, o| {
            t(2); t(is_go(g)); t(ms as u32); t(fl); t(b(a)); t(b(bb)); t(b(c)); t(o.is_null() as u32)
        });
        hk::Geno_DefineTemplateSkeleton = Some(|k| { t(3); t(k as u32); (rnd() & 1) as i32 });
        hk::Geno_DefineBaseKind = Some(|k| { t(4); t(k as u32); (rnd() % 40) as i32 });
        hk::ftKb_MexKirbyLike = Some(|k| { t(5); t(k as u32); (rnd() & 1) as i32 });
        hk::ftAnim_IsFramesRemaining = Some(|g| { t(9); t(is_go(g)); sr(9) });
    }
    let mut c = Check::new(include_str!("data/tu_ftCo_Landing.txt"));
    for op in 0..8u32 {
        for k in 0..2000u32 {
            case(op, k);
            let fr = f();
            fr.kind = rnd() % 40;
            unsafe {
                fr.u.mr.x2234_tornadoCharge = rnd() as i32; fr.u.mr.x2238_isCapeBoost = rnd() as i32;
                fr.u.pe.specialairn_used = rnd() as _;
                fr.u.ms.x222C = rnd() as _; fr.u.gw.x2234 = rnd() as _; fr.u.pp.x224C = rnd() as _;
                fr.u.kb.xCC = rnd() as _; fr.u.kb.xC4 = rnd() as _; fr.u.kb.x64 = rnd() as _;
                fr.u.mt.x223C_isConfusionBoost = rnd() as _;
                fr.mv.co.landing.allow_interrupt = (rnd() % 3) as i32;
            }
            fr.cur_anim_frame = rw(); fr.frame_speed_mul = rw(); fr.co_attrs.normal_landing_lag = rw();
            fr.x2EC = rw();
            let ms = (rnd() % 400) as i32;
            let ai = rnd() & 1 != 0;
            let fl = rnd();
            let a = rw();
            let bb = rw();
            let g = go();
            match op {
                0 => m::ftCo_Landing_Enter(g, ms, ai, fl, a, bb),
                1 => m::ftCo_Landing_Enter_Basic(g),
                2 => m::ftCo_LandingFallSpecial_Enter_Basic(g),
                3 => m::ftCo_LandingFallSpecial_Enter(g, ai, a),
                4 => m::ftCo_Landing_Anim(g),
                5 => m::ftCo_Landing_IASA(g),
                6 => m::ftCo_Landing_Phys(g),
                _ => m::ftCo_Landing_Coll(g),
            }
            let fr = f();
            let mut h = Hs::new();
            unsafe {
                h.m(fr.mv.co.landing.allow_interrupt as u32);
                h.m(fr.u.mr.x2234_tornadoCharge as u32); h.m(fr.u.mr.x2238_isCapeBoost as u32); h.m(fr.u.pe.specialairn_used as u32);
                h.m(fr.u.ms.x222C as u32); h.m(fr.u.gw.x2234 as u32); h.m(fr.u.pp.x224C as u32);
                h.m(fr.u.kb.xCC as u32); h.m(fr.u.kb.xC4 as u32); h.m(fr.u.kb.x64 as u32); h.m(fr.u.mt.x223C_isConfusionBoost as u32);
            }
            c.line(op, k, 0, h.0);
        }
    }
    assert_eq!(c.done(), 16000);
}
