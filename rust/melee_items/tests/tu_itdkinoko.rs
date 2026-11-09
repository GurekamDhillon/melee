//! Written by gen/mtu/mk_driver.py from the same description as gen/mtu/tu/itdkinoko.c.
//! src/tu/itdkinoko.rs vs the C (the whole TU, shipping contraction; regenerate the expected file
//! with `python rust/melee_items/gen/mtu/run_tu.py itdkinoko`).
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables, unused_unsafe)]
#[path = "mtu_w/mod.rs"]
mod mtu_harness;
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;
use melee_items::tu::itdkinoko as m;
use mtu_harness::*;
#[path = "mtu/itdkinoko_gen.rs"]
mod g;

static mut SATTR: [[u32; 4]; 2] = [[0; 4]; 2];

unsafe fn setup_stem() {

{ for q in 0..2usize { ART[q].x4_specialAttributes = (&raw mut SATTR[q]) as *mut _; for w in 0..4usize { *(&raw mut SATTR[q] as *mut f32).add(w) = rw(); } } }
}

unsafe fn hash_stem() {
    g::hashg_all();

{ for q in 0..2usize { for w in 0..4usize { hm(SATTR[q][w]); } } }
}

#[test]
fn itdkinoko_matches_c() {
    unsafe {
        g::install();
        preg_world();
        g::freg_all();
        freg_world();
preg(&raw const SATTR, core::mem::size_of_val(&SATTR));
        let mut c = Check::new(include_str!("data/mtu_itdkinoko.txt"));
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
                setup_stem();
                let mut ret = 0u32;
                match op {
                    0 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::itDKinoko_Logic27_Spawned(a0);
 }
                    1 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_80293A70(a0);
 }
                    2 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::itDkinoko_UnkMotion0_Anim(a0) as u32;
 }
                    3 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::itDkinoko_UnkMotion0_Phys(a0);
 }
                    4 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::itDkinoko_UnkMotion0_Coll(a0) as u32;
 }
                    5 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_80293BE8(a0);
 }
                    6 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_80293C10(a0);
 }
                    7 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::itDkinoko_UnkMotion1_Anim(a0) as u32;
 }
                    8 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::itDkinoko_UnkMotion1_Phys(a0);
 }
                    9 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::itDkinoko_UnkMotion1_Coll(a0) as u32;
 }
                    10 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::itDKinoko_Logic27_DmgDealt(a0) as u32;
 }
                    11 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = &raw mut GO[(rnd() % 4) as usize];
m::itDKinoko_Logic27_EvtUnk(a0, a1);
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
