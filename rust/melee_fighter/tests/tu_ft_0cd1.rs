//! Smoke test for src/tu/ft_0cd1.rs (ftCo_800CD278 is checked bit-exact in tu_ft_0d27.rs).
use melee_fighter::tu::ft_0cd1 as t;
use melee_fighter::types::*;
fn mk() -> (Box<HSD_GObj>, Box<Fighter>) {
    let mut f: Box<Fighter> = Box::new(unsafe { core::mem::zeroed() });
    let mut g: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    g.user_data = (&mut *f as *mut Fighter).cast();
    (g, f)
}
#[test]
fn swing_helpers() {
    let (mut g, mut f) = mk();
    let gp: *mut HSD_GObj = &mut *g;
    t::ft_800cd31c(gp); // no item: no-op
    unsafe { f.mv.co.swing.x0 = 1 };
    f.input.held_buttons[0] = 0x100 as _;
    t::ftco_800cd204(gp); // A held, b3 clear: nothing
    assert_eq!(unsafe { f.mv.co.swing.x0 }, 1);
    f.input.held_buttons[0] = 0 as _;
    t::ftco_800cd204(gp);
    assert_eq!(unsafe { f.mv.co.swing.x0 }, 0);
    unsafe { f.anon_2210.anon_0.set_throw_flags_b0(1) };
    assert!(t::ft_check_throw_b0(&mut f) && !t::ft_check_throw_b0(&mut f));
    unsafe { f.anon_2210.anon_0.set_throw_flags_b4(1) };
    assert!(t::ft_check_throw_b4(&mut f) && !t::ft_check_throw_b4(&mut f));
}
