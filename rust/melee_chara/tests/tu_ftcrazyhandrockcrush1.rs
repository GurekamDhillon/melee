//! Smoke test of tu::ftcrazyhandrockcrush1 (float countdown against an int attribute).
#![allow(static_mut_refs)]

use melee_chara::stubs as st;
use melee_chara::tu::ftcrazyhandrockcrush1 as t;
use melee_types::*;

/// ftBossLib_8015C190 (the last call of Phys) reaches the mp floor helpers, still todo!() stubs in
/// ftbosslib's `ext`; everything before it has run by then, which is what is checked.
fn phys(gp: *mut HSD_GObj) {
    let addr = gp as usize;
    let r = std::panic::catch_unwind(move || t::ftCh_RockCrushDown_Phys(addr as *mut HSD_GObj));
    let _ = r; // Err while the mpFloor stub is todo!(); Ok once it is ported
}

#[test]
fn smoke() {
    let mut f: Box<Fighter> = Box::new(Fighter::zeroed());
    let mut g: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    let mut fd: Box<ftData> = Box::new(unsafe { core::mem::zeroed() });
    let mut da = Box::new(ftCrazyHand_DatAttrs::zeroed());
    da.x48 = 50;
    da.x44 = 1.5;
    fd.ext_attr = &mut *da as *mut _ as *mut _;
    f.ft_data = &mut *fd;
    g.user_data = &mut *f as *mut Fighter as *mut _;
    let gp: *mut HSD_GObj = &mut *g;
    unsafe {
        st::FT_80085134 = Some(|_| {});
        melee_fighter::hooks::PLINK_HEAD = Some(|_| core::ptr::null_mut());
    }
    // countdown above x48: velocity is cleared and the boss helpers are not asked for a target
    f.self_vel.x = 9.0;
    f.mv.ch.unk0.x0 = 100.0;
    phys(gp);
    assert_eq!(unsafe { f.mv.ch.unk0.x0 }, 99.0);
    assert_eq!(f.self_vel.x, 0.0);
    // negative countdown also clears
    f.self_vel.x = 9.0;
    f.mv.ch.unk0.x0 = 0.0;
    phys(gp);
    assert_eq!(unsafe { f.mv.ch.unk0.x0 }, -1.0);
    assert_eq!(f.self_vel.x, 0.0);
}
