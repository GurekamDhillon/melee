//! tu::gmvs vs the C (gen/cref/tu/gmvs.c: the whole TU, shipping contraction, 4-byte pointers;
//! regenerate with `python rust/melee_modes/gen/cref/tu/run_tu.py gmvs`).
//!
//! Every callee out of the TU is a recording stub (tests/tu_gmvs_stubs, generated); the state the TU reads (the
//! `controller` it owns, the pads, the match results...) is synthetic and filled from the shared generator. The ops
//! (tests/tu_gmvs_ops) are generated from the OPS block of the C driver. One line per case: op case ret state-hash trace.
//!
//! The static inline `fn_8016CFE0_inline` (debug-pad end check + respawn timers) has no op of its own: it runs inside the
//! ops `fn_8016CFE0` and `gm_Scene_Training_OnFrame`, whose state hash covers `controller` and the pads.
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, dead_code)]
mod tu_gmvs_hs;
mod tu_gmvs_ops;
mod tu_gmvs_stubs;
mod tu_gm_harness;
use core::ffi::c_void;
use core::ptr::{addr_of_mut, null_mut};
use melee_modes::tu::globals::{DbLevel, HSD_PadCopyStatus, HSD_PadMasterStatus, HSD_RandSeedPtr, __OSBusClock};
use melee_modes::tu::gmvs as m;
use melee_types::*;
use tu_gm_harness::*;
use tu_gmvs_hs::*;

pub const G_CTRL: u32 = 1;
pub const G_SMD: u32 = 2;
pub const G_PIDX: u32 = 4;
pub const G_EMD: u32 = 8;
pub const G_PADS: u32 = 16;
pub const G_MISC: u32 = 32;
pub const G_LBL: u32 = 64;
pub const G_ME: u32 = 128;
pub const G_ALLSTAR: u32 = 256;
pub const G_RAND: u32 = 512;
const NCASES: u32 = 600;

pub use m::controller;

pub static mut LBL: lbl_8046B488_t = unsafe { core::mem::zeroed() };
pub static mut ME: MatchEnd = unsafe { core::mem::zeroed() };
pub static mut SMD: StartMeleeData = unsafe { core::mem::zeroed() };
pub static mut PIDX: PlayerInitData = unsafe { core::mem::zeroed() };
pub static mut EMD: m::EndMeleeData = unsafe { core::mem::zeroed() };
pub static mut PINT: i32 = 0;
pub static mut BYTEX: u8 = 0;
pub static mut ALLSTAR: UnkAllstarData = unsafe { core::mem::zeroed() };
pub static mut UNK1P: Unk1PData = unsafe { core::mem::zeroed() };
pub static mut X54OBJ: StartMeleeRules_anon_250 = unsafe { core::mem::zeroed() };
static mut RANDSEED: u32 = 0;
static mut SLOTT: [u8; 6] = [0; 6];
static mut TEAMS: [u8; 6] = [0; 6];
static mut STOCKS: [u8; 6] = [0; 6];
static mut PLID: [u8; 6] = [0; 6];
static mut SAVEDATA: gmm_x0 = unsafe { core::mem::zeroed() };

fn fnid(p: usize) -> u32 {
    if p == 0 {
        return 0;
    }
    if p == m::fn_8016B784 as usize {
        return 1;
    }
    if p == m::fn_8016B7B4 as usize {
        return 2;
    }
    if p == m::fn_8016B7F8 as usize {
        return 3;
    }
    if p == m::fn_8016B88C as usize {
        return 4;
    }
    if p == m::fn_8016C7D0 as usize {
        return 5;
    }
    if p == m::gm_AnyControllerPressedStart as usize {
        return 6;
    }
    if p == m::gm_AnyControllerPressedZ as usize {
        return 7;
    }
    if p == m::db_RunEveryFrame_ptr as usize {
        return 8;
    }
    99
}

/// A MatchEnd address handed over as an int: which of the two blocks the TU passes (or neither).
unsafe fn meid(a: isize) -> u32 {
    if a == addr_of_mut!(controller.state.x24C) as isize {
        return 1;
    }
    if a == addr_of_mut!(EMD.xC) as isize {
        return 2;
    }
    0
}

unsafe extern "C" fn cb_frame_start() {
    t(3001);
}
unsafe extern "C" fn cb_frame_end() {
    t(3002);
}
unsafe extern "C" fn cb_match_start() {
    t(3003);
}
unsafe extern "C" fn cb_match_end(o: u8) {
    t(3004);
    t(o as u32);
}
unsafe extern "C" fn cb_unpause(a: i32) {
    t(3005);
    t(a as u32);
}
unsafe extern "C" fn cb_pause(a: i32) {
    t(3006);
    t(a as u32);
}
unsafe extern "C" fn cb_pauser() -> i32 {
    t(3007);
    (rnd() % 6) as i32 - 1
}

fn install_manual() {
    use m::ext::hook as h;
    unsafe {
        h::memzero = Some(|p, n| {
            t(2000);
            t(n as u32);
            core::ptr::write_bytes(p as *mut u8, 0, n as usize);
        });
        h::gm_1601_GetUnkData = Some(|| {
            t(2001);
            addr_of_mut!(LBL)
        });
        h::gm_GetCurrentGameMode = Some(|| {
            t(2002);
            let r = rnd();
            if r % 3 == 0 {
                31
            } else {
                (rnd() % 45) as u8
            }
        });
        h::Ground_801C1DE4 = Some(|a, b| {
            t(2003);
            *a = (rnd() % 4) as i32 - 1;
            *b = (rnd() % 5) as i32;
        });
        h::ifStatus_802F6EA4 = Some(|a, b, c, d, e, f| {
            t(2004);
            t(a as u32);
            t(b as u32);
            t(c as u32);
            t(d as u32);
            t(fnid(e.map(|x| x as usize).unwrap_or(0)));
            t(fnid(f.map(|x| x as usize).unwrap_or(0)));
        });
        h::fn_8017EE40 = Some(|a| {
            t(2005);
            t(meid(a));
        });
        h::fn_8017E8A4 = Some(|a| {
            t(2006);
            t(meid(a));
        });
        h::fn_8018A364 = Some(|a| {
            t(2007);
            t(meid(a));
        });
        h::gm_801B6320 = Some(|| {
            t(2008);
            addr_of_mut!(BYTEX) as *mut c_void
        });
        h::gm_GetAllStarData = Some(|| {
            t(2009);
            addr_of_mut!(ALLSTAR)
        });
        h::gm_GetButtonsTriggered = Some(|a| {
            t(2010);
            t(a as u32);
            let r = rnd();
            if r % 3 == 0 {
                0
            } else {
                (rnd() & 0x1170) as u64
            }
        });
        h::gm_GetButtonsPressed = Some(|a| {
            t(2011);
            t(a as u32);
            let r = rnd();
            if r % 2 != 0 {
                (0x1160 | (rnd() & 0x1170)) as u64
            } else {
                (rnd() & 0x1170) as u64
            }
        });
        h::ifStatus_802F7034 = Some(|f| {
            t(2012);
            t(fnid(f as usize));
        });
        h::fn_8017DEC8 = Some(|a| {
            t(2013);
            t(a as u32);
            addr_of_mut!(UNK1P)
        });
        h::gm_SetDbPauseInputHandlers = Some(|a, b| {
            t(2014);
            t(fnid(a.map(|x| x as usize).unwrap_or(0)));
            t(fnid(b.map(|x| x as usize).unwrap_or(0)));
        });
        h::gm_SetPreGObjProcCallback = Some(|cb| {
            t(2015);
            t(fnid(cb.map(|x| x as usize).unwrap_or(0)));
        });
        h::HSD_GObj_SetupProc = Some(|g, f, p| {
            t(2016);
            t(!g.is_null() as u32);
            t(fnid(f.map(|x| x as usize).unwrap_or(0)));
            t(p as u32);
            if rnd() & 1 != 0 {
                1usize as *mut HSD_GObjProc
            } else {
                null_mut()
            }
        });
        h::Stage_80224E64 = Some(|id, v| {
            t(2017);
            t(id as u32);
            (*v).x = rg();
            (*v).y = rg();
            (*v).z = rg();
        });
        h::Player_GetPlayerSlotType = Some(|slot| {
            t(2020);
            t(slot as u32);
            SLOTT[(slot as u32 % 6) as usize] as Gm_PKind
        });
        h::Player_GetTeam = Some(|slot| {
            t(2021);
            t(slot as u32);
            TEAMS[(slot as u32 % 6) as usize] as i32
        });
        h::Player_GetStocks = Some(|slot| {
            t(2022);
            t(slot as u32);
            STOCKS[(slot as u32 % 6) as usize] as i32
        });
        h::Player_GetPlayerId = Some(|slot| {
            t(2023);
            t(slot as u32);
            PLID[(slot as u32 % 6) as usize] as i32
        });
        h::Player_LoadPlayerCoords = Some(|slot, v| {
            t(2018);
            t(slot as u32);
            (*v).x = rg();
            (*v).y = rg();
            (*v).z = rg();
        });
    }
}

unsafe fn fix_me(me: *mut MatchEnd, _k: u32) {
    fl_MatchEnd(&mut *me);
    let me = &mut *me;
    me.x0 = rnd() % 3;
    for i in 0..6 {
        let p = &mut me.player_standings[i];
        p.pkind = (rnd() % 4) as u8;
        p.team = (rnd() % 5) as u8;
        p.is_big_loser = (rnd() % 6) as u8;
        p.score = (rnd() % 9) as i32 - 4;
        p.x20 = rnd() % 5;
    }
}

unsafe fn fix_rules(r: *mut StartMeleeRules) {
    let r = &mut *r;
    r.stkind = (rnd() % 0x50) as u16;
    r.x9 = (rnd() % 4) as u8;
    r.xD = (rnd() % 4) as u8;
    r.xA = if rnd() % 3 == 0 { rnd() as u8 } else { 0 };
    r.time_limit = rnd() % 8;
    r.game_speed = (1 + rnd() % 8) as f32 * 0.25;
    r.x14 = (rnd() % 5) as u8;
    r.is_teams = (rnd() % 3) as u8;
    X54OBJ.set_x10_b1((rnd() & 1) as u8);
    r.x54 = if rnd() & 1 != 0 { addr_of_mut!(X54OBJ) } else { null_mut() };
    r.on_unpause_override = if rnd() & 1 != 0 { Some(cb_unpause as unsafe extern "C" fn(i32)) } else { None };
    r.on_pause_override = if rnd() & 1 != 0 { Some(cb_pause as unsafe extern "C" fn(i32)) } else { None };
    r.check_for_pauser_override = if rnd() & 1 != 0 { Some(cb_pauser as unsafe extern "C" fn() -> i32) } else { None };
    r.on_match_start = if rnd() & 1 != 0 { Some(cb_match_start as unsafe extern "C" fn()) } else { None };
    r.on_frame_start = if rnd() & 1 != 0 { Some(cb_frame_start as unsafe extern "C" fn()) } else { None };
    r.on_frame_end = if rnd() & 1 != 0 { Some(cb_frame_end as unsafe extern "C" fn()) } else { None };
    r.on_match_end = if rnd() & 1 != 0 { Some(cb_match_end as unsafe extern "C" fn(u8)) } else { None };
}

unsafe fn setup(g: u32, k: u32) {
    for i in 0..6 {
        SLOTT[i] = (rnd() % 5) as u8;
        TEAMS[i] = (rnd() % 3) as u8;
        STOCKS[i] = (rnd() % 4) as u8;
        PLID[i] = (rnd() % 4) as u8;
    }
    if g & G_CTRL != 0 {
        fl_VsSceneController(&mut controller);
        fix_rules(addr_of_mut!(controller.start));
        controller.state.unk_1C = m::lbl_803D5630.as_ptr() as *mut u32;
        controller.state.unk_20 = m::lbl_803D5620.as_ptr() as *mut u32;
        controller.state.timer_seconds = if rnd() & 1 != 0 { rnd() % 8 } else { rnd() };
        controller.state.unk_2C = (rnd() % 64) as u16;
        controller.state.unk_30 = (rnd() % 4) as u8;
        controller.state.unk_0 = (rnd() % 5) as u8;
        controller.state.is_singleplayer = (rnd() % 4) as u8;
        controller.state.match_result = if rnd() % 3 == 0 { 0 } else { (rnd() % 10) as u8 };
        controller.state.terminate_match = (rnd() % 3) as u8;
        controller.state.pauser = (rnd() % 5) as i8 - 1;
        for i in 0..6 {
            controller.state.fighters[i].x1 = (rnd() % 0x54) as u8;
            controller.state.fighters[i].respawn_timer = (rnd() % 3) as u8;
            controller.state.fighters[i].spawn_point = (rnd() % 6) as i8 - 1;
            controller.state.fighters[i].slot_type = (rnd() % 5) as u8;
        }
        fix_me(addr_of_mut!(controller.state.x24C), k);
        controller.start.time_limit = if rnd() & 1 != 0 { controller.state.timer_seconds } else { rnd() % 8 };
    }
    if g & G_SMD != 0 {
        fl_StartMeleeData(&mut SMD);
        fix_rules(addr_of_mut!(SMD.rules));
        for i in 0..6 {
            SMD.players[i].slot_type = (if i < 4 { rnd() % 5 } else { 1 + rnd() % 4 }) as u8;
        }
    }
    if g & G_PIDX != 0 {
        fl_PlayerInitData(&mut PIDX);
        PIDX.slot = (rnd() % 5) as u8;
    }
    if g & G_EMD != 0 {
        fix_me(addr_of_mut!(EMD.xC), k);
        EMD.x0 = rnd();
        EMD.x4 = rnd();
        EMD.x8 = rnd();
    }
    if g & G_PADS != 0 {
        for i in 0..4 {
            HSD_PadCopyStatus[i].err = (rnd() % 2) as i8;
            HSD_PadCopyStatus[i].trigger = if rnd() & 1 != 0 { rnd() & 0x1418 } else { 0 };
            HSD_PadCopyStatus[i].button = if rnd() & 1 != 0 { rnd() & 0x1578 } else { 0 };
            HSD_PadMasterStatus[i].err = (rnd() % 2) as i8;
            HSD_PadMasterStatus[i].trigger = if rnd() & 1 != 0 { rnd() & 0x1418 } else { 0 };
        }
        DbLevel = rnd() % 5;
    }
    if g & G_MISC != 0 {
        PINT = rnd() as i32;
    }
    if g & G_LBL != 0 {
        fl_lbl_8046B488_t(&mut LBL);
    }
    if g & G_ME != 0 {
        fix_me(addr_of_mut!(ME), k);
    }
    if g & G_ALLSTAR != 0 {
        ALLSTAR.x0.x0.ckind = (rnd() % 26) as _;
        BYTEX = (rnd() % 26) as u8;
        UNK1P.x8 = rnd() as _;
    }
    if g & G_RAND != 0 {
        RANDSEED = rnd();
    }
}

unsafe fn state_hash(g: u32) -> u32 {
    let mut h: u32 = 0;
    h = h.wrapping_mul(31).wrapping_add(tu_gmvs_stubs::pool_hash());
    if g & G_CTRL != 0 {
        h = hs_VsSceneController(&controller, h);
    }
    if g & G_SMD != 0 {
        h = hs_StartMeleeData(&SMD, h);
    }
    if g & G_PIDX != 0 {
        h = hs_PlayerInitData(&PIDX, h);
    }
    if g & G_EMD != 0 {
        h = hm(h, EMD.x0);
        h = hm(h, EMD.x4);
        h = hm(h, EMD.x8);
        h = hs_MatchEnd(&EMD.xC, h);
    }
    if g & G_MISC != 0 {
        h = hm(h, PINT as u32);
    }
    if g & G_LBL != 0 {
        h = hs_lbl_8046B488_t(&LBL, h);
    }
    if g & G_ME != 0 {
        h = hs_MatchEnd(&ME, h);
    }
    if g & G_RAND != 0 {
        h = hm(h, RANDSEED);
    }
    h
}

#[test]
fn gmvs_matches_c() {
    tu_gmvs_stubs::install();
    install_manual();
    unsafe {
        HSD_RandSeedPtr = addr_of_mut!(RANDSEED);
        super_globals_init();
    }
    let mut c = Check::new(include_str!("data/tu_gmvs.txt"));
    for op in 1..=tu_gmvs_ops::NOPS {
        let g = tu_gmvs_ops::op_groups(op);
        for k in 0..NCASES {
            case(op, k);
            unsafe {
                tu_gmvs_stubs::pool_reset();
                setup(g, k);
                let ret = tu_gmvs_ops::run_op(op);
                c.line(op, k, ret, state_hash(g));
            }
        }
    }
    assert_eq!(c.done(), tu_gmvs_ops::NOPS * NCASES);
}

unsafe fn super_globals_init() {
    __OSBusClock = 162_000_000;
    melee_modes::tu::globals::gmMainLib_804D3EE0 = addr_of_mut!(SAVEDATA);
}
