//! Smoke test for tu::ftcaptainspeciallw (stateful; no fused sites) through recording hooks.
//! One #[test]: the hooks are process globals.
#![allow(static_mut_refs)]

use melee_chara::stubs as st;
use melee_chara::tu::ftcaptainspeciallw as t;
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
    let da: &'static mut ftCaptain_DatAttrs = Box::leak(Box::new(ftCaptain_DatAttrs::zeroed()));
    f.dat_attrs = da as *mut _ as *mut _;
    g.user_data = &mut *f as *mut Fighter as *mut _;
    let gp: *mut HSD_GObj = &mut *g;
    unsafe {
        st::FT_80085134 = Some(|_| log("85134".into()));
        st::FT_80081D0C = Some(|_| 1);
        st::FTCOMMON_8007D7FC = Some(|_| log("7D7FC".into()));
        st::FIGHTER_CHANGEMOTIONSTATE = Some(|_, ms, fl, _, sp, _, _| log(format!("ms {ms} {fl:x} {sp}")));
        t::ext::FTPARTSETROTZ = Some(|_, p, r| log(format!("rotz {p} {r}")));
    }

    // On-hit: counts up to speciallw_unk2 and scales the friction each time.
    da.speciallw_unk2 = 1;
    da.speciallw_on_hit_spd_modifier = 0.5;
    unsafe {
        f.mv.ca.speciallw.x0 = 0;
        f.mv.ca.speciallw.friction = 1.0;
    }
    for _ in 0..4 {
        t::ftCa_SpecialHi_800E400C(gp);
    }
    unsafe {
        assert_eq!(f.mv.ca.speciallw.x0, 2);
        assert_eq!(f.mv.ca.speciallw.friction, 0.25);
    }

    // Aerial Phys: no flag set -> no effect; velocity scaled by the friction.
    f.ground_or_air = GA_Air;
    f.self_vel.x = 4.0;
    f.self_vel.y = -8.0;
    t::ftCa_SpecialLw_Phys(gp);
    assert_eq!(take(), ["rotz 0 0", "85134"]);
    assert_eq!((f.self_vel.x, f.self_vel.y), (1.0, -2.0));

    // Flag set, first time: Captain flame effect on the left foot at the angle in radians.
    let mut parts: std::vec::Vec<FighterBone> = (0..80).map(|_| unsafe { core::mem::zeroed() }).collect();
    f.parts = parts.as_mut_ptr();
    f.kind = Ft_Kind_Captain;
    f.motion_id = 0x167;
    da.speciallw_flame_particle_angle = 90.0;
    let mut p2j: std::vec::Vec<u8> = (0..80u8).collect();
    let mut tbl: FighterPartsTable = unsafe { core::mem::zeroed() };
    tbl.part_to_joint = p2j.as_mut_ptr();
    let mut tbls: std::vec::Vec<*mut FighterPartsTable> = vec![&mut tbl; 40];
    unsafe {
        melee_fighter::hooks::FT_PARTS_TABLE = tbls.as_mut_ptr();
        f.x2210.x0.set_throw_flags_b1(1);
        t::ext::EFASYNC_SPAWN = Some(|_, _, k, gfx, _, a| log(format!("ef {k} {gfx:x} {:08x}", (*a).to_bits())));
        melee_fighter::hooks::FTLIB_GET_KIND = Some(|_| Ft_Kind_Captain);
    }
    t::ftCa_SpecialAirLw_Phys(gp);
    assert_eq!(take(), ["85134".to_string(), format!("ef 3 490 {:08x}", (0.017453292f32 * 90.0).to_bits())]);
    assert_eq!(f.x2219_b0(), 1);
    assert_eq!(unsafe { f.x2210.x0.throw_flags_b1() }, 0);

    // Air collision reports landing -> SpecialAirLwEnd at the landing-lag rate.
    da.speciallw_landing_lag_mul = 2.0;
    f.cmd_vars = [1, 1, 1, 0];
    t::ftCa_SpecialAirLw_Coll(gp);
    assert_eq!(take(), ["7D7FC", "ms 360 0 2"]);
    assert_eq!(f.cmd_vars[..3], [0, 0, 0]);
}
