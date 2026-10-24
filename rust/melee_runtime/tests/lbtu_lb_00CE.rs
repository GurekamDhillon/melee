//! Written by gen/lbtu/mk_driver.py from the same description as gen/lbtu/tu/lb_00CE.c.
//! src/tu/lb_00CE.rs vs the C (the whole TU, shipping contraction; regenerate the expected file
//! with `python rust/melee_runtime/gen/lbtu/run_tu.py lb_00CE`).
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables, unused_unsafe, unused_parens)]
#[path = "lbtu_harness/mod.rs"]
mod mtu_harness;
use core::ptr::null_mut;
use melee_types::*;
use melee_runtime::tu::lb_00CE as m;
use mtu_harness::*;
#[path = "lbtu_gen/lb_00CE_gen.rs"]
mod g;




unsafe fn setup() {
    let _ = 0;



}
unsafe fn hash_all() {



}
#[test]
fn lb_00CE_matches_c() {
    unsafe {
        g::install();

        g::freg_all();

        let mut c = Check::new(include_str!("data/lbtu_lb_00CE.txt"));
        for op in 0..5u32 {
            for k in 0..400u32 {
                case(op, k);
                setup();
                let mut ret = 0u32;
                match op {
                    0 => {  let v0: f32 = ((fv() % 81) as i32 - 40) as f32 / 4.0; ret = (m::expf(v0)).to_bits(); } // expf
                    1 => {  let v0: f32 = (fv() % 64) as f32 / 8.0; let v1: f32 = ((fv() % 25) as i32 - 12) as f32 / 4.0; ret = (m::powf(v0, v1)).to_bits(); } // powf
                    2 => {  let v0: i32 = (fv() % 9) as i32 - 4; let v1: i32 = (fv() % 40) as i32 - 3; ret = m::powi(v0, v1) as u32; } // powi
                    3 => {  let v0: f32 = (rf()) as f32; let v1: f32 = (rf()) as f32; ret = (m::lb_8000D008(v0, v1)).to_bits(); } // lb_8000D008
                    4 => {  let v0: f32 = (rf()) as f32; let v1: f32 = (rf()) as f32; let v2: f32 = (rf()) as f32; let v3: f32 = (rf()) as f32; let v4: f32 = (rf()) as f32; let v5: f32 = (rf()) as f32; let v6: f32 = (rf()) as f32; ret = m::lb_8000D148(v0, v1, v2, v3, v4, v5, v6) as u32; } // lb_8000D148
                    _ => unreachable!(),
                }
                hash_all();
                c.line(op, k, ret);
            }
        }
        assert_eq!(c.done(), 2000);
    }
}
