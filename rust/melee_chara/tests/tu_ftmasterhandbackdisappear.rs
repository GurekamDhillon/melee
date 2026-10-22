//! tu::ftmasterhandbackdisappear vs the C (gen/turef/tu_ftmasterhandbackdisappear_ref.c: the game
//! TU #included verbatim with lbVector_Diff/Normalize cut from lbvector.c, shipping contraction;
//! data in tests/data/tu_ftmasterhandbackdisappear.txt; regenerate with `python
//! rust/melee_chara/gen/turef/tu_shard361.py`). 4000 Wait1_1 Phys velocities (fused length, Newton
//! sqrt, normalize) and 400 MS_372 float->int / anim-rate setups, bit for bit; plus smoke tests of
//! the rest of the TU.
#![allow(static_mut_refs)]
#[path = "common_mh/mod.rs"]
mod common;
use common::*;
use melee_chara::tu::ftmasterhandbackdisappear as t;
use melee_types::*;

#[test]
fn matches_c_and_smoke() {
    let want = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/tu_ftmasterhandbackdisappear.txt")).unwrap();
    let mut r = Rig::new();
    let gp = r.gp();
    hook_stubs(1);
    let (mut nphys, mut n372) = (0, 0);
    for (i, line) in want.lines().enumerate() {
        let w: std::vec::Vec<&str> = line.split(' ').collect();
        if w[0] == "phys" {
            let tgt = Vec3 { x: fl(w[2]), y: fl(w[3]), z: fl(w[4]) };
            let cur = Vec3 { x: fl(w[5]), y: fl(w[6]), z: fl(w[7]) };
            r.da.x2C = fl(w[8]);
            r.da.x28 = fl(w[9]);
            r.da.x108 = fl(w[10]);
            r.da.x10C = fl(w[11]);
            let (vx, vy) = t::wait1_1_vel(&tgt, &cur, &r.da);
            assert_eq!((vx.to_bits(), vy.to_bits()), (hex(w[12]), hex(w[13])), "phys line {i}");
            nphys += 1;
        } else {
            // "ms372 x110x x110y anim -> x8 rate"
            r.da.x110_pos = Vec2 { x: fl(w[1]), y: fl(w[2]) };
            r.f.cur_anim_frame = fl(w[3]) as _;
            let _ = take_log();
            t::ftMh_MS_372_801542E0(gp);
            let log = take_log();
            let rate = log.split(',').find_map(|e| e.strip_prefix("rate ")).unwrap();
            assert_eq!(rate.parse::<f32>().unwrap().to_bits(), hex(w[6]), "372 rate line {i}");
            assert_eq!(unsafe { r.f.mv.mh.unk0.x8 }, w[5].parse::<i32>().unwrap(), "372 x8 line {i}");
            n372 += 1;
        }
    }
    assert_eq!((nphys, n372), (4000, 400));

    // Wait1_1_Anim: the countdown hits zero -> rate back to 1 and the Grab state (374 + ftCommon_8007E2D0).
    hook_stubs(1);
    unsafe { r.f.mv.mh.unk0.x8 = 2 };
    t::ftMh_Wait1_1_Anim(gp);
    assert_eq!(unsafe { r.f.mv.mh.unk0.x8 }, 1);
    assert_eq!(take_log(), "");
    t::ftMh_Wait1_1_Anim(gp);
    assert_eq!(take_log(), "rate 1,cms 374 0 0 1 0 true,anim8006EBA4");
    assert_eq!(r.f.x1A68, 128);
    assert_eq!(unsafe { r.f.mv.mh.unk0.x20 }, 0);
    assert!(r.f.grab_cb.is_some() && r.f.grabbed_cb.is_some() && r.f.x2194.is_none());
    // ... and loops when the motion ends.
    hook_stubs(0);
    unsafe { r.f.mv.mh.unk0.x8 = 5 };
    t::ftMh_Wait1_1_Anim(gp);
    assert_eq!(take_log(), "cms 373 0 0 1 0 true,anim8006EBA4");

    // BackDisappear_Anim calls the stored callback only when the motion ended.
    unsafe extern "C" fn cb(_: *mut HSD_GObj) {
        log("cb".into())
    }
    unsafe { r.f.mv.mh.unk0.x4 = Some(cb) };
    hook_stubs(1);
    t::ftMh_BackDisappear_Anim(gp);
    assert_eq!(take_log(), "");
    hook_stubs(0);
    t::ftMh_BackDisappear_Anim(gp);
    assert_eq!(take_log(), "cb");

    // Fail_Anim / Cancel_Anim hand off to Wait1_2 (389) when the motion ended.
    unsafe { melee_chara::tu::ftmasterhandbackairplane3::ext::FTMH_MS_389_80151018 = Some(|_| log("389".into())) };
    t::ftMh_Fail_Anim(gp);
    assert_eq!(take_log(), "389");
    unsafe {
        t::ext::FTMH_CAPTUREWAITMASTERHAND_80155D1C = Some(|_| log("capwait".into()));
        r.f.mv.mh.unk0.x24 = 2.0;
    }
    r.f.cmd_vars[0] = 1;
    r.f.victim_gobj = gp;
    t::ftMh_Cancel_Anim(gp);
    assert_eq!(unsafe { r.f.mv.mh.unk0.x24 }, 1.0);
    assert_eq!(take_log(), "389");
    t::ftMh_Cancel_Anim(gp);
    assert_eq!(unsafe { r.f.mv.mh.unk0.x24 }, 0.0);
    assert_eq!(r.f.cmd_vars[0], 0);
    assert_eq!(take_log(), "capwait,389");
    // cmd_vars[0] already clear: the countdown keeps running but nothing fires.
    t::ftMh_Cancel_Anim(gp);
    assert_eq!(unsafe { r.f.mv.mh.unk0.x24 }, -1.0);
    assert_eq!(take_log(), "389");

    // Cancel_Coll stops the hand only when x18 == 0.
    r.f.self_vel = Vec3 { x: 1.0, y: 2.0, z: 3.0 };
    unsafe { r.f.mv.mh.unk0.x18 = 1.0 };
    t::ftMh_Cancel_Coll(gp);
    assert_eq!(r.f.self_vel.y, 2.0);
    unsafe { r.f.mv.mh.unk0.x18 = 0.0 };
    t::ftMh_Cancel_Coll(gp);
    assert_eq!((r.f.self_vel.x, r.f.self_vel.y, r.f.self_vel.z), (0.0, 0.0, 0.0));

    // IASA (slot type stub returns player_id = 0) runs the no-op ftBossLib_8015BD20; Coll are empty.
    t::ftMh_BackDisappear_IASA(gp);
    t::ftMh_Grab_IASA(gp);
    t::ftMh_BackDisappear_Coll(gp);
    t::ftMh_Wait1_1_Coll(gp);
    t::ftMh_Grab_Coll(gp);
    t::ftMh_Fail_Coll(gp);
}
