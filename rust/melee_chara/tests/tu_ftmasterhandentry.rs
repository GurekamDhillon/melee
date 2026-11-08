//! Smoke test of tu::ftmasterhandentry: the entry setup, the Anim hand-off and the whole camera
//! cut-scene state machine (every Camera_ call and its f32-converted double argument).
#![allow(static_mut_refs)]

use melee_chara::stubs as st;
use melee_chara::tu::ftmasterhandentry as t;
use melee_types::*;

static mut LOG: std::vec::Vec<String> = std::vec::Vec::new();
static mut FRAMES: i32 = 0;
static mut CAM_DONE: bool = false;
static mut ENEMY: *mut HSD_GObj = core::ptr::null_mut();
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
    unsafe {
        st::FIGHTER_CHANGEMOTIONSTATE = Some(|_, m, _, _, _, _, _| log(format!("ms {m}")));
        st::FTANIM_8006EBA4 = Some(|_| log("anim".into()));
        st::FTANIM_ISFRAMESREMAINING = Some(|_| FRAMES);
        melee_chara::tu::ftmasterhanddamage0::ext::FT_800881D8 = Some(|_, id, v, p| log(format!("voice {id} {v} {p}")));
        t::ext::FTBOSSLIB_8015C09C = Some(|_, x| log(format!("face {x}")));
        melee_chara::tu::ftmasterhanddamage0::ext::FTMH_MS_389_80151018 = Some(|_| log("wait".into()));
        t::ext::FTLIB_FINDNEARESTOPPONENT = Some(|_, _| unsafe { ENEMY });
        t::ext::FTLIB_GETPLAYERINDEX = Some(|_| 3);
        t::ext::CAMERA_8002E6FC = Some(|a| log(format!("target {a}")));
        t::ext::CAMERA_8002ED9C = Some(|a| log(format!("dist {a}")));
        t::ext::CAMERA_8002EEC8 = Some(|a| log(format!("fov {a}")));
        t::ext::CAMERA_8002EC7C = Some(|a| log(format!("yaw {}", a.to_bits())));
        t::ext::CAMERA_8002EB5C = Some(|a| log(format!("pitch {}", a.to_bits())));
        t::ext::CAMERA_8002EF14 = Some(|| log("go".into()));
        t::ext::CAMERA_8002F0E4 = Some(|a| log(format!("frames {a}")));
        t::ext::CAMERA_8002F260 = Some(|| CAM_DONE);
        t::ext::CAMERA_8002F274 = Some(|| log("snap".into()));
        t::ext::CAMERA_8002E948 = Some(|_| log("anchor".into()));
        t::ext::CAMERA_SETMODETOSTANDARD = Some(|| log("standard".into()));
    }
    // entry setup
    da.x30_pos2 = Vec2 { x: 12.5, y: -7.0 };
    f.cur_pos.z = 9.0;
    t::ftMh_MS_343_801510B0(gp);
    assert_eq!(take(), "ms 343,anim,voice 320001 127 64,face -1");
    assert_eq!((f.cur_pos.x, f.cur_pos.y, f.cur_pos.z), (12.5, -7.0, 0.0));
    assert!(f.accessory4_cb.is_some());
    assert_eq!(f.mv.mh.unk4.x0, ftMh_UnkEnum0_Unk00);
    // Anim: nothing until the motion ends, then Wait1_0 is remembered and the Wait hand-off runs
    unsafe { FRAMES = 1 };
    t::ftMh_Entry_Anim(gp);
    assert_eq!(take(), "");
    unsafe { FRAMES = 0 };
    t::ftMh_Entry_Anim(gp);
    assert_eq!((take(), f.u.mh.x2258), ("wait".to_string(), ftMh_MS_Wait1_0 as i32));
    t::ftMh_Entry_Phys(gp);
    t::ftMh_Entry_Coll(gp);

    // camera sequence
    let pi = core::f64::consts::PI;
    let yaw = |v: f64| (v as f32).to_bits();
    let cb = |n: usize| {
        for _ in 0..n {
            unsafe { t::ftMh_MS_343_801511FC(gp) };
        }
    };
    f.mv.mh.unk4.x0 = ftMh_UnkEnum0_Unk00;
    unsafe { ENEMY = 0x500 as *mut HSD_GObj };
    cb(1);
    assert_eq!(take(), format!("target 3,dist 40,fov 45,yaw {},go,yaw 0,frames 120", yaw(-pi)));
    assert_eq!(f.mv.mh.unk4.x0, ftMh_UnkEnum0_Unk01);
    // waits for the camera
    unsafe { CAM_DONE = false };
    cb(1);
    assert_eq!((take(), f.mv.mh.unk4.x0), (String::new(), ftMh_UnkEnum0_Unk01));
    unsafe { CAM_DONE = true };
    cb(1);
    assert_eq!((take(), f.mv.mh.unk4.x0), ("anchor,dist 120,frames 20".to_string(), ftMh_UnkEnum0_Unk02));
    cb(1);
    assert_eq!((take(), f.mv.mh.unk4.x0), (format!("yaw {},frames 60", yaw(pi / 2.0)), 3));
    cb(1);
    assert_eq!(take(), format!("dist 90,yaw {},pitch {},go", yaw(-pi / 9.0), yaw(-pi / 12.0)));
    assert_eq!(f.mv.mh.unk4.x0, ftMh_UnkEnum0_Unk04);
    cb(1);
    assert_eq!((take(), f.mv.mh.unk4.x8, f.mv.mh.unk4.x0), ("snap".to_string(), 120, ftMh_UnkEnum0_Unk05));
    cb(119);
    assert_eq!((take(), f.mv.mh.unk4.x8), (String::new(), 1));
    cb(1);
    assert_eq!(take(), format!("yaw {},go,dist 120", yaw(-pi / 9.0)));
    assert_eq!((f.mv.mh.unk4.x8, f.mv.mh.unk4.x0), (30, ftMh_UnkEnum0_Unk06));
    cb(30);
    assert_eq!(take(), format!("yaw {},go", yaw(pi / 9.0)));
    assert_eq!((f.mv.mh.unk4.x8, f.mv.mh.unk4.x0), (30, ftMh_UnkEnum0_Unk07));
    cb(30);
    assert_eq!(take(), format!("dist 180,yaw {},pitch {},go", yaw(-pi / 2.0), yaw(-pi / 9.0)));
    assert_eq!((f.mv.mh.unk4.x8, f.mv.mh.unk4.x0), (50, ftMh_UnkEnum0_Unk08));
    cb(1);
    assert_eq!((take(), f.mv.mh.unk4.x0), ("snap".to_string(), ftMh_UnkEnum0_Unk09));
    cb(50);
    assert_eq!((take(), f.mv.mh.unk4.x0), ("standard".to_string(), ftMh_UnkEnum0_Unk10));
    cb(3);
    assert_eq!(take(), "");
    // no enemy: the camera targets slot 0
    f.mv.mh.unk4.x0 = ftMh_UnkEnum0_Unk00;
    unsafe { ENEMY = core::ptr::null_mut() };
    cb(1);
    assert!(take().starts_with("target 0,dist 40"));
}
