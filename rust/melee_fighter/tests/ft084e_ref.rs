//! ft084e.rs (group 3 round 12) vs the C of src/melee/ft/ft_084E.c + ft_80084CB0/DB0
//! (gen/cref/ft084e_ref.c, expected values in tests/data/ft084e.txt; regenerate with
//! `python rust/melee_fighter/gen/cref/run.py ft084e`). Fill order must match the C.
#![allow(static_mut_refs)]

use melee_fighter::ft084e as m;
use melee_fighter::hooks;
use melee_fighter::types::{ftCollisionBox, ftCommonData, Fighter, Fighter_804D6520_t, HSD_GObj};

static mut G_FRIC: f32 = 0.0;
static mut G_SFX: i32 = 0;
static mut G_WAIT: i32 = 0;

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

#[test]
fn ft084e_matches_c() {
    unsafe {
        hooks::GROUND_FRICTION_MUL = Some(|_| G_FRIC);
        hooks::PLAY_SFX = Some(|_, id, vol, pan| G_SFX += id + vol + pan);
        hooks::FT_8008A2BC = Some(|_| G_WAIT += 1);
    }
    let mut cd: Box<ftCommonData> = Box::new(ftCommonData::zeroed());
    let mut bh: Box<Fighter_804D6520_t> = Box::new(Fighter_804D6520_t::zeroed());
    let mut other: Box<HSD_GObj> = Box::new(HSD_GObj::zeroed());
    let expected = include_str!("data/ft084e.txt");
    let mut lines = expected.lines();
    let mut checked = 0;
    for op in 0..19u32 {
        for k in 0..400u32 {
            let mut fp: Box<Fighter> = Box::new(Fighter::zeroed());
            let mut g: Box<HSD_GObj> = Box::new(HSD_GObj::zeroed());
            g.user_data = &mut *fp as *mut Fighter as *mut core::ffi::c_void;
            *cd = ftCommonData::zeroed();
            *bh = Fighter_804D6520_t::zeroed();
            let mut bx = ftCollisionBox::zeroed();
            unsafe { G_SFX = 0; G_WAIT = 0 };
            let mut r = Rng(0x84E1Cu32.wrapping_add(op.wrapping_mul(7919)).wrapping_add(k.wrapping_mul(104729)));
            fp.gr_vel = r.rf();
            fp.xE4_ground_accel_1 = r.rf();
            fp.self_vel.x = r.rf();
            fp.self_vel.y = r.rf();
            fp.self_vel.z = r.rf();
            fp.x74_self_accel.x = r.rf();
            fp.x74_self_accel.y = r.rf();
            fp.x74_self_accel.z = r.rf();
            fp.input.lstick[0].x = r.rf();
            fp.input.lstick[0].y = r.rf();
            fp.coll_data.floor.normal.x = r.rf();
            fp.coll_data.floor.normal.y = r.rf();
            fp.co_attrs.gravity = r.rf();
            fp.co_attrs.terminal_velocity = r.rf();
            fp.co_attrs.fast_fall_velocity = r.rf();
            fp.co_attrs.aerial_friction = r.rf();
            fp.co_attrs.ground_friction = r.rf();
            fp.co_attrs.walk_max_vel = r.rf();
            fp.co_attrs.air_drift_max = r.rf();
            fp.co_attrs.air_drift_stick_mul = r.rf();
            fp.co_attrs.aerial_drift_base = r.rf();
            fp.co_attrs.air_max_horizontal_velocity = r.rf();
            fp.set_fall_fast((r.rnd() & 1) as u8);
            fp.active_timer.lstick.y = r.rnd() as u8;
            fp.facing_dir = r.rf();
            fp.facing_dir1 = r.rf();
            unsafe { fp.anon_594.anon_0.set_x594_b0((r.rnd() & 1) as u8) };
            fp.x6A4_transNOffset.y = r.rf();
            fp.x6A4_transNOffset.z = r.rf();
            fp.lstick_angle = r.rf();
            fp.x197C = if r.rnd() & 1 != 0 { &mut *other } else { core::ptr::null_mut() };
            fp.coll_data.ecb.top.y = r.rf();
            fp.coll_data.ecb.bottom.y = r.rf();
            fp.coll_data.ecb.right.x = r.rf();
            fp.coll_data.ecb.right.y = r.rf();
            fp.coll_data.ecb.left.x = r.rf();
            fp.coll_data.ecb.left.y = r.rf();
            cd.x88 = r.rf();
            cd.x8C = (r.rnd() % 8) as i32;
            cd.friction_when_above_walk_speed = r.rf();
            bh.x14 = r.rf();
            cd.walk_stick_threshold = r.rf();
            unsafe {
                G_FRIC = r.rf();
                hooks::P_FT_COMMON_DATA = &*cd;
                hooks::FIGHTER_804D6520 = &*bh;
            }
            let (a, b, c) = (r.rf(), r.rf(), r.rf());
            let gp: *mut HSD_GObj = &mut *g;
            let mut ret = 0i32;
            match op {
                0 => m::ft_80084e1c(gp, a, b, c),
                1 => m::ft_80084eec(gp),
                2 => m::ft_80084f3c(gp),
                3 => m::ft_80084fa8(gp),
                4 => m::ft_80085004(gp),
                5 => m::ft_80085030(gp, a, b),
                6 => m::ft_80085088(gp),
                7 => m::ft_800850b4(gp),
                8 => m::ft_800850e0(gp, a, b),
                9 => m::ft_80085134(gp),
                10 => m::ft_80085154(gp),
                11 => m::ft_800851c0(gp),
                12 => m::ft_800851d0(gp),
                13 => m::ft_80085204(gp),
                14 => m::ft_80084cb0(&fp, &mut bx),
                15 => m::ft_80084db0(gp),
                16 => ret = m::ft_8008a1fc(gp) as i32,
                17 => ret = m::ft_8008a244(gp) as i32,
                18 => ret = m::ft_8008a244_inline(gp) as i32,
                _ => unreachable!(),
            }
            let got = format!(
                "{} {} {:08X} {:08X} {:08X} {:08X} {:08X} {:08X} {:08X} {:08X} {} {} {} {:08X} {:08X} {:08X} {:08X} {:08X} {:08X} {} {}",
                op, k, fp.gr_vel.to_bits(), fp.xE4_ground_accel_1.to_bits(), fp.self_vel.x.to_bits(),
                fp.self_vel.y.to_bits(), fp.self_vel.z.to_bits(), fp.x74_self_accel.x.to_bits(),
                fp.x74_self_accel.y.to_bits(), fp.x74_self_accel.z.to_bits(), fp.fall_fast(),
                fp.active_timer.lstick.y, unsafe { G_SFX }, bx.top.to_bits(), bx.bottom.to_bits(),
                bx.left.x.to_bits(), bx.left.y.to_bits(), bx.right.x.to_bits(), bx.right.y.to_bits(), ret, unsafe { G_WAIT }
            );
            assert_eq!(got, lines.next().expect("ran out of C reference lines"), "op {op} case {k}");
            checked += 1;
        }
    }
    assert_eq!(checked, 7600);
}
