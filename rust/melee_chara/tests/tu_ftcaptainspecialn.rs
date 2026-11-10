//! tu::ftcaptainspecialn vs the C (gen/turef/tu_ftcaptainspecialn_ref.c, shipping contraction;
//! data in tests/data/tu_ftcaptainspecialn.txt; regenerate with
//! `python rust/melee_chara/gen/turef/tu_shard89.py tu_ftcaptainspecialn`), plus a smoke test
//! of the stateful callbacks through recording hooks.
#![allow(static_mut_refs)]

use melee_chara::stubs as st;
use melee_chara::tu::ftcaptainspecialn as t;
use melee_types::*;

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
    let want = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/tu_ftcaptainspecialn.txt")).unwrap();
    let mut f: Box<Fighter> = Box::new(Fighter::zeroed());
    let da: &'static mut ftCaptain_DatAttrs = Box::leak(Box::new(ftCaptain_DatAttrs::zeroed()));
    f.dat_attrs = da as *mut _ as *mut _;
    let mut r = Rng(0x2468ACE);
    let mut n = 0;
    for (i, line) in want.lines().enumerate() {
        f.input.lstick[0].y = r.rf();
        da.specialn_stick_range_y_pos = r.rf();
        da.specialn_stick_range_y_neg = r.rf();
        da.specialn_angle_diff = r.rf() * 60.0;
        da.specialn_vel_x = r.rf();
        f.facing_dir = if r.rnd() & 1 != 0 { 1.0 } else { -1.0 };
        f.cmd_vars[0] = r.rnd() & 1;
        f.self_vel.x = r.rf();
        f.self_vel.y = r.rf();
        let a = t::ftCaptain_SpecialN_GetAngleVel(&f);
        t::special_air_n_iasa(&mut f);
        let got = format!("{} {:08x} {:08x} {:08x} {}", i, a.to_bits(), f.self_vel.x.to_bits(), f.self_vel.y.to_bits(), f.cmd_vars[0]);
        assert_eq!(got, line, "case {i}");
        n += 1;
    }
    assert_eq!(n, 4000);
}

static mut LOG: std::vec::Vec<String> = std::vec::Vec::new();
fn log(s: String) {
    unsafe { LOG.push(s) }
}

#[test]
fn smoke_callbacks() {
    let mut f: Box<Fighter> = Box::new(Fighter::zeroed());
    let mut g: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    let da: &'static mut ftCaptain_DatAttrs = Box::leak(Box::new(ftCaptain_DatAttrs::zeroed()));
    let mut parts: std::vec::Vec<FighterBone> = (0..80).map(|_| unsafe { core::mem::zeroed() }).collect();
    f.dat_attrs = da as *mut _ as *mut _;
    f.parts = parts.as_mut_ptr();
    f.kind = Ft_Kind_Ganon;
    g.user_data = &mut *f as *mut Fighter as *mut _;
    let gp: *mut HSD_GObj = &mut *g;
    unsafe {
        st::FTANIM_ISFRAMESREMAINING = Some(|_| 0);
        st::FT_8008A2BC = Some(|_| log("8008A2BC".into()));
        st::FTCO_FALL_ENTER = Some(|_| log("Fall".into()));
        st::FT_80084FA8 = Some(|_| log("84FA8".into()));
        st::FT_80084EEC = Some(|_| log("84EEC".into()));
        t::ext::LB_800119DC = Some(|_, a, b, _, _| log(format!("lb {a} {b}")));
        t::ext::EFSYNC_SPAWN = Some(|id, _, args| {
            log(format!("ef {id} {}", args.len()));
            core::ptr::null_mut()
        });
    }
    // Ganondorf's wind: odd frames 16..=50 use scale 2, 51..=68 scale 4.
    f.cur_anim_frame = 17.0;
    t::ftCa_SpecialN_Anim(gp);
    f.cur_anim_frame = 53.0;
    t::ftCa_SpecialAirN_Anim(gp);
    f.cur_anim_frame = 20.0;
    t::ftCa_SpecialN_Anim(gp);
    // doPhys: first pulse spawns the effect and sets x2219_b0.
    unsafe { f.x2210.x0.set_throw_flags_b1(1) };
    t::ftCa_SpecialN_Phys(gp);
    assert_eq!(f.x2219_b0(), 1);
    assert_eq!(unsafe { f.x2210.x0.throw_flags_b1() }, 0);
    // SpecialAirN_Phys: case 0 -> ft_80084EEC, case 1 scales self_vel.
    f.cmd_vars[1] = 0;
    t::ftCa_SpecialAirN_Phys(gp);
    f.cmd_vars[1] = 1;
    da.specialn_vel_mul = 0.5;
    f.self_vel.x = 3.0;
    f.self_vel.y = -2.0;
    t::ftCa_SpecialAirN_Phys(gp);
    assert_eq!((f.self_vel.x, f.self_vel.y), (1.5, -1.0));
    let got = unsafe { LOG.join(",") };
    assert_eq!(got, "lb 2 2,8008A2BC,lb 2 4,Fall,8008A2BC,ef 1291 2,84FA8,84EEC");
    drop(parts);
}
