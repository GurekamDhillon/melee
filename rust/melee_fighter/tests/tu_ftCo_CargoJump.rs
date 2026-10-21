//! src/tu/ftCo_CargoJump.rs vs the C (gen/cref/tu/ftCo_CargoJump.c: the whole TU, shipping contraction;
//! regenerate with `python rust/melee_fighter/gen/cref/tu/run_tu.py ftCo_CargoJump`).
#![allow(static_mut_refs, non_snake_case)]
mod tu_ftco_harness;
mod tu_ftco_cargo;
use tu_ftco_harness::*;
use tu_ftco_cargo::*;
use melee_fighter::types::*;

macro_rules! v { ($($n:ident = $id:expr),*) => { unsafe { $( hk::$n = Some(|g| { t($id); t(pi(g)) }); )* } } }
const CMS: fn(*mut HSD_GObj, i32, u32, f32, f32, f32, *mut HSD_GObj) = |g, ms, fl, a, bb, c, o| {
    t(9); t(pi(g)); t(ms as u32); t(fl); t(b(a)); t(b(bb)); t(b(c)); t(pi(o))
};
use melee_fighter::tu::ftCo_CargoJump as m;
use m::ext::hook as hk;

#[test]
fn ftCo_CargoJump_matches_c() {
    cargo_init();
    v!(ftCo_Jump_Phys_Inner = 8, ftCo_CargoFall_Coll = 10);
    unsafe {
        hk::Fighter_ChangeMotionState = Some(CMS);
        hk::ftCo_Jump_GetInput = Some(|g| { t(1); t(pi(g)); SRV[1] % 3 });
        hk::ftCo_8009B9C8 = Some(|g, a| { t(2); t(pi(g)); t(a as u32) });
        hk::ftCommon_8007D5D4 = Some(|f| { t(3); t(pi(f as *const Fighter)) });
        hk::ftAnim_SetAnimRate = Some(|g, r| { t(4); t(pi(g)); t(b(r)) });
        hk::ftCo_800CB110 = Some(|g, bo, x| { t(5); t(pi(g)); t(bo as u32); t(b(x)) });
        hk::ftCo_8009C5A4 = Some(|g, ms| { t(6); t(pi(g)); t(ms as u32) });
        hk::ftCo_8009BF3C = Some(|g| { t(7); t(pi(g)); sr(7) });
    }
    let mut c = Check::new(include_str!("data/tu_ftCo_CargoJump.txt"));
    for op in 0..6u32 {
        for k in 0..1000u32 {
            case(op, k);
            cargo_fill();
            let g = go();
            let ret: u32 = match op {
                0 => m::ftCo_8009BB1C(g) as u32,
                1 => { m::ftCo_8009BB64(g); 0 }
                2 => { m::ftCo_CargoJump_Anim(g); 0 }
                3 => { m::ftCo_CargoJump_IASA(g); 0 }
                4 => { m::ftCo_CargoJump_Phys(g); 0 }
                _ => { m::ftCo_CargoJump_Coll(g); 0 }
            };
            c.line(op, k, ret, state_hash());
        }
    }
    assert_eq!(c.done(), 6000);
}
