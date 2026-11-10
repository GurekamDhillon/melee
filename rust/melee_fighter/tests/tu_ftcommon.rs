//! src/tu/ftcommon.rs: ftCommon_8007E5AC vs the C (gen/cref/tu/ftcommon.c, shipping
//! contraction, MSL atan2f), 20000 cases; smoke test of the transform hand-over
//! ftCommon_8007EFC8 with its out-of-TU callees recorded.
#![allow(static_mut_refs)]
mod tu_s25;
use melee_fighter::hooks;
use melee_fighter::tu::ftcommon as c;
use melee_fighter::types::*;
use tu_s25::*;

static mut FP: usize = 0;
fn rotz(f: &Fighter, part: i32, v: f32) {
    t(1);
    t((f as *const Fighter as usize == unsafe { FP }) as u32);
    t(part as u32);
    t(v.to_bits());
}

#[test]
fn ftcommon_8007e5ac_vs_c() {
    let exp = expected("tu_ftcommon");
    let cd: Box<ftCommonData> = zeroed();
    unsafe {
        hooks::P_FT_COMMON_DATA = &*cd;
        c::ext::FTPART_SET_ROT_Z = Some(rotz);
    }
    for k in 0..20000u32 {
        let mut h = H::case(0, k);
        unsafe { TR = 0 };
        let mut f: Box<Fighter> = zeroed();
        unsafe { FP = &*f as *const Fighter as usize };
        f.ground_or_air = GA_Ground as _;
        f.coll_data.floor.normal.x = if h.rnd() & 1 != 0 { h.rw() } else { h.rg() };
        f.coll_data.floor.normal.y = if h.rnd() & 1 != 0 { h.rw() } else { h.rg() };
        c::ftcommon_8007e5ac(&mut f);
        assert_eq!(format!("{} {:08X}", k, unsafe { TR }), exp[k as usize], "case {k}");
    }
    assert_eq!(exp.len(), 20000);
}

static mut DST_G: usize = 0;
static mut LOG: std::vec::Vec<&'static str> = std::vec::Vec::new();
fn log(s: &'static str) { unsafe { LOG.push(s) } }

#[test]
fn ftcommon_8007efc8_smoke() {
    let mut cd: Box<ftCommonData> = zeroed();
    cd.x5F0 = 0;
    let mut src: Box<Fighter> = zeroed();
    let mut dst: Box<Fighter> = zeroed();
    let mut sg: Box<HSD_GObj> = zeroed();
    let mut dg: Box<HSD_GObj> = zeroed();
    let mut cb: Box<CmSubject> = zeroed();
    sg.user_data = &mut *src as *mut Fighter as *mut _;
    dg.user_data = &mut *dst as *mut Fighter as *mut _;
    src.gobj = &mut *sg;
    dst.gobj = &mut *dg;
    src.x890_cameraBox = &mut *cb;
    let mut cb2: Box<CmSubject> = zeroed();
    dst.x890_cameraBox = &mut *cb2;
    let mut fd: Box<ftData> = zeroed();
    let mut at: Box<ftCo_DatAttrs> = zeroed();
    let mut cam: Box<UnkFloat6_Camera> = zeroed();
    let mut x8: Box<ftData_x8> = zeroed();
    let mut pick: Box<[u8; 0x400]> = zeroed();
    let mut x50: Box<[u8; 0x400]> = zeroed();
    fd.x0 = &mut *at;
    fd.x3C = &mut *cam;
    fd.x8 = &mut *x8;
    fd.x40 = pick.as_mut_ptr() as *mut _;
    fd.x50 = x50.as_mut_ptr() as *mut _;
    dst.ft_data = &mut *fd;
    dst.x34_scale.y = 1.0;
    src.player_idx = 2;
    dst.player_idx = 2;
    src.set_is_sub_fighter(0);
    dst.set_is_sub_fighter(1);
    src.cur_pos = Vec3 { x: 12.5, y: -3.0, z: 0.0 };
    src.facing_dir = -1.0;
    src.dmg.x1830_percent = 47.0;
    src.gr_vel = 1.25;
    src.x68A = 9;
    src.shield_health = 33.0;
    src.x1968_jumpsUsed = 1;
    src.x34_scale.y = 1.0;
    src.set_x2221_b5(1);
    src.x1990 = 0;
    unsafe {
        DST_G = &mut *dg as *mut HSD_GObj as usize;
        hooks::P_FT_COMMON_DATA = &*cd;
        c::ext::PLAYER_GET_ENTITY_AT_INDEX = Some(|_, _| { log("get"); DST_G as *mut HSD_GObj });
        c::ext::PLAYER_SWAP_TRANSFORMED_STATES = Some(|_, _, _| log("swap"));
        c::ext::FIGHTER_UNK_INIT_RESET_80067C98 = Some(|_| log("reset"));
        c::ext::PLAYER_SET_HP_BY_INDEX = Some(|slot, sub, hp| { assert_eq!((slot, sub, hp), (2, 0, 47)); log("hp") });
        c::ext::FTLIB_SET_SCALE = Some(|_, _| log("scale"));
        c::ext::FT_800849EC = Some(|_, _| log("849ec"));
        c::ext::FT_80081C88 = Some(|_, _| log("81c88"));
        c::ext::UN_80322314 = Some(|| log("un"));
        melee_fighter::tu::ftcolanim::ext::FIGHTER_CHANGE_MOTION_STATE = Some(|_, ms, _, _, _, _, _| { assert_eq!(ms, ftCo_MS_Sleep as i32); log("sleep") });
        let mut tbl: Box<[Fighter_804D653C_t; 0x7B]> = zeroed();
        melee_fighter::tu::ftcolanim::ext::Fighter_804D653C = tbl.as_mut_ptr();
        core::mem::forget(tbl);
        melee_fighter::tu::ftcolanim::ext::LB_80014498 = Some(|_| log("colreset"));
        melee_fighter::tu::ftchangeparam::ext::Geno_ApplyAttrs = Some(|_| log("geno"));
        melee_fighter::tu::ftcamera::ext::Stage_GetCamFixedZoom = Some(|| 1.0);
        melee_fighter::tu::ftchangeparam::ext::ftKindCalcIndiviParamTable_present = Some(|_| true);
        melee_fighter::tu::ftchangeparam::ext::Mex_OnReapplyAttrDispatch = Some(|_, _| log("reapply"));
    }
    c::ftcommon_8007efc8(&mut *sg, |_| log("arg1"));
    let lg = unsafe { LOG.clone() };
    assert_eq!(&lg[..4], &["get", "swap", "reset", "hp"]);
    assert!(lg.contains(&"colreset"));
    assert!(lg.contains(&"scale"));
    assert_eq!(dst.cur_pos.x, 12.5);
    assert_eq!(dst.facing_dir, -1.0);
    assert_eq!(dst.gr_vel, 1.25);
    assert_eq!(dst.x68A, 9);
    assert_eq!(dst.shield_health, 33.0);
    assert_eq!(dst.x1968_jumpsUsed, 1);
    assert_eq!(dst.x2221_b5(), 1);
    assert_eq!(dst.is_sub_fighter(), 0);
    assert_eq!(src.is_sub_fighter(), 1);
    assert_eq!(src.x198C, 2);
    assert_eq!(&lg[lg.len() - 4..], &["81c88", "un", "sleep", "arg1"]);
    assert!(lg.contains(&"geno"));
    assert_eq!(unsafe { (*dst.x890_cameraBox).pos.x }, 12.5);
}
