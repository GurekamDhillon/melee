//! Smoke test for tu::ftkirbyspecialhi (stateful; no fused sites) through recording hooks.
//! One #[test]: the hooks are process globals.
#![allow(static_mut_refs)]

use melee_chara::stubs as st;
use melee_chara::tu::ftkirbyspecialhi as t;
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
    let da: &'static mut ftKb_DatAttrs = Box::leak(Box::new(ftKb_DatAttrs::zeroed()));
    let mut bones: std::vec::Vec<FighterBone> = (0..4).map(|_| unsafe { core::mem::zeroed() }).collect();
    f.parts = bones.as_mut_ptr();
    f.dat_attrs = da as *mut _ as *mut _;
    g.user_data = &mut *f as *mut Fighter as *mut _;
    let gp: *mut HSD_GObj = &mut *g;
    unsafe {
        st::FIGHTER_CHANGEMOTIONSTATE = Some(|_, ms, fl, _, _, _, _| log(format!("ms {ms} {fl:x}")));
        st::FTANIM_8006EBA4 = Some(|_| {});
        st::FTANIM_ISFRAMESREMAINING = Some(|_| 0);
        st::FT_8008A2BC = Some(|_| log("Wait".into()));
        st::FTCO_FALL_ENTER = Some(|_| log("Fall".into()));
        st::FT_80082708 = Some(|_| 0);
        st::FT_80081D0C = Some(|_| 1);
        st::FT_CHECKGROUNDANDLEDGE = Some(|_, d| {
            log(format!("ledge {d}"));
            1
        });
        st::FT_80084FA8 = Some(|_| log("84FA8".into()));
        st::FT_80085134 = Some(|_| log("85134".into()));
        t::ext::FTPARTSETROTX = Some(|_, p, r| log(format!("rotx {p} {r}")));
        t::ext::FTPARTSETROTY = Some(|_, p, r| log(format!("roty {p} {r}")));
        t::ext::EFSYNC_SPAWN = Some(|id, _, va| {
            log(format!("efsync {id:x} {}", va.len()));
            core::ptr::null_mut()
        });
        t::ext::LB_8000B1CC = Some(|_, off, out| {
            assert!(off.is_null());
            unsafe { *out = Vec3 { x: 10.0, y: 20.0, z: 5.0 } };
        });
        t::ext::EFLIB_SETFLAGS = Some(|_, fl| log(format!("efflags {fl}")));
        t::ext::IT_8029BAB8 = Some(|_, pos, fd| log(format!("cutter {} {} {} {fd}", unsafe { (*pos).x }, unsafe { (*pos).y }, unsafe { (*pos).z })));
        t::ext::FTCLIFFCOMMON_80081370 = Some(|_| log("cliff".into()));
        t::ext::FT_80082578 = Some(|_| {
            log("82578".into());
            true
        });
    }

    // Enter: clears the vars, state change, effect, hitlag callbacks.
    f.cmd_vars = [1, 2, 3, 4];
    t::ftKb_SpecialHi_Enter(gp);
    assert_eq!(take(), ["ms 385 0", "efsync 494 0"]);
    assert_eq!(f.cmd_vars, [0; 4]);
    assert_eq!(f.x2219_b0(), 1);
    assert!(f.pre_hitlag_cb.is_some() && f.post_hitlag_cb.is_some());
    t::ftKb_SpecialAirHi_Enter(gp);
    assert_eq!(take(), ["ms 389 0", "efsync 494 0"]);

    // Anim: end of the Final Cutter landing goes to Wait on the ground, Fall in the air.
    f.ground_or_air = GA_Ground;
    t::ftKb_SpecialHi4_Anim(gp);
    assert_eq!(take(), ["rotx 0 0", "Wait"]);
    f.ground_or_air = GA_Air;
    t::ftKb_SpecialAirHiEnd_Anim(gp);
    assert_eq!(take(), ["rotx 0 0", "Fall"]);

    // Reverse IASA: stick opposite the facing past the range turns Kirby around once.
    da.specialhi_reverse_upb_stick_range = 0.5;
    f.facing_dir = 1.0;
    f.input.lstick[0].x = -0.9;
    f.cmd_vars = [0; 4];
    unsafe { f.mv.kb.specialhi.x4 = 0 };
    t::ftKb_SpecialHi1_IASA(gp);
    assert_eq!(f.cmd_vars[3], 1);
    assert_eq!(unsafe { f.mv.kb.specialhi.x4 }, 1);
    assert_eq!(f.facing_dir, -1.0);
    assert_eq!(take(), [format!("roty 0 {}", (-core::f64::consts::FRAC_PI_2) as f32)]);
    t::ftKb_SpecialAirHi1_IASA(gp);
    assert!(take().is_empty());

    // Cutter spawn on the accessory callback: x offset follows facing, y drops by 1.
    da.specialhi_projectile_spawn_x = 3.0;
    da.specialhi_projectile_spawn_y = 4.0;
    f.facing_dir = -1.0;
    f.cmd_vars[2] = 1;
    unsafe { f.mv.kb.specialhi.xC = 0 };
    t::fn_800F21E8(gp);
    assert_eq!(take(), ["efflags 1", "cutter 7 23 0 -1"]);
    assert_eq!(f.cmd_vars[2], 0);
    t::fn_800F21E8(gp);
    assert!(take().is_empty());

    // Air Phys: upward velocity is scaled by the vertical momentum.
    da.specialhi_vertical_momentum = 0.5;
    f.self_vel.y = 8.0;
    t::ftKb_SpecialAirHi1_Phys(gp);
    assert_eq!(f.self_vel.y, 4.0);
    assert_eq!(take(), ["85134"]);

    // Coll: ground contact lost on Hi1 enters the air state; Hi3 on a ledge enters the landing state.
    t::ftKb_SpecialHi1_Coll(gp);
    assert_eq!(take(), ["ms 389 c1082", "rotx 0 0"]);
    f.coll_data.floor.normal = Vec3 { x: 0.0, y: 1.0, z: 0.0 };
    f.facing_dir = 1.0;
    t::ftKb_SpecialHi3_Coll(gp);
    let l = take();
    assert_eq!(l[0], "ledge 1");
    assert_eq!(l[1], "ms 392 c1082");
    assert_eq!(l[2], "rotx 0 0");
    assert!(f.accessory4_cb.is_some());
    // Hi2 Coll before the 0x14-frame mark counts up and takes the floor-edge path.
    unsafe { f.mv.kb.specialn_pe.facing_dir = 3 };
    t::ftKb_SpecialHi2_Coll(gp);
    assert_eq!(unsafe { f.mv.kb.specialn_pe.facing_dir }, 4);
    assert_eq!(take(), ["82578"]);
}
