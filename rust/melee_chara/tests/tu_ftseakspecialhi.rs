//! tu::ftseakspecialhi vs the C (gen/turef/tu_ftseakspecialhi_ref.c, shipping contraction; data in
//! tests/data/tu_ftseakspecialhi.txt; regenerate with
//! `python rust/melee_chara/gen/turef/tu_shard441.py tu_ftseakspecialhi`): the state functions
//! ftSk_SpecialHi_80113EAC / 80113F68 (with their inlines), ftSk_SpecialHi_801137C8 and the travel
//! timers ftSk_SpecialHiStart_1_Anim / ftSk_SpecialAirHiStart_1_Anim, compared field by field with
//! the C result of the same 4000 seeded states. One #[test]: the hooks are process globals.
#![allow(static_mut_refs)]

use melee_chara::stubs as st;
use melee_chara::tu::ftseakspecialhi as t;
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

static mut NCHG: i32 = 0;
static mut MSID: i32 = -1;
static mut FLAGS: u32 = 0;
static mut ANIM: i32 = 0;
static mut D7FC: i32 = 0;
static mut FRAMES: i32 = 0;

#[test]
fn state_functions_match_c() {
    let want = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/tu_ftseakspecialhi.txt")).unwrap();
    let mut f: Box<Fighter> = Box::new(Fighter::zeroed());
    let mut g: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    let da: &'static mut ftSeakAttributes = Box::leak(Box::new(ftSeakAttributes::zeroed()));
    f.dat_attrs = da as *mut _ as *mut _;
    g.user_data = &mut *f as *mut Fighter as *mut _;
    let gp: *mut HSD_GObj = &mut *g;
    unsafe {
        st::FIGHTER_CHANGEMOTIONSTATE = Some(|_, ms, fl, _, _, _, _| {
            NCHG += 1;
            MSID = ms;
            FLAGS = fl;
        });
        st::FTANIM_8006EBA4 = Some(|_| ANIM += 1);
        st::FTANIM_ISFRAMESREMAINING = Some(|_| (FRAMES != 0) as i32);
        st::FTCOMMON_8007D7FC = Some(|_: &mut Fighter| D7FC += 1);
    }
    let mut r = Rng(0xF0F1F2);
    let mut lines = want.lines();
    for i in 0..4000 {
        let mode = (r.rnd() % 5) as i32;
        f.self_vel.x = r.rf();
        f.self_vel.y = r.rf();
        f.gr_vel = r.rf();
        f.mv.co.walk.fast_anim_frame = r.rf();
        f.mv.co.common.x14 = r.rf();
        f.mv.co.common.x18 = r.rf();
        f.mv.sk.specialn.x0 = (r.rnd() % 7) as i32 - 2;
        f.mv.sk.specialhi.x0 = (r.rnd() % 7) as i32 - 2;
        f.mv.sk.specialhi.xC = (r.rnd() % 9) as _;
        f.mv.sk.specialhi.vel.x = r.rf();
        f.mv.sk.specialhi.vel.y = r.rf();
        f.set_invisible((r.rnd() & 1) as u8);
        f.set_x2223_b4((r.rnd() & 1) as u8);
        f.x1968_jumpsUsed = (r.rnd() % 4) as u8;
        f.cur_anim_frame = r.rf() * 40.0;
        da.x54 = r.rf();
        da.x38 = (r.rnd() % 5) as i32;
        unsafe { FRAMES = (r.rnd() & 1) as i32 };
        f.accessory4_cb = if r.rnd() & 1 != 0 { Some(t::cb_fn_80113038) } else { None };
        unsafe {
            NCHG = 0;
            ANIM = 0;
            D7FC = 0;
            MSID = -1;
            FLAGS = 0;
            match mode {
                0 => t::ftSk_SpecialHi_80113EAC(gp),
                1 => t::ftSk_SpecialHi_80113F68(gp),
                2 => t::ftSk_SpecialHi_801137C8(gp),
                3 => t::ftSk_SpecialHiStart_1_Anim(gp),
                _ => t::ftSk_SpecialAirHiStart_1_Anim(gp),
            }
            let cb_is_113038 = f.accessory4_cb.map(|c| c as usize) == Some(t::cb_fn_80113038 as usize);
            let got = format!(
                "{} {} {} {} {:08x} {} {} {} {} {} {} {} {} {:08x} {:08x} {:08x} {:08x} {:08x} {:08x} {:08x} {:08x}",
                i,
                mode,
                NCHG,
                MSID,
                FLAGS,
                ANIM,
                D7FC,
                cb_is_113038 as i32,
                f.invisible(),
                f.x2223_b4(),
                f.mv.sk.specialhi.x0,
                f.mv.sk.specialhi.xC,
                f.mv.sk.specialn.x0,
                f.self_vel.x.to_bits(),
                f.self_vel.y.to_bits(),
                f.gr_vel.to_bits(),
                f.mv.co.walk.fast_anim_frame.to_bits(),
                f.mv.co.common.x14.to_bits(),
                f.mv.co.common.x18.to_bits(),
                f.mv.sk.specialhi.vel.x.to_bits(),
                f.mv.sk.specialhi.vel.y.to_bits(),
            );
            assert_eq!(got, lines.next().unwrap(), "case {i} mode {mode}");
        }
    }
    assert!(lines.next().is_none());
}
