//! Written by gen/lbtu/mk_driver.py from the same description as gen/lbtu/tu/lbtime.c.
//! src/tu/lbtime.rs vs the C (the whole TU, shipping contraction; regenerate the expected file
//! with `python rust/melee_runtime/gen/lbtu/run_tu.py lbtime`).
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables, unused_unsafe, unused_parens)]
#[path = "lbtu_harness/mod.rs"]
mod mtu_harness;
use core::ptr::null_mut;
use melee_types::*;
use melee_runtime::tu::lbtime as m;
use mtu_harness::*;
#[path = "lbtu_gen/lbtime_gen.rs"]
mod g;

static mut CT: [OSCalendarTime; 2] = unsafe { core::mem::zeroed() };


unsafe fn setup() {
    let _ = 0;
    CT = core::mem::zeroed();
    for i in 0..2 { g::fill_OSCalendarTime(&mut CT[i]); }
    m::ext::__OSBusClock = 162000000;
}
unsafe fn hash_all() {

    for i in 0..2 { g::hash_OSCalendarTime(&CT[i]); }

}
#[test]
fn lbtime_matches_c() {
    unsafe {
        g::install();
        preg(&raw const CT, core::mem::size_of_val(&CT));
        g::freg_all();

        let mut c = Check::new(include_str!("data/lbtu_lbtime.txt"));
        for op in 0..6u32 {
            for k in 0..400u32 {
                case(op, k);
                setup();
                let mut ret = 0u32;
                match op {
                    0 => {  let v0: u32 = (fv()) as u32; let v1: u32 = (fv()) as u32; ret = m::lbTime_8000AEC8(v0, v1) as u32; } // lbTime_8000AEC8
                    1 => {  let v0: u32 = (fv()) as u32; let v1: i32 = (fv() as i32).wrapping_sub((fv() & 0x300) as i32); ret = m::lbTime_8000AEE4(v0, v1) as u32; } // lbTime_8000AEE4
                    2 => {  let v0: u32 = (fv()) as u32; let v1: i32 = (fv() as i32) as i32; ret = m::lbTime_8000AF24(v0, v1) as u32; } // lbTime_8000AF24
                    3 => {  let v0: u32 = (fv()) as u32; let v1: i32 = (fv() as i32) as i32; ret = m::lbTime_8000AF74(v0, v1) as u32; } // lbTime_8000AF74
                    4 => {   ret = m::lbTime_GetTimeInSeconds() as u32; } // lbTime_GetTimeInSeconds
                    5 => {  let v1: u32 = (fv()) as u32; m::lbTime_8000B028(&raw mut CT[0], v1); } // lbTime_8000B028
                    _ => unreachable!(),
                }
                hash_all();
                c.line(op, k, ret);
            }
        }
        assert_eq!(c.done(), 2400);
    }
}
