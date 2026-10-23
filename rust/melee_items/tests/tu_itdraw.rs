//! Written by gen/mtu/mk_driver.py from the same description as gen/mtu/tu/itdraw.c.
//! src/tu/itdraw.rs vs the C (the whole TU, shipping contraction; regenerate the expected file
//! with `python rust/melee_items/gen/mtu/run_tu.py itdraw`).
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables, unused_unsafe)]
mod mtu_harness;
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;
use melee_items::tu::itdraw as m;
use mtu_harness::*;
#[path = "mtu/itdraw_gen.rs"]
mod g;

pub static mut COB: HSD_CObj = unsafe { core::mem::zeroed() }; static mut IDXA: [u8; 16] = [0; 16]; static mut IV: [it_266F_ItemVars; 2] = unsafe { core::mem::zeroed() };

unsafe fn setup_stem() {

{ COB = core::mem::zeroed(); IV = core::mem::zeroed(); for q in 0..16usize { IDXA[q] = (rnd() & 3) as u8; } for q in 0..12usize { *(&raw mut COB.view_mtx as *mut f32).add(q) = rw(); } for q in 0..2usize { IV[q].x0 = (fv() % 5) as u16; IV[q].x4 = (&raw mut IDXA) as *mut u8; IV[q].x8 = (fv() % 5) as u16; IV[q].xC = ((&raw mut IDXA) as *mut u8).add(4); } for q in 0..4usize { IT[q].xAC8_hurtboxNum = (rnd() % 3) as u8; if rnd() % 3 == 0 { IT[q].kind = It_Kind_Unk4; } *((&raw mut IT[q].xDD4_itemVar) as *mut *mut it_266F_ItemVars) = &raw mut IV[q & 1]; } }
}

unsafe fn hash_stem() {
    g::hashg_all();


}

#[test]
fn itdraw_matches_c() {
    unsafe {
        g::install();
        preg_world();
        g::freg_all();
        freg_world();
preg(&raw const COB, core::mem::size_of_val(&COB)); preg(&raw const IDXA, 16); preg(&raw const IV, core::mem::size_of_val(&IV));
        let mut c = Check::new(include_str!("data/mtu_itdraw.txt"));
        let mut total = 0u32;
        for op in 0..10u32 {
            let (n, ft) = match op {
                0 => (300u32, false),
                1 => (300u32, false),
                2 => (300u32, false),
                3 => (300u32, false),
                4 => (300u32, false),
                5 => (300u32, false),
                6 => (300u32, false),
                7 => (300u32, false),
                8 => (300u32, false),
                9 => (300u32, false),
                _ => unreachable!(),
            };
            for k in 0..n {
                case(op, k);
                setup_world();
                setup_stem();
                let mut ret = 0u32;
                match op {
                    0 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = fv() as _;
let a2 = &raw mut VA[(rnd() % 4) as usize];
m::it_8026EB18(a0, a1, a2);
for q in 0..4usize { *((&raw mut IT[q].xDD4_itemVar) as *mut *mut it_266F_ItemVars) = null_mut(); } }
                    1 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = (fv() % 5) as u16;
let a2 = (&raw mut IDXA) as *mut u8;
m::it_8026EBC8(a0, a1, a2);
for q in 0..4usize { *((&raw mut IT[q].xDD4_itemVar) as *mut *mut it_266F_ItemVars) = null_mut(); } }
                    2 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = (fv() % 5) as u16;
let a2 = (&raw mut IDXA) as *mut u8;
m::it_8026EC54(a0, a1, a2);
for q in 0..4usize { *((&raw mut IT[q].xDD4_itemVar) as *mut *mut it_266F_ItemVars) = null_mut(); } }
                    3 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = fv() as _;
ret = m::it_8026ECE0(a0, a1) as u32;
for q in 0..4usize { *((&raw mut IT[q].xDD4_itemVar) as *mut *mut it_266F_ItemVars) = null_mut(); } }
                    4 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = fv() as _;
let a2 = &raw mut VA[(rnd() % 4) as usize];
m::it_8026EECC_inline_1(a0, a1, a2);
for q in 0..4usize { *((&raw mut IT[q].xDD4_itemVar) as *mut *mut it_266F_ItemVars) = null_mut(); } }
                    5 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = fv() as _;
let a2 = &raw mut VA[(rnd() % 4) as usize];
m::it_8026EECC_inline_2(a0, a1, a2);
for q in 0..4usize { *((&raw mut IT[q].xDD4_itemVar) as *mut *mut it_266F_ItemVars) = null_mut(); } }
                    6 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = fv() as _;
let a2 = &raw mut VA[(rnd() % 4) as usize];
m::it_8026EECC_inline_3(a0, a1, a2);
for q in 0..4usize { *((&raw mut IT[q].xDD4_itemVar) as *mut *mut it_266F_ItemVars) = null_mut(); } }
                    7 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = &raw mut VA[(rnd() % 4) as usize];
ret = pi(m::it_8026EECC_inline_0(a0, a1));
for q in 0..4usize { *((&raw mut IT[q].xDD4_itemVar) as *mut *mut it_266F_ItemVars) = null_mut(); } }
                    8 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = fv() as _;
let a2 = &raw mut VA[(rnd() % 4) as usize];
m::it_8026EECC_inline_sw(a0, a1, a2);
for q in 0..4usize { *((&raw mut IT[q].xDD4_itemVar) as *mut *mut it_266F_ItemVars) = null_mut(); } }
                    9 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = fv() as _;
m::it_8026EECC(a0, a1);
for q in 0..4usize { *((&raw mut IT[q].xDD4_itemVar) as *mut *mut it_266F_ItemVars) = null_mut(); } }
                    _ => unreachable!(),
                }
                hash_world(ft);
                hash_stem();
                c.line(op, k, ret);
                total += 1;
            }
        }
        assert_eq!(c.done(), total);
    }
}
