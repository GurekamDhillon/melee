//! tu::ftkirbyspecialcaptain vs the C (gen/turef/tu_ftkirbyspecialcaptain_ref.c, shipping contraction;
//! data in tests/data/tu_ftkirbyspecialcaptain.txt; regenerate with
//! `python rust/melee_chara/gen/turef/tu_shard305.py`), plus a smoke test of the stateful callbacks
//! through recording hooks.
#![allow(static_mut_refs)]

use melee_chara::stubs as st;
use melee_chara::tu::ftkirbyspecialcaptain as t;
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
    let want = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/tu_ftkirbyspecialcaptain.txt")).unwrap();
    let mut f: Box<Fighter> = Box::new(Fighter::zeroed());
    let da: &'static mut ftKb_DatAttrs = Box::leak(Box::new(ftKb_DatAttrs::zeroed()));
    f.dat_attrs = da as *mut _ as *mut _;
    let mut r = Rng(0x13579BD);
    let mut n = 0;
    for (i, line) in want.lines().enumerate() {
        f.input.lstick[0].y = r.rf();
        da.specialn_ca_y_axis_range = r.rf();
        da.specialn_ca_x_axis_range = r.rf();
        da.specialn_ca_angle_difference = r.rf() * 60.0;
        da.specialn_ca_forward_momentum = r.rf();
        f.facing_dir = if r.rnd() & 1 != 0 { 1.0 } else { -1.0 };
        f.cmd_vars[0] = r.rnd() & 1;
        f.self_vel.x = r.rf();
        f.self_vel.y = r.rf();
        t::ca_special_air_n_iasa(&mut f);
        let got = format!("{} {:08x} {:08x} {}", i, f.self_vel.x.to_bits(), f.self_vel.y.to_bits(), f.cmd_vars[0]);
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
    let da: &'static mut ftKb_DatAttrs = Box::leak(Box::new(ftKb_DatAttrs::zeroed()));
    let mut parts: std::vec::Vec<FighterBone> = (0..60).map(|_| unsafe { core::mem::zeroed() }).collect();
    f.dat_attrs = da as *mut _ as *mut _;
    f.parts = parts.as_mut_ptr();
    f.u.kb.hat.kind = Ft_Kind_Ganon;
    g.user_data = &mut *f as *mut Fighter as *mut _;
    let gp: *mut HSD_GObj = &mut *g;
    unsafe {
        st::FIGHTER_CHANGEMOTIONSTATE = Some(|_, msid, flags, start, speed, _, _| log(format!("cms {msid} {flags:x} {start} {speed}")));
        st::FTANIM_8006EBA4 = Some(|_| log("8006EBA4".into()));
        st::FT_800827A0 = Some(|_| 0);
        st::FT_80081D0C = Some(|_| 1);
        st::FTCOMMON_8007D7FC = Some(|_| {});
        st::FTCOMMON_8007DB24 = Some(|_| log("7DB24".into()));
        st::FT_80084FA8 = Some(|_| log("84FA8".into()));
        st::FT_80084EEC = Some(|_| log("84EEC".into()));
        st::FT_80084DB0 = Some(|_| log("84DB0".into()));
        t::ext::EFSYNC_SPAWN = Some(|id, _, args| {
            log(format!("ef {id} {}", args.len()));
            core::ptr::null_mut()
        });
    }
    // Entries pick the Warlock Punch ids for a non-Captain hat and clear the script variables.
    f.cmd_vars = [7, 7, 7, 7];
    f.anon_2210.throw_flags = 0xFFFF;
    t::ftKb_SpecialNCa_800F99BC(gp);
    assert_eq!(f.cmd_vars[..2], [0, 0]);
    assert_eq!(unsafe { f.anon_2210.throw_flags }, 0);
    assert!(f.pre_hitlag_cb.is_some() && f.post_hitlag_cb.is_some());
    f.u.kb.hat.kind = Ft_Kind_Captain;
    t::ftKb_SpecialNCa_800F9A54(gp);
    // Phys: first throw_flags_b1 pulse spawns the hat's effect (Falcon: 1195) and sets x2219_b0.
    unsafe { f.anon_2210.anon_0.set_throw_flags_b1(1) };
    t::ftKb_CaSpecialN_Phys(gp);
    assert_eq!(f.x2219_b0(), 1);
    unsafe { f.anon_2210.anon_0.set_throw_flags_b1(1) };
    t::ftKb_CaSpecialN_Phys(gp);
    // Air Phys: cmd_vars[1] 0 / 1 / 2 pick ft_80084EEC / scale self_vel / ft_80084DB0.
    da.specialn_ca_additional_vertical_momentum = 0.5;
    f.self_vel.x = 3.0;
    f.self_vel.y = -2.0;
    for v in 0..3 {
        f.cmd_vars[1] = v;
        t::ftKb_CaSpecialAirN_Phys(gp);
    }
    assert_eq!((f.self_vel.x, f.self_vel.y), (1.5, -1.0));
    // Coll: ground callback does nothing while grounded, changes state when airborne.
    f.cur_anim_frame = 12.0;
    t::ftKb_CaSpecialN_Coll(gp);
    t::ftKb_CaSpecialAirN_Coll(gp);
    let got = unsafe { LOG.join(",") };
    let flags = format!("{:x}", t::ftKb_Ca_transition_flags);
    let want = format!(
        "cms {} 0 0 1,8006EBA4,cms {} 0 0 1,8006EBA4,ef 1195 2,84FA8,7DB24,84FA8,84EEC,84DB0,cms {} {flags} 12 1,cms {} {flags} 12 1",
        ftKb_MS_GnSpecialN, ftKb_MS_CaSpecialAirN, ftKb_MS_CaSpecialAirN, ftKb_MS_CaSpecialN
    );
    assert_eq!(got, want);
    drop(parts);
}
