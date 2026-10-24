//! tu::ftco_0c60 smoke test: every callee is an unported `ext` stub, so entering the hammer fall
//! must reach ftCo_80099F1C's TODO panic (no silent default).

use melee_fighter::tu::ftco_0c60::ftco_800c60c8;
use melee_fighter::types::*;

// 2026-10-24: the stub for ftCo_80099F1C now falls back to the real port, which stops earlier on missing global data.
#[test]
#[should_panic(expected = "p_ftCommonData not loaded")]
fn reaches_unported_callee() {
    let mut f = Box::new(Fighter::zeroed());
    let mut go: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    go.user_data = (&mut *f as *mut Fighter).cast();
    ftco_800c60c8(&mut *go);
}
