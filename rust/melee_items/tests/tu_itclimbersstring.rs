//! Written by gen/mtu/mk_driver.py from the same description as gen/mtu/tu/itclimbersstring.c.
//! src/tu/itclimbersstring.rs vs the C (the whole TU, shipping contraction; regenerate the expected file
//! with `python rust/melee_items/gen/mtu/run_tu.py itclimbersstring`).
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables, unused_unsafe)]
#[path = "mtu_w/mod.rs"]
mod mtu_harness;
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;
use melee_items::tu::itclimbersstring as m;
use mtu_harness::*;
#[path = "mtu/itclimbersstring_gen.rs"]
mod g;

static mut SATTR: [[u32; 14]; 2] = [[0; 14]; 2];
static mut LKN: usize = 0; static mut PARTS: [FighterBone; 32] = unsafe { core::mem::zeroed() }; static mut OUTP: [*mut ItemLink; 2] = [null_mut(); 2]; static mut MXA: [f32; 12] = [0.0; 12]; unsafe fn setup_xpools_() { g::setup_xpools(); }

unsafe fn setup_stem() {

{ for q in 0..2usize { ART[q].x4_specialAttributes = (&raw mut SATTR[q]) as *mut c_void; for w in 0..14usize { if w == 0 || w == 1 || w == 6 || w == 7 || w == 8 { *(&raw mut SATTR[q] as *mut i32).add(w) = (fv() % 6) as i32; } else { *(&raw mut SATTR[q] as *mut f32).add(w) = rw(); } } } } { for q in 0..8usize { JO[q].flags &= !(JOBJ_USE_QUATERNION as u32); } } { setup_xpools_(); LKN = 0; PARTS = core::mem::zeroed(); OUTP = [null_mut(); 2]; MXA = [0.0; 12]; for q in 0..12usize { MXA[q] = rw(); } for q in 0..32usize { PARTS[q].joint = &raw mut JO[(rnd() % 8) as usize]; } for q in 0..3usize { FT[q].parts = (&raw mut PARTS) as *mut FighterBone; FT[q].u.pp.x2240.x = if rnd() & 1 != 0 { 0.0 } else { rw() }; FT[q].u.pp.x2240.y = if rnd() & 1 != 0 { 0.0 } else { rw() }; } for j in 0..6usize { g::LK[j].next = if j != 0 { &raw mut g::LK[j - 1] } else { null_mut() }; g::LK[j].prev = if j < 5 { &raw mut g::LK[j + 1] } else { null_mut() }; g::LK[j].gobj = &raw mut GO[7 + (rnd() & 1) as usize]; g::LK[j].jobj = &raw mut JO[(rnd() % 8) as usize]; } g::LK[5].set_x2C_b0(1); for q in 0..4usize { let r = rnd(); let v = &mut IT[q].xDD4_itemVar.climbersstring; v.x0 = rw(); v.x4 = &raw mut g::LK[5]; v.x8 = &raw mut g::LK[0]; v.xC = &raw mut GO[4 + ((r >> 8) % 3) as usize]; v.x14 = &raw mut JO[((r >> 12) % 8) as usize]; } }
}

unsafe fn hash_stem() {
    g::hashg_all();

{ for q in 0..2usize { for w in 0..14usize { hm(SATTR[q][w]); } } } g::hash_xpools(); { for q in 0..32usize { hm(pi(PARTS[q].joint)); } hm(pi(OUTP[0])); hm(pi(OUTP[1])); for q in 0..12usize { hm(MXA[q].to_bits()); } }
}

#[test]
fn itclimbersstring_matches_c() {
    unsafe {
        g::install();
        preg_world();
        g::freg_all();
        freg_world();
preg(&raw const SATTR, core::mem::size_of_val(&SATTR)); g::preg_xpools(); preg(&raw const PARTS, core::mem::size_of_val(&PARTS)); preg(&raw const OUTP, core::mem::size_of_val(&OUTP)); preg(&raw const MXA, core::mem::size_of_val(&MXA));
        let mut c = Check::new(include_str!("data/mtu_itclimbersstring.txt"));
        let mut total = 0u32;
        for op in 0..25u32 {
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
                _ => unreachable!(),
            };
            for k in 0..n {
                case(op, k);
                setup_world();
                setup_stem();
                let mut ret = 0u32;
                match op {
                    0 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = &raw mut JO[(rnd() % 8) as usize];
let a2 = core::mem::transmute::<usize, _>(pick_cb(1));
m::it_802C248C_setupGObj(a0, a1, a2);
for q in 0..4usize { let v = &mut IT[q].xDD4_itemVar.climbersstring; hm(pi(v.x4)); hm(pi(v.x8)); hm(pi(v.xC)); hm(pi(v.x14)); v.x4 = null_mut(); v.x8 = null_mut(); v.xC = null_mut(); v.x14 = null_mut(); } }
                    1 => { let a0 = &raw mut IT[(rnd() % 4) as usize];
ret = pi(m::it_802C248C_joint24(a0));
for q in 0..4usize { let v = &mut IT[q].xDD4_itemVar.climbersstring; hm(pi(v.x4)); hm(pi(v.x8)); hm(pi(v.xC)); hm(pi(v.x14)); v.x4 = null_mut(); v.x8 = null_mut(); v.xC = null_mut(); v.x14 = null_mut(); } }
                    2 => { let a0 = &raw mut IT[(rnd() % 4) as usize];
ret = pi(m::it_802C248C_joint28(a0));
for q in 0..4usize { let v = &mut IT[q].xDD4_itemVar.climbersstring; hm(pi(v.x4)); hm(pi(v.x8)); hm(pi(v.xC)); hm(pi(v.x14)); v.x4 = null_mut(); v.x8 = null_mut(); v.xC = null_mut(); v.x14 = null_mut(); } }
                    3 => { let a0 = &raw mut IT[(rnd() % 4) as usize];
let a1 = &raw mut JO[(rnd() % 8) as usize];
ret = pi(m::it_802C248C(a0, a1));
for q in 0..4usize { let v = &mut IT[q].xDD4_itemVar.climbersstring; hm(pi(v.x4)); hm(pi(v.x8)); hm(pi(v.xC)); hm(pi(v.x14)); v.x4 = null_mut(); v.x8 = null_mut(); v.xC = null_mut(); v.x14 = null_mut(); } }
                    4 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_802C2750(a0);
for q in 0..4usize { let v = &mut IT[q].xDD4_itemVar.climbersstring; hm(pi(v.x4)); hm(pi(v.x8)); hm(pi(v.xC)); hm(pi(v.x14)); v.x4 = null_mut(); v.x8 = null_mut(); v.xC = null_mut(); v.x14 = null_mut(); } }
                    5 => { let a0 = &raw mut GO[4 + (rnd() % 3) as usize];
let a1 = &raw mut VA[(rnd() % 4) as usize];
let a2 = fv() as _;
let a3 = rw();
ret = pi(m::it_802C27D4(a0, a1, a2, a3));
for q in 0..4usize { let v = &mut IT[q].xDD4_itemVar.climbersstring; hm(pi(v.x4)); hm(pi(v.x8)); hm(pi(v.xC)); hm(pi(v.x14)); v.x4 = null_mut(); v.x8 = null_mut(); v.xC = null_mut(); v.x14 = null_mut(); } }
                    6 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::fn_802C28B8(a0);
for q in 0..4usize { let v = &mut IT[q].xDD4_itemVar.climbersstring; hm(pi(v.x4)); hm(pi(v.x8)); hm(pi(v.xC)); hm(pi(v.x14)); v.x4 = null_mut(); v.x8 = null_mut(); v.xC = null_mut(); v.x14 = null_mut(); } }
                    7 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::fn_802C28DC(a0);
for q in 0..4usize { let v = &mut IT[q].xDD4_itemVar.climbersstring; hm(pi(v.x4)); hm(pi(v.x8)); hm(pi(v.xC)); hm(pi(v.x14)); v.x4 = null_mut(); v.x8 = null_mut(); v.xC = null_mut(); v.x14 = null_mut(); } }
                    8 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::fn_802C29E8(a0);
for q in 0..4usize { let v = &mut IT[q].xDD4_itemVar.climbersstring; hm(pi(v.x4)); hm(pi(v.x8)); hm(pi(v.xC)); hm(pi(v.x14)); v.x4 = null_mut(); v.x8 = null_mut(); v.xC = null_mut(); v.x14 = null_mut(); } }
                    9 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::fn_802C2AF4(a0);
for q in 0..4usize { let v = &mut IT[q].xDD4_itemVar.climbersstring; hm(pi(v.x4)); hm(pi(v.x8)); hm(pi(v.xC)); hm(pi(v.x14)); v.x4 = null_mut(); v.x8 = null_mut(); v.xC = null_mut(); v.x14 = null_mut(); } }
                    10 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::itClimbersstring_Cleanup(a0);
for q in 0..4usize { let v = &mut IT[q].xDD4_itemVar.climbersstring; hm(pi(v.x4)); hm(pi(v.x8)); hm(pi(v.xC)); hm(pi(v.x14)); v.x4 = null_mut(); v.x8 = null_mut(); v.xC = null_mut(); v.x14 = null_mut(); } }
                    11 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::itClimbersstring_UnkMotion3_Anim(a0) as u32;
for q in 0..4usize { let v = &mut IT[q].xDD4_itemVar.climbersstring; hm(pi(v.x4)); hm(pi(v.x8)); hm(pi(v.xC)); hm(pi(v.x14)); v.x4 = null_mut(); v.x8 = null_mut(); v.xC = null_mut(); v.x14 = null_mut(); } }
                    12 => { let a0 = &raw mut g::LK[(rnd() % 6) as usize];
let a1 = &raw mut VA[(rnd() % 4) as usize];
let a2 = (&raw mut SATTR[(rnd() & 1) as usize]) as *mut itClimbersStringAttributes;
let a3 = rw();
m::it_802C2CA8(a0, a1, a2, a3);
for q in 0..4usize { let v = &mut IT[q].xDD4_itemVar.climbersstring; hm(pi(v.x4)); hm(pi(v.x8)); hm(pi(v.xC)); hm(pi(v.x14)); v.x4 = null_mut(); v.x8 = null_mut(); v.xC = null_mut(); v.x14 = null_mut(); } }
                    13 => { let a0 = &raw mut g::LK[(rnd() % 6) as usize];
let a1 = &raw mut VA[(rnd() % 4) as usize];
let a2 = (&raw mut SATTR[(rnd() & 1) as usize]) as *mut itClimbersStringAttributes;
let a3 = rw();
m::it_802C2DB0(a0, a1, a2, a3);
for q in 0..4usize { let v = &mut IT[q].xDD4_itemVar.climbersstring; hm(pi(v.x4)); hm(pi(v.x8)); hm(pi(v.xC)); hm(pi(v.x14)); v.x4 = null_mut(); v.x8 = null_mut(); v.xC = null_mut(); v.x14 = null_mut(); } }
                    14 => { let a0 = &raw mut g::LK[(rnd() % 6) as usize];
let a1 = &raw mut VA[(rnd() % 4) as usize];
let a2 = (&raw mut SATTR[(rnd() & 1) as usize]) as *mut itClimbersStringAttributes;
let a3 = &raw mut IT[(rnd() % 4) as usize];
ret = m::it_802C2EC4(a0, a1, a2, a3) as u32;
for q in 0..4usize { let v = &mut IT[q].xDD4_itemVar.climbersstring; hm(pi(v.x4)); hm(pi(v.x8)); hm(pi(v.xC)); hm(pi(v.x14)); v.x4 = null_mut(); v.x8 = null_mut(); v.xC = null_mut(); v.x14 = null_mut(); } }
                    15 => { let a0 = &raw mut g::LK[(rnd() % 6) as usize];
let a1 = &raw mut VA[(rnd() % 4) as usize];
let a2 = (&raw mut SATTR[(rnd() & 1) as usize]) as *mut itClimbersStringAttributes;
let a3 = rw();
m::it_802C2CA8_outline(a0, a1, a2, a3);
for q in 0..4usize { let v = &mut IT[q].xDD4_itemVar.climbersstring; hm(pi(v.x4)); hm(pi(v.x8)); hm(pi(v.xC)); hm(pi(v.x14)); v.x4 = null_mut(); v.x8 = null_mut(); v.xC = null_mut(); v.x14 = null_mut(); } }
                    16 => { let a0 = &raw mut g::LK[(rnd() % 6) as usize];
let a1 = &raw mut VA[(rnd() % 4) as usize];
let a2 = (&raw mut SATTR[(rnd() & 1) as usize]) as *mut itClimbersStringAttributes;
let a3 = &raw mut IT[(rnd() % 4) as usize];
ret = m::it_802C30E8(a0, a1, a2, a3) as u32;
for q in 0..4usize { let v = &mut IT[q].xDD4_itemVar.climbersstring; hm(pi(v.x4)); hm(pi(v.x8)); hm(pi(v.xC)); hm(pi(v.x14)); v.x4 = null_mut(); v.x8 = null_mut(); v.xC = null_mut(); v.x14 = null_mut(); } }
                    17 => { let a0 = &raw mut g::LK[(rnd() % 6) as usize];
let a1 = &raw mut VA[(rnd() % 4) as usize];
let a2 = (&raw mut SATTR[(rnd() & 1) as usize]) as *mut itClimbersStringAttributes;
let a3 = &raw mut IT[(rnd() % 4) as usize];
let a4 = rw();
ret = m::it_802C32D4(a0, a1, a2, a3, a4) as u32;
for q in 0..4usize { let v = &mut IT[q].xDD4_itemVar.climbersstring; hm(pi(v.x4)); hm(pi(v.x8)); hm(pi(v.xC)); hm(pi(v.x14)); v.x4 = null_mut(); v.x8 = null_mut(); v.xC = null_mut(); v.x14 = null_mut(); } }
                    18 => { let a0 = &raw mut IT[(rnd() % 4) as usize];
m::it_802C33B8(a0);
for q in 0..4usize { let v = &mut IT[q].xDD4_itemVar.climbersstring; hm(pi(v.x4)); hm(pi(v.x8)); hm(pi(v.xC)); hm(pi(v.x14)); v.x4 = null_mut(); v.x8 = null_mut(); v.xC = null_mut(); v.x14 = null_mut(); } }
                    19 => { let a0 = &raw mut IT[(rnd() % 4) as usize];
let a1 = &raw mut VA[(rnd() % 4) as usize];
m::it_802C3520(a0, a1);
for q in 0..4usize { let v = &mut IT[q].xDD4_itemVar.climbersstring; hm(pi(v.x4)); hm(pi(v.x8)); hm(pi(v.xC)); hm(pi(v.x14)); v.x4 = null_mut(); v.x8 = null_mut(); v.xC = null_mut(); v.x14 = null_mut(); } }
                    20 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_2725_Logic70_PickedUp(a0);
for q in 0..4usize { let v = &mut IT[q].xDD4_itemVar.climbersstring; hm(pi(v.x4)); hm(pi(v.x8)); hm(pi(v.xC)); hm(pi(v.x14)); v.x4 = null_mut(); v.x8 = null_mut(); v.xC = null_mut(); v.x14 = null_mut(); } }
                    21 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_802C3810(a0);
for q in 0..4usize { let v = &mut IT[q].xDD4_itemVar.climbersstring; hm(pi(v.x4)); hm(pi(v.x8)); hm(pi(v.xC)); hm(pi(v.x14)); v.x4 = null_mut(); v.x8 = null_mut(); v.xC = null_mut(); v.x14 = null_mut(); } }
                    22 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_802C3864(a0);
for q in 0..4usize { let v = &mut IT[q].xDD4_itemVar.climbersstring; hm(pi(v.x4)); hm(pi(v.x8)); hm(pi(v.xC)); hm(pi(v.x14)); v.x4 = null_mut(); v.x8 = null_mut(); v.xC = null_mut(); v.x14 = null_mut(); } }
                    23 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_802C3950(a0);
for q in 0..4usize { let v = &mut IT[q].xDD4_itemVar.climbersstring; hm(pi(v.x4)); hm(pi(v.x8)); hm(pi(v.xC)); hm(pi(v.x14)); v.x4 = null_mut(); v.x8 = null_mut(); v.xC = null_mut(); v.x14 = null_mut(); } }
                    24 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = &raw mut GO[(rnd() % 7) as usize];
m::it_2725_Logic70_EvtUnk(a0, a1);
for q in 0..4usize { let v = &mut IT[q].xDD4_itemVar.climbersstring; hm(pi(v.x4)); hm(pi(v.x8)); hm(pi(v.xC)); hm(pi(v.x14)); v.x4 = null_mut(); v.x8 = null_mut(); v.xC = null_mut(); v.x14 = null_mut(); } }
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
