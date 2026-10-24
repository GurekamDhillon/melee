//! Smoke test for melee_items::tu::itcoin (src/melee/it/kinds/itcoin.c).
use core::ptr::null_mut;
use melee_items::tu::itcoin::*;

#[test]
fn itcoin_smoke() {
    assert_eq!(it_803F93C8.len(), 6);
    assert!(it_803F93C8[2].collided.is_none());
    unsafe {
        assert!(!itCoin_UnkMotion0_Anim(null_mut()));
        assert!(!itCoin_UnkMotion1_Anim(null_mut()));
        assert!(!itCoin_UnkMotion2_Anim(null_mut()));
        assert!(!itCoin_UnkMotion4_Anim(null_mut()));
        itCoin_UnkMotion2_Phys(null_mut());
        itCoin_UnkMotion5_Phys(null_mut());
    }
}
