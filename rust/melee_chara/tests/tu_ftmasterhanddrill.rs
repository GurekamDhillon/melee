//! Smoke test of tu::ftmasterhanddrill: Anim hand-offs and the Drill Phys countdown (f32 counter
//! against the s32 `x84`, then either the chase call or a zeroed x velocity).
#![allow(static_mut_refs)]

use melee_chara::stubs as st;
use melee_chara::tu::ftmasterhanddrill as t;
use melee_types::*;

static mut LOG: std::vec::Vec<String> = std::vec::Vec::new();
static mut FRAMES: i32 = 0;
fn take() -> String {
    unsafe { core::mem::take(&mut LOG).join(",") }
}

#[test]
fn smoke() {
    unsafe { smoke_inner() }
}

#[allow(unused_unsafe)]
unsafe fn smoke_inner() {
    let mut f: Box<Fighter> = Box::new(Fighter::zeroed());
    let mut g: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    let mut fd: Box<ftData> = Box::new(unsafe { core::mem::zeroed() });
    let mut da = Box::new(ftMasterHand_SpecialAttrs::zeroed());
    fd.ext_attr = &mut *da as *mut _ as *mut _;
    f.ft_data = &mut *fd;
    g.user_data = &mut *f as *mut Fighter as *mut _;
    let gp: *mut HSD_GObj = &mut *g;
    unsafe {
        st::FT_80085134 = Some(|_| LOG.push("phys".into()));
        st::FTANIM_ISFRAMESREMAINING = Some(|_| FRAMES);
        melee_chara::tu::ftmasterhanddamage0::ext::FTMH_MS_389_80151018 = Some(|_| LOG.push("wait".into()));
        t::ext::FTBOSSLIB_8015C010 = Some(|_, x| LOG.push(format!("chase {x}")));
        t::ext::FTBOSSLIB_8015C190 = Some(|_| LOG.push("clamp".into()));
    }
    unsafe { FRAMES = 1 };
    t::ftMh_WalkWait_Anim(gp);
    t::ftMh_Drill_Anim(gp);
    assert_eq!(take(), "");
    unsafe { FRAMES = 0 };
    t::ftMh_WalkWait_Anim(gp);
    t::ftMh_Drill_Anim(gp);
    assert_eq!(take(), "wait,wait");
    t::ftMh_WalkWait_Coll(gp);

    da.x84 = 100;
    da.x80 = 2.5;
    // counter after the decrement in [0, x84]: chase; above x84 or below 0: stop
    for (x0, chase) in [(107.0f32, false), (102.0, false), (101.0, true), (100.0, true), (50.0, true), (1.0, true), (0.0, false), (-3.0, false)] {
        f.mv.mh.unk0.x0 = x0;
        f.self_vel.x = 4.0;
        t::ftMh_Drill_Phys(gp);
        assert_eq!(f.mv.mh.unk0.x0, x0 - 1.0);
        if chase {
            assert_eq!((take(), f.self_vel.x), ("phys,chase 2.5,clamp".to_string(), 4.0), "x0 {x0}");
        } else {
            assert_eq!((take(), f.self_vel.x), ("phys,clamp".to_string(), 0.0), "x0 {x0}");
        }
    }
}
