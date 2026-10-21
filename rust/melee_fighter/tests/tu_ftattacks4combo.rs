//! Smoke tests for tu/ftattacks4combo.rs (no float math; the C has no fused sites).
use melee_fighter::tu::ftattacks4combo as m;
use melee_fighter::types::{Fighter, HSD_GObj, Ft_Kind_Mario};

#[test]
fn no_input_no_combo() {
    let mut fi: Fighter = unsafe { core::mem::zeroed() };
    let mut g: HSD_GObj = unsafe { core::mem::zeroed() };
    g.user_data = &mut fi as *mut Fighter as *mut _;
    assert!(!m::ftCo_800CECE8(&mut g));
    unsafe { (*(g.user_data as *mut Fighter)).cmd_vars[0] = 1 };
    assert!(!m::ftCo_800CECE8(&mut g));
    // IASA with allow_interrupt clear does nothing.
    m::ftCo_AttackS42_IASA(&mut g);
}

#[test]
#[should_panic(expected = "smash42")]
fn non_link_asserts() {
    let mut fi: Fighter = unsafe { core::mem::zeroed() };
    fi.kind = Ft_Kind_Mario;
    let mut g: HSD_GObj = unsafe { core::mem::zeroed() };
    g.user_data = &mut fi as *mut Fighter as *mut _;
    m::ftCo_800CED30(&mut g);
}
