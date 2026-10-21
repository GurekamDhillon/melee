//! Smoke test for tu::ftdonkeyspecialhi (stateful; no fused sites) through recording hooks.
#![allow(static_mut_refs)]

use melee_chara::stubs as st;
use melee_chara::tu::ftdonkeyspecialhi as t;
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
    let a: &'static mut ftDonkeyAttributes = Box::leak(Box::new(ftDonkeyAttributes::zeroed()));
    let mut parts: std::vec::Vec<FighterBone> = (0..4).map(|_| unsafe { core::mem::zeroed() }).collect();
    let joint = 0x1000 as *mut HSD_JObj;
    parts[FtPart_TopN as usize].joint = joint;
    f.parts = parts.as_mut_ptr();
    f.dat_attrs = a as *mut _ as *mut _;
    f.co_attrs.max_jumps = 2;
    g.user_data = &mut *f as *mut Fighter as *mut _;
    let gp: *mut HSD_GObj = &mut *g;
    a.SpecialHi.x54_GROUNDED_HORIZONTAL_VELOCITY = 1.5;
    a.SpecialHi.x58_AERIAL_HORIZONTAL_VELOCITY = 2.5;
    a.SpecialHi.x4C_AERIAL_VERTICAL_VELOCITY = 3.0;
    unsafe {
        st::FIGHTER_CHANGEMOTIONSTATE = Some(|_, ms, fl, _, sp, _, _| log(format!("ms {ms} {fl:x} {sp}")));
        st::FTANIM_8006EBA4 = Some(|_| log("8006EBA4".into()));
        st::FTANIM_ISFRAMESREMAINING = Some(|_| 0);
        st::FTCO_FALL_ENTER = Some(|_| log("Fall".into()));
        st::FTCO_80096900 = Some(|_, a, b, c, m, lag| log(format!("fs {a} {b} {c} {m} {lag}")));
        st::FT_80082708 = Some(|_| 0);
        st::FT_80081D0C = Some(|_| 1);
        st::FT_CHECKGROUNDANDLEDGE = Some(|_, d| d + 1);
        st::FTCLIFFCOMMON_80081298 = Some(|_| 1);
        st::FTCOMMON_8007D7FC = Some(|_| log("7D7FC".into()));
        t::ext::EFSYNC_SPAWN = Some(|id, _, j| log(format!("ef {id} {}", j as usize)));
        t::ext::FTCLIFFCOMMON_80081370 = Some(|_| log("cliff".into()));
    }
    assert_eq!(t::coll_mf, 0xC4C5080);

    // Enter (grounded): clamps gr_vel, copies it to self_vel.x, all jumps used.
    f.gr_vel = 9.0;
    t::ftDk_SpecialHi_Enter(gp);
    assert_eq!(take(), ["ms 381 0 1", "8006EBA4", "ef 1226 4096"]);
    assert_eq!(f.gr_vel, 1.5);
    assert_eq!((f.self_vel.x, f.self_vel.y), (1.5, 0.0));
    assert_eq!(f.x1968_jumpsUsed, 2);

    // Enter (air): self_vel.x clamped, y set from the table.
    f.self_vel.x = -9.0;
    t::ftDk_SpecialAirHi_Enter(gp);
    assert_eq!(take(), ["ms 382 0 1", "8006EBA4", "ef 1226 4096"]);
    assert_eq!((f.self_vel.x, f.self_vel.y), (-1.5, 3.0));

    // Air anim: no landing lag -> Fall; with lag -> FallSpecial.
    t::ftDk_SpecialAirHi_Anim(gp);
    assert_eq!(take(), ["Fall"]);
    a.SpecialHi.x64_LANDING_LAG = 12.0;
    t::ftDk_SpecialAirHi_Anim(gp);
    assert_eq!(take(), ["fs 1 0 1 1 12"]);

    // Ground coll running off the ledge goes to the air state with coll_mf and the current frame.
    f.cur_anim_frame = 4.0;
    f.self_vel.x = 9.0;
    t::ftDk_SpecialHi_Coll(gp);
    assert_eq!(take(), ["ms 382 c4c5080 1"]);
    assert_eq!(f.self_vel.x, 2.5);

    // Air coll: rising + ground -> land; falling + ground/ledge -> land; falling + cliff -> cliff grab.
    f.self_vel.y = 1.0;
    f.gr_vel = 9.0;
    t::ftDk_SpecialAirHi_Coll(gp);
    assert_eq!(take(), ["7D7FC", "ms 381 c4c5080 1"]);
    assert_eq!(f.gr_vel, 1.5);
    f.self_vel.y = -1.0;
    t::ftDk_SpecialAirHi_Coll(gp);
    assert_eq!(take(), ["7D7FC", "ms 381 c4c5080 1"]);
    unsafe { st::FT_CHECKGROUNDANDLEDGE = Some(|_, _| 0) };
    t::ftDk_SpecialAirHi_Coll(gp);
    assert_eq!(take(), ["cliff"]);
}
