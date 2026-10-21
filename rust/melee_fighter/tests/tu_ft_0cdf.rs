//! Smoke test for src/tu/ft_0cdf.rs (accessory callback without an item is a no-op).
use melee_fighter::tu::ft_0cdf as t;
use melee_fighter::types::*;
fn mk() -> (Box<HSD_GObj>, Box<Fighter>) {
    let mut f: Box<Fighter> = Box::new(unsafe { core::mem::zeroed() });
    let mut g: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    g.user_data = (&mut *f as *mut Fighter).cast();
    (g, f)
}
#[test]
fn lgun_no_item() {
    let (mut g, _f) = mk();
    t::ftco_800ce1d4(&mut *g);
    t::ftco_lgunshoot_iasa(&mut *g);
    t::ftco_lgunshootair_iasa(&mut *g);
}
