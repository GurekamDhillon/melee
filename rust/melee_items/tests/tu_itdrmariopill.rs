//! Smoke test for melee_items::tu::itdrmariopill (src/melee/it/kinds/itdrmariopill.c).
use melee_items::tu::itdrmariopill::*;

#[test]
fn itdrmariopill_smoke() {
    assert_eq!(it_803F75D0.len(), 7);
    assert_eq!(it_803F75D0[3].anim_id, 1);
    assert!(it_803F75D0[0].physics_updated.is_some());
    assert!(it_803F75D0[6].collided.is_some());
}
