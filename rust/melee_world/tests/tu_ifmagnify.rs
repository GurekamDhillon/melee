//! Written by gen/mtu/if_drv.py from the same description as gen/mtu/tu/ifmagnify.c.
//! src/tu/ifmagnify.rs vs the C (the whole TU, shipping contraction; regenerate the expected file
//! with `python rust/melee_world/gen/mtu/run_if.py ifmagnify`).
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables, unused_unsafe)]
mod mtu_harness;
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;
use melee_world::tu::ifmagnify as m;
use mtu_harness::*;
#[path = "mtu/ifmagnify_gen.rs"]
mod g;

pub static mut GCOL: [_GXColor; 9] = unsafe { core::mem::zeroed() };
pub static mut STG: [f32; 4] = [0.0; 4];
pub static mut VEC2A: [Vec2; 2] = unsafe { core::mem::zeroed() };
pub static mut V3A: Vec3 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
pub static mut NULLRET: i32 = 0;
pub static mut PMDL: [*mut DynamicModelDesc; 2] = [null_mut(); 2];
pub static mut HEAPB: [u8; 16] = [0; 16];
pub static mut ARCP: *mut HSD_Archive = null_mut();
fn rcol() -> _GXColor { let r = fv() as u8; let g = fv() as u8; let b = fv() as u8; let a = fv() as u8; _GXColor { r, g, b, a } }
fn view_bits() -> i32 { const K: [u32; 3] = [0x3FAAAAAB, 0x3FE38E39, 0x40000000]; let q = fv() & 3; (if q < 3 { K[q as usize] } else { fv() }) as i32 }




unsafe fn setup_extra() {
    let mut n: u32;
    let mut i: usize;

}

#[test]
fn ifmagnify_matches_c() {
    unsafe {
        g::install();
        g::preg_state();
        g::freg_all();
        let mut c = Check::new(include_str!("data/mtu_ifmagnify.txt"));
        let nc: [u32; 20] = [400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400];
        case(99, 0); g::hash_init_state(); c.line(99, 0, 0);
        for op in 0..20u32 {
            for k in 0..nc[op as usize] {
                case(op, k);
                g::reset_state();
                setup_extra();
                let mut ret = 0u32;
                match op {
                    0 => {   ret = m::ifMagnify_ViewAspect().to_bits(); }
                    1 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; ret = m::ifMagnify_VisualOffscreen(a0) as u32; }
                    2 => {   ret = m::ifMagnify_IsHUDVisible() as u32; }
                    3 => {  let a0: *mut _GXColor = (&raw mut GCOL[0]) as *mut _GXColor; let a1: i32 = (rnd() % 6) as i32; m::ifMagnify_GetPlayerColor(a0, a1); }
                    4 => {  let a0: i32 = (rnd() % 6) as i32; ret = m::ifMagnify_802FB6E8(a0) as u32; }
                    5 => {  let a0: *mut ifMagnifyPlayer = (&raw mut m::ifMagnify_804A1DE0.player[0]) as *mut ifMagnifyPlayer; let a1: *mut Vec2 = (&raw mut VEC2A[0]) as *mut Vec2; let a2: *mut Vec2 = (&raw mut VEC2A[1]) as *mut Vec2; ret = pi(m::ifMagnify_802FB73C(a0, a1, a2)); }
                    6 => { g::P_HSD_GObj[0].user_data = &raw mut m::ifMagnify_804A1DE0.player[(rnd() % 6) as usize] as *mut _; let a0: *mut HSD_GObj = (&raw mut g::P_HSD_GObj[0]) as *mut HSD_GObj; let a1: i32 = ((0 + (rnd() % 3) as i32)) as i32; m::ifMagnify_802FB8C0(a0, a1 as isize); }
                    7 => {  let a0: *mut _GXColor = (&raw mut GCOL[0]) as *mut _GXColor; let a1: *mut Vec = (&raw mut V3A) as *mut Vec; m::ifMagnify_GetCornerColors(a0, a1); }
                    8 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; m::ifMagnify_802FBBDC(a0); }
                    9 => {   m::ifMagnify_802FC3BC(); }
                    10 => {  let a0: i32 = (rnd() % 6) as i32; m::ifMagnify_802FC3C0(a0); }
                    11 => {   m::ifMagnify_802FC618(); }
                    12 => {   m::ifMagnify_802FC750(); }
                    13 => { NULLRET = 1; let a0: *mut ifMagnify = (&raw mut m::ifMagnify_804A1DE0) as *mut ifMagnify; m::ifMagnify_802FC7C0(a0); }
                    14 => {   m::ifMagnify_802FC870(); }
                    15 => {   m::ifMagnify_802FC8E8(); }
                    16 => {   m::ifMagnify_802FC940(); }
                    17 => {   m::ifMagnify_LogicDisarm(); }
                    18 => {   m::ifMagnify_UpdateLogicOffscreen(); }
                    19 => {  let a0: i32 = (rnd() % 6) as i32; ret = m::ifMagnify_802FC998(a0) as u32; }
                    _ => unreachable!(),
                }
                g::hash_state();
                c.line(op, k, ret);
            }
        }
        assert_eq!(c.done(), 8001);
    }
}
