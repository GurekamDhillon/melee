//! Smoke test of tu::ftmasterhandtagcancel (hooks record the calls).
#![allow(static_mut_refs)]
mod mh_common;
use mh_common::*;
use melee_chara::tu::ftmasterhandtagcancel as t;
use melee_types::*;

#[test]
fn smoke() {
    let mut r = Rig::new();
    let gp = r.gp();
    let mut vf: Box<Fighter> = Box::new(Fighter::zeroed());
    vf.facing_dir = -1.0;
    let mut vg: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    vg.user_data = &mut *vf as *mut Fighter as *mut _;
    let vgp: *mut HSD_GObj = &mut *vg;
    unsafe {
        t::ext::FTCOMMON_8007DB58 = Some(|_| log("db58"));
        t::ext::FTCO_8009750C = Some(|_| log("750c"));
        t::ext::FTCO_800DD168 = Some(|_| log("d168"));
        t::ext::FTCO_800DA824 = Some(|_| {
            log("a824");
            12.5
        });
        t::ext::FTCOMMON_INITGRAB = Some(|_, a, timer| log(format!("initgrab {a} {timer}")));
        t::ext::FTCO_800DB368 = Some(|_, _| log("b368"));
    }
    r.f.anon_221b.set_x221B_b5(1);
    r.f.anon_221b.set_x221B_b7(1);
    t::ftMh_MS_388_80155A58(gp, vgp);
    assert_eq!(
        take(),
        "db58,750c,d168,a824,initgrab 0 12.5,b368,cms 327 0 0 1 0,8006EBA4"
    );
    assert_eq!(r.f.x1A5C, vgp);
    assert_eq!(r.f.victim_gobj, vgp);
    assert_eq!(r.f.facing_dir, -1.0);
    assert_eq!((r.f.invisible(), r.f.x2220_b3()), (1, 1));
    assert_eq!((r.f.anon_221b.x221B_b5(), r.f.anon_221b.x221B_b7()), (0, 0));
    assert!(r.f.accessory1_cb.is_some());
}
