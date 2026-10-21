//! Smoke test for src/tu/ft_0c88.rs (state bits only; paths that reach unported callees are
//! checked to panic, not run).
use melee_fighter::tu::ft_0c88 as t;
use melee_fighter::types::*;

fn mk() -> (Box<HSD_GObj>, Box<Fighter>) {
    let mut f: Box<Fighter> = Box::new(unsafe { core::mem::zeroed() });
    let mut g: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    g.user_data = (&mut *f as *mut Fighter).cast();
    (g, f)
}

#[test]
fn state_bits() {
    let (mut g, mut f) = mk();
    let gp: *mut HSD_GObj = &mut *g;
    f.set_x2226_b4(1);
    f.set_x2226_b5(1);
    f.set_x2226_b7(1);
    f.x2030 = 9;
    f.smash_attrs.x2134_vibrateFrame = 4;
    t::ftco_800c88a0(&mut f);
    assert_eq!((f.x2226_b4(), f.x2226_b5(), f.x2226_b7(), f.x2030, f.smash_attrs.x2134_vibrateFrame), (0, 0, 0, 0, 0));

    t::ftco_800c8b60(&mut f, 3, 7);
    assert_eq!((f.x2226_b5(), f.smash_attrs.x2134_vibrateFrame), (1, 7));
    t::fn_800c8b1c(gp);
    assert_eq!(f.smash_attrs.x2134_vibrateFrame, 0);

    f.set_x2226_b6(1);
    assert!(!t::ftco_800c8b2c(&mut f, 0x7C, false));

    // b4 set, b7 clear: only x2030 is written.
    f.set_x2226_b4(1);
    t::ftco_800c88d4(gp, 5, true);
    assert_eq!(f.x2030, 5);

    // b3 set: ftCo_800C8A64 only clears x2030.
    f.set_x2226_b3(1);
    t::ftco_800c8a64(gp);
    assert_eq!((f.x2030, f.x2226_b4()), (0, 1));

    f.set_x2226_b5(0);
    t::ftco_800c8af0(&mut f);
    f.set_x2226_b5(1);
    assert!(std::panic::catch_unwind(move || {
        let mut f2: Fighter = unsafe { core::mem::zeroed() };
        f2.set_x2226_b5(1);
        t::ftco_800c8af0(&mut f2);
    })
    .is_err());
}
