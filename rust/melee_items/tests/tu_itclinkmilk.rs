//! Written by gen/mtu/mk_driver.py from the same description as gen/mtu/tu/itclinkmilk.c.
//! src/tu/itclinkmilk.rs vs the C (the whole TU, shipping contraction; regenerate the expected file
//! with `python rust/melee_items/gen/mtu/run_tu.py itclinkmilk`).
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables, unused_unsafe)]
#[path = "mtu_w/mod.rs"]
mod mtu_harness;
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;
use melee_items::tu::itclinkmilk as m;
use mtu_harness::*;
#[path = "mtu/itclinkmilk_gen.rs"]
mod g;



unsafe fn setup_stem() {

{ for q in 0..4usize { let r = rnd(); let p: *mut c_void = if r & 3 == 0 { null_mut() } else { &raw mut GO[4 + ((r >> 8) % 3) as usize] as *mut c_void }; *((&raw mut IT[q].xDD4_itemVar) as *mut *mut c_void) = p; IT[q].owner = if r & 16 != 0 { p as *mut HSD_GObj } else { pick_fgo() }; } }
}

unsafe fn hash_stem() {
    g::hashg_all();


}

#[test]
fn itclinkmilk_matches_c() {
    unsafe {
        g::install();
        preg_world();
        g::freg_all();
        freg_world();

        let mut c = Check::new(include_str!("data/mtu_itclinkmilk.txt"));
        let mut total = 0u32;
        for op in 0..9u32 {
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
let a2 = fv() as _;
let a3 = rw();
ret = pi(m::it_802C8B28(a0, a1, a2, a3));
for q in 0..4usize { let pp = (&raw mut IT[q].xDD4_itemVar) as *mut *mut c_void; hm(pi(*pp)); *pp = null_mut(); } }
                    1 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_802C8C34(a0);
for q in 0..4usize { let pp = (&raw mut IT[q].xDD4_itemVar) as *mut *mut c_void; hm(pi(*pp)); *pp = null_mut(); } }
                    2 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_2725_Logic80_PickedUp(a0);
for q in 0..4usize { let pp = (&raw mut IT[q].xDD4_itemVar) as *mut *mut c_void; hm(pi(*pp)); *pp = null_mut(); } }
                    3 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::itCLinkMilk_NotifyParent(a0);
for q in 0..4usize { let pp = (&raw mut IT[q].xDD4_itemVar) as *mut *mut c_void; hm(pi(*pp)); *pp = null_mut(); } }
                    4 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::itCLinkMilk_Destroy(a0) as u32;
for q in 0..4usize { let pp = (&raw mut IT[q].xDD4_itemVar) as *mut *mut c_void; hm(pi(*pp)); *pp = null_mut(); } }
                    5 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::itClinkmilk_UnkMotion1_Anim(a0) as u32;
for q in 0..4usize { let pp = (&raw mut IT[q].xDD4_itemVar) as *mut *mut c_void; hm(pi(*pp)); *pp = null_mut(); } }
                    6 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::itClinkmilk_UnkMotion1_Phys(a0);
for q in 0..4usize { let pp = (&raw mut IT[q].xDD4_itemVar) as *mut *mut c_void; hm(pi(*pp)); *pp = null_mut(); } }
                    7 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::itClinkmilk_UnkMotion1_Coll(a0) as u32;
for q in 0..4usize { let pp = (&raw mut IT[q].xDD4_itemVar) as *mut *mut c_void; hm(pi(*pp)); *pp = null_mut(); } }
                    8 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = &raw mut GO[(rnd() % 4) as usize];
m::itCLinkMilk_Logic80_EvtUnk(a0, a1);
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
