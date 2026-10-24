//! Smoke test for melee_items::tu::itdkinoko (src/melee/it/kinds/itdkinoko.c).
use core::ptr::null_mut;
use melee_items::tu::itdkinoko::*;

#[test]
fn itdkinoko_smoke() {
    assert_eq!(it_803F6130.len(), 2);
    assert!(it_803F6130[1].collided.is_some());
    unsafe {
        assert!(!itDkinoko_UnkMotion0_Anim(null_mut()));
        assert!(!itDkinoko_UnkMotion1_Anim(null_mut()));
        assert!(itDKinoko_Logic27_DmgDealt(null_mut()));
    }
}
