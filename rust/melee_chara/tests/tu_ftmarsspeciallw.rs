//! Smoke test for tu::ftmarsspeciallw (no fused sites) through recording hooks.
#![allow(static_mut_refs)]

use melee_chara::stubs as st;
use melee_chara::tu::ftmarsspeciallw as t;
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
    let mut parts: std::vec::Vec<FighterBone> = (0..80).map(|_| unsafe { core::mem::zeroed() }).collect();
    f.parts = parts.as_mut_ptr();
    let mut p2j: std::vec::Vec<u8> = (0..80u8).collect();
    let mut tbl: FighterPartsTable = unsafe { core::mem::zeroed() };
    tbl.part_to_joint = p2j.as_mut_ptr();
    let mut tbls: std::vec::Vec<*mut FighterPartsTable> = vec![&mut tbl; 40];
    unsafe {
        melee_fighter::hooks::FT_PARTS_TABLE = tbls.as_mut_ptr();
        st::FIGHTER_CHANGEMOTIONSTATE = Some(|_, ms, fl, st_, sp, _, _| log(format!("ms {ms} {fl:x} {st_} {sp}")));
        st::FTANIM_8006EBA4 = Some(|_| log("anim".into()));
        st::FTANIM_ISFRAMESREMAINING = Some(|_| 0);
        st::FT_8008A2BC = Some(|_| log("wait".into()));
        st::FTCO_FALL_ENTER = Some(|_| log("fall".into()));
        st::FT_800827A0 = Some(|_| 0);
        st::FT_80081D0C = Some(|_| 1);
        st::FTCOMMON_8007D7FC = Some(|_| log("7D7FC".into()));
        st::FTCOLL_8007AEE0 = Some(|_| log("AEE0".into()));
        t::ext::FTCOLL_8007B1B8 = Some(|_, d, cb| log(format!("shield {} {}", !d.is_null(), cb.is_some())));
        t::ext::FTCOLL_8007ABD0 = Some(|h, d, _| log(format!("dmg {d} {}", (*h).state)));
        t::ext::LB_8000B1CC = Some(|_, _, out| *out = Vec3 { x: 1.0, y: 2.0, z: 3.0 });
        t::ext::LB_800119DC = Some(|p, a, b, c, d| log(format!("shake {} {a} {b} {c} {}", (*p).x, d.to_bits())));
        t::ext::EFSYNC_SPAWN = Some(|id, _, a| {
            log(format!("ef {id} {}", a.len()));
            core::ptr::null_mut()
        });
    }

    // Enter: ground / air.
    da.x4C = 2.0;
    f.self_vel.x = 6.0;
    f.self_vel.y = 9.0;
    t::ftMs_SpecialLw_Enter(gp);
    assert_eq!(take(), ["ms 369 0 0 1", "anim"]);
    assert_eq!((f.self_vel.y, f.cmd_vars[1], unsafe { f.mv.ms.speciallw.x0 }), (0.0, 0, 0));
    t::ftMs_SpecialAirLw_Enter(gp);
    assert_eq!(f.self_vel.x, 3.0);
    assert_eq!(take(), ["ms 371 0 0 1", "anim"]);

    // Anim: shield window opens when the script sets cmd_vars[1] = 1.
    da.x60 = 12.0;
    f.cmd_vars[1] = 1;
    t::ftMs_SpecialLw_Anim(gp);
    assert_eq!(take(), ["shield true true", "wait"]);
    assert_eq!((f.cmd_vars[1], f.anon_221b.x221B_b1(), f.shield_unk0, f.shield_unk1), (2, 1, 12.0, 12.0));
    f.cmd_vars[1] = 0;
    f.anon_221b.set_x221B_b0(1);
    t::ftMs_SpecialAirLw_Anim(gp);
    assert_eq!(f.anon_221b.x221B_b0(), 0);
    assert_eq!(take(), ["fall"]);

    // Phys / Coll.
    da.x54 = 0.1;
    da.x58 = 1.0;
    da.x50 = 0.2;
    f.self_vel = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    f.co_attrs.gravity = 0.1;
    t::ftMs_SpecialAirLw_Phys(gp);
    assert_eq!(take(), ["AEE0"]);
    assert_eq!(f.self_vel.y, -0.1);
    f.cmd_vars[1] = 2;
    t::ftMs_SpecialAirLw_Coll(gp);
    let l = take();
    assert_eq!(l[0], "7D7FC");
    assert!(l[1].starts_with("ms 369 c4c508c "));
    assert_eq!(l[2], "shield true true");

    // Roy's counter hit scales the live hitboxes; Marth's does not.
    f.mv.ms.speciallw.x0 = 30;
    f.x914[1].state = HitCapsule_Enabled;
    unsafe { melee_fighter::hooks::FTLIB_GET_KIND = Some(|_| Ft_Kind_Emblem) };
    t::ftMs_SpecialLwHit_Anim(gp);
    assert_eq!(take(), ["dmg 30 1", "wait"]);
    unsafe { melee_fighter::hooks::FTLIB_GET_KIND = Some(|_| Ft_Kind_Mars) };
    t::ftMs_SpecialAirLwHit_Anim(gp);
    assert_eq!(take(), ["fall"]);

    // Shield-hit callback: damage taken, facing restored, effect once, callbacks cleared.
    da.x5C = 1.5;
    f.x19A4 = 10;
    f.specialn_facing_dir = -1.0;
    f.ground_or_air = GA_Ground;
    f.pre_hitlag_cb = Some(melee_chara::tu::ftmarsspeciallw::cb_ftMs_SpecialLw_80139140);
    t::ftMs_SpecialLw_80139140(gp);
    assert_eq!(unsafe { f.mv.ms.speciallw.x0 }, 15);
    assert_eq!(f.facing_dir, -1.0);
    assert_eq!(f.x2219_b0(), 1);
    let l = take();
    assert_eq!(l[0], format!("shake 1 120 {} {} {}", 0.9f32, 0.02f32, ((core::f64::consts::PI / 3.0) as f32).to_bits()));
    assert_eq!(l[1], "ms 370 0 0 1");
    assert_eq!(l[2], "ef 1265 2");
    assert!(f.pre_hitlag_cb.is_none() && f.accessory4_cb.is_none());
}
