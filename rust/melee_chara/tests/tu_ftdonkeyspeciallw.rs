//! Smoke test for tu::ftdonkeyspeciallw (stateful; no fused sites) through recording hooks.
#![allow(static_mut_refs)]

use melee_chara::stubs as st;
use melee_chara::tu::ftdonkeyspeciallw as t;
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
    let mut parts: std::vec::Vec<FighterBone> = (0..4).map(|_| unsafe { core::mem::zeroed() }).collect();
    parts[FtPart_TopN as usize].joint = 0x1000 as *mut HSD_JObj;
    parts[FtPart_TransN as usize].joint = 0x2000 as *mut HSD_JObj;
    f.parts = parts.as_mut_ptr();
    g.user_data = &mut *f as *mut Fighter as *mut _;
    let gp: *mut HSD_GObj = &mut *g;
    unsafe {
        st::FIGHTER_CHANGEMOTIONSTATE = Some(|_, ms, fl, _, sp, _, _| log(format!("ms {ms} {fl:x} {sp}")));
        st::FTANIM_ISFRAMESREMAINING = Some(|_| 0);
        t::ext::EFSYNC_SPAWN = Some(|id, _, j| log(format!("sync {id} {}", j as usize)));
        t::ext::EFASYNC_SPAWN = Some(|_, _, k, gfx, j| log(format!("async {k} {gfx} {}", j as usize)));
        t::ext::FT_80089824 = Some(|_| log("89824".into()));
        t::ext::FT_800892A0 = Some(|_| log("892A0".into()));
        t::ext::FTDK_INIT_8010DB3C = Some(|_| log("DB3C".into()));
    }
    // Start anim done -> loop: flags cleared, effect spawned once, callbacks installed.
    unsafe { f.anon_2210.throw_flags = 5 };
    t::ftDk_SpecialLwStart_Anim(gp);
    assert_eq!(take(), ["ms 384 0 1", "async 1 1228 4096"]);
    unsafe { assert_eq!(f.anon_2210.throw_flags, 0) };
    assert_eq!(f.x2219_b0(), 1);
    assert!(f.x21EC.is_some() && f.accessory4_cb.is_some());
    assert!(f.pre_hitlag_cb.is_some() && f.post_hitlag_cb.is_some());
    unsafe {
        (f.x21EC.unwrap())(gp);
        (f.accessory4_cb.unwrap())(gp);
    }
    assert_eq!(take(), ["89824", "892A0", "DB3C"]);

    // Second pass: x2219_b0 already set -> no second effect.
    t::ftDk_SpecialLwStart_Anim(gp);
    assert_eq!(take(), ["ms 384 0 1"]);

    // Loop anim: a queued B press restarts the loop, otherwise it ends (End0 = 385).
    unsafe { f.mv.dk.speciallw.x0 = 1 };
    t::ftDk_SpecialLwLoop_Anim(gp);
    assert_eq!(take(), ["ms 384 0 1"]);
    unsafe { assert_eq!(f.mv.dk.speciallw.x0, 0) };
    t::ftDk_SpecialLwLoop_Anim(gp);
    assert_eq!(take(), ["ms 385 0 1"]);
    unsafe { st::FTANIM_ISFRAMESREMAINING = Some(|_| 1) };
    t::ftDk_SpecialLwLoop_Anim(gp);
    t::ftDk_SpecialLwStart_Anim(gp);
    assert!(take().is_empty());

    // Accessory callbacks: spawn the sync effect on TransN once, clear the callback.
    f.set_x2219_b0(0);
    f.accessory4_cb = Some(t::cb_ftDk_SpecialLw_8010E0CC);
    unsafe { (f.accessory4_cb.unwrap())(gp) };
    assert_eq!(take(), ["sync 1222 8192"]);
    assert!(f.accessory4_cb.is_none());
    f.set_x2219_b0(0);
    t::ftDk_SpecialLw_8010E148(gp);
    assert_eq!(take(), ["sync 1223 8192"]);
    t::ftDk_SpecialLw_8010E148(gp);
    assert!(take().is_empty());
}
