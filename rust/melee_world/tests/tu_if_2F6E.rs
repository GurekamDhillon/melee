//! Written by gen/mtu/if_drv.py from the same description as gen/mtu/tu/if_2F6E.c.
//! src/tu/if_2F6E.rs vs the C (the whole TU, shipping contraction; regenerate the expected file
//! with `python rust/melee_world/gen/mtu/run_if.py if_2F6E`).
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables, unused_unsafe)]
mod mtu_harness;
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;
use melee_world::tu::if_2F6E as m;
use mtu_harness::*;
#[path = "mtu/if_2F6E_gen.rs"]
mod g;

pub static mut MODELS: [*mut DynamicModelDesc; 8] = [null_mut(); 8];
pub static mut ARCP: *mut HSD_Archive = null_mut();
unsafe extern "C" fn cbA(a: i32) { t(7001); t(a as u32); }
unsafe extern "C" fn cbB(a: i32) { t(7002); t(a as u32); }
fn cbsel() -> usize { let q = rnd() % 3; if q == 0 { 0 } else if q == 1 { cbA as usize } else { cbB as usize } }




unsafe fn setup_extra() {
    let mut n: u32;
    let mut i: usize;

}

#[test]
fn if_2F6E_matches_c() {
    unsafe {
        g::install();
        g::preg_state();
        g::freg_all();
        freg(cbA as *const () as usize);
        freg(cbB as *const () as usize);
        let mut c = Check::new(include_str!("data/mtu_if_2F6E.txt"));
        let nc: [u32; 4] = [400, 400, 400, 400];
        case(99, 0); g::hash_init_state(); c.line(99, 0, 0);
        for op in 0..4u32 {
            for k in 0..nc[op as usize] {
                case(op, k);
                g::reset_state();
                setup_extra();
                let mut ret = 0u32;
                match op {
                    0 => {  let a0: i32 = ((0 + (rnd() % 9) as i32)) as i32; let a1: i32 = (fv()) as i32; let a2: i32 = (fv()) as i32; let a3: i32 = (fv()) as i32; let a4: Option<unsafe extern "C" fn()> = (core::mem::transmute::<usize, Event>(cbsel())) as Option<unsafe extern "C" fn()>; let a5: Option<unsafe extern "C" fn()> = (core::mem::transmute::<usize, Event>(cbsel())) as Option<unsafe extern "C" fn()>; m::ifStatus_802F6EA4(a0, a1, a2, a3, a4, a5); }
                    1 => {  let a0: *mut c_void = (cbsel() as *mut c_void) as *mut c_void; m::ifStatus_802F7034(a0); }
                    2 => {   m::ifStatus_802F7134(); }
                    3 => {   m::ifStatus_802F7220(); }
                    _ => unreachable!(),
                }
                g::hash_state();
                c.line(op, k, ret);
            }
        }
        assert_eq!(c.done(), 1601);
    }
}
