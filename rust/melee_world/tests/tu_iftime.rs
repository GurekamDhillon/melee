//! Written by gen/mtu/if_drv.py from the same description as gen/mtu/tu/iftime.c.
//! src/tu/iftime.rs vs the C (the whole TU, shipping contraction; regenerate the expected file
//! with `python rust/melee_world/gen/mtu/run_if.py iftime`).
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables, unused_unsafe)]
mod mtu_harness;
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;
use melee_world::tu::iftime as m;
use mtu_harness::*;
#[path = "mtu/iftime_gen.rs"]
mod g;

pub static mut TIMM: [*mut DynamicModelDesc; 4] = [null_mut(); 4];
pub static mut CDM: [*mut DynamicModelDesc; 4] = [null_mut(); 4];
pub static mut ARCP: *mut HSD_Archive = null_mut();
pub static mut TPOS: Vec3 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };




unsafe fn setup_extra() {
    let mut n: u32;
    let mut i: usize;

}

#[test]
fn iftime_matches_c() {
    unsafe {
        g::install();
        g::preg_state();
        g::freg_all();
        let mut c = Check::new(include_str!("data/mtu_iftime.txt"));
        let nc: [u32; 15] = [400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400];

        for op in 0..15u32 {
            for k in 0..nc[op as usize] {
                case(op, k);
                g::reset_state();
                setup_extra();
                let mut ret = 0u32;
                match op {
                    0 => {   ret = m::ifTime_LoadModels() as u32; }
                    1 => {  let a0: *mut HSD_JObj = g::pick_HSD_JObj(0) as *mut HSD_JObj; let a1: u32 = (fv()) as u32; m::ifTime_SetDigit(a0, a1); }
                    2 => {  let a0: *mut HSD_JObj = g::pick_HSD_JObj(0) as *mut HSD_JObj; let a1: i32 = (fv()) as i32; let a2: i32 = ((0 + (rnd() % 100) as i32)) as i32; m::ifTime_SetTime(a0, a1, a2); }
                    3 => {   m::ifTime_HideTimers(); }
                    4 => {   m::ifTime_ShowTimers(); }
                    5 => {   ret = m::ifTime_GetCountdownSeconds() as u32; }
                    6 => {   ret = m::ifTime_GetCountdownSeconds_dontinline() as u32; }
                    7 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; let a1: i32 = (fv()) as i32; m::ifTime_RenderGuarded(a0, a1 as isize); }
                    8 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; m::ifTime_UpdateCountdown(a0); }
                    9 => {   m::ifTime_FreeCountdown(); }
                    10 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; m::ifTime_UpdateTimers(a0); }
                    11 => {   m::ifTime_CreateTimers(); }
                    12 => {   m::ifTime_Reset(); }
                    13 => {   m::ifTime_FreeTimers(); }
                    14 => {   ret = m::ifTime_IsTimerHidden() as u32; }
                    _ => unreachable!(),
                }
                g::hash_state();
                c.line(op, k, ret);
            }
        }
        assert_eq!(c.done(), 6000);
    }
}
