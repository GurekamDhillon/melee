//! src/tu/ftCo_CargoKneebend.rs vs the C (gen/cref/tu/ftCo_CargoKneebend.c: the whole TU, shipping contraction;
//! regenerate with `python rust/melee_fighter/gen/cref/tu/run_tu.py ftCo_CargoKneebend`).
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
use melee_fighter::tu::ftCo_CargoKneebend as m;
use m::ext::hook as hk;

#[test]
fn ftCo_CargoKneebend_matches_c() {
    cargo_init();
    v!(ftCo_8009BB64 = 3, ftCo_KneeBend_Check_ShortHop = 5, ft_80084F3C = 6, ftCo_CargoWait_Coll = 7);
    unsafe {
        hk::Fighter_ChangeMotionState = Some(CMS);
        hk::ftAnim_SetAnimRate = Some(|g, r| { t(1); t(pi(g)); t(b(r)) });
        hk::ftCo_8009C5A4 = Some(|g, ms| { t(2); t(pi(g)); t(ms as u32) });
        hk::ftCo_8009BF3C = Some(|g| { t(4); t(pi(g)); sr(4) });
    }
    let mut c = Check::new(include_str!("data/tu_ftCo_CargoKneebend.txt"));
    for op in 0..5u32 {
        for k in 0..1000u32 {
            case(op, k);
            cargo_fill();
            let g = go();
            let ret: u32 = match op {
                0 => { m::ftCo_8009B9C8(g, rnd() as i32); 0 }
                1 => { m::ftCo_CargoKneebend_Anim(g); 0 }
                2 => { m::ftCo_CargoKneebend_IASA(g); 0 }
                3 => { m::ftCo_CargoKneebend_Phys(g); 0 }
                _ => { m::ftCo_CargoKneebend_Coll(g); 0 }
            };
            c.line(op, k, ret, state_hash());
        }
    }
    assert_eq!(c.done(), 5000);
}
