//! Smoke test for tu/itzako.rs: the zero-argument branch of the knockback rotation and the kind table.
use melee_items::tu::itzako as m;
use melee_types::*;

#[test]
fn itzako_smoke() {
    unsafe {
        let mut v = Vec3 { x: 1.0, y: 2.0, z: 3.0 };
        let mut dir = Vec3 { x: 1.0, y: 0.0, z: 0.0 };
        m::it_8027C8D0(&raw mut v, &raw mut dir, 0.0);
        assert_eq!((v.x, v.y, v.z), (0.0, 0.0, 0.0));
        assert_eq!(m::it_803B85A8[0].x, 1.0);
    }
}
