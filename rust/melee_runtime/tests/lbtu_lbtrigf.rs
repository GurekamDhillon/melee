//! Written by gen/lbtu/mk_driver.py from the same description as gen/lbtu/tu/lbtrigf.c.
//! src/tu/lbtrigf.rs vs the C (the whole TU, shipping contraction; regenerate the expected file
//! with `python rust/melee_runtime/gen/lbtu/run_tu.py lbtrigf`).
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables, unused_unsafe, unused_parens)]
#[path = "lbtu_harness/mod.rs"]
mod mtu_harness;
use core::ptr::null_mut;
use melee_types::*;
use melee_runtime::tu::lbtrigf as m;
use mtu_harness::*;
#[path = "lbtu_gen/lbtrigf_gen.rs"]
mod g;




unsafe fn setup() {
    let _ = 0;



}
unsafe fn hash_all() {



}
#[test]
fn lbtrigf_matches_c() {
    unsafe {
        g::install();

        g::freg_all();

        let mut c = Check::new(include_str!("data/lbtu_lbtrigf.txt"));
        for op in 0..7u32 {
            for k in 0..400u32 {
                case(op, k);
                setup();
                let mut ret = 0u32;
                match op {
                    0 => {  let v0: f32 = (rf()) as f32; let v1: f32 = (rf()) as f32; ret = (m::atan2f(v0, v1)).to_bits(); } // atan2f
                    1 => {  let v0: f32 = (rf()) as f32; ret = (m::acosf(v0)).to_bits(); } // acosf
                    2 => {  let v0: f32 = (rf()) as f32; ret = (m::asinf(v0)).to_bits(); } // asinf
                    3 => {  let v0: f32 = (rf()) as f32; ret = (m::lb_sqrtf(v0)).to_bits(); } // lb_sqrtf
                    4 => {  let v0: f32 = (rf()) as f32; let v1: f32 = (rf()) as f32; let v2: f32 = (rf()) as f32; ret = (m::lbTrigf_Fmadds(v0, v1, v2)).to_bits(); } // lbTrigf_Fmadds
                    5 => {  let v0: f32 = (rf()) as f32; let v1: f32 = (rf()) as f32; let v2: f32 = (rf()) as f32; ret = (m::lbTrigf_Fnmsubs(v0, v1, v2)).to_bits(); } // lbTrigf_Fnmsubs
                    6 => {  let v0: f32 = (rf()) as f32; ret = (m::atanf(v0)).to_bits(); } // atanf
                    _ => unreachable!(),
                }
                hash_all();
                c.line(op, k, ret);
            }
        }
        assert_eq!(c.done(), 2800);
    }
}
