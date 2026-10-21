//! src/tu/ftCo_CargoLanding.rs vs the C (gen/cref/tu/ftCo_CargoLanding.c: the whole TU, shipping contraction;
//! regenerate with `python rust/melee_fighter/gen/cref/tu/run_tu.py ftCo_CargoLanding`).
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
use melee_fighter::tu::ftCo_CargoLanding as m;
use m::ext::hook as hk;

#[test]
fn ftCo_CargoLanding_matches_c() {
    cargo_init();
    v!(ftCo_8009B518 = 4, ft_80084F3C = 5, ftCo_CargoWait_Coll = 6);
    unsafe {
        hk::ftCo_Landing_Enter = Some(|g, ms, ai, fl, a, bb| { t(1); t(pi(g)); t(ms as u32); t(ai as u32); t(fl); t(b(a)); t(b(bb)) });
        hk::ftAnim_SetAnimRate = Some(|g, r| { t(2); t(pi(g)); t(b(r)) });
        hk::ftCo_8009C5A4 = Some(|g, ms| { t(3); t(pi(g)); t(ms as u32) });
    }
    let mut c = Check::new(include_str!("data/tu_ftCo_CargoLanding.txt"));
    for op in 0..4u32 {
        for k in 0..1000u32 {
            case(op, k);
            cargo_fill();
            let g = go();
            let ret: u32 = match op {
                0 => { m::ftCo_8009BD4C(g); 0 }
                1 => { m::ftCo_CargoLanding_Anim(g); 0 }
                2 => { m::ftCo_CargoLanding_Phys(g); 0 }
                _ => { m::ftCo_CargoLanding_Coll(g); 0 }
            };
            c.line(op, k, ret, state_hash());
        }
    }
    assert_eq!(c.done(), 4000);
}
