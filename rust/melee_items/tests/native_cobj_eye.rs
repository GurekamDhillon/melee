//! src/tu/cobj.rs `HSD_CObjGetEyeDistance` vs the C (gen/native/cobj_eye_ref.c, built by gen/native/build_cobj_eye.sh:
//! shipping contraction + gwtool lowering over the real mtx_pc.c VECSubtract / VECMag): 20000 generated eye / interest
//! pairs incl. zeros, small ints, huge magnitudes and arbitrary bit patterns (NaN / inf). Bit equality (NaN canonical).
#![allow(non_snake_case)]
use melee_items::tu::cobj::HSD_CObjGetEyeDistance;
use melee_types::*;

#[test]
fn eye_distance_matches_c() {
    let mut rows = 0;
    for line in include_str!("data/native_cobj_eye.txt").lines() {
        let w: std::vec::Vec<u32> = line.split_whitespace().map(|s| u32::from_str_radix(s, 16).unwrap()).collect();
        assert_eq!(w.len(), 7);
        let f = |i: usize| f32::from_bits(w[i]);
        unsafe {
            let mut e: HSD_WObj = core::mem::zeroed();
            let mut t: HSD_WObj = core::mem::zeroed();
            e.pos.x = f(0);
            e.pos.y = f(1);
            e.pos.z = f(2);
            t.pos.x = f(3);
            t.pos.y = f(4);
            t.pos.z = f(5);
            let mut c: HSD_CObj = HSD_CObj::zeroed();
            c.eyepos = &mut e;
            c.interest = &mut t;
            let d = HSD_CObjGetEyeDistance(&mut c);
            let got = if d.is_nan() { 0x7FC0_0000 } else { d.to_bits() };
            assert_eq!(got, w[6], "row {rows}");
        }
        rows += 1;
    }
    assert_eq!(rows, 20000);
    unsafe { assert_eq!(HSD_CObjGetEyeDistance(core::ptr::null_mut()).to_bits(), 0) };
}
