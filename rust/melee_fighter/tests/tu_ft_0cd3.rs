//! Smoke test for src/tu/ft_0cd3.rs (Harisen callbacks without an item are no-ops).
use melee_fighter::tu::ft_0cd3 as t;
use melee_fighter::types::*;
fn mk() -> (Box<HSD_GObj>, Box<Fighter>) {
    let mut f: Box<Fighter> = Box::new(unsafe { core::mem::zeroed() });
    let mut g: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    g.user_data = (&mut *f as *mut Fighter).cast();
    (g, f)
}
#[test]
fn harisen_no_item() {
    let (mut g, _f) = mk();
    let gp: *mut HSD_GObj = &mut *g;
    t::ft_800cd604(gp);
    t::ft_800cd65c(gp);
    assert!(std::panic::catch_unwind(|| { let (mut g, _f) = mk(); t::ftco_swordswing_anim(&mut *g) }).is_err());
}
