//! Smoke test for src/tu/ft_0cdd.rs (ftCo_800CDE94 is checked bit-exact in tu_ft_0d27.rs).
use melee_fighter::tu::ft_0cdd as t;
use melee_fighter::types::*;
fn mk() -> (Box<HSD_GObj>, Box<Fighter>) {
    let mut f: Box<Fighter> = Box::new(unsafe { core::mem::zeroed() });
    let mut g: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    g.user_data = (&mut *f as *mut Fighter).cast();
    (g, f)
}
#[test]
fn unscale() {
    let (_g, mut f) = mk();
    f.x34_scale.y = 2.0;
    let v = t::unscale(&f, &Vec3 { x: 2.0, y: -4.0, z: 1.0 });
    assert_eq!((v.x, v.y, v.z), (1.0, -2.0, 0.5));
    f.x34_scale.y = 1.0;
    let v = t::unscale(&f, &Vec3 { x: 3.0, y: 0.0, z: -0.0 });
    assert_eq!((v.x, v.z.to_bits()), (3.0, 0x8000_0000));
}
