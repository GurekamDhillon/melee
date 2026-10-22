//! Written by gen/mtu/if_drv.py from the same description as gen/mtu/tu/ifprize.c.
//! src/tu/ifprize.rs vs the C (the whole TU, shipping contraction; regenerate the expected file
//! with `python rust/melee_world/gen/mtu/run_if.py ifprize`).
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables, unused_unsafe)]
mod mtu_harness;
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;
use melee_world::tu::ifprize as m;
use mtu_harness::*;
#[path = "mtu/ifprize_gen.rs"]
mod g;

pub static mut PRZM: [*mut DynamicModelDesc; 2] = [null_mut(); 2];
pub static mut S16A: [i16; 4] = [0; 4];
pub static mut U8A: [u8; 4] = [0; 4];
pub static mut S32A: [i32; 4] = [0; 4];




unsafe fn setup_extra() {
    let mut n: u32;
    let mut i: usize;

}

#[test]
fn ifprize_matches_c() {
    unsafe {
        g::install();
        g::preg_state();
        g::freg_all();
        let mut c = Check::new(include_str!("data/mtu_ifprize.txt"));
        let nc: [u32; 12] = [400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400];
        case(99, 0); g::hash_init_state(); c.line(99, 0, 0);
        for op in 0..12u32 {
            for k in 0..nc[op as usize] {
                case(op, k);
                g::reset_state();
                setup_extra();
                let mut ret = 0u32;
                match op {
                    0 => {  let a0: i32 = ((0 + (rnd() % 71) as i32)) as i32; let a1: i32 = ((0 + (rnd() % 31) as i32)) as i32; let a2: *mut i16 = (if rnd() & 1 != 0 { &raw mut S16A[0] } else { null_mut() }) as *mut i16; let a3: *mut i16 = (if rnd() & 1 != 0 { &raw mut S16A[2] } else { null_mut() }) as *mut i16; m::un_802FE3F8(a0, a1, a2, a3); }
                    1 => {  let a0: i32 = ((0 + (rnd() % 71) as i32)) as i32; let a1: i32 = ((0 + (rnd() % 31) as i32)) as i32; let a2: *mut i16 = (if rnd() & 1 != 0 { &raw mut S16A[0] } else { null_mut() }) as *mut i16; let a3: *mut i16 = (if rnd() & 1 != 0 { &raw mut S16A[2] } else { null_mut() }) as *mut i16; m::un_802FE3F8_inner(a0, a1, a2, a3); }
                    2 => {  let a0: i32 = ((0 + (rnd() % 71) as i32)) as i32; let a1: i32 = ((0 + (rnd() % 31) as i32)) as i32; let a2: *mut i16 = (if rnd() & 1 != 0 { &raw mut S16A[0] } else { null_mut() }) as *mut i16; let a3: *mut i16 = (if rnd() & 1 != 0 { &raw mut S16A[2] } else { null_mut() }) as *mut i16; m::un_802FE3F8_noinline(a0, a1, a2, a3); }
                    3 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; let a1: i32 = (fv()) as i32; m::ifPrize_RenderGuarded(a0, a1); }
                    4 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; m::fn_802FE470(a0); }
                    5 => {   m::un_802FE6A8(); }
                    6 => {   m::un_802FE8CC(); }
                    7 => {   m::setArchive(); }
                    8 => {  let a0: *mut u8 = (&raw mut U8A[0]) as *mut u8; let a1: *mut i32 = (&raw mut S32A[0]) as *mut i32; m::un_802FE918_update_x3(a0, a1); }
                    9 => {  let a0: i32 = ((0 + (rnd() % 71) as i32)) as i32; let a1: i32 = ((0 + (rnd() % 31) as i32)) as i32; let a2: i32 = (fv()) as i32; m::un_802FE918(a0, a1, a2); }
                    10 => {  let a0: *mut c_void = g::pick_un_802FEBE0_OnEnter_arg0(0) as *mut c_void; m::ifPrize_Scene_OnEnter(a0); }
                    11 => {  let a0: *mut c_void = core::mem::zeroed(); m::ifPrize_Scene_OnExit(a0); }
                    _ => unreachable!(),
                }
                g::hash_state();
                c.line(op, k, ret);
            }
        }
        assert_eq!(c.done(), 4801);
    }
}
