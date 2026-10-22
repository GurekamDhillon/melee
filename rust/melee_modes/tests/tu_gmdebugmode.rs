//! src/tu/gmdebugmode.rs vs the C (gen/cref/tu/gmdebugmode.c: the whole TU, shipping
//! contraction; regenerate with `python rust/melee_modes/gen/cref/run_tu.py gmdebugmode`).
#![allow(static_mut_refs, non_snake_case)]
mod mtu_harness;
use melee_modes::tu::gmdebugmode as m;
use melee_types::*;
use mtu_harness::*;
#[path = "mtu/gmdebugmode_gen.rs"]
mod g;

static mut ST: GameModeState = unsafe { core::mem::zeroed() };

fn reg<T>(p: *const T) {
    preg(p, core::mem::size_of::<T>());
}

fn zero<T>(p: *mut T) {
    unsafe { core::ptr::write_bytes(p, 0, 1) }
}

unsafe fn hash_all() {
    g::hash_enterMenu0_Data(&m::menu_enter_data);
    g::hash_ResultsMatchInfo(&m::results0_data);
    g::hash_StartMeleeData(&m::vs_enter_data);
    g::hash_MatchExitInfo(&m::vs_exit_data);
    g::hash_DebugGameOverData(&m::game_over_data);
    g::hash_DebugResultsData(&m::results1_enter_data);
    g::hash_exitIntroData(&m::intro_easy_exit_data);
    hm(rh(&m::approach_data));
    g::hash_un_80301C64_t(&m::intro_allstar_enter_data);
    g::hash_exitIntroData(&m::intro_allstar_exit_data);
    g::hash_DebugMemcardData(&m::memcard_enter_data);
    g::hash_DebugMemcardData(&m::memcard_exit_data);
    hm(rh(&m::intro_easy_enter_data));
    hm(rh(&m::prize_enter_data));
}

#[test]
fn gmdebugmode_matches_c() {
    unsafe {
        g::install();
        reg(&raw const ST);
        reg(&raw const m::gm_Mode_Debug_States);
        reg(&raw const m::intro_easy_enter_data);
        reg(&raw const m::vs_enter_data);
        reg(&raw const m::vs_exit_data);
        reg(&raw const m::results0_data);
        reg(&raw const m::prize_enter_data);
        reg(&raw const m::game_over_data);
        reg(&raw const m::results1_enter_data);
        reg(&raw const m::intro_easy_exit_data);
        reg(&raw const m::intro_allstar_enter_data);
        reg(&raw const m::intro_allstar_exit_data);
        reg(&raw const m::menu_enter_data);
        reg(&raw const m::approach_data);
        reg(&raw const m::memcard_enter_data);
        reg(&raw const m::memcard_exit_data);
        reg(&raw const m::ext::un_803FA258);
        reg(&raw const m::ext::un_803FA4E0);
        reg(&raw const m::ext::un_803FA790);
        reg(&raw const m::ext::un_803FC4CC);
        g::freg_all();
        let mut c = Check::new(include_str!("data/mtu_gmdebugmode.txt"));
        for op in 0..20u32 {
            for k in 0..400u32 {
                case(op, k);
                zero(&raw mut ST);
                zero(&raw mut m::menu_enter_data);
                zero(&raw mut m::approach_data);
                zero(&raw mut m::intro_easy_exit_data);
                zero(&raw mut m::intro_allstar_enter_data);
                zero(&raw mut m::intro_allstar_exit_data);
                zero(&raw mut m::memcard_enter_data);
                zero(&raw mut m::memcard_exit_data);
                zero(&raw mut m::intro_easy_enter_data);
                zero(&raw mut m::prize_enter_data);
                g::fill_ResultsMatchInfo(&mut m::results0_data);
                g::fill_StartMeleeData(&mut m::vs_enter_data);
                g::fill_MatchExitInfo(&mut m::vs_exit_data);
                g::fill_DebugGameOverData(&mut m::game_over_data);
                g::fill_DebugResultsData(&mut m::results1_enter_data);
                g::fill_HSD_PadStatus(&mut m::ext::HSD_PadCopyStatus[0]);
                m::ext::HSD_PadCopyStatus[0].button = rnd();
                for i in 0..128 {
                    m::ext::un_803FA258[i] = fv() as i32;
                }
                let mut ret = 0u32;
                let st = &raw mut ST;
                match op {
                    0 => m::onExitIntro(st),
                    1 => {
                        ST.info.enter_data = &raw mut m::menu_enter_data as *mut _;
                        m::onEnterMenu0(st)
                    }
                    2 => ret = m::fn_801B09F8((fv() % 3) as i32) as u32,
                    3 => {
                        ST.info.enter_data = &raw mut m::menu_enter_data as *mut _;
                        m::onEnterMenu1(st)
                    }
                    4 => ret = m::fn_801B0A8C((fv() % 3) as i32) as u32,
                    5 => {
                        ST.info.enter_data = &raw mut m::menu_enter_data as *mut _;
                        m::onEnterMenu2(st)
                    }
                    6 => {
                        ST.info.enter_data = &raw mut m::prize_enter_data as *mut _;
                        m::onEnterPrize(st)
                    }
                    7 => m::onExitPrize(st),
                    8 => {
                        ST.info.enter_data = &raw mut m::vs_enter_data as *mut _;
                        m::onEnterVs(st)
                    }
                    9 => {
                        ST.info.enter_data = &raw mut m::results0_data as *mut _;
                        m::onEnterResults0(st)
                    }
                    10 => {
                        ST.info.exit_data = &raw mut m::results0_data as *mut _;
                        m::onExitResults0(st)
                    }
                    11 => {
                        ST.info.enter_data = &raw mut m::intro_easy_enter_data as *mut _;
                        m::onEnterIntroEasy(st)
                    }
                    12 => {
                        ST.info.enter_data = &raw mut m::intro_allstar_enter_data as *mut _;
                        m::onEnterIntroAllstar(st)
                    }
                    13 => {
                        ST.info.enter_data = &raw mut m::game_over_data as *mut _;
                        m::onEnterGameOver(st)
                    }
                    14 => {
                        ST.info.enter_data = &raw mut m::approach_data as *mut _;
                        m::onEnterApproach(st)
                    }
                    15 => {
                        ST.info.enter_data = &raw mut m::results1_enter_data as *mut _;
                        m::onEnterResults1(st)
                    }
                    16 => m::onExitResults1(st),
                    17 => {
                        ST.info.enter_data = &raw mut m::memcard_enter_data as *mut _;
                        m::onEnterMemCard(st)
                    }
                    18 => {
                        ST.info.exit_data = &raw mut m::memcard_exit_data as *mut _;
                        m::onExitMemCard(st)
                    }
                    _ => {
                        for i in 0..16 {
                            g::hash_GameModeState(&m::gm_Mode_Debug_States[i]);
                        }
                    }
                }
                hash_all();
                g::hash_HSD_PadStatus(&m::ext::HSD_PadCopyStatus[0]);
                for i in 0..128 {
                    hm(m::ext::un_803FA258[i] as u32);
                }
                c.line(op, k, ret);
            }
        }
        assert_eq!(c.done(), 20 * 400);
    }
}
