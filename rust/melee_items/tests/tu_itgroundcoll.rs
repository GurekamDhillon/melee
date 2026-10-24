//! Written by gen/mtu/mk_driver.py from the same description as gen/mtu/tu/itgroundcoll.c.
//! src/tu/itgroundcoll.rs vs the C (the whole TU, shipping contraction; regenerate the expected file
//! with `python rust/melee_items/gen/mtu/run_tu.py itgroundcoll`).
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables, unused_unsafe)]
#[path = "mtu_harness_world/mod.rs"]
mod mtu_harness;
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;
use melee_items::tu::itgroundcoll as m;
use mtu_harness::*;
#[path = "mtu/itgroundcoll_gen.rs"]
mod g;



unsafe fn setup_stem() {
    m::ext::it_804D6D28 = &raw mut ICD[0];

}

unsafe fn hash_stem() {
    g::hashg_all();
    hm(pi(m::ext::it_804D6D28));

}

#[test]
fn itgroundcoll_matches_c() {
    unsafe {
        g::install();
        preg_world();
        g::freg_all();
        freg_world();

        let mut c = Check::new(include_str!("data/mtu_itgroundcoll.txt"));
        let mut total = 0u32;
        for op in 0..45u32 {
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
                22 => (300u32, false),
                23 => (300u32, false),
                24 => (300u32, false),
                25 => (300u32, false),
                26 => (300u32, false),
                27 => (300u32, false),
                28 => (300u32, false),
                29 => (300u32, false),
                30 => (300u32, false),
                31 => (300u32, false),
                32 => (300u32, false),
                33 => (300u32, false),
                34 => (300u32, false),
                35 => (300u32, false),
                36 => (300u32, false),
                37 => (300u32, false),
                38 => (300u32, false),
                39 => (300u32, false),
                40 => (300u32, false),
                41 => (300u32, false),
                42 => (300u32, false),
                43 => (300u32, false),
                44 => (300u32, false),
                _ => unreachable!(),
            };
            for k in 0..n {
                case(op, k);
                setup_world();
                setup_stem();
                let mut ret = 0u32;
                match op {
                    0 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::it_8026D564(a0) as u32;
 }
                    1 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::it_8026D5CC(a0) as u32;
 }
                    2 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::it_8026D604(a0) as u32;
 }
                    3 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = unsafe { core::mem::transmute::<usize, _>(pick_cb(1)) };
m::it_8026D62C(a0, a1);
 }
                    4 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = unsafe { core::mem::transmute::<usize, _>(pick_cb(1)) };
m::it_8026D6F4(a0, a1);
 }
                    5 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = unsafe { core::mem::transmute::<usize, _>(pick_cb(1)) };
m::it_8026D78C(a0, a1);
 }
                    6 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_8026D82C(a0);
 }
                    7 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = unsafe { core::mem::transmute::<usize, _>(pick_cb(1)) };
ret = m::it_8026D8A4(a0, a1) as u32;
 }
                    8 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = unsafe { core::mem::transmute::<usize, _>(pick_cb(1)) };
m::it_8026D938(a0, a1);
 }
                    9 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::it_8026D9A0(a0) as u32;
 }
                    10 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::it_8026DA08(a0) as u32;
 }
                    11 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::it_8026DA70(a0) as u32;
 }
                    12 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::it_8026DAA8(a0) as u32;
 }
                    13 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::it_8026DB40(a0) as u32;
 }
                    14 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::it_8026DBC8(a0) as u32;
 }
                    15 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::it_8026DC24(a0) as u32;
 }
                    16 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::it_8026DD5C(a0) as u32;
 }
                    17 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::it_8026DDFC(a0) as u32;
 }
                    18 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::it_8026DE98(a0) as u32;
 }
                    19 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::it_8026DF34(a0) as u32;
 }
                    20 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::it_8026DFB0(a0) as u32;
 }
                    21 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::it_8026E058(a0) as u32;
 }
                    22 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_8026E0F4(a0);
 }
                    23 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::it_8026E_inline(a0) as u32;
 }
                    24 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::it_8026E15C_inline1(a0) as u32;
 }
                    25 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::it_8026E15C_inline2(a0) as u32;
 }
                    26 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = unsafe { core::mem::transmute::<usize, _>(pick_cb(1)) };
m::it_8026E15C(a0, a1);
 }
                    27 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::it_8026E248_inline(a0) as u32;
 }
                    28 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::it_8026DE98_outline(a0) as u32;
 }
                    29 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::it_8026DE98_outline2(a0) as u32;
 }
                    30 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::it_8026DE98_outline3(a0) as u32;
 }
                    31 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::it_8026DE98_outline4(a0) as u32;
 }
                    32 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = unsafe { core::mem::transmute::<usize, _>(pick_cb(1)) };
m::it_8026E248(a0, a1);
 }
                    33 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::it_8026E32C_inline(a0) as u32;
 }
                    34 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = unsafe { core::mem::transmute::<usize, _>(pick_cb(1)) };
ret = m::it_8026E32C(a0, a1) as u32;
 }
                    35 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = unsafe { core::mem::transmute::<usize, _>(pick_cb(1)) };
m::it_8026E414(a0, a1);
 }
                    36 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = unsafe { core::mem::transmute::<usize, _>(pick_cb(1)) };
m::it_8026E4D0(a0, a1);
 }
                    37 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = unsafe { core::mem::transmute::<usize, _>(pick_cb(1)) };
m::it_8026E5A0(a0, a1);
 }
                    38 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = unsafe { core::mem::transmute::<usize, _>(pick_cb(1)) };
m::it_8026E664(a0, a1);
 }
                    39 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = unsafe { core::mem::transmute::<usize, _>(pick_cb(1)) };
m::it_8026E71C(a0, a1);
 }
                    40 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = unsafe { core::mem::transmute::<usize, _>(pick_cb(1)) };
m::it_8026E7E0(a0, a1);
 }
                    41 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = unsafe { core::mem::transmute::<usize, _>(pick_cb(1)) };
let a2 = unsafe { core::mem::transmute::<usize, _>(pick_cb(1)) };
m::it_8026E8C4(a0, a1, a2);
 }
                    42 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = &raw mut VA[(rnd() % 4) as usize];
let a2 = &raw mut VA[(rnd() % 4) as usize];
let a3 = &raw mut VA[(rnd() % 4) as usize];
ret = m::it_8026E9A4(a0, a1, a2, a3) as u32;
 }
                    43 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = &raw mut VA[(rnd() % 4) as usize];
let a2 = &raw mut VA[(rnd() % 4) as usize];
let a3 = &raw mut VA[(rnd() % 4) as usize];
let a4 = (&raw mut IA) as *mut i32;
let a5 = (&raw mut UA) as *mut u32;
ret = m::it_8026EA20(a0, a1, a2, a3, a4, a5) as u32;
 }
                    44 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = &raw mut VA[(rnd() % 4) as usize];
let a2 = &raw mut VA[(rnd() % 4) as usize];
let a3 = &raw mut VA[(rnd() % 4) as usize];
let a4 = (&raw mut IA) as *mut i32;
let a5 = (&raw mut UA) as *mut u32;
let a6 = fv() as _;
ret = m::it_8026EA9C(a0, a1, a2, a3, a4, a5, a6) as u32;
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
