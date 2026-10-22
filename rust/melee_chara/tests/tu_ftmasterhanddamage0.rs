//! Smoke test of tu::ftmasterhanddamage0 (no float arithmetic): the damage entry (free items, stop
//! sounds, attribute copies), the camera anchor callback and the damage Anim countdown.
#![allow(static_mut_refs)]

use melee_chara::stubs as st;
use melee_chara::tu::ftmasterhanddamage0 as t;
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
    let mut fd: Box<ftData> = Box::new(unsafe { core::mem::zeroed() });
    let mut da = Box::new(ftMasterHand_SpecialAttrs::zeroed());
    fd.ext_attr = &mut *da as *mut _ as *mut _;
    f.ft_data = &mut *fd;
    g.user_data = &mut *f as *mut Fighter as *mut _;
    let gp: *mut HSD_GObj = &mut *g;
    // a second fighter (the Master Hand the camera follows) with a bone table
    let mut other: Box<Fighter> = Box::new(Fighter::zeroed());
    let mut og: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    let mut parts: std::vec::Vec<FighterBone> = (0..40).map(|_| FighterBone::zeroed()).collect();
    for (i, p) in parts.iter_mut().enumerate() {
        p.joint = (0x1000 + i * 8) as *mut HSD_JObj;
    }
    other.parts = parts.as_mut_ptr();
    og.user_data = &mut *other as *mut Fighter as *mut _;
    let ogp: *mut HSD_GObj = &mut *og;
    unsafe {
        st::FIGHTER_CHANGEMOTIONSTATE = Some(|_, m, _, _, _, _, _| log(format!("ms {m}")));
        st::FTANIM_8006EBA4 = Some(|_| log("anim".into()));
        st::FTANIM_ISFRAMESREMAINING = Some(|_| FRAMES);
        t::ext::IT_802F046C = Some(|i| log(format!("free {:x}", i as usize)));
        t::ext::LBAUDIOAX_800236B8 = Some(|id| {
            log(format!("stop {id}"));
            0
        });
        t::ext::LBAUDIOAX_8002438C = Some(|id| {
            log(format!("snd {id}"));
            0
        });
        t::ext::FT_800881D8 = Some(|_, id, v, p| log(format!("voice {id} {v} {p}")));
        t::ext::FTCO_800D4F24 = Some(|_, i| log(format!("ko {i}")));
        t::ext::FTMH_CAPTUREWAITMASTERHAND_80155D1C = Some(|v| log(format!("capwait {:x}", v as usize)));
        t::ext::FTBOSSLIB_GETFIGHTERGOBJ = Some(|k| {
            assert_eq!(k, Ft_Kind_MasterH);
            unsafe { OTHER }
        });
        t::ext::LB_8000B1CC = Some(|j, off, out| unsafe {
            assert!(off.is_null());
            *out = Vec3 { x: j as usize as f32, y: 1.0, z: 2.0 };
        });
        fh::PLAY_SFX = Some(|_, id, v, p| log(format!("sfx {id} {v} {p}")));
        OTHER = ogp;
    }
    // camera anchor: the waist bone of the Master Hand
    let mut v: Vec3 = unsafe { core::mem::zeroed() };
    assert!(unsafe { t::ftMh_MS_343_80151428(&mut v) });
    assert_eq!(v.x, (0x1000 + FtPart_WaistN as usize * 8) as f32);
    unsafe { OTHER = core::ptr::null_mut() };
    assert!(!unsafe { t::ftMh_MS_343_80151428(&mut v) });
    unsafe { OTHER = ogp };

    // damage entry
    da.x14C = 1.5;
    da.x154 = -2.5;
    da.x15C = 7.9;
    f.self_vel = Vec3 { x: 5.0, y: 5.0, z: 5.0 };
    f.cmd_vars[0] = 3;
    // (native layout: unk0.x20 is the same slot as dmg0.x28, see the module doc; leave it 0 here)
    f.mv.mh.unk0.x20 = 0;
    f.mv.mh.dmg0.x34 = 0x2000 as *mut Item_GObj;
    f.mv.mh.dmg0.x38 = 0x2008 as *mut Item_GObj;
    f.mv.mh.dmg0.x3C = 0x2010 as *mut Item_GObj;
    f.mv.mh.dmg0.x40 = 0x2018 as *mut Item_GObj;
    f.mv.mh.dmg0.x28 = 0; // == unk0.x20 natively (module doc): 0 keeps the capture flag clear
    f.mv.mh.dmg0.x2C = 12;
    f.mv.mh.dmg0.x30 = 13;
    f.mv.mh.unk0.x74 = 9;
    f.mv.mh.unk0.x78 = 9;
    t::ftMh_MS_343_80151484(gp);
    assert_eq!(take(), "free 2000,free 2008,free 2010,free 2018,stop 0,stop 12,stop 13,ms 344,anim,voice 320023 127 64,sfx 320024 127 64");
    assert_eq!((f.self_vel.x, f.self_vel.y, f.self_vel.z), (0.0, 1.5, -2.5));
    assert_eq!((f.mv.mh.unk0.x8, f.mv.mh.unk0.x74, f.mv.mh.unk0.x78, f.cmd_vars[0]), (7, 0, 0, 0));
    assert!(f.mv.mh.dmg0.x34.is_null() && f.mv.mh.dmg0.x38.is_null() && f.mv.mh.dmg0.x3C.is_null() && f.mv.mh.dmg0.x40.is_null());
    // a captured fighter is released first
    f.mv.mh.unk0.x20 = 1;
    f.victim_gobj = 0x3000 as *mut Fighter_GObj;
    t::ftMh_MS_343_80151484(gp);
    assert!(take().contains("stop 1,stop 12,stop 13,capwait 3000"));

    // damage Anim: the two sound counters, the Damage2 hand-off and the countdown
    da.x144 = 1;
    da.x148 = 1;
    for anim in [t::ftMh_Damage_Anim, t::ftMh_MS_345_Anim] {
        unsafe { FRAMES = 1 };
        f.cmd_vars[0] = 1;
        f.mv.mh.unk0.x74 = 0;
        f.mv.mh.unk0.x78 = 0;
        f.mv.mh.unk0.x8 = 5;
        anim(gp);
        assert_eq!((take(), f.mv.mh.unk0.x74, f.mv.mh.unk0.x78, f.mv.mh.unk0.x8, f.cmd_vars[0]), ("snd 530002".to_string(), 1, 0, 4, 0));
        f.cmd_vars[0] = 1;
        anim(gp); // x74 -> 2 > 1: second counter plays
        assert_eq!((take(), f.mv.mh.unk0.x74, f.mv.mh.unk0.x78), ("snd 530003".to_string(), 2, 1));
        f.cmd_vars[0] = 1;
        anim(gp); // both exhausted: silent
        assert_eq!((take(), f.mv.mh.unk0.x74, f.mv.mh.unk0.x78), (String::new(), 3, 2));
        // animation over while the countdown is positive -> Damage2
        unsafe { FRAMES = 0 };
        f.mv.mh.unk0.x8 = 3;
        anim(gp);
        assert_eq!((take(), f.mv.mh.unk0.x8), ("ms 345,anim".to_string(), 2));
        // the countdown reaching zero ends the hand
        f.mv.mh.unk0.x8 = 1;
        unsafe { FRAMES = 1 };
        anim(gp);
        assert_eq!(take(), "ko 0");
    }
}

static mut OTHER: *mut HSD_GObj = core::ptr::null_mut();
