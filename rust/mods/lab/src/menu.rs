//! M3: the full menu (START hold): tabs (part 9), setting rows (part 10), help pane (part 11); the Dummy tab drives
//! the dummy through `gd_lab_cpu` / `gd_lab_reaction` / `gd_lab_set_percent` (lab commands: online both peers apply
//! them on the same frame, and the other peer's menu follows them through `gd_lab_applied`).
//! M9: the radial quick menu (Z hold, part 13) and the command bar (part 14, keyboard while it is open).
//!
//! Drawn with RECT / SLICE / TEXT records into the HUD's one buffer (no host nodes: one submit, one batch, and the
//! menu works the same offline and in LabNetplay). The game is frozen while the menu is open.

use super::*;

#[link(wasm_import_module = "gd_lab")]
extern "C" {
    fn gd_lab_cpu(port: u32, beh: i32, param: i32) -> i32;
    fn gd_lab_reaction(port: u32, kind: i32, option: i32, param: u32) -> i32;
    fn gd_lab_set_percent(port: u32, pct: f32) -> i32;
    fn gd_lab_set_pos(port: u32, x: f32, y: f32) -> i32;
    fn gd_lab_rec_start(mask: u32) -> u32;
    fn gd_lab_rec_stop(rec: u32) -> i32;
    fn gd_lab_rec_len(rec: u32) -> i32;
    fn gd_lab_rec_free(rec: u32) -> i32;
    fn gd_lab_play(rec: u32, mask: u32, flags: u32) -> i32;
    fn gd_lab_play_stop() -> i32;
    fn gd_lab_play_set(port: u32, entries: *const u32, n: u32, order_flags: u32) -> i32;
}
#[link(wasm_import_module = "gd_ui")]
extern "C" {
    fn gd_ui_keys(out: *mut u32, max: u32) -> i32;
}

// lab command ops seen in gd_lab_applied (the other peer's dummy changes)
pub const OP_CPU: u8 = 13;
pub const OP_REACTION: u8 = 14;
const RK_DI: i32 = 0;
const RK_SDI: i32 = 1;
const RK_TECH: i32 = 2;
const RK_GETUP: i32 = 3;
const RK_LEDGE: i32 = 4;
const RK_SHIELD: i32 = 5;
const RK_OOS: i32 = 6;
const PLAY_LOOP: u32 = 1;
const PLAY_MIRROR: u32 = 2;
const PLAY_FROM_SLOT: u32 = 4;

const KEY_ENTER: u32 = 1;
const KEY_ESCAPE: u32 = 2;
const KEY_BACKSPACE: u32 = 3;
const KEY_UP: u32 = 4;
const KEY_DOWN: u32 = 5;

// ------------------------------------------------------------------------------------------------ the rows

#[derive(Clone, Copy, PartialEq)]
pub enum K {
    /// pick one: (label, value sent)
    Choice(&'static [(&'static str, i32)]),
    /// lo, hi, step, unit
    Slider(i32, i32, i32, &'static str),
    /// a slider whose value is sent on A
    SliderApply(i32, i32, i32, &'static str),
    Toggle,
    Action,
}

pub struct RowDef {
    pub id: usize,
    pub name: &'static str,
    pub kind: K,
    /// the help pane, lines split at '|'
    pub help: &'static str,
}

// dummy settings (per port, ids < D_N)
pub const D_CONTROL: usize = 0;
pub const D_LEVEL: usize = 1;
pub const D_BLOCK: usize = 2;
pub const D_BLOCK_PCT: usize = 3;
pub const D_OOS: usize = 4;
pub const D_OOS_DELAY: usize = 5;
pub const D_TECH: usize = 6;
pub const D_DI: usize = 7;
pub const D_DI_STR: usize = 8;
pub const D_SDI: usize = 9;
pub const D_SDI_EVERY: usize = 10;
pub const D_GETUP: usize = 11;
pub const D_LEDGE: usize = 12;
pub const D_WAKE_DELAY: usize = 13;
pub const D_PERCENT: usize = 14;
pub const D_N: usize = 15;
pub const D_DEFAULT: [i32; D_N] = [0, 9, 0, 50, 0, 0, 0, 0, 100, 0, 2, 0, 0, 0, 0];
// local settings
const R_SLOT: usize = 15;
const R_PORT: usize = 16;
const R_REC: usize = 17;
const R_LOOP: usize = 18;
const R_MIRROR: usize = 19;
const R_FROM: usize = 20;
const R_PLAY: usize = 21;
const R_STOP: usize = 22;
const R_WEIGHT: usize = 23;
const R_ORDER: usize = 24;
const R_PLAYALL: usize = 25;
const R_FREE: usize = 26;
const V_PRESET: usize = 27;
pub const V_HIT: usize = 28;
const V_HURT: usize = 29;
const V_ECB: usize = 30;
const V_LEDGE: usize = 31;
const V_DI: usize = 32;
pub const H_METER: usize = 33;
pub const H_MOVE: usize = 34;
pub const H_STATE: usize = 35;
pub const H_DUMMY: usize = 36;
pub const H_COMBO: usize = 37;
pub const H_INPUT: usize = 38;
pub const H_HINTS: usize = 39;
const S_SLOT: usize = 40;
const S_SAVE: usize = 41;
const S_LOAD: usize = 42;
const S_RESET: usize = 43;
const S_SWAP: usize = 44;
const S_CMD: usize = 45;
const S_RESUME: usize = 46;
const S_CONTROLS: usize = 47;
pub const H_LOOK: usize = 48;
const S_LOOK_LIVE: usize = 49;
const S_LOOK_BUDGET: usize = 50;
pub const N_IDS: usize = 51;

pub const CONTROL: &[(&str, i32)] = &[("Live pad", 0), ("Stand", 1), ("Crouch", 2), ("Jump", 3), ("Shield", 4), ("CPU level", 5), ("Recording", 6)];
pub const BLOCK: &[(&str, i32)] = &[("None", 0), ("All", 1), ("After first hit", 2), ("Random", 3)];
pub const OOS: &[(&str, i32)] = &[("None", 0), ("Jump", 1), ("Nair", 2), ("Grab", 3), ("Up-B", 4), ("Spot-dodge", 5), ("Roll", 6), ("Random", 8)];
pub const TECH: &[(&str, i32)] = &[("None", 0), ("In place", 1), ("Roll in", 2), ("Roll away", 3), ("Miss", 4), ("Random", 5)];
pub const DI: &[(&str, i32)] = &[("None", 0), ("In", 1), ("Out", 2), ("Away", 3), ("Toward", 4), ("Up", 6), ("Down", 7), ("Random", 5)];
pub const SDI: &[(&str, i32)] = &[("None", 0), ("In", 1), ("Away", 3), ("Up", 6), ("Down", 7), ("Random", 5)];
const GETUP: &[(&str, i32)] = &[("None", 0), ("Stand", 1), ("Roll in", 2), ("Roll away", 3), ("Attack", 4), ("Random", 5)];
const LEDGE: &[(&str, i32)] = &[("None", 0), ("Stand", 1), ("Roll", 2), ("Attack", 4), ("Jump", 6), ("Drop", 7), ("Random", 5)];
const PORTS: &[(&str, i32)] = &[("P1", 0), ("P2", 1)];
const ORDER: &[(&str, i32)] = &[("Ordered", 0), ("Random", 1), ("Weighted", 2)];
const PRESETS: &[(&str, i32)] = &[("Clean", 0), ("Frame data", 1), ("Full", 2)];
const SLOTS8: &[(&str, i32)] = &[("1", 0), ("2", 1), ("3", 2), ("4", 3), ("5", 4), ("6", 5), ("7", 6), ("8", 7)];

const fn r(id: usize, name: &'static str, kind: K, help: &'static str) -> RowDef {
    RowDef { id, name, kind, help }
}

pub const DUMMY_ROWS: &[RowDef] = &[
    r(D_CONTROL, "Control", K::Choice(CONTROL), "What drives the dummy between reactions.|Live pad: its own controller (or nobody;|a CPU slot: its AI). Recording: loops the|Record tab's slot. On a CPU slot every option|but CPU level takes over from the AI."),
    r(D_LEVEL, "CPU level", K::Slider(1, 9, 1, ""), "The retail CPU level, for Control: CPU level.|The port must be a CPU at character select."),
    r(D_BLOCK, "Block", K::Choice(BLOCK), "Shield while you attack nearby.|After first hit: blocks for 2 s after a hit.|Random: blocks at the chance below."),
    r(D_BLOCK_PCT, "Block chance", K::Slider(10, 100, 10, "%"), "How often Block: Random shields."),
    r(D_OOS, "After shield", K::Choice(OOS), "What the dummy does when shield stun ends:|jump, nair, grab, up-B, spot-dodge or roll."),
    r(D_OOS_DELAY, "OoS delay", K::Slider(0, 30, 1, "f"), "Frames the dummy keeps shielding before|its out-of-shield option."),
    r(D_TECH, "Tech", K::Choice(TECH), "What the dummy does when it lands after|being hit: tech in place, roll in, roll away,|or miss."),
    r(D_DI, "DI", K::Choice(DI), "Directional influence on the last hitlag|frame. In / out are toward or away from|the stage centre; away / toward the attacker."),
    r(D_DI_STR, "DI strength", K::Slider(10, 100, 10, "%"), "How far the stick is pushed for DI and SDI."),
    r(D_SDI, "SDI", K::Choice(SDI), "Smash DI during hitlag: a fresh push|every few frames (below)."),
    r(D_SDI_EVERY, "SDI every", K::Slider(1, 8, 1, "f"), "Frames between SDI pushes."),
    r(D_GETUP, "Getup", K::Choice(GETUP), "From lying down after a missed tech:|stand, roll in, roll away or getup attack."),
    r(D_LEDGE, "Ledge", K::Choice(LEDGE), "From hanging on the ledge: stand, roll,|attack, jump or drop."),
    r(D_WAKE_DELAY, "Getup delay", K::Slider(0, 60, 1, "f"), "Frames the dummy waits on the ground or|the ledge before its option."),
    r(D_PERCENT, "Percent", K::SliderApply(0, 300, 10, "%"), "Pick a percent, A sets it on the dummy now.|Setting it forks the timeline."),
];
const RECORD_ROWS: &[RowDef] = &[
    r(R_SLOT, "Slot", K::Choice(SLOTS8), "The recording slot the rows below use."),
    r(R_PORT, "Record port", K::Choice(PORTS), "Whose inputs are recorded."),
    r(R_REC, "Record", K::Action, "A starts recording (the menu closes and the|game runs); A again, or the radial, stops."),
    r(R_LOOP, "Loop", K::Toggle, "Playback starts over when it ends."),
    r(R_MIRROR, "Mirror", K::Toggle, "Play the recording for the other side|(stick X flipped)."),
    r(R_FROM, "From start state", K::Toggle, "Playback first loads the state the|recording started from."),
    r(R_PLAY, "Play on dummy", K::Action, "The dummy plays this slot."),
    r(R_STOP, "Stop playback", K::Action, "Every port goes back to its own input."),
    r(R_WEIGHT, "Weight", K::Slider(0, 9, 1, ""), "How often Play all picks this slot when|the order is Weighted (shown as x2)."),
    r(R_ORDER, "Order", K::Choice(ORDER), "How Play all picks the next recording:|in order, at random, or by weight.|Random draws from the lab RNG (synced online)."),
    r(R_PLAYALL, "Play all", K::Action, "The dummy loops through every recorded slot."),
    r(R_FREE, "Delete slot", K::Action, "Frees this slot's recording."),
];
const DISPLAY_ROWS: &[RowDef] = &[
    r(V_PRESET, "HUD preset", K::Choice(PRESETS), "Clean: the chip and toasts. Frame data:|meter and move readout. Full: every panel.|Z + D-up cycles it during play."),
    r(V_HIT, "Hitboxes", K::Toggle, "Draw hitboxes on the fighters."),
    r(V_HURT, "Hurtboxes", K::Toggle, "Draw hurtboxes."),
    r(V_ECB, "ECB", K::Toggle, "Draw the environment collision box."),
    r(V_LEDGE, "Ledge grab", K::Toggle, "Draw the ledge grab boxes."),
    r(V_DI, "DI line", K::Toggle, "Draw the launch direction after DI."),
];
const HUD_ROWS: &[RowDef] = &[
    r(H_METER, "Frame meter", K::Toggle, "One cell per frame for both players and|the frame advantage."),
    r(H_MOVE, "Move readout", K::Toggle, "P1's last move: startup, active, landing,|damage, IASA, on shield."),
    r(H_STATE, "State readout", K::Toggle, "P1's action, frame, speed and hitlag."),
    r(H_DUMMY, "Dummy panel", K::Toggle, "What the dummy is set to."),
    r(H_COMBO, "Combo panel", K::Toggle, "Hits, damage and the move chain."),
    r(H_INPUT, "Input display", K::Toggle, "P1's stick, buttons and input history."),
    r(H_HINTS, "Button hints", K::Toggle, "The key caps at the bottom right."),
    r(H_LOOK, "Lookahead", K::Toggle, "True frame advantage and the DI fan,|simulated ahead on every hit and shield."),
];
const SESSION_ROWS: &[RowDef] = &[
    r(S_SLOT, "Save slot", K::Slider(1, 16, 1, ""), "The slot D-down saves to and D-up loads."),
    r(S_SAVE, "Save state", K::Action, "Save the game to the slot (D-down in play)."),
    r(S_LOAD, "Load state", K::Action, "Load the slot (D-up in play)."),
    r(S_RESET, "Reset positions", K::Action, "Both fighters back to the middle, P1 left."),
    r(S_SWAP, "Swap sides", K::Action, "P1 and P2 trade places."),
    r(S_CMD, "Command bar", K::Action, "Type part of a setting or action and press|Enter (keyboard). X opens it from any tab."),
    r(S_LOOK_LIVE, "Lookahead live", K::Toggle, "Simulate ahead while playing too (else|only when frozen). Costs frame time."),
    r(S_LOOK_BUDGET, "Lookahead budget", K::Slider(1, 8, 1, " ms"), "Time per frame the live lookahead may use."),
    r(S_CONTROLS, "Controls", K::Action, "Every LAB button, on one page.|B closes it."),
    r(S_RESUME, "Resume", K::Action, "Close the menu (B or START does too)."),
];
pub const TABS: [(&str, &[RowDef]); 5] = [("Dummy", DUMMY_ROWS), ("Record", RECORD_ROWS), ("Display", DISPLAY_ROWS), ("HUD", HUD_ROWS), ("Session", SESSION_ROWS)];

/// Every LAB control: (buttons, what they do). The one list of them: the Controls page (Session tab) draws it and
/// lib.rs's header points here, so the two cannot drift.
pub const CONTROLS: &[(&str, &str)] = &[
    ("START tap", "freeze / resume"),
    ("D-left / D-right", "frozen: step -1 / +1 frame (hold to repeat)"),
    ("Z + L / Z + R", "frozen: step -10 / +10 frames"),
    ("Z + A", "frozen: take over (go live from this frame)"),
    ("D-down", "save to the current slot (on release)"),
    ("D-up", "load the current slot"),
    ("Z + D-left / D-right", "pick the save slot"),
    ("Z + D-up", "next HUD preset"),
    ("D-down + START", "the settings overlay"),
    ("Hold START", "this menu"),
    ("Hold Z (frozen)", "the radial quick menu: stick picks, release runs"),
    ("X (in the menu)", "the command bar: type, Enter runs"),
    ("L / R", "menu: previous / next tab"),
    ("Up / down", "menu: row"),
    ("Left / right", "menu: change the value"),
    ("A", "menu: apply / run"),
    ("B / START", "menu: close (B closes this page)"),
    ("L + R + A + START", "the retail pause (quit the match)"),
    ("HUD tab: Lookahead", "the lookahead panel: true advantage, DI fan on hit / shield"),
    ("Session: Lookahead live", "also simulate ahead while playing (budget: ms per frame)"),
];

/// The radial's 8 actions, clockwise from up (spec 4).
const RADIAL: [&str; 8] = ["Save", "Load", "Reset", "Record", "Play", "Mirror", "Swap", "Menu"];

// ------------------------------------------------------------------------------------------------ state

pub struct Menu {
    pub open: bool,
    pub port: u32,
    pub tab: usize,
    pub row: usize,
    pub scroll: usize,
    /// resume the game when the menu closes (it was running when START went down)
    pub resume: bool,
    pub hold: (u16, u32),
    // radial
    pub radial: bool,
    pub rport: u32,
    pub rsel: i32,
    pub z_frames: u32,
    pub z_combo: bool,
    // command bar
    pub cmd: bool,
    pub kcap: u32,
    pub q: Txt,
    pub hits: [(u8, u8, i8); 8],
    pub n_hits: usize,
    pub csel: usize,
    // recordings: slot -> handle (0 empty), the open recording's slot
    pub recs: [u32; 8],
    pub weights: [i32; 8],
    pub recording: i32,
    pub keys: [u32; 32],
    /// the Controls page is open
    pub controls: bool,
}

impl Menu {
    pub const fn new() -> Menu {
        Menu {
            open: false, port: 0, tab: 0, row: 0, scroll: 0, resume: false, hold: (0, 0), radial: false, rport: 0, rsel: -1, z_frames: 0, z_combo: false,
            cmd: false, kcap: 0, q: Txt::new(), hits: [(0, 0, 0); 8], n_hits: 0, csel: 0, recs: [0; 8], weights: [1; 8], recording: -1, keys: [0; 32], controls: false,
        }
    }
}

fn find(t: &[(&str, i32)], v: i32) -> i32 {
    t.iter().position(|o| o.1 == v).map_or(0, |i| i as i32)
}

/// ASCII case-insensitive substring.
fn contains(h: &str, n: &str) -> bool {
    let (h, n) = (h.as_bytes(), n.as_bytes());
    if n.is_empty() {
        return true;
    }
    h.windows(n.len()).any(|w| w.iter().zip(n).all(|(a, b)| a.to_ascii_lowercase() == b.to_ascii_lowercase()))
}

impl Lab {
    /// The port the Dummy tab edits: the opener's opponent. Online the opener is this peer's player (the pads the
    /// mod reads are this machine's controllers, not the match ports).
    pub fn dport(&self) -> usize {
        let me = if self.online { self.status[8] as usize } else { self.menu.port as usize } & 3;
        let pref = 1 - (me & 1);
        // P1 + P3 (or a dummy on P3 / P4): the first other port with a fighter, so a setting never goes nowhere
        let present = |p: usize| unsafe { gd_lab_fighter(p as u32, core::ptr::null_mut(), 0) } >= 0;
        if present(pref) {
            return pref;
        }
        (0..4).find(|&p| p != me && present(p)).unwrap_or(pref)
    }

    pub fn val(&self, id: usize) -> i32 {
        if id < D_N {
            self.dv[self.dport()][id]
        } else if id == R_WEIGHT {
            self.menu.weights[self.vals[R_SLOT] as usize & 7]
        } else {
            self.vals[id]
        }
    }

    fn set_val(&mut self, id: usize, v: i32) {
        if id < D_N {
            let p = self.dport();
            self.dv[p][id] = v;
        } else if id == R_WEIGHT {
            self.menu.weights[self.vals[R_SLOT] as usize & 7] = v;
        } else {
            self.vals[id] = v;
        }
    }

    pub fn menu_init(&mut self) {
        self.dv = [D_DEFAULT; 4];
        self.vals[R_LOOP] = 1;
        self.vals[S_SLOT] = 1;
        self.vals[S_LOOK_BUDGET] = 4;
    }

    /// The lookahead settings (local: never a command): auto on hit / shield, live per the toggle.
    pub fn look_cfg(&mut self) {
        let live = if self.vals[S_LOOK_LIVE] != 0 { LOOK_LIVE } else { 0 };
        unsafe { gd_lab_lookahead_cfg(LOOK_AUTO | live, self.vals[S_LOOK_BUDGET].clamp(1, 8) as u32 * 1000) };
    }

    /// Show mask and overlay toggles from the preset (spec 5.1).
    pub fn preset_widgets(&mut self) {
        let p = self.preset;
        let fd = p != CLEAN;
        let full = p == FULL;
        self.vals[V_PRESET] = p as i32;
        self.vals[H_METER] = fd as i32;
        self.vals[H_MOVE] = fd as i32;
        for id in [H_STATE, H_DUMMY, H_COMBO, H_INPUT] {
            self.vals[id] = full as i32;
        }
        self.vals[H_HINTS] = (p != FRAMEDATA) as i32;
        self.vals[H_LOOK] = fd as i32;
        self.vals[V_HIT] = full as i32;
        self.apply_overlay();
    }

    fn apply_overlay(&mut self) {
        let bits = [(V_HIT, 1u32), (V_HURT, 2), (V_ECB, 4), (V_LEDGE, 8), (V_DI, 1 << 13)].iter().fold(0, |m, &(id, b)| if self.vals[id] != 0 { m | b } else { m });
        unsafe { gd_lab_overlay(0x3, bits) };
    }

    // -------------------------------------------------------------------------------------------- dummy commands

    /// Send what the row `id` of the dummy on `port` is set to (a lab command).
    fn send_dummy(&mut self, port: usize, id: usize) {
        let d = self.dv[port];
        let opt = |t: &[(&str, i32)], i: usize| t[(d[i].max(0) as usize).min(t.len() - 1)].1;
        let p = port as u32;
        unsafe {
            match id {
                D_CONTROL | D_LEVEL => {
                    let beh = opt(CONTROL, D_CONTROL);
                    let param = match beh {
                        5 => d[D_LEVEL],
                        6 => {
                            let h = self.menu.recs[self.vals[R_SLOT] as usize & 7];
                            if h == 0 {
                                let mut m = Txt::new();
                                m.s("Record slot ").i(self.vals[R_SLOT] + 1).s(" is empty");
                                self.toast(&m, None);
                                return;
                            }
                            h as i32
                        }
                        _ => 0,
                    };
                    gd_lab_cpu(p, beh, param);
                }
                D_BLOCK | D_BLOCK_PCT => {
                    gd_lab_reaction(p, RK_SHIELD, opt(BLOCK, D_BLOCK), (d[D_BLOCK_PCT] as u32 & 0xFF) << 16);
                }
                D_OOS | D_OOS_DELAY => {
                    gd_lab_reaction(p, RK_OOS, opt(OOS, D_OOS), d[D_OOS_DELAY] as u32 & 0xFF);
                }
                D_TECH => {
                    gd_lab_reaction(p, RK_TECH, opt(TECH, D_TECH), 0);
                }
                D_DI | D_DI_STR => {
                    gd_lab_reaction(p, RK_DI, opt(DI, D_DI), (d[D_DI_STR] as u32 & 0xFF) << 8);
                }
                D_SDI | D_SDI_EVERY => {
                    gd_lab_reaction(p, RK_SDI, opt(SDI, D_SDI), (d[D_SDI_EVERY] as u32 & 0xFF) | (d[D_DI_STR] as u32 & 0xFF) << 8);
                }
                D_GETUP | D_LEDGE | D_WAKE_DELAY => {
                    if id != D_LEDGE {
                        gd_lab_reaction(p, RK_GETUP, opt(GETUP, D_GETUP), d[D_WAKE_DELAY] as u32 & 0xFF);
                    }
                    if id != D_GETUP {
                        gd_lab_reaction(p, RK_LEDGE, opt(LEDGE, D_LEDGE), d[D_WAKE_DELAY] as u32 & 0xFF);
                    }
                }
                D_PERCENT => {
                    gd_lab_set_percent(p, d[D_PERCENT] as f32);
                }
                _ => {}
            }
        }
    }

    /// A dummy command applied this frame (either peer's): the menus of both peers show the same settings.
    pub fn applied_dummy(&mut self, op: u8, port: usize, a: u32, b: u32, c: u32) {
        if port > 1 {
            return;
        }
        let d = &mut self.dv[port];
        if op == OP_CPU {
            d[D_CONTROL] = find(CONTROL, a as i32);
            if a == 5 {
                d[D_LEVEL] = b as i32;
            }
            return;
        }
        let (o, delay, strength, chance) = (b as i32, (c & 0xFF) as i32, ((c >> 8) & 0xFF) as i32, ((c >> 16) & 0xFF) as i32);
        match a as i32 {
            RK_SHIELD => {
                d[D_BLOCK] = find(BLOCK, o);
                if chance > 0 {
                    d[D_BLOCK_PCT] = chance;
                }
            }
            RK_OOS => {
                d[D_OOS] = find(OOS, o);
                d[D_OOS_DELAY] = delay;
            }
            RK_TECH => d[D_TECH] = find(TECH, o),
            RK_DI => {
                d[D_DI] = find(DI, o);
                if strength > 0 {
                    d[D_DI_STR] = strength;
                }
            }
            RK_SDI => {
                d[D_SDI] = find(SDI, o);
                d[D_SDI_EVERY] = delay.max(1);
            }
            RK_GETUP => {
                d[D_GETUP] = find(GETUP, o);
                d[D_WAKE_DELAY] = delay;
            }
            RK_LEDGE => {
                d[D_LEDGE] = find(LEDGE, o);
                d[D_WAKE_DELAY] = delay;
            }
            _ => {}
        }
    }

    // -------------------------------------------------------------------------------------------- actions

    fn rec_slot(&self) -> usize {
        self.vals[R_SLOT] as usize & 7
    }

    fn play_flags(&self) -> u32 {
        (if self.vals[R_LOOP] != 0 { PLAY_LOOP } else { 0 }) | (if self.vals[R_MIRROR] != 0 { PLAY_MIRROR } else { 0 }) | (if self.vals[R_FROM] != 0 { PLAY_FROM_SLOT } else { 0 })
    }

    fn toast_slot(&mut self, a: &str, n: i32, b: &str) {
        let mut m = Txt::new();
        m.s(a).i(n).s(b);
        self.toast(&m, None);
    }

    pub fn toggle_record(&mut self) {
        let s = self.rec_slot();
        unsafe {
            if self.menu.recording >= 0 {
                let rs = self.menu.recording as usize;
                gd_lab_rec_stop(self.menu.recs[rs]);
                self.menu.recording = -1;
                self.toast_slot("Recorded slot ", rs as i32 + 1, "");
                return;
            }
            if self.menu.recs[s] != 0 {
                gd_lab_rec_free(self.menu.recs[s]);
            }
            let h = gd_lab_rec_start(1 << (self.vals[R_PORT] as u32 & 1));
            self.menu.recs[s] = h;
            if h != 0 {
                self.menu.recording = s as i32;
                let mut m = Txt::new();
                m.s("● REC ").i(s as i32 + 1).s(" · P").i(self.vals[R_PORT] + 1);
                self.toast(&m, None);
            } else {
                self.toast_s("Recording refused");
            }
        }
    }

    fn play(&mut self, mirror: bool) {
        let s = self.rec_slot();
        let h = self.menu.recs[s];
        if h == 0 {
            self.toast_slot("Record slot ", s as i32 + 1, " is empty");
            return;
        }
        let mut f = self.play_flags();
        if mirror {
            f |= PLAY_MIRROR;
        }
        unsafe { gd_lab_play(h, 1 << self.dport(), f) };
        self.toast_slot(if mirror { "Mirror slot " } else { "Playing slot " }, s as i32 + 1, "");
    }

    fn play_all(&mut self) {
        let mut e = [0u32; 16];
        let mut n = 0;
        for s in 0..8 {
            if self.menu.recs[s] != 0 {
                e[n * 2] = self.menu.recs[s];
                e[n * 2 + 1] = self.menu.weights[s].clamp(0, 9) as u32;
                n += 1;
            }
        }
        if n == 0 {
            self.toast_s("No recordings");
            return;
        }
        let order = self.vals[R_ORDER] as u32 | (self.play_flags() & !PLAY_LOOP) << 8;
        unsafe { gd_lab_play_set(self.dport() as u32, e.as_ptr(), n as u32, order) };
        self.toast_slot("Playing ", n as i32, " recordings");
    }

    fn reset_pos(&mut self) {
        unsafe {
            gd_lab_set_pos(0, -15.0, 10.0);
            gd_lab_set_pos(1, 15.0, 10.0);
        }
        self.toast_s("Reset positions");
    }

    fn swap(&mut self) {
        if !(self.have[0] && self.have[1]) {
            return;
        }
        let (a, b) = (self.fighter[0], self.fighter[1]);
        unsafe {
            gd_lab_set_pos(0, rd_f32(&b, F_POS_X), rd_f32(&b, F_POS_Y) + 1.0);
            gd_lab_set_pos(1, rd_f32(&a, F_POS_X), rd_f32(&a, F_POS_Y) + 1.0);
        }
        self.toast_s("Swapped sides");
    }

    fn save_slot(&mut self) {
        self.slot = (self.vals[S_SLOT] - 1).clamp(0, 15) as u32;
        unsafe { gd_lab_save(self.slot) };
        self.toast_slot("Saved slot ", self.slot as i32 + 1, "");
    }

    fn load_slot(&mut self) {
        self.slot = (self.vals[S_SLOT] - 1).clamp(0, 15) as u32;
        if self.slots[self.slot as usize] < 0 {
            self.toast_slot("Slot ", self.slot as i32 + 1, " is empty");
        } else {
            unsafe { gd_lab_load(self.slot) };
            self.toast_slot("Loaded slot ", self.slot as i32 + 1, "");
        }
    }

    /// A on a row (or the command bar's Enter).
    fn activate(&mut self, id: usize) {
        match id {
            D_PERCENT => {
                let p = self.dport();
                self.send_dummy(p, D_PERCENT);
                let mut m = Txt::new();
                m.s("P").i(p as i32 + 1).s(" ").i(self.dv[p][D_PERCENT]).s("%");
                self.toast(&m, None);
            }
            R_REC => {
                let was = self.menu.recording >= 0;
                self.toggle_record();
                if !was && self.menu.recording >= 0 {
                    // recording needs the game running
                    self.menu.resume = true;
                    self.close_menu();
                }
            }
            R_PLAY => self.play(false),
            R_STOP => {
                unsafe { gd_lab_play_stop() };
                self.toast_s("Playback stopped");
            }
            R_PLAYALL => self.play_all(),
            R_FREE => {
                let s = self.rec_slot();
                if self.menu.recs[s] != 0 {
                    unsafe { gd_lab_rec_free(self.menu.recs[s]) };
                    self.menu.recs[s] = 0;
                    self.toast_slot("Deleted slot ", s as i32 + 1, "");
                }
            }
            S_SAVE => self.save_slot(),
            S_LOAD => self.load_slot(),
            S_RESET => self.reset_pos(),
            S_SWAP => self.swap(),
            S_CMD => self.open_cmd(),
            S_RESUME => self.close_menu(),
            S_CONTROLS => self.menu.controls = true,
            _ => {}
        }
    }

    /// A value changed: what it drives.
    fn changed(&mut self, id: usize) {
        match id {
            0..=13 => {
                let p = self.dport();
                self.send_dummy(p, id);
            }
            V_PRESET => {
                self.preset = self.vals[V_PRESET] as u8;
                self.chip_key = u32::MAX;
                self.preset_widgets();
            }
            V_HIT..=V_DI => self.apply_overlay(),
            S_SLOT => self.slot = (self.vals[S_SLOT] - 1).clamp(0, 15) as u32,
            S_LOOK_LIVE | S_LOOK_BUDGET => self.look_cfg(),
            _ => {}
        }
    }

    /// Step a row's value by `d` (choice wraps, slider clamps, toggle flips).
    fn step(&mut self, id: usize, kind: K, d: i32) {
        let v = self.val(id);
        let nv = match kind {
            K::Choice(t) => (v + d).rem_euclid(t.len() as i32),
            K::Slider(lo, hi, st, _) | K::SliderApply(lo, hi, st, _) => (v + d * st).clamp(lo, hi),
            K::Toggle => (v == 0) as i32,
            K::Action => return,
        };
        if nv != v {
            self.set_val(id, nv);
            self.changed(id);
        }
    }

    // -------------------------------------------------------------------------------------------- open / close

    pub fn open_menu(&mut self, port: u32, resume: bool) {
        self.menu.open = true;
        self.menu.port = port;
        self.menu.resume = resume;
        self.menu.hold = (0, 0);
        self.rec_lens();
    }

    pub fn close_menu(&mut self) {
        if self.menu.cmd {
            self.close_cmd();
        }
        self.menu.open = false;
        self.menu.controls = false;
        if self.menu.resume {
            unsafe { gd_lab_pause(0) };
        }
        self.menu.resume = false;
    }

    fn rec_lens(&mut self) {
        for s in 0..8 {
            let h = self.menu.recs[s];
            self.rec_len[s] = if h == 0 { -1 } else { unsafe { gd_lab_rec_len(h) } };
        }
    }

    fn open_cmd(&mut self) {
        self.menu.cmd = true;
        self.menu.q.clear();
        self.menu.csel = 0;
        self.menu.kcap = unsafe { gd_ui_capture(0, CAP_KEYBOARD) };
        self.search();
    }

    fn close_cmd(&mut self) {
        self.menu.cmd = false;
        if self.menu.kcap != 0 {
            unsafe { gd_ui_release(self.menu.kcap) };
            self.menu.kcap = 0;
        }
    }

    // -------------------------------------------------------------------------------------------- input

    /// The menu's pad input (the opener's port). True while the menu holds the controls.
    pub fn menu_input(&mut self, held: u16, edge: u16, stick: (i8, i8)) -> bool {
        if self.menu.cmd {
            self.cmd_keys();
            if edge & BB != 0 {
                self.close_cmd();
            }
            return true;
        }
        if self.menu.controls {
            if edge & (BB | START) != 0 {
                self.menu.controls = false;
            }
            return true;
        }
        let rows = TABS[self.menu.tab].1;
        // stick or D-pad, with repeat (15 frames, then every 4)
        let mut dir = held & (DL | DR | DU | DD);
        if stick.1 > 50 { dir |= DU; }
        if stick.1 < -50 { dir |= DD; }
        if stick.0 > 50 { dir |= DR; }
        if stick.0 < -50 { dir |= DL; }
        let fire = if dir != 0 && dir == self.menu.hold.0 {
            self.menu.hold.1 += 1;
            self.menu.hold.1 >= 15 && (self.menu.hold.1 - 15) % 4 == 0
        } else {
            self.menu.hold = (dir, 0);
            dir != 0
        };
        if edge & (BB | START) != 0 {
            self.close_menu();
            return true;
        }
        if edge & BL != 0 {
            self.menu.tab = (self.menu.tab + TABS.len() - 1) % TABS.len();
            self.menu.row = 0;
            self.menu.scroll = 0;
            return true;
        }
        if edge & BR != 0 {
            self.menu.tab = (self.menu.tab + 1) % TABS.len();
            self.menu.row = 0;
            self.menu.scroll = 0;
            return true;
        }
        if edge & BX != 0 {
            self.open_cmd();
            return true;
        }
        let rd = &rows[self.menu.row];
        if fire {
            if dir & DU != 0 {
                self.menu.row = (self.menu.row + rows.len() - 1) % rows.len();
            } else if dir & DD != 0 {
                self.menu.row = (self.menu.row + 1) % rows.len();
            } else if dir & DL != 0 {
                self.step(rd.id, rd.kind, -1);
            } else if dir & DR != 0 {
                self.step(rd.id, rd.kind, 1);
            }
            if rd.id == R_SLOT {
                self.rec_lens();
            }
        }
        if edge & BA != 0 {
            match rd.kind {
                K::Toggle => self.step(rd.id, rd.kind, 1),
                K::Action | K::SliderApply(..) => self.activate(rd.id),
                // the footer says "A apply": a dummy row re-sends its value (left / right already sent it) and says
                // which port it went to, so a setting that seems to do nothing shows where it landed
                _ if rd.id < D_N => {
                    let p = self.dport();
                    self.send_dummy(p, rd.id);
                    let mut m = Txt::new();
                    m.s("P").i(p as i32 + 1).s(": ").s(rd.name).s(" applied");
                    self.toast(&m, None);
                }
                _ => {}
            }
            self.rec_lens();
        }
        true
    }

    /// Z hold: the radial. Returns true while it holds the controls.
    pub fn radial_input(&mut self, port: u32, held: u16, edge: u16, stick: (i8, i8)) -> bool {
        if !self.menu.radial {
            return false;
        }
        if port != self.menu.rport {
            return true;
        }
        let (sx, sy) = (stick.0 as i32, stick.1 as i32);
        if sx * sx + sy * sy > 60 * 60 {
            // 8 sectors clockwise from up
            let a = libm_atan2(sx as f32, sy as f32); // 0 = up, clockwise positive
            let s = ((a / (core::f32::consts::PI / 4.0)) + 8.5) as i32 % 8;
            self.menu.rsel = s;
        }
        if edge & BB != 0 {
            self.menu.radial = false;
            return true;
        }
        if held & BZ == 0 {
            self.menu.radial = false;
            let s = self.menu.rsel;
            if s >= 0 {
                self.radial_run(s as usize, port);
            }
        }
        true
    }

    fn radial_run(&mut self, s: usize, port: u32) {
        match RADIAL[s] {
            "Save" => {
                unsafe { gd_lab_save(self.slot) };
                self.toast_slot("Saved slot ", self.slot as i32 + 1, "");
            }
            "Load" => {
                self.vals[S_SLOT] = self.slot as i32 + 1;
                self.load_slot();
            }
            "Reset" => self.reset_pos(),
            "Record" => {
                // this player's own inputs (online the pads are this machine's, the port is the peer's player)
                self.vals[R_PORT] = if self.online { self.status[8] } else { port as i32 } & 1;
                self.toggle_record();
            }
            "Play" => {
                self.menu.port = port;
                self.play(false);
            }
            "Mirror" => {
                self.menu.port = port;
                self.play(true);
            }
            "Swap" => self.swap(),
            _ => {
                let resume = !self.paused;
                if resume {
                    unsafe { gd_lab_pause(1) };
                }
                self.open_menu(port, resume);
            }
        }
    }

    // -------------------------------------------------------------------------------------------- command bar

    /// Every searchable entry: (tab, row, option); option -1 = the row itself (actions, sliders).
    fn entry(&self, t: usize, r: usize, o: i32, out: &mut Txt) {
        let rd = &TABS[t].1[r];
        out.clear().s(TABS[t].0).s(" ").s(rd.name);
        match rd.kind {
            K::Choice(tb) if o >= 0 => {
                out.s(": ").s(tb[o as usize].0);
            }
            K::Toggle if o >= 0 => {
                out.s(if o == 1 { ": on" } else { ": off" });
            }
            _ => {}
        }
    }

    fn search(&mut self) {
        self.menu.n_hits = 0;
        let mut words = [("", 0usize); 6];
        let mut nw = 0;
        let q = self.menu.q;
        for w in q.as_str().split(' ').filter(|w| !w.is_empty()) {
            if nw < 6 {
                words[nw] = (w, 0);
                nw += 1;
            }
        }
        let mut e = Txt::new();
        for t in 0..TABS.len() {
            for r in 0..TABS[t].1.len() {
                let n = match TABS[t].1[r].kind {
                    K::Choice(tb) => tb.len() as i32,
                    K::Toggle => 2,
                    _ => 0,
                };
                for o in if n == 0 { -1..0 } else { 0..n } {
                    self.entry(t, r, o, &mut e);
                    if words[..nw].iter().all(|w| contains(e.as_str(), w.0)) && self.menu.n_hits < 8 {
                        self.menu.hits[self.menu.n_hits] = (t as u8, r as u8, o as i8);
                        self.menu.n_hits += 1;
                    }
                }
            }
        }
        self.menu.csel = self.menu.csel.min(self.menu.n_hits.saturating_sub(1));
    }

    fn cmd_run(&mut self) {
        if self.menu.n_hits == 0 {
            return;
        }
        let (t, r, o) = self.menu.hits[self.menu.csel];
        let rd = &TABS[t as usize].1[r as usize];
        self.menu.tab = t as usize;
        self.menu.row = r as usize;
        let mut e = Txt::new();
        self.entry(t as usize, r as usize, o as i32, &mut e);
        match rd.kind {
            K::Choice(_) | K::Toggle => {
                if self.val(rd.id) != o as i32 {
                    self.set_val(rd.id, o as i32);
                    self.changed(rd.id);
                }
                self.toast(&e, None);
            }
            K::Action | K::SliderApply(..) => self.activate(rd.id),
            K::Slider(..) => {}
        }
        self.close_cmd();
    }

    fn cmd_keys(&mut self) {
        let n = unsafe { gd_ui_keys(self.menu.keys.as_mut_ptr(), 32) }.max(0) as usize;
        let mut dirty = false;
        for i in 0..n {
            let k = self.menu.keys[i];
            let (kind, v) = (k >> 24, k & 0xFF_FFFF);
            match kind {
                3 if (0x20..0x7F).contains(&v) => {
                    if v == b'/' as u32 && self.menu.q.is_empty() {
                        continue;
                    }
                    let c = [v as u8];
                    self.menu.q.s(unsafe { core::str::from_utf8_unchecked(&c) });
                    dirty = true;
                }
                1 => match v {
                    KEY_ENTER => {
                        self.cmd_run();
                        return;
                    }
                    KEY_ESCAPE => {
                        self.close_cmd();
                        return;
                    }
                    KEY_BACKSPACE => {
                        let s = self.menu.q;
                        let l = s.as_str().len();
                        self.menu.q.clear().s(&s.as_str()[..l.saturating_sub(1)]);
                        dirty = true;
                    }
                    KEY_UP => self.menu.csel = self.menu.csel.saturating_sub(1),
                    KEY_DOWN => self.menu.csel = (self.menu.csel + 1).min(self.menu.n_hits.saturating_sub(1)),
                    _ => {}
                },
                _ => {}
            }
        }
        if dirty {
            self.menu.csel = 0;
            self.search();
        }
    }

    // -------------------------------------------------------------------------------------------- drawing

    /// Part 10: one row's value text.
    fn value_text(&self, rd: &RowDef, t: &mut Txt) {
        let v = self.val(rd.id);
        t.clear();
        match rd.kind {
            K::Choice(tb) => {
                t.s("◀ ").s(tb[(v.max(0) as usize).min(tb.len() - 1)].0).s(" ▶");
            }
            K::Slider(_, _, _, u) | K::SliderApply(_, _, _, u) => {
                t.i(v).s(u);
            }
            K::Toggle => {
                t.s(if v != 0 { "ON" } else { "OFF" });
            }
            K::Action => {
                match rd.id {
                    R_REC if self.menu.recording >= 0 => t.s("● stop  A ›"),
                    R_FREE | R_PLAY => {
                        let l = self.rec_len[self.rec_slot()];
                        if l < 0 { t.s("empty") } else { t.fix(l as f32 / 60.0, 1).s(" s  A ›") }
                    }
                    _ => t.s("A ›"),
                };
            }
        }
    }

    pub fn draw_menu(&mut self, vw: f32) {
        let (w, h) = (820.0, 452.0);
        let x0 = (vw - w) * 0.5;
        let y0 = 96.0;
        let (ac, ink, mu, tx, dv, bd) = (self.t(T_ACCENT), self.t(T_ACCENT_INK), self.t(T_MUTED), self.t(T_TEXT), self.t(T_DIVIDER), self.t(T_BORDER));
        // the tab bar: L, the tabs, R
        let mut x = x0;
        self.key(x, y0 + 4.0, 22.0, "L", false);
        x += 30.0;
        for (i, (name, _)) in TABS.iter().enumerate() {
            let tw = self.w_tab[i];
            let on = i == self.menu.tab;
            if on {
                self.enc.slice(SLICE_FILL_R4, x, y0, tw, 26.0, ac);
            } else {
                self.panel(x, y0, tw, 26.0);
            }
            self.enc.text_ex(x + tw * 0.5, y0 + 6.0, ROLE_VALUE, if on { ink } else { tx }, ALIGN_CENTER, 0.0, name);
            x += tw + 4.0;
        }
        self.key(x + 4.0, y0 + 4.0, 22.0, "R", false);
        // whose dummy (online: the opener's opponent)
        let mut who = Txt::new();
        who.s("dummy P").i(self.dport() as i32 + 1);
        self.enc.text_ex(x0 + w, y0 + 8.0, ROLE_LABEL, mu, ALIGN_RIGHT, 0.0, who.as_str());
        let by = y0 + 34.0;
        if self.menu.controls {
            // the Controls page: the CONTROLS table, buttons left, action right
            self.panel(x0, by, w, h - 34.0);
            self.enc.text(x0 + 16.0, by + 12.0, ROLE_TITLE, tx, "Controls");
            for (i, (k, what)) in CONTROLS.iter().enumerate() {
                let ry = by + 44.0 + i as f32 * 20.0;
                self.enc.text(x0 + 16.0, ry, ROLE_VALUE, tx, k);
                self.enc.text(x0 + 230.0, ry, ROLE_BODY, mu, what);
            }
            self.enc.text_ex(x0 + w - 16.0, by + 14.0, ROLE_LABEL, mu, ALIGN_RIGHT, 0.0, "B close");
            return;
        }
        // rows
        let lw = 500.0;
        self.panel(x0, by, lw, h - 34.0);
        let rows = TABS[self.menu.tab].1;
        const VIS: usize = 12;
        if self.menu.row < self.menu.scroll {
            self.menu.scroll = self.menu.row;
        } else if self.menu.row >= self.menu.scroll + VIS {
            self.menu.scroll = self.menu.row + 1 - VIS;
        }
        let mut t = Txt::new();
        for (k, rd) in rows.iter().enumerate().skip(self.menu.scroll).take(VIS) {
            let ry = by + 8.0 + (k - self.menu.scroll) as f32 * 30.0;
            let focus = k == self.menu.row;
            if focus {
                self.enc.rect(x0 + 1.0, ry, lw - 2.0, 30.0, (ac & 0xFFFF_FF00) | 0x26);
                self.enc.rect(x0 + 1.0, ry, 4.0, 30.0, ac);
            } else if k > self.menu.scroll {
                self.enc.rect(x0 + 14.0, ry, lw - 28.0, 1.0, dv);
            }
            self.enc.text(x0 + 16.0, ry + 8.0, ROLE_VALUE, if focus { tx } else { mu }, rd.name);
            self.value_text(rd, &mut t);
            let vx = x0 + lw - 16.0;
            match rd.kind {
                K::Slider(lo, hi, _, _) | K::SliderApply(lo, hi, _, _) => {
                    // part 10 slider: 80x4 track + value
                    let v = self.val(rd.id);
                    let tx0 = vx - 140.0;
                    self.enc.rect(tx0, ry + 13.0, 80.0, 4.0, self.t(T_CELL));
                    self.enc.rect(tx0, ry + 13.0, 80.0 * (v - lo) as f32 / (hi - lo).max(1) as f32, 4.0, ac);
                    self.enc.text_ex(vx, ry + 8.0, R_NUM, tx, ALIGN_RIGHT, 0.0, t.as_str());
                }
                K::Toggle => {
                    let on = self.val(rd.id) != 0;
                    self.enc.text_ex(vx, ry + 8.0, R_NUM, if on { ac } else { mu }, ALIGN_RIGHT, 0.0, t.as_str());
                }
                _ => {
                    self.enc.text_ex(vx, ry + 8.0, R_NUM, tx, ALIGN_RIGHT, 0.0, t.as_str());
                }
            }
        }
        if rows.len() > VIS {
            let sh = (h - 50.0) * VIS as f32 / rows.len() as f32;
            let sy = by + 8.0 + (h - 50.0 - sh) * self.menu.scroll as f32 / (rows.len() - VIS) as f32;
            self.enc.rect(x0 + lw - 5.0, sy, 2.0, sh, bd);
        }
        // part 11: the help pane
        let hx = x0 + lw + 10.0;
        let hw = w - lw - 10.0;
        self.panel(hx, by, hw, h - 34.0);
        let rd = &rows[self.menu.row.min(rows.len() - 1)];
        self.enc.text(hx + 14.0, by + 14.0, ROLE_TITLE, tx, rd.name);
        for (i, line) in rd.help.split('|').enumerate() {
            self.enc.text(hx + 14.0, by + 42.0 + i as f32 * 18.0, ROLE_BODY, mu, line);
        }
        // the recording slots' state on the Record tab
        if self.menu.tab == 1 {
            for s in 0..8 {
                let cx = hx + 14.0 + (s % 4) as f32 * 64.0;
                let cy = by + 150.0 + (s / 4) as f32 * 52.0;
                let sel = s == self.rec_slot();
                let l = self.rec_len[s];
                let a = |c: u32| if l < 0 { (c & 0xFFFF_FF00) | ((c & 0xFF) * 140 / 255) } else { c };
                self.enc.slice(SLICE_LINE_R4, cx, cy, 58.0, 44.0, if sel { ac } else { a(bd) });
                t.clear().s("Slot ").i(s as i32 + 1);
                self.enc.text(cx + 6.0, cy + 6.0, ROLE_LABEL, if sel { ac } else { a(tx) }, t.as_str());
                t.clear();
                if self.menu.recording == s as i32 {
                    t.s("● rec");
                } else if l < 0 {
                    t.s("empty");
                } else {
                    t.fix(l as f32 / 60.0, 1).s("s ×").i(self.menu.weights[s]);
                }
                self.enc.text(cx + 6.0, cy + 24.0, R_NUM, a(mu), t.as_str());
            }
        }
        // part 7 hints at the bottom of the help pane
        let hy = by + h - 34.0 - 26.0;
        self.enc.text_ex(hx + 14.0, hy, ROLE_LABEL, mu, ALIGN_LEFT, hw - 28.0, "A apply · ◀▶ change · L R tab · X find · B close");
        if self.menu.cmd {
            self.draw_cmd(x0 + 60.0, y0 + 60.0, w - 120.0);
        }
    }

    /// Part 14: the command bar.
    fn draw_cmd(&mut self, x: f32, y: f32, w: f32) {
        let (ac, mu, tx, bd) = (self.t(T_ACCENT), self.t(T_MUTED), self.t(T_TEXT), self.t(T_BORDER));
        let n = self.menu.n_hits;
        let h = 40.0 + n.max(1) as f32 * 28.0 + 8.0;
        self.enc.slice(SLICE_FILL_R4, x, y, w, h, 0x080B11F4).slice(SLICE_LINE_R4, x, y, w, h, ac);
        let mut p = Txt::new();
        p.s("› ").s(self.menu.q.as_str()).s("▍");
        self.enc.text(x + 14.0, y + 12.0, R_NUM, tx, p.as_str());
        self.enc.rect(x + 1.0, y + 38.0, w - 2.0, 1.0, bd);
        if n == 0 {
            self.enc.text(x + 14.0, y + 48.0, ROLE_VALUE, mu, "no match");
        }
        let mut e = Txt::new();
        for i in 0..n {
            let (t, r, o) = self.menu.hits[i];
            self.entry(t as usize, r as usize, o as i32, &mut e);
            let ry = y + 44.0 + i as f32 * 28.0;
            let sel = i == self.menu.csel;
            if sel {
                self.enc.rect(x + 1.0, ry, w - 2.0, 28.0, (ac & 0xFFFF_FF00) | 0x30);
            }
            self.enc.text(x + 14.0, ry + 7.0, ROLE_VALUE, if sel { tx } else { mu }, e.as_str());
        }
        self.enc.text_ex(x + w - 14.0, y + 14.0, ROLE_LABEL, mu, ALIGN_RIGHT, 0.0, "↑↓ pick · Enter run · Esc close");
    }

    /// Part 13: the radial quick menu (200 px, labels at radius 78, 52 wide, centre knob 32).
    pub fn draw_radial(&mut self, cx: f32, cy: f32) {
        let (ac, ink, tx, bd) = (self.t(T_ACCENT), self.t(T_ACCENT_INK), self.t(T_TEXT), self.t(T_KEY_BORDER));
        self.enc.slice(SLICE_FILL_R14, cx - 110.0, cy - 110.0, 220.0, 220.0, 0x080B11C8).slice(SLICE_LINE_R14_3, cx - 110.0, cy - 110.0, 220.0, 220.0, bd);
        const DIRS: [(f32, f32); 8] = [(0.0, -1.0), (0.7071, -0.7071), (1.0, 0.0), (0.7071, 0.7071), (0.0, 1.0), (-0.7071, 0.7071), (-1.0, 0.0), (-0.7071, -0.7071)];
        for (i, (dx, dy)) in DIRS.iter().enumerate() {
            let (lx, ly) = (cx + dx * 78.0 - 30.0, cy + dy * 78.0 - 11.0);
            let sel = self.menu.rsel == i as i32;
            if sel {
                self.enc.slice(SLICE_FILL_R4, lx, ly, 60.0, 22.0, ac);
            } else {
                self.panel(lx, ly, 60.0, 22.0);
            }
            let label = if i == 3 && self.menu.recording >= 0 { "Stop rec" } else { RADIAL[i] };
            self.enc.text_ex(lx + 30.0, ly + 5.0, ROLE_LABEL, if sel { ink } else { tx }, ALIGN_CENTER, 0.0, label);
        }
        let (sx, sy) = self.stick_of(self.menu.rport);
        let kx = (sx as f32 / 80.0).clamp(-1.0, 1.0) * 20.0;
        let ky = -(sy as f32 / 80.0).clamp(-1.0, 1.0) * 20.0;
        if self.kit_knob != 0 {
            self.enc.kit(self.kit_knob, cx - 16.0 + kx, cy - 16.0 + ky, 1.0, ac);
        }
    }
}

/// atan2 without libm: (y, x) in the usual sense; here called with (stick x, stick y) for 0 = up, clockwise.
fn libm_atan2(y: f32, x: f32) -> f32 {
    use core::f32::consts::PI;
    if x == 0.0 && y == 0.0 {
        return 0.0;
    }
    let (ax, ay) = (if x < 0.0 { -x } else { x }, if y < 0.0 { -y } else { y });
    let a = if ax > ay { ay / ax } else { ax / ay };
    // a polynomial atan on [0, 1] (max error ~0.005 rad: plenty for 8 sectors)
    let s = a * a;
    let mut r = ((-0.0464964749 * s + 0.15931422) * s - 0.327622764) * s * a + a;
    if ay > ax {
        r = PI / 2.0 - r;
    }
    if x < 0.0 {
        r = PI - r;
    }
    if y < 0.0 {
        r = -r;
    }
    r
}
