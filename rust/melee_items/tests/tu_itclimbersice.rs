//! Written by gen/mtu/mk_driver.py from the same description as gen/mtu/tu/itclimbersice.c.
//! src/tu/itclimbersice.rs vs the C (the whole TU, shipping contraction; regenerate the expected file
//! with `python rust/melee_items/gen/mtu/run_tu.py itclimbersice`).
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables, unused_unsafe)]
#[path = "mtu_w/mod.rs"]
mod mtu_harness;
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;
use melee_items::tu::itclimbersice as m;
use mtu_harness::*;
#[path = "mtu/itclimbersice_gen.rs"]
mod g;

static mut SATTR: [[u32; 13]; 2] = [[0; 13]; 2];

unsafe fn setup_stem() {
    m::ext::it_804D6D28 = &raw mut ICD[0];
{ for q in 0..2usize { ART[q].x4_specialAttributes = (&raw mut SATTR[q]) as *mut c_void; for w in 0..13usize { if w == 11 || w == 12 { *(&raw mut SATTR[q] as *mut i32).add(w) = (fv() % 6) as i32; } else { *(&raw mut SATTR[q] as *mut f32).add(w) = rw(); } } } } { for q in 0..4usize { let r = rnd(); IT[q].kind = It_Kind_IceClimber_Ice + (r & 1) as i32; *((&raw mut IT[q].xDD4_itemVar) as *mut *mut c_void) = if r & 6 == 0 { null_mut() } else { &raw mut GO[4 + ((r >> 8) % 3) as usize] as *mut c_void }; } }
}

unsafe fn hash_stem() {
    g::hashg_all();
    hm(pi(m::ext::it_804D6D28));
{ for q in 0..2usize { for w in 0..13usize { hm(SATTR[q][w]); } } }
}

#[test]
fn itclimbersice_matches_c() {
    unsafe {
        g::install();
        preg_world();
        g::freg_all();
        freg_world();
preg(&raw const SATTR, core::mem::size_of_val(&SATTR));
        let mut c = Check::new(include_str!("data/mtu_itclimbersice.txt"));
        let mut total = 0u32;
        for op in 0..33u32 {
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
                _ => unreachable!(),
            };
            for k in 0..n {
                case(op, k);
                setup_world();
                setup_stem();
                let mut ret = 0u32;
                match op {
                    0 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::itClimbersIce_sub_x4(a0);
for q in 0..4usize { let pp = (&raw mut IT[q].xDD4_itemVar) as *mut *mut c_void; hm(pi(*pp)); *pp = null_mut(); } }
                    1 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = fv() as _;
let a2 = fv() as _;
m::itClimbersice_Spawn(a0, a1, a2);
for q in 0..4usize { let pp = (&raw mut IT[q].xDD4_itemVar) as *mut *mut c_void; hm(pi(*pp)); *pp = null_mut(); } }
                    2 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::itClimbersice_Spawn2(a0);
for q in 0..4usize { let pp = (&raw mut IT[q].xDD4_itemVar) as *mut *mut c_void; hm(pi(*pp)); *pp = null_mut(); } }
                    3 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = &raw mut VA[(rnd() % 4) as usize];
let a2 = fv() as _;
let a3 = rw();
ret = pi(m::it_802C1590(a0, a1, a2, a3));
for q in 0..4usize { let pp = (&raw mut IT[q].xDD4_itemVar) as *mut *mut c_void; hm(pi(*pp)); *pp = null_mut(); } }
                    4 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_802C16F8(a0);
for q in 0..4usize { let pp = (&raw mut IT[q].xDD4_itemVar) as *mut *mut c_void; hm(pi(*pp)); *pp = null_mut(); } }
                    5 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_802C17DC(a0);
for q in 0..4usize { let pp = (&raw mut IT[q].xDD4_itemVar) as *mut *mut c_void; hm(pi(*pp)); *pp = null_mut(); } }
                    6 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_2725_Logic90_Destroyed(a0);
for q in 0..4usize { let pp = (&raw mut IT[q].xDD4_itemVar) as *mut *mut c_void; hm(pi(*pp)); *pp = null_mut(); } }
                    7 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::it_802C1854(a0) as u32;
for q in 0..4usize { let pp = (&raw mut IT[q].xDD4_itemVar) as *mut *mut c_void; hm(pi(*pp)); *pp = null_mut(); } }
                    8 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_802C1950(a0);
for q in 0..4usize { let pp = (&raw mut IT[q].xDD4_itemVar) as *mut *mut c_void; hm(pi(*pp)); *pp = null_mut(); } }
                    9 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::itClimbersice_UnkMotion0_Anim(a0) as u32;
for q in 0..4usize { let pp = (&raw mut IT[q].xDD4_itemVar) as *mut *mut c_void; hm(pi(*pp)); *pp = null_mut(); } }
                    10 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::itClimbersice_UnkMotion0_Phys(a0);
for q in 0..4usize { let pp = (&raw mut IT[q].xDD4_itemVar) as *mut *mut c_void; hm(pi(*pp)); *pp = null_mut(); } }
                    11 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::itClimbersice_UnkMotion0_Coll(a0) as u32;
for q in 0..4usize { let pp = (&raw mut IT[q].xDD4_itemVar) as *mut *mut c_void; hm(pi(*pp)); *pp = null_mut(); } }
                    12 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_802C1A58(a0);
for q in 0..4usize { let pp = (&raw mut IT[q].xDD4_itemVar) as *mut *mut c_void; hm(pi(*pp)); *pp = null_mut(); } }
                    13 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::itClimbersice_UnkMotion1_Anim(a0) as u32;
for q in 0..4usize { let pp = (&raw mut IT[q].xDD4_itemVar) as *mut *mut c_void; hm(pi(*pp)); *pp = null_mut(); } }
                    14 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::itClimbersice_UnkMotion1_Phys(a0);
for q in 0..4usize { let pp = (&raw mut IT[q].xDD4_itemVar) as *mut *mut c_void; hm(pi(*pp)); *pp = null_mut(); } }
                    15 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::itClimbersice_UnkMotion1_Coll(a0) as u32;
for q in 0..4usize { let pp = (&raw mut IT[q].xDD4_itemVar) as *mut *mut c_void; hm(pi(*pp)); *pp = null_mut(); } }
                    16 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_802C1AE4(a0);
for q in 0..4usize { let pp = (&raw mut IT[q].xDD4_itemVar) as *mut *mut c_void; hm(pi(*pp)); *pp = null_mut(); } }
                    17 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::itClimbersice_UnkMotion2_Anim(a0) as u32;
for q in 0..4usize { let pp = (&raw mut IT[q].xDD4_itemVar) as *mut *mut c_void; hm(pi(*pp)); *pp = null_mut(); } }
                    18 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::itClimbersice_Phys_inline(a0);
for q in 0..4usize { let pp = (&raw mut IT[q].xDD4_itemVar) as *mut *mut c_void; hm(pi(*pp)); *pp = null_mut(); } }
                    19 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::itClimbersice_UnkMotion2_Phys(a0);
for q in 0..4usize { let pp = (&raw mut IT[q].xDD4_itemVar) as *mut *mut c_void; hm(pi(*pp)); *pp = null_mut(); } }
                    20 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::itClimbersice_Coll(a0) as u32;
for q in 0..4usize { let pp = (&raw mut IT[q].xDD4_itemVar) as *mut *mut c_void; hm(pi(*pp)); *pp = null_mut(); } }
                    21 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::itClimbersice_UnkMotion2_Coll(a0) as u32;
for q in 0..4usize { let pp = (&raw mut IT[q].xDD4_itemVar) as *mut *mut c_void; hm(pi(*pp)); *pp = null_mut(); } }
                    22 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::fn_802C1D44(a0);
for q in 0..4usize { let pp = (&raw mut IT[q].xDD4_itemVar) as *mut *mut c_void; hm(pi(*pp)); *pp = null_mut(); } }
                    23 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::itClimbersice_UnkMotion3_Anim(a0) as u32;
for q in 0..4usize { let pp = (&raw mut IT[q].xDD4_itemVar) as *mut *mut c_void; hm(pi(*pp)); *pp = null_mut(); } }
                    24 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::itClimbersice_UnkMotion3_Phys(a0);
for q in 0..4usize { let pp = (&raw mut IT[q].xDD4_itemVar) as *mut *mut c_void; hm(pi(*pp)); *pp = null_mut(); } }
                    25 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::itClimbersice_UnkMotion3_Coll(a0) as u32;
for q in 0..4usize { let pp = (&raw mut IT[q].xDD4_itemVar) as *mut *mut c_void; hm(pi(*pp)); *pp = null_mut(); } }
                    26 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::itClimbersIce_Logic90_DmgDealt(a0) as u32;
for q in 0..4usize { let pp = (&raw mut IT[q].xDD4_itemVar) as *mut *mut c_void; hm(pi(*pp)); *pp = null_mut(); } }
                    27 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::itClimbersIce_Logic90_Reflected(a0) as u32;
for q in 0..4usize { let pp = (&raw mut IT[q].xDD4_itemVar) as *mut *mut c_void; hm(pi(*pp)); *pp = null_mut(); } }
                    28 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::itClimbersIce_Logic90_Clanked(a0) as u32;
for q in 0..4usize { let pp = (&raw mut IT[q].xDD4_itemVar) as *mut *mut c_void; hm(pi(*pp)); *pp = null_mut(); } }
                    29 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::it_2725_Logic90_HitShield(a0) as u32;
for q in 0..4usize { let pp = (&raw mut IT[q].xDD4_itemVar) as *mut *mut c_void; hm(pi(*pp)); *pp = null_mut(); } }
                    30 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::itClimbersIce_Logic90_Absorbed(a0) as u32;
for q in 0..4usize { let pp = (&raw mut IT[q].xDD4_itemVar) as *mut *mut c_void; hm(pi(*pp)); *pp = null_mut(); } }
                    31 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::itClimbersIce_Logic90_ShieldBounced(a0) as u32;
for q in 0..4usize { let pp = (&raw mut IT[q].xDD4_itemVar) as *mut *mut c_void; hm(pi(*pp)); *pp = null_mut(); } }
                    32 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = &raw mut GO[(rnd() % 4) as usize];
m::itClimbersIce_Logic90_EvtUnk(a0, a1);
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
