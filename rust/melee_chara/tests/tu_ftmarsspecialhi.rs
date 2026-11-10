//! Smoke test for tu::ftmarsspecialhi (no fused sites) through recording hooks; the IASA angle
//! is checked against the C expression (double 1.0 literal, f32 rounding at the assignment).
#![allow(static_mut_refs)]

use melee_chara::stubs as st;
use melee_chara::tu::ftmarsspecialhi as t;
use melee_types::*;

static mut LOG: std::vec::Vec<String> = std::vec::Vec::new();
fn log(s: String) {
    unsafe { LOG.push(s) }
}
fn take() -> std::vec::Vec<String> {
    unsafe { core::mem::take(&mut LOG) }
}

#[test]
fn smoke() {
    let mut f: Box<Fighter> = Box::new(Fighter::zeroed());
    let mut g: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    let da: &'static mut MarsAttributes = Box::leak(Box::new(MarsAttributes::zeroed()));
    f.dat_attrs = da as *mut _ as *mut _;
    g.user_data = &mut *f as *mut Fighter as *mut _;
    let gp: *mut HSD_GObj = &mut *g;
    unsafe {
        st::FIGHTER_CHANGEMOTIONSTATE = Some(|_, ms, fl, _, sp, _, _| log(format!("ms {ms} {fl:x} {sp}")));
        st::FTANIM_8006EBA4 = Some(|_| log("anim".into()));
        st::FT_80085154 = Some(|_| log("85154".into()));
        st::FT_80084FA8 = Some(|_| log("84FA8".into()));
        st::FT_80084104 = Some(|_| log("84104".into()));
        t::ext::FT_80083B68 = Some(|_| log("83B68".into()));
        t::ext::FT_800831CC = Some(|_, _, _| log("831CC".into()));
        t::ext::FTPARTSETROTY = Some(|_, p, r| log(format!("roty {p} {r}")));
    }

    // AirHi enter: clears vars, scales the x velocity.
    da.x3C = 0.5;
    f.self_vel.x = 8.0;
    f.self_vel.y = 3.0;
    f.cmd_vars = [5, 5, 5, 5];
    t::ftMs_SpecialAirHi_Enter(gp);
    assert_eq!(take(), ["ms 368 0 1", "anim"]);
    assert_eq!((f.self_vel.x, f.self_vel.y), (4.0, 0.0));
    assert_eq!(f.cmd_vars[..3], [0, 0, 0]);

    // IASA angle: da->x38 * ((|x| - x34) / (1.0 - x34)) in double, then * deg->rad in float.
    da.x34 = 0.3;
    da.x38 = 40.0;
    da.x30 = 0.9;
    f.cmd_vars[0] = 0;
    f.lstick_angle = 0.0;
    for &sx in &[0.7f32, -0.55, 0.31, 1.0, 0.29] {
        f.input.lstick[0].x = sx;
        f.lstick_angle = 0.0;
        t::ftMs_SpecialHi_IASA(gp);
        let ax = if sx < 0.0 { -sx } else { sx };
        let want = if ax > da.x34 {
            let t1 = (da.x38 as f64 * (((ax - da.x34) as f64) / (1.0f64 - da.x34 as f64))) as f32;
            let d2r = 3.14159265359f32 / 180.0f32;
            let t1 = if sx > 0.0 { -(d2r * t1) } else { d2r * t1 };
            if t1.abs() > 0.0 { t1 } else { 0.0 }
        } else {
            0.0
        };
        assert_eq!(f.lstick_angle.to_bits(), want.to_bits(), "stick {sx}");
    }
    // A smaller magnitude never replaces a larger stored angle.
    f.input.lstick[0].x = 0.4;
    f.lstick_angle = -5.0;
    t::ftMs_SpecialAirHi_IASA(gp);
    assert_eq!(f.lstick_angle, -5.0);
    // Throw flag set and stick past x30: facing update + 90 degree turn toward facing.
    f.input.lstick[0].x = 0.95;
    f.facing_dir = 1.0;
    unsafe { f.x2210.x0.set_throw_flags_b3(1) };
    take();
    t::ftMs_SpecialHi_IASA(gp);
    assert_eq!(unsafe { f.x2210.x0.throw_flags_b3() }, 0);
    let l = take();
    assert_eq!(l.len(), 1);
    assert!(l[0].starts_with("roty 0 "));

    // Phys: falling branch, then the air-drift branch.
    f.ground_or_air = GA_Air;
    f.cmd_vars[2] = 0;
    f.self_vel.x = -2.0;
    f.facing_dir = 1.0;
    f.x6A4_transNOffset.y = -1.0;
    t::ftMs_SpecialHi_Phys(gp);
    assert_eq!(take(), ["85154"]);
    assert_eq!(f.self_vel.x, 2.0);
    assert_eq!(f.cmd_vars[2], 1);
    f.ground_or_air = GA_Ground;
    t::ftMs_SpecialHi_Phys(gp);
    assert_eq!(take(), ["84FA8"]);

    // Coll: ground -> 84104; air cases.
    t::ftMs_SpecialHi_Coll(gp);
    assert_eq!(take(), ["84104"]);
    f.ground_or_air = GA_Air;
    f.cmd_vars[0] = 0;
    t::ftMs_SpecialAirHi_Coll(gp);
    assert_eq!(take(), ["83B68"]);
    f.cmd_vars[0] = 1;
    f.self_vel.y = -1.0;
    f.cmd_vars[1] = 0;
    t::ftMs_SpecialHi_Coll(gp);
    assert_eq!((take(), f.cmd_vars[1]), (vec!["83B68".to_string()], 1));
    t::ftMs_SpecialHi_Coll(gp);
    assert_eq!(take(), ["831CC"]);
}
