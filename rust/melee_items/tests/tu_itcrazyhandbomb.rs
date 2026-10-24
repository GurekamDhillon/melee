//! Smoke test for melee_items::tu::itcrazyhandbomb (src/melee/it/kinds/itcrazyhandbomb.c).
use core::ptr::null_mut;
use melee_items::tu::itcrazyhandbomb::*;

#[test]
fn itcrazyhandbomb_smoke() {
    assert_eq!(it_803F93A8.len(), 2);
    assert!(it_803F93A8[0].collided.is_some());
    unsafe {
        assert!(!itCrazyhandbomb_UnkMotion0_Anim(null_mut()));
        assert!(!itCrazyhandbomb_UnkMotion1_Coll(null_mut()));
        it_802F1340();
        it_802F13B0(null_mut());
    }
}
