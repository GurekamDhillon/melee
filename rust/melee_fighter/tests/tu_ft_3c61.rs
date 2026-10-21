//! ft_3C61.c data: list shape and spot values from the C initialisers.
#![allow(static_mut_refs)]
use melee_fighter::tu::ft_3c61::*;

fn walk(mut p: *mut melee_types::ftCo_803C6594_t) -> Vec<(f32, u8, f32)> {
    let mut v = Vec::new();
    while !p.is_null() {
        unsafe { v.push(((*p).x0.x, (*p).x44, (*p).x4C)); p = (*p).next; }
    }
    v
}

#[test]
fn lists() {
    unsafe {
        for (i, p) in ftCo_803C6594.iter().enumerate() {
            assert_eq!(p.is_null(), i != 6 && i != 7, "kind {i}");
        }
        let l0 = walk(ftCo_803C6594[7]);
        let l1 = walk(ftCo_803C6594[6]);
        assert_eq!(l0.len(), 5);
        assert_eq!(l1.len(), 6);
        assert_eq!(l0[0].0.to_bits(), 37.0f32.to_bits());
        assert_eq!(l0[2], (-60.0, 1, -95.0));
        assert_eq!(LIST0_0.x3C.to_bits(), 0x3F490FDB);
    }
}
