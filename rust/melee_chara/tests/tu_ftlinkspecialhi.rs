//! Smoke test of tu::ftlinkspecialhi (Spin Attack) through recording hooks. The physics callback
//! is verified against the C by tests/chars_ref.rs; `ftLk_SpecialAirHi_Coll` reads the landing lag
//! from `ftLk_DatAttrs::x30` (the C's `fp->facing_dir1` on the attribute block). One #[test].
#![allow(static_mut_refs, unused_unsafe)]

use melee_chara::stubs as st;
use melee_chara::tu::ftlinkspecialhi as t;
use melee_fighter::hooks as fh;
use melee_types::*;

static mut LOG: std::vec::Vec<String> = std::vec::Vec::new();
fn log(s: String) {
    unsafe { LOG.push(s) }
}
fn take_log() -> String {
    unsafe { std::mem::take(&mut LOG).join(",") }
}

#[test]
fn smoke() {
    unsafe { body() }
}

unsafe fn body() {
    let mut f: Box<Fighter> = Box::new(Fighter::zeroed());
    let mut g: Box<HSD_GObj> = Box::new(core::mem::zeroed());
    let da: &'static mut ftLk_DatAttrs = Box::leak(Box::new(ftLk_DatAttrs::zeroed()));
    f.dat_attrs = da as *mut _ as *mut _;
    g.user_data = &mut *f as *mut Fighter as *mut _;
    let gp: *mut HSD_GObj = &mut *g;

    st::FIGHTER_CHANGEMOTIONSTATE = Some(|_, ms, fl, start, _, _, _| log(format!("ms {ms} {fl:x} {start}")));
    st::FTANIM_8006EBA4 = Some(|_| {});
    st::FTANIM_ISFRAMESREMAINING = Some(|_| 0);
    st::FT_8008A2BC = Some(|_| log("Wait".into()));
    st::FTCO_80096900 = Some(|_, a, b, ai, mob, lag| log(format!("fallspecial {a} {b} {ai} {mob} {lag}")));
    st::FT_80084F3C = Some(|_| log("phys-ground".into()));
    st::FT_80082708 = Some(|_| 0);
    st::FT_CHECKGROUNDANDLEDGE = Some(|_, d| {
        log(format!("ledge {d}"));
        0
    });
    st::FTCLIFFCOMMON_80081298 = Some(|_| {
        log("cliff".into());
        0
    });

    // Enter: the ground spin arms accessory4; the air spin also sets velocity and jump count.
    t::ftLk_SpecialHi_Enter(gp);
    assert_eq!(take_log(), "ms 356 0 0");
    assert!(f.accessory4_cb.is_some());
    da.x34 = 0.5;
    da.x40 = 7.0;
    f.self_vel.x = 4.0;
    f.co_attrs.max_jumps = 2;
    t::ftLk_SpecialAirHi_Enter(gp);
    assert_eq!(take_log(), "ms 357 0 0");
    assert_eq!((f.self_vel.x, f.self_vel.y, f.x1968_jumpsUsed), (2.0, 7.0, 2));

    // Anim: ground waits, air falls special with the stick/landing attributes.
    da.specialairhi_drift_stick_mul = 0.25;
    da.x30 = 12.0;
    t::ftLk_SpecialHi_Anim(gp);
    t::ftLk_SpecialAirHi_Anim(gp);
    assert_eq!(take_log(), "Wait,fallspecial 1 1 0 0.25 12");

    // Phys: air uses the verified air physics, ground the shared ground one.
    f.ground_or_air = GA_Ground;
    t::ftLk_SpecialHi_Phys(gp);
    assert_eq!(take_log(), "phys-ground");

    // Coll: leaving the ground restarts the air spin at the same animation frame.
    f.cur_anim_frame = 5.5;
    t::ftLk_SpecialHi_Coll(gp);
    assert_eq!(take_log(), "ms 357 c4c508a 5.5");
    st::FT_80082708 = Some(|_| 1);
    t::ftLk_SpecialHi_Coll(gp);
    assert_eq!(take_log(), "");
    // AirHi_Coll: no ground and no ledge -> nothing; ground found -> landing lag x30.
    t::ftLk_SpecialAirHi_Coll(gp);
    assert_eq!(take_log(), "ledge 0,cliff");
    st::FT_CHECKGROUNDANDLEDGE = Some(|_, _| 1);
    st::FTCO_LANDINGFALLSPECIAL_ENTER = Some(|_, ai, lag| log(format!("landing {ai} {lag}")));
    t::ftLk_SpecialAirHi_Coll(gp);
    assert_eq!(take_log(), "landing 0 12");

    // onAccessory4: spawns the spin effect once (Link: L2ndNa joint), then clears the callback.
    let mut parts: std::vec::Vec<FighterBone> = (0..80).map(|_| core::mem::zeroed()).collect();
    for (i, p) in parts.iter_mut().enumerate() {
        p.joint = (0x1000 + i * 8) as *mut _;
    }
    f.parts = parts.as_mut_ptr();
    f.kind = Ft_Kind_Link;
    fh::FTPARTS_GET_BONE_INDEX = Some(|_, i| i);
    t::ext::EFSYNC_SPAWN = Some(|id, _, a| {
        log(format!("ef {id} {:x} {:x}", a[0] as usize, a[1] as usize));
        core::ptr::null_mut()
    });
    f.ground_or_air = GA_Ground;
    t::onAccessory4(gp);
    assert_eq!(take_log(), format!("ef 1211 {:x} {:x}", 0x1000 + FtPart_TransN as usize * 8, 0x1000 + FtPart_L2ndNa as usize * 8));
    assert_eq!(f.x2219_b0(), 1);
    assert!(f.accessory4_cb.is_none() && f.pre_hitlag_cb.is_some() && f.post_hitlag_cb.is_some());
    t::onAccessory4(gp);
    assert_eq!(take_log(), "");

    // Getters.
    assert_eq!(t::ftLk_SpecialHi_ProcessPartLThumbNb(gp), FtPart_LThumbNb);
    f.u.lk.x4 = 3;
    assert_eq!(t::ftLk_SpecialHi_GetFv4(gp), 3);
    f.cur_pos = Vec3 { x: 1.0, y: 2.0, z: 3.0 };
    da.specialhi_pos_y_offset = 0.5;
    let mut pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    t::ftLk_SpecialHi_GetPosWithAdjustedY(gp, &mut pos);
    assert_eq!((pos.x, pos.y, pos.z), (1.0, 2.5, 3.0));
}
