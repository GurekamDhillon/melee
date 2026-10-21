//! Smoke test of tu::ftcrazyhandpoke (no float math): the laser-spawning accessory callback and
//! the Poke1 / Poke2 teardown.
#![allow(static_mut_refs)]

use melee_chara::stubs as st;
use melee_chara::tu::{ftcrazyhandfingerbeam0 as fb, ftcrazyhandpoke as t};
use melee_fighter::hooks as fh;
use melee_types::*;

static mut LOG: std::vec::Vec<String> = std::vec::Vec::new();
fn log(s: String) {
    unsafe { LOG.push(s) }
}

#[test]
fn smoke() {
    let mut f: Box<Fighter> = Box::new(Fighter::zeroed());
    let mut g: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    let mut parts: std::vec::Vec<FighterBone> = (0..40).map(|_| FighterBone::zeroed()).collect();
    for (i, p) in parts.iter_mut().enumerate() {
        p.joint = (0x1000 + i * 8) as *mut HSD_JObj;
    }
    f.parts = parts.as_mut_ptr();
    f.facing_dir = 1.0;
    g.user_data = &mut *f as *mut Fighter as *mut _;
    let gp: *mut HSD_GObj = &mut *g;
    unsafe {
        st::FTANIM_ISFRAMESREMAINING = Some(|_| 0);
        st::FIGHTER_CHANGEMOTIONSTATE = Some(|_, m, _, _, _, _, _| log(format!("ms {m:x}")));
        st::FTANIM_8006EBA4 = Some(|_| log("anim".into()));
        fb::ext::FTCH_GRABUNK1_8015BC88 = Some(|_| log("wait".into()));
        fb::ext::LB_8000B1CC = Some(|j, _, out| unsafe { *out = Vec3 { x: j as usize as f32, y: 1.0, z: 2.0 } });
        t::ext::IT_802F0340 = Some(|_, prev, pos, part, kind, facing| {
            assert_eq!(prev, pos);
            log(format!("laser part{part} kind{kind} f{facing} x{}", unsafe { (*pos).x }));
            (0x9000 + part as usize) as *mut Item_GObj
        });
        t::ext::IT_802F046C = Some(|i| log(format!("free {:x}", i as usize)));
        t::ext::LBAUDIOAX_800236B8 = Some(|id| {
            log(format!("stop {id}"));
            0
        });
        fh::LBAUDIOAX_800237A8 = Some(|s, v, p| {
            log(format!("snd {s} {v} {p}"));
            s - 319000
        });
    }
    // callback does nothing until the script sets cmd_vars[0]
    unsafe { t::fn_80158534(gp) };
    assert!(unsafe { LOG.is_empty() });
    f.cmd_vars[0] = 1;
    unsafe { t::fn_80158534(gp) };
    assert_eq!(f.cmd_vars[0], 0);
    let k = It_Kind_MasterHand_Laser;
    let x = |p: Fighter_Part| (0x1000 + p as usize * 8) as f32;
    let want = format!(
        "laser part{} kind{k} f1 x{},laser part{} kind{k} f1 x{},laser part{} kind{k} f1 x{},laser part{} kind{k} f1 x{},snd 320004 127 64,snd 320005 127 64,snd 320006 127 64",
        FtPart_RLegJA, x(FtPart_RLegJA), FtPart_BustN, x(FtPart_BustN), FtPart_LHandN, x(FtPart_LHandN), FtPart_L3rdNa, x(FtPart_L3rdNa)
    );
    assert_eq!(unsafe { LOG.join(",") }, want);
    assert_eq!(unsafe { (f.mv.ch.unk0.x38, f.mv.ch.unk0.x3C, f.mv.ch.unk0.x40) }, (1004, 1005, 1006));

    // Poke1: finish -> Poke2 start (sounds stopped, ids reset), lasers freed and cleared.
    // (Native layout: grab.x30/x34 are 8-byte pointers overlaying unk0.x38..x40, so only x28/x2C
    // carry items here; see the caveat in ftCh_Poke1_Anim.)
    unsafe {
        LOG.clear();
        f.mv.ch.unk0.x38 = 0;
        f.mv.ch.unk0.x3C = 0;
        f.mv.ch.unk0.x40 = 0;
        f.mv.ch.grab.x28 = 0x9001 as *mut Item_GObj;
        f.mv.ch.grab.x2C = 0x9002 as *mut Item_GObj;
        f.mv.ch.unk0.x38 = 1004;
        f.mv.ch.unk0.x3C = 1005;
        f.mv.ch.unk0.x40 = 1006;
    }
    t::ftCh_Poke1_Anim(gp);
    let got = unsafe { LOG.join(",") };
    assert!(got.starts_with("ms 168,anim,stop 1004,stop 1005,stop 1006,free 9001,free 9002,free "), "{got}");
    assert_eq!(got.matches("free").count(), 4);
    assert_eq!(unsafe { (f.mv.ch.unk0.x38, f.mv.ch.unk0.x3C, f.mv.ch.unk0.x40) }, (-1, -1, -1));
    unsafe { LOG.clear() };
    t::ftCh_Poke2_Anim(gp);
    t::ftCh_Poke2_Coll(gp);
    t::ftCh_Poke1_Coll(gp);
    t::ftCh_PaperCrush_Coll(gp);
    assert_eq!(unsafe { LOG.join(",") }, "wait");
}
