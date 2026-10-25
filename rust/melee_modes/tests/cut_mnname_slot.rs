//! tu::mnname::mnName_8023749C vs the C text itself (gen/cref/cut/mnname_slot.c; regenerate with
//! `python rust/melee_modes/gen/cref/cutref.py mnname_slot`).
#![allow(static_mut_refs, non_snake_case)]
mod mtu_harness;
use melee_modes::tu::mnname as m;
use mtu_harness::*;

unsafe extern "C" fn is_us() -> i32 {
    t(1);
    (fv() & 1) as i32
}

#[test]
fn mnname_slot_matches_c() {
    unsafe {
        m::ext::H_lbLang_IsSavedLanguageUS = Some(is_us);
        let mut c = Check::new(include_str!("data/cut_mnname_slot.txt"));
        for k in 0..512u32 {
            case(0, k);
            let r = m::mnName_8023749C(if k < 256 { k as i32 } else { fv() as i32 });
            let ret = if r.is_null() { 0 } else { sh(r as *const u8) | 1 };
            hm(0);
            c.line(0, k, ret);
        }
        assert_eq!(c.done(), 512);
    }
}
