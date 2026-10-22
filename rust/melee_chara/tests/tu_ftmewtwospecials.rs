//! Smoke test of tu::ftmewtwospecials (Confusion): entry, grab hand-off, reflect arm/disarm.
#![allow(static_mut_refs)]
use melee_chara::stubs as st;
use melee_types::*;

static mut LOG: std::vec::Vec<String> = std::vec::Vec::new();
fn log(s: String) {
    unsafe { LOG.push(s) }
}
fn take() -> String {
    unsafe { std::mem::take(&mut LOG).join(",") }
}
static mut FRAMES_LEFT: i32 = 0;

use melee_chara::tu::ftmewtwospecials as t;

#[test]
fn smoke() {
    let mut attrs: Box<ftMewtwoAttributes> = Box::new(unsafe { core::mem::zeroed() });
    let mut f: Box<Fighter> = Box::new(Fighter::zeroed());
    f.dat_attrs = &mut *attrs as *mut _ as *mut _;
    let mut g: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    g.user_data = &mut *f as *mut Fighter as *mut _;
    let gp: *mut HSD_GObj = &mut *g;
    let mut victim: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    unsafe {
        st::FTANIM_ISFRAMESREMAINING = Some(|_| FRAMES_LEFT);
        st::FT_8008A2BC = Some(|_| log("wait".into()));
        st::FTCO_FALL_ENTER = Some(|_| log("fall".into()));
        st::FIGHTER_CHANGEMOTIONSTATE = Some(|_, ms, _, _, _, _, _| log(format!("M{ms}")));
        st::FTANIM_8006EBA4 = Some(|_| log("8006EBA4".into()));
        t::ext::FTCO_800DE2A8 = Some(|_, _| log("throw".into()));
        t::ext::FTCO_80090780 = Some(|_| log("fall_victim".into()));
        t::ext::FTCOLL_CREATEREFLECTHIT = Some(|_, _, cb| log(format!("reflect cb={}", cb.is_some())));
    }

    // Grounded entry: clears the script vars and arms the reflect think callback.
    f.cmd_vars[0] = 9;
    f.cmd_vars[1] = 9;
    t::ftMt_SpecialS_Enter(gp);
    assert_eq!(take(), format!("M{},8006EBA4", ftMt_MS_SpecialS));
    assert_eq!((f.cmd_vars[0], f.cmd_vars[1]), (0, 0));
    assert!(f.accessory4_cb.is_some());

    // Aerial entry: the one-time boost.
    attrs.x18_MEWTWO_CONFUSION_AIR_BOOST = 1.25;
    t::ftMt_SpecialAirS_Enter(gp);
    assert_eq!(take(), format!("M{},8006EBA4", ftMt_MS_SpecialAirS));
    assert_eq!(unsafe { (f.self_vel.y, f.u.mt.x223C_isConfusionBoost) }, (1.25, 1));
    f.self_vel.y = 0.0;
    t::ftMt_SpecialAirS_Enter(gp);
    take();
    assert_eq!(f.self_vel.y, 0.0);

    // Anim: a pulse with a victim hands it to the throw once; the animation end leaves the move.
    f.victim_gobj = &mut *victim;
    f.cmd_vars[0] = 1;
    unsafe { FRAMES_LEFT = 1 };
    t::ftMt_SpecialS_Anim(gp);
    assert_eq!(take(), "throw,fall_victim");
    assert_eq!(f.cmd_vars[0], 0);
    unsafe { FRAMES_LEFT = 0 };
    t::ftMt_SpecialS_Anim(gp);
    assert_eq!(take(), "wait");
    t::ftMt_SpecialAirS_Anim(gp);
    assert_eq!(take(), "fall");

    // ReflectThink: ON creates the bubble once, OFF removes it.
    f.cmd_vars[1] = t::CONFUSION_REFLECT_ON;
    t::ftMt_SpecialS_ReflectThink(gp);
    assert_eq!(take(), "reflect cb=true");
    assert_eq!(f.cmd_vars[1], t::CONFUSION_REFLECT_NONE);
    assert!(unsafe { f.mv.mt.SpecialS.isConfusionReflect() } != 0);
    f.cmd_vars[1] = t::CONFUSION_REFLECT_OFF;
    t::ftMt_SpecialS_ReflectThink(gp);
    assert!(unsafe { f.mv.mt.SpecialS.isConfusionReflect() } == 0);
    assert!(f.reflect_hit_cb.is_none());
    assert_eq!(f.cmd_vars[1], t::CONFUSION_REFLECT_NONE);
}
