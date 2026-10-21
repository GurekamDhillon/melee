//! Smoke test of tu::ftcrazyhandgrabunk1b174: Init, the camera script state machine
//! (`fn_8015B2C0`, one `mv.ch.unk4.x0` step per call), and the camera follow callback.
#![allow(static_mut_refs)]

use melee_chara::stubs as st;
use melee_chara::tu::{ftcrazyhandfingerbeam0 as fb, ftcrazyhandgrabunk1b174 as t};
use melee_fighter::hooks as fh;
use melee_types::*;

static mut LOG: std::vec::Vec<String> = std::vec::Vec::new();
fn log(s: String) {
    unsafe { LOG.push(s) }
}
fn take() -> String {
    unsafe { core::mem::take(&mut LOG).join(",") }
}
static mut READY: bool = false;

#[test]
fn smoke() {
    let mut f: Box<Fighter> = Box::new(Fighter::zeroed());
    let mut g: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    let mut fd: Box<ftData> = Box::new(unsafe { core::mem::zeroed() });
    let mut da = Box::new(ftCrazyHand_DatAttrs::zeroed());
    da.x18 = 11.0;
    da.x1C = -12.0;
    fd.ext_attr = &mut *da as *mut _ as *mut _;
    f.ft_data = &mut *fd;
    g.user_data = &mut *f as *mut Fighter as *mut _;
    let gp: *mut HSD_GObj = &mut *g;
    unsafe {
        st::FTANIM_ISFRAMESREMAINING = Some(|_| 0);
        st::FIGHTER_CHANGEMOTIONSTATE = Some(|_, m, _, _, _, _, _| log(format!("ms {m:x}")));
        st::FTANIM_8006EBA4 = Some(|_| log("anim".into()));
        fb::ext::FTCH_GRABUNK1_8015BC88 = Some(|_| log("wait".into()));
        fh::PLAY_SFX = Some(|_, id, v, p| log(format!("sfx {id} {v} {p}")));
        // ftBossLib_8015C09C -> HSD_JObjSetRotation is a todo!() stub in ftbosslib; the Init test
        // below stops at it (everything before has run)
        t::ext::CAMERA_8002E6FC = Some(|a| log(format!("E6FC {a}")));
        t::ext::CAMERA_8002E948 = Some(|_| log("E948".into()));
        t::ext::CAMERA_8002EB5C = Some(|a| log(format!("EB5C {a}")));
        t::ext::CAMERA_8002EC7C = Some(|a| log(format!("EC7C {a}")));
        t::ext::CAMERA_8002ED9C = Some(|a| log(format!("ED9C {a}")));
        t::ext::CAMERA_8002EEC8 = Some(|a| log(format!("EEC8 {a}")));
        t::ext::CAMERA_8002EF14 = Some(|| log("EF14".into()));
        t::ext::CAMERA_8002F0E4 = Some(|a| log(format!("F0E4 {a}")));
        t::ext::CAMERA_8002F260 = Some(|| READY);
        t::ext::CAMERA_8002F274 = Some(|| log("F274".into()));
        t::ext::CAMERA_SETMODETOSTANDARD = Some(|| log("std".into()));
        fb::ext::LB_8000B1CC = Some(|_, _, _| log("waist".into()));
        fh::PLINK_HEAD = Some(|_| core::ptr::null_mut());
    }
    // Anim: TagFail marks x2258 then hands back to the wait state
    t::ftCh_TagFail_Anim(gp);
    assert_eq!(unsafe { f.u.mh.x2258 }, 341);
    assert_eq!(take(), "wait");

    // camera script. No fighters on the field: the lookup of an enemy fails and uses 0.
    unsafe { f.mv.ch.unk4.x0 = 0 };
    f.cur_pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    unsafe { t::fn_8015B2C0(gp) };
    let pi = core::f64::consts::PI;
    assert_eq!(
        take(),
        format!("E6FC 0,ED9C 40,EEC8 45,EC7C {},EF14,EC7C 0,F0E4 120", (-pi) as f32)
    );
    assert_eq!(unsafe { f.mv.ch.unk4.x0 }, 1);
    unsafe { READY = false };
    unsafe { t::fn_8015B2C0(gp) };
    assert_eq!(take(), "");
    unsafe { READY = true };
    unsafe { t::fn_8015B2C0(gp) };
    assert_eq!(take(), "E948,ED9C 120,F0E4 20");
    unsafe { t::fn_8015B2C0(gp) };
    assert_eq!(take(), format!("EC7C {},F0E4 60", (pi / 2.0) as f32));
    unsafe { t::fn_8015B2C0(gp) };
    assert_eq!(take(), format!("ED9C 90,EC7C {},EB5C {},EF14", (-pi / 9.0) as f32, (-pi / 12.0) as f32));
    unsafe { t::fn_8015B2C0(gp) };
    assert_eq!(take(), "F274");
    assert_eq!(unsafe { (f.mv.ch.unk4.x0, f.mv.ch.unk4.x8) }, (5, 120));
    for _ in 0..120 {
        unsafe { t::fn_8015B2C0(gp) };
    }
    assert_eq!(take(), format!("EC7C {},EF14,ED9C 120", (-pi / 9.0) as f32));
    for _ in 0..30 {
        unsafe { t::fn_8015B2C0(gp) };
    }
    assert_eq!(take(), format!("EC7C {},EF14", (pi / 9.0) as f32));
    for _ in 0..30 {
        unsafe { t::fn_8015B2C0(gp) };
    }
    assert_eq!(take(), format!("ED9C 180,EC7C {},EB5C {},EF14", (-pi / 2.0) as f32, (-pi / 9.0) as f32));
    unsafe { t::fn_8015B2C0(gp) };
    assert_eq!(take(), "F274");
    for _ in 0..50 {
        unsafe { t::fn_8015B2C0(gp) };
    }
    assert_eq!(take(), "std");
    assert_eq!(unsafe { f.mv.ch.unk4.x0 }, 10);
    unsafe { t::fn_8015B2C0(gp) };
    assert_eq!(take(), "");

    // camera follow: false with no Crazy Hand, true (and the waist position) with one
    let mut v = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    assert!(!t::fn_8015B4EC(&mut v));
    t::ftCh_TagFail_Phys(gp);
    t::ftCh_TagFail_Coll(gp);

    // Init: position, callback and script state are set before the sound / facing helpers run;
    // those still reach todo!() stubs (ft_800881D8's audio, ftBossLib_8015C09C's joint rotation),
    // so only the state set before them is checked.
    let addr = gp as usize;
    let _ = std::panic::catch_unwind(move || t::ftCh_GrabUnk1_8015B174(addr as *mut HSD_GObj));
    assert_eq!(take().split(',').take(2).collect::<std::vec::Vec<_>>(), ["ms 183", "anim"]);
    assert_eq!((f.cur_pos.x, f.cur_pos.y, f.cur_pos.z), (11.0, -12.0, 0.0));
    assert_eq!(f.accessory4_cb.map(|c| c as usize), Some(t::fn_8015B2C0 as usize));
    assert_eq!(unsafe { f.mv.ch.unk4.x0 }, 0);
}
