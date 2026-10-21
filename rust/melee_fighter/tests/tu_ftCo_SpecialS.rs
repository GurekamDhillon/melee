//! src/tu/ftCo_SpecialS.rs vs the C (gen/cref/tu/ftCo_SpecialS.c: the whole TU, shipping
//! contraction; regenerate with `python rust/melee_fighter/gen/cref/tu/run_tu.py ftCo_SpecialS`).
#![allow(static_mut_refs, non_snake_case)]
mod tu_ftco_harness;
mod tu_s217;
use melee_fighter::types::*;
use tu_ftco_harness::*;
use tu_s217::*;
use melee_fighter::tu::ftCo_SpecialS as m;
use m::ext::hook as hk;

unsafe extern "C" fn vanilla(_g: *mut HSD_GObj) { t(99) }

#[test]
fn ftCo_SpecialS_matches_c() {
    s_init();
    unsafe {
        hk::ScriptGame_CapsActionAllowed = Some(|f, fl| { t(1); t(pi(f)); t(fl as u32); (chk(1) | chk(7)) as i32 });
        hk::ftData_SpecialS = Some(|k| if k == f().kind && SRV[1] & 1 != 0 { Some(vanilla) } else { None });
        hk::ftCommon_UpdateFacing = Some(|f| { t(2); t(pi(f)) });
        hk::ft_GetGroundFrictionMultiplier = Some(|f| { t(3); t(pi(f)); rg() });
        hk::Geno_SpecialEnter = Some(|g, w| { t(4); t(pi(g)); t(w as u32); chk(4) as i32 });
        hk::Mex_SpecialSDispatch = Some(|k, g, v| { t(5); t(k as u32); t(pi(g)); t(v.is_some() as u32) });
    }
    let mut c = Check::new(include_str!("data/tu_ftCo_SpecialS.txt"));
    for op in 0..3u32 {
        for k in 0..3000u32 {
            case(op, k);
            s_fill();
            let g = go();
            let ret = match op {
                0 => m::ftCo_SpecialS_HasInput(f()) as u32,
                1 => m::ftCo_SpecialS_CheckInput(g) as u32,
                _ => { m::doEnter(g); 0 }
            };
            c.line(op, k, ret, state_hash());
        }
    }
    assert_eq!(c.done(), 9000);
}
