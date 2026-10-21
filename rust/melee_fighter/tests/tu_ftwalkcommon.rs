//! src/tu/ftwalkcommon.rs vs the C of src/melee/ft/ftwalkcommon.c (gen/cref/tu_ftwalkcommon_ref.c,
//! expected log in tests/data/tu_ftwalkcommon.txt; regenerate with
//! `python rust/melee_fighter/gen/cref/tu_run_s57.py tu_ftwalkcommon`). Fill order must match the C.
#![allow(static_mut_refs)]

use melee_fighter::hooks;
use melee_fighter::tu::ftwalkcommon::{self as m, ext};
use melee_fighter::types::{ftCommonData, Fighter, HSD_GObj};

static mut LOG: String = String::new();
static mut G_FRIC: f32 = 0.0;
static mut G_F484: f32 = 0.0;

fn log(s: String) {
    unsafe {
        LOG.push_str(&s);
        LOG.push('\n');
    }
}

struct Rng(u32);
impl Rng {
    fn rnd(&mut self) -> u32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 17;
        self.0 ^= self.0 << 5;
        self.0
    }
    fn rf(&mut self) -> f32 {
        let k = self.rnd() % 16;
        if k == 0 {
            return 0.0;
        }
        if k == 1 {
            return -0.0;
        }
        ((self.rnd() % 2001) as i32 - 1000) as f32 / 7.0
    }
}

pub fn fnv<T>(v: &T) -> u32 {
    let b = unsafe { core::slice::from_raw_parts(v as *const T as *const u8, core::mem::size_of::<T>()) };
    let mut h = 2166136261u32;
    for &x in b {
        h ^= x as u32;
        h = h.wrapping_mul(16777619);
    }
    h
}

#[test]
fn ftwalkcommon_matches_c() {
    unsafe {
        hooks::GROUND_FRICTION_MUL = Some(|_| G_FRIC);
        hooks::FTANIM_SET_ANIM_RATE = Some(|_, r| log(format!("RATE {:08X}", r.to_bits())));
        ext::FIGHTER_CHANGE_MOTION_STATE = Some(|_, msid, fl, a, r, b, o| {
            log(format!("CMS {} {} {:08X} {:08X} {:08X} {}", msid, fl, a.to_bits(), r.to_bits(), b.to_bits(), !o.is_null() as i32))
        });
        ext::FTANIM_8006EBA4 = Some(|_| log("EBA4".into()));
        ext::FTANIM_8006F484 = Some(|_| G_F484);
    }
    let mut cd: Box<ftCommonData> = Box::new(ftCommonData::zeroed());
    unsafe { hooks::P_FT_COMMON_DATA = &*cd };
    for op in 0..6u32 {
        for k in 0..500u32 {
            let mut fp: Box<Fighter> = Box::new(Fighter::zeroed());
            let mut g: Box<HSD_GObj> = Box::new(HSD_GObj::zeroed());
            g.user_data = &mut *fp as *mut Fighter as *mut core::ffi::c_void;
            let gp: *mut HSD_GObj = &mut *g;
            *cd = ftCommonData::zeroed();
            let mut r = Rng(0x0DFBF8u32.wrapping_add(op.wrapping_mul(7919)).wrapping_add(k.wrapping_mul(104729)));
            fp.gr_vel = r.rf();
            fp.xE4_ground_accel_1 = r.rf();
            fp.input.lstick[0].x = r.rf();
            fp.facing_dir = r.rf();
            fp.coll_data.floor.normal.x = r.rf();
            fp.coll_data.floor.normal.y = r.rf();
            fp.co_attrs.walk_max_vel = r.rf();
            fp.co_attrs.walk_accel_mul = r.rf();
            fp.co_attrs.walk_accel_base = r.rf();
            fp.co_attrs.ground_friction = r.rf();
            fp.cur_anim_frame = r.rf();
            unsafe {
                let w = &mut fp.mv.co.walk;
                w.x0 = r.rf();
                w.msid = (r.rnd() % 64) as _;
                let mid = (w.msid as i32 + (r.rnd() % 5) as i32 - 1) as _;
                fp.motion_id = mid;
                let w = &mut fp.mv.co.walk;
                w.slow_anim_frame = r.rf();
                w.middle_anim_frame = r.rf();
                w.fast_anim_frame = r.rf();
                w.slow_anim_rate = r.rf();
                w.middle_anim_rate = r.rf();
                w.fast_anim_rate = r.rf();
                w.accel_mul = r.rf();
            }
            cd.walk_stick_threshold = r.rf();
            cd.walk_middle_animation_stick_threshold = r.rf();
            cd.walk_fast_stick_threshold = r.rf();
            cd.walk_accel_taper_gain = r.rf();
            cd.x440 = r.rf();
            unsafe {
                G_FRIC = if r.rnd() & 1 != 0 { r.rf() } else { 1.0 };
                G_F484 = r.rf();
            }
            let a: Vec<f32> = (0..11).map(|_| r.rf()).collect();
            log(format!("OP {op} {k}"));
            let mut ret = 0i32;
            match op {
                0 => ret = m::ftwalkcommon_get_walk_type(gp) as i32,
                1 => ret = m::ftwalkcommon_800dfc70(gp) as i32,
                2 => {
                    let msid = (r.rnd() % 64) as _;
                    let fl = r.rnd() as _;
                    m::ftwalkcommon_800dfca4(gp, msid, fl, a[0], a[1], a[2], a[3], a[4], a[5], a[6], a[7]);
                }
                3 => m::ftwalkcommon_800dfddc(gp),
                4 => {
                    let wt = m::ftwalkcommon_get_walk_type(gp);
                    if fp.motion_id as i32 == unsafe { fp.mv.co.walk.msid } as i32 + wt as i32 {
                        fp.motion_id = (fp.motion_id as i32 + 1) as _;
                    }
                    m::ftwalkcommon_800dfec8(gp, |_, f| log(format!("CB {:08X}", f.to_bits())));
                }
                _ => m::ftwalkcommon_800e0060(gp),
            }
            log(format!("R {} {:08X}", ret, fnv(&*fp)));
        }
    }
    let got = unsafe { LOG.clone() };
    let expected = include_str!("data/tu_ftwalkcommon.txt");
    let (mut n, mut el) = (0, expected.lines());
    for gl in got.lines() {
        let e = el.next().expect("C log shorter");
        assert_eq!(gl, e, "line {n}");
        n += 1;
    }
    assert!(el.next().is_none(), "C log longer");
    assert_eq!(n, 8000);
}

