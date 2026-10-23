//! Written by gen/mtu/if_drv.py from the same description as gen/mtu/tu/ifhazard.c.
//! src/tu/ifhazard.rs vs the C (the whole TU, shipping contraction; regenerate the expected file
//! with `python rust/melee_world/gen/mtu/run_if.py ifhazard`).
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables, unused_unsafe)]
mod mtu_harness;
use core::ptr::null_mut;
use melee_types::*;
use melee_world::tu::ifhazard as m;
use mtu_harness::*;
#[path = "mtu/ifhazard_gen.rs"]
mod g;





unsafe fn setup_extra() {
    let mut n: u32;
    let mut i: usize;

}

#[test]
fn ifhazard_matches_c() {
    unsafe {
        g::install();
        g::preg_state();
        g::freg_all();
        let mut c = Check::new(include_str!("data/mtu_ifhazard.txt"));
        let nc: [u32; 12] = [400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400];
        for op in 0..12u32 {
            for k in 0..nc[op as usize] {
                case(op, k);
                g::reset_state();
                setup_extra();
                let mut ret = 0u32;
                match op {
                    0 => {  let a0: i32 = (fv()) as i32; m::un_802FD604(a0); }
                    1 => {   m::un_802FD65C(); }
                    2 => {   m::un_802FD668(); }
                    3 => {   m::un_802FD674(); }
                    4 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; m::fn_802FD680(a0); }
                    5 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; let a1: i32 = (fv()) as i32; m::fn_802FD6CC(a0, a1); }
                    6 => {   m::un_802FD704(); }
                    7 => {  let a0: *mut DynamicModelDesc = g::pick_DynamicModelDesc(0) as *mut DynamicModelDesc; let a1: i32 = (fv()) as i32; m::un_802FD708(a0, a1); }
                    8 => {  let a0: *mut DynamicModelDesc = g::pick_DynamicModelDesc(0) as *mut DynamicModelDesc; m::un_802FD8A0(a0); }
                    9 => {  let a0: *mut DynamicModelDesc = g::pick_DynamicModelDesc(0) as *mut DynamicModelDesc; m::un_802FD8C4(a0); }
                    10 => {  let a0: *mut DynamicModelDesc = g::pick_DynamicModelDesc(0) as *mut DynamicModelDesc; m::un_802FD8E8(a0); }
                    11 => {   m::un_802FD90C(); }
                    _ => unreachable!(),
                }
                g::hash_state();
                c.line(op, k, ret);
            }
        }
        assert_eq!(c.done(), 4800);
    }
}
