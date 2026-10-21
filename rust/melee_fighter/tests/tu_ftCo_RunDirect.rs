//! src/tu/ftCo_RunDirect.rs vs the C (gen/cref/tu/ftCo_RunDirect.c: the whole TU, shipping
//! contraction; regenerate with `python rust/melee_fighter/gen/cref/tu/run_tu.py ftCo_RunDirect`).
#![allow(static_mut_refs, non_snake_case)]
mod tu_ftco_harness;
mod tu_ftco_s209;
use tu_ftco_harness::*;
use tu_ftco_s209::*;
use melee_fighter::tu::ftCo_RunDirect as m;
use m::ext::hook as hk;

macro_rules! vs { ($($n:ident = $id:expr),*) => { unsafe { $( hk::$n = Some(|g| { t($id); t(pi(g)) }); )* } } }
macro_rules! bs { ($($n:ident = $id:expr),*) => { unsafe { $( hk::$n = Some(|g| { t($id); t(pi(g)); sr($id) }); )* } } }

#[test]
fn ftCo_RunDirect_matches_c() {
    s209_init();
    vs!(ftCo_Run_Anim = 1, ftCo_Run_Phys = 2, ftCo_Run_Coll = 3, ftCo_AttackDash_SetMv0 = 10, ftCo_80091B9C = 13);
    bs!(ftCo_SpecialS_CheckInput = 4, ftCo_Attack100_CheckInput = 5, ftCo_800D6824 = 6, ftCo_800D68C0 = 7,
        ftCo_800D8A38 = 8, ftCo_AttackDash_CheckInput = 9, ftCo_80091A4C = 11, ftCo_800DE9D8 = 14,
        fn_800CAF78 = 15, fn_800CA698 = 16, ft_8008A244 = 17);
    unsafe {
        hk::ftCo_80091B90 = Some(|g, n| { t(12); t(pi(g)); t(n as u32) });
    }
    let mut c = Check::new(include_str!("data/tu_ftCo_RunDirect.txt"));
    for op in 0..4u32 {
        for k in 0..1000u32 {
            case(op, k);
            s209_fill();
            let g = go();
            match op {
                0 => m::ftCo_RunDirect_Anim(g),
                1 => m::ftCo_RunDirect_IASA(g),
                2 => m::ftCo_RunDirect_Phys(g),
                _ => m::ftCo_RunDirect_Coll(g),
            }
            c.line(op, k, 0, state_hash());
        }
    }
    assert_eq!(c.done(), 4000);
}
