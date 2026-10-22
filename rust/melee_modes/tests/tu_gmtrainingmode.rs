//! tu::gmtrainingmode vs the C (gen/cref/tu/gmtrainingmode.c: the whole TU, shipping contraction;
//! regenerate with `python rust/melee_modes/gen/cref/tu/run_tu.py gmtrainingmode`).
#![allow(static_mut_refs, non_snake_case)]
mod tu_gm_harness;
mod tu_gmtrainingmode_hs;
use melee_modes::tu::globals::{gm_80473814, gmMainLib_804D3EE0};
use melee_modes::tu::gmtrainingmode as m;
use melee_types::*;
use tu_gm_harness::*;
use tu_gmtrainingmode_hs::*;

static mut GS: GameModeState = unsafe { core::mem::zeroed() };
static mut CSS: CSSData = unsafe { core::mem::zeroed() };
static mut SSS: SSSData = unsafe { core::mem::zeroed() };
static mut SMD: StartMeleeData = unsafe { core::mem::zeroed() };
static mut MEI: MatchExitInfo = unsafe { core::mem::zeroed() };
static mut PRE: PreloadedGameModeState = unsafe { core::mem::zeroed() };

unsafe fn vsrow() -> *mut VsModeData {
    &mut (*gmMainLib_804D3EE0).modes.table[GmVsMode_Training as usize]
}
fn isgs(s: *mut GameModeState) -> u32 {
    (s == unsafe { core::ptr::addr_of_mut!(GS) }) as u32
}
fn t64(v: u64) {
    t(v as u32);
    t((v >> 32) as u32);
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
        h::lbCardNew_AllocWorkArea = Some(|| t(3));
        h::lbCardGame_LoadArchive = Some(|a| {
            t(4);
            t(a as u32)
        });
        h::lbCardGame_SaveChanges = Some(|| t(5));
        h::gm_801B06B0 = Some(|c, mt, ck, st, col, nt, lvl, slot| {
            t(6);
            t((c == core::ptr::addr_of_mut!(CSS)) as u32);
            t(hs_CSSData(&*c, 0));
            t(mt as u32);
            t(ck as i32 as u32);
            t(st as i32 as u32);
            t(col as i32 as u32);
            t(nt as u32);
            t(lvl as u32);
            t(slot as u32);
        });
        h::gm_801B07B4 = Some(|c, ck, st, col, nt, lvl, slot| {
            t(7);
            t((c == core::ptr::addr_of_mut!(CSS)) as u32);
            t(hs_CSSData(&*c, 0));
            t(ck as i32 as u32);
            t(st as i32 as u32);
            t(col as i32 as u32);
            t(nt as u32);
            t(lvl as u32);
            t(slot as u32);
        });
        h::lbDvd_SetupVsPreloadCache = Some(|| t(8));
        h::lbTime_8000AF74 = Some(|a, b| {
            t(39);
            t(a);
            t(b as u32);
            rnd()
        });
        h::gmFrontend_TrainingSelect = Some(|s, sss| {
            t(9);
            t(isgs(s));
            t(sss as u32)
        });
        h::gm_801B0730 = Some(|c, ck, st, col, nt, lvl| {
            t(10);
            t((c == core::ptr::addr_of_mut!(CSS)) as u32);
            t(hs_CSSData(&*c, 0));
            t((!ck.is_null()) as u32);
            t((!st.is_null()) as u32);
            t((!col.is_null()) as u32);
            t((!nt.is_null()) as u32);
            t((!lvl.is_null()) as u32);
            if !ck.is_null() {
                *ck = rnd() as i8
            }
            if !st.is_null() {
                *st = rnd() as u8
            }
            if !col.is_null() {
                *col = rnd() as u8
            }
            if !nt.is_null() {
                *nt = rnd() as u8
            }
            if !lvl.is_null() {
                *lvl = rnd() as u8
            }
        });
        h::gm_801B07E8 = Some(|c, ck, st, col, a4, lvl| {
            t(11);
            t((c == core::ptr::addr_of_mut!(CSS)) as u32);
            t(hs_CSSData(&*c, 0));
            t((!ck.is_null()) as u32);
            t((!st.is_null()) as u32);
            t((!col.is_null()) as u32);
            t((!a4.is_null()) as u32);
            t((!lvl.is_null()) as u32);
            if !ck.is_null() {
                *ck = rnd() as i8
            }
            if !st.is_null() {
                *st = rnd() as i8
            }
            if !col.is_null() {
                *col = rnd() as i8
            }
            if !a4.is_null() {
                *a4 = rnd() as i8
            }
            if !lvl.is_null() {
                *lvl = rnd() as u8
            }
        });
        h::gm_ChangeGameModeAfterCurrentScene = Some(|m| {
            t(12);
            t(m as u32)
        });
        h::gm_SetupAllPlayerDefaults = Some(|p| {
            t(13);
            t(if p == (*vsrow()).start.players.as_mut_ptr() {
                1
            } else if p == SMD.players.as_mut_ptr() {
                2
            } else {
                0
            });
            (*p).ckind = rnd() as _;
            (*p.add(1)).color = rnd() as _;
        });
        h::gm_GetNumCostumesForCKind = Some(|ck| {
            t(14);
            t(ck as u32);
            (1 + rnd() % 7) as u8
        });
        h::lbDvd_GetPreloadCacheScene = Some(|| {
            t(15);
            core::ptr::addr_of_mut!(PRE)
        });
        h::lbDvd_80018254 = Some(|| t(16));
        h::lbAudioAx_80026E84 = Some(|ck| {
            t(17);
            t(ck);
            let lo = rnd() as u64;
            let hi = rnd() as u64;
            (hi << 32) | lo
        });
        h::lbAudioAx_80026F2C = Some(|a| {
            t(18);
            t(a)
        });
        h::lbAudioAx_8002702C = Some(|f, mask| {
            t(19);
            t(f);
            t64(mask)
        });
        h::lbAudioAx_80027168 = Some(|| t(20));
        h::gm_SetNextGameModeStateId = Some(|id| {
            t(21);
            t(id as u32)
        });
        h::lbAudioAx_80026EBC = Some(|sk| {
            t(22);
            t(sk);
            let lo = rnd() as u64;
            let hi = rnd() as u64;
            (hi << 32) | lo
        });
        h::gm_SetupRulesDefaults = Some(|r| {
            t(23);
            t((r == core::ptr::addr_of_mut!(SMD.rules)) as u32);
            (*r).stkind = rnd() as _;
            (*r).time_limit = rnd();
        });
        h::gm_SetupHumanPlayer = Some(|p, ck, col, st, slot| {
            t(24);
            t(p.offset_from(SMD.players.as_mut_ptr()) as u32);
            t(ck as u32);
            t(col as u32);
            t(st as u32);
            t(slot as u32);
            (*p).ckind = rnd() as _;
            (*p).color = rnd() as _;
            (*p).nametag = rnd() as _;
        });
        h::gm_SetupCpuPlayer = Some(|p, ck, col, st, slot| {
            t(25);
            t(p.offset_from(SMD.players.as_mut_ptr()) as u32);
            t(ck as u32);
            t(col as u32);
            t(st as u32);
            t(slot as u32);
            (*p).slot = rnd() as _;
            (*p).team = rnd() as _;
        });
        h::gm_LoadRumbleEnabled = Some(|d| {
            t(26);
            t((d == core::ptr::addr_of_mut!(SMD)) as u32);
            t(hs_StartMeleeData(&*d, 0));
        });
        h::gm_80189CDC = Some(|d| {
            t(27);
            t((d == core::ptr::addr_of_mut!(SMD)) as u32)
        });
        h::gm_80162968 = Some(|s| {
            t(28);
            t(s);
            0
        });
        h::gm_8016247C = Some(|a| {
            t(29);
            t(a as u32);
            0
        });
        h::gm_80163298 = Some(|ck, a| {
            t(30);
            t(ck as i32 as u32);
            t(a as u32);
            false
        });
        h::gm_80188454 = Some(|idx| {
            t(31);
            t(idx as u32);
            rnd() as i32
        });
        h::gm_80173BC4 = Some(|a| {
            t(32);
            t(a as i32 as u32)
        });
        h::gm_80173EEC = Some(|| t(33));
        h::gm_80173754 = Some(|a, b| {
            t(34);
            t(a as u32);
            t(b as u32);
            sr(7)
        });
        h::lbAudioAx_80024030 = Some(|a| {
            t(35);
            t(a as u32)
        });
        h::gm_InitVsMode = Some(|v| {
            t(36);
            t((v == vsrow()) as u32);
            t(hs_VsModeData(&*v, 0));
            (*v).loser = rnd() as _;
        });
        h::gm_801677F0 = Some(|| {
            t(37);
            (rnd() % 6) as u8
        });
        h::SceneLaunch_SeedVs = Some(|vs, d| {
            t(38);
            t((vs == vsrow()) as u32);
            t(d as u32);
            sr(9)
        });
    }
}

unsafe fn state_hash() -> u32 {
    let mut h = 0;
    h = hs_VsModeData(&*vsrow(), h);
    h = hs_CSSData(&CSS, h);
    h = hs_SSSData(&SSS, h);
    h = hs_StartMeleeData(&SMD, h);
    h = hs_PreloadedGameModeState(&PRE, h);
    h = hs_CssSubStruct(&gm_80473814, h);
    h = hm(h, m::gm_804D68C0 as u32);
    h = hm(h, m::gm_804D68C1 as u32);
    h
}

#[test]
fn gmtrainingmode_matches_c() {
    install();
    let save: &'static mut gmm_x0 = Box::leak(Box::new(unsafe { core::mem::zeroed::<gmm_x0>() }));
    let mut c = Check::new(include_str!("data/tu_gmtrainingmode.txt"));
    unsafe {
        // the state table, once
        case(0, 0);
        let tb = &m::gm_Mode_Training_States;
        for e in tb.iter() {
            t(e.id as u32);
            t(e.preload as u32);
            t(e.flags as u32);
            t(e.on_enter.is_some() as u32);
            t(e.on_exit.is_some() as u32);
            t(e.info.scene_kind as u32);
            t(!e.info.enter_data.is_null() as u32);
            t(!e.info.exit_data.is_null() as u32);
        }
        let p = |x: *mut core::ffi::c_void, y: *mut core::ffi::c_void| (x == y) as u32;
        t(p(tb[0].info.enter_data, core::ptr::addr_of_mut!(m::css_data) as *mut _));
        t(p(tb[0].info.exit_data, core::ptr::addr_of_mut!(m::css_data) as *mut _));
        t(p(tb[1].info.enter_data, core::ptr::addr_of_mut!(m::sss_data) as *mut _));
        t(p(tb[1].info.exit_data, core::ptr::addr_of_mut!(m::sss_data) as *mut _));
        t(p(tb[2].info.enter_data, core::ptr::addr_of_mut!(m::training_enter_data) as *mut _));
        t(p(tb[2].info.exit_data, core::ptr::addr_of_mut!(m::training_exit_data) as *mut _));
        t((core::mem::size_of_val(&m::training_exit_data) / core::mem::size_of::<*mut core::ffi::c_void>()) as u32);
        c.line_tag("T", 0, 0);
    }
    unsafe {
        gmMainLib_804D3EE0 = save;
    }
    for op in 1..=9u32 {
        for k in 0..1500u32 {
            case(op, k);
            unsafe {
                zero(&mut GS);
                zero(&mut CSS);
                zero(&mut SSS);
                zero(&mut SMD);
                zero(&mut MEI);
                zero(&mut PRE);
                zero(&mut *gmMainLib_804D3EE0);
                zero(&mut gm_80473814);
                fl_VsModeData(&mut *vsrow());
                fl_CSSData(&mut CSS);
                fl_SSSData(&mut SSS);
                fl_StartMeleeData(&mut SMD);
                fl_PreloadedGameModeState(&mut PRE);
                fl_CssSubStruct(&mut gm_80473814);
                fl_MatchExitInfo(&mut MEI);
                m::gm_804D68C0 = (rnd() % 6) as u8;
                m::gm_804D68C1 = if rnd() & 1 != 0 { 0 } else { rnd() as u8 };
                CSS.pending_scene_change = if rnd() % 4 == 0 { 2 } else { rnd() as u8 };
                SSS.start_game = (rnd() & 1) as u8;
                GS.info.enter_data = core::ptr::addr_of_mut!(CSS) as *mut _;
                GS.info.exit_data = core::ptr::addr_of_mut!(CSS) as *mut _;
                let gs = core::ptr::addr_of_mut!(GS);
                match op {
                    1 => m::gm_801B1B74(gs),
                    2 => m::gm_801B1C24(gs),
                    3 => {
                        GS.info.enter_data = core::ptr::addr_of_mut!(SSS) as *mut _;
                        m::gm_801B1EB8(gs)
                    }
                    4 => {
                        GS.info.exit_data = core::ptr::addr_of_mut!(SSS) as *mut _;
                        m::gm_801B1EEC(gs)
                    }
                    5 => {
                        GS.info.enter_data = core::ptr::addr_of_mut!(SMD) as *mut _;
                        m::gm_801B1F70(gs)
                    }
                    6 => {
                        GS.info.exit_data = core::ptr::addr_of_mut!(MEI) as *mut _;
                        m::gm_801B2204(gs)
                    }
                    7 => m::gm_Mode_Training_OnInit(),
                    8 => m::gm_Mode_Training_OnLoad(),
                    _ => {
                        m::fn_801B1F6C(1);
                        m::gm_801B07E8_layer(core::ptr::addr_of_mut!(CSS), core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut());
                    }
                }
                t(hs_MatchExitInfo(&MEI, 0));
                c.line(op, k, 0, state_hash());
            }
        }
    }
    assert_eq!(c.done(), 1 + 9 * 1500);
}
