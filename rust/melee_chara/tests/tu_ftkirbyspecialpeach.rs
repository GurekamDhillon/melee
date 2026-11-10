//! tu::ftkirbyspecialpeach (ftkirbyspecialpeach.c): smoke test of Kirby's Toad (Peach copy) through recording
//! hooks. The file has no fused sites (tools/rust-port/fma_sites.py); the only float arithmetic is
//! `self_vel.x /= pe_friction` and the Phys momentum. One #[test]: the hooks are process globals.
#![allow(static_mut_refs)]

use melee_chara::stubs as st;
use melee_chara::tu::ftkirbyspecialpeach as t;
use melee_types::*;

static mut LOG: std::vec::Vec<String> = std::vec::Vec::new();
static mut ITEM: *mut HSD_GObj = core::ptr::null_mut();

fn log(s: String) {
    unsafe { LOG.push(s) }
}

fn take() -> String {
    unsafe { std::mem::take(&mut LOG).join(",") }
}

#[test]
fn smoke() {
    let mut f: Box<Fighter> = Box::new(Fighter::zeroed());
    let mut g: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    let mut toad: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    let mut dummy: Box<[u8; 64]> = Box::new([0; 64]);
    let mut parts: std::vec::Vec<FighterBone> = (0..100).map(|_| unsafe { core::mem::zeroed() }).collect();
    for p in parts.iter_mut() {
        p.joint = dummy.as_mut_ptr() as *mut _;
    }
    let da: &'static mut ftKb_DatAttrs = Box::leak(Box::new(ftKb_DatAttrs::zeroed()));
    f.dat_attrs = da as *mut _ as *mut _;
    f.parts = parts.as_mut_ptr();
    g.user_data = &mut *f as *mut Fighter as *mut _;
    let gp: *mut HSD_GObj = &mut *g;
    unsafe {
        ITEM = &mut *toad;
        t::ext::LB_8000B1CC = Some(|_, off, out| unsafe {
            log(format!("lb {}", off.is_null()));
            (*out).x = 1.0;
            (*out).y = 2.0;
            (*out).z = 3.0;
        });
        t::ext::IT_802BDE18 = Some(|_, pos, part, kind, dir| unsafe {
            log(format!("toad {} {} {} {} {}", (*pos).x, (*pos).y, part, kind, dir));
            ITEM
        });
        t::ext::IT_802BE214 = Some(|_, pos, kind, dir| unsafe {
            log(format!("spore {} {} {} {} {}", (*pos).x, (*pos).y, (*pos).z, kind, dir));
        });
        t::ext::IT_802BDF40 = Some(|_| log("it40".into()));
        t::ext::IT_802BDFA0 = Some(|_| log("itA0".into()));
        t::ext::IT_802BDFC0 = Some(|_| log("itC0".into()));
        t::ext::IT_802BE100 = Some(|_| log("it100".into()));
        t::ext::FTCOLL_8007B1B8 = Some(|_, _, cb| log(format!("shield {}", cb.is_some())));
        t::ext::FTCOMMON_FALLBASIC = Some(|_| log("fallbasic".into()));
        t::ext::FTCOMMON_CALCSELFACCEL_DEACCEL = Some(|_, fr| log(format!("deaccel {fr}")));
        melee_fighter::hooks::UN_803224DC = Some(|_, _, _| false);
        melee_fighter::hooks::GENO_GROUND_EDGE = Some(|_, _| {});
        melee_fighter::hooks::MPLIB_8004D164_LINE_COUNT = Some(|| None);
        st::FTCOMMON_8007D7FC = Some(|_| {});
        st::FIGHTER_CHANGEMOTIONSTATE = Some(|_, ms, fl, a, _, _, _| log(format!("ms {ms} {fl:x} {a}")));
        st::FTANIM_8006EBA4 = Some(|_| log("anim".into()));
        st::FTANIM_ISFRAMESREMAINING = Some(|_| 0);
        st::FT_8008A2BC = Some(|_| log("wait".into()));
        st::FTCO_FALL_ENTER = Some(|_| log("fall".into()));
        st::FT_80082708 = Some(|_| 0);
        st::FT_80081D0C = Some(|_| 1);
        st::FT_800827A0 = Some(|_| 0);
        st::FTCOLL_8007AEE0 = Some(|_| log("coll".into()));
        st::FT_80084F3C = Some(|_| log("phys".into()));
    }
    f.facing_dir = -1.0;
    // Enter on the ground: velocity cleared, accessory callback armed, mv facing = (int) facing.
    f.self_vel.y = 5.0;
    t::ftKb_SpecialNPe_8010C4D4(gp);
    assert_eq!(f.self_vel.y, 0.0);
    assert_eq!(unsafe { f.mv.kb.specialn_pe.facing_dir }, -1);
    assert!(f.accessory4_cb.is_some());
    assert_eq!(take(), format!("ms {} 0 0,anim", ftKb_MS_PeSpecialLw));
    // The accessory callback spawns the Toad once, wires death callbacks and the hitlag callbacks.
    f.cmd_vars[2] = 0;
    t::fn_8010C288(gp);
    assert_eq!(f.cmd_vars[2], 1);
    assert_eq!(unsafe { f.u.kb.xD0 }, unsafe { ITEM });
    assert!(f.accessory4_cb.is_none() && f.pre_hitlag_cb.is_some() && f.post_hitlag_cb.is_some());
    assert!(f.death2_cb.is_some() && f.take_dmg_cb.is_some());
    assert_eq!(take(), format!("lb true,toad 1 2 {} {} -1", FtPart_R3rdNa, It_Kind_Kirby_PeachToad));
    t::fn_8010C288(gp);
    assert_eq!(take(), "");
    // Air Enter divides the horizontal speed by the friction attr.
    da.specialn_pe_friction = 4.0;
    f.self_vel.x = 2.0;
    t::ftKb_SpecialNPe_8010C560(gp);
    assert_eq!(f.self_vel.x, 0.5);
    take();
    // hitlag callbacks poke the item; the cleanup frees it.
    t::fn_8010C44C(gp);
    t::ftKb_SpecialNPe_8010C47C(gp);
    t::ftKb_SpecialNPe_8010C3F4(gp);
    assert!(unsafe { f.u.kb.xD0 }.is_null());
    assert_eq!(take(), "itA0,itC0,it40,itC0");
    t::ftKb_SpecialNPe_8010C3F4(gp);
    assert_eq!(take(), "");
    // doPeAnim: cmd 1 arms the absorb shield (two attrs copied), then the end-of-anim callback.
    da.specialn_pe_unk3 = 3.5;
    f.cmd_vars[1] = 1;
    t::ftKb_PeSpecialLw_Anim(gp);
    assert_eq!(f.cmd_vars[1], 2);
    assert_eq!((f.shield_unk0, f.shield_unk1, f.x221B.x221B_b3()), (3.5, 3.5, 1));
    assert_eq!(take(), "shield true,wait");
    f.cmd_vars[1] = 0;
    f.x221B.set_x221B_b0(1);
    t::ftKb_PeSpecialAirLw_Anim(gp);
    assert_eq!(f.x221B.x221B_b0(), 0);
    assert_eq!(take(), "fall");
    // AirLw Phys: cmd 1 -> first use sets the jump momentum; later use zeroes it.
    da.specialn_pe_air_initial_vertical_momentum = 6.0;
    da.specialn_pe_air_horizontal_momentum_preservation = 0.125;
    f.cmd_vars[0] = 1;
    unsafe { f.u.kb.xCC = 0 };
    f.self_vel.y = 1.0;
    t::ftKb_PeSpecialAirLw_Phys(gp);
    assert_eq!((f.self_vel.y, f.cmd_vars[0], unsafe { f.u.kb.xCC }), (6.0, 2, 1));
    f.cmd_vars[0] = 1;
    t::ftKb_PeSpecialAirLw_Phys(gp);
    assert_eq!(f.self_vel.y, 0.0);
    f.cmd_vars[0] = 0;
    take();
    t::ftKb_PeSpecialAirLw_Phys(gp);
    assert_eq!(take(), "fallbasic,deaccel 0.125,coll");
    // Coll: ground loss -> Toad air state; landing -> ground state; both re-arm the callbacks.
    f.cmd_vars[0] = 1;
    f.cmd_vars[1] = 2;
    f.accessory4_cb = None;
    t::ftKb_PeSpecialLw_Coll(gp);
    assert_eq!(f.cmd_vars[0], 2);
    assert!(f.accessory4_cb.is_some());
    take();
    unsafe { f.u.kb.xCC = 1 };
    t::ftKb_PeSpecialAirLw_Coll(gp);
    assert_eq!(unsafe { f.u.kb.xCC }, 0);
    // Hit state: the absorb callback switches to the Hit state at frame 9.
    f.ground_or_air = GA_Ground as _;
    f.specialn_facing_dir = -1.0;
    unsafe { f.u.kb.xD0 = ITEM };
    take();
    t::fn_8010CD88(gp);
    assert_eq!(unsafe { f.mv.kb.specialn_pe.facing_dir }, -1);
    assert_eq!(take(), format!("ms {} 0 9,anim,it100", ftKb_MS_PeSpecialLwHit));
    // Hit anim: cmd 3 arms the spore callback, which then spawns it.
    f.cmd_vars[3] = 1;
    t::ftKb_PeSpecialLwHit_Anim(gp);
    assert_eq!(f.cmd_vars[3], 0);
    t::fn_8010C344(gp);
    assert!(f.accessory4_cb.is_none());
    assert_eq!(take(), format!("wait,lb true,spore 1 4.5 0 {} -1", It_Kind_Kirby_PeachToadSpore));
    assert!(t::ftKb_SpecialNPe_8010C4AC(gp) || !t::ftKb_SpecialNPe_8010C4AC(gp));
    drop(parts);
}
