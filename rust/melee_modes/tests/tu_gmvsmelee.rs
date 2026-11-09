//! tu::gmvsmelee vs the C (gen/cref/tu/gmvsmelee.c: the whole TU, shipping contraction;
//! regenerate with `python rust/melee_modes/gen/cref/tu/run_tu.py gmvsmelee`).
#![allow(static_mut_refs, non_snake_case)]
mod tu_gm_harness;
mod tu_gmvsmelee_hs;
use core::ptr::{addr_of, addr_of_mut, null_mut};
use melee_modes::tu::globals::gmMainLib_804D3EE0;
use melee_modes::tu::gmvsmelee as m;
use melee_types::*;
use tu_gm_harness::*;
use tu_gmvsmelee_hs::*;

static mut VSARG: VsModeData = unsafe { core::mem::zeroed() };
static mut VS: *mut VsModeData = null_mut();
static mut CSS: CSSData = unsafe { core::mem::zeroed() };
static mut SSS: SSSData = unsafe { core::mem::zeroed() };
static mut START: StartMeleeData = unsafe { core::mem::zeroed() };
static mut MEI: MatchExitInfo = unsafe { core::mem::zeroed() };
static mut PRE: PreloadedGameModeState = unsafe { core::mem::zeroed() };
static mut R98: gmm_retval_ED98 = unsafe { core::mem::zeroed() };
static mut RB0: gmm_retval_EDB0 = unsafe { core::mem::zeroed() };
static mut GS: [GameModeState; 2] = unsafe { core::mem::zeroed() };

unsafe fn row() -> *mut VsModeData {
    &mut (*gmMainLib_804D3EE0).modes.table[GmVsMode_Melee as usize]
}
fn t64(v: u64) {
    t(v as u32);
    t((v >> 32) as u32);
}
fn strh(s: &str) -> u32 {
    hbuf(s.as_ptr(), s.len())
}
unsafe fn whoend(e: *mut MatchEnd) -> u32 {
    if e == addr_of_mut!(m::gmVsMelee_VsExitInfo.match_end) {
        1
    } else if e == addr_of_mut!(m::gmVsMelee_SuddenDeathExitInfo.match_end) {
        2
    } else if e == addr_of_mut!(MEI.match_end) {
        3
    } else if e == addr_of_mut!(m::gmVsMelee_ResultsEnterData.match_end) {
        4
    } else {
        0
    }
}
unsafe fn whovs(v: *mut VsModeData) -> u32 {
    if v == row() {
        1
    } else if v == addr_of_mut!(VSARG) {
        2
    } else {
        0
    }
}

unsafe extern "C" fn cbS(s: *mut StartMeleeData, v: *mut StartMeleeData) {
    t(60);
    t((s == addr_of_mut!(START)) as u32);
    t((v == addr_of_mut!((*VS).start)) as u32);
    t(hs_StartMeleeData(&*v, 0));
    (*s).rules.x6 = rnd() as u8;
}
unsafe extern "C" fn cbP(a: *mut PlayerInitData, b: *mut PlayerInitData) {
    t(61);
    let ai = a.offset_from(START.players.as_mut_ptr());
    t(ai as u32);
    t((b == (*VS).start.players.as_mut_ptr().offset(ai)) as u32);
    (*a).nametag = rnd() as u8;
    (*b).team = rnd() as u8;
}

fn install() {
    use m::ext::hook as h;
    unsafe {
        h::Mex_Enabled = Some(|s| {
            t(100);
            t(strh(s));
            sr(0) as i32
        });
        h::OSReport = Some(|s| {
            t(101);
            t(strh(s))
        });
        h::memzero = Some(|p, n| {
            t(102);
            t((p as *mut u8 == m::ko_counts.as_mut_ptr()) as u32);
            t(n as u32);
            core::ptr::write_bytes(p as *mut u8, 0, n as usize);
        });
        h::lbTime_8000AF74 = Some(|a, b| {
            t(103);
            t(a);
            t(b as u32);
            rnd()
        });
        h::SceneLaunch_Active = Some(|| {
            t(104);
            sr(1) as i32
        });
        h::SceneLaunch_EntryStateId = Some(|| {
            t(105);
            if sr(2) {
                2
            } else {
                (rnd() % 3) as i32
            }
        });
        h::SceneLaunch_PlayerStocks = Some(|slot| {
            t(106);
            t(slot as u32);
            (rnd() % 5) as i32 - 1
        });
        h::SceneLaunch_HudCount = Some(|s| {
            t(107);
            t((s == addr_of_mut!(START)) as u32);
            t(hs_StartMeleeData(&*s, 0));
        });
        h::SceneLaunch_SeedVs = Some(|v, d| {
            t(108);
            t((v == row()) as u32);
            t(d as u32);
            sr(3)
        });
        h::gm_GetGameModeStateEnterData = Some(|s| {
            t(1);
            t((s == addr_of_mut!(GS[0])) as u32);
            (*s).info.enter_data
        });
        h::gm_GetGameModeStateExitData = Some(|s| {
            t(2);
            t((s == addr_of_mut!(GS[0])) as u32);
            (*s).info.exit_data
        });
        h::gm_ChangeGameModeAfterCurrentScene = Some(|x| {
            t(3);
            t(x as u32)
        });
        h::gm_SetNextGameModeStateId = Some(|id| {
            t(4);
            t(id as u32)
        });
        h::gm_InitVsMode = Some(|v| {
            t(5);
            t((v == row()) as u32);
            t(hs_VsModeData(&*v, 0));
        });
        h::gmMainLib_8015CDEC = Some(|| t(6));
        h::lbDvd_GetPreloadCacheScene = Some(|| {
            t(7);
            addr_of_mut!(PRE)
        });
        h::lbDvd_SetupVsPreloadCache = Some(|| t(8));
        h::Frontend_OnlinePick = Some(|| {
            t(9);
            (rnd() % 3) as i32
        });
        h::Frontend_OnlinePicked = Some(|w, a, b| {
            t(10);
            t(w as u32);
            t(a as u32);
            t(b as u32)
        });
        h::Frontend_BackToOnline = Some(|| t(11));
        h::lbAudioAx_80026E84 = Some(|ck| {
            t(12);
            t(ck);
            let lo = rnd() as u64;
            let hi = rnd() as u64;
            (hi << 32) | lo
        });
        h::lbAudioAx_80026F2C = Some(|a| {
            t(13);
            t(a)
        });
        h::lbAudioAx_8002702C = Some(|f, mask| {
            t(14);
            t(f);
            t64(mask)
        });
        h::lbAudioAx_80027168 = Some(|| t(15));
        h::lbAudioAx_80026EBC = Some(|sk| {
            t(16);
            t(sk);
            let lo = rnd() as u64;
            let hi = rnd() as u64;
            (hi << 32) | lo
        });
        h::gm_80167FC4 = Some(|s| {
            t(17);
            t((s == addr_of_mut!(SSS)) as u32);
            t(hs_SSSData(&*s, 0));
        });
        h::gm_80167BC8 = Some(|v| {
            t(18);
            t(whovs(v));
            t(hs_VsModeData(&*v, 0));
            (*v).start.rules.stkind = rnd() as u16;
        });
        h::gm_SetupSubColors = Some(|s| {
            t(19);
            t((s == addr_of_mut!(START)) as u32);
            t(hs_StartMeleeData(&*s, 0));
        });
        h::gm_LoadRumbleEnabled = Some(|s| {
            t(20);
            t((s == addr_of_mut!(START)) as u32)
        });
        h::gm_LoadAnnouncer = Some(|| t(21));
        h::Netplay_StageEnd = Some(|w, s0, s1| {
            t(22);
            t(w as u32);
            t(s0 as u32);
            t(s1 as u32)
        });
        h::Netplay_RunActive = Some(|| {
            t(23);
            (sr(4) && sr(5)) as i32
        });
        h::Netplay_GameResult = Some(|w| {
            t(24);
            t(w as u32)
        });
        h::Netplay_Enabled = Some(|| {
            t(25);
            sr(6) as i32
        });
        h::Netplay_NoteTie = Some(|| t(26));
        h::Netplay_RematchPending = Some(|| {
            t(27);
            (sr(7) && sr(8)) as i32
        });
        h::gm_WasMatchCanceled = Some(|o| {
            t(28);
            t(o as u32);
            sr(9)
        });
        h::gm_MatchHasMultipleWinners = Some(|e| {
            t(29);
            t(whoend(e));
            t(hs_MatchEnd(&*e, 0));
            sr(10)
        });
        h::gm_80162574 = Some(|ck, o| {
            t(30);
            t(ck as u32);
            t(o as u32)
        });
        h::gm_SetupHumanResultsScreen = Some(|mk, o| {
            t(31);
            t(mk as u32);
            t(o as u32)
        });
        h::gm_SetupResultsScreenPlayTime = Some(|s, a| {
            t(32);
            t(s);
            t(a)
        });
        h::gm_80162800 = Some(|e| {
            t(33);
            t(whoend(e));
            rnd()
        });
        h::gm_SetupSuddenDeath = Some(|s, e| {
            t(34);
            t((s == addr_of_mut!(START)) as u32);
            t(hs_StartMeleeData(&*s, 0));
            t(whoend(e));
            t(hs_MatchEnd(&*e, 0));
        });
        h::gm_80166CCC = Some(|a, b| {
            t(35);
            t(whoend(a));
            t(whoend(b));
            t(hs_MatchEnd(&*a, 0));
            t(hs_MatchEnd(&*b, 0));
        });
        h::gm_80177724 = Some(|r| {
            t(36);
            t((r == addr_of_mut!(m::gmVsMelee_ResultsEnterData)) as u32);
            t(hs_ResultsMatchInfo(&*r, 0));
        });
        h::gm_80168638 = Some(|e| {
            t(37);
            t(whoend(e))
        });
        h::gm_80168710 = Some(|e, v| {
            t(38);
            t(whoend(e));
            t(whovs(v))
        });
        h::gm_8016247C = Some(|a| {
            t(39);
            t(a as u32);
            0
        });
        h::gm_801688AC = Some(|e| {
            t(40);
            t(whoend(e));
            rnd() as i32
        });
        h::gm_GetVsPlayMatchTotal = Some(|| {
            t(41);
            rnd()
        });
        h::gm_80172DD4 = Some(|a| {
            t(42);
            t(a);
            if sr(11) {
                ChKind_None as u8
            } else {
                rnd() as u8
            }
        });
        h::gmMainLib_8015ED98 = Some(|| {
            t(43);
            addr_of_mut!(R98)
        });
        h::gm_InitChallengerData = Some(|a, b, c, d, e, f| {
            t(44);
            t(a as u32);
            t(b as u32);
            t(c as u32);
            t(d as u32);
            t(e as u32);
            t(f as u32)
        });
        h::gm_80172D78 = Some(|| {
            t(45);
            if sr(12) {
                ChKind_None as u8
            } else {
                rnd() as u8
            }
        });
        h::gm_80172E74 = Some(|| {
            t(46);
            if sr(13) {
                ChKind_None as u8
            } else {
                rnd() as u8
            }
        });
        h::gm_80172F00 = Some(|a| {
            t(47);
            t(a);
            if sr(14) {
                328
            } else {
                rnd() as u16
            }
        });
        h::gmMainLib_8015EDB0 = Some(|| {
            t(48);
            addr_of_mut!(RB0)
        });
        h::gm_80164504 = Some(|a| {
            t(49);
            t(a as u32)
        });
        h::gm_80173DE4 = Some(|e| {
            t(50);
            t(whoend(e))
        });
        h::gm_80172898 = Some(|a| {
            t(51);
            t(a as u32);
            0
        });
        h::gm_80173EEC = Some(|| t(52));
        h::gm_801721EC = Some(|| {
            t(53);
            sr(15)
        });
        h::lbCardNew_AllocWorkArea = Some(|| t(54));
        h::lbCardGame_LoadArchive = Some(|a| {
            t(55);
            t(a as u32)
        });
        h::lbCardGame_SaveChanges = Some(|| t(56));
    }
}

unsafe fn fix_end(e: *mut MatchEnd) {
    for i in 0..6 {
        let p = &mut (*e).player_standings[i];
        p.pkind = (rnd() % 5) as u8;
        p.is_small_loser = (rnd() % 200) as u8;
        p.team = (rnd() % 5) as u8;
        p.x3_u.x0.set_x3_b0((rnd() & 63) as u8);
    }
    if rnd() & 1 != 0 {
        (*e).player_standings[4].pkind = 3;
        (*e).player_standings[5].pkind = 3;
    }
    for i in 0..5 {
        (*e).team_standings[i].is_small_loser = (rnd() % 200) as u8;
    }
    (*e).is_teams = (rnd() % 3) as u8;
    (*e).outcome = (rnd() % 9) as u8;
    (*e).n_winners = (rnd() % 3) as u8;
    (*e).winners[0] = (rnd() % 4) as u8;
    (*e).frame_count = rnd() % 100000;
}
unsafe fn fix_vs(v: *mut VsModeData) {
    for i in 0..6 {
        let p = &mut (*v).start.players[i];
        p.slot_type = (rnd() % 5) as u8;
        p.ckind = if rnd() % 5 == 0 { rnd() as i8 } else { (rnd() % 0x28) as i8 };
    }
    if rnd() & 1 != 0 {
        (*v).start.players[4].slot_type = 3;
        (*v).start.players[5].slot_type = 3;
    }
    (*v).start.rules.stkind = (rnd() % 40) as u16;
    (*v).start.rules.set_match_kind(rnd() % 4);
}

unsafe fn state_hash() -> u32 {
    let mut h = 0;
    h = hs_VsModeData(&*row(), h);
    h = hs_VsModeData(&VSARG, h);
    h = hs_CSSData(&CSS, h);
    h = hs_SSSData(&SSS, h);
    h = hs_StartMeleeData(&START, h);
    h = hs_MatchExitInfo(&MEI, h);
    h = hs_MatchExitInfo(&m::gmVsMelee_VsExitInfo, h);
    h = hs_MatchExitInfo(&m::gmVsMelee_SuddenDeathExitInfo, h);
    h = hs_ResultsMatchInfo(&m::gmVsMelee_ResultsEnterData, h);
    h = hs_PreloadedGameModeState(&PRE, h);
    hbuf(m::ko_counts.as_ptr(), 6) ^ h.wrapping_mul(31)
}

#[test]
fn gmvsmelee_matches_c() {
    install();
    let save: &'static mut gmm_x0 = Box::leak(Box::new(unsafe { core::mem::zeroed::<gmm_x0>() }));
    let mut c = Check::new(include_str!("data/tu_gmvsmelee.txt"));
    for op in 1..=19u32 {
        for k in 0..1200u32 {
            case(op, k);
            let mut ret = 0u32;
            unsafe {
                *save = core::mem::zeroed();
                zero(&mut VSARG);
                zero(&mut CSS);
                zero(&mut SSS);
                zero(&mut START);
                zero(&mut MEI);
                zero(&mut PRE);
                zero(&mut R98);
                zero(&mut RB0);
                zero(&mut GS);
                zero(&mut m::gmVsMelee_CssData);
                zero(&mut m::gmVsMelee_SssData);
                zero(&mut m::gmVsMelee_StartData);
                zero(&mut m::gmVsMelee_VsExitInfo);
                zero(&mut m::gmVsMelee_SuddenDeathExitInfo);
                zero(&mut m::gmVsMelee_ResultsEnterData);
                gmMainLib_804D3EE0 = save;
                fl_VsModeData(&mut *row());
                fl_VsModeData(&mut VSARG);
                fl_CSSData(&mut CSS);
                fl_SSSData(&mut SSS);
                fl_StartMeleeData(&mut START);
                fl_MatchExitInfo(&mut MEI);
                fl_MatchExitInfo(&mut m::gmVsMelee_VsExitInfo);
                fl_MatchExitInfo(&mut m::gmVsMelee_SuddenDeathExitInfo);
                fl_ResultsMatchInfo(&mut m::gmVsMelee_ResultsEnterData);
                fl_PreloadedGameModeState(&mut PRE);
                fl_gmm_retval_ED98(&mut R98);
                fl_gmm_retval_EDB0(&mut RB0);
                for i in 0..6 {
                    m::ko_counts[i] = rnd() as u8;
                }
                let mut kc = [0u8; 6];
                for i in 0..6 {
                    kc[i] = rnd() as u8;
                }
                fix_vs(row());
                fix_vs(addr_of_mut!(VSARG));
                fix_end(addr_of_mut!(MEI.match_end));
                fix_end(addr_of_mut!(m::gmVsMelee_VsExitInfo.match_end));
                fix_end(addr_of_mut!(m::gmVsMelee_SuddenDeathExitInfo.match_end));
                fix_end(addr_of_mut!(m::gmVsMelee_ResultsEnterData.match_end));
                VS = if rnd() & 1 != 0 { row() } else { addr_of_mut!(VSARG) };
                CSS.pending_scene_change = if rnd() % 3 == 0 { 2 } else { rnd() as u8 };
                SSS.start_game = (rnd() & 1) as u8;
                GS[0].info.enter_data = addr_of_mut!(CSS) as *mut _;
                GS[0].info.exit_data = addr_of_mut!(CSS) as *mut _;
                GS[1].id = if rnd() % 3 == 0 { 255 } else { (rnd() % 200) as u8 };
                let gs = addr_of_mut!(GS[0]);
                match op {
                    1 => t((m::gmVsMelee_GetVsData() == row()) as u32),
                    2 => t((m::gmVsMelee_GetKOCounts() == m::ko_counts.as_mut_ptr()) as u32),
                    3 => {
                        m::gmVsMelee_UpdateKOCounts(kc.as_mut_ptr(), addr_of_mut!(MEI.match_end));
                        t(hbuf(kc.as_ptr(), 6));
                    }
                    4 => ret = m::gmVsMelee_WasAnyPlayerHuman(&MEI.match_end) as u32,
                    5 => {
                        MEI.match_end.player_standings[(rnd() % 6) as usize].pkind = 0;
                        ret = m::findSmallestLoser(addr_of_mut!(MEI.match_end)) as u32;
                    }
                    6 => m::gmVsMelee_Mode_OnInit(),
                    7 => m::gmVsMelee_ResetKOCounts(),
                    8 => m::gmVsMelee_Mode_OnLoad(),
                    9 => m::gm_Mode_Vs_OnUnload(),
                    10 => {
                        let mt = rnd() % 3;
                        m::gmVsMelee_EnterCss(gs, VS, mt)
                    }
                    11 => m::gmVsMelee_ExitCss(gs, VS),
                    12 => {
                        GS[0].info.enter_data = addr_of_mut!(SSS) as *mut _;
                        m::gmVsMelee_EnterSss(gs, VS)
                    }
                    13 => {
                        GS[0].info.exit_data = addr_of_mut!(SSS) as *mut _;
                        let id = rnd() as u8;
                        m::gmVsMelee_ExitSss(gs, VS, id)
                    }
                    14 => {
                        GS[0].info.enter_data = addr_of_mut!(START) as *mut _;
                        let c1: m::gm_StartMeleeCallback = if rnd() % 3 != 0 { Some(cbS) } else { None };
                        let c2: m::gm_PlayerInitCallback = if rnd() % 3 != 0 { Some(cbP) } else { None };
                        m::gmVsMelee_EnterVs(gs, VS, c1, c2)
                    }
                    15 => {
                        GS[0].info.exit_data = addr_of_mut!(MEI) as *mut _;
                        let a = rnd() as u8;
                        let b = rnd() as u8;
                        m::gmVsMelee_ExitVs(gs, a, b)
                    }
                    16 => {
                        GS[0].info.enter_data = addr_of_mut!(START) as *mut _;
                        let c1: m::gm_StartMeleeCallback = if rnd() % 3 != 0 { Some(cbS) } else { None };
                        let c2: m::gm_PlayerInitCallback = if rnd() % 3 != 0 { Some(cbP) } else { None };
                        m::gmVsMelee_EnterSuddenDeath(gs, VS, c1, c2)
                    }
                    17 => {
                        GS[0].info.exit_data = addr_of_mut!(MEI) as *mut _;
                        m::gmVsMelee_ExitSuddenDeath(gs)
                    }
                    18 => {
                        GS[0].info.enter_data = addr_of_mut!(m::gmVsMelee_ResultsEnterData) as *mut _;
                        m::gmVsMelee_EnterResults(gs)
                    }
                    _ => {
                        let id = rnd() as u8;
                        m::gmVsMelee_ExitResults(gs, VS, id)
                    }
                }
                c.line(op, k, ret, state_hash());
            }
        }
    }
    assert_eq!(c.done(), 19 * 1200);
    let _ = addr_of!(GS);
}
