//! Smoke test for tu::ftkirbyspeciallw (stateful) through recording hooks. The one fused site
//! (ftKb_SpecialHi_800F3570: `normal_y_slide * sign + pos_x_alias`) multiplies by exactly +-1, so
//! it cannot be told from the unfused form by any input; the test pins the value it passes.
//! One #[test]: the hooks are process globals.
#![allow(static_mut_refs)]

use melee_chara::stubs as st;
use melee_chara::tu::ftkirbyspeciallw as t;
use melee_types::*;

static mut LOG: std::vec::Vec<String> = std::vec::Vec::new();
fn log(s: String) {
    unsafe { LOG.push(s) }
}
fn take() -> std::vec::Vec<String> {
    unsafe { core::mem::take(&mut LOG) }
}

fn v3(v: Vec3) -> (f32, f32, f32) {
    (v.x, v.y, v.z)
}

#[test]
fn smoke() {
    unsafe { smoke_inner() }
}

#[allow(unsafe_op_in_unsafe_fn)]
unsafe fn smoke_inner() {
    let mut f: Box<Fighter> = Box::new(Fighter::zeroed());
    let mut g: Box<HSD_GObj> = Box::new(core::mem::zeroed());
    let da: &'static mut ftKb_DatAttrs = Box::leak(Box::new(ftKb_DatAttrs::zeroed()));
    let mut bones: std::vec::Vec<FighterBone> = (0..64).map(|_| core::mem::zeroed()).collect();
    f.parts = bones.as_mut_ptr();
    f.dat_attrs = da as *mut _ as *mut _;
    g.user_data = &mut *f as *mut Fighter as *mut _;
    let gp: *mut HSD_GObj = &mut *g;
    st::FIGHTER_CHANGEMOTIONSTATE = Some(|_, ms, fl, af, sp, _, _| log(format!("ms {ms} {fl:x} {af} {sp}")));
    st::FTANIM_8006EBA4 = Some(|_| {});
    st::FTANIM_SETANIMRATE = Some(|_, r| log(format!("rate {r}")));
    st::FTANIM_ISFRAMESREMAINING = Some(|_| 0);
    st::FTCO_FALL_ENTER = Some(|_| log("Fall".into()));
    st::FTCO_80096900 = Some(|_, _, _, _, m, lag| log(format!("fallspecial {m} {lag}")));
    st::FT_80082708 = Some(|_| 0);
    st::FT_80081D0C = Some(|_| 1);
    st::FT_80084FA8 = Some(|_| log("84FA8".into()));
    st::FT_80085134 = Some(|_| log("85134".into()));
    st::FTPARTS_80074B0C = Some(|_, m, v| log(format!("part {m} {v}")));
    st::FTCOMMON_8007EBAC = Some(|_, a, b| log(format!("ebac {a} {b}")));
    t::ext::FTCOMMON_8007F5CC = Some(|_, v| log(format!("f5cc {v}")));
    t::ext::HSD_RANDI = Some(|n| {
        log(format!("randi {n}"));
        2
    });
    t::ext::FT_GETGROUNDFRICTIONMULTIPLIER = Some(|_| 1.0);
    t::ext::MPCHECKALLREMAP = Some(|_, _, _, normal, _, _, x0, y0, x1, y1| {
        log(format!("mp {x0} {y0} {x1} {y1}"));
        unsafe { *normal = Vec3 { x: -0.5, y: 0.5, z: 0.0 } };
        true
    });
    t::ext::FT_PLAYSFX = Some(|_, id, _, _| log(format!("sfx {id:x}")));
    t::ext::CAMERA_REQUESTQUAKE = Some(|k, _| log(format!("quake {k}")));
    t::ext::EFLIB_DESTROYALL = Some(|_| log("destroyall".into()));
    t::ext::EFSYNC_SPAWN = Some(|id, _, va| {
        log(format!("efsync {id:x} {}", va.len()));
        core::ptr::null_mut()
    });
    t::ext::FTPARTSETROTX = Some(|_, p, r| log(format!("rotx {p} {r}")));

    da.speciallw_max_time_in_stone = 100;
    da.speciallw_min_time_in_stone = 30;
    da.speciallw_hp = 25;
    da.speciallw_freefall_toggle = 0.0;
    da.speciallw_min_slant_angle_slide = 0.1;
    da.speciallw_max_slant_angle_slide = 0.5;
    da.speciallw_slide_max_speed = 4.0;
    da.speciallw_slide_acceleration = 0.5;
    da.speciallw_gravity = 1.25;

    // Enter: timers, vec init, state change, efSync on the YRotN joint; x221C_b4 survives the change.
    f.cmd_vars = [1, 2, 3, 4];
    t::ftKb_SpecialLw_Enter(gp);
    assert_eq!(take(), ["ms 393 0 0 1", "efsync 49b 1"]);
    assert_eq!(f.cmd_vars, [0; 4]);
    assert_eq!((f.mv.kb.speciallw.x0, f.mv.kb.speciallw.x2, f.mv.kb.speciallw.x10), (100, 30, -1));
    assert_eq!(v3(f.mv.kb.speciallw.x24[3]), (0.0, 1.0, 0.0));
    assert!(f.take_dmg_cb.is_some() && f.death2_cb.is_some());
    t::ftKb_SpecialAirLw_Enter(gp);
    assert_eq!(take(), ["ms 396 0 0 1", "efsync 49b 1"]);

    // Start Anim: cmd 1 starts the fade-in (hp, random costume pick); the fade steps the counter.
    f.mv.kb.speciallw.x8 = 0;
    f.mv.kb.speciallw.xC = 0;
    f.cmd_vars[0] = 1;
    f.u.kb.hat.x4 = 3;
    t::ftKb_SpecialLw1_Anim(gp);
    let l = take();
    assert_eq!(l[0], "randi 4");
    assert_eq!(f.dmg.x1834, 25.0);
    assert_eq!(f.mv.kb.specialhi.x10.i, (3 + 2) % 5 + 1);
    assert_eq!(f.u.kb.hat.x4, 1);
    assert_eq!(f.mv.kb.specialhi.xC, 1);
    // The counter was reset to 0, then stepped by the fade-in (x8 == 1): flag 0 hides the model.
    assert_eq!(f.mv.kb.specialhi.x14, 1);
    assert_eq!(l[1], "part 0 2"); // ints[1] visible at x14 == 1
    assert_eq!(l.last().unwrap(), "rate 0");
    assert!(l.contains(&"ms 394 4440012 0 0".to_string()));
    // Fade-out reaches the end of the table, which clears the fade mode.
    f.mv.kb.specialhi.x8.i = 2;
    f.mv.kb.specialhi.x14 = 0;
    t::ftKb_SpecialHi_800F331C(gp);
    assert_eq!(f.mv.kb.specialhi.x14, -1);
    assert_eq!(unsafe { f.mv.kb.specialhi.x8.i }, 0);
    take();

    // IASA: the timer counts down; a B press (0x200) inside the window, or an expired timer, ends the Stone.
    f.mv.kb.speciallw.x0 = 90;
    t::ftKb_SpecialLw_IASA(gp);
    assert_eq!(f.mv.kb.speciallw.x0, 89);
    assert!(take().is_empty());
    f.mv.kb.speciallw.x0 = 60;
    f.input.pressed_buttons = 0x200;
    t::ftKb_SpecialAirLw_IASA(gp);
    assert_eq!(take(), [format!("ms {} 4440012 0 1", ftKb_MS_SpecialAirLwEnd), "efsync 49c 1".to_string()]);
    f.mv.kb.speciallw.x0 = 0;
    t::ftKb_SpecialLw_IASA(gp);
    assert_eq!(take().len(), 2 + 0); // ground IASA also runs 8007D5D4 (silent) then the state change + efSync

    // End Anim: no freefall toggle falls, a toggle goes to the special fall with the landing lag.
    t::ftKb_SpecialLwEnd_Anim(gp);
    assert!(take().contains(&"Fall".to_string()));
    da.speciallw_freefall_toggle = 20.0;
    t::ftKb_SpecialAirLwEnd_Anim(gp);
    assert!(take().contains(&"fallspecial 1 20".to_string()));
    assert_eq!(f.dmg.x1834, 0.0);

    // Phys.
    t::ftKb_SpecialAirLw_Phys(gp);
    assert_eq!(f.self_vel.y, -1.25);
    assert_eq!(take(), ["85134"]);
    f.self_vel.y = 9.0;

    // Stone slide bookkeeping: the 4-frame history shifts and averages the floor normals.
    f.coll_data.floor.normal = Vec3 { x: 0.0, y: 1.0, z: 0.0 };
    t::ftKb_SpecialHi_800F36DC(gp);
    f.mv.kb.speciallw.x18 = Vec3 { x: 0.0, y: 1.0, z: 0.0 };
    f.mv.kb.speciallw.x84 = 0.5;
    t::ftKb_SpecialHi_800F37EC(gp);
    assert_eq!(f.mv.kb.speciallw.x88[0], 0.5);
    assert_eq!(f.mv.kb.speciallw.x88[4], 0.5 * 0.25);
    assert_eq!(v3(f.mv.kb.speciallw.x54[0]), (0.0, 1.0, 0.0));
    assert_eq!(f.mv.kb.speciallw.x88[8], 1.0); // one sample only: nothing to compare against
    f.mv.kb.speciallw.x84 = 0.0;
    t::ftKb_SpecialHi_800F37EC(gp);
    assert_eq!(f.mv.kb.speciallw.x88[..4], [0.0, 0.5, 0.0, 0.0]);
    // A near-flat older sample after a slanted one: the damping drops to 0.9.
    assert_eq!(f.mv.kb.speciallw.x88[8], 0.9);

    // Floor probe: slanted floor probes the line and zeroes the slide when the far side is uphill.
    f.coll_data.floor.normal = Vec3 { x: 0.6, y: 0.8, z: 0.0 };
    f.cur_pos = Vec3 { x: 10.0, y: 20.0, z: 0.0 };
    f.facing_dir = 1.0;
    f.gr_vel = 2.0;
    t::ftKb_SpecialHi_800F3570(gp);
    // sp1C.x = -0.5 < 0 with a positive normal x: the normal snaps to the unit-Y vector.
    assert_eq!(take(), ["mp 10 24 13.2 16"]);
    assert_eq!(v3(f.mv.kb.specialhi.x18), (0.0, 1.0, 0.0));
    assert_eq!((f.gr_vel, f.mv.kb.specialhi.xC4), (0.0, 0.0));

    // Stone contact with the ground: reset vectors, damage callbacks, state 0x18C.
    f.cur_anim_frame = 4.0;
    t::ftKb_SpecialLw1_Coll(gp);
    assert_eq!(take(), ["ms 396 c4c5092 4 1", "rotx 0 0"]);
}
