//! Smoke test of tu::ftmasterhandsqueeze (Coll velocity clear, Anim branch on x2250).
#![allow(static_mut_refs)]
mod mh_common;
use mh_common::*;
use melee_chara::tu::ftmasterhandsqueeze as t;
use melee_types::*;

#[test]
fn smoke() {
    let mut r = Rig::new();
    let gp = r.gp();
    r.f.self_vel = Vec3 { x: 1.0, y: 2.0, z: 3.0 };
    r.f.mv.mh.unk0.x18 = 4.0;
    t::ftMh_Squeeze_Coll(gp);
    assert_eq!(r.f.self_vel.y, 2.0);
    r.f.mv.mh.unk0.x18 = 0.0;
    t::ftMh_Squeeze_Coll(gp);
    assert_eq!((r.f.self_vel.x, r.f.self_vel.y, r.f.self_vel.z), (0.0, 0.0, 0.0));
    // Anim: frames remain -> nothing; done with x2250 == Throw -> the Throw state, else 379_80155014
    t::ftMh_Squeeze_Anim(gp);
    assert_eq!(take(), "");
    unsafe { FRAMES = 0 };
    r.f.u.mh.x2250 = 379;
    t::ftMh_Squeeze_Anim(gp);
    let a = take();
    assert!(a.starts_with("cms 379 "), "{a}");
    r.f.u.mh.x2250 = 0;
    t::ftMh_Squeeze_Anim(gp);
    let b = take();
    assert!(b.starts_with("cms "), "{b}");
}
