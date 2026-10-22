//! tu::ftdonkeyspecialn vs the C (gen/turef/tu_ftdonkeyspecialn_ref.c, shipping contraction; data
//! in tests/data/tu_ftdonkeyspecialn.txt; regenerate with
//! `python rust/melee_chara/gen/turef/tu_shard281.py tu_ftdonkeyspecialn`): the Enter / Anim /
//! IASA functions run on 6000 random fighter states, comparing the state afterwards and the log
//! of every outside call. One #[test]: the hooks are process globals.
#![allow(static_mut_refs)]

use melee_chara::stubs as st;
use melee_chara::tu::ftdonkeyspecialn as t;
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
            _ => ((self.rnd() % 4001) as i32 - 2000) as f32 / 97.0,
        }
    }
}

static mut LOG: String = String::new();
static mut FRAMES_REMAINING: i32 = 0;
static mut ESCAPE_RET: bool = false;
fn log(s: String) {
    unsafe { LOG.push_str(&s) }
}

unsafe extern "C" fn dummy_cb(_g: *mut HSD_GObj) {}

#[test]
fn matches_c_and_smoke() {
    let want = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/tu_ftdonkeyspecialn.txt")).unwrap();
    let mut f: Box<Fighter> = Box::new(Fighter::zeroed());
    let mut g: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    let da: &'static mut ftDonkeyAttributes = Box::leak(Box::new(ftDonkeyAttributes::zeroed()));
    let parts: &'static mut [FighterBone; 2] = Box::leak(Box::new(unsafe { core::mem::zeroed() }));
    f.dat_attrs = da as *mut _ as *mut _;
    f.parts = parts.as_mut_ptr();
    g.user_data = &mut *f as *mut Fighter as *mut _;
    let gp: *mut HSD_GObj = &mut *g;
    unsafe {
        st::FTANIM_ISFRAMESREMAINING = Some(|_| FRAMES_REMAINING);
        st::FTANIM_8006EBA4 = Some(|_| log("Z ".into()));
        st::FIGHTER_CHANGEMOTIONSTATE = Some(|_, ms, _, _, _, _, _| log(format!("M{ms} ")));
        st::FT_8008A2BC = Some(|_| log("W ".into()));
        st::FTCO_FALL_ENTER = Some(|_| log("F ".into()));
        st::FTCO_80096900 = Some(|_, _, _, _, _, lag| log(format!("L{:x} ", lag.to_bits())));
        st::FTCOMMON_8007D7FC = Some(|_| log("G ".into()));
        t::ext::EFSYNC_SPAWN = Some(|id, _, _| {
            log(format!("E{id} "));
            core::ptr::null_mut()
        });
        t::ext::FTCOLL_8007ABD0 = Some(|_, dmg, _| log(format!("D{dmg} ")));
        t::ext::FTCO_800BFFD0 = Some(|_, a, _| {
            log(format!("C{a} "));
            true
        });
        t::ext::FTCO_8009917C = Some(|_| ESCAPE_RET);
    }
    let mut r = Rng(0xD0DE5A1);
    let mut n = 0;
    for (i, line) in want.lines().enumerate() {
        let k = i % 11;
        unsafe { LOG.clear() };
        da.SpecialN.x2C_MAX_ARM_SWINGS = 1 + (r.rnd() % 5) as i32;
        da.SpecialN.x30_DAMAGE_PER_SWING = (r.rnd() % 5) as i32;
        da.SpecialN.x34_PUNCH_HORIZONTAL_VEL = r.rf();
        da.SpecialN.x38_LANDING_LAG = if r.rnd() & 1 != 0 { 0.0 } else { r.rf() };
        f.u.dk.x222C = (r.rnd() % 6) as i32;
        let sn = unsafe { &mut f.mv.dk.specialn };
        sn.x0 = (r.rnd() % 2) as i32;
        sn.x4 = (r.rnd() % 3) as i32;
        sn.x8 = (r.rnd() % 3) as i32;
        sn.xC = (r.rnd() % 6) as i32;
        sn.x10 = if r.rnd() & 1 != 0 { -1 } else { (r.rnd() % 30) as i32 };
        sn.x14 = if r.rnd() & 1 != 0 { -1 } else { (r.rnd() % 30) as i32 };
        f.cmd_vars[0] = r.rnd() % 2;
        f.ground_or_air = if r.rnd() & 1 != 0 { GA_Air } else { GA_Ground };
        f.x914[0].state = r.rnd() % 2;
        f.x914[0].damage = r.rf();
        f.x914[1].damage = r.rf();
        f.cur_anim_frame = if r.rnd() & 1 != 0 { 0.0 } else { 1.0 };
        f.facing_dir = if r.rnd() & 1 != 0 { 1.0 } else { -1.0 };
        f.gr_vel = 0.0;
        f.self_vel.y = 5.0;
        f.input.pressed_buttons = r.rnd() & 0x8000_0200;
        f.take_dmg_cb = None;
        f.death2_cb = None;
        f.take_dmg_2_cb = if r.rnd() & 1 != 0 { Some(dummy_cb) } else { None };
        f.pre_hitlag_cb = if r.rnd() & 1 != 0 { Some(dummy_cb) } else { None };
        f.post_hitlag_cb = if r.rnd() & 1 != 0 { Some(dummy_cb) } else { None };
        unsafe {
            FRAMES_REMAINING = (r.rnd() % 2) as i32;
            ESCAPE_RET = r.rnd() % 2 != 0;
        }
        match k {
            0 => t::ftDk_SpecialN_Enter(gp),
            1 => t::ftDk_SpecialAirN_Enter(gp),
            2 => t::ftDk_SpecialNLoop_Anim(gp),
            3 => t::ftDk_SpecialN_Anim(gp),
            4 => t::ftDk_SpecialNFull_Anim(gp),
            5 => t::ftDk_SpecialAirNLoop_Anim(gp),
            6 => t::ftDk_SpecialAirN_Anim(gp),
            7 => t::ftDk_SpecialAirNFull_Anim(gp),
            8 => t::ftDk_SpecialNLoop_IASA(gp),
            9 => {
                t::ftDk_SpecialN_Anim(gp);
                t::ftDk_SpecialN_Anim(gp);
            }
            _ => {
                t::ftDk_SpecialAirN_Enter(gp);
                t::ftDk_SpecialN_Enter(gp);
            }
        }
        let sn = unsafe { &f.mv.dk.specialn };
        let got = format!(
            "{} {} {:08x} {} {} {} {} {} {} {} {}{}{}{} | {}",
            i,
            k,
            f.gr_vel.to_bits(),
            sn.x0,
            sn.x4,
            sn.x8,
            sn.xC,
            sn.x10,
            sn.x14,
            unsafe { f.u.dk.x222C },
            f.take_dmg_cb.is_some() as i32,
            f.death2_cb.is_some() as i32,
            f.take_dmg_2_cb.is_some() as i32,
            (f.pre_hitlag_cb.is_some() || f.post_hitlag_cb.is_some()) as i32,
            unsafe { &LOG }
        );
        assert_eq!(got, line, "case {i}");
        n += 1;
    }
    assert_eq!(n, 6000);

    // The empty IASA / Phys-less stubs are no-ops and the unported-in-this-TU wrappers re-export.
    unsafe { LOG.clear() };
    t::ftDk_SpecialNStart_IASA(gp);
    t::ftDk_SpecialNCancel_IASA(gp);
    t::ftDk_SpecialN_IASA(gp);
    t::ftDk_SpecialNFull_IASA(gp);
    t::ftDk_SpecialAirNStart_IASA(gp);
    t::ftDk_SpecialAirNCancel_IASA(gp);
    t::ftDk_SpecialAirN_IASA(gp);
    t::ftDk_SpecialAirNFull_IASA(gp);
    assert!(unsafe { LOG.is_empty() });
}
