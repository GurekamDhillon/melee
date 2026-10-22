//! Smoke test for tu::ftluigi: OnLoad (PUSH_ATTRS plus the PC zeroing of the Cyclone charge) and
//! LoadSpecialAttrs (COPY_ATTRS), and the demo file names.
#![allow(static_mut_refs)]

use melee_chara::tu::ftluigi as t;
use melee_types::*;

static mut LOG: std::vec::Vec<String> = std::vec::Vec::new();

#[test]
fn attrs_push_and_copy() {
    let mut f: Box<Fighter> = Box::new(Fighter::zeroed());
    let mut g: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    let mut fd: Box<ftData> = Box::new(unsafe { core::mem::zeroed() });
    let mut ext = Box::new(ftLuigiAttributes::zeroed());
    let mut backup = Box::new(ftLuigiAttributes::zeroed());
    let mut live = Box::new(ftLuigiAttributes::zeroed());
    let mut items: [*mut core::ffi::c_void; 1] = [0x10 as *mut _];
    ext.x64_LUIGI_SUPERJUMP_VEL_X = 2.25;
    fd.ext_attr = &mut *ext as *mut _ as *mut _;
    fd.x48_items = items.as_mut_ptr();
    f.ft_data = &mut *fd;
    f.dat_attrs_backup = &mut *backup as *mut _ as *mut _;
    f.dat_attrs = &mut *live as *mut _ as *mut _;
    f.u.lg.x222C_cycloneCharge = 5;
    g.user_data = &mut *f as *mut Fighter as *mut _;
    let gp = &mut *g as *mut HSD_GObj;
    unsafe { t::ext::IT_8026B3F8 = Some(|a, k| LOG.push(format!("{:x} {k}", a as usize))) };

    t::ftLg_Init_LoadSpecialAttrs(gp);
    assert_eq!(live.x64_LUIGI_SUPERJUMP_VEL_X, 2.25);

    t::ftLg_Init_OnLoad(gp);
    assert_eq!(f.dat_attrs, &mut *backup as *mut _ as *mut _);
    assert_eq!(backup.x64_LUIGI_SUPERJUMP_VEL_X, 2.25);
    assert_eq!(unsafe { f.u.lg.x222C_cycloneCharge }, 0);
    assert_eq!(unsafe { LOG.join(",") }, "10 105");

    assert_eq!(t::ftLg_Init_GetMotionFileString(9), "ftDemoVi0102MotionFileLuigi");
    assert_eq!(t::ftLg_Init_GetMotionFileString(10), "ftDemoVi1101MotionFileLuigi");
    assert_eq!(t::ftLg_Init_CostumeStrings[3][2], "PlyLuigi5KPi_Share_matanim_joint");
    let (mut a, mut b) = (0, 0);
    t::ftLg_Init_UnkDemoCallbacks0(9, &mut a, &mut b);
    assert_eq!((a, b), (14, 14));
}
