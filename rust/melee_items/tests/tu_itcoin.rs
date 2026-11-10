//! Written by gen/mtu/mk_driver.py from the same description as gen/mtu/tu/itcoin.c.
//! src/tu/itcoin.rs vs the C (the whole TU, shipping contraction; regenerate the expected file
//! with `python rust/melee_items/gen/mtu/run_tu.py itcoin`).
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables, unused_unsafe)]
#[path = "mtu_w/mod.rs"]
mod mtu_harness;
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;
use melee_items::tu::itcoin as m;
use mtu_harness::*;
#[path = "mtu/itcoin_gen.rs"]
mod g;

static mut SATTR: [[u32; 19]; 2] = [[0; 19]; 2];

unsafe fn setup_stem() {

{ for q in 0..2usize { ART[q].x4_specialAttributes = (&raw mut SATTR[q]) as *mut _; for w in 0..19usize { *(&raw mut SATTR[q] as *mut f32).add(w) = rw(); } } } { for q in 0..8usize { JO[q].flags &= !(JOBJ_USE_QUATERNION as u32); } }
}

unsafe fn hash_stem() {
    g::hashg_all();

{ for q in 0..2usize { for w in 0..19usize { hm(SATTR[q][w]); } } }
}

#[test]
fn itcoin_matches_c() {
    unsafe {
        g::install();
        preg_world();
        g::freg_all();
        freg_world();
preg(&raw const SATTR, core::mem::size_of_val(&SATTR));
        let mut c = Check::new(include_str!("data/mtu_itcoin.txt"));
        let mut total = 0u32;
        for op in 0..32u32 {
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
                _ => unreachable!(),
            };
            for k in 0..n {
                case(op, k);
                setup_world();
                setup_stem();
                let mut ret = 0u32;
                match op {
                    0 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = fv() as _;
m::it_802F13B4(a0, a1);
for q in 0..4usize { hm(pi(IT[q].xDD4_itemVar.coin.x14)); IT[q].xDD4_itemVar.coin.x14 = null_mut(); } }
                    1 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::itCoin_Logic116_Spawned(a0);
for q in 0..4usize { hm(pi(IT[q].xDD4_itemVar.coin.x14)); IT[q].xDD4_itemVar.coin.x14 = null_mut(); } }
                    2 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::itCoin_Logic116_Destroyed(a0);
for q in 0..4usize { hm(pi(IT[q].xDD4_itemVar.coin.x14)); IT[q].xDD4_itemVar.coin.x14 = null_mut(); } }
                    3 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_802F1588(a0);
for q in 0..4usize { hm(pi(IT[q].xDD4_itemVar.coin.x14)); IT[q].xDD4_itemVar.coin.x14 = null_mut(); } }
                    4 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::itCoin_UnkMotion0_Anim(a0) as u32;
for q in 0..4usize { hm(pi(IT[q].xDD4_itemVar.coin.x14)); IT[q].xDD4_itemVar.coin.x14 = null_mut(); } }
                    5 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::itCoin_UnkMotion0_Phys(a0);
for q in 0..4usize { hm(pi(IT[q].xDD4_itemVar.coin.x14)); IT[q].xDD4_itemVar.coin.x14 = null_mut(); } }
                    6 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::itCoin_UnkMotion0_Coll(a0) as u32;
for q in 0..4usize { hm(pi(IT[q].xDD4_itemVar.coin.x14)); IT[q].xDD4_itemVar.coin.x14 = null_mut(); } }
                    7 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::itCoin_ResetRotation(a0);
for q in 0..4usize { hm(pi(IT[q].xDD4_itemVar.coin.x14)); IT[q].xDD4_itemVar.coin.x14 = null_mut(); } }
                    8 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_802F1630(a0);
for q in 0..4usize { hm(pi(IT[q].xDD4_itemVar.coin.x14)); IT[q].xDD4_itemVar.coin.x14 = null_mut(); } }
                    9 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::itCoin_UnkMotion1_Anim(a0) as u32;
for q in 0..4usize { hm(pi(IT[q].xDD4_itemVar.coin.x14)); IT[q].xDD4_itemVar.coin.x14 = null_mut(); } }
                    10 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::itCoin_UnkMotion1_Phys(a0);
for q in 0..4usize { hm(pi(IT[q].xDD4_itemVar.coin.x14)); IT[q].xDD4_itemVar.coin.x14 = null_mut(); } }
                    11 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::itCoin_UnkMotion1_Coll(a0) as u32;
for q in 0..4usize { hm(pi(IT[q].xDD4_itemVar.coin.x14)); IT[q].xDD4_itemVar.coin.x14 = null_mut(); } }
                    12 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::itCoin_Logic116_PickedUp(a0);
for q in 0..4usize { hm(pi(IT[q].xDD4_itemVar.coin.x14)); IT[q].xDD4_itemVar.coin.x14 = null_mut(); } }
                    13 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::itCoin_UnkMotion2_Anim(a0) as u32;
for q in 0..4usize { hm(pi(IT[q].xDD4_itemVar.coin.x14)); IT[q].xDD4_itemVar.coin.x14 = null_mut(); } }
                    14 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::itCoin_UnkMotion2_Phys(a0);
for q in 0..4usize { hm(pi(IT[q].xDD4_itemVar.coin.x14)); IT[q].xDD4_itemVar.coin.x14 = null_mut(); } }
                    15 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = &raw mut GO[(rnd() % 4) as usize];
m::itCoin_Logic116_EvtUnk(a0, a1);
for q in 0..4usize { hm(pi(IT[q].xDD4_itemVar.coin.x14)); IT[q].xDD4_itemVar.coin.x14 = null_mut(); } }
                    16 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::itCoin_Logic116_DmgReceived(a0) as u32;
for q in 0..4usize { hm(pi(IT[q].xDD4_itemVar.coin.x14)); IT[q].xDD4_itemVar.coin.x14 = null_mut(); } }
                    17 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::itCoin_UnkMotion3_Anim(a0) as u32;
for q in 0..4usize { hm(pi(IT[q].xDD4_itemVar.coin.x14)); IT[q].xDD4_itemVar.coin.x14 = null_mut(); } }
                    18 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::itCoin_UnkMotion3_Phys(a0);
for q in 0..4usize { hm(pi(IT[q].xDD4_itemVar.coin.x14)); IT[q].xDD4_itemVar.coin.x14 = null_mut(); } }
                    19 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::itCoin_UnkMotion3_Coll(a0) as u32;
for q in 0..4usize { hm(pi(IT[q].xDD4_itemVar.coin.x14)); IT[q].xDD4_itemVar.coin.x14 = null_mut(); } }
                    20 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::itCoin_Logic116_Thrown(a0);
for q in 0..4usize { hm(pi(IT[q].xDD4_itemVar.coin.x14)); IT[q].xDD4_itemVar.coin.x14 = null_mut(); } }
                    21 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::itCoin_UnkMotion4_Anim(a0) as u32;
for q in 0..4usize { hm(pi(IT[q].xDD4_itemVar.coin.x14)); IT[q].xDD4_itemVar.coin.x14 = null_mut(); } }
                    22 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::itCoin_UnkMotion4_Phys(a0);
for q in 0..4usize { hm(pi(IT[q].xDD4_itemVar.coin.x14)); IT[q].xDD4_itemVar.coin.x14 = null_mut(); } }
                    23 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::itCoin_UnkMotion4_Coll(a0) as u32;
for q in 0..4usize { hm(pi(IT[q].xDD4_itemVar.coin.x14)); IT[q].xDD4_itemVar.coin.x14 = null_mut(); } }
                    24 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::itCoin_Logic116_EnteredAir(a0);
for q in 0..4usize { hm(pi(IT[q].xDD4_itemVar.coin.x14)); IT[q].xDD4_itemVar.coin.x14 = null_mut(); } }
                    25 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::itCoin_UnkMotion5_Anim(a0) as u32;
for q in 0..4usize { hm(pi(IT[q].xDD4_itemVar.coin.x14)); IT[q].xDD4_itemVar.coin.x14 = null_mut(); } }
                    26 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::itCoin_UnkMotion5_Phys(a0);
for q in 0..4usize { hm(pi(IT[q].xDD4_itemVar.coin.x14)); IT[q].xDD4_itemVar.coin.x14 = null_mut(); } }
                    27 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::itCoin_UnkMotion5_Coll(a0) as u32;
for q in 0..4usize { hm(pi(IT[q].xDD4_itemVar.coin.x14)); IT[q].xDD4_itemVar.coin.x14 = null_mut(); } }
                    28 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = fv() as _;
m::it_802F2014(a0, a1);
for q in 0..4usize { hm(pi(IT[q].xDD4_itemVar.coin.x14)); IT[q].xDD4_itemVar.coin.x14 = null_mut(); } }
                    29 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::it_802F2020(a0) as u32;
for q in 0..4usize { hm(pi(IT[q].xDD4_itemVar.coin.x14)); IT[q].xDD4_itemVar.coin.x14 = null_mut(); } }
                    30 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_802F202C(a0);
for q in 0..4usize { hm(pi(IT[q].xDD4_itemVar.coin.x14)); IT[q].xDD4_itemVar.coin.x14 = null_mut(); } }
                    31 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = &raw mut VA[(rnd() % 4) as usize];
let a2 = fv() as _;
let a3 = fv() as _;
ret = pi(m::it_802F2094(a0, a1, a2, a3));
for q in 0..4usize { hm(pi(IT[q].xDD4_itemVar.coin.x14)); IT[q].xDD4_itemVar.coin.x14 = null_mut(); } }
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
