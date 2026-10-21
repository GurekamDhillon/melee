//! Smoke test for src/tu/ftco_800c7ca0.rs (DownReflect); callees are stubbed.
#![allow(static_mut_refs)]
use melee_fighter::hooks as h;
use melee_fighter::tu::ftco_800c7ca0 as d;
use melee_fighter::types::*;

#[test]
fn downreflect_right_wall() {
    let mut cd: Box<ftCommonData> = Box::new(unsafe { core::mem::zeroed() });
    cd.x1B0 = 1.0;
    cd.x1BC = 0.5;
    cd.x1B8 = 3;
    let mut f: Box<Fighter> = Box::new(unsafe { core::mem::zeroed() });
    let mut go: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    go.user_data = &mut *f as *mut Fighter as *mut _;
    let g = &mut *go as *mut HSD_GObj;
    unsafe {
        h::P_FT_COMMON_DATA = &*cd;
        h::PLAYER_8003544C = Some(|_, _| false);
        d::ext::FTKB_SPECIALN_800F1F1C = Some(|_, _| {});
        d::ext::EFASYNC_SPAWN = Some(|_, _, _, _, _, _, _| {});
        d::ext::CAMERA_REQUEST_QUAKE = Some(|_, _| {});
        d::ext::FIGHTER_CHANGE_MOTION_STATE = Some(|_, _, _, _, _, _, _| {});
        d::ext::FTCOLL_8007B760 = Some(|_, v| assert_eq!(v, 3));
        h::FT_80081DD4 = Some(|_| {});
    }
    f.x8c_kb_vel.x = 0.5;
    assert!(!d::ftco_800c7ca0(g));
    f.x8c_kb_vel.x = -2.0;
    f.coll_data.env_flags = Collide_RightWallHug;
    f.coll_data.ecb.left = Vec2 { x: -1.0, y: 2.0 };
    f.coll_data.right_facing_wall.normal = Vec3 { x: 1.0, y: 0.0, z: 0.0 };
    f.xF0_ground_kb_vel = 4.0;
    f.cur_pos = Vec3 { x: 10.0, y: 5.0, z: 0.0 };
    f.x68C_transNPos = Vec3 { x: 0.0, y: 0.0, z: 0.25 };
    assert!(d::ftco_800c7ca0(g));
    assert_eq!(unsafe { f.mv.co.downreflect.x4 }, 1);
    assert_eq!(f.x8c_kb_vel.x, 2.0);
    assert_eq!(f.facing_dir, 1.0);
    // -(0.25 * -1 - (10 + -1)) = 9.25
    assert_eq!(f.cur_pos.x, 9.25);
    assert_eq!(f.dmg.x18A8, 4.0);
    assert_eq!(f.ground_or_air, GA_Air);
    // second hit on the same wall is ignored
    f.x8c_kb_vel.x = -2.0;
    assert!(!d::ftco_800c7ca0(g));
    d::ftco_downreflect_iasa(g);
}
