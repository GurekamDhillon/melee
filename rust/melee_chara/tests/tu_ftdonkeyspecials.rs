//! tu::ftdonkeyspecials vs the C (gen/turef/tu_ftdonkeyspecials_ref.c, shipping contraction; data
//! in tests/data/tu_ftdonkeyspecials.txt; regenerate with
//! `python rust/melee_chara/gen/turef/tu_shard281.py tu_ftdonkeyspecials`): `ftDk_SpecialAirS_Enter`
//! (odd cases) and `ftDk_SpecialAirS_Phys` (even cases) on 4000 random states, then a smoke test
//! of the Coll transitions. One #[test]: the hooks are process globals.
#![allow(static_mut_refs)]

use melee_chara::stubs as st;
use melee_chara::tu::ftdonkeyspecials as t;
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
fn log(s: String) {
    unsafe { LOG.push_str(&s) }
}

#[test]
fn matches_c_and_smoke() {
    let want = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/tu_ftdonkeyspecials.txt")).unwrap();
    let mut f: Box<Fighter> = Box::new(Fighter::zeroed());
    let mut g: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    let da: &'static mut ftDonkeyAttributes = Box::leak(Box::new(ftDonkeyAttributes::zeroed()));
    f.dat_attrs = da as *mut _ as *mut _;
    g.user_data = &mut *f as *mut Fighter as *mut _;
    let gp: *mut HSD_GObj = &mut *g;
    unsafe {
        st::FTANIM_8006EBA4 = Some(|_| log("Z ".into()));
        st::FIGHTER_CHANGEMOTIONSTATE = Some(|_, ms, _, _, _, _, _| log(format!("M{ms} ")));
    }
    let mut r = Rng(0xD0D0A1B);
    let mut n = 0;
    for (i, line) in want.lines().enumerate() {
        unsafe { LOG.clear() };
        da.SpecialS.x3C_MIN_STICK_X_MOMENTUM = r.rf();
        da.SpecialS.x40_MOMENTUM_TRANSITION_MODIFIER = r.rf();
        da.SpecialS.x44_AERIAL_GRAVITY = r.rf();
        f.co_attrs.terminal_velocity = r.rf();
        f.cmd_vars[0] = r.rnd() % 2;
        f.self_vel.x = r.rf();
        f.self_vel.y = r.rf();
        f.x74_self_accel.x = r.rf();
        f.accessory4_cb = None;
        if i & 1 != 0 {
            t::ftDk_SpecialAirS_Enter(gp);
        } else {
            t::ftDk_SpecialAirS_Phys(gp);
        }
        let got = format!(
            "{} {:08x} {:08x} {:08x} {} {} | {}",
            i,
            f.self_vel.x.to_bits(),
            f.self_vel.y.to_bits(),
            f.x74_self_accel.x.to_bits(),
            f.cmd_vars[0],
            f.accessory4_cb.is_some() as i32,
            unsafe { &LOG }
        );
        assert_eq!(got, line, "case {i}");
        n += 1;
    }
    assert_eq!(n, 4000);

    // Coll transitions: ground path picks ft_800827A0 when cmd_vars[0] is set, ft_80082708 else.
    unsafe {
        melee_fighter::hooks::UN_803224DC = Some(|_, _, _| false);
        melee_fighter::hooks::MPLIB_8004D164_LINE_COUNT = Some(|| None);
        melee_fighter::hooks::FTCO_FALL_ENTER = Some(|_| {});
        st::FT_800827A0 = Some(|_| {
            log("A0 ".into());
            0
        });
        st::FT_80082708 = Some(|_| {
            log("08 ".into());
            0
        });
        st::FT_80081D0C = Some(|_| {
            log("1D ".into());
            1
        });
        LOG.clear();
    }
    f.cmd_vars[0] = 1;
    f.set_x2219_b0(1);
    f.pre_hitlag_cb = None;
    t::ftDk_SpecialS_Coll(gp);
    assert_eq!(unsafe { LOG.as_str() }, format!("A0 M{} ", ftDk_MS_SpecialAirS));
    assert!(f.pre_hitlag_cb.is_some() && f.post_hitlag_cb.is_some());
    unsafe { LOG.clear() };
    f.cmd_vars[0] = 0;
    f.set_x2219_b0(0);
    f.pre_hitlag_cb = None;
    t::ftDk_SpecialS_Coll(gp);
    assert_eq!(unsafe { LOG.as_str() }, format!("08 M{} ", ftDk_MS_SpecialAirS));
    assert!(f.pre_hitlag_cb.is_none());
    unsafe { LOG.clear() };
    t::ftDk_SpecialAirS_Coll(gp);
    assert_eq!(unsafe { LOG.as_str() }, format!("1D M{} ", ftDk_MS_SpecialS));
    t::ftDk_SpecialS_IASA(gp);
    t::ftDk_SpecialAirS_IASA(gp);
}
