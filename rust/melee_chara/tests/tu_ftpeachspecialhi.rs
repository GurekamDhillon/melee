//! tu::ftpeachspecialhi smoke test through recording hooks.
#![allow(static_mut_refs)]
use melee_chara::stubs as st;
use melee_chara::tu::ftpeachspecialhi as t;
use melee_types::*;
static mut LOG: std::vec::Vec<String> = std::vec::Vec::new();
fn log(s: String) {
    unsafe { LOG.push(s) }
}
#[test]
fn smoke() {
    let mut f: Box<Fighter> = Box::new(Fighter::zeroed());
    let mut g: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    g.user_data = &mut *f as *mut Fighter as *mut _;
    let gp: *mut HSD_GObj = &mut *g;
    let da: &'static mut ftPe_DatAttrs = Box::leak(Box::new(unsafe { core::mem::zeroed() }));
    f.dat_attrs = da as *mut _ as *mut _;
    unsafe {
        st::FT_80085154 = Some(|_| log("85154".into()));
        st::FT_80084FA8 = Some(|_| log("84FA8".into()));
        st::FTCO_LANDINGFALLSPECIAL_ENTER = Some(|_, _, lag| log(format!("land {lag}")));
    }
    // Phys: ground vs air dispatch, then the air-start scale by x8C once the move is active.
    f.ground_or_air = GA_Ground;
    t::ftPe_SpecialHiStart_Phys(gp);
    f.ground_or_air = GA_Air;
    t::ftPe_SpecialHiStart_Phys(gp);
    f.cmd_vars[0] = 1;
    da.x8C = 0.5;
    f.self_vel = Vec { x: 4.0, y: -2.0, z: 1.0 };
    t::ftPe_SpecialAirHiStart_Phys(gp);
    assert_eq!((f.self_vel.x, f.self_vel.y, f.self_vel.z), (2.0, -1.0, 0.5));
    da.x74 = 9.0;
    t::ftPe_SpecialHi_8011DD8C(gp);
    assert_eq!(unsafe { LOG.join(",") }, "84FA8,85154,85154,land 9");
}
