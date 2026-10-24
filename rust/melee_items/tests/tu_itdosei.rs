//! Written by gen/mtu/mk_driver.py from the same description as gen/mtu/tu/itdosei.c.
//! src/tu/itdosei.rs vs the C (the whole TU, shipping contraction; regenerate the expected file
//! with `python rust/melee_items/gen/mtu/run_tu.py itdosei`).
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables, unused_unsafe)]
#[path = "mtu_w/mod.rs"]
mod mtu_harness;
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;
use melee_items::tu::itdosei as m;
use mtu_harness::*;
#[path = "mtu/itdosei_gen.rs"]
mod g;

static mut SATTR: [[u32; 6]; 2] = [[0; 6]; 2];

unsafe fn setup_stem() {
    m::ext::it_804D6D28 = &raw mut ICD[0];
{ for q in 0..2usize { ART[q].x4_specialAttributes = (&raw mut SATTR[q]) as *mut c_void; for w in 0..6usize { if w == 1 || w == 3 || w == 4 || w == 5 { *(&raw mut SATTR[q] as *mut i32).add(w) = (fv() % 6) as i32; } else { *(&raw mut SATTR[q] as *mut f32).add(w) = rw(); } } } } { for q in 0..8usize { JO[q].flags &= !(JOBJ_USE_QUATERNION as u32); } } { for q in 0..4usize { let v = &mut IT[q].xDD4_itemVar.dosei; v.xDD4 = (fv() % 5) as i32; v.xDD8 = (fv() % 4) as i32; v.xDDC = rw(); v.xDE0 = rw(); v.xDE4.x = rw(); v.xDE4.y = rw(); v.xDE4.z = rw(); v.xDF0 = (fv() % 4) as i32; v.xDF4 = rw(); v.xDF8.x = rw(); v.xDF8.y = rw(); v.xDF8.z = rw(); } }
}

unsafe fn hash_stem() {
    g::hashg_all();
    hm(pi(m::ext::it_804D6D28));
{ for q in 0..2usize { for w in 0..6usize { hm(SATTR[q][w]); } } }
}

#[test]
fn itdosei_matches_c() {
    unsafe {
        g::install();
        preg_world();
        g::freg_all();
        freg_world();
preg(&raw const SATTR, core::mem::size_of_val(&SATTR));
        let mut c = Check::new(include_str!("data/mtu_itdosei.txt"));
        let mut total = 0u32;
        for op in 0..63u32 {
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
                37 => (300u32, false),
                38 => (300u32, false),
                39 => (300u32, false),
                40 => (300u32, false),
                41 => (300u32, false),
                42 => (300u32, false),
                43 => (300u32, false),
                44 => (300u32, false),
                45 => (300u32, false),
                46 => (300u32, false),
                47 => (300u32, false),
                48 => (300u32, false),
                49 => (300u32, false),
                50 => (300u32, false),
                51 => (300u32, false),
                52 => (300u32, false),
                53 => (300u32, false),
                54 => (300u32, false),
                55 => (300u32, false),
                56 => (300u32, false),
                57 => (300u32, false),
                58 => (300u32, false),
                59 => (300u32, false),
                60 => (300u32, false),
                61 => (300u32, false),
                62 => (300u32, false),
                _ => unreachable!(),
            };
            for k in 0..n {
                case(op, k);
                setup_world();
                setup_stem();
                let mut ret = 0u32;
                match op {
                    0 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = &raw mut IT[(rnd() % 4) as usize];
let a2 = rw();
m::itDosei_SetSpeed(a0, a1, a2);
for q in 0..4usize { let v = IT[q].xDD4_itemVar.dosei; hm(v.xDD4 as u32); hm(v.xDD8 as u32); hm(v.xDDC.to_bits()); hm(v.xDE0.to_bits()); hm(v.xDE4.x.to_bits()); hm(v.xDE4.y.to_bits()); hm(v.xDE4.z.to_bits()); hm(v.xDF0 as u32); hm(v.xDF4.to_bits()); hm(v.xDF8.x.to_bits()); hm(v.xDF8.y.to_bits()); hm(v.xDF8.z.to_bits()); core::ptr::write_bytes(&raw mut IT[q].xDD4_itemVar as *mut u8, 0, core::mem::size_of_val(&IT[q].xDD4_itemVar)); } }
                    1 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::HSD_JObjSetRotationZero(a0);
for q in 0..4usize { let v = IT[q].xDD4_itemVar.dosei; hm(v.xDD4 as u32); hm(v.xDD8 as u32); hm(v.xDDC.to_bits()); hm(v.xDE0.to_bits()); hm(v.xDE4.x.to_bits()); hm(v.xDE4.y.to_bits()); hm(v.xDE4.z.to_bits()); hm(v.xDF0 as u32); hm(v.xDF4.to_bits()); hm(v.xDF8.x.to_bits()); hm(v.xDF8.y.to_bits()); hm(v.xDF8.z.to_bits()); core::ptr::write_bytes(&raw mut IT[q].xDD4_itemVar as *mut u8, 0, core::mem::size_of_val(&IT[q].xDD4_itemVar)); } }
                    2 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::HSD_JObjSetRotationZeroWithMtxDirty(a0);
for q in 0..4usize { let v = IT[q].xDD4_itemVar.dosei; hm(v.xDD4 as u32); hm(v.xDD8 as u32); hm(v.xDDC.to_bits()); hm(v.xDE0.to_bits()); hm(v.xDE4.x.to_bits()); hm(v.xDE4.y.to_bits()); hm(v.xDE4.z.to_bits()); hm(v.xDF0 as u32); hm(v.xDF4.to_bits()); hm(v.xDF8.x.to_bits()); hm(v.xDF8.y.to_bits()); hm(v.xDF8.z.to_bits()); core::ptr::write_bytes(&raw mut IT[q].xDD4_itemVar as *mut u8, 0, core::mem::size_of_val(&IT[q].xDD4_itemVar)); } }
                    3 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = rw();
ret = m::itDosei_FacingAngle(a0, a1).to_bits();
for q in 0..4usize { let v = IT[q].xDD4_itemVar.dosei; hm(v.xDD4 as u32); hm(v.xDD8 as u32); hm(v.xDDC.to_bits()); hm(v.xDE0.to_bits()); hm(v.xDE4.x.to_bits()); hm(v.xDE4.y.to_bits()); hm(v.xDE4.z.to_bits()); hm(v.xDF0 as u32); hm(v.xDF4.to_bits()); hm(v.xDF8.x.to_bits()); hm(v.xDF8.y.to_bits()); hm(v.xDF8.z.to_bits()); core::ptr::write_bytes(&raw mut IT[q].xDD4_itemVar as *mut u8, 0, core::mem::size_of_val(&IT[q].xDD4_itemVar)); } }
                    4 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::itDosei_Logic7_Spawned(a0);
for q in 0..4usize { let v = IT[q].xDD4_itemVar.dosei; hm(v.xDD4 as u32); hm(v.xDD8 as u32); hm(v.xDDC.to_bits()); hm(v.xDE0.to_bits()); hm(v.xDE4.x.to_bits()); hm(v.xDE4.y.to_bits()); hm(v.xDE4.z.to_bits()); hm(v.xDF0 as u32); hm(v.xDF4.to_bits()); hm(v.xDF8.x.to_bits()); hm(v.xDF8.y.to_bits()); hm(v.xDF8.z.to_bits()); core::ptr::write_bytes(&raw mut IT[q].xDD4_itemVar as *mut u8, 0, core::mem::size_of_val(&IT[q].xDD4_itemVar)); } }
                    5 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::itDosei_80281390(a0);
for q in 0..4usize { let v = IT[q].xDD4_itemVar.dosei; hm(v.xDD4 as u32); hm(v.xDD8 as u32); hm(v.xDDC.to_bits()); hm(v.xDE0.to_bits()); hm(v.xDE4.x.to_bits()); hm(v.xDE4.y.to_bits()); hm(v.xDE4.z.to_bits()); hm(v.xDF0 as u32); hm(v.xDF4.to_bits()); hm(v.xDF8.x.to_bits()); hm(v.xDF8.y.to_bits()); hm(v.xDF8.z.to_bits()); core::ptr::write_bytes(&raw mut IT[q].xDD4_itemVar as *mut u8, 0, core::mem::size_of_val(&IT[q].xDD4_itemVar)); } }
                    6 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::itDosei_UnkMotion0_Anim(a0) as u32;
for q in 0..4usize { let v = IT[q].xDD4_itemVar.dosei; hm(v.xDD4 as u32); hm(v.xDD8 as u32); hm(v.xDDC.to_bits()); hm(v.xDE0.to_bits()); hm(v.xDE4.x.to_bits()); hm(v.xDE4.y.to_bits()); hm(v.xDE4.z.to_bits()); hm(v.xDF0 as u32); hm(v.xDF4.to_bits()); hm(v.xDF8.x.to_bits()); hm(v.xDF8.y.to_bits()); hm(v.xDF8.z.to_bits()); core::ptr::write_bytes(&raw mut IT[q].xDD4_itemVar as *mut u8, 0, core::mem::size_of_val(&IT[q].xDD4_itemVar)); } }
                    7 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::itDosei_UnkMotion0_Phys(a0);
for q in 0..4usize { let v = IT[q].xDD4_itemVar.dosei; hm(v.xDD4 as u32); hm(v.xDD8 as u32); hm(v.xDDC.to_bits()); hm(v.xDE0.to_bits()); hm(v.xDE4.x.to_bits()); hm(v.xDE4.y.to_bits()); hm(v.xDE4.z.to_bits()); hm(v.xDF0 as u32); hm(v.xDF4.to_bits()); hm(v.xDF8.x.to_bits()); hm(v.xDF8.y.to_bits()); hm(v.xDF8.z.to_bits()); core::ptr::write_bytes(&raw mut IT[q].xDD4_itemVar as *mut u8, 0, core::mem::size_of_val(&IT[q].xDD4_itemVar)); } }
                    8 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::itDosei_UnkMotion0_Coll(a0) as u32;
for q in 0..4usize { let v = IT[q].xDD4_itemVar.dosei; hm(v.xDD4 as u32); hm(v.xDD8 as u32); hm(v.xDDC.to_bits()); hm(v.xDE0.to_bits()); hm(v.xDE4.x.to_bits()); hm(v.xDE4.y.to_bits()); hm(v.xDE4.z.to_bits()); hm(v.xDF0 as u32); hm(v.xDF4.to_bits()); hm(v.xDF8.x.to_bits()); hm(v.xDF8.y.to_bits()); hm(v.xDF8.z.to_bits()); core::ptr::write_bytes(&raw mut IT[q].xDD4_itemVar as *mut u8, 0, core::mem::size_of_val(&IT[q].xDD4_itemVar)); } }
                    9 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::itDosei_80281734(a0);
for q in 0..4usize { let v = IT[q].xDD4_itemVar.dosei; hm(v.xDD4 as u32); hm(v.xDD8 as u32); hm(v.xDDC.to_bits()); hm(v.xDE0.to_bits()); hm(v.xDE4.x.to_bits()); hm(v.xDE4.y.to_bits()); hm(v.xDE4.z.to_bits()); hm(v.xDF0 as u32); hm(v.xDF4.to_bits()); hm(v.xDF8.x.to_bits()); hm(v.xDF8.y.to_bits()); hm(v.xDF8.z.to_bits()); core::ptr::write_bytes(&raw mut IT[q].xDD4_itemVar as *mut u8, 0, core::mem::size_of_val(&IT[q].xDD4_itemVar)); } }
                    10 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::itDosei_802817A0(a0);
for q in 0..4usize { let v = IT[q].xDD4_itemVar.dosei; hm(v.xDD4 as u32); hm(v.xDD8 as u32); hm(v.xDDC.to_bits()); hm(v.xDE0.to_bits()); hm(v.xDE4.x.to_bits()); hm(v.xDE4.y.to_bits()); hm(v.xDE4.z.to_bits()); hm(v.xDF0 as u32); hm(v.xDF4.to_bits()); hm(v.xDF8.x.to_bits()); hm(v.xDF8.y.to_bits()); hm(v.xDF8.z.to_bits()); core::ptr::write_bytes(&raw mut IT[q].xDD4_itemVar as *mut u8, 0, core::mem::size_of_val(&IT[q].xDD4_itemVar)); } }
                    11 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::itDosei_UnkMotion1_Anim(a0) as u32;
for q in 0..4usize { let v = IT[q].xDD4_itemVar.dosei; hm(v.xDD4 as u32); hm(v.xDD8 as u32); hm(v.xDDC.to_bits()); hm(v.xDE0.to_bits()); hm(v.xDE4.x.to_bits()); hm(v.xDE4.y.to_bits()); hm(v.xDE4.z.to_bits()); hm(v.xDF0 as u32); hm(v.xDF4.to_bits()); hm(v.xDF8.x.to_bits()); hm(v.xDF8.y.to_bits()); hm(v.xDF8.z.to_bits()); core::ptr::write_bytes(&raw mut IT[q].xDD4_itemVar as *mut u8, 0, core::mem::size_of_val(&IT[q].xDD4_itemVar)); } }
                    12 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::itDosei_UnkMotion1_Phys(a0);
for q in 0..4usize { let v = IT[q].xDD4_itemVar.dosei; hm(v.xDD4 as u32); hm(v.xDD8 as u32); hm(v.xDDC.to_bits()); hm(v.xDE0.to_bits()); hm(v.xDE4.x.to_bits()); hm(v.xDE4.y.to_bits()); hm(v.xDE4.z.to_bits()); hm(v.xDF0 as u32); hm(v.xDF4.to_bits()); hm(v.xDF8.x.to_bits()); hm(v.xDF8.y.to_bits()); hm(v.xDF8.z.to_bits()); core::ptr::write_bytes(&raw mut IT[q].xDD4_itemVar as *mut u8, 0, core::mem::size_of_val(&IT[q].xDD4_itemVar)); } }
                    13 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::itDosei_UnkMotion1_Coll(a0) as u32;
for q in 0..4usize { let v = IT[q].xDD4_itemVar.dosei; hm(v.xDD4 as u32); hm(v.xDD8 as u32); hm(v.xDDC.to_bits()); hm(v.xDE0.to_bits()); hm(v.xDE4.x.to_bits()); hm(v.xDE4.y.to_bits()); hm(v.xDE4.z.to_bits()); hm(v.xDF0 as u32); hm(v.xDF4.to_bits()); hm(v.xDF8.x.to_bits()); hm(v.xDF8.y.to_bits()); hm(v.xDF8.z.to_bits()); core::ptr::write_bytes(&raw mut IT[q].xDD4_itemVar as *mut u8, 0, core::mem::size_of_val(&IT[q].xDD4_itemVar)); } }
                    14 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::itDosei_80281C6C(a0);
for q in 0..4usize { let v = IT[q].xDD4_itemVar.dosei; hm(v.xDD4 as u32); hm(v.xDD8 as u32); hm(v.xDDC.to_bits()); hm(v.xDE0.to_bits()); hm(v.xDE4.x.to_bits()); hm(v.xDE4.y.to_bits()); hm(v.xDE4.z.to_bits()); hm(v.xDF0 as u32); hm(v.xDF4.to_bits()); hm(v.xDF8.x.to_bits()); hm(v.xDF8.y.to_bits()); hm(v.xDF8.z.to_bits()); core::ptr::write_bytes(&raw mut IT[q].xDD4_itemVar as *mut u8, 0, core::mem::size_of_val(&IT[q].xDD4_itemVar)); } }
                    15 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::itDosei_UnkMotion2_Anim(a0) as u32;
for q in 0..4usize { let v = IT[q].xDD4_itemVar.dosei; hm(v.xDD4 as u32); hm(v.xDD8 as u32); hm(v.xDDC.to_bits()); hm(v.xDE0.to_bits()); hm(v.xDE4.x.to_bits()); hm(v.xDE4.y.to_bits()); hm(v.xDE4.z.to_bits()); hm(v.xDF0 as u32); hm(v.xDF4.to_bits()); hm(v.xDF8.x.to_bits()); hm(v.xDF8.y.to_bits()); hm(v.xDF8.z.to_bits()); core::ptr::write_bytes(&raw mut IT[q].xDD4_itemVar as *mut u8, 0, core::mem::size_of_val(&IT[q].xDD4_itemVar)); } }
                    16 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::itDosei_UnkMotion2_Phys(a0);
for q in 0..4usize { let v = IT[q].xDD4_itemVar.dosei; hm(v.xDD4 as u32); hm(v.xDD8 as u32); hm(v.xDDC.to_bits()); hm(v.xDE0.to_bits()); hm(v.xDE4.x.to_bits()); hm(v.xDE4.y.to_bits()); hm(v.xDE4.z.to_bits()); hm(v.xDF0 as u32); hm(v.xDF4.to_bits()); hm(v.xDF8.x.to_bits()); hm(v.xDF8.y.to_bits()); hm(v.xDF8.z.to_bits()); core::ptr::write_bytes(&raw mut IT[q].xDD4_itemVar as *mut u8, 0, core::mem::size_of_val(&IT[q].xDD4_itemVar)); } }
                    17 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::itDosei_SetupWalk_FC(a0);
for q in 0..4usize { let v = IT[q].xDD4_itemVar.dosei; hm(v.xDD4 as u32); hm(v.xDD8 as u32); hm(v.xDDC.to_bits()); hm(v.xDE0.to_bits()); hm(v.xDE4.x.to_bits()); hm(v.xDE4.y.to_bits()); hm(v.xDE4.z.to_bits()); hm(v.xDF0 as u32); hm(v.xDF4.to_bits()); hm(v.xDF8.x.to_bits()); hm(v.xDF8.y.to_bits()); hm(v.xDF8.z.to_bits()); core::ptr::write_bytes(&raw mut IT[q].xDD4_itemVar as *mut u8, 0, core::mem::size_of_val(&IT[q].xDD4_itemVar)); } }
                    18 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::itDosei_UnkMotion2_Coll(a0) as u32;
for q in 0..4usize { let v = IT[q].xDD4_itemVar.dosei; hm(v.xDD4 as u32); hm(v.xDD8 as u32); hm(v.xDDC.to_bits()); hm(v.xDE0.to_bits()); hm(v.xDE4.x.to_bits()); hm(v.xDE4.y.to_bits()); hm(v.xDE4.z.to_bits()); hm(v.xDF0 as u32); hm(v.xDF4.to_bits()); hm(v.xDF8.x.to_bits()); hm(v.xDF8.y.to_bits()); hm(v.xDF8.z.to_bits()); core::ptr::write_bytes(&raw mut IT[q].xDD4_itemVar as *mut u8, 0, core::mem::size_of_val(&IT[q].xDD4_itemVar)); } }
                    19 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::itDosei_80282074(a0);
for q in 0..4usize { let v = IT[q].xDD4_itemVar.dosei; hm(v.xDD4 as u32); hm(v.xDD8 as u32); hm(v.xDDC.to_bits()); hm(v.xDE0.to_bits()); hm(v.xDE4.x.to_bits()); hm(v.xDE4.y.to_bits()); hm(v.xDE4.z.to_bits()); hm(v.xDF0 as u32); hm(v.xDF4.to_bits()); hm(v.xDF8.x.to_bits()); hm(v.xDF8.y.to_bits()); hm(v.xDF8.z.to_bits()); core::ptr::write_bytes(&raw mut IT[q].xDD4_itemVar as *mut u8, 0, core::mem::size_of_val(&IT[q].xDD4_itemVar)); } }
                    20 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::itDosei_UnkMotion3_Anim(a0) as u32;
for q in 0..4usize { let v = IT[q].xDD4_itemVar.dosei; hm(v.xDD4 as u32); hm(v.xDD8 as u32); hm(v.xDDC.to_bits()); hm(v.xDE0.to_bits()); hm(v.xDE4.x.to_bits()); hm(v.xDE4.y.to_bits()); hm(v.xDE4.z.to_bits()); hm(v.xDF0 as u32); hm(v.xDF4.to_bits()); hm(v.xDF8.x.to_bits()); hm(v.xDF8.y.to_bits()); hm(v.xDF8.z.to_bits()); core::ptr::write_bytes(&raw mut IT[q].xDD4_itemVar as *mut u8, 0, core::mem::size_of_val(&IT[q].xDD4_itemVar)); } }
                    21 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::itDosei_UnkMotion3_Phys(a0);
for q in 0..4usize { let v = IT[q].xDD4_itemVar.dosei; hm(v.xDD4 as u32); hm(v.xDD8 as u32); hm(v.xDDC.to_bits()); hm(v.xDE0.to_bits()); hm(v.xDE4.x.to_bits()); hm(v.xDE4.y.to_bits()); hm(v.xDE4.z.to_bits()); hm(v.xDF0 as u32); hm(v.xDF4.to_bits()); hm(v.xDF8.x.to_bits()); hm(v.xDF8.y.to_bits()); hm(v.xDF8.z.to_bits()); core::ptr::write_bytes(&raw mut IT[q].xDD4_itemVar as *mut u8, 0, core::mem::size_of_val(&IT[q].xDD4_itemVar)); } }
                    22 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::itDosei_UnkMotion5_Coll(a0) as u32;
for q in 0..4usize { let v = IT[q].xDD4_itemVar.dosei; hm(v.xDD4 as u32); hm(v.xDD8 as u32); hm(v.xDDC.to_bits()); hm(v.xDE0.to_bits()); hm(v.xDE4.x.to_bits()); hm(v.xDE4.y.to_bits()); hm(v.xDE4.z.to_bits()); hm(v.xDF0 as u32); hm(v.xDF4.to_bits()); hm(v.xDF8.x.to_bits()); hm(v.xDF8.y.to_bits()); hm(v.xDF8.z.to_bits()); core::ptr::write_bytes(&raw mut IT[q].xDD4_itemVar as *mut u8, 0, core::mem::size_of_val(&IT[q].xDD4_itemVar)); } }
                    23 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::itDosei_Logic7_PickedUp(a0);
for q in 0..4usize { let v = IT[q].xDD4_itemVar.dosei; hm(v.xDD4 as u32); hm(v.xDD8 as u32); hm(v.xDDC.to_bits()); hm(v.xDE0.to_bits()); hm(v.xDE4.x.to_bits()); hm(v.xDE4.y.to_bits()); hm(v.xDE4.z.to_bits()); hm(v.xDF0 as u32); hm(v.xDF4.to_bits()); hm(v.xDF8.x.to_bits()); hm(v.xDF8.y.to_bits()); hm(v.xDF8.z.to_bits()); core::ptr::write_bytes(&raw mut IT[q].xDD4_itemVar as *mut u8, 0, core::mem::size_of_val(&IT[q].xDD4_itemVar)); } }
                    24 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = &raw mut JO[(rnd() % 8) as usize];
m::itDosei_UnkMotion4_Anim_inline(a0, a1);
for q in 0..4usize { let v = IT[q].xDD4_itemVar.dosei; hm(v.xDD4 as u32); hm(v.xDD8 as u32); hm(v.xDDC.to_bits()); hm(v.xDE0.to_bits()); hm(v.xDE4.x.to_bits()); hm(v.xDE4.y.to_bits()); hm(v.xDE4.z.to_bits()); hm(v.xDF0 as u32); hm(v.xDF4.to_bits()); hm(v.xDF8.x.to_bits()); hm(v.xDF8.y.to_bits()); hm(v.xDF8.z.to_bits()); core::ptr::write_bytes(&raw mut IT[q].xDD4_itemVar as *mut u8, 0, core::mem::size_of_val(&IT[q].xDD4_itemVar)); } }
                    25 => { let a0 = (&raw mut SATTR[(rnd() & 1) as usize]) as *mut itDoseiAttributes;
ret = m::itDosei_UnkMotion4_GetAnimSpeed(a0).to_bits();
for q in 0..4usize { let v = IT[q].xDD4_itemVar.dosei; hm(v.xDD4 as u32); hm(v.xDD8 as u32); hm(v.xDDC.to_bits()); hm(v.xDE0.to_bits()); hm(v.xDE4.x.to_bits()); hm(v.xDE4.y.to_bits()); hm(v.xDE4.z.to_bits()); hm(v.xDF0 as u32); hm(v.xDF4.to_bits()); hm(v.xDF8.x.to_bits()); hm(v.xDF8.y.to_bits()); hm(v.xDF8.z.to_bits()); core::ptr::write_bytes(&raw mut IT[q].xDD4_itemVar as *mut u8, 0, core::mem::size_of_val(&IT[q].xDD4_itemVar)); } }
                    26 => { let a0 = &raw mut IT[(rnd() % 4) as usize];
ret = pi(m::itDosei_UnkMotion4_GetAttrs(a0));
for q in 0..4usize { let v = IT[q].xDD4_itemVar.dosei; hm(v.xDD4 as u32); hm(v.xDD8 as u32); hm(v.xDDC.to_bits()); hm(v.xDE0.to_bits()); hm(v.xDE4.x.to_bits()); hm(v.xDE4.y.to_bits()); hm(v.xDE4.z.to_bits()); hm(v.xDF0 as u32); hm(v.xDF4.to_bits()); hm(v.xDF8.x.to_bits()); hm(v.xDF8.y.to_bits()); hm(v.xDF8.z.to_bits()); core::ptr::write_bytes(&raw mut IT[q].xDD4_itemVar as *mut u8, 0, core::mem::size_of_val(&IT[q].xDD4_itemVar)); } }
                    27 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::itDosei_UnkMotion4_Anim(a0) as u32;
for q in 0..4usize { let v = IT[q].xDD4_itemVar.dosei; hm(v.xDD4 as u32); hm(v.xDD8 as u32); hm(v.xDDC.to_bits()); hm(v.xDE0.to_bits()); hm(v.xDE4.x.to_bits()); hm(v.xDE4.y.to_bits()); hm(v.xDE4.z.to_bits()); hm(v.xDF0 as u32); hm(v.xDF4.to_bits()); hm(v.xDF8.x.to_bits()); hm(v.xDF8.y.to_bits()); hm(v.xDF8.z.to_bits()); core::ptr::write_bytes(&raw mut IT[q].xDD4_itemVar as *mut u8, 0, core::mem::size_of_val(&IT[q].xDD4_itemVar)); } }
                    28 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::itDosei_UnkMotion4_Phys(a0);
for q in 0..4usize { let v = IT[q].xDD4_itemVar.dosei; hm(v.xDD4 as u32); hm(v.xDD8 as u32); hm(v.xDDC.to_bits()); hm(v.xDE0.to_bits()); hm(v.xDE4.x.to_bits()); hm(v.xDE4.y.to_bits()); hm(v.xDE4.z.to_bits()); hm(v.xDF0 as u32); hm(v.xDF4.to_bits()); hm(v.xDF8.x.to_bits()); hm(v.xDF8.y.to_bits()); hm(v.xDF8.z.to_bits()); core::ptr::write_bytes(&raw mut IT[q].xDD4_itemVar as *mut u8, 0, core::mem::size_of_val(&IT[q].xDD4_itemVar)); } }
                    29 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = pi(m::itDosei_GetJObj(a0));
for q in 0..4usize { let v = IT[q].xDD4_itemVar.dosei; hm(v.xDD4 as u32); hm(v.xDD8 as u32); hm(v.xDDC.to_bits()); hm(v.xDE0.to_bits()); hm(v.xDE4.x.to_bits()); hm(v.xDE4.y.to_bits()); hm(v.xDE4.z.to_bits()); hm(v.xDF0 as u32); hm(v.xDF4.to_bits()); hm(v.xDF8.x.to_bits()); hm(v.xDF8.y.to_bits()); hm(v.xDF8.z.to_bits()); core::ptr::write_bytes(&raw mut IT[q].xDD4_itemVar as *mut u8, 0, core::mem::size_of_val(&IT[q].xDD4_itemVar)); } }
                    30 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::itDosei_Logic7_Dropped(a0);
for q in 0..4usize { let v = IT[q].xDD4_itemVar.dosei; hm(v.xDD4 as u32); hm(v.xDD8 as u32); hm(v.xDDC.to_bits()); hm(v.xDE0.to_bits()); hm(v.xDE4.x.to_bits()); hm(v.xDE4.y.to_bits()); hm(v.xDE4.z.to_bits()); hm(v.xDF0 as u32); hm(v.xDF4.to_bits()); hm(v.xDF8.x.to_bits()); hm(v.xDF8.y.to_bits()); hm(v.xDF8.z.to_bits()); core::ptr::write_bytes(&raw mut IT[q].xDD4_itemVar as *mut u8, 0, core::mem::size_of_val(&IT[q].xDD4_itemVar)); } }
                    31 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::itDosei_Logic7_Thrown(a0);
for q in 0..4usize { let v = IT[q].xDD4_itemVar.dosei; hm(v.xDD4 as u32); hm(v.xDD8 as u32); hm(v.xDDC.to_bits()); hm(v.xDE0.to_bits()); hm(v.xDE4.x.to_bits()); hm(v.xDE4.y.to_bits()); hm(v.xDE4.z.to_bits()); hm(v.xDF0 as u32); hm(v.xDF4.to_bits()); hm(v.xDF8.x.to_bits()); hm(v.xDF8.y.to_bits()); hm(v.xDF8.z.to_bits()); core::ptr::write_bytes(&raw mut IT[q].xDD4_itemVar as *mut u8, 0, core::mem::size_of_val(&IT[q].xDD4_itemVar)); } }
                    32 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::itDosei_UnkMotion5_Anim(a0) as u32;
for q in 0..4usize { let v = IT[q].xDD4_itemVar.dosei; hm(v.xDD4 as u32); hm(v.xDD8 as u32); hm(v.xDDC.to_bits()); hm(v.xDE0.to_bits()); hm(v.xDE4.x.to_bits()); hm(v.xDE4.y.to_bits()); hm(v.xDE4.z.to_bits()); hm(v.xDF0 as u32); hm(v.xDF4.to_bits()); hm(v.xDF8.x.to_bits()); hm(v.xDF8.y.to_bits()); hm(v.xDF8.z.to_bits()); core::ptr::write_bytes(&raw mut IT[q].xDD4_itemVar as *mut u8, 0, core::mem::size_of_val(&IT[q].xDD4_itemVar)); } }
                    33 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::itDosei_UnkMotion5_Phys(a0);
for q in 0..4usize { let v = IT[q].xDD4_itemVar.dosei; hm(v.xDD4 as u32); hm(v.xDD8 as u32); hm(v.xDDC.to_bits()); hm(v.xDE0.to_bits()); hm(v.xDE4.x.to_bits()); hm(v.xDE4.y.to_bits()); hm(v.xDE4.z.to_bits()); hm(v.xDF0 as u32); hm(v.xDF4.to_bits()); hm(v.xDF8.x.to_bits()); hm(v.xDF8.y.to_bits()); hm(v.xDF8.z.to_bits()); core::ptr::write_bytes(&raw mut IT[q].xDD4_itemVar as *mut u8, 0, core::mem::size_of_val(&IT[q].xDD4_itemVar)); } }
                    34 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::itDosei_Logic7_EnteredAir(a0);
for q in 0..4usize { let v = IT[q].xDD4_itemVar.dosei; hm(v.xDD4 as u32); hm(v.xDD8 as u32); hm(v.xDDC.to_bits()); hm(v.xDE0.to_bits()); hm(v.xDE4.x.to_bits()); hm(v.xDE4.y.to_bits()); hm(v.xDE4.z.to_bits()); hm(v.xDF0 as u32); hm(v.xDF4.to_bits()); hm(v.xDF8.x.to_bits()); hm(v.xDF8.y.to_bits()); hm(v.xDF8.z.to_bits()); core::ptr::write_bytes(&raw mut IT[q].xDD4_itemVar as *mut u8, 0, core::mem::size_of_val(&IT[q].xDD4_itemVar)); } }
                    35 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::itDosei_UnkMotion6_Anim(a0) as u32;
for q in 0..4usize { let v = IT[q].xDD4_itemVar.dosei; hm(v.xDD4 as u32); hm(v.xDD8 as u32); hm(v.xDDC.to_bits()); hm(v.xDE0.to_bits()); hm(v.xDE4.x.to_bits()); hm(v.xDE4.y.to_bits()); hm(v.xDE4.z.to_bits()); hm(v.xDF0 as u32); hm(v.xDF4.to_bits()); hm(v.xDF8.x.to_bits()); hm(v.xDF8.y.to_bits()); hm(v.xDF8.z.to_bits()); core::ptr::write_bytes(&raw mut IT[q].xDD4_itemVar as *mut u8, 0, core::mem::size_of_val(&IT[q].xDD4_itemVar)); } }
                    36 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::itDosei_UnkMotion6_Phys(a0);
for q in 0..4usize { let v = IT[q].xDD4_itemVar.dosei; hm(v.xDD4 as u32); hm(v.xDD8 as u32); hm(v.xDDC.to_bits()); hm(v.xDE0.to_bits()); hm(v.xDE4.x.to_bits()); hm(v.xDE4.y.to_bits()); hm(v.xDE4.z.to_bits()); hm(v.xDF0 as u32); hm(v.xDF4.to_bits()); hm(v.xDF8.x.to_bits()); hm(v.xDF8.y.to_bits()); hm(v.xDF8.z.to_bits()); core::ptr::write_bytes(&raw mut IT[q].xDD4_itemVar as *mut u8, 0, core::mem::size_of_val(&IT[q].xDD4_itemVar)); } }
                    37 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::itDosei_UnkMotion6_Coll(a0) as u32;
for q in 0..4usize { let v = IT[q].xDD4_itemVar.dosei; hm(v.xDD4 as u32); hm(v.xDD8 as u32); hm(v.xDDC.to_bits()); hm(v.xDE0.to_bits()); hm(v.xDE4.x.to_bits()); hm(v.xDE4.y.to_bits()); hm(v.xDE4.z.to_bits()); hm(v.xDF0 as u32); hm(v.xDF4.to_bits()); hm(v.xDF8.x.to_bits()); hm(v.xDF8.y.to_bits()); hm(v.xDF8.z.to_bits()); core::ptr::write_bytes(&raw mut IT[q].xDD4_itemVar as *mut u8, 0, core::mem::size_of_val(&IT[q].xDD4_itemVar)); } }
                    38 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::itDosei_80282BFC(a0);
for q in 0..4usize { let v = IT[q].xDD4_itemVar.dosei; hm(v.xDD4 as u32); hm(v.xDD8 as u32); hm(v.xDDC.to_bits()); hm(v.xDE0.to_bits()); hm(v.xDE4.x.to_bits()); hm(v.xDE4.y.to_bits()); hm(v.xDE4.z.to_bits()); hm(v.xDF0 as u32); hm(v.xDF4.to_bits()); hm(v.xDF8.x.to_bits()); hm(v.xDF8.y.to_bits()); hm(v.xDF8.z.to_bits()); core::ptr::write_bytes(&raw mut IT[q].xDD4_itemVar as *mut u8, 0, core::mem::size_of_val(&IT[q].xDD4_itemVar)); } }
                    39 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::itDosei_UnkMotion8_Anim(a0) as u32;
for q in 0..4usize { let v = IT[q].xDD4_itemVar.dosei; hm(v.xDD4 as u32); hm(v.xDD8 as u32); hm(v.xDDC.to_bits()); hm(v.xDE0.to_bits()); hm(v.xDE4.x.to_bits()); hm(v.xDE4.y.to_bits()); hm(v.xDE4.z.to_bits()); hm(v.xDF0 as u32); hm(v.xDF4.to_bits()); hm(v.xDF8.x.to_bits()); hm(v.xDF8.y.to_bits()); hm(v.xDF8.z.to_bits()); core::ptr::write_bytes(&raw mut IT[q].xDD4_itemVar as *mut u8, 0, core::mem::size_of_val(&IT[q].xDD4_itemVar)); } }
                    40 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::itDosei_UnkMotion8_Phys(a0);
for q in 0..4usize { let v = IT[q].xDD4_itemVar.dosei; hm(v.xDD4 as u32); hm(v.xDD8 as u32); hm(v.xDDC.to_bits()); hm(v.xDE0.to_bits()); hm(v.xDE4.x.to_bits()); hm(v.xDE4.y.to_bits()); hm(v.xDE4.z.to_bits()); hm(v.xDF0 as u32); hm(v.xDF4.to_bits()); hm(v.xDF8.x.to_bits()); hm(v.xDF8.y.to_bits()); hm(v.xDF8.z.to_bits()); core::ptr::write_bytes(&raw mut IT[q].xDD4_itemVar as *mut u8, 0, core::mem::size_of_val(&IT[q].xDD4_itemVar)); } }
                    41 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::itDosei_UnkMotion8_Coll(a0) as u32;
for q in 0..4usize { let v = IT[q].xDD4_itemVar.dosei; hm(v.xDD4 as u32); hm(v.xDD8 as u32); hm(v.xDDC.to_bits()); hm(v.xDE0.to_bits()); hm(v.xDE4.x.to_bits()); hm(v.xDE4.y.to_bits()); hm(v.xDE4.z.to_bits()); hm(v.xDF0 as u32); hm(v.xDF4.to_bits()); hm(v.xDF8.x.to_bits()); hm(v.xDF8.y.to_bits()); hm(v.xDF8.z.to_bits()); core::ptr::write_bytes(&raw mut IT[q].xDD4_itemVar as *mut u8, 0, core::mem::size_of_val(&IT[q].xDD4_itemVar)); } }
                    42 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::itDosei_80282CD4(a0);
for q in 0..4usize { let v = IT[q].xDD4_itemVar.dosei; hm(v.xDD4 as u32); hm(v.xDD8 as u32); hm(v.xDDC.to_bits()); hm(v.xDE0.to_bits()); hm(v.xDE4.x.to_bits()); hm(v.xDE4.y.to_bits()); hm(v.xDE4.z.to_bits()); hm(v.xDF0 as u32); hm(v.xDF4.to_bits()); hm(v.xDF8.x.to_bits()); hm(v.xDF8.y.to_bits()); hm(v.xDF8.z.to_bits()); core::ptr::write_bytes(&raw mut IT[q].xDD4_itemVar as *mut u8, 0, core::mem::size_of_val(&IT[q].xDD4_itemVar)); } }
                    43 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::itDosei_UnkMotion7_Anim(a0) as u32;
for q in 0..4usize { let v = IT[q].xDD4_itemVar.dosei; hm(v.xDD4 as u32); hm(v.xDD8 as u32); hm(v.xDDC.to_bits()); hm(v.xDE0.to_bits()); hm(v.xDE4.x.to_bits()); hm(v.xDE4.y.to_bits()); hm(v.xDE4.z.to_bits()); hm(v.xDF0 as u32); hm(v.xDF4.to_bits()); hm(v.xDF8.x.to_bits()); hm(v.xDF8.y.to_bits()); hm(v.xDF8.z.to_bits()); core::ptr::write_bytes(&raw mut IT[q].xDD4_itemVar as *mut u8, 0, core::mem::size_of_val(&IT[q].xDD4_itemVar)); } }
                    44 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::itDosei_UnkMotion7_Phys(a0);
for q in 0..4usize { let v = IT[q].xDD4_itemVar.dosei; hm(v.xDD4 as u32); hm(v.xDD8 as u32); hm(v.xDDC.to_bits()); hm(v.xDE0.to_bits()); hm(v.xDE4.x.to_bits()); hm(v.xDE4.y.to_bits()); hm(v.xDE4.z.to_bits()); hm(v.xDF0 as u32); hm(v.xDF4.to_bits()); hm(v.xDF8.x.to_bits()); hm(v.xDF8.y.to_bits()); hm(v.xDF8.z.to_bits()); core::ptr::write_bytes(&raw mut IT[q].xDD4_itemVar as *mut u8, 0, core::mem::size_of_val(&IT[q].xDD4_itemVar)); } }
                    45 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::itDosei_UnkMotion7_Coll(a0) as u32;
for q in 0..4usize { let v = IT[q].xDD4_itemVar.dosei; hm(v.xDD4 as u32); hm(v.xDD8 as u32); hm(v.xDDC.to_bits()); hm(v.xDE0.to_bits()); hm(v.xDE4.x.to_bits()); hm(v.xDE4.y.to_bits()); hm(v.xDE4.z.to_bits()); hm(v.xDF0 as u32); hm(v.xDF4.to_bits()); hm(v.xDF8.x.to_bits()); hm(v.xDF8.y.to_bits()); hm(v.xDF8.z.to_bits()); core::ptr::write_bytes(&raw mut IT[q].xDD4_itemVar as *mut u8, 0, core::mem::size_of_val(&IT[q].xDD4_itemVar)); } }
                    46 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::itDosei_80282DE4(a0);
for q in 0..4usize { let v = IT[q].xDD4_itemVar.dosei; hm(v.xDD4 as u32); hm(v.xDD8 as u32); hm(v.xDDC.to_bits()); hm(v.xDE0.to_bits()); hm(v.xDE4.x.to_bits()); hm(v.xDE4.y.to_bits()); hm(v.xDE4.z.to_bits()); hm(v.xDF0 as u32); hm(v.xDF4.to_bits()); hm(v.xDF8.x.to_bits()); hm(v.xDF8.y.to_bits()); hm(v.xDF8.z.to_bits()); core::ptr::write_bytes(&raw mut IT[q].xDD4_itemVar as *mut u8, 0, core::mem::size_of_val(&IT[q].xDD4_itemVar)); } }
                    47 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::itDosei_UnkMotion9_Anim(a0) as u32;
for q in 0..4usize { let v = IT[q].xDD4_itemVar.dosei; hm(v.xDD4 as u32); hm(v.xDD8 as u32); hm(v.xDDC.to_bits()); hm(v.xDE0.to_bits()); hm(v.xDE4.x.to_bits()); hm(v.xDE4.y.to_bits()); hm(v.xDE4.z.to_bits()); hm(v.xDF0 as u32); hm(v.xDF4.to_bits()); hm(v.xDF8.x.to_bits()); hm(v.xDF8.y.to_bits()); hm(v.xDF8.z.to_bits()); core::ptr::write_bytes(&raw mut IT[q].xDD4_itemVar as *mut u8, 0, core::mem::size_of_val(&IT[q].xDD4_itemVar)); } }
                    48 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::itDosei_UnkMotion9_Phys(a0);
for q in 0..4usize { let v = IT[q].xDD4_itemVar.dosei; hm(v.xDD4 as u32); hm(v.xDD8 as u32); hm(v.xDDC.to_bits()); hm(v.xDE0.to_bits()); hm(v.xDE4.x.to_bits()); hm(v.xDE4.y.to_bits()); hm(v.xDE4.z.to_bits()); hm(v.xDF0 as u32); hm(v.xDF4.to_bits()); hm(v.xDF8.x.to_bits()); hm(v.xDF8.y.to_bits()); hm(v.xDF8.z.to_bits()); core::ptr::write_bytes(&raw mut IT[q].xDD4_itemVar as *mut u8, 0, core::mem::size_of_val(&IT[q].xDD4_itemVar)); } }
                    49 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::itDosei_UnkMotion9_Coll(a0) as u32;
for q in 0..4usize { let v = IT[q].xDD4_itemVar.dosei; hm(v.xDD4 as u32); hm(v.xDD8 as u32); hm(v.xDDC.to_bits()); hm(v.xDE0.to_bits()); hm(v.xDE4.x.to_bits()); hm(v.xDE4.y.to_bits()); hm(v.xDE4.z.to_bits()); hm(v.xDF0 as u32); hm(v.xDF4.to_bits()); hm(v.xDF8.x.to_bits()); hm(v.xDF8.y.to_bits()); hm(v.xDF8.z.to_bits()); core::ptr::write_bytes(&raw mut IT[q].xDD4_itemVar as *mut u8, 0, core::mem::size_of_val(&IT[q].xDD4_itemVar)); } }
                    50 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::itDosei_UnkMotion10_Anim(a0) as u32;
for q in 0..4usize { let v = IT[q].xDD4_itemVar.dosei; hm(v.xDD4 as u32); hm(v.xDD8 as u32); hm(v.xDDC.to_bits()); hm(v.xDE0.to_bits()); hm(v.xDE4.x.to_bits()); hm(v.xDE4.y.to_bits()); hm(v.xDE4.z.to_bits()); hm(v.xDF0 as u32); hm(v.xDF4.to_bits()); hm(v.xDF8.x.to_bits()); hm(v.xDF8.y.to_bits()); hm(v.xDF8.z.to_bits()); core::ptr::write_bytes(&raw mut IT[q].xDD4_itemVar as *mut u8, 0, core::mem::size_of_val(&IT[q].xDD4_itemVar)); } }
                    51 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::itDosei_UnkMotion10_Phys(a0);
for q in 0..4usize { let v = IT[q].xDD4_itemVar.dosei; hm(v.xDD4 as u32); hm(v.xDD8 as u32); hm(v.xDDC.to_bits()); hm(v.xDE0.to_bits()); hm(v.xDE4.x.to_bits()); hm(v.xDE4.y.to_bits()); hm(v.xDE4.z.to_bits()); hm(v.xDF0 as u32); hm(v.xDF4.to_bits()); hm(v.xDF8.x.to_bits()); hm(v.xDF8.y.to_bits()); hm(v.xDF8.z.to_bits()); core::ptr::write_bytes(&raw mut IT[q].xDD4_itemVar as *mut u8, 0, core::mem::size_of_val(&IT[q].xDD4_itemVar)); } }
                    52 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::itDosei_UnkMotion10_Coll(a0) as u32;
for q in 0..4usize { let v = IT[q].xDD4_itemVar.dosei; hm(v.xDD4 as u32); hm(v.xDD8 as u32); hm(v.xDDC.to_bits()); hm(v.xDE0.to_bits()); hm(v.xDE4.x.to_bits()); hm(v.xDE4.y.to_bits()); hm(v.xDE4.z.to_bits()); hm(v.xDF0 as u32); hm(v.xDF4.to_bits()); hm(v.xDF8.x.to_bits()); hm(v.xDF8.y.to_bits()); hm(v.xDF8.z.to_bits()); core::ptr::write_bytes(&raw mut IT[q].xDD4_itemVar as *mut u8, 0, core::mem::size_of_val(&IT[q].xDD4_itemVar)); } }
                    53 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::itDosei_Logic7_DmgReceived(a0) as u32;
for q in 0..4usize { let v = IT[q].xDD4_itemVar.dosei; hm(v.xDD4 as u32); hm(v.xDD8 as u32); hm(v.xDDC.to_bits()); hm(v.xDE0.to_bits()); hm(v.xDE4.x.to_bits()); hm(v.xDE4.y.to_bits()); hm(v.xDE4.z.to_bits()); hm(v.xDF0 as u32); hm(v.xDF4.to_bits()); hm(v.xDF8.x.to_bits()); hm(v.xDF8.y.to_bits()); hm(v.xDF8.z.to_bits()); core::ptr::write_bytes(&raw mut IT[q].xDD4_itemVar as *mut u8, 0, core::mem::size_of_val(&IT[q].xDD4_itemVar)); } }
                    54 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::itDosei_UnkMotion11_Anim(a0) as u32;
for q in 0..4usize { let v = IT[q].xDD4_itemVar.dosei; hm(v.xDD4 as u32); hm(v.xDD8 as u32); hm(v.xDDC.to_bits()); hm(v.xDE0.to_bits()); hm(v.xDE4.x.to_bits()); hm(v.xDE4.y.to_bits()); hm(v.xDE4.z.to_bits()); hm(v.xDF0 as u32); hm(v.xDF4.to_bits()); hm(v.xDF8.x.to_bits()); hm(v.xDF8.y.to_bits()); hm(v.xDF8.z.to_bits()); core::ptr::write_bytes(&raw mut IT[q].xDD4_itemVar as *mut u8, 0, core::mem::size_of_val(&IT[q].xDD4_itemVar)); } }
                    55 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::itDosei_UnkMotion11_Phys(a0);
for q in 0..4usize { let v = IT[q].xDD4_itemVar.dosei; hm(v.xDD4 as u32); hm(v.xDD8 as u32); hm(v.xDDC.to_bits()); hm(v.xDE0.to_bits()); hm(v.xDE4.x.to_bits()); hm(v.xDE4.y.to_bits()); hm(v.xDE4.z.to_bits()); hm(v.xDF0 as u32); hm(v.xDF4.to_bits()); hm(v.xDF8.x.to_bits()); hm(v.xDF8.y.to_bits()); hm(v.xDF8.z.to_bits()); core::ptr::write_bytes(&raw mut IT[q].xDD4_itemVar as *mut u8, 0, core::mem::size_of_val(&IT[q].xDD4_itemVar)); } }
                    56 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::itDosei_UnkMotion11_Coll(a0) as u32;
for q in 0..4usize { let v = IT[q].xDD4_itemVar.dosei; hm(v.xDD4 as u32); hm(v.xDD8 as u32); hm(v.xDDC.to_bits()); hm(v.xDE0.to_bits()); hm(v.xDE4.x.to_bits()); hm(v.xDE4.y.to_bits()); hm(v.xDE4.z.to_bits()); hm(v.xDF0 as u32); hm(v.xDF4.to_bits()); hm(v.xDF8.x.to_bits()); hm(v.xDF8.y.to_bits()); hm(v.xDF8.z.to_bits()); core::ptr::write_bytes(&raw mut IT[q].xDD4_itemVar as *mut u8, 0, core::mem::size_of_val(&IT[q].xDD4_itemVar)); } }
                    57 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::itDosei_Logic7_DmgDealt(a0) as u32;
for q in 0..4usize { let v = IT[q].xDD4_itemVar.dosei; hm(v.xDD4 as u32); hm(v.xDD8 as u32); hm(v.xDDC.to_bits()); hm(v.xDE0.to_bits()); hm(v.xDE4.x.to_bits()); hm(v.xDE4.y.to_bits()); hm(v.xDE4.z.to_bits()); hm(v.xDF0 as u32); hm(v.xDF4.to_bits()); hm(v.xDF8.x.to_bits()); hm(v.xDF8.y.to_bits()); hm(v.xDF8.z.to_bits()); core::ptr::write_bytes(&raw mut IT[q].xDD4_itemVar as *mut u8, 0, core::mem::size_of_val(&IT[q].xDD4_itemVar)); } }
                    58 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::itDosei_Logic7_Reflected(a0) as u32;
for q in 0..4usize { let v = IT[q].xDD4_itemVar.dosei; hm(v.xDD4 as u32); hm(v.xDD8 as u32); hm(v.xDDC.to_bits()); hm(v.xDE0.to_bits()); hm(v.xDE4.x.to_bits()); hm(v.xDE4.y.to_bits()); hm(v.xDE4.z.to_bits()); hm(v.xDF0 as u32); hm(v.xDF4.to_bits()); hm(v.xDF8.x.to_bits()); hm(v.xDF8.y.to_bits()); hm(v.xDF8.z.to_bits()); core::ptr::write_bytes(&raw mut IT[q].xDD4_itemVar as *mut u8, 0, core::mem::size_of_val(&IT[q].xDD4_itemVar)); } }
                    59 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::itDosei_Logic7_Clanked(a0) as u32;
for q in 0..4usize { let v = IT[q].xDD4_itemVar.dosei; hm(v.xDD4 as u32); hm(v.xDD8 as u32); hm(v.xDDC.to_bits()); hm(v.xDE0.to_bits()); hm(v.xDE4.x.to_bits()); hm(v.xDE4.y.to_bits()); hm(v.xDE4.z.to_bits()); hm(v.xDF0 as u32); hm(v.xDF4.to_bits()); hm(v.xDF8.x.to_bits()); hm(v.xDF8.y.to_bits()); hm(v.xDF8.z.to_bits()); core::ptr::write_bytes(&raw mut IT[q].xDD4_itemVar as *mut u8, 0, core::mem::size_of_val(&IT[q].xDD4_itemVar)); } }
                    60 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::itDosei_Logic7_HitShield(a0) as u32;
for q in 0..4usize { let v = IT[q].xDD4_itemVar.dosei; hm(v.xDD4 as u32); hm(v.xDD8 as u32); hm(v.xDDC.to_bits()); hm(v.xDE0.to_bits()); hm(v.xDE4.x.to_bits()); hm(v.xDE4.y.to_bits()); hm(v.xDE4.z.to_bits()); hm(v.xDF0 as u32); hm(v.xDF4.to_bits()); hm(v.xDF8.x.to_bits()); hm(v.xDF8.y.to_bits()); hm(v.xDF8.z.to_bits()); core::ptr::write_bytes(&raw mut IT[q].xDD4_itemVar as *mut u8, 0, core::mem::size_of_val(&IT[q].xDD4_itemVar)); } }
                    61 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::itDosei_Logic7_ShieldBounced(a0) as u32;
for q in 0..4usize { let v = IT[q].xDD4_itemVar.dosei; hm(v.xDD4 as u32); hm(v.xDD8 as u32); hm(v.xDDC.to_bits()); hm(v.xDE0.to_bits()); hm(v.xDE4.x.to_bits()); hm(v.xDE4.y.to_bits()); hm(v.xDE4.z.to_bits()); hm(v.xDF0 as u32); hm(v.xDF4.to_bits()); hm(v.xDF8.x.to_bits()); hm(v.xDF8.y.to_bits()); hm(v.xDF8.z.to_bits()); core::ptr::write_bytes(&raw mut IT[q].xDD4_itemVar as *mut u8, 0, core::mem::size_of_val(&IT[q].xDD4_itemVar)); } }
                    62 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = &raw mut GO[(rnd() % 4) as usize];
m::itDosei_Logic7_EvtUnk(a0, a1);
for q in 0..4usize { let v = IT[q].xDD4_itemVar.dosei; hm(v.xDD4 as u32); hm(v.xDD8 as u32); hm(v.xDDC.to_bits()); hm(v.xDE0.to_bits()); hm(v.xDE4.x.to_bits()); hm(v.xDE4.y.to_bits()); hm(v.xDE4.z.to_bits()); hm(v.xDF0 as u32); hm(v.xDF4.to_bits()); hm(v.xDF8.x.to_bits()); hm(v.xDF8.y.to_bits()); hm(v.xDF8.z.to_bits()); core::ptr::write_bytes(&raw mut IT[q].xDD4_itemVar as *mut u8, 0, core::mem::size_of_val(&IT[q].xDD4_itemVar)); } }
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
