//! tu::gm_1B03 vs the C (state oracle): gen/turef/tu_gm_1B03_ref.c is the game's gm_1B03.c compiled natively
//! (shipping contraction) with gm_InitVsMode, the scene-launch queries and the costume tables stubbed; the
//! expected lines are tests/data/tu_gm_1B03.txt (regenerate with
//! `python rust/melee_modes/gen/turef/tu_shard537_gm.py tu_gm_1B03`). Each line is the callee log, the return
//! value, the out parameters and the StartMeleeData / CSSData fields afterwards.
#![allow(static_mut_refs)]

mod tu537;
use melee_modes::tu::gm_1B03 as t;
use melee_types::*;
use tu537::Rng;

const NCASES: usize = 400;
const FUNC_NAMES: [&str; 12] = [
    "gm_SetupSubColors",
    "SceneLaunch_SixPresentationTest",
    "gm_SetupSuddenDeath",
    "gm_801B05F4",
    "gm_SetupHumanPlayer",
    "gm_SetupCpuPlayer",
    "gm_801B06B0",
    "gm_801B0730",
    "gm_801B07B4",
    "gm_801B07E8",
    "SceneLaunch_ApplyTeamCostumes",
    "SceneLaunch_HudCount",
];
const CK: [i8; 6] = [2, 0x12, 0x13, 5, 0, 0x12];

static mut LOG: String = String::new();
static mut ACTIVE: i32 = 0;
static mut ETC: i32 = 0;
static mut VARIANT: i32 = 0;
static mut CALLS: i32 = 0;

fn fill_start(r: &mut Rng, s: &mut StartMeleeData) {
    *s = StartMeleeData::zeroed();
    s.rules.is_teams = (r.rnd() % 3) as u8;
    s.rules.set_x0_3(r.rnd() % 8);
    s.rules.set_match_kind(r.rnd() % 8);
    s.rules.set_timer_enabled(r.rnd() % 2);
    s.rules.set_x2_5(r.rnd() % 2);
    for i in 0..6 {
        let p = &mut s.players[i];
        p.slot_type = (r.rnd() % 5) as u8;
        p.ckind = CK[(r.rnd() % 6) as usize];
        p.team = (r.rnd() % 3) as u8;
        p.sub_color = (r.rnd() % 3) as u8;
        p.stocks = ((r.rnd() % 6) as i8).wrapping_sub(1);
        p.color = (r.rnd() % 5) as u8;
        p.cpu_level = (r.rnd() % 9) as u8;
        p.nametag = (r.rnd() % 200) as u8;
        p.damage1 = (r.rnd() % 400) as u16;
        p.slot = (r.rnd() % 7) as u8;
    }
}

fn dump_start(tag: &str, s: &StartMeleeData) -> String {
    let mut o = format!(
        "|{}={},{},{},{},{}",
        tag,
        s.rules.is_teams,
        s.rules.x0_3(),
        s.rules.match_kind(),
        s.rules.timer_enabled(),
        s.rules.x2_5()
    );
    for p in &s.players {
        o += &format!(
            ";{},{},{},{},{},{},{},{},{},{},{}",
            p.slot_type, p.ckind, p.stocks, p.color, p.slot, p.sub_color, p.team, p.nametag, p.cpu_level, p.damage1, p.xB
        );
    }
    o
}

#[test]
fn matches_c() {
    let want = tu537::data("tu_gm_1B03");
    unsafe {
        t::ext::GM_INITVSMODE = Some(|vs| {
            LOG.push_str("init;");
            core::ptr::write_bytes(vs as *mut u8, 0x11, core::mem::size_of::<VsModeData>());
            (*vs).start.rules.set_x0_3(5);
            (*vs).start.rules.set_match_kind(3);
            (*vs).start.rules.set_timer_enabled(1);
            (*vs).start.rules.set_x2_5(0);
        });
        t::ext::SCENELAUNCH_ACTIVE = Some(|| ACTIVE);
        t::ext::SCENELAUNCH_ENEMYTEAMCOLORS = Some(|| ETC);
        t::ext::GM_801692BC = Some(|c| unsafe {
            let n = CALLS;
            CALLS += 1;
            (c as u8).wrapping_mul(7).wrapping_add(3).wrapping_add(if VARIANT & 1 != 0 { (n % 2) as u8 } else { 0 })
        });
        t::ext::GM_80169290 = Some(|c| (c as u8).wrapping_mul(5).wrapping_add(1));
        t::ext::GM_80169264 = Some(|c| (c as u8).wrapping_mul(3).wrapping_add(2));
    }
    let mut r = Rng::new();
    let mut lines = want.lines();
    let mut st = StartMeleeData::zeroed();
    let mut cssd = CSSData::zeroed();
    let mut end = MatchEnd::zeroed();
    for f in 0..FUNC_NAMES.len() {
        for i in 0..NCASES {
            // setup_case(), in the C driver's order.
            fill_start(&mut r, &mut st);
            cssd = CSSData::zeroed();
            fill_start(&mut r, &mut cssd.vs.start);
            cssd.unk_0x0 = (r.rnd() % 6 + 1) as u16;
            cssd.match_type = (r.rnd() % 5) as u8;
            end = MatchEnd::zeroed();
            end.outcome = (r.rnd() % 3) as u8;
            end.match_kind = (r.rnd() % 3) as u8;
            end.is_teams = (r.rnd() % 3) as u8;
            end.frame_count = r.rnd() % 50;
            for k in 0..6 {
                end.player_standings[k].ckind = CK[(r.rnd() % 6) as usize];
                end.player_standings[k].ftkind = (r.rnd() % 9) as i8;
                end.player_standings[k].stocks = ((r.rnd() % 5) as i8).wrapping_sub(1);
                end.player_standings[k].x28 = r.rnd() % 60;
                end.player_standings[k].is_big_loser = (r.rnd() % 2) as u8;
                end.player_standings[k].team = (r.rnd() % 5) as u8;
            }
            for k in 0..5 {
                end.team_standings[k].is_big_loser = (r.rnd() % 2) as u8;
            }
            unsafe {
                ACTIVE = (r.rnd() % 4 != 0) as i32;
                ETC = (r.rnd() % 4 != 0) as i32;
                VARIANT = r.rnd() as i32;
                CALLS = 0;
            }
            let (mut oc, mut os, mut ocol, mut on, mut ol) = (0x55i8, 0x55u8, 0x55u8, 0x55u8, 0x55u8);
            let (mut os8, mut ocol8, mut on8) = (0x55i8, 0x55i8, 0x55i8);
            let mask = r.rnd();
            let slot = (r.rnd() % 6) as usize;
            let a = r.rnd();
            let b = r.rnd();
            let c = r.rnd();
            let d = r.rnd();
            unsafe { LOG.clear() };
            let mut ret = 0;
            match f {
                0 => t::gm_SetupSubColors(&mut st),
                1 => ret = t::SceneLaunch_SixPresentationTest((a & 1) as i32),
                2 => t::gm_SetupSuddenDeath(&mut st, &end),
                3 => t::gm_801B05F4(&mut st.players[slot], slot as i32),
                4 => t::gm_SetupHumanPlayer(&mut st.players[slot], a as u8, b as u8, c as u8, (d % 6) as u8),
                5 => t::gm_SetupCpuPlayer(&mut st.players[slot], a as u8, b as u8, c as u8, (d % 6) as u8),
                6 => t::gm_801B06B0(&mut cssd, a as u8, b as i8, c as i8, d as i8, (a >> 8) as u8, (b >> 8) as u8, slot as u8),
                7 => t::gm_801B0730(
                    &cssd,
                    if mask & 1 != 0 { Some(&mut oc) } else { None },
                    if mask & 2 != 0 { Some(&mut os) } else { None },
                    if mask & 4 != 0 { Some(&mut ocol) } else { None },
                    if mask & 8 != 0 { Some(&mut on) } else { None },
                    if mask & 16 != 0 { Some(&mut ol) } else { None },
                ),
                8 => t::gm_801B07B4(&mut cssd, a as i8, b as i8, c as i8, d as u8, (a >> 8) as u8, (b >> 8) as u8),
                10 => t::SceneLaunch_ApplyTeamCostumes(&mut st),
                11 => t::SceneLaunch_HudCount(&mut st),
                _ => t::gm_801B07E8(
                    &cssd,
                    if mask & 1 != 0 { Some(&mut oc) } else { None },
                    if mask & 2 != 0 { Some(&mut os8) } else { None },
                    if mask & 4 != 0 { Some(&mut ocol8) } else { None },
                    if mask & 8 != 0 { Some(&mut on8) } else { None },
                    if mask & 16 != 0 { Some(&mut ol) } else { None },
                ),
            }
            let mut got = format!(
                "{}.{} {}|log={}|ret={}|out={},{},{},{},{},{},{},{}|css={},{}",
                f,
                i,
                FUNC_NAMES[f],
                unsafe { &LOG },
                ret,
                oc,
                os,
                ocol,
                on,
                ol,
                os8,
                ocol8,
                on8,
                cssd.unk_0x0,
                cssd.match_type
            );
            got += &dump_start("st", &st);
            got += &dump_start("cs", &cssd.vs.start);
            let line = lines.next().expect("fewer lines than cases");
            assert_eq!(got, line, "function {} case {}", FUNC_NAMES[f], i);
        }
    }
    assert!(lines.next().is_none(), "more lines than cases");
}
