//! The Slippi online character select (`--slippi-menus on`), plan step 7. Logic follows Hero's `mu_slippi_css.c` and
//! the online major of `mu_slippi_menu.c` (Melee Unlocked, GPLv3, used with permission), native forms of Slippi's
//! Online/Menus CSS and Online Scene Gecko codes (GPL-2.0-or-later).
//!
//! GM_HANYU_CSS (unused in retail) becomes the online major, as on Slippi: its one state is the event-style one-door
//! character select (`gm_801B06B0(css, 0xE, ...)`, Hero's `css_prep`). The opponent comes first here (the code or the
//! queue was picked on the online menu, and the host's session searches as this screen opens); the screen shows the
//! search and the opponent (Hero's `text_think`), Start locks the fighter in once connected (`tx_lock_in`), and when
//! both are locked in the vanilla Start path leaves the screen (`mu_slippi_css_inputs` returning 1). The match then
//! runs in the host's match window (`play --slippi-peer`). B on the screen cancels the search and goes back to the
//! online menu.
//!
//! Direct follows Hero's winner / loser rule: the loser of the last game (the first game: the player in slot 0) goes
//! to the stage select on Start, picks, comes back and locks in with that stage; the winner locks in without one (the
//! loser's stage stands). Connected, holding Z disconnects (`DISCONNECT_HOLD_DELAY`) and the select stays up, idle
//! (Hero: the cleanup command, then `MU_SLP_MM_IDLE`); Start there searches again. Locked in, the fighter can't be
//! unselected and the colour can't change (Hero `mu_slippi_css_block_unselect` / `_block_costume_change`).
//!
//! The loser's stage select picks only from Hero's online stage set (`ONLINE_STAGES`); Random draws from it too.
//!
//! Not ported (noted for later): chat (`mu_slippi_chat.c`, SlippiCSS.dat), Teams' colour button.

#![allow(non_snake_case, non_upper_case_globals, static_mut_refs, clippy::missing_safety_doc)]

use super::slippi_online as so;
use super::support::*;
use core::sync::atomic::{AtomicBool, Ordering};
use melee_types::*;

/// Hero's MSRB connection states (`MU_SLP_MM_*`, Slippi's `ConnectionState`).
pub const CONN_IDLE: u8 = 0;
pub const CONN_INITIALIZING: u8 = 1;
pub const CONN_MATCHMAKING: u8 = 2;
pub const CONN_OPPONENT_CONNECTING: u8 = 3;
pub const CONN_SUCCESS: u8 = 4;
pub const CONN_ERROR: u8 = 5;

/// Hero's `MU_SLP_WINNER_*`: the last game's result, which decides Direct's stage pick.
pub const WINNER_NULL: u8 = 0;
pub const WINNER_WON: u8 = 1;
pub const WINNER_LOST: u8 = 2;
/// Hero's `DISCONNECT_HOLD_DELAY`: frames of Z held, connected, before the link is dropped.
pub const DISCONNECT_HOLD_DELAY: u16 = 48;

/// What the host's session and this screen share (Hero's match-state read, `mu_slippi_load_match_state`).
#[derive(Clone, Debug, Default)]
pub struct Online {
    pub mode: i32,
    pub conn: u8,
    /// Direct: the code searched for.
    pub search_code: String,
    /// The opponent's code once matchmaking answered.
    pub opp_code: String,
    pub error: String,
    pub local_ready: bool,
    pub remote_ready: bool,
    /// The screen's lock-in: the fighter (external id) and colour, for the session to send (Slippi's B5).
    pub lock: Option<(u8, u8)>,
    /// The screen was left with B: the session drops its search or link.
    pub cancel: bool,
    /// This side's player index in the link (0: the one who sent the request; set when connected).
    pub local_index: u8,
    /// The last game against this opponent (`WINNER_*`; Hero `st->is_winner`).
    pub winner: u8,
    /// The stage the loser picked on the stage select (Hero `st->chose_stage` + `picked_stage`).
    pub chose_stage: Option<u16>,
    /// The stage that goes with the lock-in (None: unset, the opponent's stands).
    pub lock_stage: Option<u16>,
    /// The screen is leaving for the stage select (the loser's pick).
    pub to_sss: bool,
    /// Frames Z has been held while connected.
    pub z_hold: u16,
    /// Hold-Z disconnect: the session drops the link, the screen stays up, idle.
    pub disconnect: bool,
    /// Games played against this opponent before this one (0: the first).
    pub game: u32,
    /// Unselect / colour-change attempts refused while locked in (tests).
    pub refused: u32,
    /// Idle after a hold-Z disconnect: Start (a fighter chosen) searches again (Hero's `MU_SLP_MM_IDLE` case).
    pub search_again: bool,
    /// Slippi / Hero: the Start that searched also locked the fighter in; once connected the lock-in goes out with no
    /// second press.
    pub start_locks: bool,
    /// A fighter is chosen on the select (the vanilla flag; Hero `*char_chosen`), for the panel's first line.
    pub chosen: bool,
}

pub static ONLINE: std::sync::Mutex<Online> = std::sync::Mutex::new(Online {
    mode: -1,
    conn: CONN_IDLE,
    search_code: String::new(),
    opp_code: String::new(),
    error: String::new(),
    local_ready: false,
    remote_ready: false,
    lock: None,
    cancel: false,
    local_index: 0,
    winner: WINNER_NULL,
    chose_stage: None,
    lock_stage: None,
    to_sss: false,
    z_hold: 0,
    disconnect: false,
    game: 0,
    refused: 0,
    search_again: false,
    start_locks: false,
    chosen: false,
});

pub fn with<R>(f: impl FnOnce(&mut Online) -> R) -> R {
    match ONLINE.lock() {
        Ok(mut g) => f(&mut g),
        Err(p) => f(&mut p.into_inner()),
    }
}

/// A new session for `mode` (the host calls it when it starts searching).
pub fn begin(mode: i32, search_code: &str) {
    with(|o| {
        *o = Online { mode, conn: CONN_INITIALIZING, search_code: search_code.to_string(), ..Default::default() };
    });
}

/// The link is closed after a hold-Z disconnect: the select stays up, idle (Hero `MU_SLP_MM_IDLE`), the fighter kept.
pub fn go_idle() {
    with(|o| {
        let keep = (o.mode, o.search_code.clone());
        *o = Online { mode: keep.0, conn: CONN_IDLE, search_code: keep.1, ..Default::default() };
    });
    eprintln!("online css: idle (disconnected); Start searches again");
}

/// The select opens idle (Hero `MU_SLP_MM_IDLE`): no search until Start with a fighter chosen.
pub fn wait_start(mode: i32, search_code: &str) {
    with(|o| {
        *o = Online { mode, conn: CONN_IDLE, search_code: search_code.to_string(), ..Default::default() };
    });
    eprintln!("online css: idle; Start with a fighter chosen searches");
}

/// Idle and Start was pressed: the host starts the search again (once).
pub fn take_search_again() -> bool {
    with(|o| {
        let r = o.search_again && o.conn == CONN_IDLE;
        if r {
            o.search_again = false;
        }
        r
    })
}

/// The screen left with both locked in: the host starts the match (once).
static GO: AtomicBool = AtomicBool::new(false);
pub fn take_go() -> bool {
    GO.swap(false, Ordering::Relaxed)
}

/// The online major is on screen.
static ON_CSS: AtomicBool = AtomicBool::new(false);
pub fn active() -> bool {
    ON_CSS.load(Ordering::Relaxed) && so::enabled()
}

static mut CSS_DATA: CSSData = unsafe { core::mem::zeroed() };
static mut SSS_DATA: SSSData = unsafe { core::mem::zeroed() };
static mut STATES: [GameModeState; 3] = unsafe {
    [
        state(0, lbDvdPreload_3, Some(css_prep), Some(css_decide), GS_CSS, &raw mut CSS_DATA as *mut core::ffi::c_void, &raw mut CSS_DATA as *mut core::ffi::c_void),
        // Direct's stage pick (Hero: the CSS's scene request 1, MU_SLP_CB_SSS_LOCK_IN): back to the select after it
        state(1, lbDvdPreload_3, Some(sss_prep), Some(sss_decide), GS_SSS, &raw mut SSS_DATA as *mut core::ffi::c_void, &raw mut SSS_DATA as *mut core::ffi::c_void),
        end(),
    ]
};

/// Hero's online stage set (`g_allowed_stages`, external ids): Fountain of Dreams, Pokemon Stadium, Yoshi's Story,
/// Dream Land, Battlefield, Final Destination. The online stage select picks only these; the match's setup refuses others.
pub const ONLINE_STAGES: [u16; 6] = [0x02, 0x03, 0x08, 0x1C, 0x1F, 0x20];
pub fn stage_allowed(stage: u16) -> bool {
    ONLINE_STAGES.contains(&stage)
}

/// The online stage select is on screen (the loser's pick).
static ON_SSS: AtomicBool = AtomicBool::new(false);
pub fn on_sss() -> bool {
    ON_SSS.load(Ordering::Relaxed) && so::enabled()
}

/// A pick outside the online stage set, refused on the stage select.
pub fn refused_stage(stage: u16) {
    eprintln!("online css: stage {stage} is not in the online stage set: refused");
}

/// The stage select for the loser's pick: the select's players and rules go with it (`gmVsMelee_EnterSss`).
unsafe extern "C" fn sss_prep(state: *mut GameModeState) {
    let sss = super::gm_1A3F::gm_GetGameModeStateEnterData(state) as *mut SSSData;
    (*sss).vs = CSS_DATA.vs;
    super::gm_1601::gm_80167FC4(sss);
    ON_SSS.store(true, Ordering::Relaxed);
    eprintln!("online css: stage select (the loser's pick)");
}

/// The pick (Hero's SSS lock-in callback): the stage is kept and the select comes back, which locks in with it.
unsafe extern "C" fn sss_decide(state: *mut GameModeState) {
    let sss = super::gm_1A3F::gm_GetGameModeStateExitData(state) as *mut SSSData;
    ON_SSS.store(false, Ordering::Relaxed);
    let picked = if (*sss).start_game != 0 { Some((*sss).vs.start.rules.stkind as u16) } else { None };
    // never outside the online set (the select refuses those; anything else is dropped and picked again)
    let picked = picked.filter(|&s| stage_allowed(s));
    with(|o| {
        o.to_sss = false;
        o.chose_stage = picked;
    });
    match picked {
        Some(st) => eprintln!("online css: the loser picked stage {st}"),
        None => eprintln!("online css: stage select left without a pick"),
    }
    super::gm_1A3F::gm_SetNextGameModeStateId(0);
}

/// GM_HANYU_CSS becomes the online major (Hero `mu_slippi_scene_install`, Slippi main.asm). Called with the menus' install.
pub unsafe fn install_major() {
    if !so::enabled() {
        return;
    }
    for m in super::gmscdata::modes.iter_mut() {
        if m.kind as u32 == GM_HANYU_CSS as u32 {
            m.states = &raw mut STATES as *mut GameModeState;
            m.on_load = None;
            m.on_unload = None;
            m.on_init = None;
            break;
        }
    }
}

/// The online menu's pick goes on to this major (the menu's exit, `pending_mode`).
pub unsafe fn enter_from_menu() {
    let data = super::mnmain::ext::gm_GetCurrentSceneExitData() as *mut MenuExitData;
    (*data).pending_mode = GM_HANYU_CSS as s8;
    super::mnmain::ext::gm_801A4B60();
}

unsafe fn ev() -> *mut EventData {
    &raw mut (*super::gmevent::ext::gmMainLib_804D3EE0).vs.unk_530
}

/// CSSScenePrep (Hero `css_prep`): the event-style one-door select.
unsafe extern "C" fn css_prep(state: *mut GameModeState) {
    let ev = ev();
    let css = super::gm_1A3F::gm_GetGameModeStateEnterData(state) as *mut CSSData;
    super::gm_1B03::gm_801B06B0(&mut *css, 0xE, (*ev).x2, 0, (*ev).x3 as i8, (*ev).nametag, 0, (*ev).x6);
    melee_runtime::tu::lbdvd::lbDvd_SetupVsPreloadCache();
    ON_CSS.store(true, Ordering::Relaxed);
    TEXT = core::ptr::null_mut();
    SHOWN = [const { String::new() }; LINES];
    eprintln!("online css: entered ({})", so::mode_name(with(|o| o.mode)));
}

/// CSSSceneDecide (Hero `css_decide`): B goes back to the online menu; a start with both locked in, to the match.
unsafe extern "C" fn css_decide(state: *mut GameModeState) {
    let css = super::gm_1A3F::gm_GetGameModeStateExitData(state) as *mut CSSData;
    ON_CSS.store(false, Ordering::Relaxed);
    TEXT = core::ptr::null_mut();
    let (conn, to_sss, disconnect) = with(|o| (o.conn, o.to_sss, o.disconnect));
    eprintln!("online css: exit request {} connection {conn}", (*css).pending_scene_change);
    if to_sss && !disconnect && (*css).pending_scene_change != 2 {
        // the fighter is kept for the way back (the event-style select restores it), and the stage select is next
        let ev = ev();
        super::gmevent::ext::gm_801B0730(css, &raw mut (*ev).x2, core::ptr::null_mut(), &raw mut (*ev).x3, &raw mut (*ev).nametag, core::ptr::null_mut());
        super::gm_1A3F::gm_SetNextGameModeStateId(1);
        eprintln!("online css: the loser goes to the stage select");
        return;
    }
    if (*css).pending_scene_change == 2 {
        with(|o| o.cancel = true);
        eprintln!("online css: cancelled, back to the online menu");
        super::gm_1A3F::gm_ChangeGameModeAfterCurrentScene(GM_MENU as i32);
        return;
    }
    let ev = ev();
    super::gmevent::ext::gm_801B0730(css, &raw mut (*ev).x2, core::ptr::null_mut(), &raw mut (*ev).x3, &raw mut (*ev).nametag, core::ptr::null_mut());
    (*ev).x8 = -1;
    (*ev).x9 = -1;
    (*ev).xA = -1;
    GO.store(true, Ordering::Relaxed);
    // the host's match window takes over from here; the menu is what shows meanwhile
    super::gm_1A3F::gm_ChangeGameModeAfterCurrentScene(GM_MENU as i32);
}

/// HandleInputsOnCSS (Hero `mu_slippi_css_inputs`) at the CSS's Start test: returns the trigger the vanilla path sees.
/// Start locks the fighter in once connected; Start reaches the vanilla path (which leaves the screen) only when both
/// players are locked in, or when Direct's loser leaves for the stage select. `held`: the port's held buttons (hold
/// Z); `ready`: the vanilla "a fighter is chosen" flag; `ck`/`color`: the door's fighter.
pub unsafe fn inputs(trigger: u32, held: u32, ready: bool, ck: i8, color: u8) -> u32 {
    const START: u32 = HSD_PAD_START as u32;
    const Z: u32 = HSD_PAD_Z as u32;
    let (conn, local, remote, mode) = with(|o| {
        o.chosen = ready && ck >= 0;
        (o.conn, o.local_ready, o.remote_ready, o.mode)
    });
    text_think();
    if trigger & START != 0 && !local {
        eprintln!("online css: Start (connection {conn}, fighter chosen {ready}, fighter {ck})");
    }
    if conn == CONN_IDLE {
        // Hero's MU_SLP_MM_IDLE case: Start with a fighter chosen searches again (the host restarts the session)
        if trigger & START != 0 && ready && ck >= 0 {
            with(|o| {
                if !o.search_again {
                    o.search_again = true;
                    o.start_locks = true;
                    o.winner = WINNER_NULL;
                    o.chose_stage = None;
                    o.error.clear(); // the "opponent left" note goes with the new search
                    eprintln!("online css: idle: Start, searching again");
                }
            });
        }
        return trigger & !START;
    }
    if conn != CONN_SUCCESS {
        return trigger & !START;
    }
    // connected: hold Z to disconnect (the session drops the link; the screen stays up, idle)
    let gone = with(|o| {
        if o.disconnect {
            return false;
        }
        if held & Z != 0 {
            o.z_hold += 1;
            if o.z_hold > DISCONNECT_HOLD_DELAY {
                o.z_hold = 0;
                o.disconnect = true;
                o.cancel = true;
                return true;
            }
        } else {
            o.z_hold = 0;
        }
        false
    });
    if gone {
        // Hero: the cleanup command (0xBA, the Z reason); the select stays up and goes idle once the link is closed
        eprintln!("online css: hold Z: disconnecting");
        return trigger & !START;
    }
    if local && remote && ready {
        return trigger | START;
    }
    let direct = mode == so::MODE_DIRECT;
    // a session's first game: the player in slot 0 picks the stage, as the loser of a game picks the next one
    let (winner, chose) = with(|o| {
        if direct && o.winner == WINNER_NULL {
            o.winner = if o.local_index == 0 { WINNER_LOST } else { WINNER_WON };
            o.chose_stage = None;
            eprintln!("online css: no game yet, local index {} {} the stage", o.local_index, if o.local_index == 0 { "picks" } else { "waits for" });
        }
        (o.winner, o.chose_stage)
    });
    // the search's Start locked the fighter in (Slippi: one press, then the match loads by itself)
    let auto = with(|o| core::mem::take(&mut o.start_locks));
    let mut advance = trigger & START != 0 || (auto && !local);
    if auto {
        eprintln!("online css: connected; the search's Start locks in (no second press)");
    }
    if !advance && direct && winner == WINNER_LOST && chose.is_some() {
        advance = true; // back from the stage select: the pick locks in
    }
    if advance && !local && ready && ck >= 0 {
        if direct && winner == WINNER_LOST && chose.is_none() {
            with(|o| o.to_sss = true);
            return trigger | START; // the vanilla path leaves the screen; css_decide goes on to the stage select
        }
        let stage = if direct && winner == WINNER_LOST { chose } else { None };
        with(|o| {
            o.lock = Some((ck as u8, color));
            o.lock_stage = stage;
        });
        super::mnmain::ext::lbAudioAx_80024030(1);
        eprintln!("online css: locked in fighter {ck} colour {color} stage {}", stage.map_or("unset".to_string(), |s| s.to_string()));
    }
    trigger & !START
}

/// No unselect or costume change once locked in (Hero `mu_slippi_css_block_unselect`).
pub fn block_unselect() -> bool {
    active() && with(|o| o.lock.is_some())
}

/// An unselect (A on the coin, B) or colour change refused while locked in.
pub fn refused_unselect() {
    let n = with(|o| {
        o.refused += 1;
        o.refused
    });
    eprintln!("online css: locked in: unselect refused ({n})");
}

// ---- the panel (Hero `text_think`, LoadCSSText, FN_UserTextUpdate): header, status, error, prompt, opponent, user --
const LINES: usize = 15;
const L_HEADER: usize = 0;
const L_STATUS: usize = 1; // two status lines, or the first two of the four error lines
const L_ERR: usize = 1; // L_ERR..L_ERR + 4: the error, wrapped (Hero ST_ERR1..ST_ERR4)
const L_PROMPT: usize = 5;
const L_PLAYING: usize = 6;
const L_OPP: usize = 7;
const L_USER: usize = 8; // "User", the name, "Connect Code", the code (Hero FN_InitUserDisplay)
const L_SPIN: usize = 12; // L_SPIN..L_SPIN + 3: the status lines' spinners, SPIN_WAIT / SPIN_DONE / empty (Hero ST_SPIN1..3)
const SPIN_WAIT: &str = "wait";
const SPIN_DONE: &str = "done";
/// Hero's SPINNER_TRANSITION_FRAMES: the wait glyph alternates every 15 frames.
const SPIN_FRAMES: u32 = 15;
static mut FRAME: u32 = 0;
/// Hero's sound edges (mu_slippi_css_inputs): the last lock-in and connection state seen.
static mut PREV_LOCK: bool = false;
static mut PREV_CONN: u8 = CONN_IDLE;
static mut TEXT: *mut HSD_Text = core::ptr::null_mut();
static mut SHOWN: [String; LINES] = [const { String::new() }; LINES];

/// The signed-in player (Hero `mu_slippi_user_name` / `mu_slippi_user_code`): the name and connect code of the login
/// the session searches with. Shown on the select, never written to the log.
static USER: std::sync::Mutex<(String, String)> = std::sync::Mutex::new((String::new(), String::new()));
pub fn set_user(name: &str, code: &str) {
    if let Ok(mut u) = USER.lock() {
        *u = (name.to_string(), code.to_string());
    }
}
pub fn user() -> (String, String) {
    USER.lock().map(|u| u.clone()).unwrap_or_default()
}

/// Printable ASCII only, at most `n` characters (the SIS font has no glyphs for the rest, and a line has a fixed room).
fn printable(s: &str, n: usize) -> String {
    s.chars().filter(|c| c.is_ascii_graphic() || *c == ' ').take(n).collect()
}

/// A connect code for the SIS font: its '#' has no ASCII glyph, so the fullwidth form (as Hero's code_sjis18).
fn code_text(s: &str, n: usize) -> String {
    printable(s, n).replace('#', "\u{FF03}")
}

/// Hero's error wrap: at 30 letters on the last space, four lines at most, from the first 241 bytes. The session's
/// own prefixes ("matchmaking: server error: ") are dropped, as Hero shows the service's words alone.
pub fn wrap_error(e: &str) -> std::vec::Vec<String> {
    let mut e = e.trim();
    for p in ["matchmaking: ", "server error: "] {
        e = e.strip_prefix(p).unwrap_or(e);
    }
    let e = if e.is_empty() { "The connection failed".to_string() } else { printable(e, 241) };
    let mut out = std::vec::Vec::new();
    let mut rest = e.as_str().trim_start();
    while !rest.is_empty() && out.len() < 4 {
        if rest.len() <= 30 {
            out.push(rest.to_string());
            break;
        }
        let cut = rest[..=30].rfind(' ').filter(|&i| i > 0).unwrap_or(30);
        out.push(rest[..cut].trim_end().to_string());
        rest = rest[cut..].trim_start();
    }
    out
}

/// The panel's lines for a state (pure, tested).
pub fn panel_lines(o: &Online, user: &(String, String)) -> [String; LINES] {
    let mut l: [String; LINES] = Default::default();
    let opp = if !o.opp_code.is_empty() {
        code_text(&o.opp_code, 18)
    } else if o.mode == so::MODE_DIRECT && !o.search_code.is_empty() {
        code_text(&o.search_code, 18)
    } else {
        "opponent".into()
    };
    l[L_HEADER] = if o.conn == CONN_ERROR { "Error".into() } else { format!("Slippi {}", so::mode_name(o.mode)) };
    if o.conn == CONN_ERROR {
        for (i, w) in wrap_error(&o.error).into_iter().enumerate() {
            l[L_ERR + i] = w;
        }
        l[L_PROMPT] = "Press B to go back".into();
    } else {
        // Hero's text_think steps: the fighter, then the search or lock-in, then the wait on the opponent, each
        // with its spinner (wait while it is pending, done once it is)
        let direct = o.mode == so::MODE_DIRECT;
        let searching = matches!(o.conn, CONN_INITIALIZING | CONN_MATCHMAKING) || (o.conn == CONN_IDLE && o.search_again);
        let link = if searching {
            format!("Searching for {opp}")
        } else if o.conn == CONN_OPPONENT_CONNECTING {
            format!("Connecting to {opp}")
        } else {
            String::new()
        };
        if !o.chosen {
            l[L_STATUS] = "Select your character".into();
            l[L_SPIN] = SPIN_WAIT.into();
            if !link.is_empty() {
                l[L_STATUS + 1] = link;
                l[L_SPIN + 1] = SPIN_WAIT.into();
            }
        } else {
            l[L_STATUS] = "Character selected".into();
            l[L_SPIN] = SPIN_DONE.into();
            if !o.local_ready {
                l[L_STATUS + 1] = if !link.is_empty() {
                    link
                } else if o.conn == CONN_SUCCESS && direct && o.winner == WINNER_LOST && o.chose_stage.is_none() {
                    "Press START to select stage".into()
                } else if o.conn == CONN_SUCCESS {
                    "Press START to lock in".into()
                } else {
                    "Press START to search".into()
                };
                l[L_SPIN + 1] = SPIN_WAIT.into();
            } else {
                l[L_STATUS + 1] = "Locked in".into();
                l[L_SPIN + 1] = SPIN_DONE.into();
                l[L_STATUS + 2] = match o.conn {
                    CONN_SUCCESS if o.remote_ready => "Starting the match".into(),
                    CONN_SUCCESS => format!("Waiting on {opp}"),
                    _ => link,
                };
                l[L_SPIN + 2] = if o.conn == CONN_SUCCESS && o.remote_ready { SPIN_DONE } else { SPIN_WAIT }.into();
            }
        }
        l[L_PROMPT] = match o.conn {
            CONN_SUCCESS => "Hold Z to disconnect".into(),
            CONN_INITIALIZING | CONN_MATCHMAKING | CONN_OPPONENT_CONNECTING => "Press B to cancel".into(),
            _ => String::new(),
        };
    }
    // idle after the opponent left (the host keeps a note): the select says so until the next Start
    let left = o.conn == CONN_IDLE && !o.search_again && !o.error.is_empty();
    l[L_PLAYING] = match o.conn {
        CONN_SUCCESS => "Playing:".into(),
        CONN_OPPONENT_CONNECTING => "Found:".into(),
        _ if left => "Opponent left".into(),
        _ => String::new(),
    };
    if !l[L_PLAYING].is_empty() && !left {
        l[L_OPP] = opp;
    }
    let (name, code) = user;
    if !name.is_empty() || !code.is_empty() {
        l[L_USER] = "User".into();
        l[L_USER + 1] = printable(name, 31);
        if !code.is_empty() {
            l[L_USER + 2] = "Connect Code".into();
            l[L_USER + 3] = code_text(code, 10);
        }
    }
    l
}

unsafe fn text_think() {
    use melee_runtime::tu::hsd_3A64 as sis;
    let u = user();
    let (mut lines, lock, conn) = with(|o| (panel_lines(o, &u), o.lock.is_some(), o.conn));
    // Hero's sounds: lock-in lost -> "back" (0); any -> error -> "error" (3); connected -> any -> "back"
    if PREV_LOCK && !lock {
        super::mnmain::ext::lbAudioAx_80024030(0);
    } else if PREV_CONN != CONN_ERROR && conn == CONN_ERROR {
        super::mnmain::ext::lbAudioAx_80024030(3);
    } else if PREV_CONN == CONN_SUCCESS && conn != CONN_SUCCESS {
        super::mnmain::ext::lbAudioAx_80024030(0);
    }
    if PREV_CONN != conn {
        eprintln!("online css: panel: connection {PREV_CONN} -> {conn}");
    }
    (PREV_LOCK, PREV_CONN) = (lock, conn);
    // the wait spinner's phase is part of the line, so the text is rebuilt when it turns
    let phase = (FRAME / SPIN_FRAMES) & 1;
    FRAME = (FRAME + 1) % (2 * SPIN_FRAMES);
    for l in lines[L_SPIN..L_SPIN + 3].iter_mut() {
        if l == SPIN_WAIT {
            *l = format!("{SPIN_WAIT}{phase}");
        }
    }
    if lines == SHOWN && !TEXT.is_null() {
        return;
    }
    if !TEXT.is_null() {
        melee_runtime::tu::sislib::HSD_SisLib_803A5CC4(TEXT);
        TEXT = core::ptr::null_mut();
    }
    let t = sis::HSD_SisLib_803A6754(0, 0);
    if t.is_null() {
        return;
    }
    (*t).default_kerning = 1;
    (*t).default_alignment = 0;
    (*t).pos_x = 0.0;
    (*t).pos_y = 0.0;
    (*t).pos_z = 0.0;
    // Hero's text_struct(0, 0.1) and create_subtext places / scales (LoadCSSText, FN_InitUserDisplay)
    (*t).font_size.x = 0.1;
    (*t).font_size.y = 0.1;
    let error = lines[L_HEADER] == "Error";
    let white = GXColor { r: 255, g: 255, b: 255, a: 255 };
    let gray = GXColor { r: 170, g: 170, b: 170, a: 255 };
    let red = GXColor { r: 255, g: 80, b: 80, a: 255 };
    // the status lines at Hero's line places, or the error at ST_ERR1..4
    let status_or_err = |i: usize| -> (f32, f32, f32, GXColor) {
        if error {
            (70.0, [52.0, 70.0, 88.0, 106.0][i], 0.4, red)
        } else {
            (90.0, [52.0, 75.0, 98.0, 121.0][i], 0.4, white)
        }
    };
    let place: [(f32, f32, f32, GXColor); LINES] = [
        (70.0, 23.0, 0.5, white),
        status_or_err(0),
        status_or_err(1),
        status_or_err(2),
        status_or_err(3),
        (70.0, 132.5, 0.4, gray),
        (-130.0, -256.0, 0.5, gray),
        (-50.0, -256.0, 0.5, white),
        (-112.0, 20.0, 0.4, gray),
        (-112.0, 40.0, 0.5, white),
        (-112.0, 65.0, 0.4, gray),
        (-112.0, 85.0, 0.5, white),
        (70.0, 52.0, 0.45, white),
        (70.0, 75.0, 0.45, white),
        (70.0, 98.0, 0.45, white),
    ];
    // Hero's spinner glyphs (Shift-JIS) and colours
    let wait = GXColor { r: 0x3C, g: 0xBC, b: 0xFF, a: 0xFF };
    let done = GXColor { r: 0x33, g: 0xFF, b: 0x2F, a: 0xFF };
    for (i, (l, &(x, y, scale, c))) in lines.iter().zip(place.iter()).enumerate() {
        let s = if l.is_empty() { " " } else { l.as_str() };
        let (n, c) = if i >= L_SPIN && !l.is_empty() {
            let (glyph, col): (&[u8], GXColor) = match l.as_str() {
                "wait0" => (&[0x81, 0x7B], wait),
                "wait1" => (&[0x81, 0x7E], wait),
                _ => (&[0x81, 0x7C], done),
            };
            (sis::HSD_SisLib_803A6B98_bytes(t, x, y, Some(glyph)), col)
        } else {
            (sis::HSD_SisLib_803A6B98(t, x, y, Some(s)), c)
        };
        sis::HSD_SisLib_803A7548(t, n, scale, scale);
        sis::HSD_SisLib_803A74F0(t, n, &c);
    }
    TEXT = t;
    SHOWN = lines;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn panel() {
        let none = (String::new(), String::new());
        let mut o = Online { mode: so::MODE_DIRECT, conn: CONN_MATCHMAKING, search_code: "BBBB#002".into(), ..Default::default() };
        let l = panel_lines(&o, &none);
        assert_eq!((l[L_STATUS].as_str(), l[L_SPIN].as_str()), ("Select your character", SPIN_WAIT));
        assert_eq!((l[L_STATUS + 1].as_str(), l[L_SPIN + 1].as_str()), ("Searching for BBBB\u{FF03}002", SPIN_WAIT));
        o.chosen = true;
        let l = panel_lines(&o, &none);
        assert_eq!((l[L_STATUS].as_str(), l[L_SPIN].as_str()), ("Character selected", SPIN_DONE));
        assert_eq!(l[L_STATUS + 1], "Searching for BBBB\u{FF03}002");
        o.conn = CONN_OPPONENT_CONNECTING;
        assert_eq!(panel_lines(&o, &none)[L_STATUS + 1], "Connecting to BBBB\u{FF03}002");
        o.conn = CONN_MATCHMAKING;
        assert_eq!(panel_lines(&o, &none)[L_PROMPT], "Press B to cancel");
        o.conn = CONN_SUCCESS;
        let l = panel_lines(&o, &none);
        assert_eq!((l[L_PLAYING].as_str(), l[L_OPP].as_str()), ("Playing:", "BBBB\u{FF03}002"));
        assert_eq!(l[L_STATUS + 1], "Press START to lock in");
        o.local_ready = true;
        let l = panel_lines(&o, &none);
        assert_eq!((l[L_STATUS + 1].as_str(), l[L_SPIN + 1].as_str()), ("Locked in", SPIN_DONE));
        assert_eq!((l[L_STATUS + 2].as_str(), l[L_SPIN + 2].as_str()), ("Waiting on BBBB\u{FF03}002", SPIN_WAIT));
        o.remote_ready = true;
        assert_eq!(panel_lines(&o, &none)[L_STATUS + 2], "Starting the match");
        let u = Online { mode: so::MODE_UNRANKED, conn: CONN_MATCHMAKING, ..Default::default() };
        let l = panel_lines(&u, &("Name".into(), "TEST#123".into()));
        assert_eq!((l[L_HEADER].as_str(), l[L_STATUS + 1].as_str()), ("Slippi Unranked", "Searching for opponent"));
        assert_eq!((l[L_USER].as_str(), l[L_USER + 1].as_str(), l[L_USER + 3].as_str()), ("User", "Name", "TEST\u{FF03}123"));
        let idl = Online { mode: so::MODE_DIRECT, conn: CONN_IDLE, chosen: true, ..Default::default() };
        assert_eq!(panel_lines(&idl, &none)[L_STATUS + 1], "Press START to search");
        // the opponent left: idle, the note shown, no search; after Start it reads "Searching"
        let mut gone = Online { mode: so::MODE_UNRANKED, conn: CONN_IDLE, chosen: true, error: "Your opponent disconnected".into(), ..Default::default() };
        let l = panel_lines(&gone, &none);
        assert_eq!((l[L_PLAYING].as_str(), l[L_OPP].as_str(), l[L_STATUS + 1].as_str()), ("Opponent left", "", "Press START to search"));
        gone.search_again = true;
        gone.error.clear();
        let l = panel_lines(&gone, &none);
        assert_eq!((l[L_PLAYING].as_str(), l[L_STATUS + 1].as_str()), ("", "Searching for opponent"));
        assert!(stage_allowed(0x1F) && stage_allowed(0x02) && !stage_allowed(0x15) && !stage_allowed(0));
    }

    #[test]
    fn server_error_wraps_safely() {
        let e = Online {
            mode: so::MODE_UNRANKED,
            conn: CONN_ERROR,
            error: "matchmaking: server error: A new version has been released. Return to menu to update".into(),
            ..Default::default()
        };
        let l = panel_lines(&e, &(String::new(), String::new()));
        assert_eq!(l[L_HEADER], "Error");
        assert_eq!(&l[L_ERR..L_ERR + 3], ["A new version has been", "released. Return to menu to", "update"]);
        let long = "x".repeat(500) + " \u{1F600} " + &"word ".repeat(80);
        let w = wrap_error(&long);
        assert!(w.len() <= 4 && w.iter().all(|s| s.len() <= 30 && s.is_ascii()));
        assert_eq!(wrap_error(""), ["The connection failed"]);
    }
}
