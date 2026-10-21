//! src/tu/ftCo_Walk.rs vs the C (gen/cref/tu/ftCo_Walk.c: the whole TU, shipping contraction;
//! regenerate with `python rust/melee_fighter/gen/cref/tu/run_tu.py ftCo_Walk`).
#![allow(static_mut_refs, non_snake_case)]
mod tu_ftco_harness;
use tu_ftco_harness::*;
use melee_fighter::tu::ftCo_Walk as m;
use m::ext::hook as hk;

macro_rules! ck { ($($n:ident = $id:expr),*) => { unsafe { $( hk::$n = Some(|g| { t($id); t(is_go(g)); sr($id) && sr($id + 16) && sr($id + 32) }); )* } } }
macro_rules! vs { ($($n:ident = $id:expr),*) => { unsafe { $( hk::$n = Some(|g| { t($id); t(is_go(g)) }); )* } } }

static mut M1: f32 = 0.0;
static mut M2: f32 = 0.0;
static mut M3: f32 = 0.0;
static mut KR: f32 = 0.0;

#[test]
fn ftCo_Walk_matches_c() {
    init();
    ck!(ftCo_Catch_CheckInput = 1, ftCo_SpecialS_CheckInput = 2, ftCo_Attack100_CheckInput = 3, ftCo_800D6824 = 4,
        ftCo_800D68C0 = 5, ftCo_AttackS4_CheckInput = 6, ftCo_AttackHi4_CheckInput = 7, ftCo_AttackLw4_CheckInput = 8,
        ftCo_AttackS3_CheckInput = 9, ftCo_AttackHi3_CheckInput = 10, ftCo_AttackLw3_CheckInput = 11,
        ftCo_Attack1_CheckInput = 12, ftCo_80091A4C = 13, ftCo_800DE9D8 = 14, ftCo_Jump_CheckInput = 15,
        ftCo_Dash_CheckInput = 32, ftCo_800D5FB0 = 33, ft_8008A244 = 34);
    vs!(ftWalkCommon_800DFDDC = 42, ftWalkCommon_800E0060 = 44, ft_80084280 = 45);
    unsafe {
        hk::ftWalkCommon_800DFC70 = Some(|g| { t(40); t(is_go(g)); sr(40) });
        hk::ftWalkCommon_800DFCA4 = Some(|g, ms, fl, a, bb, c, d, e, ff, h, acc| {
            t(41); t(is_go(g)); t(ms as u32); t(fl); t(b(a)); t(b(bb)); t(b(c)); t(b(d)); t(b(e)); t(b(ff)); t(b(h)); t(b(acc))
        });
        hk::ftWalkCommon_800DFEC8 = Some(|g, cb| { t(43); t(is_go(g)); if sr(43) { cb(g, 0.5) } });
        hk::ftCo_CalcYScaledKnockback = Some(|a, s, c| { t(46); t(b(a)); t(b(s)); t(b(c)); KR });
        hk::Fighter_804D651C_x0 = Some(|| M1);
        hk::Fighter_804D6520_x0 = Some(|| M2);
        hk::Fighter_804D6524_xC = Some(|| M3);
    }
    let mut c = Check::new(include_str!("data/tu_ftCo_Walk.txt"));
    for op in 0..7u32 {
        for k in 0..3000u32 {
            case(op, k);
            unsafe { M1 = rw(); M2 = rw(); M3 = rg(); KR = rw(); }
            cd().teeter_walk_threshold = rg();
            let fr = f();
            fr.input.lstick[0].x = rg();
            fr.facing_dir = rg();
            fr.set_is_metal((rnd() & 1) as _);
            fr.x34_scale.y = if rnd() & 1 != 0 { 1.0 } else { rg() };
            fr.x197C = if rnd() & 1 != 0 { go2() } else { core::ptr::null_mut() };
            fr.x2DC = rw(); fr.x2E0 = rw(); fr.x2E4 = rw();
            fr.co_attrs.slow_walk_max = rw(); fr.co_attrs.mid_walk_point = rw(); fr.co_attrs.fast_walk_min = rw();
            let g = go();
            let ret = match op {
                0 => m::ftCo_Walk_CheckInput(g) as u32,
                1 => m::ftCo_Walk_CheckInput_Ottotto(g) as u32,
                2 => { m::ftCo_Walk_Enter(g, rw()); 0 }
                3 => { m::ftCo_Walk_Anim(g); 0 }
                4 => { m::ftCo_Walk_IASA(g); 0 }
                5 => { m::ftCo_Walk_Phys(g); 0 }
                _ => { m::ftCo_Walk_Coll(g); 0 }
            };
            c.line(op, k, ret, 0);
        }
    }
    assert_eq!(c.done(), 21000);
}
