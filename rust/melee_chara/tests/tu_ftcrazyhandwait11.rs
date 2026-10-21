//! Smoke test of tu::ftcrazyhandwait11: the back-crush scale ramp (float math checked exactly).
#![allow(static_mut_refs)]
#[path = "common_ch/mod.rs"]
mod common;
use common::*;
use melee_chara::tu::ftcrazyhandwait11 as t;
use melee_types::*;

#[test]
fn smoke() {
    let mut r = Rig::new();
    let gp = r.gp();
    let mut jobj: Box<HSD_JObj> = Box::new(unsafe { core::mem::zeroed() });
    // Independent SRT: SetScale skips the matrix-dirty call.
    jobj.flags = JOBJ_MTX_INDEP_SRT as u32;
    jobj.scale = Vec3 { x: 2.0, y: 3.0, z: 4.0 };
    r.g.hsd_obj = &mut *jobj as *mut HSD_JObj as *mut _;
    hook_stubs(1);
    unsafe {
        t::ext::FTCH_GRABUNK1_8015BC88 = Some(|_| log("bc88".into()));
        t::ext::HSD_JOBJSETMTXDIRTYSUB = Some(|_| log("dirty".into()));
    }
    // Ramp finished: scale restored from x50, flag cleared.
    unsafe {
        r.f.cmd_vars[0] = 1;
        r.f.mv.ch.ch_backcrush.x5C = 0;
        r.f.mv.ch.ch_backcrush.x50 = Vec3 { x: 5.0, y: 6.0, z: 7.0 };
    }
    t::ftCh_Wait1_1_Anim(gp);
    assert_eq!(r.f.cmd_vars[0], 0);
    assert_eq!((jobj.scale.x, jobj.scale.y, jobj.scale.z), (5.0, 6.0, 7.0));
    assert_eq!(take_log(), "");
    // Mid-ramp: scale += step.
    unsafe {
        r.f.cmd_vars[0] = 1;
        r.f.mv.ch.ch_backcrush.x5C = 5;
        r.f.mv.ch.ch_backcrush.x44 = Vec3 { x: 0.5, y: 0.25, z: 0.125 };
    }
    t::ftCh_Wait1_1_Anim(gp);
    assert_eq!((jobj.scale.x, jobj.scale.y, jobj.scale.z), (5.5, 6.25, 7.125));
    assert_eq!(unsafe { r.f.mv.ch.ch_backcrush.x5C }, 4);
}
