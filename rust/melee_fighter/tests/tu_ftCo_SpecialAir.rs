//! src/tu/ftCo_SpecialAir.rs vs the C (gen/cref/tu/ftCo_SpecialAir.c: the whole TU, shipping
//! contraction; regenerate with `python rust/melee_fighter/gen/cref/tu/run_tu.py ftCo_SpecialAir`).
#![allow(static_mut_refs, non_snake_case)]
mod tu_ftco_harness;
mod tu_ftco_s209;
use tu_ftco_harness::*;
use tu_ftco_s209::*;
use melee_fighter::tu::ftCo_SpecialAir as m;
use m::ext::hook as hk;
use melee_fighter::types::*;

unsafe extern "C" fn vanilla_fn(_g: *mut HSD_GObj) {}
fn tab(bit: u32) -> HSD_GObjEvent {
    if unsafe { SRV[1] } & bit != 0 { Some(vanilla_fn) } else { None }
}
fn van(v: HSD_GObjEvent) -> u32 {
    match v {
        None => 0,
        Some(f) if f as usize == vanilla_fn as usize => 1,
        Some(_) => 9,
    }
}

#[test]
fn ftCo_SpecialAir_matches_c() {
    s209_init();
    unsafe {
        hk::ScriptGame_CapsActionAllowed = Some(|fr, fl| { t(1); t(pi(fr as *const Fighter)); t(fl as u32); (sr(1) | sr(17)) as i32 });
        hk::ftData_SpecialAirHi = Some(|_| tab(1));
        hk::ftData_SpecialAirLw = Some(|_| tab(2));
        hk::ftData_SpecialAirS = Some(|_| tab(4));
        hk::ftData_SpecialAirN = Some(|_| tab(8));
        hk::Geno_SpecialEnter = Some(|g, w| { t(2); t(pi(g)); t(w as u32); sr(2) as i32 });
        hk::Mex_SpecialHiAirDispatch = Some(|k, g, v| { t(3); t(k as u32); t(pi(g)); t(van(v)) });
        hk::Mex_SpecialLwAirDispatch = Some(|k, g, v| { t(4); t(k as u32); t(pi(g)); t(van(v)) });
        hk::Mex_SpecialSAirDispatch = Some(|k, g, v| { t(5); t(k as u32); t(pi(g)); t(van(v)) });
        hk::Mex_SpecialNAirDispatch = Some(|k, g, v| { t(6); t(k as u32); t(pi(g)); t(van(v)) });
        hk::ftCommon_UpdateFacing = Some(|fr| { t(8); t(pi(fr as *const Fighter)) });
    }
    let mut c = Check::new(include_str!("data/tu_ftCo_SpecialAir.txt"));
    for k in 0..8000u32 {
        case(0, k);
        s209_fill();
        if unsafe { SRV[0] } & 1 != 0 {
            f().input.lstick[0].x = 0.0;
            f().input.lstick[0].y = 0.0;
        }
        let ret = m::ftCo_SpecialAir_CheckInput(go()) as u32;
        c.line(0, k, ret, state_hash());
    }
    assert_eq!(c.done(), 8000);
}
