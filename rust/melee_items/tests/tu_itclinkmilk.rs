//! Smoke test for melee_items::tu::itclinkmilk (src/melee/it/kinds/itclinkmilk.c).
use core::ptr::null_mut;
use melee_items::tu::itclinkmilk::*;

#[test]
fn itclinkmilk_smoke() {
    assert_eq!(it_803F7A28.len(), 2);
    assert_eq!(it_803F7A28[1].anim_id, 1);
    unsafe {
        assert!(!itClinkmilk_UnkMotion1_Coll(null_mut()));
        itClinkmilk_UnkMotion1_Phys(null_mut());
    }
}
