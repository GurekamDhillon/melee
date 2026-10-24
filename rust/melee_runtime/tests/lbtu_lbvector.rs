//! Written by gen/lbtu/mk_driver.py from the same description as gen/lbtu/tu/lbvector.c.
//! src/tu/lbvector.rs vs the C (the whole TU, shipping contraction; regenerate the expected file
//! with `python rust/melee_runtime/gen/lbtu/run_tu.py lbvector`).
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables, unused_unsafe, unused_parens)]
#[path = "lbtu_harness/mod.rs"]
mod mtu_harness;
use core::ptr::null_mut;
use melee_types::*;
use melee_runtime::tu::lbvector as m;
use mtu_harness::*;
#[path = "lbtu_gen/lbvector_gen.rs"]
mod g;

static mut V: [Vec; 4] = unsafe { core::mem::zeroed() };
static mut CO: [HSD_CObj; 1] = unsafe { core::mem::zeroed() };
static mut Q: [Quaternion; 1] = unsafe { core::mem::zeroed() };
static mut MT: [Mtx; 2] = unsafe { core::mem::zeroed() };


unsafe fn setup() {
    let _ = 0;
    V = core::mem::zeroed();
    CO = core::mem::zeroed();
    Q = core::mem::zeroed();
    MT = core::mem::zeroed();
    for i in 0..4 { g::fill_Vec(&mut V[i]); }
    for i in 0..1 { g::fill_HSD_CObj(&mut CO[i]); }
    for i in 0..1 { g::fill_Quaternion(&mut Q[i]); }
    for m in 0..2 { for i in 0..3 { for j in 0..4 { MT[m][i][j] = rf(); } } }
}
unsafe fn hash_all() {

    for i in 0..4 { g::hash_Vec(&V[i]); }
    for i in 0..1 { g::hash_HSD_CObj(&CO[i]); }
    for i in 0..1 { g::hash_Quaternion(&Q[i]); }
    for m in 0..2 { for i in 0..3 { for j in 0..4 { hm(MT[m][i][j].to_bits()); } } }
}
#[test]
fn lbvector_matches_c() {
    unsafe {
        g::install();
        preg(&raw const V, core::mem::size_of_val(&V));
        preg(&raw const CO, core::mem::size_of_val(&CO));
        preg(&raw const Q, core::mem::size_of_val(&Q));
        preg(&raw const MT, core::mem::size_of_val(&MT));
        g::freg_all();

        let mut c = Check::new(include_str!("data/lbtu_lbvector.txt"));
        for op in 0..24u32 {
            for k in 0..400u32 {
                case(op, k);
                setup();
                let mut ret = 0u32;
                match op {
                    0 => {   ret = (m::lbVector_Normalize(&raw mut V[0])).to_bits(); } // lbVector_Normalize
                    1 => {   ret = (m::lbVector_NormalizeXY(&raw mut V[0])).to_bits(); } // lbVector_NormalizeXY
                    2 => {   ret = pi(m::lbVector_Add(&raw mut V[0], &raw mut V[1])); } // lbVector_Add
                    3 => {   ret = pi(m::lbVector_Add_xy(&raw mut V[0], &raw mut V[1])); } // lbVector_Add_xy
                    4 => {   ret = pi(m::lbVector_Sub(&raw mut V[0], &raw mut V[1])); } // lbVector_Sub
                    5 => {   ret = pi(m::lbVector_Diff(&raw mut V[0], &raw mut V[1], &raw mut V[2])); } // lbVector_Diff
                    6 => {   ret = pi(m::lbVector_CrossprodNormalized(&raw mut V[0], &raw mut V[1], &raw mut V[2])); } // lbVector_CrossprodNormalized
                    7 => {   ret = (m::lbVector_Angle(&raw mut V[0], &raw mut V[1])).to_bits(); } // lbVector_Angle
                    8 => {   ret = (m::lbVector_AngleXY(&raw mut V[0], &raw mut V[1])).to_bits(); } // lbVector_AngleXY
                    9 => {  let v0: f32 = (rf()) as f32; ret = (m::lbvector_sin(v0)).to_bits(); } // lbvector_sin
                    10 => {  let v0: f32 = (rf()) as f32; ret = (m::lbvector_cos(v0)).to_bits(); } // lbvector_cos
                    11 => {  let v2: f32 = (rf()) as f32; m::lbVector_RotateAboutUnitAxis(&raw mut V[0], &raw mut V[1], v2); } // lbVector_RotateAboutUnitAxis
                    12 => {  let v1: i32 = (fv() % 6) as i32; let v2: f32 = (rf()) as f32; m::lbVector_Rotate(&raw mut V[0], v1, v2); } // lbVector_Rotate
                    13 => {   m::lbVector_Mirror(&raw mut V[0], &raw mut V[1]); } // lbVector_Mirror
                    14 => {   ret = (m::lbVector_CosAngle(&raw mut V[0], &raw mut V[1])).to_bits(); } // lbVector_CosAngle
                    15 => {  let v3: f32 = (rf()) as f32; ret = pi(m::lbVector_Lerp(&raw mut V[0], &raw mut V[1], &raw mut V[2], v3)); } // lbVector_Lerp
                    16 => {  let v2: f32 = (rf()) as f32; ret = pi(m::lbVector_8000DE38(MT[0].as_mut_ptr(), &raw mut V[0], v2)); } // lbVector_8000DE38
                    17 => {   ret = pi(m::lbVector_EulerAnglesFromONB(&raw mut V[0], &raw mut V[1], &raw mut V[2], &raw mut V[3])); } // lbVector_EulerAnglesFromONB
                    18 => {   ret = pi(m::lbVector_EulerAnglesFromPartialONB(&raw mut V[0], &raw mut V[1], &raw mut V[2])); } // lbVector_EulerAnglesFromPartialONB
                    19 => {   ret = pi(m::lbVector_ApplyEulerRotation(&raw mut V[0], &raw mut V[1])); } // lbVector_ApplyEulerRotation
                    20 => {  let v0: f32 = (rf()) as f32; ret = (m::lbVector_sqrtf_accurate(v0)).to_bits(); } // lbVector_sqrtf_accurate
                    21 => { V[0].x = ((rnd() % 2001) as i32 - 1000) as f32 * 0.5; V[0].y = ((rnd() % 2001) as i32 - 1000) as f32 * 0.5; V[0].z = ((rnd() % 2001) as i32 - 1000) as f32 * 0.5; let v3: i32 = (fv() & 1) as i32; ret = pi(m::lbVector_WorldToScreen(&raw mut CO[0], &raw mut V[0], &raw mut V[1], v3)); } // lbVector_WorldToScreen
                    22 => {   m::lbVector_CreateEulerMatrix(MT[0].as_mut_ptr(), &raw mut Q[0]); } // lbVector_CreateEulerMatrix
                    23 => {   ret = (m::lbVector_8000E838(&raw mut V[0], &raw mut V[1], &raw mut V[2], &raw mut V[3])).to_bits(); } // lbVector_8000E838
                    _ => unreachable!(),
                }
                hash_all();
                c.line(op, k, ret);
            }
        }
        assert_eq!(c.done(), 9600);
    }
}
