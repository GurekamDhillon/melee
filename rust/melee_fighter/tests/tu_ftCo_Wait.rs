//! src/tu/ftCo_Wait.rs vs the C (gen/cref/tu/ftCo_Wait.c: the whole TU, shipping contraction;
//! regenerate with `python rust/melee_fighter/gen/cref/tu/run_tu.py ftCo_Wait`).
#![allow(static_mut_refs, non_snake_case)]
mod tu_ftco_harness;
use tu_ftco_harness::*;
use melee_fighter::tu::ftCo_Wait as m;
use m::ext::hook as hk;
use melee_fighter::types::*;

macro_rules! ck { ($($n:ident = $id:expr),*) => { unsafe { $( hk::$n = Some(|g| { t($id); t(is_go(g)); sr($id) && sr($id + 16) && sr($id + 32) }); )* } } }
macro_rules! vs { ($($n:ident = $id:expr),*) => { unsafe { $( hk::$n = Some(|g| { t($id); t(is_go(g)) }); )* } } }

#[test]
fn ftCo_Wait_matches_c() {
    init();
    ck!(ftCo_SpecialS_CheckInput = 1, ftCo_Attack100_CheckInput = 2, ftCo_800D6824 = 3, ftCo_800D68C0 = 4,
        ftCo_Catch_CheckInput = 5, ftCo_AttackS4_CheckInput = 6, ftCo_AttackHi4_CheckInput = 7,
        ftCo_AttackLw4_CheckInput = 8, ftCo_AttackS3_CheckInput = 9, ftCo_AttackHi3_CheckInput = 10,
        ftCo_AttackLw3_CheckInput = 11, ftCo_Attack1_CheckInput = 12, ftCo_80099794 = 13, ftCo_80091A4C = 14,
        ftFx_AppealS_CheckInput = 15, ftCo_800DE9D8 = 32, ftCo_Jump_CheckInput = 33, ftCo_Dash_CheckInput = 34,
        ftCo_800D5FB0 = 35, ftCo_Turn_CheckInput = 36, ftCo_Walk_CheckInput = 37);
    vs!(ftCo_DownSpot_Enter = 40, ft_80084F3C = 42, ftColl_8007AEE0 = 43, ft_80084280 = 44);
    unsafe {
        hk::ftCo_8008A7A8 = Some(|g, ws| { t(41); t(is_go(g)); t((ws as *mut HSD_GObj == igo()) as u32) });
    }
    let fd: &'static mut ftData = Box::leak(Box::new(unsafe { core::mem::zeroed::<ftData>() }));
    let fdp = fd as *mut ftData;
    let mut c = Check::new(include_str!("data/tu_ftCo_Wait.txt"));
    for op in 0..4u32 {
        for k in 0..3000u32 {
            case(op, k);
            unsafe {
                *fdp = core::mem::zeroed();
                (*fdp).x24 = igo() as *mut WaitStruct;
            }
            let fr = f();
            fr.ft_data = fdp;
            fr.set_stamina_dead((rnd() & 1) as _);
            let g = go();
            match op {
                0 => m::ftCo_Wait_Anim(g),
                1 => m::ftCo_Wait_IASA(g),
                2 => m::ftCo_Wait_Phys(g),
                _ => m::ftCo_Wait_Coll(g),
            }
            c.line(op, k, 0, 0);
        }
    }
    assert_eq!(c.done(), 12000);
}
