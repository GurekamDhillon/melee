//! Smoke test for melee_items::tu::itdosei (src/melee/it/kinds/itdosei.c).
use core::ptr::null_mut;
use melee_items::tu::itdosei::*;

#[test]
fn itdosei_smoke() {
    assert_eq!(it_803F55D0.len(), 12);
    assert_eq!(it_803F56B8, [0x109, 0x10A, 0x10B, 0]);
    assert!(it_803F55D0[4].collided.is_none());
    unsafe {
        assert!(!itDosei_UnkMotion5_Anim(null_mut()));
        itDosei_UnkMotion0_Phys(null_mut());
        itDosei_UnkMotion7_Phys(null_mut());
    }
}
