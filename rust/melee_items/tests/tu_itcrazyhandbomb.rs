//! Written by gen/mtu/mk_driver.py from the same description as gen/mtu/tu/itcrazyhandbomb.c.
//! src/tu/itcrazyhandbomb.rs vs the C (the whole TU, shipping contraction; regenerate the expected file
//! with `python rust/melee_items/gen/mtu/run_tu.py itcrazyhandbomb`).
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables, unused_unsafe)]
#[path = "mtu_w/mod.rs"]
mod mtu_harness;
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;
use melee_items::tu::itcrazyhandbomb as m;
use mtu_harness::*;
#[path = "mtu/itcrazyhandbomb_gen.rs"]
mod g;

static mut SATTR: [[u32; 3]; 2] = [[0; 3]; 2];

unsafe fn setup_stem() {

{ for q in 0..2usize { ART[q].x4_specialAttributes = (&raw mut SATTR[q]) as *mut c_void; for w in 0..3usize { *(&raw mut SATTR[q] as *mut f32).add(w) = rw(); } } }
}

unsafe fn hash_stem() {
    g::hashg_all();

{ for q in 0..2usize { for w in 0..3usize { hm(SATTR[q][w]); } } }
}

#[test]
fn itcrazyhandbomb_matches_c() {
    unsafe {
        g::install();
        preg_world();
        g::freg_all();
        freg_world();
preg(&raw const SATTR, core::mem::size_of_val(&SATTR));
        let mut c = Check::new(include_str!("data/mtu_itcrazyhandbomb.txt"));
        let mut total = 0u32;
        for op in 0..15u32 {
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
                _ => unreachable!(),
            };
            for k in 0..n {
                case(op, k);
                setup_world();
                setup_stem();
                let mut ret = 0u32;
                match op {
                    0 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = &raw mut VA[(rnd() % 4) as usize];
let a2 = &raw mut VA[(rnd() % 4) as usize];
let a3 = fv() as _;
let a4 = rw();
m::it_802F0F6C(a0, a1, a2, a3, a4);
 }
                    1 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_802F1030(a0);
 }
                    2 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = &raw mut GO[(rnd() % 4) as usize];
m::itCrazyHandBomb_Logic86_EvtUnk(a0, a1);
 }
                    3 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::itCrazyHandBomb_Logic86_DmgDealt(a0) as u32;
 }
                    4 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::itCrazyHandBomb_Logic86_Reflected(a0) as u32;
 }
                    5 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_802F10F8(a0);
 }
                    6 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::itCrazyhandbomb_UnkMotion0_Anim(a0) as u32;
 }
                    7 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::itCrazyhandbomb_UnkMotion0_Phys(a0);
 }
                    8 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::itCrazyhandbomb_UnkMotion0_Coll(a0) as u32;
 }
                    9 => { 
m::it_802F1340();
 }
                    10 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_802F1344(a0);
 }
                    11 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::itCrazyhandbomb_UnkMotion1_Anim(a0) as u32;
 }
                    12 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::itCrazyhandbomb_UnkMotion1_Phys(a0);
 }
                    13 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::itCrazyhandbomb_UnkMotion1_Coll(a0) as u32;
 }
                    14 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_802F13B0(a0);
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
