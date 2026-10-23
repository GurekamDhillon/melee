//! Written by gen/mtu/mk_driver.py from the same description as gen/mtu/tu/itlgunray.c.
//! src/tu/itlgunray.rs vs the C (the whole TU, shipping contraction; regenerate the expected file
//! with `python rust/melee_items/gen/mtu/run_tu.py itlgunray`).
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables, unused_unsafe)]
mod mtu_harness;
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;
use melee_items::tu::itlgunray as m;
use mtu_harness::*;
#[path = "mtu/itlgunray_gen.rs"]
mod g;

static mut CUR_OP: i32 = -1;
const OP_it_80298168: i32 = 0;
const OP_it_802982AC: i32 = 1;
const OP_itLgunray_UnkMotion0_Anim: i32 = 2;
const OP_itLgunray_UnkMotion0_Phys: i32 = 3;
const OP_itLgunray_UnkMotion0_Coll: i32 = 4;
const OP_itLGunRay_Logic35_DmgDealt: i32 = 5;
const OP_itLGunRay_Logic35_Clanked: i32 = 6;
const OP_itLGunRay_Logic35_HitShield: i32 = 7;
const OP_itLGunRay_Logic35_Absorbed: i32 = 8;
const OP_itLGunRay_Logic35_Reflected: i32 = 9;
const OP_itLGunRay_Logic35_ShieldBounced: i32 = 10;
const OP_itLGunRay_Logic35_EvtUnk: i32 = 11;
static mut SPA: [[u32; 96]; 2] = [[0; 96]; 2];


unsafe fn setup_stem() {

for j in 0..2usize { for q in 0..96usize { SPA[j][q] = if rnd() & 3 == 0 { rnd() % 8 } else { rw().to_bits() }; } ART[j].x4_specialAttributes = (&raw mut SPA[j]) as *mut c_void; } for j in 0..8usize { JO[j].flags &= !(JOBJ_USE_QUATERNION as u32); } 
}

unsafe fn hash_stem() {
    g::hashg_all();

for j in 0..2usize { for q in 0..96usize { hm(SPA[j][q]); } } 
}

#[test]
fn itlgunray_matches_c() {
    unsafe {
        g::install();
        preg_world();
        g::freg_all();
        freg_world();
preg(&raw const SPA, core::mem::size_of_val(&SPA)); 
        let mut c = Check::new(include_str!("data/mtu_itlgunray.txt"));
        let mut total = 0u32;
        for op in 0..12u32 {
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
                _ => unreachable!(),
            };
            for k in 0..n {
                case(op, k);
                setup_world();
                CUR_OP = op as i32;
                setup_stem();
                let mut ret = 0u32;
                match op {
                    0 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = &raw mut VA[(rnd() % 4) as usize];
let a2 = rw();
m::it_80298168(a0, a1, a2);
 }
                    1 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = &raw mut GO[(rnd() % 4) as usize];
let a2 = &raw mut ART[(rnd() % 2) as usize];
m::it_802982AC(a0, a1, a2);
 }
                    2 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::itLgunray_UnkMotion0_Anim(a0) as u32;
 }
                    3 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::itLgunray_UnkMotion0_Phys(a0);
 }
                    4 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::itLgunray_UnkMotion0_Coll(a0) as u32;
 }
                    5 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::itLGunRay_Logic35_DmgDealt(a0) as u32;
 }
                    6 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::itLGunRay_Logic35_Clanked(a0) as u32;
 }
                    7 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::itLGunRay_Logic35_HitShield(a0) as u32;
 }
                    8 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::itLGunRay_Logic35_Absorbed(a0) as u32;
 }
                    9 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::itLGunRay_Logic35_Reflected(a0) as u32;
 }
                    10 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::itLGunRay_Logic35_ShieldBounced(a0) as u32;
 }
                    11 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = &raw mut GO[(rnd() % 4) as usize];
m::itLGunRay_Logic35_EvtUnk(a0, a1);
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
