//! Smoke test of tu::ftsandbag: init callbacks and the joint-constraint setup, through
//! recording hooks.
#![allow(static_mut_refs)]

use melee_chara::stubs as st;
use melee_chara::tu::ftsandbag as t;
use melee_types::*;

static mut LOG: std::vec::Vec<String> = std::vec::Vec::new();
fn log(s: String) {
    unsafe { LOG.push(s) }
}
fn take() -> String {
    unsafe { std::mem::take(&mut LOG).join(",") }
}

#[test]
fn smoke() {
    let mut f: Box<Fighter> = Box::new(Fighter::zeroed());
    let mut g: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    let fd: &'static mut ftData = Box::leak(Box::new(unsafe { core::mem::zeroed() }));
    let ext_attr: &'static mut [u32; 2] = Box::leak(Box::new([7u32, 9u32]));
    let backup: &'static mut [u32; 2] = Box::leak(Box::new([0u32; 2]));
    let live: &'static mut [u32; 2] = Box::leak(Box::new([0u32; 2]));
    fd.ext_attr = ext_attr.as_mut_ptr() as *mut core::ffi::c_void;
    f.ft_data = fd;
    f.dat_attrs_backup = backup.as_mut_ptr() as *mut core::ffi::c_void;
    // Fake joints: each part gets two distinct non-null pointers (value = part * 16 + {1, 2}).
    let mut parts: std::vec::Vec<FighterBone> = (0..64).map(|_| FighterBone::zeroed()).collect();
    for (i, p) in parts.iter_mut().enumerate() {
        p.joint = (i * 16 + 1) as *mut HSD_JObj;
        p.x4_jobj2 = (i * 16 + 2) as *mut HSD_JObj;
    }
    f.parts = parts.as_mut_ptr();
    g.user_data = &mut *f as *mut Fighter as *mut _;
    let gp: *mut HSD_GObj = &mut *g;
    unsafe {
        t::ext::LB_8000C1C0 = Some(|a, b| log(format!("c1c0 {} {}", a as usize, b as usize)));
        t::ext::LB_8000C228 = Some(|a, b| log(format!("c228 {} {}", a as usize, b as usize)));
        t::ext::LB_8000C420 = Some(|a, fl, l| log(format!("c420 {} {} {:08x}", a as usize, fl, l.to_bits())));
    }

    t::ftSb_Init_OnLoad(gp);
    assert_eq!(f.is_sandbag(), 1);
    assert_eq!(f.x2226_b0(), 1);
    assert_eq!(f.dat_attrs, backup.as_mut_ptr() as *mut core::ffi::c_void);
    assert_eq!(*backup, [7, 9]);
    let w = FtPart_WaistN as usize * 16;
    let rl = FtPart_RLegJ as usize * 16;
    let ll = FtPart_LLegJ as usize * 16;
    let lla = FtPart_LLegJA as usize * 16;
    let ls = FtPart_LShoulderN as usize * 16;
    let rs = FtPart_RShoulderJ as usize * 16;
    let hp = (-core::f64::consts::PI / 2.0) as f32;
    let d86 = -86.0_f32 * 0.01745329252_f32;
    let want = [
        format!("c1c0 {} {}", w + 1, rl + 1),
        format!("c1c0 {} {}", w + 2, rl + 2),
        format!("c1c0 {} {}", w + 1, ls + 1),
        format!("c1c0 {} {}", w + 2, ls + 2),
        format!("c228 {} {}", ll + 1, rs + 1),
        format!("c228 {} {}", ll + 2, rs + 2),
        format!("c420 {} 1 {:08x}", ll + 1, hp.to_bits()),
        format!("c420 {} 2 {:08x}", ll + 1, hp.to_bits()),
        format!("c420 {} 1 {:08x}", ll + 2, hp.to_bits()),
        format!("c420 {} 2 {:08x}", ll + 2, hp.to_bits()),
        format!("c228 {} {}", lla + 1, w + 1),
        format!("c228 {} {}", lla + 2, w + 2),
        format!("c420 {} 1 {:08x}", lla + 1, d86.to_bits()),
        format!("c420 {} 2 {:08x}", lla + 1, d86.to_bits()),
        format!("c420 {} 1 {:08x}", lla + 2, d86.to_bits()),
        format!("c420 {} 2 {:08x}", lla + 2, d86.to_bits()),
    ]
    .join(",");
    assert_eq!(take(), want);

    f.dat_attrs = live.as_mut_ptr() as *mut core::ffi::c_void;
    t::ftSb_Init_LoadSpecialAttrs(gp);
    assert_eq!(*live, [7, 9]);

    // 8014FBA4: grounded -> no air-to-ground call; changes to motion 0x155 and sets the timer.
    unsafe {
        st::FIGHTER_CHANGEMOTIONSTATE = Some(|_, msid, fl, s, sp, bl, a3| {
            log(format!("cms {msid} {fl} {s} {sp} {bl} {}", a3 as usize))
        });
        st::FTCOMMON_8007D7FC = Some(|_| log("to_ground".into()));
    }
    let cd: &'static mut ftCommonData = Box::leak(Box::new(unsafe { core::mem::zeroed() }));
    cd.x5F0 = 12;
    unsafe { melee_fighter::hooks::P_FT_COMMON_DATA = cd };
    f.ground_or_air = GA_Ground;
    t::ftSb_Init_8014FBA4(gp);
    assert_eq!(take(), "cms 341 0 0 1 0 0");
    f.ground_or_air = GA_Air;
    t::ftSb_Init_8014FBA4(gp);
    assert_eq!(take(), "to_ground,cms 341 0 0 1 0 0");
    assert_eq!(t::ftSb_Init_CostumeStrings[0][0], "PlSbNr.dat");
}
