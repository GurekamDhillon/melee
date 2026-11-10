//! tu::ftcaptainspeciallw vs the C (gen/turef/tu_ftcaptainspeciallw_ref.c: the game TU verbatim,
//! shipping contraction, recording stubs for its callees; data in tests/data/tu_ftcaptainspeciallw.txt,
//! regenerate with `python rust/melee_chara/gen/turef/tu_pre.py tu_ftcaptainspeciallw`).
//! Covers ftCa_SpecialHi_800E3EAC, ftCa_SpecialLw_Coll, ftCa_SpecialLw_Anim,
//! ftCa_SpecialLwEndAir_Phys, ftCa_SpecialLw_Phys, ftCa_SpecialAirLw_Phys.
//! One #[test]: the hooks are process globals.
#![allow(static_mut_refs)]

use melee_chara::stubs as st;
use melee_chara::tu::ftcaptainspeciallw as t;
use melee_types::*;
use std::fmt::Write;

static mut LOG: String = String::new();
fn lg(s: &str) {
    unsafe { LOG.push_str(s) }
}
static mut R_82708: i32 = 0;
static mut R_81D0C: i32 = 0;
static mut R_827A0: i32 = 0;
static mut R_FRAMES: i32 = 0;

struct Rng(u32);
impl Rng {
    fn rnd(&mut self) -> u32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 17;
        self.0 ^= self.0 << 5;
        self.0
    }
    fn rf(&mut self) -> f32 {
        match self.rnd() % 8 {
            0 => 0.0,
            1 => -0.0,
            2 => 1.0,
            3 => -1.0,
            _ => ((self.rnd() % 4001) as i32 - 2000) as f32 / 997.0,
        }
    }
}

#[test]
fn matches_c() {
    let want = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/tu_ftcaptainspeciallw.txt")).unwrap();
    unsafe {
        st::FTCOMMON_8007D7FC = Some(|_| lg("7D7FC;"));
        st::FTCOMMON_8007D5D4 = Some(|_| lg("7D5D4;"));
        st::FTCOMMON_8007DB24 = Some(|_| lg("7DB24;"));
        t::ext::FTCOMMON_8007E5AC = Some(|_| lg("7E5AC;"));
        st::FT_80085088 = Some(|_| lg("85088;"));
        st::FT_80085134 = Some(|_| lg("85134;"));
        st::FT_80084EEC = Some(|_| lg("84EEC;"));
        st::FT_80084F3C = Some(|_| lg("84F3C;"));
        st::FT_80082708 = Some(|_| {
            lg("82708;");
            R_82708
        });
        st::FT_800827A0 = Some(|_| {
            lg("827A0;");
            R_827A0
        });
        st::FT_80081D0C = Some(|_| {
            lg("81D0C;");
            R_81D0C
        });
        st::FT_8008A2BC = Some(|_| lg("8A2BC;"));
        st::FTCO_FALL_ENTER = Some(|_| lg("Fall;"));
        st::FTANIM_8006EBA4 = Some(|_| lg("6EBA4;"));
        st::FTANIM_ISFRAMESREMAINING = Some(|_| {
            lg("Frames;");
            R_FRAMES
        });
        t::ext::FTPARTSETROTZ = Some(|_, p, v| lg(&format!("rotz {} {:08x};", p, v.to_bits())));
        st::FIGHTER_CHANGEMOTIONSTATE = Some(|_, ms, fl, a, b, c, _| lg(&format!("ms {} {:x} {:08x} {:08x} {:08x};", ms, fl, a.to_bits(), b.to_bits(), c.to_bits())));
        t::ext::EFASYNC_SPAWN = Some(|_, _, k, gfx, j, a| lg(&format!("ef {} {:x} {} {:08x};", k, gfx, j as usize as u32, unsafe { (*a).to_bits() })));
    }

    let mut f: Box<Fighter> = Box::new(Fighter::zeroed());
    let mut g: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    let da: &'static mut ftCaptain_DatAttrs = Box::leak(Box::new(ftCaptain_DatAttrs::zeroed()));
    let mut parts: std::vec::Vec<FighterBone> = (0..80).map(|_| unsafe { core::mem::zeroed() }).collect();
    let mut p2j: std::vec::Vec<std::vec::Vec<u8>> = (0..64).map(|_| std::vec![0u8; 256]).collect();
    let mut tbls: std::vec::Vec<FighterPartsTable> = (0..64).map(|_| FighterPartsTable::zeroed()).collect();
    let mut tblp: std::vec::Vec<*mut FighterPartsTable> = std::vec::Vec::new();
    for k in 0..64 {
        tbls[k].part_to_joint = p2j[k].as_mut_ptr();
    }
    for k in 0..64 {
        tblp.push(&mut tbls[k]);
    }
    unsafe { melee_fighter::hooks::FT_PARTS_TABLE = tblp.as_mut_ptr() };
    g.user_data = &mut *f as *mut Fighter as *mut _;
    f.dat_attrs = da as *mut _ as *mut _;
    f.parts = parts.as_mut_ptr();
    let gp: *mut HSD_GObj = &mut *g;

    let mut r = Rng(0xCA17F0);
    let mut n = 0;
    for (i, line) in want.lines().enumerate() {
        for k in 0..80 {
            parts[k].joint = (0x1000 + k) as *mut _;
        }
        for k in 0..64 {
            for j in 0..256 {
                p2j[k][j] = (r.rnd() % 80) as u8;
            }
        }
        f.motion_id = match r.rnd() % 4 {
            0 => 0x165,
            1 => 0x167,
            2 => 0x166,
            _ => (r.rnd() % 0x200) as _,
        };
        f.kind = if r.rnd() & 1 != 0 {
            if r.rnd() & 1 != 0 { Ft_Kind_Captain as _ } else { Ft_Kind_Ganon as _ }
        } else {
            (r.rnd() % Ft_Kind_Mex0 as u32) as _
        };
        f.ground_or_air = if r.rnd() & 1 != 0 { GA_Ground as _ } else { GA_Air as _ };
        let tf = r.rnd();
        unsafe {
            f.x2210.throw_flags = tf as _;
            let b1 = r.rnd() & 1;
            f.x2210.x0.set_throw_flags_b1(b1 as _);
        }
        let b0 = r.rnd() & 1;
        f.set_x2219_b0(b0 as _);
        for c in 0..3 {
            f.cmd_vars[c] = if r.rnd() & 1 != 0 { 0 } else { r.rnd() % 3 } as _;
        }
        f.cmd_vars[3] = (r.rnd() % 3) as _;
        f.facing_dir = if r.rnd() & 1 != 0 { 1.0 } else { -1.0 };
        let mut env = r.rnd();
        if r.rnd() & 1 != 0 {
            env |= if r.rnd() & 1 != 0 { Collide_RightWallHug as u32 } else { Collide_LeftWallHug as u32 };
        }
        f.coll_data.env_flags = env as _;
        f.self_vel.x = r.rf();
        f.self_vel.y = r.rf();
        unsafe { f.mv.ca.speciallw.friction = r.rf() };
        f.pre_hitlag_cb = None;
        f.post_hitlag_cb = None;
        da.speciallw_flame_particle_angle = r.rf() * 90.0;
        da.speciallw_ground_lag_mul = r.rf();
        unsafe {
            R_82708 = (r.rnd() % 3) as i32;
            R_81D0C = (r.rnd() % 3) as i32;
            R_827A0 = (r.rnd() & 1) as i32;
            R_FRAMES = (r.rnd() & 1) as i32;
            LOG.clear();
        }
        let mut out = String::new();
        let name = match i % 5 {
            0 => {
                t::ftCa_SpecialHi_800E3EAC(gp);
                "3EAC"
            }
            1 => {
                t::ftCa_SpecialLw_Coll(gp);
                "LwColl"
            }
            2 => {
                t::ftCa_SpecialLw_Anim(gp);
                "LwAnim"
            }
            3 => {
                t::ftCa_SpecialLwEndAir_Phys(gp);
                "EndAirPhys"
            }
            _ => {
                if i & 5 != 0 {
                    t::ftCa_SpecialLw_Phys(gp);
                    "LwPhys"
                } else {
                    t::ftCa_SpecialAirLw_Phys(gp);
                    "AirLwPhys"
                }
            }
        };
        unsafe {
            write!(
                out,
                "{} {} |{} | ga {} cv {} {} {} {} thr0 {} b1 {} x2219 {} vel {:08x} {:08x} pre {} post {}",
                i,
                name,
                LOG,
                f.ground_or_air as i32,
                f.cmd_vars[0],
                f.cmd_vars[1],
                f.cmd_vars[2],
                f.cmd_vars[3],
                (f.x2210.throw_flags == 0) as i32,
                f.x2210.x0.throw_flags_b1(),
                f.x2219_b0(),
                f.self_vel.x.to_bits(),
                f.self_vel.y.to_bits(),
                f.pre_hitlag_cb.is_some() as i32,
                f.post_hitlag_cb.is_some() as i32
            )
            .unwrap();
        }
        assert_eq!(out, line, "case {i}");
        n += 1;
    }
    assert_eq!(n, 4000);
}
