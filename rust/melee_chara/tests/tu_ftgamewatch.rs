//! The Mr. Game & Watch TUs (game_tus.txt lines 297-304) vs the C: the state-machine logic of
//! Oil Panic (release damage, IASA), Fire Rescue (spawn height, aim), Chef (sausage pick, IASA,
//! loop), Judgement (roll, spawn) is cut verbatim from the game's .c files and run with the
//! shipping build's float contraction (gen/turef/tu_ftgamewatch_ref.c, data in
//! tests/data/tu_ftgamewatch.txt; regenerate with `python rust/melee_chara/gen/turef/tu_shard297.py`).
//! Every callee outside the cut functions is a stand-in that folds its arguments into a running
//! hash on both sides, so the call sequence and arguments are compared too.
//! One #[test]: the hooks are process globals.
#![allow(static_mut_refs, unused_unsafe)]

use melee_chara::stubs as st;
use melee_chara::tu::ftgamewatch::ext as gw;
use melee_chara::tu::{
    ftgamewatchspecialhi as shi, ftgamewatchspeciallw as slw, ftgamewatchspecialn as sn, ftgamewatchspecials as ss,
};
use melee_fighter::hooks as fh;
use melee_types::*;

static mut LOG: u64 = 0;
fn lg(v: u64) {
    unsafe { LOG = (LOG ^ v).wrapping_mul(1099511628211) }
}
static mut HS: u32 = 0x2545F491;
fn hnext() -> u32 {
    unsafe {
        HS ^= HS << 13;
        HS ^= HS >> 17;
        HS ^= HS << 5;
        HS
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
        match self.rnd() % 8 {
            0 => 0.0,
            1 => -0.0,
            2 => 1.0,
            3 => -1.0,
            _ => ((self.rnd() % 4001) as i32 - 2000) as f32 / 997.0,
        }
    }
    fn rpos(&mut self) -> f32 {
        (self.rnd() % 3001) as f32 / 211.0
    }
}

fn install() {
    unsafe {
        st::FIGHTER_CHANGEMOTIONSTATE = Some(|_, msid, flags, a, b, c, _| {
            lg(1);
            lg(msid as u32 as u64);
            lg(flags as u64);
            lg(a.to_bits() as u64);
            lg(b.to_bits() as u64);
            lg(c.to_bits() as u64);
        });
        st::FTANIM_8006EBA4 = Some(|_| lg(2));
        st::FTPARTS_80074B0C = Some(|_, idx, val| {
            lg(3);
            lg(idx as u32 as u64);
            lg(val as u32 as u64);
        });
        gw::FTCOMMON_UPDATEFACING = Some(|fp| {
            lg(4);
            fp.facing_dir = -fp.facing_dir;
        });
        gw::FTPARTSETROTY = Some(|_, part, v| {
            lg(5);
            lg(part as u32 as u64);
            lg(v.to_bits() as u64);
        });
        gw::FTCOMMON_GETMODELSCALE = Some(|fp| fp.x34_scale.y * 0.75);
        gw::FTPARTS_GETBONEINDEX = Some(|_, part| part + 3);
        gw::LB_8000B1CC = Some(|_, a, out| unsafe {
            let (x, y, z) = if a.is_null() { (0.5, 0.75, 0.25) } else { ((*a).x, (*a).y, (*a).z) };
            (*out).x = x + 1.25;
            (*out).y = y + 2.5;
            (*out).z = z + 3.75;
        });
        sn::ext::HSD_RANDI = Some(|max| if max > 0 { (hnext() % max as u32) as i32 } else { 0 });
        slw::ext::FTCOLL_CREATEABSORBHIT = Some(|_, _| lg(6));
        shi::ext::IT_802C8038 = Some(|_, pos, part, kind, facing, scale| unsafe {
            lg(7);
            lg((*pos).x.to_bits() as u64);
            lg((*pos).y.to_bits() as u64);
            lg((*pos).z.to_bits() as u64);
            lg(part as u32 as u64);
            lg(kind as u32 as u64);
            lg(facing.to_bits() as u64);
            lg(scale.to_bits() as u64);
            0x1000 as *mut HSD_GObj
        });
        ss::ext::IT_802C7774 = Some(|facing, _, pos, part, judge| unsafe {
            lg(8);
            lg(facing.to_bits() as u64);
            lg((*pos).x.to_bits() as u64);
            lg((*pos).y.to_bits() as u64);
            lg((*pos).z.to_bits() as u64);
            lg(part as u32 as u64);
            lg(judge as u32 as u64);
            0x2000 as *mut HSD_GObj
        });
        ss::ext::IT_8028FAF4 = Some(|_, pos| unsafe {
            lg(9);
            lg((*pos).x.to_bits() as u64);
            lg((*pos).y.to_bits() as u64);
            lg((*pos).z.to_bits() as u64);
        });
        sn::ext::IT_802C837C = Some(|_, pos, kind, var, facing| unsafe {
            lg(10);
            lg((*pos).x.to_bits() as u64);
            lg((*pos).y.to_bits() as u64);
            lg((*pos).z.to_bits() as u64);
            lg(kind as u32 as u64);
            lg(var as u32 as u64);
            lg(facing.to_bits() as u64);
        });
    }
}

#[test]
fn matches_c() {
    let want = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/tu_ftgamewatch.txt")).unwrap();
    install();
    let mut f: Box<Fighter> = Box::new(Fighter::zeroed());
    let mut g: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    let mut parts: std::vec::Vec<FighterBone> = (0..80).map(|_| unsafe { core::mem::zeroed() }).collect();
    let da: &'static mut ftGameWatchAttributes = Box::leak(Box::new(ftGameWatchAttributes::zeroed()));
    let cd: &'static mut ftCommonData = Box::leak(Box::new(unsafe { core::mem::zeroed() }));
    g.user_data = &mut *f as *mut Fighter as *mut _;
    f.dat_attrs = da as *mut _ as *mut _;
    f.parts = parts.as_mut_ptr();
    unsafe { fh::P_FT_COMMON_DATA = &*cd };
    let gp: *mut HSD_GObj = &mut *g;
    let mut r = Rng(0x297304);
    let mut n = 0;
    for (i, line) in want.lines().enumerate() {
        let k = i % 8;
        unsafe { LOG = 1469598103934665603 };
        f.cmd_vars[0] = r.rnd() % 4;
        f.cmd_vars[1] = r.rnd() % 300;
        f.facing_dir = if r.rnd() & 1 != 0 { 1.0 } else { -1.0 };
        f.lstick_angle = 0.0;
        f.x34_scale.y = r.rf();
        f.input.lstick[0].x = r.rf();
        f.input.held_buttons[0] = r.rnd() & HSD_PAD_B as u32;
        f.input.pressed_buttons = r.rnd() & HSD_PAD_B as u32;
        unsafe {
            f.mv.gw.SpecialLw.isRelease = (r.rnd() & 1) as i32;
            f.mv.gw.SpecialLw.turnFrames = (r.rnd() % 5) as i32 - 1;
            f.mv.gw.SpecialN.isChefLoopDisable = (r.rnd() & 1) as i32;
            f.mv.gw.SpecialN.maxSausage = (r.rnd() % 6) as i32;
            f.u.gw.x2238_panicCharge = (r.rnd() % 5) as i32;
            f.u.gw.x223C_panicDamage = (r.rnd() % 700) as i32 - 20;
            f.u.gw.x2240_chefVar1 = (r.rnd() % 5) as i32;
            f.u.gw.x2244_chefVar2 = (r.rnd() % 5) as i32;
            f.u.gw.x222C_judgeVar1 = (r.rnd() % 9) as i32;
            f.u.gw.x2230_judgeVar2 = (r.rnd() % 9) as i32;
        }
        f.accessory4_cb = None;
        da.x74_GAMEWATCH_PANIC_DAMAGE_ADD = r.rpos();
        da.x78_GAMEWATCH_PANIC_DAMAGE_MUL = r.rpos();
        da.x7C_GAMEWATCH_PANIC_TURN_FRAMES = (r.rnd() % 20) as f32;
        da.x58_GAMEWATCH_RESCUE_STICK_RANGE = r.rf();
        da.x5C_GAMEWATCH_RESCUE_ANGLE_UNK = r.rf();
        da.x1C_GAMEWATCH_CHEF_MAX = (r.rnd() % 7) as f32;
        da.x18_GAMEWATCH_CHEF_LOOPFRAME = r.rf();
        for j in 0..9 {
            da.x34_GAMEWATCH_JUDGE_ROLL[j] = (r.rnd() % 40) as i32 + 1;
        }
        cd.horizontal_stick_deadzone = r.rpos() / 8.0;
        unsafe {
            f.u.gw.x226C_rescueGObj = core::ptr::null_mut();
        }
        f.motion_id = ftGw_MS_SpecialHi as FtMotionId + (r.rnd() % 2) as i32;
        match k {
            0 => slw::ftGw_SpecialLwShoot_ReleaseOil(gp),
            1 => slw::ftGw_SpecialAirLwShoot_ReleaseOil(gp),
            2 => slw::ftGw_SpecialLw_IASA(gp),
            3 => slw::ftGw_SpecialAirLw_IASA(gp),
            4 => shi::ftGw_SpecialHi_ItemRescueSetup(gp),
            5 => shi::ftGw_SpecialAirHi_IASA(gp),
            6 => {
                sn::ftGw_SpecialN_CreateSausage(gp);
                sn::ftGw_SpecialN_IASA(gp);
                sn::ftGw_SpecialAirN_IASA(gp);
            }
            _ => {
                ss::ftGw_SpecialS_GetRandomInt(gp);
                f.cmd_vars[1] = 1;
                ss::ftGw_SpecialS_ItemJudgementSetup(gp);
            }
        }
        unsafe {
            lg(!f.u.gw.x2264_judgementGObj.is_null() as u64);
            lg(!f.u.gw.x226C_rescueGObj.is_null() as u64);
            lg(f.accessory4_cb.is_some() as u64);
            lg(f.u.gw.x222C_judgeVar1 as i64 as u64);
            lg(f.u.gw.x2230_judgeVar2 as i64 as u64);
            let rel = f.mv.gw.SpecialLw.isRelease | f.mv.gw.SpecialLw.turnFrames.wrapping_shl(8);
            let chef = f.mv.gw.SpecialN.isChefLoopDisable | f.mv.gw.SpecialN.maxSausage.wrapping_shl(8);
            let got = format!(
                "{} {} {:016x} {:08x} {:08x} {:08x} {:08x} {:08x} {:08x} {} {} {} {}",
                i,
                k,
                LOG,
                f.cmd_vars[0],
                f.cmd_vars[1],
                f.facing_dir.to_bits(),
                f.lstick_angle.to_bits(),
                rel as u32,
                chef as u32,
                f.u.gw.x2238_panicCharge,
                f.u.gw.x223C_panicDamage,
                f.u.gw.x2240_chefVar1,
                f.u.gw.x2244_chefVar2
            );
            assert_eq!(got, line, "case {i}");
        }
        n += 1;
    }
    assert_eq!(n, 3000);
    drop(parts);
}
