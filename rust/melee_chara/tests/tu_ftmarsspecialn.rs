//! Smoke test for tu::ftmarsspecialn (no fused sites) through recording hooks.
#![allow(static_mut_refs)]

use melee_chara::stubs as st;
use melee_chara::tu::ftmarsspecialn as t;
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
        st::FIGHTER_CHANGEMOTIONSTATE = Some(|_, ms, fl, s, sp, _, _| log(format!("ms {ms} {fl:x} {s} {sp}")));
        st::FTANIM_8006EBA4 = Some(|_| log("anim".into()));
        st::FTANIM_ISFRAMESREMAINING = Some(|_| 1);
        st::FT_8008A2BC = Some(|_| log("wait".into()));
        st::FTCO_FALL_ENTER = Some(|_| log("fall".into()));
        st::FT_80082708 = Some(|_| 0);
        st::FT_80081D0C = Some(|_| 1);
        st::FTCOMMON_8007D7FC = Some(|_| log("7D7FC".into()));
        t::ext::FTCOLL_8007ABD0 = Some(|_, d, _| log(format!("dmg {d}")));
        t::ext::LB_8000B1CC = Some(|_, _, out| *out = Vec3 { x: 1.0, y: 2.0, z: 3.0 });
        t::ext::LB_800119DC = Some(|_, a, b, c, d| log(format!("shake {a} {b} {c} {}", d.to_bits())));
    }

    // Enters.
    da.specialn_friction = 2.0;
    f.gr_vel = 5.0;
    t::ftMs_SpecialN_Enter(gp);
    assert_eq!(f.gr_vel, 2.5);
    assert!(f.x21EC.is_some());
    assert_eq!(take(), ["ms 341 0 0 1", "anim"]);
    f.self_vel = Vec3 { x: 4.0, y: -2.0, z: 0.0 };
    t::ftMs_SpecialAirN_Enter(gp);
    assert_eq!((f.self_vel.x, f.self_vel.y), (2.0, 0.0));
    assert_eq!(take(), ["ms 345 0 0 1", "anim"]);

    // Loop anim: pulse on multiples of 30, release past x0 * 30 frames.
    da.x0 = 1;
    unsafe { f.mv.ms.specialn.cur_frame = 0 };
    f.cmd_vars[0] = 0;
    t::ftMs_SpecialNLoop_Anim(gp);
    assert_eq!(unsafe { f.mv.ms.specialn.cur_frame }, 1);
    assert_eq!(take(), [format!("shake 10 0.5 0.05 {}", (60.0f32 * 0.01745329252f32).to_bits())]);
    unsafe { f.mv.ms.specialn.cur_frame = 30 };
    t::ftMs_SpecialAirNLoop_Anim(gp);
    assert_eq!(f.cmd_vars[0], 1);
    let l = take();
    assert_eq!(l.len(), 2);
    assert_eq!(l[1], "ms 348 0 1 1");
    assert!(f.accessory4_cb.is_some());

    // Loop IASA: releasing B ends the charge, holding does not.
    f.input.held_buttons[0] = HSD_PAD_B as u32;
    t::ftMs_SpecialNLoop_IASA(gp);
    assert!(take().is_empty());
    f.input.held_buttons[0] = 0;
    f.cmd_vars[0] = 1;
    t::ftMs_SpecialNLoop_IASA(gp);
    assert_eq!(f.cmd_vars[0], 0);
    assert_eq!(take(), ["ms 343 0 1 1"]);

    // End anim: damage = x4 + (cur_frame / 30) * x8 on live hitboxes while uncharged-release.
    da.x4 = 5;
    da.x8 = 3;
    unsafe { f.mv.ms.specialn.cur_frame = 65 };
    f.cmd_vars[0] = 0;
    f.x914[2].state = HitCapsule_Enabled;
    f.cur_anim_frame = 9.0;
    t::ftMs_SpecialNEnd_Anim(gp);
    let l = take();
    assert_eq!(l[0], "dmg 11");
    assert!(l[1].starts_with("shake 120 "));
    f.cur_anim_frame = 3.0;
    unsafe { st::FTANIM_ISFRAMESREMAINING = Some(|_| 0) };
    t::ftMs_SpecialAirNEnd_Anim(gp);
    assert_eq!(take(), ["dmg 11", "fall"]);

    // End coll picks End0 / End1 by the script flag and re-arms hitlag callbacks when set.
    f.cmd_vars[0] = 1;
    f.set_x2219_b0(1);
    t::ftMs_SpecialNEnd_Coll(gp);
    let l = take();
    assert_eq!(l, ["ms 348 c4c508e 3 1"]);
    assert!(f.pre_hitlag_cb.is_some());
}
