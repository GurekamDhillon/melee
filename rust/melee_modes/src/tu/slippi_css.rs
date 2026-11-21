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
//! Not ported (noted for later): chat (`mu_slippi_chat.c`, SlippiCSS.dat), Direct's loser-picks-the-stage SSS (the
//! stage is the decider's here), hold-Z disconnect, unselect guards while locked in, Teams' colour button.

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
static mut STATES: [GameModeState; 2] = unsafe {
    [
        state(0, lbDvdPreload_3, Some(css_prep), Some(css_decide), GS_CSS, &raw mut CSS_DATA as *mut core::ffi::c_void, &raw mut CSS_DATA as *mut core::ffi::c_void),
        end(),
    ]
};

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
    let conn = with(|o| o.conn);
    eprintln!("online css: exit request {} connection {conn}", (*css).pending_scene_change);
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
/// players are locked in. `ready`: the vanilla "a fighter is chosen" flag; `ck`/`color`: the door's fighter.
pub unsafe fn inputs(trigger: u32, ready: bool, ck: i8, color: u8) -> u32 {
    const START: u32 = HSD_PAD_START as u32;
    let (conn, local, remote) = with(|o| (o.conn, o.local_ready, o.remote_ready));
    text_think();
    if trigger & START != 0 && !local {
        eprintln!("online css: Start (connection {conn}, fighter chosen {ready}, fighter {ck})");
    }
    if conn == CONN_SUCCESS && local && remote && ready {
        return trigger | START;
    }
    if trigger & START != 0 && conn == CONN_SUCCESS && !local && ready && ck >= 0 {
        with(|o| o.lock = Some((ck as u8, color)));
        super::mnmain::ext::lbAudioAx_80024030(1);
        eprintln!("online css: locked in fighter {ck} colour {color}");
    }
    trigger & !START
}

/// No unselect or costume change once locked in (Hero `mu_slippi_css_block_unselect`).
pub fn block_unselect() -> bool {
    active() && with(|o| o.lock.is_some())
}

// ---- the panel (Hero `text_think`, LoadCSSText): status, opponent ----------------------------------------------
const LINES: usize = 3;
static mut TEXT: *mut HSD_Text = core::ptr::null_mut();
static mut SHOWN: [String; LINES] = [const { String::new() }; LINES];

/// The panel's lines for a state (pure, tested).
pub fn panel_lines(o: &Online) -> [String; LINES] {
    let opp = if !o.opp_code.is_empty() { o.opp_code.clone() } else if o.mode == so::MODE_DIRECT && !o.search_code.is_empty() { o.search_code.clone() } else { "opponent".into() };
    let mode = format!("Slippi {}", so::mode_name(o.mode));
    let status = match o.conn {
        CONN_IDLE | CONN_INITIALIZING => "Starting...".to_string(),
        CONN_MATCHMAKING => format!("Searching for {opp}"),
        CONN_OPPONENT_CONNECTING => format!("Connecting to {opp}"),
        CONN_SUCCESS if o.local_ready && !o.remote_ready => format!("Waiting on {opp}"),
        CONN_SUCCESS if o.local_ready => "Starting the match".to_string(),
        CONN_SUCCESS => "Press START to lock in".to_string(),
        _ => format!("Error: {}", if o.error.is_empty() { "the connection failed" } else { &o.error }),
    };
    let playing = if o.conn == CONN_SUCCESS { format!("Playing: {opp}") } else { String::new() };
    [mode, status, playing]
}

unsafe fn text_think() {
    use melee_runtime::tu::hsd_3A64 as sis;
    let lines = with(|o| panel_lines(o));
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
    (*t).font_size.x = 0.5;
    (*t).font_size.y = 0.5;
    // Hero's places: the status lines right of the door, "Playing:" at the top of the bar
    let at = [(130.0f32, 80.0f32), (130.0, 104.0), (-200.0, -256.0)];
    for (i, l) in lines.iter().enumerate() {
        let s = if l.is_empty() { " " } else { l.as_str() };
        let n = sis::HSD_SisLib_803A6B98(t, at[i].0 / 0.5, at[i].1 / 0.5, Some(s));
        let c = if i == 1 && l.starts_with("Error") { GXColor { r: 255, g: 80, b: 80, a: 255 } } else { GXColor { r: 255, g: 255, b: 255, a: 255 } };
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
        let mut o = Online { mode: so::MODE_DIRECT, conn: CONN_MATCHMAKING, search_code: "BBBB#002".into(), ..Default::default() };
        assert_eq!(panel_lines(&o)[1], "Searching for BBBB#002");
        o.conn = CONN_SUCCESS;
        assert_eq!(panel_lines(&o)[2], "Playing: BBBB#002");
        o.local_ready = true;
        assert_eq!(panel_lines(&o)[1], "Waiting on BBBB#002");
        o.remote_ready = true;
        assert_eq!(panel_lines(&o)[1], "Starting the match");
        let u = Online { mode: so::MODE_UNRANKED, conn: CONN_MATCHMAKING, ..Default::default() };
        assert_eq!(panel_lines(&u)[1], "Searching for opponent");
        let e = Online { mode: so::MODE_UNRANKED, conn: CONN_ERROR, error: "x".into(), ..Default::default() };
        assert_eq!(panel_lines(&e)[1], "Error: x");
    }
}
