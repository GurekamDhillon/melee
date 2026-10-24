//! Written by gen/mtu/mk_driver.py from the same description as gen/mtu/tu/itzako.c.
//! src/tu/itzako.rs vs the C (the whole TU, shipping contraction; regenerate the expected file
//! with `python rust/melee_items/gen/mtu/run_tu.py itzako`).
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables, unused_unsafe)]
#[path = "mtu_w/mod.rs"]
mod mtu_harness;
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;
use melee_items::tu::itzako as m;
use mtu_harness::*;
#[path = "mtu/itzako_gen.rs"]
mod g;

static mut D40: it_804D6D40_t = unsafe { core::mem::zeroed() }; static mut FCD: ftCommonData = unsafe { core::mem::zeroed() };

unsafe fn setup_stem() {
    m::ext::it_804D6D28 = &raw mut ICD[0];
{ D40 = core::mem::zeroed(); FCD = core::mem::zeroed(); g::fill_it_804D6D40_t(&mut D40); g::fill_ftCommonData(&mut FCD); m::ext::it_804D6D40 = &raw mut D40; m::ext::p_ftCommonData = &raw mut FCD; D40.x0 = (fv() % 8) as i32; D40.x14 = (fv() % 6) as f32; D40.x18 = (fv() % 8) as f32; for q in 0..8usize { JO[q].flags &= !(JOBJ_USE_QUATERNION as u32); } }
}

unsafe fn hash_stem() {
    g::hashg_all();
    hm(pi(m::ext::it_804D6D28));
g::hash_it_804D6D40_t(&D40);
}

#[test]
fn itzako_matches_c() {
    unsafe {
        g::install();
        preg_world();
        g::freg_all();
        freg_world();
preg(&raw const D40, core::mem::size_of_val(&D40)); preg(&raw const FCD, core::mem::size_of_val(&FCD));
        let mut c = Check::new(include_str!("data/mtu_itzako.txt"));
        let mut total = 0u32;
        for op in 0..26u32 {
            let (n, ft) = match op {
                0 => (300u32, false),
                1 => (300u32, false),
                2 => (300u32, false),
                3 => (300u32, false),
                4 => (300u32, false),
                5 => (1500u32, false),
                6 => (300u32, false),
                7 => (300u32, false),
                8 => (300u32, false),
                9 => (300u32, false),
                10 => (300u32, false),
                11 => (300u32, false),
                12 => (300u32, false),
                13 => (300u32, false),
                14 => (300u32, false),
                15 => (6000u32, false),
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
                _ => unreachable!(),
            };
            for k in 0..n {
                case(op, k);
                setup_world();
                setup_stem();
                let mut ret = 0u32;
                match op {
                    0 => { let a0 = fv() as _;
let a1 = if rnd() & 3 != 0 { &raw mut VA[(rnd() % 4) as usize] } else { null_mut() };
let a2 = &raw mut JO[(rnd() % 8) as usize];
let a3 = if rnd() & 3 != 0 { &raw mut VA[(rnd() % 4) as usize] } else { null_mut() };
let a4 = (fv() % 3) as i32;
ret = pi(m::it_8027B5B0(a0, a1, a2, a3, a4));
for q in 0..4usize { let pp = (&raw mut IT[q].xDD4_itemVar) as *mut *mut c_void; hm(pi(*pp)); *pp = null_mut(); } }
                    1 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_8027B730(a0);
 }
                    2 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = &raw mut VA[(rnd() % 4) as usize];
ret = m::it_8027B798(a0, a1) as u32;
 }
                    3 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = fv() as _;
m::it_8027B964(a0, a1);
 }
                    4 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = &raw mut VA[(rnd() % 4) as usize];
m::it_8027BA54(a0, a1);
 }
                    5 => { let a0 = &raw mut VA[(rnd() % 4) as usize];
let a1 = &raw mut VA[(rnd() % 4) as usize];
m::it_8027BB1C(a0, a1);
 }
                    6 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = fv() as _;
let a2 = rw() as f64;
let a3 = rw();
m::it_8027BBF4(a0, a1, a2, a3);
 }
                    7 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = rw();
let a2 = rw();
m::it_8027C0A8(a0, a1, a2);
 }
                    8 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = rw();
let a2 = rw();
m::it_8027C0CC(a0, a1, a2);
 }
                    9 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = if rnd() & 3 != 0 { &raw mut VA[(rnd() % 4) as usize] } else { null_mut() };
let a2 = rw() as f64;
let a3 = rw();
m::it_8027C0F0(a0, a1, a2, a3);
 }
                    10 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = rw();
m::it_8027C56C(a0, a1);
 }
                    11 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::it_8027C794(a0) as u32;
 }
                    12 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::it_8027C79C(a0) as u32;
 }
                    13 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = unsafe { core::mem::transmute::<usize, _>(pick_fn(2)) };
ret = m::it_8027C824(a0, a1) as u32;
 }
                    14 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_2725_Logic9_Destroyed(a0);
 }
                    15 => { let a0 = &raw mut VA[(rnd() % 4) as usize];
let a1 = &raw mut VA[(rnd() % 4) as usize];
let a2 = rw();
m::it_8027C8D0(a0, a1, a2);
 }
                    16 => { let a0 = &raw mut IT[(rnd() % 4) as usize];
m::it_8027C9D8(a0);
 }
                    17 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::it_8027CA7C(a0) as u32;
 }
                    18 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_8027CAD8(a0);
 }
                    19 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_8027CB3C(a0);
 }
                    20 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_8027CBA4(a0);
 }
                    21 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::it_8027CBFC(a0).to_bits();
 }
                    22 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = pi(m::it_8027CC88(a0));
 }
                    23 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_8027CE18(a0);
 }
                    24 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_8027CE44(a0);
 }
                    25 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = &raw mut GO[(rnd() % 4) as usize];
let a2 = (&raw mut IT[(rnd() % 4) as usize]) as *mut c_void;
m::it_8027CE64(a0, a1, a2);
for q in 0..4usize { let pp = (&raw mut IT[q].xDD4_itemVar) as *mut *mut c_void; hm(pi(*pp)); *pp = null_mut(); } }
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
