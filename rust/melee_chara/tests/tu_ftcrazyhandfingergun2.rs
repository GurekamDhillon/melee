//! Smoke test of tu::ftcrazyhandfingergun2: the Anim branch on mv.ch.unk0.x20, and the
//! `ftCh_UpdateBossMotion` inline (ft_80085134 + ftBossLib_8015BE40 on xC/x18) against the
//! group-3 math it calls.
#![allow(static_mut_refs)]

use melee_chara::stubs as st;
use melee_chara::tu::ftcrazyhandfingergun2 as t;
use melee_types::*;

static mut LOG: std::vec::Vec<String> = std::vec::Vec::new();
fn log(s: String) {
    unsafe { LOG.push(s) }
}

#[test]
fn smoke() {
    let mut f: Box<Fighter> = Box::new(Fighter::zeroed());
    let mut g: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    let mut fd: Box<ftData> = Box::new(unsafe { core::mem::zeroed() });
    let mut da = Box::new(ftCrazyHand_DatAttrs::zeroed());
    da.x10 = 0.125;
    da.x14 = 2.0;
    da.x18 = 5.5;
    da.x1C = -6.5;
    fd.ext_attr = &mut *da as *mut _ as *mut _;
    f.ft_data = &mut *fd;
    g.user_data = &mut *f as *mut Fighter as *mut _;
    let gp: *mut HSD_GObj = &mut *g;
    unsafe {
        st::FTANIM_ISFRAMESREMAINING = Some(|_| 1);
        st::FIGHTER_CHANGEMOTIONSTATE = Some(|_, m, _, _, _, _, _| log(format!("ms {m:x}")));
        st::FTANIM_8006EBA4 = Some(|_| log("anim".into()));
        st::FT_80085134 = Some(|_| log("85134".into()));
        t::ext::FTCH_INIT_801592D4 = Some(|_| log("592D4".into()));
    }
    t::ftCh_FingerGun2_Anim(gp);
    assert!(unsafe { LOG.is_empty() });
    unsafe { st::FTANIM_ISFRAMESREMAINING = Some(|_| 0) };
    f.mv.ch.unk0.x20 = 1;
    t::ftCh_FingerGun2_Anim(gp);
    f.mv.ch.unk0.x20 = 0;
    t::ftCh_FingerGun2_Anim(gp);
    assert_eq!(unsafe { LOG.join(",") }, "592D4,ms 172,anim");
    assert_eq!((unsafe { f.mv.ch.unk0.xC.x }, unsafe { f.mv.ch.unk0.xC.y }), (5.5, -6.5));

    unsafe { LOG.clear() };
    f.cur_pos = Vec3 { x: 1.0, y: 2.0, z: 0.0 };
    let (sx, sy, out) = melee_fighter::tu::ftbosslib::boss_8015be40_math(&Vec3 { x: 5.5, y: -6.5, z: 0.0 }, &f.cur_pos, 2.0, 0.125);
    t::ftCh_FingerGun2_Phys(gp);
    assert_eq!(unsafe { LOG.join(",") }, "85134");
    assert_eq!((f.self_vel.x.to_bits(), f.self_vel.y.to_bits()), (sx.to_bits(), sy.to_bits()));
    assert_eq!(unsafe { f.mv.ch.unk0.x18 }.to_bits(), out.to_bits());
    t::ftCh_FingerGun2_Coll(gp);
}
