//! tu::ftco_0dc2 smoke tests: the empty IASA callbacks run, the Coll callbacks skip their
//! ft_081B call when x2226_b2 is set, and entering a state reaches the unported motion change.

use melee_fighter::tu::ftco_0dc2 as t;
use melee_fighter::types::*;

fn world() -> (Box<Fighter>, Box<HSD_GObj>) {
    let mut f = Box::new(Fighter::zeroed());
    let mut go: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    go.user_data = (&mut *f as *mut Fighter).cast();
    (f, go)
}

#[test]
fn coll_skips_when_flagged() {
    let (mut f, mut go) = world();
    f.set_x2226_b2(1);
    t::ftco_capturedamagehi_coll(&mut *go);
    t::ftco_capturedamagelw_coll(&mut *go);
    t::ftco_capturedamagehi_iasa(&mut *go);
    t::ftco_capturedamagelw_iasa(&mut *go);
}

#[test]
#[should_panic(expected = "ft_80083C00")]
fn coll_reaches_ft_80083c00() {
    let (_f, mut go) = world();
    t::ftco_capturedamagehi_coll(&mut *go);
}

#[test]
#[should_panic(expected = "Fighter_ChangeMotionState")]
fn enter_reaches_motion_change() {
    let (_f, mut go) = world();
    t::ftco_800dc284(&mut *go);
}
