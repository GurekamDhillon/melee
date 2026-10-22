//! Smoke test of tu::ftmasterhandwait12 (hooks record the calls).
#![allow(static_mut_refs)]
mod mh_common;
use mh_common::*;
use melee_chara::tu::ftmasterhandwait12 as t;
use melee_types::*;

unsafe extern "C" fn cb(_g: *mut HSD_GObj) {
    log("cb");
}

#[test]
fn smoke() {
    let mut r = Rig::new();
    let gp = r.gp();
    // 80150DC4: x2258 == Wait2_0 -> Wait2_1 with the current frame.
    let pos = Vec3 { x: 1.0, y: 2.0, z: 3.0 };
    r.f.u.mh.x2258 = ftMh_MS_Wait2_0 as i32;
    r.f.cur_anim_frame = 7.0;
    t::ftMh_MS_389_80150DC4(gp, Some(cb), &pos);
    assert_eq!(take(), "cms 0x186 0 7 1 0");
    unsafe {
        assert_eq!(r.f.u.mh.x2258, 390);
        assert_eq!(r.f.mv.mh.unk0.xC.z, 3.0);
        assert!(r.f.mv.mh.unk0.x4.is_some());
    }
    // From another state, the Wait1_2 branch restarts at frame 0.
    r.f.u.mh.x2258 = 5;
    t::ftMh_MS_389_80150DC4(gp, Some(cb), &pos);
    assert_eq!(take(), "cms 0x185 0 0 1 0,8006EBA4");
    assert_eq!(unsafe { r.f.u.mh.x2258 }, 389);
    // Coll: x18 != 0 does nothing; == 0 zeroes the velocity and calls the callback.
    unsafe { r.f.mv.mh.unk0.x18 = 1.0 };
    r.f.self_vel.x = 4.0;
    t::ftMh_Wait1_2_Coll(gp);
    assert_eq!((take(), r.f.self_vel.x), ("".to_string(), 4.0));
    unsafe { r.f.mv.mh.unk0.x18 = 0.0 };
    t::ftMh_Wait1_2_Coll(gp);
    assert_eq!((take(), r.f.self_vel.x), ("cb".to_string(), 0.0));
    // 80151018: x20 = 0, position from x30_pos2, Wait2_1 state -> 80150C8C path (Wait1_2).
    r.da.x30_pos2 = Vec2 { x: 5.0, y: 6.0 };
    unsafe { r.f.mv.mh.unk0.x20 = 9 };
    r.f.u.mh.x2258 = 0;
    t::ftMh_MS_389_80151018(gp);
    assert_eq!(take(), "cms 0x185 0 0 1 0,8006EBA4");
    unsafe {
        assert_eq!(r.f.mv.mh.unk0.x20, 0);
        assert_eq!((r.f.mv.mh.unk0.xC.x, r.f.mv.mh.unk0.xC.y, r.f.mv.mh.unk0.xC.z), (5.0, 6.0, 0.0));
        assert!(r.f.mv.mh.unk0.x4.is_some());
        assert_eq!(r.f.u.mh.x2258, 389);
    }
}
