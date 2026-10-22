//! tu::gmvsmode vs the C (gen/cref/tu/gmvsmode.c: the whole TU, shipping contraction;
//! regenerate with `python rust/melee_modes/gen/cref/tu/run_tu.py gmvsmode`).
#![allow(static_mut_refs, non_snake_case)]
mod tu_gm_harness;
mod tu_gmvsmode_hs;
use core::ffi::c_void;
use core::ptr::addr_of_mut;
use melee_modes::tu::globals::{gmVsMelee_ApproachData, if_Scene_Prize_EnterData};
use melee_modes::tu::gmvsmelee as vm;
use melee_modes::tu::gmvsmode as m;
use melee_types::*;
use tu_gm_harness::*;
use tu_gmvsmode_hs::*;

static mut GS: GameModeState = unsafe { core::mem::zeroed() };
static mut VSROW: VsModeData = unsafe { core::mem::zeroed() };
static mut START: StartMeleeData = unsafe { core::mem::zeroed() };
static mut MEI: MatchExitInfo = unsafe { core::mem::zeroed() };
static mut SSS: SSSData = unsafe { core::mem::zeroed() };

fn isgs(s: *mut GameModeState) -> u32 {
    (s == unsafe { addr_of_mut!(GS) }) as u32
}

fn install() {
    use m::ext::hook as h;
    unsafe {
        h::gm_GetGameModeStateEnterData = Some(|s| {
            t(1);
            t(isgs(s));
            (*s).info.enter_data
        });
        h::gm_GetGameModeStateExitData = Some(|s| {
            t(2);
            t(isgs(s));
            (*s).info.exit_data
        });
        h::gm_SetNextGameModeStateId = Some(|id| {
            t(3);
            t(id as u32)
        });
        h::gm_SetupRulesDefaults = Some(|r| {
            t(4);
            t((r == addr_of_mut!(START.rules)) as u32);
            (*r).x6 = rnd() as u8;
            (*r).time_limit = rnd();
        });
        h::gm_SetupPlayerDefaults = Some(|p| {
            t(5);
            t(p.offset_from(START.players.as_mut_ptr()) as u32);
            (*p).color = rnd() as u8;
            (*p).stocks = rnd() as i8;
            (*p).cpu_kind = rnd() as u8;
            (*p).ckind = rnd() as i8;
            (*p).slot_type = rnd() as u8;
            (*p).set_rumble_enabled(rnd() as u8);
        });
        h::gm_LoadAnnouncer = Some(|| t(6));
        h::gmFrontend_SelectScene = Some(|s, sss| {
            t(7);
            t(isgs(s));
            t(sss as u32)
        });
        h::gmFrontend_BeginLoading = Some(|| t(8));
        h::gmVsMelee_GetVsData = Some(|| {
            t(9);
            addr_of_mut!(VSROW)
        });
        h::gmVsMelee_EnterCss = Some(|s, v, mt| {
            t(10);
            t(isgs(s));
            t((v == addr_of_mut!(VSROW)) as u32);
            t(mt)
        });
        h::gmVsMelee_ExitCss = Some(|s, v| {
            t(11);
            t(isgs(s));
            t((v == addr_of_mut!(VSROW)) as u32)
        });
        h::gmVsMelee_EnterSss = Some(|s, v| {
            t(12);
            t(isgs(s));
            t((v == addr_of_mut!(VSROW)) as u32)
        });
        h::gmVsMelee_ExitSss = Some(|s, v, id| {
            t(13);
            t(isgs(s));
            t((v == addr_of_mut!(VSROW)) as u32);
            t(id as u32)
        });
        h::gmVsMelee_EnterVs = Some(|s, v, c1, c2| {
            t(14);
            t(isgs(s));
            t((v == addr_of_mut!(VSROW)) as u32);
            t(c1.is_some() as u32);
            t(c2.is_some() as u32)
        });
        h::gmVsMelee_ExitVs = Some(|s, a, b| {
            t(15);
            t(isgs(s));
            t(a as u32);
            t(b as u32)
        });
        h::gmVsMelee_EnterSuddenDeath = Some(|s, v, c1, c2| {
            t(16);
            t(isgs(s));
            t((v == addr_of_mut!(VSROW)) as u32);
            t(c1.is_some() as u32);
            t(c2.is_some() as u32)
        });
        h::gmVsMelee_ExitSuddenDeath = Some(|s| {
            t(17);
            t(isgs(s))
        });
        h::gmVsMelee_EnterResults = Some(|s| {
            t(18);
            t(isgs(s))
        });
        h::gmVsMelee_ExitResults = Some(|s, v, id| {
            t(19);
            t(isgs(s));
            t((v == addr_of_mut!(VSROW)) as u32);
            t(id as u32)
        });
        h::gm_80162A98 = Some(|a| {
            t(20);
            t(a as u32);
            core::ptr::null_mut()
        });
        h::gm_RecordSelfDestructs = Some(|a| {
            t(21);
            t(a as u32)
        });
        h::gm_80162A4C = Some(|a| {
            t(22);
            t(a as u32);
            0
        });
        h::gm_WasMatchCanceled = Some(|o| {
            t(23);
            t(o as u32);
            sr(3)
        });
        h::gm_801623A4 = Some(|e| {
            t(24);
            t((e == addr_of_mut!(vm::gmVsMelee_ResultsEnterData.match_end)) as u32);
            0
        });
        h::gm_ModeState_Approach_OnEnter = Some(|s| {
            t(70);
            t(isgs(s))
        });
        h::gm_ModeState_ApproachVs_OnEnter = Some(|s| {
            t(71);
            t(isgs(s))
        });
        h::gm_ModeState_ApproachVs_OnExit = Some(|s| {
            t(72);
            t(isgs(s))
        });
        h::gm_ModeState_Prize_OnEnter = Some(|s| {
            t(73);
            t(isgs(s))
        });
        h::gm_ModeState_Prize_OnExit = Some(|s| {
            t(74);
            t(isgs(s))
        });
    }
}

unsafe fn who(p: *mut c_void) -> u32 {
    if p.is_null() {
        0
    } else if p == addr_of_mut!(vm::gmVsMelee_CssData) as *mut c_void {
        1
    } else if p == addr_of_mut!(vm::gmVsMelee_SssData) as *mut c_void {
        2
    } else if p == addr_of_mut!(vm::gmVsMelee_StartData) as *mut c_void {
        3
    } else if p == addr_of_mut!(vm::gmVsMelee_VsExitInfo) as *mut c_void {
        4
    } else if p == addr_of_mut!(vm::gmVsMelee_SuddenDeathExitInfo) as *mut c_void {
        5
    } else if p == addr_of_mut!(vm::gmVsMelee_ResultsEnterData) as *mut c_void {
        6
    } else if p == addr_of_mut!(gmVsMelee_ApproachData) as *mut c_void {
        7
    } else if p == addr_of_mut!(if_Scene_Prize_EnterData) as *mut c_void {
        8
    } else {
        99
    }
}

unsafe fn table(tb: *mut GameModeState) {
    let mut i = 0usize;
    loop {
        let e = &*tb.add(i);
        t(e.id as u32);
        t(e.preload as u32);
        t(e.flags as u32);
        t(e.on_enter.is_some() as u32);
        t(e.on_exit.is_some() as u32);
        t(e.info.scene_kind as u32);
        t(who(e.info.enter_data));
        t(who(e.info.exit_data));
        zero(&mut GS);
        GS.info.enter_data = addr_of_mut!(START) as *mut c_void;
        GS.info.exit_data = addr_of_mut!(MEI) as *mut c_void;
        if let Some(f) = e.on_enter {
            t(1000 + i as u32);
            f(addr_of_mut!(GS));
        }
        if let Some(f) = e.on_exit {
            t(2000 + i as u32);
            f(addr_of_mut!(GS));
        }
        if e.id == 255 {
            break;
        }
        i += 1;
    }
}

unsafe fn state_hash() -> u32 {
    let mut h = 0;
    h = hs_StartMeleeData(&START, h);
    h = hs_MatchExitInfo(&MEI, h);
    h = hs_SSSData(&SSS, h);
    h = hs_ResultsMatchInfo(&vm::gmVsMelee_ResultsEnterData, h);
    h
}

#[test]
fn gmvsmode_matches_c() {
    install();
    let mut c = Check::new(include_str!("data/tu_gmvsmode.txt"));
    unsafe {
        case(0, 0);
        zero(&mut MEI);
        zero(&mut vm::gmVsMelee_ResultsEnterData);
        table(m::gm_Mode_Vs_States.as_mut_ptr());
        t(0xDEB);
        table(m::gm_Mode_DebugVs_States.as_mut_ptr());
        c.line_tag("T", 0, 0);
    }
    for op in 1..=13u32 {
        for k in 0..600u32 {
            case(op, k);
            unsafe {
                zero(&mut GS);
                zero(&mut START);
                zero(&mut MEI);
                zero(&mut SSS);
                zero(&mut vm::gmVsMelee_ResultsEnterData);
                fl_StartMeleeData(&mut START);
                fl_MatchExitInfo(&mut MEI);
                fl_SSSData(&mut SSS);
                fl_ResultsMatchInfo(&mut vm::gmVsMelee_ResultsEnterData);
                for i in 0..6 {
                    MEI.match_end.player_standings[i].pkind = (rnd() % 5) as u8;
                }
                SSS.start_game = (rnd() & 1) as u8;
                GS.info.enter_data = addr_of_mut!(START) as *mut c_void;
                GS.info.exit_data = addr_of_mut!(MEI) as *mut c_void;
                let gs = addr_of_mut!(GS);
                match op {
                    1 => m::onEnterDebugVs(gs),
                    2 => m::onEnterCss(gs),
                    3 => m::onExitCss(gs),
                    4 => m::onEnterSss(gs),
                    5 => {
                        GS.info.exit_data = addr_of_mut!(SSS) as *mut c_void;
                        m::onExitSss(gs)
                    }
                    6 => m::onEnterLoading(gs),
                    7 => m::onExitLoading(gs),
                    8 => m::onEnterVs(gs),
                    9 => m::onExitVs(gs),
                    10 => m::onEnterSuddenDeath(gs),
                    11 => m::onExitSuddenDeath(gs),
                    12 => m::onEnterResults(gs),
                    _ => {
                        vm::gmVsMelee_ResultsEnterData.match_end.outcome = (rnd() % 9) as u8;
                        m::onExitResults(gs)
                    }
                }
                c.line(op, k, 0, state_hash());
            }
        }
    }
    assert_eq!(c.done(), 1 + 13 * 600);
}
