//! Written by gen/mtu/mk_driver.py from the same description as gen/mtu/tu/ithitbox.c.
//! src/tu/ithitbox.rs vs the C (the whole TU, shipping contraction; regenerate the expected file
//! with `python rust/melee_items/gen/mtu/run_tu.py ithitbox`).
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables, unused_unsafe)]
mod mtu_harness;
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;
use melee_items::tu::ithitbox as m;
use mtu_harness::*;
#[path = "mtu/ithitbox_gen.rs"]
mod g;



unsafe fn setup_stem() {

}

unsafe fn hash_stem() {
    g::hashg_all();

}

#[test]
fn ithitbox_matches_c() {
    unsafe {
        g::install();
        preg_world();
        g::freg_all();
        freg_world();

        let mut c = Check::new(include_str!("data/mtu_ithitbox.txt"));
        let mut total = 0u32;
        for op in 0..22u32 {
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
                10 => (300u32, false),
                11 => (300u32, false),
                12 => (300u32, false),
                13 => (300u32, false),
                14 => (300u32, false),
                15 => (300u32, false),
                16 => (300u32, false),
                17 => (300u32, false),
                18 => (300u32, false),
                19 => (300u32, false),
                20 => (300u32, false),
                21 => (300u32, false),
                _ => unreachable!(),
            };
            for k in 0..n {
                case(op, k);
                setup_world();
                setup_stem();
                let mut ret = 0u32;
                match op {
                    0 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_802753DC(a0);
 }
                    1 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_80275414(a0);
 }
                    2 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_8027542C(a0);
 }
                    3 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_80275444(a0);
 }
                    4 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_80275474(a0);
 }
                    5 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_802754A4(a0);
 }
                    6 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_802754BC(a0);
 }
                    7 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_802754D4(a0);
 }
                    8 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_80275504(a0);
 }
                    9 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = rw();
m::it_80275534(a0, a1);
 }
                    10 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = (fv() & 3) as i32;
let a2 = rw();
m::it_80275594(a0, a1, a2);
 }
                    11 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = rw();
m::it_802755C0(a0, a1);
 }
                    12 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = rw();
m::it_80275640(a0, a1);
 }
                    13 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_802756D0(a0);
 }
                    14 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_802756E0(a0);
 }
                    15 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = (fv() & 3) as i32;
m::it_8027570C(a0, a1);
 }
                    16 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = (fv() & 3) as i32;
m::it_8027572C(a0, a1);
 }
                    17 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_8027574C(a0);
 }
                    18 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_80275788(a0);
 }
                    19 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = &raw mut VA[(rnd() % 4) as usize];
let a2 = &raw mut VA[(rnd() % 4) as usize];
let a3 = (fv() & 3) as i32;
m::it_80275820(a0, a1, a2, a3);
 }
                    20 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::it_80275870(a0) as u32;
 }
                    21 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::it_802758D4(a0).to_bits();
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
