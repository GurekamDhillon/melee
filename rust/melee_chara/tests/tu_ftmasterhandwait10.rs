//! Smoke test of tu::ftmasterhandwait10 (hooks record the calls; the Lasers float math is
//! checked against the fused formula).
#![allow(static_mut_refs)]
mod mh_common;
use mh_common::*;
use melee_chara::tu::ftmasterhandwait10 as t;
use melee_core::fp::fmadds;
use melee_types::*;

static mut BUTTONS: u32 = 0;
static mut RANDI: i32 = 0;
static mut RANDF: f32 = 0.0;

#[test]
fn smoke() {
    let mut r = Rig::new();
    let gp = r.gp();
    unsafe {
        melee_fighter::hooks::PLINK_HEAD = Some(|_| core::ptr::null_mut());
        t::ext::STAGE_80225194 = Some(|| 1);
        t::ext::FTMH_MS_343_801510B0 = Some(|_| log("entry"));
        t::ext::HSD_PADMASTERSTATUS_BUTTON = Some(|p| {
            assert_eq!(p, 2);
            BUTTONS
        });
        t::ext::HSD_RANDI = Some(|m| {
            assert!(RANDI < m);
            RANDI
        });
        t::ext::HSD_RANDF = Some(|| RANDF);
        melee_fighter::hooks::HSD_RANDI = Some(|_| 0);
        t::ext::MPFLOORGETRIGHT = Some(|_, v| (*v).x = 9.0);
        t::ext::FTMH_MS_358_80152880 = Some(|_| log("poke"));
        t::ext::FTMH_MS_363_801530A4 = Some(|_| log("gun"));
        t::ext::FTMH_MS_372_801542E0 = Some(|_| log("grab"));
        t::ext::FTMH_MS_371_801541C8 = Some(|_, cb| log(format!("backattack {}", cb.is_some())));
    }
    t::ftMh_MS_341_8014FE10(gp);
    assert_eq!(take(), "entry");
    t::ftMh_MS_341_8014FE58(gp);
    t::ftMh_Wait1_0_Coll(gp);
    t::ftMh_Wait1_0_Phys(gp);
    assert_eq!(take(), "85134");

    // Debug pad chooser: R + dpad-left = Poke; Z + dpad-up = Grab (x2250 = Throw).
    unsafe { BUTTONS = (HSD_PAD_R | HSD_PAD_DPADLEFT) as u32 };
    t::ftMh_MS_341_80150894(gp);
    assert_eq!(take(), "poke");
    unsafe { BUTTONS = (HSD_PAD_Z | HSD_PAD_DPADUP) as u32 };
    t::ftMh_MS_341_80150894(gp);
    assert_eq!(take(), "grab");
    assert_eq!(unsafe { r.f.u.mh.x2250 }, ftMh_MS_Throw as i32);
    // L + dpad-up: stage-wide slap at (floor right, da->x38) via Wait1_2.
    r.da.x38 = 3.0;
    unsafe { BUTTONS = (HSD_PAD_L | HSD_PAD_DPADUP) as u32 };
    t::ftMh_MS_341_80150894(gp);
    assert_eq!(take(), "cms 389 0 0 1 0,8006EBA4");
    unsafe { assert_eq!((r.f.mv.mh.unk0.xC.x, r.f.mv.mh.unk0.xC.y), (9.0, 3.0)) };
    // B + dpad-up: Lasers, fused (xCC*rand)*2 + xC4, minus xCC.
    r.da.xCC_pos = Vec2 { x: 3.7, y: 1.3 };
    r.da.xC4_pos = Vec2 { x: 0.3, y: 5.1 };
    unsafe {
        RANDF = 0.37;
        BUTTONS = (HSD_PAD_B | HSD_PAD_DPADUP) as u32;
    }
    t::ftMh_MS_341_80150894(gp);
    take();
    unsafe {
        assert_eq!(r.f.mv.mh.unk0.xC.x.to_bits(), (fmadds(3.7f32 * 0.37, 2.0, 0.3) - 3.7).to_bits());
        assert_eq!(r.f.mv.mh.unk0.xC.y.to_bits(), (fmadds(1.3f32 * 0.37, 2.0, 5.1) - 1.3).to_bits());
    }

    // Anim, human slot (0): debug chooser runs, then the idle restart when the animation ends.
    unsafe {
        BUTTONS = 0;
        FRAMES = 0;
        SLOT = 0;
    }
    r.f.motion_id = ftMh_MS_Wait1_0 as _;
    r.f.u.mh.x2258 = ftMh_MS_Wait2_1 as i32;
    t::ftMh_Wait1_0_Anim(gp);
    assert_eq!(unsafe { r.f.u.mh.x2258 }, ftMh_MS_Wait2_0 as i32);
    assert_eq!(take(), "cms 342 0 0 1 0");

    // Anim, CPU slot: counter not yet expired -> only the frame-end restart.
    unsafe { SLOT = 1 };
    r.f.u.mh.x223C = 5.0;
    r.f.u.mh.x2258 = ftMh_MS_Wait1_2 as i32;
    t::ftMh_Wait1_0_Anim(gp);
    assert_eq!(unsafe { r.f.u.mh.x223C }, 4.0);
    assert_eq!(take(), "cms 342 0 0 1 0");
    // Counter expired: state 0 of the table picks Slap/Sweep via the randoms (all zero).
    // x2254 == da->x24 (0 == 0) forces tmp = 2 -> x48[2] = (7, 3): states[7] = BackPunch.
    unsafe {
        RANDI = 0;
        r.f.u.mh.x223C = 0.0;
    }
    r.f.u.mh.x2254 = 0;
    r.da.x24 = 0;
    r.f.cpu.level = 1;
    unsafe { r.f.u.mh.x2238 = 1.0 };
    r.da.x18 = 10;
    r.da.x20 = 5;
    r.da.x1C = 2;
    t::ftMh_Wait1_0_Anim(gp);
    assert_eq!(take(), "backattack true");
    assert_eq!(unsafe { (r.f.u.mh.x224C, r.f.u.mh.x2250, r.f.u.mh.x2254) }, (2, ftMh_MS_BackPunch as i32, 0));
}
