//! Smoke test for src/tu/ftdevice.rs (stateful registration tables, no float math).
#![allow(static_mut_refs)]
use melee_fighter::tu::ftdevice as d;

#[test]
fn register_and_reset() {
    let g1 = 0x10 as *mut melee_fighter::types::Ground_GObj;
    d::ftco_800c06c0();
    d::ftco_800c06e8(g1, 3, None);
    d::ftco_800c0764(g1, 1, None);
    d::ftco_800c0764(g1, 2, None);
    d::ftco_800c07f8(g1, 5, None);
    unsafe {
        assert_eq!(d::ft_804D6578.x0, 1);
        assert_eq!(d::ft_80459A68[0].r#type, 3);
        assert_eq!(d::ftDevice_BuryThingCount, 2);
        assert_eq!(d::ftDevice_BuryThings[1].x4, 2);
        assert_eq!(d::ft_804D6570, 1);
    }
    assert!(std::panic::catch_unwind(|| d::ftco_800c07f8(g1, 1, None)).is_err());
    d::ftco_800c06c0();
    unsafe {
        assert_eq!(d::ftDevice_BuryThingCount, 0);
        assert!(d::ft_80459A68[0].ground.is_null());
    }
    d::ftco_800c07f8(g1, 1, None);
}
