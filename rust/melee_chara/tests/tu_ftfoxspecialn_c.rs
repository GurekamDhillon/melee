//! tu::ftfoxspecialn vs the C (gen/turef/tu_ftfoxspecialn_ref.c: the game TU verbatim, shipping
//! contraction, recording stubs for its callees; data in tests/data/tu_ftfoxspecialn.txt, regenerate
//! with `python rust/melee_chara/gen/turef/tu_pre.py tu_ftfoxspecialn`).
//! Covers ftFx_SpecialN_CreateBlasterShot, ftFx_Special{N,AirN}Start_Anim, ftFx_Special{N,AirN}Loop_Anim and
//! ftFx_Special{N,AirN}{Start,Loop}_IASA. One #[test]: the hooks are process globals.
#![allow(static_mut_refs)]

use melee_chara::stubs as st;
use melee_chara::tu::ftfoxspecialn as t;
use melee_types::*;
use std::fmt::Write;

static mut OUT: String = String::new();
fn lg(s: &str) {
    unsafe { OUT.push_str(s) }
}
static mut R_FRAMES: i32 = 0;
static mut R_KIND: i32 = 0;

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
    let want = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/tu_ftfoxspecialn.txt")).unwrap();
    unsafe {
        t::ext::FT_800892A0 = Some(|_| lg("892A0;"));
        t::ext::FT_80089824 = Some(|_| lg("89824;"));
        t::ext::FTFX_INIT_800E5534 = Some(|_| {
            lg("E5534;");
            false
        });
        t::ext::FTFX_INIT_800E5588 = Some(|_| {});
        st::FT_8008A2BC = Some(|_| lg("wait;"));
        st::FTCO_FALL_ENTER = Some(|_| lg("Fall;"));
        st::FTCO_80096900 = Some(|_, a, b, c, d, e| lg(&format!("fallspecial {} {} {} {:08x} {:08x};", a, b, c, d.to_bits(), e.to_bits())));
        t::ext::FTPICKUPITEM_80094818 = Some(|_, b| lg(&format!("pickup {};", b as i32)));
        st::FTCOMMON_8007D7FC = Some(|_| lg("7D7FC;"));
        st::FT_80084F3C = Some(|_| lg("84F3C;"));
        st::FT_80084EEC = Some(|_| lg("84EEC;"));
        st::FT_80085134 = Some(|_| lg("85134;"));
        st::FT_80085088 = Some(|_| lg("85088;"));
        st::FTANIM_8006EBA4 = Some(|_| lg("6EBA4;"));
        st::FTANIM_ISFRAMESREMAINING = Some(|_| {
            lg("Frames;");
            R_FRAMES
        });
        st::FIGHTER_CHANGEMOTIONSTATE = Some(|_, ms, fl, a, b, c, _| lg(&format!("ms {} {:x} {:08x} {:08x} {:08x};", ms, fl, a.to_bits(), b.to_bits(), c.to_bits())));
        t::ext::FTPARTS_GETBONEINDEX = Some(|_, p| p ^ 5);
        t::ext::FTLIB_GETKIND = Some(|_| R_KIND as _);
        t::ext::FT_PLAYSFX = Some(|_, id, v, p| lg(&format!("sfx {} {} {};", id, v, p)));
        t::ext::LB_8000B1CC = Some(|j, off, out| {
            lg(&format!("hold {} {:08x} {:08x} {:08x};", j as usize as u32, (*off).x.to_bits(), (*off).y.to_bits(), (*off).z.to_bits()));
            (*out).x = (*off).z;
            (*out).y = (*off).y;
            (*out).z = 9.0;
        });
        t::ext::IT_8029C6A4 = Some(|a, v, _, p, k| {
            lg(&format!("shot {:08x} {:08x} {:08x} {:08x} {:08x} {};", a.to_bits(), v.to_bits(), (*p).x.to_bits(), (*p).y.to_bits(), (*p).z.to_bits(), k))
        });
        t::ext::IT_802ADDD0 = Some(|it, v| lg(&format!("vis {} {};", it as usize as u32, v)));
        t::ext::IT_802AE538 = Some(|_| lg("538;"));
        t::ext::IT_802AE608 = Some(|_| lg("608;"));
        t::ext::IT_802AE1D0 = Some(|_| lg("1D0;"));
        t::ext::IT_802AEAB4 = Some(|_| lg("remove;"));
    }

    let mut f: Box<Fighter> = Box::new(Fighter::zeroed());
    let mut g: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    let da: &'static mut ftFox_DatAttrs = Box::leak(Box::new(ftFox_DatAttrs::zeroed()));
    let mut parts: std::vec::Vec<FighterBone> = (0..80).map(|_| unsafe { core::mem::zeroed() }).collect();
    g.user_data = &mut *f as *mut Fighter as *mut _;
    f.dat_attrs = da as *mut _ as *mut _;
    f.parts = parts.as_mut_ptr();
    let gp: *mut HSD_GObj = &mut *g;

    let mut r = Rng(0xF0C5A1);
    let mut n = 0;
    for (i, line) in want.lines().enumerate() {
        for k in 0..80 {
            parts[k].joint = (0x1000 + k) as *mut _;
        }
        for k in 0..4 {
            f.cmd_vars[k] = if r.rnd() & 1 != 0 { 0 } else { r.rnd() % 3 } as _;
        }
        unsafe {
            f.mv.fx.SpecialN.isBlasterLoop = if r.rnd() % 3 == 0 { (r.rnd() % 3) as i32 } else { (r.rnd() & 1) as i32 };
        }
        f.input.pressed_buttons = if r.rnd() & 1 != 0 { HSD_PAD_B as _ } else { (r.rnd() & 0xFFFF) as _ };
        if r.rnd() & 1 != 0 {
            f.input.pressed_buttons |= HSD_PAD_B as u32;
        }
        f.facing_dir = if r.rnd() & 1 != 0 { 1.0 } else { -1.0 };
        unsafe {
            f.u.fx.x222C_blasterGObj = if r.rnd() & 1 != 0 { (0x5150 + (r.rnd() % 4) as usize) as *mut HSD_GObj } else { core::ptr::null_mut() };
        }
        f.accessory4_cb = None;
        f.x21EC = None;
        f.take_dmg_cb = None;
        f.death2_cb = None;
        da.x10_FOX_BLASTER_ANGLE = r.rf();
        da.x14_FOX_BLASTER_VEL = r.rf();
        da.x1C_FOX_BLASTER_SHOT_ITKIND = (r.rnd() % 50) as _;
        f.cur_pos.x = r.rf();
        f.cur_pos.y = r.rf();
        f.cur_pos.z = r.rf();
        unsafe {
            R_FRAMES = (r.rnd() & 1) as i32;
            R_KIND = if r.rnd() % 3 == 0 {
                Ft_Kind_Falco as i32
            } else if r.rnd() % 3 == 0 {
                (r.rnd() % 30) as i32
            } else {
                Ft_Kind_Fox as i32
            };
            OUT.clear();
        }
        let (name, call): (&str, fn(*mut HSD_GObj)) = match i % 9 {
            0 => ("Shot", t::ftFx_SpecialN_CreateBlasterShot),
            1 => ("NStartAnim", t::ftFx_SpecialNStart_Anim),
            2 => ("AirNStartAnim", t::ftFx_SpecialAirNStart_Anim),
            3 => ("NLoopAnim", t::ftFx_SpecialNLoop_Anim),
            4 => ("AirNLoopAnim", t::ftFx_SpecialAirNLoop_Anim),
            5 => ("NStartIASA", t::ftFx_SpecialNStart_IASA),
            6 => ("AirNStartIASA", t::ftFx_SpecialAirNStart_IASA),
            7 => ("NLoopIASA", t::ftFx_SpecialNLoop_IASA),
            _ => ("AirNLoopIASA", t::ftFx_SpecialAirNLoop_IASA),
        };
        call(gp);
        let mut out = format!("{} {} |", i, name);
        unsafe {
            out.push_str(&core::mem::take(&mut OUT));
            write!(
                out,
                " | cv {} {} {} {} loop {} blaster {} acc4 {} x21EC {} dmg {} {} btn {}",
                f.cmd_vars[0],
                f.cmd_vars[1],
                f.cmd_vars[2],
                f.cmd_vars[3],
                f.mv.fx.SpecialN.isBlasterLoop,
                f.u.fx.x222C_blasterGObj as usize as u32,
                f.accessory4_cb.is_some() as i32,
                f.x21EC.is_some() as i32,
                f.take_dmg_cb.is_some() as i32,
                f.death2_cb.is_some() as i32,
                f.input.pressed_buttons as i32
            )
            .unwrap();
        }
        assert_eq!(out, line, "case {i}");
        n += 1;
    }
    assert_eq!(n, 4000);
}
