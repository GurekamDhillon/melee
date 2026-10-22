//! Smoke test of tu::ftmasterhandpapercrush: Anim hand-offs and the PaperCrush Phys countdown (f32
//! counter strictly between the s32 attributes xB0 and xB4, then the chase call, else x stopped).
#![allow(static_mut_refs)]

use melee_chara::stubs as st;
use melee_chara::tu::ftmasterhandpapercrush as t;
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
        st::FIGHTER_CHANGEMOTIONSTATE = Some(|_, m, _, _, _, _, _| LOG.push(format!("ms {m}")));
        st::FTANIM_8006EBA4 = Some(|_| LOG.push("anim".into()));
        melee_chara::tu::ftmasterhanddamage0::ext::FTMH_MS_389_80151018 = Some(|_| LOG.push("wait".into()));
        melee_chara::tu::ftmasterhanddrill::ext::FTBOSSLIB_8015C010 = Some(|_, x| LOG.push(format!("chase {x}")));
        melee_chara::tu::ftmasterhanddrill::ext::FTBOSSLIB_8015C190 = Some(|_| LOG.push("clamp".into()));
    }
    unsafe { FRAMES = 1 };
    t::ftMh_RockCrushDown_Anim(gp);
    t::ftMh_PaperCrush_Anim(gp);
    assert_eq!(take(), "");
    unsafe { FRAMES = 0 };
    t::ftMh_RockCrushDown_Anim(gp);
    t::ftMh_PaperCrush_Anim(gp);
    assert_eq!(take(), "wait,wait");
    t::ftMh_RockCrushDown_Coll(gp);
    // re-exported entry: PaperCrush enter resets the counter
    f.mv.mh.unk0.x0 = 55.0;
    t::ftMh_MS_357_801526D8(gp);
    assert_eq!((take(), f.mv.mh.unk0.x0), ("ms 358,anim".to_string(), 0.0));

    da.xB0 = 10;
    da.xB4 = 20;
    da.xB8 = 1.25;
    // the counter after the increment must lie strictly inside (xB0, xB4)
    for (x0, chase) in [(0.0f32, false), (8.0, false), (9.0, false), (10.0, true), (18.0, true), (19.0, false), (30.0, false)] {
        f.mv.mh.unk0.x0 = x0;
        f.self_vel.x = 4.0;
        t::ftMh_PaperCrush_Phys(gp);
        assert_eq!(f.mv.mh.unk0.x0, x0 + 1.0);
        if chase {
            assert_eq!((take(), f.self_vel.x), ("phys,chase 1.25,clamp".to_string(), 4.0), "x0 {x0}");
        } else {
            assert_eq!((take(), f.self_vel.x), ("phys,clamp".to_string(), 0.0), "x0 {x0}");
        }
    }
}
