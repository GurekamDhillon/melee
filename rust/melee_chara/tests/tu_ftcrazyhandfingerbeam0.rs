//! Smoke test of tu::ftcrazyhandfingerbeam0 (no float math to check against the C): the accessory
//! callback spawns one bomb per set cmd_vars flag, at the bone position plus the data offset.
#![allow(static_mut_refs)]

use melee_chara::stubs as st;
use melee_chara::tu::ftcrazyhandfingerbeam0 as t;
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
    let mut fd: Box<ftData> = Box::new(unsafe { core::mem::zeroed() });
    let mut da = Box::new(ftCrazyHand_DatAttrs::zeroed());
    let mut parts: std::vec::Vec<FighterBone> = (0..40).map(|_| FighterBone::zeroed()).collect();
    for (i, p) in parts.iter_mut().enumerate() {
        p.joint = (0x1000 + i * 8) as *mut HSD_JObj;
    }
    da.xA0 = 0.5;
    da.xA4 = 0.25;
    da.xA8_pos.x = 10.0;
    da.xA8_pos.y = 20.0;
    da.xB0 = 3.0;
    da.xB4 = 4.0;
    da.xB8 = 7.0;
    da.xBC_pos.x = 8.0;
    fd.ext_attr = &mut *da as *mut _ as *mut _;
    f.ft_data = &mut *fd;
    f.parts = parts.as_mut_ptr();
    f.facing_dir = -1.0;
    g.user_data = &mut *f as *mut Fighter as *mut _;
    let gp: *mut HSD_GObj = &mut *g;
    unsafe {
        st::FTANIM_ISFRAMESREMAINING = Some(|_| 0);
        t::ext::FTCH_GRABUNK1_8015BC88 = Some(|_| log("wait".into()));
        t::ext::LB_8000B1CC = Some(|j, off, out| unsafe {
            assert!(off.is_null());
            *out = Vec3 { x: j as usize as f32, y: 2.0, z: 3.0 };
        });
        t::ext::IT_802F0F6C = Some(|_, pos, prev, kind, facing| unsafe {
            log(format!("bomb {} {} {}/{} {} {} k{} f{}", (*pos).x, (*pos).y, (*pos).z, (*prev).x, (*prev).y, (*prev).z, kind, facing));
        });
        fh::PLAY_SFX = Some(|_, id, vol, pan| log(format!("sfx {id:x} {vol} {pan}")));
    }
    t::ftCh_FingerBeamStart_Anim(gp);
    f.cmd_vars[1] = 1;
    f.cmd_vars[3] = 5;
    unsafe { t::fn_801588B8(gp) };
    assert_eq!(f.cmd_vars[..4], [0, 0, 0, 0]);
    let b = |i: usize| (0x1000 + i * 8) as f32;
    let bx = |i: usize| (0x1000 + i * 8) as f32;
    let (b1, b3) = (bx(FtPart_BustN as usize), bx(FtPart_L3rdNa as usize));
    let k = It_Kind_CrazyHand_Bomb;
    let want = format!(
        "wait,bomb {} 22 3/{} 2 3 k{k} f-1,sfx 4e203 127 64,bomb {} 10 3/{} 2 3 k{k} f-1,sfx 4e203 127 64",
        b1 + 10.0, b1, b3 + 7.0, b3
    );
    assert_eq!(unsafe { LOG.join(",") }, want);
    t::ftCh_FingerBeamStart_Coll(gp);
}
