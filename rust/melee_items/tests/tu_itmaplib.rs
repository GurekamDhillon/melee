//! Written by gen/mtu/mk_driver.py from the same description as gen/mtu/tu/itmaplib.c.
//! src/tu/itmaplib.rs vs the C (the whole TU, shipping contraction; regenerate the expected file
//! with `python rust/melee_items/gen/mtu/run_tu.py itmaplib`).
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables, unused_unsafe)]
#[path = "mtu_harness_world/mod.rs"]
mod mtu_harness;
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;
use melee_items::tu::itmaplib as m;
use mtu_harness::*;
#[path = "mtu/itmaplib_gen.rs"]
mod g;



unsafe fn setup_stem() {
    m::ext::it_804D6D28 = &raw mut ICD[0];
for i in 0..8 { JO[i].flags &= !(JOBJ_USE_QUATERNION as u32); }
}

unsafe fn hash_stem() {
    g::hashg_all();
    hm(pi(m::ext::it_804D6D28));

}

#[test]
fn itmaplib_matches_c() {
    unsafe {
        g::install();
        preg_world();
        g::freg_all();
        freg_world();

        let mut c = Check::new(include_str!("data/mtu_itmaplib.txt"));
        let mut total = 0u32;
        for op in 0..37u32 {
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
                _ => unreachable!(),
            };
            for k in 0..n {
                case(op, k);
                setup_world();
                setup_stem();
                let mut ret = 0u32;
                match op {
                    0 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = &raw mut GO[(rnd() % 4) as usize];
m::it_802759DC(a0, a1);
 }
                    1 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = &raw mut GO[(rnd() % 4) as usize];
m::it_80275BC8(a0, a1);
 }
                    2 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = &raw mut EC[(rnd() % 3) as usize];
m::it_80275D5C(a0, a1);
 }
                    3 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_80275DFC(a0);
 }
                    4 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = &raw mut SP;
m::it_80275E98(a0, a1);
 }
                    5 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = &raw mut VA[(rnd() % 4) as usize];
m::it_80276100(a0, a1);
 }
                    6 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = &raw mut VA[(rnd() % 4) as usize];
m::it_80276174(a0, a1);
 }
                    7 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_80276214(a0);
 }
                    8 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_80276278(a0);
 }
                    9 => { let a0 = &raw mut IT[(rnd() % 4) as usize];
m::it_802762B0(a0);
 }
                    10 => { let a0 = &raw mut IT[(rnd() % 4) as usize];
m::it_802762BC(a0);
 }
                    11 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::it_802762D8(a0) as u32;
 }
                    12 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::it_80276308(a0) as u32;
 }
                    13 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = &raw mut VA[(rnd() % 4) as usize];
ret = m::it_80276348(a0, a1) as u32;
 }
                    14 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::it_802763B8(a0) as u32;
 }
                    15 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::it_802763E0(a0) as u32;
 }
                    16 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = &raw mut IT[(rnd() % 4) as usize].x378_itemColl;
let a2 = &raw mut VA[(rnd() % 4) as usize];
m::it_80276408(a0, a1, a2);
 }
                    17 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::it_8027649C(a0).to_bits();
 }
                    18 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = &raw mut VA[(rnd() % 4) as usize];
let a2 = &raw mut VA[(rnd() % 4) as usize];
ret = m::it_8027649C_ref(a0, a1, a2).to_bits();
 }
                    19 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = fv() as _;
m::it_802765BC(a0, a1);
 }
                    20 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = fv() as _;
m::it_80276934(a0, a1);
 }
                    21 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_80276CB8(a0);
 }
                    22 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_80276CEC(a0);
 }
                    23 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = fv() as _;
ret = m::it_80276D9C(a0, a1) as u32;
 }
                    24 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = fv() as _;
m::it_80276FC4(a0, a1);
 }
                    25 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::it_80277040(a0) as u32;
 }
                    26 => { let a0 = rw();
let a1 = &raw mut VA[(rnd() % 4) as usize].x;
ret = m::sqrtf_accurate_store(a0, a1).to_bits();
 }
                    27 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = &raw mut VA[(rnd() % 4) as usize];
m::it_8027737C(a0, a1);
 }
                    28 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::it_80277544(a0) as u32;
 }
                    29 => { let a0 = rw();
ret = m::sqrtf_accurate_sp18(a0).to_bits();
 }
                    30 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = &raw mut VA[(rnd() % 4) as usize];
m::it_802775F0(a0, a1);
 }
                    31 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::it_8027770C(a0) as u32;
 }
                    32 => { let a0 = &raw mut VA[(rnd() % 4) as usize];
let a1 = &raw mut VA[(rnd() % 4) as usize];
ret = m::product_xy(a0, a1).to_bits();
 }
                    33 => { let a0 = rw();
ret = m::sqrtf_accurate_local(a0).to_bits();
 }
                    34 => { let a0 = &raw mut VA[(rnd() % 4) as usize];
ret = m::return_sqrt_value(a0).to_bits();
 }
                    35 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::it_8027781C(a0) as u32;
 }
                    36 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = fv() as _;
m::it_80277C40(a0, a1);
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
