//! tu::ftcaptainspecials vs the C (gen/turef/tu_ftcaptainspecials_ref.c, shipping contraction;
//! data in tests/data/tu_ftcaptainspecials.txt; regenerate with
//! `python rust/melee_chara/gen/turef/tu_shard89.py tu_ftcaptainspecials`), plus a smoke test.
//! One #[test]: the hooks are process globals.
#![allow(static_mut_refs)]

use melee_chara::stubs as st;
use melee_chara::tu::ftcaptainspecials as t;
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

static mut LOG: std::vec::Vec<String> = std::vec::Vec::new();
fn log(s: String) {
    unsafe { LOG.push(s) }
}

#[test]
fn matches_c_and_smoke() {
    let want = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/tu_ftcaptainspecials.txt")).unwrap();
    let mut f: Box<Fighter> = Box::new(Fighter::zeroed());
    let mut g: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    let da: &'static mut ftCaptain_DatAttrs = Box::leak(Box::new(ftCaptain_DatAttrs::zeroed()));
    f.dat_attrs = da as *mut _ as *mut _;
    g.user_data = &mut *f as *mut Fighter as *mut _;
    let gp: *mut HSD_GObj = &mut *g;
    unsafe { st::FT_80085134 = Some(|_| {}) };
    let mut r = Rng(0x13579BD);
    let mut n = 0;
    for (i, line) in want.lines().enumerate() {
        da.specials_grav = r.rf();
        da.specials_terminal_vel = r.rf();
        unsafe { f.mv.ca.specials.grav = r.rf() };
        f.self_vel.y = r.rf();
        f.cmd_vars[1] = r.rnd() % 3;
        if i & 1 != 0 {
            t::ftCa_SpecialAirS_Phys(gp);
        } else {
            t::ftCa_SpecialAirSStart_Phys(gp);
        }
        let got = format!("{} {:08x} {:08x}", i, unsafe { f.mv.ca.specials.grav }.to_bits(), f.self_vel.y.to_bits());
        assert_eq!(got, line, "case {i}");
        n += 1;
    }
    assert_eq!(n, 4000);

    // smoke: Enter, OnDetect (fighter hit while in SpecialSStart), the ground Coll.
    let mut parts: std::vec::Vec<FighterBone> = (0..80).map(|_| unsafe { core::mem::zeroed() }).collect();
    f.parts = parts.as_mut_ptr();
    f.kind = Ft_Kind_Captain;
    unsafe {
        st::FIGHTER_CHANGEMOTIONSTATE = Some(|_, ms, fl, _, _, _, _| log(format!("ms {ms} {fl:x}")));
        st::FTANIM_8006EBA4 = Some(|_| {});
        st::FTCOMMON_8007D7FC = Some(|_| log("7D7FC".into()));
        st::FT_80082708 = Some(|_| 0);
        st::EFLIB_DESTROYALL = Some(|_| log("destroy".into()));
        st::FTCO_FALL_ENTER = Some(|_| log("Fall".into()));
        st::FTCO_80096900 = Some(|_, _, _, _, _, lag| log(format!("fallspecial {lag}")));
        t::ext::EFSYNC_SPAWN = Some(|id, _, a| {
            log(format!("ef {id} {}", a.len()));
            core::ptr::null_mut()
        });
    }
    f.gr_vel = 5.0;
    f.self_vel = Vec { x: 1.0, y: 2.0, z: 3.0 };
    t::ftCa_SpecialS_Enter(gp);
    assert_eq!(unsafe { f.u.ca.during_specials_start }, 1);
    assert_eq!((f.gr_vel, f.self_vel.x, f.self_vel.y, f.self_vel.z), (0.0, 0.0, 0.0, 0.0));
    assert!(f.hurtbox_detect_cb.is_some() && f.take_dmg_cb.is_some());
    // OnDetect: victim is a fighter, motion is SpecialSStart -> onDetectGround
    let mut victim: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    victim.classifier = HSD_GOBJ_CLASS_FIGHTER as u16;
    f.unk_gobj = &mut *victim;
    f.cmd_vars[0] = 1;
    f.motion_id = ftCa_MS_SpecialSStart as FtMotionId;
    f.gr_vel = 2.0;
    da.specials_gr_vel_x = 0.25;
    t::ftCa_SpecialS_OnDetect(gp);
    assert_eq!(f.gr_vel, 0.5);
    // grounded start Coll losing the ground with no miss lag -> Fall
    f.cmd_vars[2] = 1;
    da.specials_miss_landing_lag = 0.0;
    t::ftCa_SpecialSStart_Coll(gp);
    da.specials_hit_landing_lag = 12.0;
    t::ftCa_SpecialS_Coll(gp);
    let got = unsafe { LOG.join(",") };
    assert_eq!(got, "7D7FC,ms 349 0,ef 1169 1,7D7FC,ms 350 c4c5082,destroy,Fall,destroy,fallspecial 12");
    drop(parts);
}
