//! Written by gen/mtu/if_drv.py from the same description as gen/mtu/tu/ifnametag.c.
//! src/tu/ifnametag.rs vs the C (the whole TU, shipping contraction; regenerate the expected file
//! with `python rust/melee_world/gen/mtu/run_if.py ifnametag`).
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables, unused_unsafe)]
mod mtu_harness;
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;
use melee_world::tu::ifnametag as m;
use mtu_harness::*;
#[path = "mtu/ifnametag_gen.rs"]
mod g;

pub static mut SLOTB: u8 = 0;
pub static mut HEAPB: [u8; 16] = [0; 16];
pub static mut NMODELS: [*mut DynamicModelDesc; 2] = [null_mut(); 2];
pub static mut ARCP: *mut HSD_Archive = null_mut();




unsafe fn setup_extra() {
    let mut n: u32;
    let mut i: usize;
g::P_HSD_GObj[0].user_data = &raw mut SLOTB as *mut _;
    for k in 1..4usize { g::P_HSD_GObj[k].user_data = &raw mut g::P_Fighter[k & 1] as *mut _; }
    for k in 0..2usize { g::P_Fighter[k].kind = if rnd() & 1 != 0 { Ft_Kind_Mewtwo as _ } else { fv() as _ }; g::P_Fighter[k].motion_id = if rnd() & 1 != 0 { 0xEC } else { fv() as _ }; }
}

#[test]
fn ifnametag_matches_c() {
    unsafe {
        g::install();
        g::preg_state();
        g::freg_all();
        let mut c = Check::new(include_str!("data/mtu_ifnametag.txt"));
        let nc: [u32; 16] = [400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400];
        case(99, 0); g::hash_init_state(); c.line(99, 0, 0);
        for op in 0..16u32 {
            for k in 0..nc[op as usize] {
                case(op, k);
                g::reset_state();
                setup_extra();
                let mut ret = 0u32;
                match op {
                    0 => {  let a0: u8 = (rnd() % 6) as u8; let a1: u8 = ((0 + (rnd() % 5) as i32)) as u8; let a2: u8 = (fv()) as u8; let a3: u8 = (fv()) as u8; ret = m::un_802FC9B4(a0, a1, a2, a3).to_bits(); }
                    1 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(1) as *mut HSD_GObj; let a1: i32 = (fv()) as i32; m::NameTag_RenderCallback(a0, a1); }
                    2 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; let a1: i32 = (fv()) as i32; m::fn_802FCAC4(a0, a1); }
                    3 => {   m::un_802FCBA0(); }
                    4 => {  let a0: i32 = (rnd() % 6) as i32; ret = m::has_nametag(a0) as u32; }
                    5 => {  let a0: *mut HSD_GObj = (&raw mut g::P_HSD_GObj[0]) as *mut HSD_GObj; m::fn_802FCC44(a0); }
                    6 => {  let a0: f32 = (rw()) as f32; ret = m::inlineA1(a0).to_bits(); }
                    7 => {  let a0: i32 = (rnd() % 6) as i32; m::NameTag_Create(a0); }
                    8 => {  let a0: i32 = (rnd() % 6) as i32; m::un_802FD28C(a0); }
                    9 => {   m::un_802FD404(); }
                    10 => {   m::un_802FD428(); }
                    11 => {   m::un_802FD450(); }
                    12 => {   m::un_802FD45C(); }
                    13 => { for q in 0..6usize { if rnd() & 1 != 0 { m::un_804A1EE0[q] = null_mut(); } }  m::un_802FD468(); }
                    14 => {  let a0: i32 = (fv()) as i32; ret = pi(m::un_802FD4C8_inline(a0)); }
                    15 => {   m::un_802FD4C8(); }
                    _ => unreachable!(),
                }
                g::hash_state();
                c.line(op, k, ret);
            }
        }
        assert_eq!(c.done(), 6401);
    }
}
