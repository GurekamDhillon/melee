//! Written by gen/mtu/mk_driver.py from the same description as gen/mtu/tu/itdrop.c.
//! src/tu/itdrop.rs vs the C (the whole TU, shipping contraction; regenerate the expected file
//! with `python rust/melee_items/gen/mtu/run_tu.py itdrop`).
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables, unused_unsafe)]
mod mtu_harness;
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;
use melee_items::tu::itdrop as m;
use mtu_harness::*;
#[path = "mtu/itdrop_gen.rs"]
mod g;

static mut A1: it_8026F3D4_arg1_t = unsafe { core::mem::zeroed() };

unsafe fn setup_stem() {
    m::ext::it_804D6D28 = &raw mut ICD[0];
A1 = core::mem::zeroed();
}

unsafe fn hash_stem() {
    g::hashg_all();
    hm(pi(m::ext::it_804D6D28));
hm(A1.x0 as u32); hm(A1.x4 as u32); hm(pi(A1.x8));
}

#[test]
fn itdrop_matches_c() {
    unsafe {
        g::install();
        preg_world();
        g::freg_all();
        freg_world();
preg(&raw const A1, core::mem::size_of_val(&A1));
        let mut c = Check::new(include_str!("data/mtu_itdrop.txt"));
        let mut total = 0u32;
        for op in 0..8u32 {
            let (n, ft) = match op {
                0 => (300u32, false),
                1 => (300u32, false),
                2 => (300u32, false),
                3 => (300u32, false),
                4 => (300u32, false),
                5 => (300u32, false),
                6 => (300u32, false),
                7 => (300u32, false),
                _ => unreachable!(),
            };
            for k in 0..n {
                case(op, k);
                setup_world();
                setup_stem();
                let mut ret = 0u32;
                match op {
                    0 => { 
ret = m::it_8026F3AC() as u32;
 }
                    1 => { let a0 = &raw mut IT[(rnd() % 4) as usize];
let a1 = fv() as _;
ret = m::it_8026F3D4_check_kind(a0, a1) as u32;
 }
                    2 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = if rnd() & 1 != 0 { &raw mut A1 } else { null_mut() };
let a2 = (fv() % 9) as i32;
let a3 = fv() as _;
ret = m::it_8026F3D4(a0, a1, a2, a3) as u32;
 }
                    3 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = &raw mut VA[(rnd() % 4) as usize];
let a2 = fv() as _;
m::it_8026F53C(a0, a1, a2);
 }
                    4 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = fv() as _;
let a2 = &raw mut VA[(rnd() % 4) as usize];
ret = pi(m::it_8026F5C8(a0, a1, a2));
 }
                    5 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = &raw mut VA[(rnd() % 4) as usize];
let a2 = &raw mut VA[(rnd() % 4) as usize];
let a3 = fv() as _;
ret = pi(m::it_8026F6BC(a0, a1, a2, a3));
 }
                    6 => { let a0 = &raw mut VA[(rnd() % 4) as usize];
let a1 = &raw mut VA[(rnd() % 4) as usize];
let a2 = fv() as _;
ret = pi(m::it_8026F7C8(a0, a1, a2));
 }
                    7 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = &raw mut VA[(rnd() % 4) as usize];
let a2 = &raw mut VA[(rnd() % 4) as usize];
let a3 = fv() as _;
ret = m::it_8026F8B4(a0, a1, a2, a3) as u32;
 }
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
