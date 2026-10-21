//! src/tu/ftwalljump.rs vs the C of src/melee/ft/ftwalljump.c (gen/cref/tu_ftwalljump_ref.c,
//! expected log in tests/data/tu_ftwalljump.txt; regenerate with
//! `python rust/melee_fighter/gen/cref/tu_run_s57.py tu_ftwalljump`). Fill order matches the C.
#![allow(static_mut_refs)]

use melee_fighter::hooks;
use melee_fighter::tu::ftwalljump::{self as m, ext};
use melee_fighter::types::*;

static mut LOG: String = String::new();
static mut G_SPEED: f32 = 0.0;
static mut G_OK: bool = false;

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
    fn side(&mut self) -> f32 {
        match self.rnd() % 4 {
            0 => -1.0,
            1 => 1.0,
            _ => self.rf(),
        }
    }
}

fn fnv<T>(v: &T) -> u32 {
    let b = unsafe { core::slice::from_raw_parts(v as *const T as *const u8, core::mem::size_of::<T>()) };
    b.iter().fold(2166136261u32, |h, &x| (h ^ x as u32).wrapping_mul(16777619))
}

#[test]
fn ftwalljump_matches_c() {
    unsafe {
        ext::MP_GET_SPEED = Some(|line, pos, speed| {
            log(format!("SPD {} {:08X} {:08X} {:08X}", line, pos.x.to_bits(), pos.y.to_bits(), pos.z.to_bits()));
            speed.x = G_SPEED;
            G_OK
        });
        ext::FTCO_800C1E64 = Some(|_, msid, t, e, d| log(format!("PW {} {} {} {:08X}", msid, t, e, d.to_bits())));
    }
    let mut cd: Box<ftCommonData> = Box::new(ftCommonData::zeroed());
    unsafe { hooks::P_FT_COMMON_DATA = &*cd };
    for k in 0..6000u32 {
        let mut fp: Box<Fighter> = Box::new(Fighter::zeroed());
        let mut g: Box<HSD_GObj> = Box::new(HSD_GObj::zeroed());
        g.user_data = &mut *fp as *mut Fighter as *mut core::ffi::c_void;
        *cd = ftCommonData::zeroed();
        let mut r = Rng(0x8169Cu32.wrapping_add(k.wrapping_mul(104729)));
        fp.set_can_walljump((r.rnd() % 8 != 0) as _);
        fp.coll_data.env_flags = (r.rnd() & (Collide_RightWallHug | Collide_LeftWallHug | 0x10001) as u32) as i32;
        fp.wall_jump_input_timer = if r.rnd() & 1 != 0 { r.rnd() % 8 } else { 250 + r.rnd() % 6 } as u8;
        fp.x2110_walljumpWallSide = r.side();
        fp.coll_data.ecb.left.x = r.rf();
        fp.coll_data.ecb.left.y = r.rf();
        fp.coll_data.ecb.right.x = r.rf();
        fp.coll_data.ecb.right.y = r.rf();
        fp.cur_pos.x = r.rf();
        fp.cur_pos.y = r.rf();
        fp.cur_pos.z = r.rf();
        fp.coll_data.right_facing_wall.index = ((r.rnd() % 100) as i32 - 1) as _;
        fp.coll_data.left_facing_wall.index = ((r.rnd() % 100) as i32 - 1) as _;
        fp.pos_delta.x = r.rf();
        fp.co_attrs.wall_jump_min_approach_speed = r.rf() / 50.0;
        fp.input.lstick[0].x = r.rf() / 100.0;
        fp.active_timer.lstick.x = (r.rnd() % 12) as u8;
        fp.x1969_walljumpUsed = if r.rnd() & 1 != 0 { 255 } else { (r.rnd() % 4) as u8 };
        cd.x768 = (r.rnd() % 20) as f32;
        cd.x76C = r.rf() / 200.0;
        cd.x770 = (r.rnd() % 12) as f32;
        cd.x774 = (r.rnd() % 30) as i32;
        unsafe {
            G_SPEED = r.rf();
            G_OK = r.rnd() & 1 != 0;
        }
        log(format!("K {k}"));
        let ret = m::ftwalljump_8008169c(&mut *g);
        // bitfields are in GC bit order here, native order in C: print them, hash the rest
        let cw = fp.can_walljump();
        fp.set_can_walljump(0);
        log(format!("R {} {} {:08X}", ret as i32, cw, fnv(&*fp)));
    }
    let got = unsafe { LOG.clone() };
    let mut el = include_str!("data/tu_ftwalljump.txt").lines();
    let mut n = 0;
    for gl in got.lines() {
        assert_eq!(gl, el.next().expect("C log shorter"), "line {n}");
        n += 1;
    }
    assert!(el.next().is_none(), "C log longer");
}
