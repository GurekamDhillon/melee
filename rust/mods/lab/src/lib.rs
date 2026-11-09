//! lab: the LAB as a gd-ui-v1 / gd-lab-v1 presentation mod (docs/rust-port/notes/lab-ui-spec.md, theme A
//! "Instrument"). Milestones: M1 kit parts (panel, chip, toast, readout list, button hint), M2 freeze / step / scrubber
//! / save-load on the D-pad / slot cards, and the M4 HUD widgets the presets need (frame meter, move and state
//! readouts, dummy and combo panels, input display): presets Clean, Frame data, Full.
//!
//! Controls (ports 1 and 2, shared capture; the Lab owns START, the host strips it from the game): the list is
//! `menu::CONTROLS`, which the menu's Controls page (Session tab) draws. While frozen every other button and the
//! sticks are the game's: what is held when a step runs is that frame's input (hold down + B, D-right: the shine comes
//! out), and no lab control reads them (only an open menu does).
//! Online (LabNetplay) every sim action is a lab command both peers apply; the other player's freeze, rewind, step,
//! save and load show here as a toast ("P2 rewound") from `gd_lab_applied`.
//!
//! Performance (spec 8): one `gd_ui_submit` per present from a fixed buffer, no allocation anywhere (no_std, no
//! alloc), readout text re-formatted only when its value changes, frame-meter and fighter queries skipped while the
//! match frame has not moved, frame data queried only when a move ends, slot frames only after a save / load.
#![no_std]
#![allow(static_mut_refs)]

mod menu;
mod names;
mod txt;

use gd_ui_enc::*;
use menu::*;
use txt::Txt;

// the lookahead (gd_lab_lookahead_*; melee_modhost::lab LOOK_*)
pub const LOOK_AUTO: u32 = 1;
pub const LOOK_LIVE: u32 = 2;
const LOOK_HEAD: usize = 16;
const LOOK_RUNS: usize = 9;
const LOOK_RUN_I32: usize = 4;
const LOOK_I32: usize = LOOK_HEAD + LOOK_RUNS * LOOK_RUN_I32;
/// gd_lab_drill_status (melee_modhost::lab DRILL_I32): see the field list there.
const DRILL_I32: usize = 40;
const DS_ACTIVE: usize = 0;
const DS_PHASE: usize = 2;
const DS_N: usize = 3;
const DS_OK: usize = 4;
const DS_SEQ: usize = 5;
const DS_VERDICT: usize = 6;
const DS_LATE: usize = 7;
const DS_OPTION: usize = 8;
const DS_NOPT: usize = 9;
const DS_STREAK: usize = 11;
const DS_BEST: usize = 12;
const DS_LATE_SUM: usize = 13;
const DS_LATE_N: usize = 14;
const DS_ERR: usize = 15;
const DS_W: usize = 16;
const DS_TRIES: usize = 24;
const DS_SUCC: usize = 32;

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}

#[link(wasm_import_module = "gd_lab")]
extern "C" {
    fn gd_lab_save(slot: u32) -> i32;
    fn gd_lab_load(slot: u32) -> i32;
    fn gd_lab_slot_frame(slot: u32) -> i32;
    fn gd_lab_ring(frames: u32) -> i32;
    fn gd_lab_pause(on: i32) -> i32;
    fn gd_lab_step(n: i32) -> i32;
    fn gd_lab_rewind(n: u32) -> i32;
    fn gd_lab_go_live() -> i32;
    fn gd_lab_overlay(mask: u32, bits: u32) -> i32;
    fn gd_lab_fighter(port: u32, out: *mut u8, len: u32) -> i32;
    fn gd_lab_framedata(ch: i32, action: i32, out: *mut u8, len: u32) -> i32;
    fn gd_lab_frame_adv(a: u32, b: u32) -> i32;
    fn gd_lab_events(out: *mut u8, max: u32) -> i32;
    fn gd_lab_events_range(from: i32, to: i32, out: *mut u8, max: u32) -> i32;
    fn gd_lab_meter(port: u32, out: *mut u8, n: u32) -> i32;
    fn gd_lab_status(out: *mut i32) -> i32;
    fn gd_lab_applied(out: *mut u8, max: u32) -> i32;
    fn gd_lab_timed() -> i32;
    fn gd_lab_lookahead_cfg(flags: u32, budget_us: u32) -> i32;
    fn gd_lab_lookahead_result(out: *mut u8, len: u32) -> i32;
    fn gd_lab_drill_status(out: *mut u8, len: u32) -> i32;
    fn gd_lab_drill_text(kind: u32, i: u32, out: *mut u8, len: u32) -> i32;
    fn gd_lab_drill_trend(out: *mut u8, max: u32) -> i32;
}

const FEATURE_LAB: u32 = 1 << 10;
const FEATURE_LAB_STATE: u32 = 1 << 11;
const FEATURE_LAB_INPUT: u32 = 1 << 12;
const FEATURE_LAB_CPU: u32 = 1 << 13;
const FEATURE_LAB_OVERLAY: u32 = 1 << 14;

// pad buttons
const DL: u16 = 0x1;
const DR: u16 = 0x2;
const DD: u16 = 0x4;
const DU: u16 = 0x8;
const BZ: u16 = 0x10;
const BR: u16 = 0x20;
const BL: u16 = 0x40;
const BA: u16 = 0x100;
const BB: u16 = 0x200;
const BX: u16 = 0x400;
const BY: u16 = 0x800;
const START: u16 = 0x1000;

// theme tokens (gd-ui-lab-v1.h GD_UI_TOK_*)
const T_PANEL_HUD: usize = 1;
const T_BORDER: usize = 2;
const T_TEXT: usize = 3;
const T_MUTED: usize = 4;
const T_ACCENT: usize = 5;
const T_ACCENT_INK: usize = 6;
const T_DIVIDER: usize = 7;
const T_KEY_BORDER: usize = 8;
const T_KEY_TEXT: usize = 9;
const T_P1: usize = 10;
const T_CELL: usize = 12;
const T_CELL_ACTIVE: usize = 14;

// lab events (B5) and command ops (GAP-13)
const EV_LCANCEL: u8 = 1;
const EV_TECH: u8 = 2;
const EV_OOS: u8 = 3;
const EV_HIT: u8 = 5;
const EV_SHIELD: u8 = 6;
const EV_POWERSHIELD: u8 = 7;
const EV_WAVEDASH: u8 = 8;
const EV_LEDGEDASH: u8 = 9;
const EV_SDI: u8 = 10;
const OP_SAVE: u8 = 1;
const OP_LOAD: u8 = 2;
const OP_REWIND: u8 = 4;
const OP_PAUSE: u8 = 5;
const OP_STEP: u8 = 6;
const OP_GO_LIVE: u8 = 7;
const OP_DRILL: u8 = 20;
const CELL_STUN: u8 = 4;
const GD_ESTATE: i32 = -4;

// gd-ui-v1.1 text styles: theme A's number font is JetBrains Mono (spec 7.1)
const R_NUM: i32 = ROLE_VALUE | TEXT_MONO;
const R_TOAST: i32 = ROLE_TOAST | TEXT_MONO;
const R_LAB: i32 = ROLE_TITLE | TEXT_MONO;
const R_ADV: i32 = ROLE_VALUE | TEXT_MONO | TEXT_BOLD | text_size(16);

const CLEAN: u8 = 0;
const FRAMEDATA: u8 = 1;
const FULL: u8 = 2;
const PRESET_NAMES: [&str; 3] = ["Clean", "Frame data", "Full"];

const CELLS: usize = 50;
const RING: i32 = 600;
const MARKS: usize = 64;

#[derive(Clone, Copy)]
struct Toast {
    main: Txt,
    sub: Txt,
    life: u32,
    w: f32,
    sub_w: f32,
    sdi: i32,
}
impl Toast {
    const fn new() -> Toast {
        Toast { main: Txt::new(), sub: Txt::new(), life: 0, w: 0.0, sub_w: 0.0, sdi: 0 }
    }
}

/// A cached name / value row: the value is re-formatted only when its key changes.
#[derive(Clone, Copy)]
struct Row {
    key: u64,
    v: Txt,
}
impl Row {
    const fn new() -> Row {
        Row { key: u64::MAX, v: Txt::new() }
    }
    fn set(&mut self, key: u64, f: impl FnOnce(&mut Txt)) {
        if self.key != key {
            self.key = key;
            self.v.clear();
            f(&mut self.v);
        }
    }
}

/// The last finished move of P1 (from the observed frame data).
#[derive(Clone, Copy)]
struct Move {
    action: i32,
    n: i32,
    first: i32,
    last: i32,
    iasa: i32,
    landing: i32,
    lcancel: bool,
    damage: i32,
    adv: i32,
    on_shield: bool,
    have_adv: bool,
}

struct Lab {
    setup: bool,
    tok: [u32; 26],
    cap: u32,
    kit_dot: u32,
    kit_ring: u32,
    kit_diamond: u32,
    kit_hatch: u32,
    kit_knob: u32,
    preset: u8,
    slot: u32,
    timed: bool,
    status: [i32; 10],
    last_frame: i32,
    head: i32,
    online: bool,
    paused: bool,
    fighter: [[u8; 128]; 2],
    have: [bool; 2],
    meter: [[u8; CELLS]; 2],
    adv: i32,
    adv_shield: bool,
    look: [u8; LOOK_I32 * 4],
    toasts: [Toast; 3],
    // P1 move tracking
    p1_action: i32,
    p1_char: i32,
    pending_fd: i32,
    pending_tries: u8,
    dmg_action: i32,
    dmg_sum: i32,
    mv: Move,
    mv_title: Row,
    mv_rows: [Row; 6],
    st_rows: [Row; 5],
    fd_cache: (i32, i32, i32), // (char, action, n) of the action the state readout shows
    // combo
    combo_hits: i32,
    combo_dmg: i32,
    combo_chain: [i32; 3],
    combo_live: bool,
    combo_quiet: i32,
    combo_row: Row,
    combo_dmg_row: Row,
    chain_row: Row,
    // input display
    in_hist: [(u16, u8, u16); 3],
    in_rows: [Row; 3],
    stick: (i8, i8),
    held_p1: u16,
    held_p: [u16; 2],
    edge_p: [u16; 2],
    stick_p: [(i8, i8); 2],
    /// START went down on this port (-1 none): frames held, and whether the game was frozen before
    start_hold: (i32, u32, bool),
    // the menu (M3), the radial and the command bar (M9)
    menu: Menu,
    /// dummy settings per port (menu.rs D_*), kept in step with the applied commands of both peers
    dv: [[i32; D_N]; 4],
    vals: [i32; N_IDS],
    rec_len: [i32; 8],
    w_tab: [f32; 6],
    // drills: the status as published, the last running drill's copy (the summary page), its labels and trend
    drill_st: [i32; DRILL_I32],
    drill_last: [i32; DRILL_I32],
    drill_seen: i32,
    drill_name: Txt,
    drill_opts: [Txt; 8],
    drill_trend: [(i32, i32); 8],
    drill_n_trend: usize,
    drill_err_seen: i32,
    // freeze controls
    hold: (u16, u32),
    dummy_toast: i32,
    /// D-down is held and nothing else was pressed with it: save on release (D-down + START opens the settings overlay)
    dd_armed: bool,
    after_modal: bool,
    prev_held: u16,
    // slots and marks
    slots: [i32; 16],
    slots_dirty: bool,
    marks: [(i32, u8); MARKS],
    n_marks: usize,
    marks_key: (i32, i32),
    // texts and widths measured once
    chip: Txt,
    chip_key: u32,
    chip_w: f32,
    lab_w: f32,
    adv_row: Row,
    sum_row: Row,
    scrub_rows: [Row; 3],
    slot_rows: [Row; 16],
    w_hint: [f32; 16],
    enc: Enc<8192>,
    pad: [u8; 16],
    fbuf: [u8; 512],
    evbuf: [u8; 16 * 16],
    abuf: [u8; 20 * 16],
}

static mut L: Lab = Lab {
    setup: false,
    tok: [0; 26],
    cap: 0,
    kit_dot: 0,
    kit_ring: 0,
    kit_diamond: 0,
    kit_hatch: 0,
    kit_knob: 0,
    preset: FRAMEDATA,
    slot: 0,
    timed: false,
    status: [0; 10],
    last_frame: -1,
    head: 0,
    online: false,
    paused: false,
    fighter: [[0; 128]; 2],
    have: [false; 2],
    meter: [[0; CELLS]; 2],
    adv: 0,
    adv_shield: false,
    look: [0; LOOK_I32 * 4],
    toasts: [Toast::new(); 3],
    p1_action: -1,
    p1_char: -1,
    pending_fd: -1,
    pending_tries: 0,
    dmg_action: -1,
    dmg_sum: 0,
    mv: Move { action: -1, n: 0, first: -1, last: -1, iasa: -1, landing: -1, lcancel: false, damage: 0, adv: 0, on_shield: false, have_adv: false },
    mv_title: Row::new(),
    mv_rows: [Row::new(); 6],
    st_rows: [Row::new(); 5],
    fd_cache: (-1, -1, 0),
    combo_hits: 0,
    combo_dmg: 0,
    combo_chain: [-1; 3],
    combo_live: false,
    combo_quiet: 0,
    combo_row: Row::new(),
    combo_dmg_row: Row::new(),
    chain_row: Row::new(),
    in_hist: [(0, 5, 0); 3],
    in_rows: [Row::new(); 3],
    stick: (0, 0),
    held_p1: 0,
    held_p: [0; 2],
    edge_p: [0; 2],
    stick_p: [(0, 0); 2],
    start_hold: (-1, 0, false),
    menu: Menu::new(),
    dv: [D_DEFAULT; 4],
    vals: [0; N_IDS],
    rec_len: [-1; 8],
    w_tab: [0.0; 6],
    drill_st: [0; DRILL_I32],
    drill_last: [0; DRILL_I32],
    drill_seen: -1,
    drill_name: Txt::new(),
    drill_opts: [Txt::new(); 8],
    drill_trend: [(0, 0); 8],
    drill_n_trend: 0,
    drill_err_seen: 0,
    hold: (0, 0),
    dummy_toast: -1,
    dd_armed: false,
    after_modal: false,
    prev_held: 0,
    slots: [-1; 16],
    slots_dirty: true,
    marks: [(0, 0); MARKS],
    n_marks: 0,
    marks_key: (-1, -1),
    chip: Txt::new(),
    chip_key: u32::MAX,
    chip_w: 0.0,
    lab_w: 0.0,
    adv_row: Row::new(),
    sum_row: Row::new(),
    scrub_rows: [Row::new(); 3],
    slot_rows: [Row::new(); 16],
    w_hint: [0.0; 16],
    enc: Enc::new(),
    pad: [0; 16],
    fbuf: [0; 512],
    evbuf: [0; 256],
    abuf: [0; 320],
};

fn rd_i32(b: &[u8], o: usize) -> i32 {
    i32::from_le_bytes([b[o], b[o + 1], b[o + 2], b[o + 3]])
}
fn rd_f32(b: &[u8], o: usize) -> f32 {
    f32::from_bits(rd_i32(b, o) as u32)
}

// gd_lab_fighter_v1 offsets
const F_CHAR: usize = 8;
const F_ACTION: usize = 12;
const F_HITLAG: usize = 24;
const F_HITSTUN: usize = 28;
const F_POS_X: usize = 56;
const F_POS_Y: usize = 60;
const F_VX: usize = 64;
const F_VY: usize = 68;
const F_ACTION_FRAME: usize = 120;

/// Hint key caps and labels: (key, label); widths measured once at init.
const HINTS_PLAY: [(&str, &str); 4] = [("START", "freeze"), ("D↓", "save"), ("D↑", "load"), ("Z+D↑", "HUD preset")];
const HINTS_FROZEN: [(&str, &str); 4] = [("◀ ▶", "step"), ("Z+A", "take over"), ("Z hold", "quick"), ("START hold", "menu · Controls")];

fn measure(s: &str, role: i32) -> f32 {
    gd_ui_enc::measure(s, role)
}

fn action_name(a: i32) -> &'static str {
    names::action(a).unwrap_or("Special")
}

/// The combo chain's short name of an action.
fn short(a: i32) -> &'static str {
    match action_name(a) {
        "AttackAirN" => "nair",
        "AttackAirF" => "fair",
        "AttackAirB" => "bair",
        "AttackAirHi" => "uair",
        "AttackAirLw" => "dair",
        "Attack11" | "Attack12" | "Attack13" | "Attack100Loop" => "jab",
        "AttackDash" => "dash attack",
        "AttackS3Hi" | "AttackS3HiS" | "AttackS3S" | "AttackS3LwS" | "AttackS3Lw" => "ftilt",
        "AttackHi3" => "utilt",
        "AttackLw3" => "dtilt",
        "AttackS4Hi" | "AttackS4HiS" | "AttackS4S" | "AttackS4LwS" | "AttackS4Lw" => "fsmash",
        "AttackHi4" => "usmash",
        "AttackLw4" => "dsmash",
        "ThrowF" => "fthrow",
        "ThrowB" => "bthrow",
        "ThrowHi" => "uthrow",
        "ThrowLw" => "dthrow",
        "Special" => "special",
        n => n,
    }
}

impl Lab {
    fn t(&self, i: usize) -> u32 {
        self.tok[i]
    }

    fn toast(&mut self, main: &Txt, sub: Option<&Txt>) {
        self.toasts.rotate_right(1);
        let t = &mut self.toasts[0];
        t.main = *main;
        t.sub.clear();
        t.sdi = 0;
        if let Some(s) = sub {
            t.sub = *s;
        }
        t.life = 150;
        gd_ui_enc::log(t.main.as_str());
        t.w = measure(t.main.as_str(), R_TOAST) + 24.0;
        t.sub_w = if t.sub.is_empty() { 0.0 } else { measure(t.sub.as_str(), R_NUM) + 20.0 };
    }
    fn toast_s(&mut self, s: &str) {
        let mut m = Txt::new();
        m.s(s);
        self.toast(&m, None);
    }

    fn init(&mut self) {
        unsafe {
            gd_ui_theme(0);
            for i in 0..26 {
                self.tok[i] = gd_ui_token(i as i32);
            }
            let k = |s: &str| gd_ui_kit_id(s.as_ptr(), s.len() as u32);
            self.kit_dot = k("ui.dot");
            self.kit_ring = k("ui.ring");
            self.kit_diamond = k("ui.diamond");
            self.kit_hatch = k("ui.hatch");
            self.kit_knob = k("ui.knob");
            // ports 1 and 2 observed (the game keeps its input)
            self.cap = gd_ui_capture(0x3, CAP_SHARED);
        }
        self.lab_w = measure("LAB", R_LAB);
        for (i, (name, _)) in TABS.iter().enumerate() {
            self.w_tab[i] = measure(name, ROLE_VALUE) + 24.0;
        }
        self.menu_init();
        for (i, (k, l)) in HINTS_PLAY.iter().chain(HINTS_FROZEN.iter()).enumerate() {
            self.w_hint[i * 2] = measure(k, ROLE_LABEL) + 12.0;
            self.w_hint[i * 2 + 1] = measure(l, ROLE_VALUE);
        }
    }

    fn match_setup(&mut self) {
        unsafe {
            gd_lab_ring(RING as u32);
            self.timed = gd_lab_timed() > 0;
        }
        self.apply_preset();
        self.slots_dirty = true;
        self.head = self.status[1];
        self.toast_s("LAB ready");
    }

    fn apply_preset(&mut self) {
        self.preset_widgets();
        self.chip_key = u32::MAX;
    }

    fn stick_of(&self, port: u32) -> (i8, i8) {
        self.stick_p[port as usize & 1]
    }

    // -------------------------------------------------------------------------------------------- input

    fn controls(&mut self) {
        let mut held = 0u16;
        let mut edge = 0u16;
        let mut modal = false;
        for port in 0..2 {
            let r = unsafe { gd_ui_pad(port, self.pad.as_mut_ptr()) };
            modal |= r == GD_ESTATE;
            let (mut h, mut e, mut st) = (0, 0, (0, 0));
            if r == 0 {
                h = u16::from_le_bytes([self.pad[0], self.pad[1]]);
                e = u16::from_le_bytes([self.pad[12], self.pad[13]]);
                st = (self.pad[2] as i8, self.pad[3] as i8);
                held |= h;
                edge |= e;
                if port == 0 {
                    self.held_p1 = h;
                    self.stick = st;
                }
            }
            self.held_p[port as usize] = h;
            self.edge_p[port as usize] = e;
            self.stick_p[port as usize] = st;
        }
        // a host overlay (the settings overlay) holds the pads: no lab controls, no pending save, and nothing counts
        // again until every button is up (the press that closed it is not a lab action)
        if modal {
            self.dd_armed = false;
            self.after_modal = true;
            return;
        }
        if self.after_modal {
            if held != 0 {
                self.prev_held = held;
                return;
            }
            self.after_modal = false;
        }
        // the full menu holds the opener's pad (the game is frozen meanwhile)
        if self.menu.open {
            let p = self.menu.port as usize & 1;
            self.menu_input(self.held_p[p], self.edge_p[p], self.stick_p[p]);
            self.prev_held = held;
            self.dd_armed = false;
            return;
        }
        // START: freeze on the press, the menu when it is held 30 frames, resume on a short tap while frozen
        if self.start_hold.0 >= 0 {
            let p = self.start_hold.0 as usize;
            if self.held_p[p] & START != 0 {
                self.start_hold.1 += 1;
                if self.start_hold.1 >= 30 {
                    self.start_hold.0 = -1;
                    self.open_menu(p as u32, !self.start_hold.2);
                }
                self.prev_held = held;
                return;
            }
            if self.start_hold.2 {
                unsafe { gd_lab_pause(0) };
            }
            self.start_hold.0 = -1;
        }
        // the radial (Z hold); D-pad presses with Z keep their Z + D-pad meaning
        let zp = if self.held_p[0] & BZ != 0 { 0 } else { 1 };
        if self.menu.radial {
            let p = self.menu.rport as usize;
            if self.edge_p[p] & (DL | DR | DU) != 0 {
                self.menu.radial = false;
                self.menu.z_combo = true;
            } else {
                self.radial_input(p as u32, self.held_p[p], self.edge_p[p], self.stick_p[p]);
                self.prev_held = held;
                return;
            }
        }
        if self.held_p[zp] & BZ != 0 {
            // only a plain Z hold opens it: with the stick still and no other button (a grab, a Z-cancel or an
            // aerial is never a menu)
            let (sx, sy) = self.stick_p[zp];
            let moved = (sx as i32).abs() > 30 || (sy as i32).abs() > 30;
            if self.edge_p[zp] & (DL | DR | DU | DD) != 0 || self.held_p[zp] & !(BZ | 0x80) != 0 || (moved && self.menu.z_frames < 12) {
                self.menu.z_combo = true;
            }
            self.menu.z_frames += 1;
            // frozen only (spec open question 3): during play Z is grab, and a held Z is ordinary play
            if self.menu.z_frames == 12 && !self.menu.z_combo && self.paused {
                self.menu.radial = true;
                self.menu.rport = zp as u32;
                self.menu.rsel = -1;
            }
        } else {
            self.menu.z_frames = 0;
            self.menu.z_combo = false;
        }
        let z = held & BZ != 0;
        // hold-to-repeat for frozen D-left / D-right: 15 frames, then every 4
        let dir = held & (DL | DR);
        if dir != 0 && dir == self.hold.0 {
            self.hold.1 += 1;
        } else {
            self.hold = (dir, 0);
        }
        let repeat = self.hold.1 >= 15 && (self.hold.1 - 15) % 4 == 0;
        unsafe {
            let released = self.prev_held & !held;
            self.prev_held = held;
            if edge & DD != 0 {
                self.dd_armed = true;
            } else if held & DD != 0 && edge != 0 {
                // a chord with D-down (the settings overlay's D-down + START, Z + D-down): not a save
                self.dd_armed = false;
            }
            if edge & START != 0 {
                if held & DD == 0 {
                    let p = if self.edge_p[0] & START != 0 { 0 } else { 1 };
                    if !self.paused {
                        gd_lab_pause(1);
                    }
                    self.start_hold = (p, 0, self.paused);
                }
                return;
            }
            if z {
                if self.paused && edge & BA != 0 {
                    gd_lab_go_live();
                    gd_lab_pause(0);
                    self.toast_s("Take over");
                } else if self.paused && edge & BL != 0 {
                    gd_lab_rewind(10);
                } else if self.paused && edge & BR != 0 {
                    gd_lab_step(10);
                } else if edge & DL != 0 {
                    self.slot = (self.slot + 15) % 16;
                    self.slot_toast();
                } else if edge & DR != 0 {
                    self.slot = (self.slot + 1) % 16;
                    self.slot_toast();
                } else if edge & DU != 0 {
                    self.preset = (self.preset + 1) % 3;
                    self.apply_preset();
                    let mut m = Txt::new();
                    m.s("HUD · ").s(PRESET_NAMES[self.preset as usize]);
                    self.toast(&m, None);
                }
                return;
            }
            if released & DD != 0 && core::mem::take(&mut self.dd_armed) {
                gd_lab_save(self.slot);
                let mut m = Txt::new();
                m.s("Saved slot ").i(self.slot as i32 + 1);
                self.toast(&m, None);
            } else if edge & DU != 0 {
                let mut m = Txt::new();
                if self.slots[self.slot as usize] < 0 {
                    m.s("Slot ").i(self.slot as i32 + 1).s(" is empty");
                } else {
                    gd_lab_load(self.slot);
                    m.s("Loaded slot ").i(self.slot as i32 + 1);
                }
                self.toast(&m, None);
            }
            // frozen: only the D-pad steps; A / B / X / Y / L / R and the sticks stay the game's (held into a step)
            if self.paused {
                if edge & DL != 0 || (repeat && dir == DL) {
                    gd_lab_rewind(1);
                } else if edge & DR != 0 || (repeat && dir == DR) {
                    gd_lab_step(1);
                }
            }
        }
    }

    fn slot_toast(&mut self) {
        let mut m = Txt::new();
        m.s("Slot ").i(self.slot as i32 + 1);
        if self.slots[self.slot as usize] < 0 {
            m.s(" · empty");
        }
        self.toast(&m, None);
    }

    /// The commands applied last frame: forks reset the scrubber head, the other player's actions become toasts.
    fn applied(&mut self) {
        let n = unsafe { gd_lab_applied(self.abuf.as_mut_ptr(), 16) };
        let me = self.status[8];
        for k in 0..n.max(0) as usize {
            let mut r = [0u8; 20];
            r.copy_from_slice(&self.abuf[k * 20..k * 20 + 20]);
            let (side, op, a) = (r[0] as i32, r[4], u16::from_le_bytes([r[6], r[7]]) as i32);
            // the other peer's dummy changes (ours are in the menu already, and may be ahead of what applied)
            if (op == OP_CPU || op == OP_REACTION || op == OP_DUMMY_OPT) && side != me {
                let b = u32::from_le_bytes([r[8], r[9], r[10], r[11]]);
                let c = u32::from_le_bytes([r[12], r[13], r[14], r[15]]);
                self.applied_dummy(op, r[5] as usize, a as u32, b, c);
            }
            if op == OP_SAVE || op == OP_LOAD {
                self.slots_dirty = true;
            }
            if op == OP_LOAD || op == OP_GO_LIVE {
                self.head = -1; // set from this frame below
            }
            if side == me {
                continue;
            }
            let mut m = Txt::new();
            m.s("P").i(side + 1);
            match op {
                OP_SAVE => m.s(" saved slot ").i(a + 1),
                OP_LOAD => m.s(" loaded slot ").i(a + 1),
                OP_REWIND => m.s(" rewound"),
                OP_PAUSE => m.s(if a != 0 { " froze" } else { " resumed" }),
                OP_STEP => m.s(" stepped"),
                OP_GO_LIVE => m.s(" took over"),
                OP_DRILL => m.s(if a == 0xFFFF { " stopped the drill" } else { " started a drill" }),
                OP_REACTION | OP_CPU => {
                    if self.dummy_toast == self.status[1] {
                        continue;
                    }
                    self.dummy_toast = self.status[1];
                    m.s(" set dummy P").i(r[5] as i32 + 1)
                }
                _ => continue,
            };
            self.toast(&m, None);
        }
    }

    // -------------------------------------------------------------------------------------------- reads

    fn events(&mut self) {
        let n = unsafe { gd_lab_events(self.evbuf.as_mut_ptr(), 16) };
        for k in 0..n.max(0) as usize {
            let e = &self.evbuf[k * 16..k * 16 + 16];
            let (kind, port, a, b) = (e[0], e[1] as i32, rd_i32(e, 8), rd_i32(e, 12));
            let mut m = Txt::new();
            if port == 1 && kind != EV_HIT && kind != EV_SHIELD {
                m.s("P2 ");
            }
            match kind {
                EV_LCANCEL => {
                    if a != 0 {
                        m.s("L-CANCEL OK ").i(b).s("/7f");
                    } else {
                        m.s("L-CANCEL MISS");
                    }
                    if port == 0 {
                        self.mv.lcancel = a != 0;
                    }
                }
                EV_TECH => {
                    m.s(match a { 1 => "TECH IN PLACE", 2 => "TECH ROLL IN", 3 => "TECH ROLL AWAY", _ => "MISSED TECH" });
                }
                EV_OOS => {
                    m.s("OUT OF SHIELD f").i(b);
                }
                EV_POWERSHIELD => {
                    m.s("POWERSHIELD f").i(a);
                }
                EV_WAVEDASH => {
                    m.s("WAVEDASH ").i(a).s("° f").i(b);
                }
                EV_LEDGEDASH => {
                    m.s("LEDGEDASH GALINT ").i(a);
                }
                EV_SDI => {
                    // coalesce into the newest SDI toast
                    if let Some(t) = self.toasts.iter_mut().find(|t| t.sdi > 0 && t.life > 0) {
                        t.sdi += 1;
                        t.main = m;
                        t.main.s("SDI x").i(t.sdi);
                        t.life = 150;
                        continue;
                    }
                    m.s("SDI x1");
                    self.toast(&m, None);
                    self.toasts[0].sdi = 1;
                    continue;
                }
                EV_HIT => {
                    if a == 0 && port == 1 {
                        // damage by the action that dealt it, so the move readout finds it whatever ends first
                        if self.dmg_action != self.p1_action {
                            self.dmg_action = self.p1_action;
                            self.dmg_sum = 0;
                        }
                        self.dmg_sum += b;
                        if self.mv.action == self.p1_action && self.pending_fd != self.p1_action {
                            self.mv.damage = self.dmg_sum;
                        }
                        if !self.combo_live {
                            self.combo_hits = 0;
                            self.combo_dmg = 0;
                            self.combo_chain = [-1; 3];
                        }
                        self.combo_live = true;
                        self.combo_quiet = 0;
                        self.combo_hits += 1;
                        self.combo_dmg += b;
                        if self.combo_chain[2] != self.p1_action || self.combo_hits == 1 {
                            self.combo_chain.rotate_left(1);
                            self.combo_chain[2] = self.p1_action;
                        }
                    }
                    self.adv_shield = false;
                    continue;
                }
                EV_SHIELD => {
                    self.adv_shield = true;
                    if self.preset == FULL && self.toasts[0].life > 0 && self.toasts[0].sub.is_empty() {
                        self.toasts[0].sub.s("shield hit");
                        self.toasts[0].sub_w = measure("shield hit", ROLE_VALUE) + 20.0;
                    }
                    continue;
                }
                _ => continue,
            }
            self.toast(&m, None);
        }
    }

    fn read_fighters(&mut self) {
        for p in 0..2 {
            self.have[p] = unsafe { gd_lab_fighter(p as u32, self.fighter[p].as_mut_ptr(), 128) } >= 128;
            let got = unsafe { gd_lab_meter(p as u32, self.meter[p].as_mut_ptr(), CELLS as u32) };
            let got = got.max(0) as usize;
            if got < CELLS {
                self.meter[p].copy_within(0..got, CELLS - got);
                for c in self.meter[p][..CELLS - got].iter_mut() {
                    *c = 0;
                }
            }
        }
        self.adv = unsafe { gd_lab_frame_adv(0, 1) };
        // P1's move: when an action ends, its observed timeline is published; a move is one that had a hitbox
        if self.have[0] {
            let f = &self.fighter[0];
            let (ch, act) = (rd_i32(f, F_CHAR), rd_i32(f, F_ACTION));
            if act != self.p1_action {
                if self.p1_action >= 0 {
                    self.pending_fd = self.p1_action;
                    self.pending_tries = 3;
                }
                self.p1_action = act;
                self.p1_char = ch;
            }
        }
        if self.pending_fd >= 0 && self.pending_tries > 0 {
            self.pending_tries -= 1;
            let a = self.pending_fd;
            let n = unsafe { gd_lab_framedata(self.p1_char, a, self.fbuf.as_mut_ptr(), 16) };
            if n >= 16 {
                let fb = &self.fbuf;
                let (cells, first, last, iasa) = (rd_i32(fb, 0), rd_i32(fb, 4), rd_i32(fb, 8), rd_i32(fb, 12));
                let name = action_name(a);
                if name.starts_with("LandingAir") {
                    self.mv.landing = cells;
                } else if first >= 0 {
                    self.mv = Move { action: a, n: cells, first, last, iasa, landing: -1, lcancel: false, damage: if self.dmg_action == a { self.dmg_sum } else { 0 }, adv: 0, on_shield: false, have_adv: false };
                }
                self.pending_fd = -1;
            }
            if self.pending_tries == 0 {
                self.pending_fd = -1;
            }
        }
        // the move's advantage once both can act (frame_adv is "after the last exchange")
        if self.mv.action >= 0 && self.have[1] {
            self.mv.adv = self.adv;
            self.mv.on_shield = self.adv_shield;
            self.mv.have_adv = true;
        }
        // combo ends when P2 leaves stun for a while
        if self.have[1] {
            let f = &self.fighter[1];
            let stunned = rd_f32(f, F_HITSTUN) > 0.0 || rd_f32(f, F_HITLAG) > 0.0 || self.meter[1][CELLS - 1] == CELL_STUN;
            if stunned {
                self.combo_quiet = 0;
            } else {
                self.combo_quiet += 1;
                if self.combo_quiet > 20 {
                    self.combo_live = false;
                }
            }
        }
        // input history (P1): (buttons, direction 1-9 as on a numpad, frames)
        let btn = self.held_p1 & (BA | BB | BX | BY | BL | BR | BZ);
        let (sx, sy) = self.stick;
        let dx = if sx > 40 { 1 } else if sx < -40 { -1 } else { 0 };
        let dy = if sy > 40 { 1 } else if sy < -40 { -1 } else { 0 };
        let dir = (5 + dx + dy * 3) as u8;
        if self.in_hist[0].0 == btn && self.in_hist[0].1 == dir {
            self.in_hist[0].2 = self.in_hist[0].2.saturating_add(1);
        } else {
            self.in_hist.rotate_right(1);
            self.in_hist[0] = (btn, dir, 1);
        }
    }

    fn read_slots(&mut self) {
        for s in 0..16 {
            self.slots[s] = unsafe { gd_lab_slot_frame(s as u32) };
        }
        self.slots_dirty = false;
    }

    // -------------------------------------------------------------------------------------------- frame

    fn frame(&mut self) {
        if unsafe { gd_lab_status(self.status.as_mut_ptr()) } != 0 || self.status[0] == 0 {
            // no lab match: nothing to draw
            self.setup = false;
            self.last_frame = -1;
            return;
        }
        self.online = unsafe { gd_ui_session() } == 4;
        if !self.setup {
            self.setup = true;
            self.match_setup();
        }
        self.paused = self.status[2] != 0;
        self.applied();
        self.controls();
        let frame = self.status[1];
        if self.head < 0 || frame > self.head {
            self.head = frame;
        }
        if frame != self.last_frame {
            self.last_frame = frame;
            self.events();
            self.read_fighters();
        }
        if self.slots_dirty {
            self.read_slots();
        }
        self.drill_frame();
        self.draw();
    }

    fn draw(&mut self) {
        let mut b = [0f32; 6];
        unsafe { gd_ui_band_rect(b.as_mut_ptr()) };
        // the presets are laid out on 1280x720: the scale fits that into the window, the extra goes to width or height
        let sc = (b[5] / 720.0).min(b[4] / 1280.0).max(0.25);
        let (vw, vh) = (b[4] / sc, b[5] / sc);
        self.enc.clear();
        self.enc.layer(LAYER_OVERLAY, 0).space(SPACE_WINDOW, sc);
        self.chip(20.0, 20.0);
        let full = self.preset == FULL;
        // keep-out (spec 5.2): nothing over the retail percent / stock row (the band's bottom 160 of 720) or, in a
        // timed match, the timer (the meter drops to y 120)
        let keep = (b[1] + b[3] * 560.0 / 720.0) / sc;
        let (bx0, bx1) = (b[0] / sc, (b[0] + b[2]) / sc);
        let over = |x0: f32, x1: f32| x1 > bx0 && x0 < bx1;
        let bottom = |x0: f32, x1: f32, h: f32, margin: f32| if over(x0, x1) { keep - 8.0 - h } else { vh - margin - h };
        // the timer is the band's top 110 of 720: below it, scaled with the band
        let meter_y = if self.timed { (b[1] + b[3] * 125.0 / 720.0) / sc } else { 20.0 };
        // the menu covers the middle: only the chip and the toasts stay with it
        let mo = !self.menu.open;
        let on = |id: usize| mo && self.vals[id] != 0;
        let (s_meter, s_move, s_state, s_dummy, s_combo, s_input, s_hints) = (on(H_METER), on(H_MOVE), on(H_STATE), on(H_DUMMY), on(H_COMBO), on(H_INPUT), on(H_HINTS));
        if s_meter {
            self.frame_meter(vw * 0.5 - 350.0, meter_y);
        }
        if s_move {
            self.move_readout(vw - 240.0, 20.0);
        }
        if s_state {
            self.state_readout(vw - 240.0, if s_move { 236.0 } else { 20.0 });
        }
        let dy = if mo && self.drill_st[DS_ACTIVE] != 0 {
            self.drill_panel(20.0, 58.0);
            74.0
        } else {
            0.0
        };
        if s_dummy {
            self.dummy_panel(20.0, 70.0 + dy);
        }
        if mo && self.vals[H_LOOK] != 0 {
            self.look_readout(vw - 480.0, 20.0);
        }
        if s_combo {
            self.combo_panel(20.0, dy + if s_dummy { 300.0 } else { 70.0 });
        }
        if s_input && !self.paused {
            self.input_display(20.0, bottom(20.0, 270.0, 80.0, 20.0));
        }
        let _ = full;
        let hints = |s: &Lab, w0: usize, n: usize| (w0..w0 + n).map(|i| s.w_hint[i * 2] + 5.0 + s.w_hint[i * 2 + 1] + 12.0).sum::<f32>();
        if self.menu.open {
            self.draw_menu(vw);
        } else if self.paused {
            let hy = bottom(vw - 20.0 - hints(self, 4, 4), vw - 20.0, 18.0, 22.0);
            let sy = (hy - 8.0 - 52.0).min(keep - 8.0 - 52.0);
            self.slot_cards(vw * 0.5, sy - 8.0 - 88.0);
            self.scrubber(vw * 0.5 - 400.0, sy);
            self.hints(vw - 20.0, hy, &HINTS_FROZEN, 4, 4);
        } else if s_hints {
            let n = if self.preset == CLEAN { 1 } else { 4 };
            let hy = bottom(vw - 20.0 - hints(self, 0, n), vw - 20.0, 18.0, 22.0);
            self.hints(vw - 20.0, hy, &HINTS_PLAY, 0, n);
        }
        if self.menu.radial {
            self.draw_radial(vw * 0.5, vh * 0.42);
        }
        self.toasts(vw * 0.5, if self.preset == CLEAN { if self.timed { meter_y } else { 40.0 } } else { meter_y + 100.0 });
        self.enc.submit();
    }

    // -------------------------------------------------------------------------------------------- drills

    /// The drill status: a new result becomes a toast; a new drill reads its name, option labels and trend.
    fn drill_frame(&mut self) {
        let mut b = [0u8; DRILL_I32 * 4];
        if unsafe { gd_lab_drill_status(b.as_mut_ptr(), b.len() as u32) } < 0 {
            return;
        }
        let was = self.drill_st[DS_ACTIVE] != 0;
        for i in 0..DRILL_I32 {
            self.drill_st[i] = rd_i32(&b, i * 4);
        }
        let st = self.drill_st;
        if st[DS_ERR] != self.drill_err_seen {
            self.drill_err_seen = st[DS_ERR];
            match st[DS_ERR] {
                1 => self.toast_s("Drill needs a recording in Record slots 1-4"),
                2 => self.toast_s("No such drill"),
                _ => {}
            }
        }
        if st[DS_ACTIVE] == 0 {
            return;
        }
        if !was || self.drill_seen < 0 {
            // a drill (re)started: its texts, once
            let mut t = [0u8; 48];
            let n = unsafe { gd_lab_drill_text(0, st[1] as u32, t.as_mut_ptr(), 48) }.max(0) as usize;
            self.drill_name.clear().s(core::str::from_utf8(&t[..n]).unwrap_or("Drill"));
            for i in 0..(st[DS_NOPT].clamp(0, 8) as usize) {
                let n = unsafe { gd_lab_drill_text(1, i as u32, t.as_mut_ptr(), 48) }.max(0) as usize;
                self.drill_opts[i].clear().s(core::str::from_utf8(&t[..n]).unwrap_or("?"));
            }
            let mut tb = [0u8; 64];
            self.drill_n_trend = unsafe { gd_lab_drill_trend(tb.as_mut_ptr(), 8) }.clamp(0, 8) as usize;
            for i in 0..self.drill_n_trend {
                self.drill_trend[i] = (rd_i32(&tb, i * 8), rd_i32(&tb, i * 8 + 4));
            }
            self.drill_seen = st[DS_SEQ];
            let mut m = Txt::new();
            m.s("Drill · ").s(self.drill_name.as_str());
            self.toast(&m, None);
        }
        self.drill_last = st;
        if st[DS_SEQ] != self.drill_seen {
            self.drill_seen = st[DS_SEQ];
            // the attempt result toast: verdict and option, then the running tally
            let o = st[DS_OPTION].clamp(0, 7) as usize;
            let mut m = Txt::new();
            match st[DS_VERDICT] {
                1 => {
                    m.s("✓ ").s(self.drill_opts[o].as_str());
                    if st[DS_LATE] > 0 {
                        m.s(" · ").i(st[DS_LATE]).s("f late");
                    } else {
                        m.s(" · on time");
                    }
                }
                3 => {
                    m.s("✗ ").s(self.drill_opts[o].as_str()).s(" · punished");
                }
                _ => {
                    m.s("✗ ").s(self.drill_opts[o].as_str()).s(" · missed");
                }
            }
            let mut sub = Txt::new();
            sub.i(st[DS_OK]).s("/").i(st[DS_N]).s(" · ").i(st[DS_OK] * 100 / st[DS_N].max(1)).s("%");
            if st[DS_STREAK] > 1 {
                sub.s(" · streak ").i(st[DS_STREAK]);
            }
            self.toast(&m, Some(&sub));
        }
    }

    /// The in-play drill panel: name, tally, what it waits for, and the option weights as small bars.
    fn drill_panel(&mut self, x: f32, y: f32) {
        let st = self.drill_st;
        let (ac, mu, tx) = (self.t(T_ACCENT), self.t(T_MUTED), self.t(T_TEXT));
        let w = 270.0;
        self.panel(x, y, w, 64.0);
        self.enc.text(x + 12.0, y + 8.0, ROLE_LABEL, ac, "DRILL");
        let name = self.drill_name;
        self.enc.text(x + 60.0, y + 7.0, ROLE_VALUE, tx, name.as_str());
        let mut t = Txt::new();
        t.i(st[DS_OK]).s("/").i(st[DS_N]);
        self.enc.text_ex(x + w - 12.0, y + 7.0, R_NUM, tx, ALIGN_RIGHT, 0.0, t.as_str());
        let phase = match st[DS_PHASE] {
            0 => "set it up",
            1 => "now",
            _ => "next attempt",
        };
        self.enc.text(x + 12.0, y + 28.0, ROLE_LABEL, if st[DS_PHASE] == 1 { ac } else { mu }, phase);
        // the weights: one bar per option, the current pick in the accent
        let n = st[DS_NOPT].clamp(0, 8) as usize;
        let wmax = (0..n).map(|i| st[DS_W + i]).max().unwrap_or(1).max(1);
        for i in 0..n {
            let h = 2.0 + 18.0 * st[DS_W + i] as f32 / wmax as f32;
            let bx = x + w - 12.0 - (n - i) as f32 * 12.0;
            let c = if i as i32 == st[DS_OPTION] && st[DS_PHASE] != 2 { ac } else { mu };
            self.enc.rect(bx, y + 56.0 - h, 8.0, h, c);
        }
    }

    /// The session summary page (Drills > Session summary): totals, per option, and the trend across sessions.
    pub fn draw_summary(&mut self, x0: f32, by: f32, w: f32, h: f32) {
        let st = self.drill_last;
        let (ac, mu, tx, dv) = (self.t(T_ACCENT), self.t(T_MUTED), self.t(T_TEXT), self.t(T_DIVIDER));
        self.panel(x0, by, w, h);
        self.enc.text(x0 + 16.0, by + 12.0, ROLE_TITLE, tx, "Session summary");
        self.enc.text_ex(x0 + w - 16.0, by + 14.0, ROLE_LABEL, mu, ALIGN_RIGHT, 0.0, "B close");
        if st[DS_N] == 0 && self.drill_name.is_empty() {
            self.enc.text(x0 + 16.0, by + 52.0, ROLE_BODY, mu, "No drill yet: Drills > Start drill.");
            return;
        }
        let mut t = Txt::new();
        t.s(self.drill_name.as_str());
        if st[DS_ACTIVE] == 0 {
            t.s(" (stopped)");
        }
        self.enc.text(x0 + 16.0, by + 40.0, ROLE_VALUE, mu, t.as_str());
        // the totals: four figures
        let mean = if st[DS_LATE_N] > 0 { st[DS_LATE_SUM] * 10 / st[DS_LATE_N] } else { -1 };
        let figs: [(&str, i32, u8); 4] = [("attempts", st[DS_N], 0), ("success", st[DS_OK] * 100 / st[DS_N].max(1), 1), ("mean frames late", mean, 2), ("best streak", st[DS_BEST], 0)];
        for (i, (k, v, kind)) in figs.iter().enumerate() {
            let fx = x0 + 16.0 + i as f32 * 196.0;
            t.clear();
            match kind {
                1 => {
                    t.i(*v).s("%");
                }
                2 if *v < 0 => {
                    t.s("–");
                }
                2 => {
                    t.fix(*v as f32 / 10.0, 1);
                }
                _ => {
                    t.i(*v);
                }
            }
            self.enc.text(fx, by + 66.0, R_ADV, if i == 1 { ac } else { tx }, t.as_str());
            self.enc.text(fx, by + 92.0, ROLE_LABEL, mu, k);
        }
        // per option: weight bar, attempts, success
        let ty = by + 124.0;
        self.enc.rect(x0 + 16.0, ty, w - 32.0, 1.0, dv);
        self.enc.text(x0 + 16.0, ty + 6.0, ROLE_LABEL, mu, "option");
        self.enc.text(x0 + 220.0, ty + 6.0, ROLE_LABEL, mu, "dummy picks it");
        self.enc.text_ex(x0 + 520.0, ty + 6.0, ROLE_LABEL, mu, ALIGN_RIGHT, 0.0, "tries");
        self.enc.text_ex(x0 + 620.0, ty + 6.0, ROLE_LABEL, mu, ALIGN_RIGHT, 0.0, "success");
        let n = st[DS_NOPT].clamp(0, 8) as usize;
        let wsum: i32 = (0..n).map(|i| st[DS_W + i]).sum::<i32>().max(1);
        for i in 0..n {
            let ry = ty + 26.0 + i as f32 * 24.0;
            let o = self.drill_opts[i];
            self.enc.text(x0 + 16.0, ry, ROLE_VALUE, tx, o.as_str());
            let share = st[DS_W + i] as f32 / wsum as f32;
            self.enc.rect(x0 + 220.0, ry + 6.0, 200.0, 4.0, self.t(T_CELL));
            self.enc.rect(x0 + 220.0, ry + 6.0, 200.0 * share, 4.0, ac);
            t.clear().i((share * 100.0 + 0.5) as i32).s("%");
            self.enc.text(x0 + 428.0, ry, R_NUM, mu, t.as_str());
            t.clear().i(st[DS_TRIES + i]);
            self.enc.text_ex(x0 + 520.0, ry, R_NUM, tx, ALIGN_RIGHT, 0.0, t.as_str());
            t.clear();
            if st[DS_TRIES + i] > 0 {
                t.i(st[DS_SUCC + i] * 100 / st[DS_TRIES + i]).s("%");
            } else {
                t.s("–");
            }
            self.enc.text_ex(x0 + 620.0, ry, R_NUM, tx, ALIGN_RIGHT, 0.0, t.as_str());
        }
        // the trend: earlier sessions, then this one (success %)
        let gx = x0 + 660.0;
        self.enc.text(gx, ty + 6.0, ROLE_LABEL, mu, "sessions");
        let k = self.drill_n_trend;
        let cur = st[DS_OK] * 1000 / st[DS_N].max(1);
        for i in 0..=k {
            let (v, cur_bar) = if i < k { (self.drill_trend[i].0, false) } else { (cur, true) };
            let bh = 4.0 + 96.0 * v.clamp(0, 1000) as f32 / 1000.0;
            let bx = gx + i as f32 * 16.0;
            self.enc.rect(bx, ty + 130.0 - bh, 11.0, bh, if cur_bar { ac } else { mu });
        }
        if k == 0 {
            self.enc.text(gx, ty + 138.0, ROLE_LABEL, mu, "first session");
        } else {
            t.clear().s("now ").i(cur / 10).s("% · was ").i(self.drill_trend[k - 1].0 / 10).s("%");
            self.enc.text(gx, ty + 138.0, ROLE_LABEL, mu, t.as_str());
        }
    }

    // -------------------------------------------------------------------------------------------- kit parts

    /// Part 1: a panel (theme A: HUD panel bg, 1px border, radius 4).
    fn panel(&mut self, x: f32, y: f32, w: f32, h: f32) {
        let (bg, bd) = (self.t(T_PANEL_HUD), self.t(T_BORDER));
        self.enc.slice(SLICE_FILL_R4, x, y, w, h, bg).slice(SLICE_LINE_R4, x, y, w, h, bd);
    }

    /// Part 5: a name / value row with the divider above it.
    fn row(&mut self, x: f32, y: f32, w: f32, k: &str, v: &str) {
        let (dv, mu, tx) = (self.t(T_DIVIDER), self.t(T_MUTED), self.t(T_TEXT));
        self.enc.rect(x, y, w, 1.0, dv);
        self.enc.text(x, y + 4.0, ROLE_VALUE, mu, k);
        self.enc.text_ex(x + w, y + 4.0, R_NUM, tx, ALIGN_RIGHT, 0.0, v);
    }

    /// Part 7: a key cap at (x, y) of width w.
    fn key(&mut self, x: f32, y: f32, w: f32, s: &str, on: bool) {
        let (bd, kt, ac, ink) = (self.t(T_KEY_BORDER), self.t(T_KEY_TEXT), self.t(T_ACCENT), self.t(T_ACCENT_INK));
        if on {
            self.enc.slice(SLICE_FILL_R4, x, y, w, 18.0, ac);
        }
        self.enc.slice(SLICE_LINE_R4, x, y, w, 18.0, if on { ac } else { bd });
        self.enc.text_ex(x + w * 0.5, y + 4.0, ROLE_LABEL, if on { ink } else { kt }, ALIGN_CENTER, 0.0, s);
    }

    /// Part 2: the session chip.
    fn chip(&mut self, x: f32, y: f32) {
        let key = self.preset as u32 | (self.online as u32) << 4 | (self.status[6] as u32 & 0xFF) << 8 | (self.paused as u32) << 16 | ((self.status[7] != 0) as u32) << 17;
        if key != self.chip_key {
            self.chip_key = key;
            self.chip.clear().s(PRESET_NAMES[self.preset as usize]);
            if self.paused {
                self.chip.s(" · frozen");
            }
            if self.status[7] != 0 {
                self.chip.s(" · ● rec");
            }
            if self.online {
                self.chip.s(" · 2P online · ").i(self.status[6] * 1000 / 60).s(" ms");
            }
            self.chip_w = measure(self.chip.as_str(), ROLE_LABEL);
        }
        let w = 12.0 + self.lab_w + 12.0 + self.chip_w + 12.0;
        self.panel(x, y, w, 30.0);
        let (ac, mu) = (self.t(T_ACCENT), self.t(T_MUTED));
        self.enc.text(x + 12.0, y + 8.0, R_LAB, ac, "LAB");
        self.enc.text(x + 24.0 + self.lab_w, y + 10.0, ROLE_LABEL, mu, self.chip.as_str());
    }

    /// Part 3: toasts, newest on top, fading 1 / 0.75 / 0.5.
    fn toasts(&mut self, cx: f32, y0: f32) {
        let mut y = y0;
        let (ac, bd, mu) = (self.t(T_ACCENT), self.t(T_BORDER), self.t(T_MUTED));
        let bg = 0x080B11E0;
        for i in 0..3 {
            let t = self.toasts[i];
            if t.life == 0 {
                continue;
            }
            self.toasts[i].life -= 1;
            let op = [255u32, 191, 128][i] * if t.life > 30 { 30 } else { t.life } / 30;
            let fade = |c: u32| (c & 0xFFFF_FF00) | ((c & 0xFF) * op / 255);
            self.enc.slice(SLICE_FILL_R4, cx - t.w * 0.5, y, t.w, 26.0, fade(bg)).slice(SLICE_LINE_R4, cx - t.w * 0.5, y, t.w, 26.0, fade(ac));
            self.enc.text_ex(cx, y + 6.0, R_TOAST, fade(ac), ALIGN_CENTER, 0.0, t.main.as_str());
            y += 30.0;
            if !t.sub.is_empty() {
                self.enc.slice(SLICE_FILL_R4, cx - t.sub_w * 0.5, y, t.sub_w, 22.0, fade(0x080B11B3)).slice(SLICE_LINE_R4, cx - t.sub_w * 0.5, y, t.sub_w, 22.0, fade(bd));
                self.enc.text_ex(cx, y + 5.0, R_NUM, fade(mu), ALIGN_CENTER, 0.0, t.sub.as_str());
                y += 26.0;
            }
        }
    }

    /// Part 4: the frame meter (2 rows x 50 cells 9x14, gap 2) with the move summary and the advantage.
    fn frame_meter(&mut self, x: f32, y: f32) {
        self.panel(x, y, 700.0, 76.0);
        let (mu, ac) = (self.t(T_MUTED), self.t(T_ACCENT));
        self.enc.text(x + 14.0, y + 12.0, ROLE_LABEL, mu, "Frame meter");
        let mv = self.mv;
        let key = (mv.first as u64 & 0xFFFF) | (mv.last as u64 & 0xFFFF) << 16 | (mv.n as u64 & 0xFFFF) << 32 | ((mv.action >= 0) as u64) << 48;
        self.sum_row.set(key, |t| {
            if mv.action >= 0 {
                t.s("startup ").i(mv.first + 1).s(" · active ").i(mv.last - mv.first + 1).s(" · recovery ").i((mv.n - mv.last - 1).max(0));
            }
        });
        let adv = self.adv;
        let sh = self.adv_shield;
        self.adv_row.set((adv as u32 as u64) | (sh as u64) << 32, |t| {
            t.int(adv, true).s(if sh { " on shield" } else { " on hit" });
        });
        self.enc.text_ex(x + 350.0, y + 11.0, R_NUM, mu, ALIGN_CENTER, 0.0, self.sum_row.v.as_str());
        self.enc.text_ex(x + 686.0, y + 8.0, R_ADV, ac, ALIGN_RIGHT, 0.0, self.adv_row.v.as_str());
        let cells_x = x + 14.0 + 24.0;
        for p in 0..2 {
            let ry = y + 36.0 + p as f32 * 17.0;
            let pc = self.t(T_P1 + p);
            self.enc.text(x + 14.0, ry + 2.0, ROLE_LABEL, pc, if p == 0 { "P1" } else { "P2" });
            for i in 0..CELLS {
                let k = (self.meter[p][i] as usize).min(5);
                let cx = cells_x + i as f32 * 11.0;
                let c = self.t(T_CELL + k);
                self.enc.rect(cx, ry, 9.0, 14.0, c); // 1 quad (a radius-1 slice is 9)
                if k == CELL_STUN as usize && self.kit_hatch != 0 {
                    // colour-blind aid: the stun hatch
                    self.enc.kit(self.kit_hatch, cx, ry, 1.0, 0xFFFFFF40);
                }
            }
        }
        let _ = T_CELL_ACTIVE;
    }

    /// Part 5: the move readout (P1's last move).
    fn move_readout(&mut self, x: f32, y: f32) {
        self.panel(x, y, 220.0, 158.0);
        let mv = self.mv;
        let ch = self.p1_char;
        self.mv_title.set((mv.action as u32 as u64) | (ch as u32 as u64) << 32, |t| {
            t.s("P1 · ").s(names::CHARS.get(ch as usize).copied().unwrap_or("?"));
            if mv.action >= 0 {
                t.s(" · ").s(short(mv.action));
            }
        });
        let mu = self.t(T_MUTED);
        let lab = unsafe { &mut *(self as *mut Lab) };
        self.enc.text_ex(x + 14.0, y + 12.0, ROLE_LABEL, mu, ALIGN_LEFT, 192.0, lab.mv_title.v.as_str());
        let have = mv.action >= 0;
        let r = &mut lab.mv_rows;
        let k0 = mv.action as u32 as u64;
        r[0].set(k0 | (mv.first as u32 as u64) << 32, |t| { if have { t.i(mv.first + 1); } else { t.s("-"); } });
        r[1].set(k0 | ((mv.last - mv.first) as u32 as u64) << 32, |t| { if have { t.i(mv.first + 1).s("-").i(mv.last + 1); } else { t.s("-"); } });
        r[2].set((mv.landing as u32 as u64) | (mv.lcancel as u64) << 32, |t| {
            if mv.landing < 0 { t.s("-"); } else { t.i(mv.landing); if mv.lcancel { t.s(" (L)"); } }
        });
        r[3].set(k0 | (mv.damage as u32 as u64) << 32, |t| { if have && mv.damage > 0 { t.fix(mv.damage as f32 / 100.0, 0).s("%"); } else { t.s("-"); } });
        r[4].set(k0 | (mv.iasa as u32 as u64) << 32, |t| { if have && mv.iasa >= 0 { t.s("f").i(mv.iasa + 1); } else { t.s("-"); } });
        r[5].set((mv.adv as u32 as u64) | (mv.on_shield as u64) << 32 | (mv.have_adv as u64) << 33, |t| {
            if mv.have_adv && mv.on_shield { t.int(mv.adv, true); } else { t.s("-"); }
        });
        const K: [&str; 6] = ["Startup", "Active", "Landing lag", "Damage", "IASA", "On shield"];
        for i in 0..6 {
            let v = lab.mv_rows[i].v;
            self.row(x + 14.0, y + 31.0 + i as f32 * 20.0, 192.0, K[i], v.as_str());
        }
    }

    /// The lookahead readout: the true advantage of the last exchange (simulated ahead) and, for a hit, the DI fan
    /// (each direction on a numpad: KO red, survives in the cell colour, the advantage it leaves).
    fn look_readout(&mut self, x: f32, y: f32) {
        let n = unsafe { gd_lab_lookahead_result(self.look.as_mut_ptr(), (LOOK_I32 * 4) as u32) };
        let lk = self.look;
        let r = |i: usize| rd_i32(&lk, i * 4);
        if n < (LOOK_HEAD * 4) as i32 || r(1) == 0 {
            return;
        }
        let (state, kind, anchor, adv, runs) = (r(1), r(2), r(3), r(7), r(9));
        self.panel(x, y, 220.0, 112.0);
        let (mu, ac) = (self.t(T_MUTED), self.t(T_ACCENT));
        let mut t = Txt::new();
        t.s("Lookahead · f").i(anchor);
        if state != 3 {
            t.s(if state == 1 { " · freeze to run" } else { " · running" });
        }
        self.enc.text(x + 14.0, y + 12.0, ROLE_LABEL, mu, t.as_str());
        let mut v = Txt::new();
        if adv == i32::MIN {
            v.s("-");
        } else {
            v.int(adv, true).s(if kind == 1 { " on shield" } else { " on hit" });
        }
        self.row(x + 14.0, y + 31.0, 192.0, "True advantage", v.as_str());
        if kind == 0 {
            // the fan: numpad layout, 3 x 3 cells of 20, the advantage per direction under the KO marks
            for i in 0..(runs.max(0) as usize).min(LOOK_RUNS) {
                let b = LOOK_HEAD + i * LOOK_RUN_I32;
                let (tag, out, df, af) = (r(b), r(b + 1), r(b + 2), r(b + 3));
                let (col, row) = (((tag - 1) % 3) as f32, (2 - (tag - 1) / 3) as f32);
                let (cx, cy) = (x + 14.0 + col * 22.0, y + 52.0 + row * 18.0);
                let c = if out == 2 { 0xE0564EE0 } else if out == 1 { self.t(T_CELL) } else { mu };
                self.enc.rect(cx, cy, 20.0, 16.0, c);
                let mut s = Txt::new();
                if out == 2 { s.s("KO"); } else if out == 1 && af >= 0 { s.int(df - af, true); } else { s.s("?"); }
                self.enc.text_ex(cx + 10.0, cy + 2.0, R_NUM, ac, ALIGN_CENTER, 0.0, s.as_str());
            }
            self.enc.text(x + 86.0, y + 56.0, ROLE_LABEL, mu, "DI fan (P-adv)");
        }
    }

    /// Part 5: P1's state readout.
    fn state_readout(&mut self, x: f32, y: f32) {
        self.panel(x, y, 220.0, 138.0);
        let mu = self.t(T_MUTED);
        self.enc.text(x + 14.0, y + 12.0, ROLE_LABEL, mu, "P1 · state");
        if !self.have[0] {
            return;
        }
        let f = self.fighter[0];
        let (ch, act, af) = (rd_i32(&f, F_CHAR), rd_i32(&f, F_ACTION), rd_i32(&f, F_ACTION_FRAME));
        let (vx, vy, hl) = (rd_f32(&f, F_VX), rd_f32(&f, F_VY), rd_f32(&f, F_HITLAG));
        // the action's length: its observed frame data, queried when the action changes (not per frame)
        if self.fd_cache.0 != ch || self.fd_cache.1 != act {
            let n = unsafe { gd_lab_framedata(ch, act, self.fbuf.as_mut_ptr(), 16) };
            self.fd_cache = (ch, act, if n >= 16 { rd_i32(&self.fbuf, 0) } else { 0 });
        }
        let total = self.fd_cache.2;
        let lab = unsafe { &mut *(self as *mut Lab) };
        let r = &mut lab.st_rows;
        r[0].set(act as u32 as u64, |t| { t.s(action_name(act)); if names::action(act).is_none() { t.s(" ").i(act); } });
        r[1].set((af as u32 as u64) | (total as u32 as u64) << 32, |t| { t.i(af); if total >= af && total > 0 { t.s(" / ").i(total); } });
        r[2].set(vx.to_bits() as u64, |t| { t.fix(vx, 2); });
        r[3].set(vy.to_bits() as u64, |t| { t.fix(vy, 2); });
        r[4].set(hl.to_bits() as u64, |t| { t.fix(hl, 0); });
        const K: [&str; 5] = ["Action", "Frame", "Speed X", "Speed Y", "Hitlag"];
        for i in 0..5 {
            let v = lab.st_rows[i].v;
            self.row(x + 14.0, y + 31.0 + i as f32 * 20.0, 192.0, K[i], v.as_str());
        }
    }

    /// The dummy panel: what drives P2 (the Dummy tab, M3, sets the reactions; until then the defaults).
    fn dummy_panel(&mut self, x: f32, y: f32) {
        self.panel(x, y, 200.0, 160.0);
        let mu = self.t(T_MUTED);
        let dp = self.dport();
        let d = self.dv[dp];
        let lab = |t: &'static [(&'static str, i32)], v: i32| t[(v.max(0) as usize).min(t.len() - 1)].0;
        let mut title = Txt::new();
        title.s("P").i(dp as i32 + 1).s(" · ").s(lab(CONTROL, d[D_CONTROL]));
        self.enc.text(x + 14.0, y + 12.0, ROLE_LABEL, mu, title.as_str());
        let rows: [(&str, &str); 5] = [("Block", lab(BLOCK, d[D_BLOCK])), ("Tech", lab(TECH, d[D_TECH])), ("DI", lab(DI, d[D_DI])), ("After shield", lab(OOS, d[D_OOS])), ("SDI", lab(SDI, d[D_SDI]))];
        for (i, (k, v)) in rows.iter().enumerate() {
            self.row(x + 14.0, y + 31.0 + i as f32 * 20.0, 172.0, k, v);
        }
        let (c, s) = if self.status[7] & 2 != 0 {
            (self.t(T_CELL_ACTIVE), "● recording")
        } else if self.status[3] != 0 {
            (self.t(T_ACCENT), "▶ replaying the log")
        } else {
            (mu, "· live")
        };
        self.enc.text(x + 14.0, y + 136.0, ROLE_VALUE, c, s);
    }

    /// The combo panel: hits, damage and the move chain.
    fn combo_panel(&mut self, x: f32, y: f32) {
        self.panel(x, y, 200.0, 84.0);
        let (mu, tx) = (self.t(T_MUTED), self.t(T_TEXT));
        self.enc.text(x + 14.0, y + 12.0, ROLE_LABEL, mu, "Combo");
        let (h, d, ch) = (self.combo_hits, self.combo_dmg, self.combo_chain);
        let lab = unsafe { &mut *(self as *mut Lab) };
        lab.combo_row.set(h as u64, |t| { t.i(h).s(if h == 1 { " hit" } else { " hits" }); });
        lab.combo_dmg_row.set(d as u64, |t| { t.fix(d as f32 / 100.0, 0).s("%"); });
        let key = ch.iter().fold(0u64, |k, &a| k << 16 | (a as u16 as u64));
        lab.chain_row.set(key, |t| {
            let mut first = true;
            for &a in ch.iter().filter(|&&a| a >= 0) {
                if !first { t.s(" › "); }
                first = false;
                t.s(short(a));
            }
        });
        self.enc.text(x + 14.0, y + 30.0, ROLE_BIG, tx, lab.combo_row.v.as_str());
        self.enc.text_ex(x + 186.0, y + 33.0, ROLE_VALUE, mu, ALIGN_RIGHT, 0.0, lab.combo_dmg_row.v.as_str());
        self.enc.text_ex(x + 14.0, y + 60.0, ROLE_LABEL, mu, ALIGN_LEFT, 172.0, lab.chain_row.v.as_str());
    }

    /// Part 6: P1's input display: stick, A B X Y L Z, history with frame counts (stick as arrows).
    fn input_display(&mut self, x: f32, y: f32) {
        self.panel(x, y, 250.0, 80.0);
        let (mu, ac) = (self.t(T_MUTED), self.t(T_ACCENT));
        let (rx, ry) = (x + 12.0, y + 12.0);
        if self.kit_ring != 0 {
            self.enc.kit(self.kit_ring, rx, ry, 1.0, self.t(T_KEY_BORDER));
        }
        let (sx, sy) = self.stick;
        let dx = (sx as f32 / 80.0).clamp(-1.0, 1.0) * 23.0;
        let dy = -(sy as f32 / 80.0).clamp(-1.0, 1.0) * 23.0;
        if self.kit_dot != 0 {
            self.enc.kit(self.kit_dot, rx + 23.0 + dx, ry + 23.0 + dy, 1.0, ac);
        }
        const KEYS: [(&str, u16); 6] = [("A", BA), ("B", BB), ("X", BX), ("Y", BY), ("L", BL), ("Z", BZ)];
        let held = self.held_p1;
        for (i, (k, m)) in KEYS.iter().enumerate() {
            let kx = x + 80.0 + (i % 3) as f32 * 26.0;
            let ky = y + 18.0 + (i / 3) as f32 * 24.0;
            self.key(kx, ky, 22.0, k, held & m != 0);
        }
        let lab = unsafe { &mut *(self as *mut Lab) };
        for i in 0..3 {
            let (b, d, n) = lab.in_hist[i];
            lab.in_rows[i].set(b as u64 | (d as u64) << 16 | (n as u64) << 24, |t| {
                if n == 0 { return; }
                // the stick as an arrow (neutral: nothing, or a dot when no button is held)
                const ARROW: [&str; 10] = ["", "↙", "↓", "↘", "←", "·", "→", "↖", "↑", "↗"];
                if d != 5 || b == 0 { t.s(ARROW[d as usize]); }
                for (k, m) in [("A", BA), ("B", BB), ("X", BX), ("Y", BY), ("L", BL), ("R", BR), ("Z", BZ)] {
                    if b & m != 0 { t.s(k); }
                }
                t.s(" ·").i(n as i32);
            });
            let v = lab.in_rows[i].v;
            self.enc.text(x + 166.0, y + 14.0 + i as f32 * 18.0, ROLE_VALUE, mu, v.as_str());
        }
    }

    /// Part 7: button hints, right-aligned at `right`.
    fn hints(&mut self, right: f32, y: f32, hs: &[(&str, &str); 4], w0: usize, n: usize) {
        let mut x = right;
        let mu = self.t(T_MUTED);
        for i in (0..n).rev() {
            let (kw, lw) = (self.w_hint[(w0 + i) * 2], self.w_hint[(w0 + i) * 2 + 1]);
            x -= lw;
            self.enc.text(x, y + 3.0, ROLE_VALUE, mu, hs[i].1);
            x -= 5.0 + kw;
            self.key(x, y, kw, hs[i].0, false);
            x -= 12.0;
        }
    }

    /// Part 8: the timeline scrubber over the last 600 frames: played part, playhead, save diamonds, hit dots.
    fn scrubber(&mut self, x: f32, y: f32) {
        let (frame, head) = (self.status[1], self.head.max(self.status[1]));
        let lo = (head - RING).max(0);
        let span = (head - lo).max(1) as f32;
        self.panel(x, y, 800.0, 52.0);
        let (tx0, tw) = (x + 14.0, 772.0);
        let (idle, ac, tx, mu, act) = (self.t(T_CELL), self.t(T_ACCENT), self.t(T_TEXT), self.t(T_MUTED), self.t(T_CELL_ACTIVE));
        self.enc.slice(SLICE_FILL_R1, tx0, y + 8.0, tw, 18.0, idle);
        let px = tx0 + tw * ((frame - lo) as f32 / span);
        self.enc.rect(tx0, y + 8.0, px - tx0, 18.0, (ac & 0xFFFF_FF00) | 0x50);
        // marks: hit events from the history (queried when the window moves), saves from the slot frames
        if self.marks_key != (lo, head) {
            self.marks_key = (lo, head);
            let n = unsafe { gd_lab_events_range(lo, head, self.fbuf.as_mut_ptr(), 32) }.max(0) as usize;
            self.n_marks = 0;
            for k in 0..n {
                let e = &self.fbuf[k * 16..k * 16 + 16];
                if (e[0] == EV_HIT || e[0] == EV_SHIELD) && self.n_marks < MARKS {
                    self.marks[self.n_marks] = (rd_i32(e, 4), e[0]);
                    self.n_marks += 1;
                }
            }
        }
        for k in 0..self.n_marks {
            let (f, _) = self.marks[k];
            let mx = tx0 + tw * ((f - lo) as f32 / span);
            if self.kit_dot != 0 {
                self.enc.kit(self.kit_dot, mx - 3.0, y + 14.0, 0.6, act);
            }
        }
        for s in 0..16 {
            let f = self.slots[s];
            if f >= lo && f <= head && self.kit_diamond != 0 {
                let mx = tx0 + tw * ((f - lo) as f32 / span);
                self.enc.kit(self.kit_diamond, mx - 4.5, y + 3.0, 1.0, if s as u32 == self.slot { ac } else { tx });
            }
        }
        self.enc.rect(px - 1.0, y + 5.0, 2.0, 24.0, tx);
        let replay = self.status[3] != 0;
        let lab = unsafe { &mut *(self as *mut Lab) };
        lab.scrub_rows[0].set((head - lo) as u64, |t| { t.s("-").i(head - lo).s("f"); });
        lab.scrub_rows[1].set(frame as u32 as u64 | (head as u32 as u64) << 32, |t| {
            t.s("frame ").i(frame);
            if frame < head { t.s(" (").i(frame - head).s(")"); }
        });
        lab.scrub_rows[2].set(replay as u64, |t| { t.s(if replay { "now · replaying" } else { "now" }); });
        self.enc.text(tx0, y + 33.0, ROLE_LABEL, mu, lab.scrub_rows[0].v.as_str());
        self.enc.text_ex(x + 400.0, y + 32.0, ROLE_VALUE, tx, ALIGN_CENTER, 0.0, lab.scrub_rows[1].v.as_str());
        self.enc.text_ex(tx0 + tw, y + 33.0, ROLE_LABEL, mu, ALIGN_RIGHT, 0.0, lab.scrub_rows[2].v.as_str());
    }

    /// Part 12: slot cards, the page of 8 around the current slot (no thumbnails yet: GAP-3).
    fn slot_cards(&mut self, cx: f32, y: f32) {
        let page = self.slot / 8 * 8;
        let x0 = cx - (8.0 * 96.0 + 7.0 * 8.0) * 0.5;
        let (idle, ac, tx, mu, bd, bg) = (self.t(T_CELL), self.t(T_ACCENT), self.t(T_TEXT), self.t(T_MUTED), self.t(T_BORDER), self.t(T_PANEL_HUD));
        for i in 0..8u32 {
            let s = page + i;
            let x = x0 + i as f32 * 104.0;
            let f = self.slots[s as usize];
            let empty = f < 0;
            let a = |c: u32| if empty { (c & 0xFFFF_FF00) | ((c & 0xFF) * 140 / 255) } else { c };
            let sel = s == self.slot;
            self.enc.slice(SLICE_FILL_R4, x, y, 96.0, 88.0, a(bg));
            self.enc.slice(SLICE_LINE_R4, x, y, 96.0, 88.0, if sel { ac } else { a(bd) });
            self.enc.slice(SLICE_FILL_R1, x + 4.0, y + 4.0, 88.0, 48.0, a(idle));
            let lab = unsafe { &mut *(self as *mut Lab) };
            lab.slot_rows[s as usize].set(f as u32 as u64, |t| { if empty { t.s("empty"); } else { t.s("f").i(f); } });
            let v = lab.slot_rows[s as usize].v;
            self.enc.text_ex(x + 48.0, y + 22.0, ROLE_LABEL, a(mu), ALIGN_CENTER, 0.0, v.as_str());
            let mut name = Txt::new();
            name.s("Slot ").i(s as i32 + 1);
            self.enc.text(x + 6.0, y + 58.0, ROLE_VALUE, if sel { ac } else { a(tx) }, name.as_str());
            self.enc.text(x + 6.0, y + 72.0, ROLE_LABEL, a(mu), if empty { "D↓ saves" } else { "D↑ loads" });
        }
    }
}

#[no_mangle]
pub extern "C" fn gd_ui_init() -> i32 {
    unsafe { (*core::ptr::addr_of_mut!(L)).init() };
    0
}

#[no_mangle]
pub extern "C" fn gd_ui_frame(_present: u32) {
    unsafe { (*core::ptr::addr_of_mut!(L)).frame() };
}

#[repr(C)]
pub struct Manifest {
    magic: u32, struct_size: u32, abi_min: u32, abi_max: u32, hook_table_version: u32, mod_version: u32,
    required_features: u32, optional_features: u32, state_bytes: u32, hook_count: u32, hooks: *const u8,
    mod_id: [u8; 32], mod_name: [u8; 48],
}
unsafe impl Sync for Manifest {}

const fn text<const N: usize>(s: &[u8]) -> [u8; N] {
    let mut o = [0u8; N];
    let mut i = 0;
    while i < s.len() {
        o[i] = s[i];
        i += 1;
    }
    o
}

#[no_mangle]
pub static gdmod_abi_version: u32 = 1 << 16;
#[no_mangle]
pub static gdmod_ui_abi: u32 = UI_ABI_1_1;
#[no_mangle]
pub static gdmod_lab_abi: u32 = 1 << 16;

#[no_mangle]
pub static gd_mod_manifest: Manifest = Manifest {
    magic: 0x444D_4447, struct_size: 124, abi_min: 1 << 16, abi_max: 1 << 16, hook_table_version: 1, mod_version: 0x0001_0000,
    required_features: FEATURE_UI_DRAW | FEATURE_UI_INPUT | FEATURE_LAB | FEATURE_LAB_STATE | FEATURE_LAB_OVERLAY | FEATURE_LAB_INPUT | FEATURE_LAB_CPU,
    optional_features: 0, state_bytes: 0, hook_count: 0,
    hooks: core::ptr::null(), mod_id: text(b"gd.lab"), mod_name: text(b"LAB"),
};
