//! Smoke test of tu::ftpeachspecialn (Toad) through recording hooks.
#![allow(static_mut_refs)]

use melee_chara::stubs as st;
use melee_chara::tu::ftpeachspecialn as t;
use melee_types::*;

static mut LOG: std::vec::Vec<String> = std::vec::Vec::new();
fn log(s: String) {
    unsafe { LOG.push(s) }
}
fn take() -> std::vec::Vec<String> {
    unsafe { core::mem::take(&mut LOG) }
}
static mut FRAMES: i32 = 0;
static mut AIR: i32 = 0;

#[test]
fn smoke() {
    let mut f: Box<Fighter> = Box::new(Fighter::zeroed());
    let mut g: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    let da: &'static mut ftPe_DatAttrs = Box::leak(Box::new(ftPe_DatAttrs::zeroed()));
    let mut bones: std::vec::Vec<FighterBone> = (0..(FtPart_109 as usize + 1)).map(|_| unsafe { core::mem::zeroed() }).collect();
    f.parts = bones.as_mut_ptr();
    f.dat_attrs = da as *mut _ as *mut _;
    g.user_data = &mut *f as *mut Fighter as *mut _;
    let gp: *mut HSD_GObj = &mut *g;
    let toad = 0x3000usize as *mut Item_GObj;
    unsafe {
        st::FIGHTER_CHANGEMOTIONSTATE = Some(|_, ms, fl, a, s, b, _| log(format!("ms {ms} {fl:x} {a} {s} {b}")));
        st::FTANIM_8006EBA4 = Some(|_| {});
        st::FTANIM_ISFRAMESREMAINING = Some(|_| FRAMES);
        st::FT_8008A2BC = Some(|_| log("Wait".into()));
        st::FTCO_FALL_ENTER = Some(|_| log("Fall".into()));
        st::FT_80081D0C = Some(|_| AIR);
        st::FT_800827A0 = Some(|_| AIR);
        st::FTCOMMON_8007D7FC = Some(|_| {});
        st::FT_80084F3C = Some(|_| log("84F3C".into()));
        st::FTCOLL_8007AEE0 = Some(|_| log("aee0".into()));
        t::ext::LB_8000B1CC = Some(|_, off, out| {
            assert!(off.is_null());
            *out = Vec3 { x: 1.0, y: 2.0, z: 3.0 };
        });
        t::ext::IT_802BDE18 = Some(|_, pos, part, kind, fd| {
            log(format!("toad {} {} {} {part} {kind} {fd}", (*pos).x, (*pos).y, (*pos).z));
            0x3000usize as *mut Item_GObj
        });
        t::ext::IT_802BE214 = Some(|_, pos, kind, fd| log(format!("spore {} {} {kind} {fd}", (*pos).y, (*pos).z)));
        t::ext::IT_802BDF40 = Some(|_| log("toad_kill".into()));
        t::ext::IT_802BDFA0 = Some(|_| log("toad_pause".into()));
        t::ext::IT_802BDFC0 = Some(|_| log("toad_resume".into()));
        t::ext::IT_802BE100 = Some(|_| log("toad_hit".into()));
        t::ext::FTCOLL_8007B1B8 = Some(|_, _, cb| log(format!("shield {}", cb.is_some())));
    }

    // Enter: resets the script vars, remembers the facing, arms the accessory callback.
    f.facing_dir = -1.0;
    f.cmd_vars = [4, 4, 4, 4];
    t::ftPe_SpecialN_Enter(gp);
    assert_eq!(take(), [format!("ms {ftPe_MS_SpecialN} 0 0 1 0")]);
    assert_eq!(f.cmd_vars, [0; 4]);
    assert_eq!(unsafe { f.mv.pe.specialn.facing_dir }, -1);
    assert!(f.accessory4_cb.is_some());
    da.specialairn_vel_x_div = 2.0;
    f.self_vel.x = 6.0;
    t::ftPe_SpecialAirN_Enter(gp);
    take();
    assert_eq!(f.self_vel.x, 3.0);

    // Accessory pulse spawns Toad once and wires the damage + hitlag callbacks.
    t::onAccessory4(gp);
    assert_eq!(take().len(), 1);
    assert_eq!(unsafe { f.u.pe.toad_gobj }, toad);
    assert!(f.death2_cb.is_some() && f.take_dmg_cb.is_some() && f.accessory4_cb.is_none());
    assert!(f.pre_hitlag_cb.is_some() && f.post_hitlag_cb.is_some());
    t::onAccessory4(gp);
    assert!(take().is_empty());
    t::onEnterHitlag(gp);
    t::onExitHitlag(gp);
    assert_eq!(take(), ["toad_pause", "toad_resume"]);

    // Death: kills Toad and clears the callbacks.
    t::ftPe_SpecialN_OnDeath2(gp);
    assert_eq!(take(), ["toad_resume", "toad_kill"].iter().rev().cloned().collect::<std::vec::Vec<_>>());
    assert!(unsafe { f.u.pe.toad_gobj }.is_null());
    assert!(f.death2_cb.is_none() && f.take_dmg_cb.is_none());
    t::ftPe_SpecialN_OnDeath2(gp);
    assert!(take().is_empty());

    // Anim: state 1 opens the absorb shield once; the animation end changes state.
    f.cmd_vars[1] = 1;
    da.xA8 = 7.0;
    unsafe { FRAMES = 1 };
    t::ftPe_SpecialN_Anim(gp);
    assert_eq!(take(), ["shield true"]);
    assert_eq!(f.cmd_vars[1], 2);
    t::ftPe_SpecialN_Anim(gp);
    assert!(take().is_empty());
    unsafe { FRAMES = 0 };
    t::ftPe_SpecialAirN_Anim(gp);
    assert_eq!(take(), ["Fall"]);

    // Air phys: first frame of state 1 applies the jump once, then falls.
    da.specialairn_vel_y = 5.0;
    da.xA0 = 1.0;
    da.xA4 = 100.0;
    f.cmd_vars[0] = 1;
    unsafe { f.u.pe.specialairn_used = 0 };
    t::ftPe_SpecialAirN_Phys(gp);
    assert_eq!(f.self_vel.y, 4.0);
    assert_eq!(take(), ["aee0"]);
    assert_eq!(f.cmd_vars[0], 2);
    assert_eq!(unsafe { f.u.pe.specialairn_used }, 1);

    // Coll: ground -> air keeps the motion flags; landing resets the air-use flag.
    unsafe { AIR = 0 };
    t::ftPe_SpecialN_Coll(gp);
    assert!(take().iter().any(|s| s.starts_with(&format!("ms {ftPe_MS_SpecialAirN} "))));
    unsafe { AIR = 1 };
    t::ftPe_SpecialAirN_Coll(gp);
    assert!(take().iter().any(|s| s.starts_with(&format!("ms {ftPe_MS_SpecialN} "))));
    assert_eq!(unsafe { f.u.pe.specialairn_used }, 0);
}

