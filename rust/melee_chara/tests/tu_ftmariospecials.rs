//! Smoke test for tu::ftmariospecials through recording hooks. The two fused sites
//! (`coords.x += 3 * facing_dir`) are checked bit-for-bit against fmadds(3, facing, x) with an x
//! where a separate multiply and add would round differently.
#![allow(static_mut_refs)]

use melee_chara::stubs as st;
use melee_chara::tu::ftmariospecials as t;
use melee_core::fp::fmadds;
use melee_types::*;

static mut LOG: std::vec::Vec<String> = std::vec::Vec::new();
fn log(s: String) {
    unsafe { LOG.push(s) }
}
fn take() -> std::vec::Vec<String> {
    unsafe { core::mem::take(&mut LOG) }
}

#[test]
fn smoke() {
    let mut f: Box<Fighter> = Box::new(Fighter::zeroed());
    let mut g: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    let da: &'static mut ftMario_DatAttrs = Box::leak(Box::new(ftMario_DatAttrs::zeroed()));
    f.dat_attrs = da as *mut _ as *mut _;
    g.user_data = &mut *f as *mut Fighter as *mut _;
    let gp: *mut HSD_GObj = &mut *g;
    let mut parts: std::vec::Vec<FighterBone> = (0..80).map(|_| unsafe { core::mem::zeroed() }).collect();
    f.parts = parts.as_mut_ptr();
    let mut p2j: std::vec::Vec<u8> = (0..80u8).collect();
    let mut tbl: FighterPartsTable = unsafe { core::mem::zeroed() };
    tbl.part_to_joint = p2j.as_mut_ptr();
    let mut tbls: std::vec::Vec<*mut FighterPartsTable> = vec![&mut tbl; 40];
    static mut CAPE: HSD_GObj = unsafe { core::mem::zeroed() };
    unsafe {
        melee_fighter::hooks::FT_PARTS_TABLE = tbls.as_mut_ptr();
        st::FIGHTER_CHANGEMOTIONSTATE = Some(|_, ms, fl, s, sp, _, _| log(format!("ms {ms} {fl:x} {s} {sp}")));
        st::FTANIM_8006EBA4 = Some(|_| log("anim".into()));
        st::FT_800827A0 = Some(|_| 0);
        st::FT_80081D0C = Some(|_| 1);
        st::FTCOMMON_8007D7FC = Some(|_| log("7D7FC".into()));
        st::FT_80084F3C = Some(|_| log("84F3C".into()));
        st::FTCOLL_8007AEF8 = Some(|_| log("AEF8".into()));
        t::ext::LB_8000B1CC = Some(|_, _, out| *out = Vec3 { x: 0.1, y: 2.0, z: 3.0 });
        t::ext::LB_800119DC = Some(|p, a, b, c, d| log(format!("shake {} {a} {b} {c} {}", (*p).x.to_bits(), d.to_bits())));
        t::ext::IT_802B2560 = Some(|_, facing, p, bone, kind| {
            log(format!("cape {facing} {} {bone} {kind}", (*p).x));
            &raw mut CAPE
        });
        t::ext::IT_802B2674 = Some(|_| log("rm".into()));
        t::ext::IT_802B26C0 = Some(|_| log("enter hitlag".into()));
        t::ext::IT_802B26E0 = Some(|_| log("exit hitlag".into()));
        t::ext::FTCOLL_CREATEREFLECTHIT = Some(|_, d, _| log(format!("reflect {}", (*d).x4_max_damage)));
    }
    da.specials.vel_x_decay = 2.0;
    da.specials.cape_kind = 77;
    da.specials.vel = Vec2 { x: 0.5, y: 4.0 };
    da.specials.grav = 0.25;
    da.specials.terminal_vel = 9.0;
    da.cape_reflection.x4_max_damage = 31;

    // Enters: cape callback armed, reflecting cleared.
    f.self_vel = Vec3 { x: 8.0, y: 5.0, z: 0.0 };
    f.cmd_vars = [1, 1, 1, 1];
    t::ftMr_SpecialS_Enter(gp);
    assert_eq!(take(), ["ms 345 0 0 1", "anim"]);
    assert_eq!((f.self_vel.y, f.cmd_vars[..3].to_vec()), (0.0, vec![0, 0, 0]));
    assert!(f.accessory4_cb.is_some());
    t::ftMr_SpecialAirS_Enter(gp);
    assert_eq!(f.self_vel.x, 4.0);
    assert_eq!(take(), ["ms 346 0 0 1", "anim"]);

    // CreateCape: once per move, installs the hitlag callbacks and clears the accessory callback.
    f.facing_dir = -1.0;
    t::ftMr_SpecialS_CreateCape(gp);
    assert_eq!(take(), [format!("cape -1 0.1 {} 77", FtPart_RThumbNb as i32)]);
    assert_eq!(f.cmd_vars[2], 1);
    assert!(unsafe { !f.u.mr.x223C_capeGObj.is_null() } && f.accessory4_cb.is_none() && f.pre_hitlag_cb.is_some() && f.death2_cb.is_some());
    t::ftMr_SpecialS_CreateCape(gp);
    assert!(take().is_empty());
    t::ftMr_SpecialS_EnterHitlag(gp);
    t::ftMr_SpecialS_ExitHitlag(gp);
    assert_eq!(take(), ["enter hitlag", "exit hitlag"]);
    t::ftMr_SpecialS_RemoveCape(gp);
    assert_eq!(take(), ["rm", "exit hitlag"]);
    assert!(!unsafe { !f.u.mr.x223C_capeGObj.is_null() } && f.death2_cb.is_none() && f.take_dmg_cb.is_none());
    t::ftMr_SpecialS_RemoveCape(gp);
    assert!(take().is_empty());

    // Phys: spawn the shake once at the hip, offset by the fused 3 * facing.
    f.facing_dir = -1.0;
    f.cmd_vars[0] = 1;
    t::ftMr_SpecialS_Phys(gp);
    let want = fmadds(3.0, -1.0, 0.1);
    let l = take();
    assert_eq!(l[0], format!("shake {} 120 {} {} {}", want.to_bits(), 0.9f32, 0.02f32, ((core::f64::consts::PI / 3.0) as f32).to_bits()));
    assert_eq!(&l[1..], ["84F3C", "AEF8"]);
    assert_eq!(f.cmd_vars[0], 2);

    // Air Phys: first call sets the boost once; later calls fall; reflect window opens/closes.
    f.cmd_vars[0] = 1;
    unsafe { f.u.mr.x2238_isCapeBoost = 0 };
    f.cmd_vars[1] = 1;
    f.self_vel = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    t::ftMr_SpecialAirS_Phys(gp);
    let l = take();
    assert_eq!(f.self_vel.y, 4.0 - 0.25);
    assert_eq!(unsafe { f.u.mr.x2238_isCapeBoost }, 1);
    assert_eq!(unsafe { f.mv.mr.SpecialS.reflecting }, 1);
    assert_eq!(&l[1..], ["reflect 31", "AEF8"]);
    f.cmd_vars[0] = 1;
    t::ftMr_SpecialAirS_Phys(gp);
    take();
    f.cmd_vars[1] = 0;
    f.set_reflecting(1);
    t::ftMr_SpecialAirS_Phys(gp);
    assert_eq!(unsafe { f.mv.mr.SpecialS.reflecting }, 0);
    assert_eq!(f.reflecting(), 0);
    take();

    // Coll: ground edge -> air state; air landing -> ground, keeping the reflect flag.
    f.cmd_vars[0] = 1;
    unsafe { f.mv.mr.SpecialS.reflecting = 1 };
    t::ftMr_SpecialS_Coll(gp);
    assert_eq!(f.cmd_vars[0], 2);
    assert_eq!(f.reflecting(), 1);
    assert!(f.accessory4_cb.is_some());
    assert_eq!(take().len(), 1);
    unsafe { f.u.mr.x2238_isCapeBoost = 1 };
    t::ftMr_SpecialAirS_Coll(gp);
    assert_eq!(unsafe { f.u.mr.x2238_isCapeBoost }, 0);
    let l = take();
    assert_eq!(l[0], "7D7FC");
    assert!(l[1].starts_with("ms 345 c4c508c "), "{}", l[1]);
}
