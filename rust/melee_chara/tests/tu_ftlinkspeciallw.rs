//! Smoke test of tu::ftlinkspeciallw (Bomb) through recording hooks. No float arithmetic, so no C
//! reference. One #[test]: hooks are globals.
#![allow(static_mut_refs, unused_unsafe)]

use melee_chara::stubs as st;
use melee_chara::tu::ftlinkspeciallw as t;
use melee_fighter::hooks as fh;
use melee_types::*;

static mut LOG: std::vec::Vec<String> = std::vec::Vec::new();
fn log(s: String) {
    unsafe { LOG.push(s) }
}
fn take_log() -> String {
    unsafe { std::mem::take(&mut LOG).join(",") }
}

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

    st::FIGHTER_CHANGEMOTIONSTATE = Some(|_, ms, fl, _, _, _, _| log(format!("ms {ms} {fl:x}")));
    st::FTANIM_8006EBA4 = Some(|_| {});
    st::FTCO_800957F4 = Some(|_, ms| log(format!("rethrow {ms}")));
    fh::IT_GET_KIND = Some(|_| It_Kind_Link_Bomb as i32 + 1);

    // Enter without an item: start the bomb pull and arm the spawn callback.
    f.anon_2210.throw_flags = 0xFFFF;
    t::ftLk_SpecialLw_Enter(gp);
    t::ftLk_SpecialAirLw_Enter(gp);
    assert_eq!(take_log(), "ms 358 0,ms 359 0");
    assert_eq!(f.anon_2210.throw_flags, 0);
    assert!(f.accessory4_cb.is_some());

    // Enter holding an item: bomb kinds re-throw (msid 108 ground / 112 air), others do nothing;
    // either way no state change.
    let mut item: Box<HSD_GObj> = Box::new(core::mem::zeroed());
    f.item_gobj = &mut *item;
    t::ftLk_SpecialLw_Enter(gp);
    fh::IT_GET_KIND = Some(|_| 3);
    t::ftLk_SpecialAirLw_Enter(gp);
    fh::IT_GET_KIND = Some(|_| It_Kind_Link_Bomb as i32);
    t::ftLk_SpecialAirLw_Enter(gp);
    assert_eq!(take_log(), "rethrow 108,rethrow 112");

    // spawnBomb: needs the throw pulse; a failed spawn leaves item_gobj null and skips the pickup.
    f.item_gobj = core::ptr::null_mut();
    let mut parts: std::vec::Vec<FighterBone> = (0..80).map(|_| core::mem::zeroed()).collect();
    for (i, p) in parts.iter_mut().enumerate() {
        p.joint = (0x2000 + i * 8) as *mut _;
    }
    f.parts = parts.as_mut_ptr();
    f.facing_dir = -1.0;
    da.x48 = 9;
    let fd: &'static mut ftData = Box::leak(Box::new(core::mem::zeroed()));
    let x8: &'static mut ftData_x8 = Box::leak(Box::new(core::mem::zeroed()));
    x8.x10 = 5;
    fd.x8 = x8;
    f.ft_data = fd;
    fh::FTPARTS_GET_BONE_INDEX = Some(|_, i| i);
    t::ext::LB_8000B1CC = Some(|j, off, out| {
        assert!(off.is_none());
        log(format!("joint {:x}", j as usize));
        *out = Vec3 { x: 1.0, y: 2.0, z: 3.0 };
    });
    t::ext::IT_8029DD58 = Some(|_, pos, x10, part, dir| {
        log(format!("bomb {} {} {} {part} {dir}", pos.x, pos.y, x10));
        core::ptr::null_mut()
    });
    st::FTPARTS_80074A4C = Some(|_, m, v| log(format!("parts {m} {v}")));
    t::spawnBomb(gp);
    assert_eq!(take_log(), "");
    f.anon_2210.anon_0.set_throw_flags_b0(1);
    t::spawnBomb(gp);
    assert_eq!(take_log(), format!("joint {:x},bomb 1 2 5 9 -1,parts 2 1", 0x2000 + FtPart_LThumbNb as usize * 8));
    assert_eq!(f.anon_2210.anon_0.throw_flags_b0(), 0);
    assert!(f.item_gobj.is_null());

    // Anim: ground waits, air falls, once the animation is over.
    st::FTANIM_ISFRAMESREMAINING = Some(|_| 1);
    st::FT_8008A2BC = Some(|_| log("Wait".into()));
    st::FTCO_FALL_ENTER = Some(|_| log("Fall".into()));
    t::ftLk_SpecialLw_Anim(gp);
    t::ftLk_SpecialAirLw_Anim(gp);
    assert_eq!(take_log(), "");
    st::FTANIM_ISFRAMESREMAINING = Some(|_| 0);
    t::ftLk_SpecialLw_Anim(gp);
    t::ftLk_SpecialAirLw_Anim(gp);
    assert_eq!(take_log(), "Wait,Fall");
}
