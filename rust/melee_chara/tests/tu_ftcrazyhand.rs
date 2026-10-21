//! Smoke test of tu::ftcrazyhand: OnLoad (PUSH_ATTRS + state init) and LoadSpecialAttrs (COPY_ATTRS).
#![allow(static_mut_refs)]
#[path = "common_ch/mod.rs"]
mod common;
use common::*;
use melee_chara::tu::ftcrazyhand as t;
use melee_types::*;

#[test]
fn smoke() {
    let mut r = Rig::new();
    let gp = r.gp();
    let mut items = [1usize as *mut core::ffi::c_void, 2usize as *mut _, 3usize as *mut _];
    r.fd.x48_items = items.as_mut_ptr();
    r.ext.x0 = 8;
    r.ext.x4 = 2;
    r.ext.x8 = 6;
    r.ext.x18 = 11.0;
    r.ext.x1C = -4.0;

    t::ftCh_Init_LoadSpecialAttrs(gp);
    assert_eq!(r.live.x18, 11.0);

    unsafe {
        t::ext::FTBOSSLIB_REPORTGOBJSLOTTYPE = Some(|_| log("report".into()));
        t::ext::IT_8026B3F8 = Some(|a, k| log(format!("it {} {k}", a as usize)));
    }
    // Native ftBossLib_8015C244 -> ftlib needs a fighter list; only the logic up to it is
    // checked through a panic-free path: run it under catch_unwind and verify what was written.
    let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| t::ftCh_Init_OnLoad(gp)));
    assert_eq!(take_log(), "report,it 1 127,it 2 128,it 3 129");
    assert_eq!(r.f.dat_attrs, &mut *r.backup as *mut _ as *mut _);
    assert_eq!(r.backup.x18, 11.0);
    assert_eq!((r.f.no_normal_motion(), r.f.x2229_b6(), r.f.no_kb()), (1, 1, 1));
    assert_eq!((r.f.x222A_b0(), r.f.x222A_b1(), r.f.x2229_b3()), (1, 1, 1));
    assert_eq!((r.f.cur_pos.x, r.f.cur_pos.y, r.f.cur_pos.z), (11.0, -4.0, 0.0));
    unsafe {
        assert_eq!((r.f.mv.ch.unk0.x38, r.f.mv.ch.unk0.x3C, r.f.mv.ch.unk0.x40), (-1, -1, -1));
        assert_eq!(r.f.mv.ch.unk0.x28 | r.f.mv.ch.unk0.x2C | r.f.mv.ch.unk0.x30 | r.f.mv.ch.unk0.x34, 0);
    }
    let _ = res; // the tail (fighter search + HSD_Randi) is exercised by the group-3 ftbosslib tests
    t::ftCh_Init_OnDeath(gp);
    assert_eq!(t::ftCh_Init_CostumeStrings[0][0], "PlChNr.dat");
}
