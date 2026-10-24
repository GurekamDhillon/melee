//! Smoke test for melee_items::tu::itclimbersstring (src/melee/it/kinds/itclimbersstring.c).
use melee_items::tu::itclimbersstring::*;

#[test]
fn itclimbersstring_smoke() {
    assert_eq!(it_803F76B8.len(), 4);
    assert!(it_803F76B8.iter().all(|s| s.animated.is_some() && s.collided.is_none()));
    assert_eq!(it_803F76B8[3].anim_id, -1);
}
