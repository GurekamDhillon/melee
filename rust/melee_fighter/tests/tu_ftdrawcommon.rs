//! Smoke test for src/tu/ftdrawcommon.rs: draw flags, view offset matrix, render-cb install.
#![allow(static_mut_refs)]
use melee_fighter::tu::ftdrawcommon as d;
use melee_fighter::types::*;

static mut HEAD: *mut HSD_GObj = core::ptr::null_mut();
static mut CAM: *mut HSD_CObj = core::ptr::null_mut();

#[test]
fn flags_matrix_callbacks() {
    unsafe {
        d::ext::WB_RENDER_ACTIVE = Some(|| 0);
        d::ext::FTLIB_GET_SHAKE_OFFSET = Some(|_, v| { (*v).x = 1.5; (*v).y = -0.0; (*v).z = 3.0; 1 });
        d::ext::HSD_COBJ_GET_CURRENT = Some(|| CAM);
        d::ext::PLINK_FIGHTER_HEAD = Some(|| HEAD);
        d::ext::MEX_ENABLED_DISABLE_FOD = Some(|| 0);
    }
    let mut f: Box<Fighter> = Box::new(Fighter::zeroed());
    f.x21FC_flag.byte = 0x81; // b0 (MSB) and b7
    assert!(d::draw_flag(&f, 0));
    assert!(d::draw_flag(&f, 7));
    assert!(!d::draw_flag(&f, 3));
    unsafe { d::ext::WB_RENDER_ACTIVE = Some(|| 1); d::ext::WB_RENDER_OPTION = Some(|_, fb| fb) };
    assert!(!d::draw_flag(&f, 0));
    assert!(d::draw_flag(&f, 7));
    unsafe { d::ext::WB_RENDER_ACTIVE = Some(|| 0) };

    let mut cam: Box<HSD_CObj> = Box::new(HSD_CObj::zeroed());
    cam.view_mtx = [[1.0, 0.0, 0.0, 10.0], [0.0, 2.0, 0.0, 0.0], [0.0, 0.0, 1.0, -1.0]];
    unsafe { CAM = &mut *cam };
    let mut out = [[0.0f32; 4]; 3];
    let r = d::ftDrawCommon_8008051C(1 as *mut HSD_GObj, &mut out);
    assert!(r.is_some());
    assert_eq!(out[0][3], 11.5);
    assert_eq!(out[1][3].to_bits(), 0.0f32.to_bits()); // -0 + 0 = +0, times 2
    assert_eq!(out[2][3], 2.0);
    unsafe { d::ext::FTLIB_GET_SHAKE_OFFSET = Some(|_, _| 0) };
    assert!(d::ftDrawCommon_8008051C(1 as *mut HSD_GObj, &mut out).is_none());

    let mut g1: Box<HSD_GObj> = Box::new(HSD_GObj::zeroed());
    let mut g0: Box<HSD_GObj> = Box::new(HSD_GObj::zeroed());
    g0.next = &mut *g1;
    unsafe { HEAD = &mut *g0 };
    d::ftDrawCommon_80081118();
    assert!(g0.render_cb.is_some() && g1.render_cb.is_some());
    let a = g0.render_cb.map(|f| f as usize);
    d::ftDrawCommon_80081140();
    assert!(g1.render_cb.is_some() && g0.render_cb.map(|f| f as usize) != a);
    unsafe { d::ext::MEX_ENABLED_DISABLE_FOD = Some(|| 1); d::ext::FTLIB_COUNT_FIGHTERS = Some(|| 4) };
    d::ftDrawCommon_80081140();
    assert!(g0.render_cb.is_none() && g1.render_cb.is_none());
}
