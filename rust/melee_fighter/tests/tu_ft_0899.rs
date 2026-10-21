//! src/tu/ft_0899.rs vs the C of src/melee/ft/ft_0899.c (gen/cref/tu_ft_0899_ref.c, expected
//! log in tests/data/tu_ft_0899.txt; regenerate with
//! `python rust/melee_fighter/gen/cref/tu_run.py tu_ft_0899`). The stubs draw from the same
//! xorshift stream at call time, so the call sequence must match too.
#![allow(static_mut_refs)]

use core::fmt::Write;
use melee_fighter::hooks;
use melee_fighter::tu::ft_0899::{self as m, ext};
use melee_fighter::types::*;

static mut RS: u32 = 0;
static mut LOG: String = String::new();

fn rnd() -> u32 {
    unsafe {
        RS ^= RS << 13;
        RS ^= RS >> 17;
        RS ^= RS << 5;
        RS
    }
}
fn rf() -> f32 {
    let k = rnd() % 16;
    if k == 0 {
        return 0.0;
    }
    if k == 1 {
        return -0.0;
    }
    ((rnd() % 2001) as i32 - 1000) as f32 / 7.0
}
fn rv(v: &mut Vec3) {
    v.x = rf();
    v.y = rf();
    v.z = rf();
}
fn b(f: f32) -> u32 {
    f.to_bits()
}
macro_rules! log { ($($t:tt)*) => { unsafe { writeln!(LOG, $($t)*).unwrap() } } }

fn install() {
    unsafe {
        ext::MPLIB_8004DD90_FLOOR = Some(|line, pos, y, fl, n| {
            log!("FLOOR {} {:08X} {:08X}", line, b(pos.x), b(pos.y));
            if rnd() % 3 == 0 {
                return -1;
            }
            *y = rf();
            *fl = rnd();
            rv(n);
            0
        });
        ext::MP_FLOOR_GET_LEFT = Some(|l, v| { log!("LEFT {}", l); rv(v) });
        ext::MP_FLOOR_GET_RIGHT = Some(|l, v| { log!("RIGHT {}", l); rv(v) });
        fn vpos(v: &mut Vec3) {
            rv(v);
            if rnd() % 2 != 0 {
                v.x /= 64.0;
                v.y /= 64.0;
            }
        }
        ext::MP_LINE_GET_V1_POS = Some(|l, v| { log!("V1 {}", l); vpos(v) });
        ext::MP_LINE_GET_V0_POS = Some(|l, v| { log!("V0 {}", l); vpos(v) });
        hooks::MP_LINE_GET_NEXT = Some(|l| { log!("NEXT {}", l); if rnd() % 3 == 0 { -1 } else { l + 1 } });
        hooks::MP_LINE_GET_PREV = Some(|l| { log!("PREV {}", l); if rnd() % 3 == 0 { -1 } else { l - 1 } });
        ext::MP_LINE_GET_KIND = Some(|l| { log!("KIND {}", l); (rnd() % 4) as i32 });
        ext::MP_LINE_GET_NORMAL = Some(|l, v| { log!("NORM {}", l); rv(v) });
        hooks::LB_8000B1CC = Some(|j, _off, out| *out = (*j).translate);
        ext::LBBGFLASH_80021410 = Some(|ik| {
            log!("IK {:08X} {:08X} {:08X} {:08X} {:08X} {:08X}", b(ik.pos3.x), b(ik.pos3.y), b(ik.pos3.z),
                 b(ik.pos4.x), b(ik.pos4.y), b(ik.pos4.z));
            (*ik.jobj0).rotate.x = rf();
            (*ik.jobj1).rotate.y = rf();
        });
        ext::LBBGFLASH_80020E38 = Some(|j, v, a, bb, c| {
            log!("E38 {:08X} {:08X} {:08X} {:08X} {:08X} {:08X}", b(v.x), b(v.y), b(v.z), b(a), b(bb), b(c));
            (*j).rotate.z = rf();
        });
        ext::FT_PART_SET_ROT_X = Some(|_, p, x| log!("ROTX {} {:08X}", p, b(x)));
    }
}

#[test]
fn ft_0899_matches_c() {
    install();
    let mut cd: Box<ftCommonData> = Box::new(ftCommonData::zeroed());
    static mut DB: i32 = 0;
    unsafe {
        hooks::P_FT_COMMON_DATA = &*cd;
        ext::DB_804D4AF8 = Some(|| DB);
    }
    let mut fd: Box<ftData> = Box::new(ftData::zeroed());
    let mut x58: Box<ftData_x58_t> = Box::new(ftData_x58_t::zeroed());
    let mut parts: std::vec::Vec<FighterBone> = (0..6).map(|_| FighterBone::zeroed()).collect();
    let mut joints: std::vec::Vec<HSD_JObj> = (0..6).map(|_| HSD_JObj::zeroed()).collect();
    for k in 0..3000u32 {
        let mut fp: Box<Fighter> = Box::new(Fighter::zeroed());
        let mut root: Box<HSD_JObj> = Box::new(HSD_JObj::zeroed());
        let mut g: Box<HSD_GObj> = Box::new(HSD_GObj::zeroed());
        for j in joints.iter_mut() {
            *j = HSD_JObj::zeroed();
        }
        g.user_data = &mut *fp as *mut Fighter as *mut core::ffi::c_void;
        g.hsd_obj = &mut *root as *mut HSD_JObj as *mut core::ffi::c_void;
        fp.gobj = &mut *g;
        fp.ft_data = &mut *fd;
        fd.x58 = &mut *x58;
        fp.parts = parts.as_mut_ptr();
        unsafe { RS = 0x0899u32.wrapping_add(k.wrapping_mul(104729)) };
        x58.x0 = 0; x58.x8 = 1; x58.x10 = 2; x58.x1 = 3; x58.x9 = 4; x58.x11 = 5;
        x58.x4 = rf(); x58.xC = rf(); x58.x18 = rf();
        for i in 0..6 {
            parts[i].joint = &mut joints[i];
            rv(&mut joints[i].translate);
            joints[i].rotate.x = rf(); joints[i].rotate.y = rf(); joints[i].rotate.z = rf(); joints[i].rotate.w = rf();
        }
        rv(&mut root.translate);
        fp.x34_scale.y = rf();
        fp.facing_dir = if rnd() % 2 != 0 { 1.0 } else { -1.0 };
        fp.coll_data.floor.index = (rnd() % 50) as i32;
        fp.coll_data.floor.normal.x = rf();
        fp.coll_data.floor.normal.y = rf();
        cd.x804 = rf();
        fp.set_x2219_b5((rnd() % 5 == 0) as u8);
        fp.ground_or_air = if rnd() % 5 == 0 { GA_Air } else { GA_Ground };
        unsafe { DB = (rnd() % 2) as i32 };
        let gp: *mut HSD_GObj = &mut *g;
        if rnd() % 8 == 0 {
            let fl = rnd() % 8;
            m::ft_8008A1B8(gp, fl);
        } else {
            let v = (rnd() % 8) as u16;
            fp.set_x221C_u16_y(v);
            m::ft_80089B08(gp);
        }
        let mut s = format!("END {}", fp.x221C_u16_y());
        for j in joints.iter() {
            write!(s, " {:08X} {:08X} {:08X}", b(j.rotate.x), b(j.rotate.y), b(j.rotate.z)).unwrap();
        }
        log!("{}", s);
    }
    let expected = include_str!("data/tu_ft_0899.txt");
    let got = unsafe { LOG.clone() };
    for (i, (e, g)) in expected.lines().zip(got.lines()).enumerate() {
        assert_eq!(e, g, "line {}", i + 1);
    }
    assert_eq!(expected.lines().count(), got.lines().count());
}
