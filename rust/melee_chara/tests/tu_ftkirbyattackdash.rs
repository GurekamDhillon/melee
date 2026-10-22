//! tu::ftkirbyattackdash (src/melee/ft/kinds/ftKirby/ftkirbyattackdash.c): no fused sites and no pure
//! math, so a smoke test of the new callbacks through recording hooks (Fighter fields, call order).
#![allow(static_mut_refs)]

use melee_chara::stubs as st;
use melee_chara::tu::ftkirbyattackdash as t;
use melee_types::*;

static mut LOG: std::vec::Vec<String> = std::vec::Vec::new();
fn log(s: &str) {
    unsafe { LOG.push(s.to_string()) }
}

#[test]
fn smoke() {
    let mut f: Box<Fighter> = Box::new(Fighter::zeroed());
    let mut g: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    let mut parts: std::vec::Vec<FighterBone> = (0..4).map(|_| unsafe { core::mem::zeroed() }).collect();
    let mut jobj: Box<HSD_JObj> = Box::new(unsafe { core::mem::zeroed() });
    jobj.rotate.x = 1.5;
    parts[0].joint = &mut *jobj;
    f.parts = parts.as_mut_ptr();
    f.co_attrs.gravity = 0.25;
    f.co_attrs.terminal_velocity = 2.0;
    g.user_data = &mut *f as *mut Fighter as *mut _;
    let gp: *mut HSD_GObj = &mut *g;

    // AirPhys: ft_80085204(gobj), then ftCommon_FallBasic(fp) (gravity 0.25, terminal 2.0).
    unsafe {
        st::FT_80085204 = Some(|_| log("85204"));
    }
    t::ftKb_AttackDashAir_Phys(gp);
    assert_eq!(f.self_vel.y, -0.25);
    t::ftKb_AttackDashAir_Phys(gp);
    assert_eq!(f.self_vel.y, -0.5);
    // IASA: empty.
    t::ftKb_AttackDashAir_IASA(gp);
    // 800F21C0: ftPartGetRotX(fp, 0).
    assert_eq!(t::ftKb_AttackDashAir_800F21C0(gp), 1.5);
    assert_eq!(unsafe { LOG.join(",") }, "85204,85204");
    drop(parts);
}
