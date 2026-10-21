//! Smoke test for src/tu/ft_0ce3.rs (callbacks without an item are no-ops).
use melee_fighter::tu::ft_0ce3 as t;
use melee_fighter::types::*;
fn mk() -> (Box<HSD_GObj>, Box<Fighter>) {
    let mut f: Box<Fighter> = Box::new(unsafe { core::mem::zeroed() });
    let mut g: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    g.user_data = (&mut *f as *mut Fighter).cast();
    (g, f)
}
#[test]
fn flower_no_item() {
    let (mut g, _f) = mk();
    t::ftco_800ce620(&mut *g);
    t::ftco_800ce650(&mut *g);
    t::ftco_fireflowershoot_anim(&mut *g);
    t::ftco_fireflowershootair_anim(&mut *g);
}
