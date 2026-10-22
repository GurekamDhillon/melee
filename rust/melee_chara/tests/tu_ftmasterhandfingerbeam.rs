//! Smoke test of tu::ftmasterhandfingerbeam (no float arithmetic): the laser-spawning accessory
//! callback, the end-of-loop teardown (sounds stopped, items freed, slots cleared).
#![allow(static_mut_refs)]

use melee_chara::stubs as st;
use melee_chara::tu::ftmasterhandfingerbeam as t;
use melee_fighter::hooks as fh;
use melee_types::*;

static mut LOG: std::vec::Vec<String> = std::vec::Vec::new();
static mut FRAMES: i32 = 0;
fn log(s: String) {
    unsafe { LOG.push(s) }
}
fn take() -> String {
    unsafe { core::mem::take(&mut LOG).join(",") }
}

#[test]
fn smoke() {
    unsafe { smoke_inner() }
}

#[allow(unused_unsafe)]
unsafe fn smoke_inner() {
    let mut f: Box<Fighter> = Box::new(Fighter::zeroed());
    let mut g: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    let mut parts: std::vec::Vec<FighterBone> = (0..40).map(|_| FighterBone::zeroed()).collect();
    for (i, p) in parts.iter_mut().enumerate() {
        p.joint = (0x1000 + i * 8) as *mut HSD_JObj;
    }
    f.parts = parts.as_mut_ptr();
    f.facing_dir = -1.0;
    g.user_data = &mut *f as *mut Fighter as *mut _;
    let gp: *mut HSD_GObj = &mut *g;
    unsafe {
        st::FIGHTER_CHANGEMOTIONSTATE = Some(|_, m, _, _, _, _, _| log(format!("ms {m}")));
        st::FTANIM_8006EBA4 = Some(|_| log("anim".into()));
        st::FTANIM_ISFRAMESREMAINING = Some(|_| FRAMES);
        melee_chara::tu::ftmasterhanddamage0::ext::LB_8000B1CC = Some(|j, off, out| unsafe {
            assert!(off.is_null());
            *out = Vec3 { x: j as usize as f32, y: 1.0, z: 2.0 };
        });
        melee_chara::tu::ftmasterhanddamage0::ext::IT_802F046C = Some(|i| log(format!("free {:x}", i as usize)));
        melee_chara::tu::ftmasterhanddamage0::ext::LBAUDIOAX_800236B8 = Some(|id| {
            log(format!("stop {id}"));
            0
        });
        t::ext::IT_802F0340 = Some(|_, prev, pos, part, kind, facing| {
            assert_eq!(prev, pos);
            assert_eq!(kind, It_Kind_MasterHand_Laser);
            log(format!("laser part{part} f{facing} x{}", unsafe { (*pos).x }));
            (0x9000 + part as usize) as *mut Item_GObj
        });
        fh::LBAUDIOAX_800237A8 = Some(|s, v, p| {
            log(format!("snd {s} {v} {p}"));
            s - 319000
        });
    }
    // the callback does nothing until the script sets cmd_vars[0]
    unsafe { t::ftMh_MS_362_80152E28(gp) };
    assert_eq!(take(), "");
    f.cmd_vars[0] = 1;
    unsafe { t::ftMh_MS_362_80152E28(gp) };
    let want = [FtPart_RLegJA, FtPart_BustN, FtPart_LHandN, FtPart_L3rdNa]
        .iter()
        .map(|p| format!("laser part{p} f-1 x{}", (0x1000 + *p as usize * 8) as f32))
        .collect::<std::vec::Vec<_>>()
        .join(",");
    assert_eq!(take(), format!("{want},snd 320004 127 64,snd 320005 127 64,snd 320006 127 64"));
    assert_eq!(f.cmd_vars[0], 0);
    assert_eq!((f.mv.mh.unk0.x28, f.mv.mh.unk0.x2C, f.mv.mh.unk0.x30), (1004, 1005, 1006));
    let items = unsafe { [f.mv.mh.fingerbeam.x34, f.mv.mh.fingerbeam.x38, f.mv.mh.fingerbeam.x3C, f.mv.mh.fingerbeam.x40] };
    // Native layout defect (module doc): the third sound handle (unk0.x30, native offset 0x38) is
    // stored over the low half of the first item pointer (fingerbeam.x34), which the GC keeps apart.
    // So slot 0 is only checked for that overlay; the other three must be the spawned items.
    assert_eq!(items[0] as usize & 0xFFFF_FFFF, 1006);
    assert_eq!(items[1..].iter().map(|i| *i as usize).collect::<std::vec::Vec<_>>(), [0x9000 + FtPart_BustN as usize, 0x9000 + FtPart_LHandN as usize, 0x9000 + FtPart_L3rdNa as usize]);

    // Loop Anim waits for the motion to end, then enters End, stops the sounds and frees the items
    unsafe { FRAMES = 1 };
    t::ftMh_FingerBeamLoop_Anim(gp);
    assert_eq!(take(), "");
    unsafe { FRAMES = 0 };
    t::ftMh_FingerBeamLoop_Anim(gp);
    // (slot 0's low half is the -1 just stored over unk0.x30: the same overlay, see above)
    let frees = ["free ffffffff".to_string(), format!("free {:x}", items[1] as usize), format!("free {:x}", items[2] as usize), format!("free {:x}", items[3] as usize)].join(",");
    assert_eq!(take(), format!("ms 363,anim,stop 1004,stop 1005,stop 1006,{frees}"));
    assert_eq!((f.mv.mh.unk0.x28, f.mv.mh.unk0.x2C, f.mv.mh.unk0.x30), (-1, -1, -1));
    assert_eq!((f.mv.mh.unk0.x34, f.mv.mh.unk0.x38, f.mv.mh.unk0.x3C, f.mv.mh.unk0.x40), (0, 0, 0, 0));
    t::ftMh_Poke1_Coll(gp);
    t::ftMh_FingerBeamStart_Coll(gp);
    t::ftMh_FingerBeamLoop_Coll(gp);
}
