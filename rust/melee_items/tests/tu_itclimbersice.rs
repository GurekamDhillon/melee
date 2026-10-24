//! Smoke test for melee_items::tu::itclimbersice (src/melee/it/kinds/itclimbersice.c).
use core::ptr::null_mut;
use melee_items::tu::itclimbersice::*;

#[test]
fn itclimbersice_smoke() {
    assert_eq!(it_803F7668.len(), 4);
    assert_eq!(it_803F7668[2].anim_id, 0);
    assert!(it_803F7668[0].physics_updated.is_some());
    unsafe {
        assert!(itClimbersIce_Logic90_DmgDealt(null_mut()));
        assert!(itClimbersIce_Logic90_Clanked(null_mut()));
        assert!(itClimbersIce_Logic90_Absorbed(null_mut()));
        assert!(!itClimbersice_UnkMotion1_Anim(null_mut()));
        itClimbersice_UnkMotion0_Phys(null_mut());
    }
}
