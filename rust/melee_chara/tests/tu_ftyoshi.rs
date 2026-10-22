//! Smoke test of tu::ftyoshi and tu::ftyoshiguard: the paths that touch no unported callee.
//! No C oracle for these yet (the state functions need recorders for the HSD matanim calls).
use melee_chara::tu::{ftyoshi as y, ftyoshiguard as yg};
use melee_types::*;

#[test]
fn smoke() {
    let mut f: Box<Fighter> = Box::new(Fighter::zeroed());
    let da: &'static mut ftYoshiAttributes = Box::leak(Box::new(ftYoshiAttributes::zeroed()));
    da.x120 = 12.5;
    f.dat_attrs = da as *mut _ as *mut _;
    assert_eq!(y::ftYs_Init_8012BAC0(&f), 12.5);

    let mut g: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    g.user_data = &mut *f as *mut Fighter as *mut _;
    let gp: *mut HSD_GObj = &mut *g;
    // Both shield sets are null, so the matanim loops are no-ops.
    y::ftYs_Init_8012B918(gp);
    // No button held: the shield-button check is false and changes nothing.
    assert!(!yg::ftYs_Shield_8012CC1C(gp));
    assert_eq!(y::ftYs_Init_8012BAC0(&f), 12.5);
}
