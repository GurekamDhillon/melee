//! Smoke test of tu::ftcrazyhandtagcancel (hooks record the calls).
#![allow(static_mut_refs)]
mod ch_common;
use ch_common::*;
use melee_chara::tu::ftcrazyhandtagcancel as t;
use melee_types::*;

unsafe extern "C" fn cb(_g: *mut HSD_GObj) {
    log("cb");
}

#[test]
fn smoke() {
    let mut r = Rig::new();
    let gp = r.gp();
    // 8015BA34: x2258 == Wait2_0 takes the 0x185 branch with the current frame.
    let mut pos = Vec3 { x: 1.0, y: 2.0, z: 3.0 };
    r.f.u.mh.x2258 = ftMh_MS_Wait2_0 as i32;
    r.f.cur_anim_frame = 7.0;
    t::ftCh_GrabUnk1_8015BA34(gp, Some(cb), &mut pos);
    assert_eq!(take(), "cms 0x185 0 7 1 0");
    assert_eq!(unsafe { r.f.u.mh.x2258 }, 0x185);
    assert_eq!(unsafe { r.f.mv.ch.unk0.xC.z }, 3.0);
    // From another state: restart 0x184 at frame 0.
    r.f.u.mh.x2258 = 5;
    t::ftCh_GrabUnk1_8015BA34(gp, Some(cb), &mut pos);
    assert_eq!(take(), "cms 0x184 0 0 1 0,8006EBA4");
    // TagCancel_Anim.
    unsafe { FRAMES = 1 };
    t::ftCh_TagCancel_Anim(gp);
    assert_eq!(take(), "");
    unsafe { FRAMES = 0 };
    t::ftCh_TagCancel_Anim(gp);
    assert_eq!(take(), "cms 0x184 0 0 1 0,8006EBA4");
    // Coll: x18 == 0 stops and calls the callback; else nothing.
    unsafe {
        r.f.mv.ch.unk0.x4 = Some(cb);
        r.f.mv.ch.unk0.x18 = 1.0;
    }
    r.f.self_vel.x = 4.0;
    t::ftCh_TagCancel_Coll(gp);
    assert_eq!((take(), r.f.self_vel.x), ("".to_string(), 4.0));
    unsafe { r.f.mv.ch.unk0.x18 = 0.0 };
    t::ftCh_TagCancel_Coll(gp);
    assert_eq!((take(), r.f.self_vel.x), ("cb".to_string(), 0.0));
    // 8015BC88: stores da->x18/x1C, enters 0x184, installs ftCh_Init_80156198 as callback.
    r.da.x18 = 5.0;
    r.da.x1C = 6.0;
    unsafe { r.f.mv.ch.unk0.x20 = 9 };
    r.f.u.mh.x2258 = 0;
    t::ftCh_GrabUnk1_8015BC88(gp);
    assert_eq!(take(), "cms 0x184 0 7 1 0");
    unsafe {
        assert_eq!(r.f.mv.ch.unk0.x20, 0);
        assert_eq!((r.f.mv.ch.unk0.xC.x, r.f.mv.ch.unk0.xC.y, r.f.mv.ch.unk0.xC.z), (5.0, 6.0, 0.0));
        assert!(r.f.mv.ch.unk0.x4.is_some());
        assert_eq!(r.f.u.mh.x2258, 0x184);
    }
}
