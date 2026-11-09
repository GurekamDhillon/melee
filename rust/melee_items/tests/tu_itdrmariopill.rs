//! Written by gen/mtu/mk_driver.py from the same description as gen/mtu/tu/itdrmariopill.c.
//! src/tu/itdrmariopill.rs vs the C (the whole TU, shipping contraction; regenerate the expected file
//! with `python rust/melee_items/gen/mtu/run_tu.py itdrmariopill`).
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables, unused_unsafe)]
#[path = "mtu_w/mod.rs"]
mod mtu_harness;
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;
use melee_items::tu::itdrmariopill as m;
use mtu_harness::*;
#[path = "mtu/itdrmariopill_gen.rs"]
mod g;

static mut SATTR: [[u32; 6]; 2] = [[0; 6]; 2];

unsafe fn setup_stem() {
    m::ext::it_804D6D28 = &raw mut ICD[0];
{ for q in 0..2usize { ART[q].x4_specialAttributes = (&raw mut SATTR[q]) as *mut _; for w in 0..6usize { *(&raw mut SATTR[q] as *mut f32).add(w) = rw(); } } } { for q in 0..8usize { JO[q].flags &= !(JOBJ_USE_QUATERNION as u32); } } { for q in 0..4usize { let r = rnd(); let p: *mut HSD_GObj = if r & 3 == 0 { null_mut() } else { &raw mut GO[4 + ((r >> 8) % 3) as usize] }; IT[q].xDD4_itemVar.drmariopill.x4 = p; IT[q].owner = if r & 16 != 0 { p } else { pick_fgo() }; } }
}

unsafe fn hash_stem() {
    g::hashg_all();
    hm(pi(m::ext::it_804D6D28));
{ for q in 0..2usize { for w in 0..6usize { hm(SATTR[q][w]); } } }
}

#[test]
fn itdrmariopill_matches_c() {
    unsafe {
        g::install();
        preg_world();
        g::freg_all();
        freg_world();
preg(&raw const SATTR, core::mem::size_of_val(&SATTR));
        let mut c = Check::new(include_str!("data/mtu_itdrmariopill.txt"));
        let mut total = 0u32;
        for op in 0..28u32 {
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
                _ => unreachable!(),
            };
            for k in 0..n {
                case(op, k);
                setup_world();
                setup_stem();
                let mut ret = 0u32;
                match op {
                    0 => { let a0 = rw();
ret = m::my_sqrtf(a0).to_bits();
for q in 0..4usize { hm(pi(IT[q].xDD4_itemVar.drmariopill.x4)); IT[q].xDD4_itemVar.drmariopill.x4 = null_mut(); } }
                    1 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = &raw mut VA[(rnd() % 4) as usize];
let a2 = fv() as _;
let a3 = fv() as _;
let a4 = rw();
m::itDrMarioPill_Spawn(a0, a1, a2, a3, a4);
for q in 0..4usize { hm(pi(IT[q].xDD4_itemVar.drmariopill.x4)); IT[q].xDD4_itemVar.drmariopill.x4 = null_mut(); } }
                    2 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::itDrMarioPill_802C061C(a0);
for q in 0..4usize { hm(pi(IT[q].xDD4_itemVar.drmariopill.x4)); IT[q].xDD4_itemVar.drmariopill.x4 = null_mut(); } }
                    3 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = &raw mut VA[(rnd() % 4) as usize];
let a2 = fv() as _;
let a3 = fv() as _;
let a4 = rw();
ret = pi(m::itDrMarioPill_Appeal_Spawn(a0, a1, a2, a3, a4));
for q in 0..4usize { hm(pi(IT[q].xDD4_itemVar.drmariopill.x4)); IT[q].xDD4_itemVar.drmariopill.x4 = null_mut(); } }
                    4 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = &raw mut VA[(rnd() % 4) as usize];
let a2 = fv() as _;
let a3 = fv() as _;
let a4 = fv() as _;
let a5 = fv() as _;
let a6 = rw();
ret = pi(m::itDrMarioPill_802C09C4(a0, a1, a2, a3, a4, a5, a6));
for q in 0..4usize { hm(pi(IT[q].xDD4_itemVar.drmariopill.x4)); IT[q].xDD4_itemVar.drmariopill.x4 = null_mut(); } }
                    5 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::itDrMarioPill_802C0B5C(a0);
for q in 0..4usize { hm(pi(IT[q].xDD4_itemVar.drmariopill.x4)); IT[q].xDD4_itemVar.drmariopill.x4 = null_mut(); } }
                    6 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::itDrMarioPill_UnkMotion0_Anim(a0) as u32;
for q in 0..4usize { hm(pi(IT[q].xDD4_itemVar.drmariopill.x4)); IT[q].xDD4_itemVar.drmariopill.x4 = null_mut(); } }
                    7 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::itDrMarioPill_UnkMotion0_Phys(a0);
for q in 0..4usize { hm(pi(IT[q].xDD4_itemVar.drmariopill.x4)); IT[q].xDD4_itemVar.drmariopill.x4 = null_mut(); } }
                    8 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::itDrmariopill_UnkMotion0_Coll(a0) as u32;
for q in 0..4usize { hm(pi(IT[q].xDD4_itemVar.drmariopill.x4)); IT[q].xDD4_itemVar.drmariopill.x4 = null_mut(); } }
                    9 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::itDrMarioPill_802C0DBC(a0);
for q in 0..4usize { hm(pi(IT[q].xDD4_itemVar.drmariopill.x4)); IT[q].xDD4_itemVar.drmariopill.x4 = null_mut(); } }
                    10 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::itDrMarioPill_802C0DF8(a0);
for q in 0..4usize { hm(pi(IT[q].xDD4_itemVar.drmariopill.x4)); IT[q].xDD4_itemVar.drmariopill.x4 = null_mut(); } }
                    11 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::itDrMarioPill_Motion2_Anim_sub(a0) as u32;
for q in 0..4usize { hm(pi(IT[q].xDD4_itemVar.drmariopill.x4)); IT[q].xDD4_itemVar.drmariopill.x4 = null_mut(); } }
                    12 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::itDrMarioPill_Motion2_Anim_flags(a0);
for q in 0..4usize { hm(pi(IT[q].xDD4_itemVar.drmariopill.x4)); IT[q].xDD4_itemVar.drmariopill.x4 = null_mut(); } }
                    13 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::itDrMarioPill_Motion2_Anim(a0) as u32;
for q in 0..4usize { hm(pi(IT[q].xDD4_itemVar.drmariopill.x4)); IT[q].xDD4_itemVar.drmariopill.x4 = null_mut(); } }
                    14 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::itDrMarioPill_Motion2_Phys(a0);
for q in 0..4usize { hm(pi(IT[q].xDD4_itemVar.drmariopill.x4)); IT[q].xDD4_itemVar.drmariopill.x4 = null_mut(); } }
                    15 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::itDrMariopill_Motion2_Coll(a0) as u32;
for q in 0..4usize { hm(pi(IT[q].xDD4_itemVar.drmariopill.x4)); IT[q].xDD4_itemVar.drmariopill.x4 = null_mut(); } }
                    16 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = fv() as _;
m::itDrMarioPill_802C1180(a0, a1);
for q in 0..4usize { hm(pi(IT[q].xDD4_itemVar.drmariopill.x4)); IT[q].xDD4_itemVar.drmariopill.x4 = null_mut(); } }
                    17 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::itDrMarioPill_PickedUp(a0);
for q in 0..4usize { hm(pi(IT[q].xDD4_itemVar.drmariopill.x4)); IT[q].xDD4_itemVar.drmariopill.x4 = null_mut(); } }
                    18 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::itDrMarioPill_Motion6_Anim(a0) as u32;
for q in 0..4usize { hm(pi(IT[q].xDD4_itemVar.drmariopill.x4)); IT[q].xDD4_itemVar.drmariopill.x4 = null_mut(); } }
                    19 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::itDrMarioPill_Motion6_Phys(a0);
for q in 0..4usize { hm(pi(IT[q].xDD4_itemVar.drmariopill.x4)); IT[q].xDD4_itemVar.drmariopill.x4 = null_mut(); } }
                    20 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::itDrMarioPill_Motion6_Coll(a0) as u32;
for q in 0..4usize { hm(pi(IT[q].xDD4_itemVar.drmariopill.x4)); IT[q].xDD4_itemVar.drmariopill.x4 = null_mut(); } }
                    21 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::itDrMarioPill_DmgDealt(a0) as u32;
for q in 0..4usize { hm(pi(IT[q].xDD4_itemVar.drmariopill.x4)); IT[q].xDD4_itemVar.drmariopill.x4 = null_mut(); } }
                    22 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::itDrMarioPill_Reflected(a0) as u32;
for q in 0..4usize { hm(pi(IT[q].xDD4_itemVar.drmariopill.x4)); IT[q].xDD4_itemVar.drmariopill.x4 = null_mut(); } }
                    23 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::itDrMarioPill_Clanked(a0) as u32;
for q in 0..4usize { hm(pi(IT[q].xDD4_itemVar.drmariopill.x4)); IT[q].xDD4_itemVar.drmariopill.x4 = null_mut(); } }
                    24 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::itDrMarioPill_HitShield(a0) as u32;
for q in 0..4usize { hm(pi(IT[q].xDD4_itemVar.drmariopill.x4)); IT[q].xDD4_itemVar.drmariopill.x4 = null_mut(); } }
                    25 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::itDrMarioPill_Absorbed(a0) as u32;
for q in 0..4usize { hm(pi(IT[q].xDD4_itemVar.drmariopill.x4)); IT[q].xDD4_itemVar.drmariopill.x4 = null_mut(); } }
                    26 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::itDrMarioPill_ShieldBounced(a0) as u32;
for q in 0..4usize { hm(pi(IT[q].xDD4_itemVar.drmariopill.x4)); IT[q].xDD4_itemVar.drmariopill.x4 = null_mut(); } }
                    27 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = &raw mut GO[(rnd() % 4) as usize];
m::itDrMarioPill_EvtUnk(a0, a1);
for q in 0..4usize { hm(pi(IT[q].xDD4_itemVar.drmariopill.x4)); IT[q].xDD4_itemVar.drmariopill.x4 = null_mut(); } }
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
