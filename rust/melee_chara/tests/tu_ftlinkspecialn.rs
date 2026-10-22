//! Smoke test of tu::ftlinkspecialn (Hero's Bow) through recording hooks. The arithmetic is
//! subtractions, a counter, `atan2f` (group 1's verified port) and `5 * 0.01745329252f`; no C
//! reference. One #[test]: hooks are globals.
#![allow(static_mut_refs, unused_unsafe)]

use melee_chara::stubs as st;
use melee_chara::tu::ftlinkspecialn as t;
use melee_fighter::hooks as fh;
use melee_types::*;

static mut LOG: std::vec::Vec<String> = std::vec::Vec::new();
fn log(s: String) {
    unsafe { LOG.push(s) }
}
fn take_log() -> String {
    unsafe { std::mem::take(&mut LOG).join(",") }
}

static mut BOW: *mut HSD_GObj = core::ptr::null_mut();
static mut ARROW: *mut HSD_GObj = core::ptr::null_mut();

#[test]
fn smoke() {
    unsafe { body() }
}

unsafe fn body() {
    let mut f: Box<Fighter> = Box::new(Fighter::zeroed());
    let mut g: Box<HSD_GObj> = Box::new(core::mem::zeroed());
    let da: &'static mut ftLk_DatAttrs = Box::leak(Box::new(ftLk_DatAttrs::zeroed()));
    f.dat_attrs = da as *mut _ as *mut _;
    g.user_data = &mut *f as *mut Fighter as *mut _;
    let gp: *mut HSD_GObj = &mut *g;
    let mut bow: Box<HSD_GObj> = Box::new(core::mem::zeroed());
    let mut arrow: Box<HSD_GObj> = Box::new(core::mem::zeroed());
    BOW = &mut *bow;
    ARROW = &mut *arrow;

    let mut parts: std::vec::Vec<FighterBone> = (0..80).map(|_| core::mem::zeroed()).collect();
    for (i, p) in parts.iter_mut().enumerate() {
        p.joint = (0x3000 + i * 8) as *mut _;
    }
    f.parts = parts.as_mut_ptr();
    f.facing_dir = 1.0;
    f.ground_or_air = GA_Air;
    fh::FTPARTS_GET_BONE_INDEX = Some(|_, i| i);
    // L thumb (1,2), R thumb (3,5), root (0.5,0.25).
    t::ext::LB_8000B1CC = Some(|j, off, out| {
        assert!(off.is_none());
        *out = match j as usize {
            x if x == 0x3000 + FtPart_LThumbNb as usize * 8 => Vec3 { x: 1.0, y: 2.0, z: 7.0 },
            x if x == 0x3000 + FtPart_RThumbNb as usize * 8 => Vec3 { x: 3.0, y: 5.0, z: 8.0 },
            _ => Vec3 { x: 0.5, y: 0.25, z: 9.0 },
        };
    });
    t::ext::IT_802AF1A4 = Some(|dir, _, pos, part, x10| {
        log(format!("bow {dir} {} {part} {x10}", pos.x));
        unsafe { BOW }
    });
    t::ext::IT_802A83E0 = Some(|dir, _, pos, part, xc| {
        log(format!("arrow {dir} {} {part} {xc}", pos.x));
        unsafe { ARROW }
    });
    t::ext::IT_802A8398 = Some(|_, r, l| log(format!("aim {} {} {} {}", r.x, r.y, l.x, l.y)));
    t::ext::IT_802A8A7C = Some(|_| log("arrow-destroy".into()));
    t::ext::IT_802AF304 = Some(|_| log("bow-destroy".into()));
    t::ext::ITLINKARROW_802A850C = Some(|_, r, l, ang, charge, max| {
        log(format!("fire {} {} {} {} {:08x} {charge} {max}", r.x, r.z, l.x, l.z, ang.to_bits()))
    });
    st::FIGHTER_CHANGEMOTIONSTATE = Some(|_, ms, fl, start, _, _, _| log(format!("ms {ms} {fl:x} {start}")));
    st::FTANIM_8006EBA4 = Some(|_| {});
    st::FTANIM_SETANIMRATE = Some(|_, r| log(format!("rate {r}")));
    st::FT_8008A2BC = Some(|_| log("Wait".into()));
    st::FTCO_FALL_ENTER = Some(|_| log("Fall".into()));
    st::FTCO_80096900 = Some(|_, a, b, ai, mob, lag| log(format!("fallspecial {a} {b} {ai} {mob} {lag}")));
    st::FTANIM_ISFRAMESREMAINING = Some(|_| 1);
    st::FT_80082708 = Some(|_| 0);
    st::FT_80081D0C = Some(|_| 1);
    st::FTPARTS_80074A4C = Some(|_, m, v| log(format!("parts {m} {v}")));
    st::FTPARTS_80074B0C = Some(|_, m, v| log(format!("parts-b {m} {v}")));
    st::FT_80084F3C = Some(|_| log("phys".into()));
    st::FTCOMMON_8007D7FC = Some(|_| log("7D7FC".into()));

    // GetIndex.
    assert_eq!(t::ftLk_SpecialN_GetIndex(core::ptr::null_mut()), ftLk_SpecialNIndex_None);
    f.motion_id = ftLk_MS_SpecialAirNLoop as _;
    assert_eq!(t::ftLk_SpecialN_GetIndex(gp), ftLk_SpecialNIndex_None);
    f.u.lk.x14 = BOW;
    assert_eq!(t::ftLk_SpecialN_GetIndex(gp), ftLk_SpecialNIndex_AirLoop);
    f.motion_id = ftCo_MS_AttackAirLw as _;
    assert_eq!(t::ftLk_SpecialN_GetIndex(gp), ftLk_SpecialNIndex_None);
    f.u.lk.x14 = core::ptr::null_mut();

    // Enter: a bow is created from the right thumb, the damage callback and anim rate are set.
    da.x10 = 4;
    da.xC = 6;
    da.specialn_anim_rate = 0.5;
    da.x0 = 3.0;
    da.x8 = 0.0;
    t::ftLk_SpecialN_Enter(gp);
    assert_eq!(take_log(), format!("7D7FC,ms 344 0 0,rate 0.5,bow 1 3 {} 4", FtPart_RThumbNb));
    assert_eq!(f.u.lk.x14, BOW);
    assert!(f.take_dmg_cb.is_some() && f.death2_cb.is_some());
    // A second draw (bow already exists) aborts back to Wait and clears the bow.
    t::ftLk_SpecialN_Enter(gp);
    assert_eq!(take_log(), "7D7FC,ms 344 0 0,rate 0.5,Wait");
    assert!(f.u.lk.x14.is_null());
    // Air: failure to create the bow falls (x8 == 0) or falls special (x8 != 0).
    t::ext::IT_802AF1A4 = Some(|_, _, _, _, _| core::ptr::null_mut());
    t::ftLk_SpecialAirN_Enter(gp);
    da.x8 = 3.0;
    t::ftLk_SpecialAirN_Enter(gp);
    assert_eq!(take_log(), "ms 347 0 0,rate 0.5,Fall,ms 347 0 0,rate 0.5,fallspecial 1 0 1 1 3");

    // Start anim: latch cmd_vars[2], create the arrow on the cmd_vars[0] pulse, track the aim,
    // move to Loop (SkipModel | SkipItemVis) when the animation is over.
    f.u.lk.arrow_gobj = core::ptr::null_mut();
    f.cmd_vars = [1, 0, 1, 0];
    f.mv.lk.specialn.x0.x = 0.0;
    st::FTANIM_ISFRAMESREMAINING = Some(|_| 0);
    t::ftLk_SpecialNStart_Anim(gp);
    let want_atan = melee_runtime::lbtrigf::atan2f(5.0 - 2.0, 3.0 - 1.0);
    assert_eq!(take_log(), format!("rate 0.5,arrow 1 1 {} 6,aim 3 5 1 2,ms 345 40010 0", FtPart_LThumbNb));
    assert_eq!(f.mv.lk.specialn.x0.x, 1.0);
    assert_eq!(f.cmd_vars[0], 0);
    assert_eq!(f.cmd_vars[2], 0);
    assert_eq!(f.u.lk.arrow_gobj, ARROW);
    assert_eq!((f.mv.lk.specialn.x8.y, f.mv.lk.specialn.x8.z, f.mv.lk.specialn.x14), (2.5, 4.75, 0.0));
    assert_eq!(f.mv.lk.specialn.x8.x.to_bits(), want_atan.to_bits());
    // Loop anim only tracks the aim.
    t::ftLk_SpecialNLoop_Anim(gp);
    assert_eq!(take_log(), "aim 3 5 1 2");

    // IASA: the charge counter rises to x0 (3.0); releasing B ends the charge, holding does not.
    f.mv.lk.specialn.x0.y = 2.5;
    f.input.held_buttons[0] = HSD_PAD_B as u32;
    t::ftLk_SpecialNStart_IASA(gp);
    assert_eq!((f.mv.lk.specialn.x0.y, take_log().as_str()), (3.0, ""));
    f.input.held_buttons[0] = 0;
    t::ftLk_SpecialNStart_IASA(gp);
    assert_eq!(take_log(), "ms 346 40010 0");
    t::ftLk_SpecialAirNStart_IASA(gp);
    assert_eq!(take_log(), "ms 349 40010 0");
    f.mv.lk.specialn.x0.y = 0.0;
    t::ftLk_SpecialNLoop_IASA(gp);
    assert_eq!((f.mv.lk.specialn.x0.y, take_log().as_str()), (3.0, "ms 346 40010 0"));
    t::ftLk_SpecialAirNLoop_IASA(gp);
    assert_eq!(take_log(), "ms 349 40010 0");
    f.mv.lk.specialn.x0.x = 0.0;
    f.mv.lk.specialn.x0.y = 7.0;
    t::ftLk_SpecialNStart_IASA(gp);
    assert_eq!((f.mv.lk.specialn.x0.y, take_log().as_str()), (7.0, ""));

    // End anim: show/hide the held-item part once, release the bow at the end.
    f.u.lk.x14 = BOW;
    st::FTANIM_ISFRAMESREMAINING = Some(|_| 1);
    f.mv.lk.specialn.unk_timer = 0;
    f.set_x221E_b3(1);
    t::ftLk_SpecialNEnd_Anim(gp);
    assert_eq!(take_log(), "parts 2 0,parts-b 2 0");
    assert_eq!(f.mv.lk.specialn.unk_timer, 1);
    t::ftLk_SpecialNEnd_Anim(gp);
    assert_eq!(take_log(), "");
    st::FTANIM_ISFRAMESREMAINING = Some(|_| 0);
    f.mv.lk.specialn.unk_timer = 0;
    let mut item: Box<HSD_GObj> = Box::new(core::mem::zeroed());
    f.item_gobj = &mut *item;
    t::ftLk_SpecialNEnd_Anim(gp);
    assert_eq!(take_log(), "parts 2 1,parts-b 2 1,Wait");
    assert!(f.u.lk.x14.is_null());
    f.item_gobj = core::ptr::null_mut();
    // Air end: arrow and bow both go, then Fall (x8 != 0 -> fall special).
    f.u.lk.x14 = BOW;
    f.u.lk.arrow_gobj = ARROW;
    f.mv.lk.specialn.unk_timer = 1;
    t::ftLk_SpecialAirNEnd_Anim(gp);
    assert_eq!(take_log(), "fallspecial 1 0 1 1 3");
    assert!(f.u.lk.x14.is_null() && f.u.lk.arrow_gobj.is_null());

    // Process callbacks destroy the items.
    f.u.lk.arrow_gobj = ARROW;
    f.u.lk.x14 = BOW;
    t::ftLk_SpecialN_ProcessFv10(gp);
    t::ftLk_SpecialN_ProcessFv14(gp);
    assert_eq!(take_log(), "arrow-destroy,bow-destroy");
    assert!(f.u.lk.x14.is_null() && f.u.lk.arrow_gobj.is_null());

    // Coll: End fires the arrow on the cmd_vars[1] pulse with MTXDegToRad(5), then Ground<->Air.
    f.u.lk.arrow_gobj = ARROW;
    f.cmd_vars[1] = 1;
    f.mv.lk.specialn.x0.y = 2.0;
    t::ftLk_SpecialNEnd_Coll(gp);
    assert_eq!(
        take_log(),
        format!("fire 3 0 1 0 {:08x} 2 3,ms 349 c4c5090 0", (5.0f32 * 0.01745329252f32).to_bits())
    );
    assert!(f.u.lk.arrow_gobj.is_null());
    assert_eq!(f.cmd_vars[1], 0);
    // Start/Loop Coll (ground to air) and AirStart/AirLoop/AirEnd Coll (air to ground).
    t::ftLk_SpecialNStart_Coll(gp);
    t::ftLk_SpecialNLoop_Coll(gp);
    assert_eq!(take_log(), "ms 347 c4c5090 0,ms 348 c4c5090 0");
    t::ftLk_SpecialAirNStart_Coll(gp);
    t::ftLk_SpecialAirNLoop_Coll(gp);
    t::ftLk_SpecialAirNEnd_Coll(gp);
    assert_eq!(take_log(), "7D7FC,ms 344 c4c5090 0,7D7FC,ms 345 c4c5090 0,7D7FC,ms 346 c4c5090 0");
    // Phys re-exports go to the ground/air physics.
    t::ftLk_SpecialNStart_Phys(gp);
    assert_eq!(take_log(), "phys");
}
