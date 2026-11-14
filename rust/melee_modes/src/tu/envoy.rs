//! Envoy run support (rust/mods/envoy_sim, envoy_ui): the native half of a single-player Envoy run.
//!
//! Everything here is inert until the Envoy sim mod publishes a plan (`gd_envoy_plan`) and the run is armed
//! (`ARMED`, from the Envoy camp); with no mod loaded PLAN[0] stays 0 and `match_begin` returns at once, so the goldens
//! and replays are unchanged, and with the mods loaded but no run armed VS is retail.
//!
//! * plan: the next VS match the run wants (stage, the player's tag pair, stocks and ratios, the CPU line-up). The
//!   mod keeps it current; `match_begin` (top of `fn_8016E730`, before the players and the stage are created)
//!   rewrites the `StartMeleeData` from it. A plan main / partner of -1 takes the CSS pick of P1 / P2: the run's
//!   pair is picked on the retail CSS.
//! * view: words the sim mod publishes for its presentation mod (`gd_ui_envoy`, read-only).
//! * live upgrades: `apply` (attack / defense ratio, stocks, attribute multipliers on both tag fighters).
//!
//! All statics: the snapshot covers them, so a rollback restores them with the game.
#![allow(non_snake_case, non_upper_case_globals, static_mut_refs, clippy::missing_safety_doc)]

use melee_fighter::tu::player as pl;
use melee_types::*;

pub const PLAN_WORDS: usize = 52;
pub const VIEW_WORDS: usize = 1536;

// plan layout (i32 words)
pub const P_ACTIVE: usize = 0;
pub const P_STAGE: usize = 1;
pub const P_MAIN: usize = 2;
pub const P_PARTNER: usize = 3;
pub const P_STOCKS: usize = 4;
pub const P_ATK: usize = 5;
pub const P_DEF: usize = 6;
pub const P_NCPU: usize = 7;
pub const P_CPU_CK: usize = 8;
pub const P_CPU_LV: usize = 11;
pub const P_CPU_STOCKS: usize = 14;
pub const P_CPU_ATK: usize = 17;
pub const P_CPU_DEF: usize = 20;
pub const P_CPU_SCALE: usize = 23;
/// 1: this fight is the run's boss (the Petey boss replaces the stand-in when `MELEE_ENVOY_BOSS=petey`).
pub const P_BOSS: usize = 26;
/// 1: local co-op (envoy-core-v2.md V2): slot 1 is a second human with its own tag pair, the CPUs take slots 2-3.
pub const P_COOP: usize = 27;
/// Co-op: the second player's main / partner (-1: the CSS pick of P2 / P4).
pub const P_MAIN2: usize = 28;
pub const P_PARTNER2: usize = 29;
/// Envoy endless loops: the Petey cages' HP x1000 (0: retail's; envoy_sim raises it 25% a loop).
pub const P_BOSS_HP: usize = 30;
/// Envoy rooms: per CPU row, the character it comes back as in the next wave (-1: no next wave) and that wave's stocks.
pub const P_WAVE_CK: usize = 32;
pub const P_WAVE_STOCKS: usize = 35;
/// An elite room's second wave (foe row 0 only; 0: none): the elite's attack, defense, scale (x1000) and CPU level.
pub const P_WAVE_ATK: usize = 38;
pub const P_WAVE_DEF: usize = 39;
pub const P_WAVE_SCALE: usize = 40;
pub const P_WAVE_LV: usize = 41;
/// Envoy gauntlets (back-to-back duels): foe rows 0-1, the characters of waves 3, 4 and 5 (-1: none; three words a
/// row). Every one is preloaded at match begin; wave 2 is P_WAVE_CK.
pub const P_GAUNT_CK: usize = 44;
/// The wave the elite arrives in (foe row 0 of an elite room; 0: none, read as 2 for an old plan with P_WAVE_SCALE).
pub const P_ELITE_WAVE: usize = 50;
/// The stage a live stage transition will go to next (Gr_Kind; 0 none): hinted to the asset worker with the plan, so
/// its file is in memory long before the hand-off frame.
pub const P_STAGE_NEXT: usize = 51;
/// Final Destination (Gr_Kind): the Petey arena for now (an original stage is later work).
const GR_FD: i32 = 37;

pub static mut PLAN: [i32; PLAN_WORDS] = [0; PLAN_WORDS];
/// The engine's asset hint for a newly published plan (the coming fight's stage and fighters): set at install. Never
/// sim-visible (a worker prepares the files; the load itself is the same with or without it).
pub static mut PLAN_HINT: Option<fn(&[i32; PLAN_WORDS])> = None;
pub static mut VIEW: [i32; VIEW_WORDS] = [0; VIEW_WORDS];
/// V3 (envoy-synergy-v3.md 12.5): per match slot, event bits the engine sets during the frame at the exact site
/// (bit 0: the fighter L-cancelled an aerial landing), read and cleared by `gd_envoy_events`. Set only while a run
/// is armed, so retail play never writes it.
pub static mut EVENTS: [u32; 4] = [0; 4];
pub const EVENT_LCANCEL: u32 = 1;
/// Envoy items: the fighter left its jump squat as a short hop (bit 1) or a full hop (bit 2).
pub const EVENT_SHORTHOP: u32 = 2;
pub const EVENT_FULLHOP: u32 = 4;

/// The jump site (`ftCo_KneeBend_Anim`, before `ftCo_Jump_Enter`): installed in `melee_fighter::hooks::ENVOY_JUMP`.
pub fn on_jump(f: &Fighter, short_hop: bool) {
    if armed() == 0 || f.is_sub_fighter() != 0 {
        return;
    }
    let s = f.player_idx as usize;
    if s < 4 {
        unsafe { EVENTS[s] |= if short_hop { EVENT_SHORTHOP } else { EVENT_FULLHOP } };
    }
}

/// The L-cancel site (`ftCo_LandingAir_EnterWithLag`, the branch where `x67F < xE4` sets the skill mode): installed
/// in `melee_fighter::hooks::ENVOY_LCANCEL`. A sub-fighter (the second Ice Climber) is ignored.
pub fn on_lcancel(f: &Fighter) {
    if armed() == 0 || f.is_sub_fighter() != 0 {
        return;
    }
    let s = f.player_idx as usize;
    if s < 4 {
        unsafe { EVENTS[s] |= EVENT_LCANCEL };
    }
}

/// Read and clear `slot`'s event bits.
pub unsafe fn events(slot: i32) -> u32 {
    if !(0..4).contains(&slot) {
        return 0;
    }
    let e = EVENTS[slot as usize];
    EVENTS[slot as usize] = 0;
    e
}

/// Grand Finale (V3 12.5): multiply the knockback of the hit `slot`'s fighter in play is taking now (called from the
/// take-damage hook, where `kb_applied` is already set and `ftCo_Damage_CalcKnockback` reads it next).
pub unsafe fn kb_scale(slot: i32, mul_milli: i32) -> i32 {
    let f = active_fighter(slot);
    if f.is_null() {
        return -1;
    }
    let m = mul_milli.clamp(1000, 1500) as f32 / 1000.0;
    (*f).dmg.kb_applied *= m;
    0
}

/// Matches rewritten so far (the mod's "a new fight began" edge).
pub static mut SEQ: i32 = 0;
/// The pair the last rewrite used: main, partner (character kinds).
pub static mut ROSTER: [i32; 2] = [-1, -1];
/// Co-op: the second player's pair (main, partner); -1 solo.
pub static mut ROSTER2: [i32; 2] = [-1, -1];

// ---- the door (envoy-redesign.md 1.2-1.3): menu-time state, never match state
/// The run is armed: only then does `match_begin` rewrite a VS match (and the sim mod publish a plan). Set by the
/// camp's Continue / New run, cleared by Abandon, by the end of the run and by entering the main menu.
pub static mut ARMED: i32 = 0;

/// Whether the run is armed: the camp's flag, or the headless audit knob `MELEE_ENVOY_ARM=1` (a scene / `--match` start has
/// no menus to arm it from).
pub fn armed() -> i32 {
    static KNOB: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    if unsafe { ARMED } != 0 || *KNOB.get_or_init(|| std::env::var_os("MELEE_ENVOY_ARM").is_some()) { 1 } else { 0 }
}
/// The salt of the next new run's seed (menu-time: set by the camp's New run, read by the sim mod when the run
/// starts, `gd_envoy_salt`; the seed it gives lives in the sim state from then on). 0 with `MELEE_ENVOY_ARM` and in
/// scripted runs (`MELEE_PAD_SCRIPT` without `MELEE_ENVOY_SEED`): those keep the fixed seed sequence.
pub static mut RUN_SALT: i32 = 0;

/// The camp starts a new run: pick its salt. `MELEE_ENVOY_SEED=<n>` fixes it (a reproducible run; 0 = the old fixed
/// sequence); a scripted run without it keeps 0; otherwise the clock and a per-process counter (every launch and
/// every new run rolls different foes, offers and drops).
pub fn new_run_salt() -> i32 {
    static N: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
    let n = N.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let salt = match std::env::var("MELEE_ENVOY_SEED").ok().and_then(|v| v.trim().parse::<u32>().ok()) {
        Some(v) => v & 0x3FFF_FFFF,
        None if std::env::var_os("MELEE_PAD_SCRIPT").is_some_and(|v| !v.is_empty()) => 0,
        // a netplay session (mod identity frozen): every peer must roll the same run, and the clock differs per peer.
        // No shared per-session value reaches the camp yet (the envoy= handshake is milestone 6): the fixed sequence.
        None if melee_data::netid::frozen() => 0,
        None => {
            let t = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_nanos() as u64);
            let mut x = (t ^ (t >> 29)).wrapping_mul(0x9E37_79B9_7F4A_7C15).wrapping_add(n as u64);
            x ^= x >> 32;
            ((x as u32) & 0x3FFF_FFFF).max(1)
        }
    } as i32;
    unsafe { RUN_SALT = salt };
    eprintln!("envoy: new run salt {salt}");
    salt
}

/// A rewritten fight has started since the run was armed: the camp replaces the VS CSS (the results land in it).
pub static mut CAMP_DUE: i32 = 0;
/// The main menu's extra Envoy row exists (a presentation mod registered the camp scene with `gd_ui_envoy_door`).
pub static mut DOOR_ON: i32 = 0;
#[allow(non_snake_case)]
pub fn DOOR_ON_now() -> bool {
    unsafe { DOOR_ON != 0 }
}
/// The main menu cursor is on the Envoy row (below the retail rows).
pub static mut DOOR_HOVER: i32 = 0;
/// Envoy Quick Tag: the tag button (the camp's switch, `gd_ui_envoy_set_tag_button`; MELEE_ENVOY_TAG_BUTTON=y|x|dpad
/// sets the start value): Y (default), X, or D-pad down with the classic cinematic tag.
pub const TAG_BTN_Y: i32 = 0;
pub const TAG_BTN_X: i32 = 1;
pub const TAG_BTN_CLASSIC: i32 = 2;
pub static mut TAG_BUTTON: i32 = -1;

pub fn tag_button() -> i32 {
    unsafe {
        if TAG_BUTTON < 0 {
            TAG_BUTTON = match std::env::var("MELEE_ENVOY_TAG_BUTTON").as_deref() {
                Ok("x") | Ok("X") => TAG_BTN_X,
                Ok("dpad") | Ok("classic") => TAG_BTN_CLASSIC,
                _ => TAG_BTN_Y,
            };
        }
        TAG_BUTTON
    }
}

/// The Envoy row (or the browser's Envoy mode) was chosen: the host opens the camp scene at the next present.
pub static mut DOOR_OPEN: i32 = 0;
/// The camp asked for the VS mode (its Continue / New run on the main menu): the menu scene starts it next frame.
pub static mut LAUNCH_VS: i32 = 0;
/// Envoy V3: player 2 joined at the camp (envoy_ui, `gd_ui_envoy_coop`): a new run starts in co-op (`gd_envoy_coop`).
pub static mut COOP_JOIN: i32 = 0;

/// The run record (envoy_sim publishes it in VIEW[REC_AT..]; the host keeps the last one on disk).
pub const REC_WORDS: usize = 48;
pub const REC_AT: usize = 704;
pub const REC_MAGIC: i32 = 0x454E_5652; // "ENVR"
pub const REC_VERSION: i32 = 9; // 9: envoy items; 8: the forked road, bank tokens, pins; 7: sessions (the bank in R_LOOP); 6: endless loops; 5 (V3) still read; older dropped
/// The record the host holds (loaded from disk, updated from the view); `SAVED_LEN` 0 = no run saved.
pub static mut SAVED: [i32; REC_WORDS] = [0; REC_WORDS];
pub static mut SAVED_LEN: i32 = 0;
/// The camp asked to forget the saved run (New run / Abandon): the host deletes the file.
pub static mut SAVED_DROP: i32 = 0;

/// The record's digest word (the last one): FNV-1a over the others. A stale or edited file fails it.
pub fn rec_digest(r: &[i32]) -> i32 {
    let mut h: u32 = 0x811C_9DC5;
    for &w in &r[..REC_WORDS - 1] {
        for b in w.to_le_bytes() {
            h ^= b as u32;
            h = h.wrapping_mul(0x0100_0193);
        }
    }
    h as i32
}

pub fn rec_valid(r: &[i32]) -> bool {
    r.len() >= REC_WORDS && r[0] == REC_MAGIC && (5..=REC_VERSION).contains(&r[1]) && r[REC_WORDS - 1] == rec_digest(r)
}

/// Envoy endless loops: the furthest loop any run on this save reached (host file `best.bin`, envoy_store; the camp
/// shows it). Menu-time state, never read by the sim.
pub static mut BEST_LOOP: i32 = 0;

/// Sessions (v3g): the armed run's fight was won and the next fight is in the same session (envoy_data view.rs
/// `V_SESS_NEXT` 760): the VS mode skips the results screen, straight back to the character select the camp replaces.


/// Envoy enemy waves (first step): the script layer's enemy seams (script_game.rs `Script_EnemyHit` / `_Defeated` /
/// `_Removed`, no-ops before). Logged only: envoy_sim polls the registry (`gd_envoy_enemy_i`), which is game state,
/// so nothing here has to be rolled back.
pub unsafe extern "C" fn enemy_hit(handle: i32, player: i32, dmg: i32) {
    if std::env::var_os("MELEE_ENVOY_ENEMY_LOG").is_some() {
        eprintln!("envoy: enemy {handle} hit by player {player} for {dmg}");
    }
}
pub unsafe extern "C" fn enemy_defeated(kind: i32, handle: i32) {
    if std::env::var_os("MELEE_ENVOY_ENEMY_LOG").is_some() {
        eprintln!("envoy: enemy {handle} (kind {kind}) defeated");
    }
}
pub unsafe extern "C" fn enemy_removed(kind: i32, handle: i32, reason: i32) {
    if std::env::var_os("MELEE_ENVOY_ENEMY_LOG").is_some() {
        eprintln!("envoy: enemy {handle} (kind {kind}) removed, reason {reason}");
    }
}

/// The armed run's match was left from the pause (L+R+A+Start: the retail No Contest). The sim keeps the run (the
/// fight starts over, as after closing the game mid-fight) instead of reading the torn-down stocks as a loss.
pub unsafe fn quit_pending() -> i32 {
    (armed() != 0 && crate::tu::gmvs::controller.state.match_result as u32 == OUTCOME_NO_CONTEST as u32) as i32
}

/// The sim took a quit (view word 24 `V_OUTCOME` 3) and the run is live: the VS mode skips the No Contest results
/// screen, straight back to the select the camp replaces (gmvsmode `onExitVs`).
pub unsafe fn quit_to_camp() -> bool {
    armed() != 0 && VIEW[0] == 0x454E_5633 && VIEW[1] == 1 && VIEW[24] == 3
}

pub unsafe fn session_continue() -> bool {
    armed() != 0 && VIEW[0] == 0x454E_5633 && VIEW[1] == 1 && (VIEW[760] == 1 || VIEW[766] == 1)
}

/// The main menu's input with the Envoy row (mnmain `mn_8022D7F4`, after the retail read): what the retail code must
/// do. `at_last`: the retail cursor is on the last unlocked row.
/// The main menu's Envoy row was chosen with the Brawl files missing: presents left of the "needs your Brawl disc"
/// message the mod draws (the camp never loads).
pub static mut DOOR_NO_DISC: i32 = 0;
/// The run ended and the player chose "Back to camp" on the road: 1 = the VS character select backs out to the main
/// menu (as a held B does), 2 = the main menu opens the camp. Never leaves the player on the character select.
pub static mut BACK_TO_CAMP: i32 = 0;
/// Fights 2+: the road's "Start the fight" goes straight to the match with the party the run has and the stage the
/// run chose (the match rewrite sets both): 1 = the VS character select confirms itself as soon as it is ready
/// (as Start would), 2 = confirmed, the stage select is skipped (gmvsmode `onExitCss`). `MELEE_ENVOY_SELECT=1`
/// keeps both screens.
pub static mut SKIP_SELECT: i32 = 0;
/// The party chosen at the camp's fighter picker (gd_ui_envoy_party): player 1's main and partner, player 2's main
/// and partner (character kinds; -1 none). Used for a new run's first fight in place of the character select's
/// picks (a saved run's own party wins: PLAN). With it set, Start run goes from the camp straight to fight 1:
/// `SKIP_SELECT` 3 = the character select confirms itself after it has settled (nothing is shown: the UI covers it)
/// and the stage select is skipped.
pub static mut PICK: [i32; 4] = [-1; 4];
/// Calls of the character select's cursor think while `SKIP_SELECT` is 3 (it confirms after 30).
pub static mut SKIP_WAIT: i32 = 0;
pub fn skip_select_on() -> bool {
    static V: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    !*V.get_or_init(|| std::env::var("MELEE_ENVOY_SELECT").is_ok_and(|v| v == "1"))
}
/// A on the main menu's Envoy row (mnmain `mn_8022DB10`): open the camp, or refuse when the Brawl files are missing.
pub unsafe fn door_main_open() -> bool {
    if !crate::tu::petey_model::brawl_present() {
        DOOR_NO_DISC = 240;
        return false;
    }
    DOOR_OPEN = 1;
    true
}
pub const DOOR_RETAIL: i32 = 0;
pub const DOOR_EAT: i32 = 1;
pub const DOOR_MOVE: i32 = 2;
pub const DOOR_OPENED: i32 = 3;
pub const DOOR_TO_TOP: i32 = 4;
pub unsafe fn door_1p(confirm: bool, back: bool, up: bool, down: bool, at_last: bool) -> i32 {
    if DOOR_ON == 0 {
        DOOR_HOVER = 0;
        return DOOR_RETAIL;
    }
    if DOOR_HOVER != 0 {
        if confirm {
            DOOR_OPEN = 1;
            return DOOR_OPENED;
        }
        if back {
            DOOR_HOVER = 0;
            return DOOR_RETAIL;
        }
        if up {
            DOOR_HOVER = 0;
            return DOOR_MOVE;
        }
        if down {
            DOOR_HOVER = 0;
            return DOOR_TO_TOP;
        }
        return DOOR_EAT;
    }
    if down && at_last && !confirm && !back {
        DOOR_HOVER = 1;
        return DOOR_MOVE;
    }
    DOOR_RETAIL
}

const CK_ICS: i32 = 0x0E;
const CK_ZELDA: i32 = 0x12;
const CK_SHEIK: i32 = 0x13;
const CK_FOX: i32 = 0x02;
const CK_MARTH: i32 = 0x09;

/// A character that can be one half of a tag pair (a retail fighter that owns no sub-fighter).
pub fn taggable(ck: i32) -> bool {
    ((0..=0x19).contains(&ck) && ck != CK_ICS && ck != CK_ZELDA && ck != CK_SHEIK) || mex_loadable(ck)
}

/// The first m-ex added fighter's character kind (`CK_MEX0` in melee_engine mex_host) and the slot count.
pub const CK_MEX0: i32 = 0x22;
pub const MEX_SLOTS: i32 = 94;

/// An m-ex added fighter the running build can load (the disc's MxDt.dat names it; false on a retail disc).
pub fn mex_loadable(ck: i32) -> bool {
    (CK_MEX0..CK_MEX0 + MEX_SLOTS).contains(&ck) && unsafe { crate::tu::gmresultplayer::ext::Mex_PortCKindToExt(ck) } > 25
}

/// An m-ex fighter's name (null on a retail fighter or without m-ex).
pub unsafe fn mex_name(ck: i32) -> *const i8 {
    if !mex_loadable(ck) {
        return core::ptr::null();
    }
    crate::tu::gmresultplayer::ext::Mex_FighterName(crate::tu::gmresultplayer::ext::Mex_PortCKindToExt(ck))
}

/// The CSS state of this match is Envoy's (`bypass_css`): its exit refills the players before the VS copy.
pub static mut BYPASSED: bool = false;

/// Envoy owns the select (the camp's picker): gmvsmode's onEnterCss calls this after `gmVsMelee_EnterCss` for a
/// launch or the road. The players come from the run (`match_begin`: the picks or the plan) and the preload cache is
/// written the way the retail select's exit leaves it. The CSS scene itself still runs (the Rust port has no other
/// scene to put in the state: the PC frontend's loading scene is not ported, `fl_open`), confirms itself as soon as
/// it has settled (mncharsel, `SKIP_SELECT` 1) under the camp's cover, takes no input, and the SSS is skipped.
/// onExitCss calls `refill_css` before the VS copy so the select's own cursor picks never reach the match.
pub unsafe fn bypass_css(css: *mut CSSData) -> bool {
    SEL_CSS = css;
    if SKIP_SELECT != 1 && SKIP_SELECT != 3 {
        // a fought run comes back to the CSS state: the camp's road is drawn over Envoy's select (the host's
        // replace_when on GS_ENVOY_SEL), which holds until the road arms the next fight or the run leaves
        if ARMED != 0 && CAMP_DUE != 0 && skip_select_on() {
            SEL_HOLD = true;
            BYPASSED = true;
            return true;
        }
        return false;
    }
    SKIP_SELECT = 2;
    SKIP_WAIT = 0;
    BYPASSED = true;
    refill_css(css, true);
    // what the retail select's exit leaves: confirmed (START), not backed out
    (*css).pending_scene_change = 1;
    true
}

/// The scene kind of Envoy's select (the CSS state's scene while `bypass_css` holds): no archive, no draw, two
/// frames under the UI's cover, so no retail character select is ever shown or loaded (an m-ex disc's
/// `mexSelectChr` included). Past the port's frontend kind (gmscdata GS_FRONTEND = GS_COUNT + 1).
pub const GS_ENVOY_SEL: u8 = GS_COUNT as u8 + 2;
static mut SEL_FRAMES: i32 = 0;
/// Envoy's select holds for the camp (the road after a fight); `SEL_CSS` is the state's CSS data.
pub static mut SEL_HOLD: bool = false;
static mut SEL_CSS: *mut CSSData = core::ptr::null_mut();

pub unsafe extern "C" fn envoy_sel_enter(_data: *mut core::ffi::c_void) {
    SEL_FRAMES = 0;
    eprintln!("envoy: select scene (hold {})", SEL_HOLD as i32);
}

pub unsafe extern "C" fn envoy_sel_frame() {
    SEL_FRAMES += 1;
    if SEL_HOLD {
        let css = SEL_CSS;
        if SKIP_SELECT == 1 {
            // the road armed the next fight: the run's players, confirmed
            SEL_HOLD = false;
            SKIP_SELECT = 2;
            refill_css(css, true);
            (*css).pending_scene_change = 1;
            SEL_FRAMES = 0;
        } else if BACK_TO_CAMP == 1 || ARMED == 0 {
            // the run is over or left: back out to the main menu (BACK_TO_CAMP opens the camp there)
            SEL_HOLD = false;
            if BACK_TO_CAMP == 1 {
                BACK_TO_CAMP = 2;
            }
            BYPASSED = false;
            (*css).pending_scene_change = 2;
            crate::tu::adapt::gm_801A4B60__d469d9();
        }
        return;
    }
    if SEL_FRAMES == 2 {
        crate::tu::adapt::gm_801A4B60__d469d9();
    }
}

/// The run's players and the matching preload cache into the CSS data (see `bypass_css`).
pub unsafe fn refill_css(css: *mut CSSData, preload: bool) {
    let start = &mut (*css).vs.start;
    // the CSS normally leaves P1 human (the rewrite below fills every slot from the run's plan / the camp's picks)
    start.players[0].slot_type = Gm_PKind_Human as u8;
    if start.players[0].ckind < 0 || start.players[0].ckind as i32 == ChKind_None as i32 {
        start.players[0].ckind = CK_FOX as i8;
    }
    // match_begin's bookkeeping (the match count, the camp-due mark) belongs to the real match start: kept as it was
    // the rules too (teams, timer, stage) stay the select's until the match start rewrites them: gmVsMelee_EnterVs
    // would otherwise see the run's teams before the engine's scene-launch hooks exist; only the stage is read here
    let (seq, due) = (SEQ, CAMP_DUE);
    let rules = start.rules;
    match_begin(start);
    let stage = start.rules.stkind;
    start.rules = rules;
    SEQ = seq;
    CAMP_DUE = due;
    if !preload {
        return;
    }
    // what the CSS exit (fs_css_finish / fs_preload_players / fs_preload_stage) leaves for the loader
    let pc = crate::tu::adapt::lbDvd_GetPreloadCacheScene__1ed990();
    for i in 0..4usize {
        let p = &start.players[i];
        let e = &mut (*pc).game_cache.entries[i];
        if p.slot_type as i32 == Gm_PKind_NA as i32 {
            e.char_id = ChKind_None as i32;
        } else {
            e.char_id = p.ckind as i32;
            e.color = p.color as _;
        }
    }
    (*pc).game_cache.stkind = stage as StKind;
    crate::tu::adapt::lbDvd_80018254__6be579();
}

fn ratio(v: i32) -> f32 {
    if v <= 0 { 1.0 } else { v as f32 / 1000.0 }
}

/// Rewrite the VS match about to start from the plan (no-op without one, outside VS, and for a replay).
pub unsafe fn match_begin(d: *mut StartMeleeData) {
    melee_core::frameprof::note(melee_core::frameprof::Kind::Envoy, || melee_core::frameprof::Text::new().s("match setup"));
    crate::tu::petey::disarm();
    crate::tu::subspace::begin_match(0x5B5A_CE01 ^ (SEQ as u32).wrapping_mul(0x9E37_79B9));
    melee_fighter::tu::player::TAG_DEFER = false;
    melee_fighter::tu::player::TAG_DEFERRED = [None; 4];
    TAG_DEFER_WAIT = 0;
    // Envoy sessions: the match-end hold is the sim's, per match (gd_envoy_end_hold); never left on for a retail match
    crate::tu::gmvs::gmVs_SetEndHold(0);
    melee_fighter::hooks::ENVOY_TAG_MASK = [0; 4];
    pl::ENVOY_WAVE_CK = [-1; 4];
    pl::ENVOY_WAVE_Q = [[-1; 3]; 4];
    WAVE_NO = 1;
    WAVE_PARK = [0; 4];
    WAVE_PENDING = [0; 4];
    if PLAN[P_ACTIVE] == 0 || armed() == 0 || d.is_null() || crate::tu::gm_1A3F::gm_GetCurrentGameMode() as u32 != GM_VS as u32 {
        return;
    }
    let d = &mut *d;
    // the tag partners are created 30 logic frames into the match (during the entry, before GO), not in its setup tick (player.rs TAG_DEFER)
    melee_fighter::tu::player::TAG_DEFER = true;
    TAG_DEFER_WAIT = 30; // logic frames: the setup tick itself runs several; well before GO (no tag possible)
    crate::tu::tag_swap::reset();
    EVENTS = [0; 4];
    melee_fighter::hooks::ENVOY_LCANCEL = Some(on_lcancel);
    melee_fighter::hooks::ENVOY_JUMP = Some(on_jump);
    let coop = PLAN[P_COOP] == 1;
    let css = |i: usize| -> i32 { let p = &d.players[i]; if p.slot_type != Gm_PKind_NA as u8 { p.ckind as i32 } else { -1 } };
    // solo: P1's pick is the main, P2's the partner; co-op: P1 + P3 are player 1's pair, P2 + P4 player 2's
    let (mut css_main, mut css_partner) = (css(0), if coop { css(2) } else { css(1) });
    // the camp's picker, when used (a saved run's party is PLAN's and wins in `pair`)
    let (mut css_m2, mut css_p2) = (css(1), css(3));
    if PICK[0] >= 0 {
        css_main = PICK[0];
        css_partner = PICK[1];
        if PICK[2] >= 0 {
            css_m2 = PICK[2];
            css_p2 = PICK[3];
        }
    }
    // test knob (headless checks, e.g. an m-ex fighter on the ACE disc): MELEE_ENVOY_TEST_MAIN=main[,partner]
    // (character kinds) replaces player 1's picks for a new run
    if let Ok(v) = std::env::var("MELEE_ENVOY_TEST_MAIN") {
        let mut it = v.split(',').filter_map(|t| t.trim().parse::<i32>().ok());
        if let Some(m) = it.next() {
            css_main = m;
            if let Some(p) = it.next() {
                css_partner = p;
            }
        }
    }
    let pair = |want_main: i32, want_partner: i32, cm: i32, cp: i32| -> (i32, i32) {
        let mut main = if want_main >= 0 { want_main } else { cm };
        if !taggable(main) {
            main = CK_FOX;
        }
        let mut partner = if want_partner >= 0 { want_partner } else { cp };
        if !taggable(partner) || partner == main {
            partner = if main == CK_MARTH { CK_FOX } else { CK_MARTH };
        }
        (main, partner)
    };
    let (main, partner) = pair(PLAN[P_MAIN], PLAN[P_PARTNER], css_main, css_partner);
    ROSTER = [main, partner];
    pl::TAG_PARTNER_CK = [partner as i8, -1, -1, -1];
    ROSTER2 = [-1, -1];
    let humans = if coop { 2 } else { 1 };
    if coop {
        let (m2, p2) = pair(PLAN[P_MAIN2], PLAN[P_PARTNER2], css_m2, css_p2);
        ROSTER2 = [m2, p2];
        pl::TAG_PARTNER_CK[1] = p2 as i8;
    }

    for h in 0..humans {
        let me = &mut d.players[h];
        me.ckind = (if h == 0 { main } else { ROSTER2[0] }) as i8;
        me.slot_type = 0;
        // test knob (headless win-path checks): MELEE_ENVOY_TEST_P1CPU=1 makes the players level-9 CPUs
        if std::env::var_os("MELEE_ENVOY_TEST_P1CPU").is_some() {
            me.slot_type = 1;
            me.cpu_level = 9;
            me.cpu_kind = 4;
        }
        me.stocks = PLAN[P_STOCKS].clamp(1, 99) as i8;
        me.team = 0;
        me.attack_ratio = ratio(PLAN[P_ATK]);
        me.defense_ratio = ratio(PLAN[P_DEF]);
        me.model_scale = 1.0;
        if h == 1 {
            me.color = 0;
            me.sub_color = 0;
            me.slot = 2; // port 2 (1-based; getPort): never port 1, or player 2 would mirror player 1's controller
            me.spawn_pos = -1;
            me.handicap = 9;
        }
    }
    // Quick Tag: the tag button no longer jumps for the run's humans (the classic D-pad tag masks nothing)
    let mask = match tag_button() {
        TAG_BTN_Y => 0x0800,
        TAG_BTN_X => 0x0400,
        _ => 0,
    };
    for h in 0..humans {
        melee_fighter::hooks::ENVOY_TAG_MASK[h] = mask;
    }
    // 0: an enemy-wave fight (no foe fighters; the sim spawns the enemies, the match end is held)
    let n = PLAN[P_NCPU].clamp(0, 4 - humans as i32) as usize;
    if n == 0 {
        crate::tu::gmvs::gmVs_SetEndHold(1); // a wave fight: no foe to eliminate; the sim ends it (gd_envoy_end_match)
    }
    for i in 0..(4 - humans) {
        let p = &mut d.players[humans + i];
        if i >= n {
            p.slot_type = Gm_PKind_NA as u8;
            continue;
        }
        p.ckind = { let k = PLAN[P_CPU_CK + i]; if mex_loadable(k) { k } else { k.clamp(0, 0x19) } } as i8;
        p.slot_type = 1;
        p.cpu_kind = 4; // the VS AI (0 is the training dummy)
        p.cpu_level = PLAN[P_CPU_LV + i].clamp(1, 9) as u8;
        p.stocks = PLAN[P_CPU_STOCKS + i].clamp(1, 99) as i8;
        p.color = 0;
        p.sub_color = 0;
        p.team = 1;
        p.slot = (humans + i + 1) as u8; // the CSS's own numbering for a CPU row
        p.spawn_pos = -1;
        p.handicap = 9; // the CSS default (0 is not a neutral handicap)
        p.attack_ratio = ratio(PLAN[P_CPU_ATK + i]);
        p.defense_ratio = ratio(PLAN[P_CPU_DEF + i]);
        p.model_scale = ratio(PLAN[P_CPU_SCALE + i]);
        let wk = PLAN[P_WAVE_CK + i];
        if wave_ck_ok(wk) {
            pl::ENVOY_WAVE_CK[humans + i] = wk as i8;
            if i < 2 {
                for k in 0..3 {
                    let g = PLAN[P_GAUNT_CK + i * 3 + k];
                    if !wave_ck_ok(g) {
                        break;
                    }
                    pl::ENVOY_WAVE_Q[humans + i][k] = g as i8;
                }
            }
        }
    }
    // 4-player rows past the first four are never used by VS
    let r = &mut d.rules;
    r.is_teams = 1;
    r.set_friendly_fire(0);
    r.set_is_stock(1);
    r.set_match_kind(MatchKind_Stock as u32);
    r.set_timer_enabled(0);
    r.time_limit = 0;
    r.item_freq = -1;
    let petey = PLAN[P_BOSS] == 1 && crate::tu::petey::selected();
    // the exploration room (explore.rs, MELEE_ENVOY_EXPLORE=1, off by default): every non-boss fight is played in the
    // Brawl map section, carried by Final Destination
    let explore = !petey && crate::tu::explore::enabled() && crate::tu::explore::map().is_some();
    crate::tu::explore::WANT = explore;
    crate::tu::explore::ARENA_WANT = petey; // Petey's fight in his own arena (explore.rs), when the owner's files have it
    let stage = if petey || explore { GR_FD } else { PLAN[P_STAGE] };
    if stage > 0 && crate::tu::stage_hop::supported(stage) {
        r.stkind = melee_world::tu::stage::Stage_StageSlotSelect(stage) as u16;
    }
    if petey {
        // the one CPU is Petey's carrier: stands still (cpu_kind 0), plain size and ratios, one stock
        let c = &mut d.players[humans];
        c.cpu_kind = 0;
        c.cpu_level = 1;
        c.stocks = 1;
        c.model_scale = 1.0;
        c.attack_ratio = 1.0;
        c.defense_ratio = 1.0;
        // HP scales with the players on the player's side (P3: 1-2)
        crate::tu::petey::arm(humans as i32, humans as i32, 0x5E7E_u32 ^ (SEQ as u32).wrapping_mul(0x9E37_79B9));
        melee_fighter::tu::ftdrawcommon::BOSS_DRAW = Some(crate::tu::petey::draw);
    } else {
        crate::tu::petey::disarm();
    }
    BASE = [[[0.0; 4]; 2]; 4];
    blast_hooks();
    BLASTS = [Blast::OFF; 4];
    LANDED = [Landed::NONE; 4];
    SEQ = SEQ.wrapping_add(1);
    CAMP_DUE = 1;
    eprintln!(
        "envoy: match {} rewritten: main {main} partner {partner} co-op {} stocks {} cpus {n} stage {} foes {:?}",
        SEQ, coop as i32, me_stocks(d), PLAN[P_STAGE], (0..n).map(|i| d.players[humans + i].ckind as i32).collect::<std::vec::Vec<_>>()
    );
}

/// A character a wave can bring in (no sub-fighter characters: the respawn refuses them).
fn wave_ck_ok(k: i32) -> bool {
    (0..=0x19).contains(&k) && k != CK_ICS && k != CK_ZELDA && k != CK_SHEIK
}

fn me_stocks(d: &StartMeleeData) -> i32 {
    d.players[0].stocks as i32
}

unsafe fn fighters(slot: i32) -> [*mut Fighter; 2] {
    let p = &*pl::Player_GetPtrForSlot(slot);
    let f = |g: *mut HSD_GObj| if g.is_null() { core::ptr::null_mut() } else { (*g).user_data as *mut Fighter };
    [f(p.player_entity[0]), f(p.player_entity[1])]
}

// ---- Envoy rooms (waves of fighter foes). A foe slot that loses its last stock while its room has another wave is
// parked by the sim (`wave_park`: its stocks are already set back): at the end of its KO countdown it stays asleep
// instead of the rebirth (gm_80167320, the retail "player out" path never runs since it has stocks). When the wave's
// last foe falls the sim starts the next wave (`wave_start`): each parked slot is replaced by its next character
// (melee_fighter `envoy_wave_respawn`, preloaded at match begin) and reborn on its platform. Plain statics.
pub static mut WAVE_PARK: [u8; 4] = [0; 4];
/// Parked and asleep (the KO countdown has ended): waiting for `wave_start`.
pub static mut WAVE_PENDING: [u8; 4] = [0; 4];
/// The wave in play (1-based; set at match begin, +1 per `wave_start`).
pub static mut WAVE_NO: i32 = 1;

/// Whether the wave starting now is foe row 0's elite (an elite room's last wave).
unsafe fn elite_now(s: usize) -> bool {
    let at = if PLAN[P_ELITE_WAVE] > 0 { PLAN[P_ELITE_WAVE] } else { 2 };
    s == 1 + (PLAN[P_COOP] != 0) as usize && PLAN[P_WAVE_SCALE] > 0 && WAVE_NO + 1 == at
}

/// Gauntlets: after a slot's swap, its next duel's character comes off the queue (the old flip-back is the
/// fallback once the queue is empty).
unsafe fn wave_queue_pop(s: usize) {
    let q = pl::ENVOY_WAVE_Q[s];
    if q[0] >= 0 {
        pl::ENVOY_WAVE_CK[s] = q[0];
        pl::ENVOY_WAVE_Q[s] = [q[1], q[2], -1];
    }
}

/// The sim parks `slot` (0 ok, -1 not in a run / not a slot).
pub unsafe fn wave_park(slot: i32) -> i32 {
    if armed() == 0 || !(0..4).contains(&slot) {
        return -1;
    }
    WAVE_PARK[slot as usize] = 1;
    0
}

/// gm_80167320's rebirth branch: a parked slot is not reborn now (true: skip the rebirth).
pub unsafe fn wave_hold_rebirth(slot: i32) -> bool {
    if !(0..4).contains(&slot) || WAVE_PARK[slot as usize] == 0 {
        return false;
    }
    WAVE_PENDING[slot as usize] = 1;
    true
}

/// Start the next wave: every parked slot whose countdown ended comes back as its next character (or as itself if
/// the swap is refused) and is reborn. Returns how many came back; a slot still in its countdown stays parked.
pub unsafe fn wave_start() -> i32 {
    // two passes: a swap gives back the old character's animation bank, which can make room for a slot refused in
    // the first pass (heap 1 is tight with three foes and a tag pair)
    let mut later = [false; 4];
    for s in 0..4 {
        if WAVE_PARK[s] == 0 || WAVE_PENDING[s] == 0 {
            continue;
        }
        let ck = pl::ENVOY_WAVE_CK[s];
        if ck >= 0 && swap_in(s, ck) {
            wave_queue_pop(s);
        } else if ck >= 0 {
            if pl::envoy_wave_old_way() {
                eprintln!("envoy: wave respawn of slot {s} as {ck} refused; it comes back as itself");
            } else {
                later[s] = true;
            }
        }
    }
    let mut n = 0;
    for s in 0..4 {
        if WAVE_PARK[s] == 0 || WAVE_PENDING[s] == 0 {
            continue;
        }
        WAVE_PARK[s] = 0;
        WAVE_PENDING[s] = 0;
        // foe row 0 of an elite room: the elite's numbers before the new fighter is built (it reads the scale)
        let elite = elite_now(s);
        if elite {
            pl::Player_SetModelScale(s as i32, ratio(PLAN[P_WAVE_SCALE]));
        }
        let ck = pl::ENVOY_WAVE_CK[s];
        if ck >= 0 && later[s] {
            if swap_in(s, ck) {
                wave_queue_pop(s);
            } else {
                eprintln!("envoy: wave respawn of slot {s} as {ck} refused; it comes back as itself");
            }
        }
        if elite {
            pl::Player_SetAttackRatio(s as i32, ratio(PLAN[P_WAVE_ATK]));
            pl::Player_SetDefenseRatio(s as i32, ratio(PLAN[P_WAVE_DEF]));
            pl::Player_SetModelScale(s as i32, ratio(PLAN[P_WAVE_SCALE]));
            pl::Player_SetPlayerAndEntityCpuLevel(s as i32, PLAN[P_WAVE_LV].clamp(1, 9));
        }
        crate::tu::gm_1601::fn_8016719C(s as i32, 0);
        n += 1;
    }
    if n > 0 {
        WAVE_NO += 1;
    }
    n
}

/// One slot's swap to its next-wave character (the elite's scale set first: the new fighter reads it).
unsafe fn swap_in(s: usize, ck: i8) -> bool {
    if elite_now(s) {
        pl::Player_SetModelScale(s as i32, ratio(PLAN[P_WAVE_SCALE]));
    }
    let ok = pl::envoy_wave_respawn(s as i32, ck as CharacterKind);
    if ok {
        eprintln!("envoy: wave new character, slot {s} as {ck}");
    }
    ok
}

/// Slots parked and asleep, as bits (the sim waits for all of them before the next wave).
pub unsafe fn wave_pending() -> i32 {
    (0..4i32).filter(|&s| WAVE_PENDING[s as usize] != 0).fold(0i32, |m, s| m | 1i32.wrapping_shl(s as u32))
}

pub const UP_ATTACK: i32 = 0;
pub const UP_DEFENSE: i32 = 1;
pub const UP_STOCKS: i32 = 2;
pub const UP_WEIGHT: i32 = 3;
pub const UP_SPEED: i32 = 4;
pub const UP_JUMP: i32 = 5;

/// `apply`'s fighter selector (`which | APPLY_MAIN`): the attribute multipliers touch only the main (entity 0) or the
/// partner (entity 1) of the tag pair; without one, both. Ratios and stocks are per slot.
pub const APPLY_MAIN: i32 = 0x100;
pub const APPLY_PARTNER: i32 = 0x200;

/// A live upgrade on `slot` (both tag fighters, or one with `APPLY_MAIN` / `APPLY_PARTNER`). `v` is x1000 for
/// ratios / multipliers, a count for stocks.
pub unsafe fn apply(slot: i32, which: i32, v: i32) -> i32 {
    if !(0..4).contains(&slot) || crate::tu::stage_hop::check(-1) == crate::tu::stage_hop::HOP_NOT_GAMEPLAY {
        return -1;
    }
    let sel = (which >> 8) & 3;
    let which = which & 0xFF;
    let m = v as f32 / 1000.0;
    match which {
        UP_ATTACK => pl::Player_SetAttackRatio(slot, m),
        UP_DEFENSE => pl::Player_SetDefenseRatio(slot, m),
        UP_STOCKS => pl::Player_SetStocks(slot, v.clamp(0, 99)),
        UP_WEIGHT | UP_SPEED | UP_JUMP => {
            if !(0.25..=4.0).contains(&m) {
                return -2;
            }
            for (i, f) in fighters(slot).into_iter().enumerate() {
                if f.is_null() || (sel != 0 && sel != i as i32 + 1) {
                    continue;
                }
                let a = &mut (*f).co_attrs;
                match which {
                    UP_WEIGHT => a.weight *= m,
                    UP_SPEED => {
                        a.walk_max_vel *= m;
                        a.dash_initial_velocity *= m;
                        a.dash_max_velocity *= m;
                    }
                    _ => a.jump_v_initial_velocity *= m,
                }
            }
        }
        _ => return -2,
    }
    0
}

/// Add `delta_milli` / 1000 percent to `slot`'s active fighter (`bench` 0) or its benched tag partner (`bench` 1),
/// clamped to 0..=999 (a heal is a negative delta). Returns the new percent x1000, or -1 (no such fighter). The
/// bench is the sleeping sub-fighter (its own percent, carried in by the swap), or in the in-place prototype the
/// benched percent the swap restores.
pub unsafe fn percent(slot: i32, bench: i32, delta_milli: i32) -> i32 {
    if !(0..4).contains(&slot) || crate::tu::stage_hop::check(-1) == crate::tu::stage_hop::HOP_NOT_GAMEPLAY {
        return -1;
    }
    if bench == 0 && slot == crate::tu::petey::carrier() {
        // the boss's HP is separate from percent: damage(amount) (heals do nothing)
        crate::tu::petey::damage(delta_milli as f32 / 1000.0);
        return 0;
    }
    let add = |p: f32| {
        let v = p + delta_milli as f32 / 1000.0;
        if v < 0.0 { 0.0 } else if v > 999.0 { 999.0 } else { v }
    };
    if bench != 0 && pl::tag_inplace() {
        if pl::TAG_SLOT[slot as usize] == 0 {
            return -1;
        }
        let v = add(pl::TAG_BENCH_PCT[slot as usize]);
        pl::TAG_BENCH_PCT[slot as usize] = v;
        return melee_core::ppc::fctiwz_f32(v * 1000.0);
    }
    let p = &*pl::Player_GetPtrForSlot(slot);
    let g = p.player_entity[p.transformed[(bench != 0) as usize] as usize];
    if g.is_null() || (*g).user_data.is_null() {
        return -1;
    }
    let f = &mut *((*g).user_data as *mut Fighter);
    let v = add(f.dmg.x1830_percent);
    f.dmg.x1830_percent = v;
    pl::Player_SetHPByIndex(slot, f.is_sub_fighter() as i32, melee_core::ppc::fctiwz_f32(v));
    melee_core::ppc::fctiwz_f32(v * 1000.0)
}

// ---- V2 (envoy-core-v2.md 2.4): speed from a stored base, the orb's absorb, hit attribution, air jumps, blasts

/// The walk / dash attributes each (slot, entity) had before Envoy's first speed write, and the character they
/// belong to (+1; 0 none): `speed` writes `base * m`, so nothing drifts (V1 multiplied and divided back) and a
/// respawned or swapped-in fighter (in-place tag, a new stock) is re-based from its own fresh attributes.
pub static mut BASE: [[[f32; 4]; 2]; 4] = [[[0.0; 4]; 2]; 4];

unsafe fn active_fighter(slot: i32) -> *mut Fighter {
    if !(0..4).contains(&slot) {
        return core::ptr::null_mut();
    }
    let p = pl::Player_GetPtrForSlot(slot);
    if p.is_null() {
        return core::ptr::null_mut();
    }
    let g = (*p).player_entity[(*p).transformed[0] as usize & 1];
    if g.is_null() || (*g).user_data.is_null() { core::ptr::null_mut() } else { (*g).user_data as *mut Fighter }
}

/// The walk and dash multipliers (x1000) of `slot`'s fighter in play (`active`) and its benched partner (`bench`;
/// ignored when it has no fighter object). Absolute: each call sets `base * m`.
pub unsafe fn speed(slot: i32, active: i32, bench: i32) -> i32 {
    if !(0..4).contains(&slot) || crate::tu::stage_hop::check(-1) == crate::tu::stage_hop::HOP_NOT_GAMEPLAY {
        return -1;
    }
    let p = &*pl::Player_GetPtrForSlot(slot);
    let act = p.transformed[0] as usize & 1;
    for (e, f) in fighters(slot).into_iter().enumerate() {
        if f.is_null() {
            continue;
        }
        let m = (if e == act { active } else { bench }).clamp(250, 4000) as f32 / 1000.0;
        let a = &mut (*f).co_attrs;
        let b = &mut BASE[slot as usize][e];
        if b[3] != (*f).kind as f32 + 1.0 {
            *b = [a.walk_max_vel, a.dash_initial_velocity, a.dash_max_velocity, (*f).kind as f32 + 1.0];
        }
        a.walk_max_vel = b[0] * m;
        a.dash_initial_velocity = b[1] * m;
        a.dash_max_velocity = b[2] * m;
    }
    0
}

/// Bulwark's orb (called from the take-damage pre hook that skips the percent): the hit's knockback goes too, so
/// the hit leaves no launch and no flinch (`ftCo_8008EC90` takes a zero knockback as no reaction).
pub unsafe fn absorb(slot: i32) -> i32 {
    let f = active_fighter(slot);
    if f.is_null() {
        return -1;
    }
    (*f).dmg.kb_applied = 0.0;
    0
}

/// Envoy's combo breaker (the sim decides when; envoy_sim `CB_*`): `slot`'s fighter in play bursts out of the combo.
/// Its launch goes (knockback velocity zeroed, the reaction to the hit now skipped), it leaves hitstun into Fall in
/// the air or Wait on the ground, and it is intangible for `frames` with the game's own white intangibility flash
/// (`ftColl_8007B760`), with the shield-break sound, a large quake and a `freeze`-frame hitlag on every fighter.
/// Game state, rolled back with the game. 0 done; -1 no fighter / not in gameplay.
pub unsafe fn combo_break(slot: i32, frames: i32, freeze: i32) -> i32 {
    if crate::tu::stage_hop::check(-1) == crate::tu::stage_hop::HOP_NOT_GAMEPLAY {
        return -1;
    }
    let f = active_fighter(slot);
    if f.is_null() || (*f).gobj.is_null() {
        return -1;
    }
    let g = (*f).gobj;
    (*f).dmg.kb_applied = 0.0;
    (*f).x8c_kb_vel = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    if (*f).self_vel.y < 0.0 {
        (*f).self_vel.y = 0.0;
    }
    if (*f).ground_or_air == GA_Ground {
        melee_fighter::tu::ft_08a1::ft_8008A2BC(g);
    } else {
        melee_fighter::tu::ftCo_Fall::ftCo_Fall_Enter(g);
    }
    melee_fighter::tu::ftcoll::ftcoll_8007b760(g, frames.clamp(0, 600));
    // unmistakable: the shield-break sound from the Melee disc, a large camera quake, and a hitlag freeze on every
    // fighter in play (`freeze` frames, as a hit's hitlag: game state, so rollback replays it the same)
    melee_fighter::ft0877::play_sfx(&mut *f, 130, 127, 64);
    let mut at = (*f).cur_pos;
    melee_world::tu::camera::Camera_RequestQuake(QuakeKind_Large, core::ptr::addr_of_mut!(at) as *mut _);
    let fz = freeze.clamp(0, 30) as f32;
    if fz > 0.0 {
        for s in 0..4 {
            let t = active_fighter(s);
            if t.is_null() || (*t).gobj.is_null() {
                continue;
            }
            if (*t).dmg.x195c_hitlag_frames < fz {
                (*t).dmg.x195c_hitlag_frames = fz;
            }
            if (*t).x2219_b5() == 0 {
                melee_fighter::tu::fighter::Fighter_UnkRecursiveFunc_8006D044((*t).gobj);
            }
        }
    }
    0
}

/// The slot whose hit `slot`'s fighter in play took last (its damage source: a fighter, or an item's owner), or -1.
pub unsafe fn attacker(slot: i32) -> i32 {
    let f = active_fighter(slot);
    if f.is_null() {
        return -1;
    }
    let mut g = (*f).dmg.x1868_source;
    if !g.is_null() && (*g).classifier as i32 == HSD_GOBJ_CLASS_ITEM && !(*g).user_data.is_null() {
        g = (*((*g).user_data as *mut Item)).owner;
    }
    if g.is_null() || (*g).classifier as i32 != HSD_GOBJ_CLASS_FIGHTER || (*g).user_data.is_null() {
        return -1;
    }
    let s = (*((*g).user_data as *mut Fighter)).player_idx as i32;
    if (0..4).contains(&s) { s } else { -1 }
}

/// `slot`'s fighter in play: jumps used | max jumps << 8 (Updraft), or -1.
pub unsafe fn jumps(slot: i32) -> i32 {
    let f = active_fighter(slot);
    if f.is_null() {
        return -1;
    }
    (*f).x1968_jumpsUsed as i32 | ((*f).co_attrs.max_jumps as i32) << 8
}

/// Updraft: give back one used air jump (0 done, -1 none used).
pub unsafe fn air_jump(slot: i32) -> i32 {
    let f = active_fighter(slot);
    if f.is_null() || (*f).x1968_jumpsUsed <= 1 {
        return -1;
    }
    (*f).x1968_jumpsUsed -= 1;
    0
}

/// A drive blast (Nova, Comet, Spire): a hitbox on the owner's fighter in play, on the engine's hit path as Petey's
/// attacks are (`petey::hitboxes`), so it launches with the game's own knockback, hits Petey's body and cages, and
/// its damage goes through the take-damage hook. It lives `BLAST_LIFE` logic frames on a capsule slot the owner's
/// move is not using; teammates are pre-registered as victims, so it never hits a teammate whatever the friendly-fire
/// rule says.
#[derive(Clone, Copy)]
pub struct Blast {
    pub slot: i32,
    pub cap: i32,
    pub left: i32,
    pub x: f32,
    pub y: f32,
    pub hit: HitCapsule,
    /// The most this blast lands on one victim, x1000 (0: no cap). A foe's blast: the sim's foe fire cap.
    pub land_cap: i32,
    /// The sim's tag for the logs (its drive id + 1 | level << 8); 0 none.
    pub tag: i32,
}
impl Blast {
    pub const OFF: Blast = Blast { slot: -1, cap: -1, left: 0, x: 0.0, y: 0.0, hit: unsafe { core::mem::zeroed() }, land_cap: 0, tag: 0 };
}

/// What drive blasts did to a victim slot in the logic frame now (written on the hit path, read by the sim in its
/// take-damage hook of the same frame, cleared in `tick`): the number of blast capsules that landed, what they
/// landed after the cap and before it (x1000), and the last one's owner slot, capsule index, tag and group. `best`:
/// 1 when a blast capsule's knockback won the frame (the blast is what launches), with that knockback x1000.
#[derive(Clone, Copy)]
pub struct Landed {
    pub n: i32,
    pub milli: i32,
    pub raw: i32,
    pub owner: i32,
    pub cap: i32,
    pub tag: i32,
    pub grp: i32,
    pub best: i32,
    pub kb: i32,
}
impl Landed {
    pub const NONE: Landed = Landed { n: 0, milli: 0, raw: 0, owner: -1, cap: -1, tag: 0, grp: 0, best: 0, kb: 0 };
}
pub static mut LANDED: [Landed; 4] = [Landed::NONE; 4];

/// The live blast `hit` belongs to (a capsule of `attacker`), if it is one.
unsafe fn blast_of(attacker: &Fighter, hit: *const HitCapsule) -> Option<usize> {
    if hit.is_null() || (*hit).x4 >> 16 != 0xB1A5 {
        return None;
    }
    let s = attacker.player_idx as i32;
    BLASTS.iter().position(|b| b.left > 0 && b.slot == s && b.hit.x4 == (*hit).x4)
}

/// `melee_fighter::hooks::ENVOY_BLAST_LAND`: the damage of a fighter capsule on `victim`, after every multiplier and
/// right before it joins the victim's damage of the frame. A blast with a cap lands at most the cap, whatever
/// multiplied it on the way (the foe fire cap: rule 2 of envoy-balance.md). Any other capsule: unchanged.
fn blast_land(attacker: &Fighter, hit: *const HitCapsule, victim: &Fighter, dmg: f32) -> f32 {
    unsafe {
        let Some(k) = blast_of(attacker, hit) else { return dmg };
        let b = &BLASTS[k];
        let cap = b.land_cap as f32 / 1000.0;
        let out = if b.land_cap > 0 && dmg > cap { cap } else { dmg };
        let v = victim.player_idx as usize;
        if v < 4 {
            let l = &mut LANDED[v];
            l.n += 1;
            l.milli += melee_core::ppc::fctiwz_f32(out * 1000.0 + 0.5);
            l.raw += melee_core::ppc::fctiwz_f32(dmg * 1000.0 + 0.5);
            l.owner = b.slot;
            l.cap = b.cap;
            l.tag = b.tag;
            l.grp = (b.hit.x4 & 0xFFFF) as i32;
        }
        out
    }
}

/// `melee_fighter::hooks::ENVOY_BLAST_BEST`: the capsule whose knockback won `victim`'s damage log (logs only).
fn blast_best(victim: &Fighter, attacker: &Fighter, hit: *const HitCapsule, kb: f32) {
    unsafe {
        let v = victim.player_idx as usize;
        if v < 4 {
            LANDED[v].best = blast_of(attacker, hit).is_some() as i32;
            LANDED[v].kb = melee_core::ppc::fctiwz_f32(kb * 1000.0);
        }
    }
}

unsafe fn blast_hooks() {
    melee_fighter::hooks::ENVOY_BLAST_LAND = Some(blast_land);
    melee_fighter::hooks::ENVOY_BLAST_BEST = Some(blast_best);
}

/// The sim caps and tags the blast `slot` armed last (this frame): it lands at most `cap_milli` / 1000 percent on a
/// victim (0: no cap). 0 done, -1 no such blast.
pub unsafe fn blast_cap(slot: i32, cap_milli: i32, tag: i32) -> i32 {
    let Some(b) = BLASTS.iter_mut().filter(|b| b.left == BLAST_LIFE && b.slot == slot).max_by_key(|b| b.hit.x4 & 0xFFFF) else { return -1 };
    b.land_cap = cap_milli.max(0);
    b.tag = tag;
    0
}

/// What blasts landed on `slot` this frame (`Landed`, as words): n, landed x1000, before the cap x1000, owner slot,
/// capsule index, tag, group, 1 if a blast launches, the winning knockback x1000.
pub unsafe fn landed(slot: i32) -> [i32; 9] {
    if !(0..4).contains(&slot) {
        return [0, 0, 0, -1, -1, 0, 0, 0, 0];
    }
    let l = LANDED[slot as usize];
    [l.n, l.milli, l.raw, l.owner, l.cap, l.tag, l.grp, l.best, l.kb]
}
pub static mut BLASTS: [Blast; 4] = [Blast::OFF; 4];
static mut BLAST_GRP: u32 = 0;
const BLAST_LIFE: i32 = 3;

unsafe fn blast_offset(f: &Fighter, x: f32, y: f32) -> Option<Vec3> {
    let root = (*f.parts).joint;
    if root.is_null() {
        return None;
    }
    let m = melee_runtime::hsd_mtx::inverse(&(*root).mtx);
    Some(Vec3 { x: m[0][0] * x + m[0][1] * y + m[0][3], y: m[1][0] * x + m[1][1] * y + m[1][3], z: m[2][0] * x + m[2][1] * y + m[2][3] })
}

/// A blast from `slot`'s fighter in play at world (x, y), radius `r`, `dmg` percent, knockback `angle` (361: the
/// Sakurai angle), growth `kbg`, base `bkb`. 0 armed; -1 no fighter / not in gameplay; -2 no free capsule.
pub unsafe fn blast(slot: i32, x: f32, y: f32, r: f32, dmg: i32, angle: i32, kbg: i32, bkb: i32) -> i32 {
    if crate::tu::stage_hop::check(-1) == crate::tu::stage_hop::HOP_NOT_GAMEPLAY {
        return -1;
    }
    let f = active_fighter(slot);
    if f.is_null() {
        return -1;
    }
    let Some(k) = BLASTS.iter().position(|b| b.left <= 0) else { return -2 };
    let busy = |i: usize| BLASTS.iter().any(|b| b.left > 0 && b.slot == slot && b.cap == i as i32);
    let Some(ci) = (0..4).rev().find(|&i| (*f).x914[i].state == HitCapsule_Disabled && !busy(i)) else { return -2 };
    let Some(off) = blast_offset(&*f, x, y) else { return -1 };
    let h = &mut *core::ptr::addr_of_mut!((*f).x914[ci]);
    *h = core::mem::zeroed();
    BLAST_GRP = BLAST_GRP.wrapping_add(1);
    h.x4 = 0xB1A5_0000 | (BLAST_GRP & 0xFFFF);
    h.state = HitCapsule_Enabled;
    h.jobj = (*(*f).parts).joint;
    h.damage = dmg.clamp(0, 99) as f32;
    h.unk_count = dmg.clamp(0, 99) as u32;
    h.kb_angle = angle;
    h.x24 = kbg.max(0) as u32;
    h.x28 = 0;
    h.x2C = bkb.max(0) as u32;
    h.element = 0;
    h.x34 = 0;
    h.sfx_severity = 1;
    h.set_x40_b2(1);
    h.set_x40_b3(1);
    h.set_x42_b5(1);
    h.set_x42_b7(1);
    h.set_x42_b1(1);
    h.b_offset = off;
    h.scale = r;
    h.set_x43_b1(1);
    // the owner's teammates (both tag entities) are victims already: the blast passes through them
    let team = d_team(slot);
    let mut v = 0;
    for s in 0..4 {
        if s == slot || team < 0 || d_team(s) != team {
            continue;
        }
        for t in fighters(s) {
            if !t.is_null() && v < 12 {
                h.victims_1[v].victim = t as *mut core::ffi::c_void;
                v += 1;
            }
        }
    }
    (*f).set_x2219_b3(1);
    BLASTS[k] = Blast { slot, cap: ci as i32, left: BLAST_LIFE, x, y, hit: *h, land_cap: 0, tag: 0 };
    blast_hooks();
    0
}

/// The team of `slot` in the match (-1: no player there).
unsafe fn d_team(slot: i32) -> i32 {
    let f = active_fighter(slot);
    if f.is_null() { -1 } else { (*f).team as i32 }
}

/// Per logic frame (after the post chain, next to `petey::tick`): keep each blast on its capsule for its life (put
/// back if the owner's own state change cleared it and its move left the slot free), then switch it off.
/// Logic frames until the deferred tag partners are created (0: none pending).
pub static mut TAG_DEFER_WAIT: i32 = 0;

pub unsafe fn tick() {
    crate::tu::subspace::sub_tick();
    if TAG_DEFER_WAIT > 0 {
        TAG_DEFER_WAIT -= 1;
        if TAG_DEFER_WAIT == 0 {
            melee_fighter::tu::player::tag_create_deferred();
            melee_fighter::tu::player::TAG_DEFER = false;
        }
    }
    if ARMED != 0 && hud_off() {
        hide_retail_hud();
    }
    LANDED = [Landed::NONE; 4];
    for b in BLASTS.iter_mut() {
        if b.left <= 0 {
            continue;
        }
        b.left -= 1;
        let f = active_fighter(b.slot);
        if f.is_null() {
            b.left = 0;
            continue;
        }
        let h = &mut *core::ptr::addr_of_mut!((*f).x914[b.cap as usize & 3]);
        let ours = h.x4 == b.hit.x4;
        if b.left <= 0 {
            if ours {
                h.state = HitCapsule_Disabled;
            }
            continue;
        }
        if ours && h.state != HitCapsule_Disabled {
            if let Some(off) = blast_offset(&*f, b.x, b.y) {
                h.b_offset = off;
            }
            b.hit = *h;
        } else if h.state == HitCapsule_Disabled {
            *h = b.hit;
            if let Some(off) = blast_offset(&*f, b.x, b.y) {
                h.b_offset = off;
            }
        }
    }
}

/// Envoy draws its own percent, stocks and timer (envoy_ui's HUD): while a run is armed the retail damage plates,
/// their digits and stock icons are hidden (JObj flags only: no game state; with no run armed the HUD is retail).
/// `MELEE_ENVOY_RETAIL_HUD=1` keeps the retail HUD.
pub fn hud_off() -> bool {
    static V: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    !*V.get_or_init(|| std::env::var("MELEE_ENVOY_RETAIL_HUD").is_ok_and(|v| v == "1"))
}
pub unsafe fn hide_retail_hud() {
    use melee_world::tu::ifstatus as ifs;
    // only in a match (the plates are freed at its end and the pointers are not cleared everywhere)
    if crate::tu::stage_hop::check(-1) == crate::tu::stage_hop::HOP_NOT_GAMEPLAY {
        return;
    }
    let h = ifs::ifStatus_GetHUDInfo();
    if h.is_null() {
        return;
    }
    for i in 0..6usize {
        let p = &mut (*h).players[i];
        p.flags.set_hide_all_digits(1);
        for g in [p.HUD_parent_entity, p.next] {
            if !g.is_null() && !(*g).hsd_obj.is_null() {
                melee_runtime::tu::jobj_ptr::HSD_JObjSetFlagsAll((*g).hsd_obj as *mut HSD_JObj, JOBJ_HIDDEN as u32);
            }
        }
    }
}

/// Read-only words for the Envoy HUD (gd_ui_fight_hud): per match slot 0-3 [present, percent, stocks]. Outside a
/// match all zero.
pub unsafe fn hud_words() -> [i32; 16] {
    let mut o = [0i32; 16];
    if crate::tu::stage_hop::check(-1) == crate::tu::stage_hop::HOP_NOT_GAMEPLAY {
        return o;
    }
    for s in 0..4i32 {
        if melee_fighter::player::get_entity(s).is_null() {
            continue;
        }
        o[s as usize * 3] = 1;
        o[s as usize * 3 + 1] = melee_fighter::player::get_damage(s);
        o[s as usize * 3 + 2] = melee_fighter::stale::stocks(s);
    }
    // word 12: the retail pause is up (its overlay owns the top of the screen)
    o[12] = (crate::tu::gmscene::gm_GetDbPauseFlag(1) != 0) as i32;
    o
}
