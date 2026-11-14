//! The Envoy Item Lab, presentation half (envoy_data::lab; owner request 2026-11-13: "SEE all the effects for
//! refining and redesigning"). Three parts:
//! * the setup (in the camp scene, opened by the camp's Item Lab row or the main menu's Item Lab cell right of the
//!   Envoy row): fighter and partner (every taggable fighter, retail and m-ex), stage, foes (none, a dummy that
//!   stands / jumps / shields / plays at a CPU level, a CPU fighter, a 1v3 crowd, a gauntlet, Subspace enemies when
//!   the Brawl files are there), then Start;
//! * the panel, in the Envoy pause style (stickers, D-pad only): over Melee's pause in a Lab session, or mid-fight
//!   with D-pad up (the D-pad stays off the game while it is open). Pages: Items (every item and modifier, filtered,
//!   with its card), Loadout (add all, clear, random, stack 10, four presets saved to the mod's data folder, an A/B
//!   swap), Triggers (fire any trigger or any item now), Effects (slow motion, frame advance, freeze frame, the
//!   category and modifier switches, the combo breaker, crowd knockback, a live stage transition, the pause preview),
//!   Sandbox (percent, damage, reset, coins, stock limit, rooms), Synergy (which modifiers change which items held);
//! * the live trigger log, bottom left, all session.
//! Everything that changes the session is a command to the sim as synced input (`gd_ui_sim_cmd`); the time controls
//! are host-side (`gd_ui_envoy_lab_time`). Every list is read from `ITEMS` / `TRIGGER_TEXT`: new items, modifiers
//! and triggers appear here by themselves. Item text goes through `item_text` (the one place the slice-2 keyword
//! tokens and glossary plug in). No timers anywhere in choosing items.
use crate::kit::*;
use crate::{ck_name, Buf, VW};
use envoy_data::items::*;
use envoy_data::lab::*;
use envoy_data::view::*;
use gd_ui_enc::*;

#[link(wasm_import_module = "gd_ui")]
extern "C" {
    fn gd_ui_strip(port: u32, bits: u32) -> i32;
    fn gd_ui_sim_cmd(port: u32, word: u32) -> i32;
    fn gd_ui_envoy_lab(ptr: *const i32, n: u32) -> i32;
    fn gd_ui_envoy_lab_time(mode: i32, steps: i32) -> i32;
    fn gd_ui_envoy_lab_door_taken() -> i32;
    fn gd_ui_envoy_party(m1: i32, p1: i32, m2: i32, p2: i32) -> i32;
    fn gd_ui_envoy_arm(op: i32) -> i32;
    fn gd_ui_envoy_roster(out: *mut i32, max: i32) -> i32;
    fn gd_ui_envoy_fighter_name(ck: i32, out: *mut u8, cap: i32) -> i32;
    fn gd_ui_data_write(s: *const u8, len: u32, ptr: *const u8, n: u32) -> i32;
    fn gd_ui_data_read(s: *const u8, len: u32, ptr: *mut u8, cap: u32) -> i32;
}

fn log(a: &str, b: &str) {
    let mut m = Buf::new();
    m.s("envoy lab: ").s(a).s(b);
    unsafe { gd_ui_log(m.str().as_ptr(), m.str().len() as u32) };
}
fn logi(a: &str, v: i32) {
    let mut m = Buf::new();
    m.s("envoy lab: ").s(a).i(v);
    unsafe { gd_ui_log(m.str().as_ptr(), m.str().len() as u32) };
}

unsafe fn send(c: u32) {
    gd_ui_sim_cmd(0, c);
    logi("command ", c as i32);
}

/// The one path item text takes to the screen: the card in slice 2's keyword language (envoy_data::keywords, drawn
/// with kit::rich: keywords in their category colours); the plain line for an item with no card.
pub fn item_text(it: &Item) -> &'static str {
    let c = envoy_data::keywords::card(it.id);
    if c.is_empty() { it.line } else { c }
}

// ------------------------------------------------------------------ the setup

const STAGE_IDS: [i32; 5] = [36, 10, 28, 12, 37];
const STAGE_NAMES: [&str; 5] = ["Battlefield", "Yoshi's Story", "Dream Land", "Fountain of Dreams", "Final Destination"];
const NR: usize = 64;
static mut ROSTER: [i32; NR] = [-1; NR];
static mut NROSTER: usize = 0;
static mut NAMES: [[u8; 24]; NR] = [[0; 24]; NR];
static mut NAMEL: [usize; NR] = [0; NR];

struct Setup {
    open: bool,
    age: i32,
    row: usize,
    main: usize,
    partner: usize,
    stage: usize,
    foes: i32,
    dummy: i32,
    level: i32,
    foe: usize, // roster index + 1 (0: random)
    sub: i32,
}
static mut SU: Setup = Setup { open: false, age: 0, row: 0, main: 0, partner: 1, stage: 0, foes: FOE_DUMMY, dummy: DUMMY_STAND, level: 9, foe: 0, sub: 0 };
const SR_MAIN: usize = 0;
const SR_PARTNER: usize = 1;
const SR_STAGE: usize = 2;
const SR_FOES: usize = 3;
const SR_DUMMY: usize = 4;
const SR_LEVEL: usize = 5;
const SR_FOE: usize = 6;
const SR_START: usize = 7;
const SR_BACK: usize = 8;
const SROWS: usize = 9;

fn name_of(i: usize) -> &'static str {
    unsafe {
        let k = ROSTER[i];
        if (0..26).contains(&k) {
            return ck_name(k);
        }
        core::str::from_utf8(&NAMES[i][..NAMEL[i]]).unwrap_or("Fighter")
    }
}

unsafe fn load_roster() {
    let mut w = [0i32; NR];
    let n = gd_ui_envoy_roster(w.as_mut_ptr(), NR as i32).clamp(0, NR as i32) as usize;
    NROSTER = n;
    for i in 0..n {
        ROSTER[i] = w[i] & 0xFFFF;
        NAMEL[i] = gd_ui_envoy_fighter_name(ROSTER[i], NAMES[i].as_mut_ptr(), 24).clamp(0, 24) as usize;
    }
    let fox = (0..n).find(|&i| ROSTER[i] == 2).unwrap_or(0);
    let falcon = (0..n).find(|&i| ROSTER[i] == 0).unwrap_or(1.min(n.saturating_sub(1)));
    SU.main = falcon;
    SU.partner = fox;
}

pub fn setup_is_open() -> bool {
    unsafe { SU.open }
}

pub unsafe fn setup_open() {
    if NROSTER == 0 {
        load_roster();
    }
    SU.open = true;
    SU.age = 0;
    SU.row = SR_START;
    gd_ui_envoy_lab_door_taken();
    log("setup open", "");
}

/// The setup screen inside the camp scene. Returns 1 when the session was started (the camp closes), 2 on Back.
pub unsafe fn setup_frame(e: &mut E, pad: &crate::camp::Pad, brawl: bool) -> i32 {
    let s = &mut SU;
    s.age += 1;
    let n = NROSTER.max(1);
    let mut out = 0;
    if s.age > 3 {
        if pad.up {
            s.row = (s.row + SROWS - 1) % SROWS;
        }
        if pad.down {
            s.row = (s.row + 1) % SROWS;
        }
        let d = if pad.right { 1 } else if pad.left { -1 } else { 0 };
        if d != 0 {
            let step = |v: usize, m: usize| ((v as i32 + d).rem_euclid(m as i32)) as usize;
            match s.row {
                SR_MAIN => s.main = step(s.main, n),
                SR_PARTNER => s.partner = step(s.partner, n),
                SR_STAGE => s.stage = step(s.stage, STAGE_IDS.len()),
                SR_FOES => {
                    let m = if brawl { 6 } else { 5 };
                    s.foes = step(s.foes as usize, m) as i32;
                }
                SR_DUMMY => s.dummy = step(s.dummy as usize, 4) as i32,
                SR_LEVEL => s.level = (s.level + d).clamp(1, 9),
                SR_FOE => s.foe = step(s.foe, n + 1),
                _ => {}
            }
        }
        if pad.b {
            out = 2;
        }
        if pad.a || pad.start {
            match s.row {
                SR_BACK => out = 2,
                SR_START => out = 1,
                _ if pad.start => out = 1,
                _ => {}
            }
        }
    }
    if out == 1 {
        let mut w = [0i32; LAB_WORDS];
        w[L_ON] = 1;
        w[L_STAGE] = STAGE_IDS[s.stage];
        w[L_FOES] = s.foes;
        w[L_DUMMY] = s.dummy;
        w[L_LEVEL] = s.level;
        w[L_FOE_CK] = if s.foe == 0 { -1 } else { ROSTER[s.foe - 1] };
        let (m, p) = (ROSTER[s.main], ROSTER[if s.partner == s.main { (s.partner + 1) % n } else { s.partner }]);
        w[L_MAIN] = m;
        w[L_PARTNER] = p;
        w[L_SUBSPACE] = if s.foes == FOE_SUBSPACE { 100 + s.sub } else { 0 };
        gd_ui_envoy_party(m, p, -1, -1);
        gd_ui_envoy_lab(w.as_ptr(), LAB_WORDS as u32);
        let rc = gd_ui_envoy_arm(5 | 0x10);
        logi("session started, arm rc ", rc);
        log("foes ", FOE_TEXT[s.foes as usize]);
        PANEL = P0;
        s.open = false;
    } else if out == 2 {
        s.open = false;
        log("setup closed", "");
    }
    draw_setup(e, brawl);
    out
}

unsafe fn draw_setup(e: &mut E, brawl: bool) {
    let s = &SU;
    e.layer(LAYER_MENU, 0);
    spr(e, S_BG_CAMP, -107.0, 0.0, 854.0, 480.0, WHITE);
    e.rect(-107.0, 0.0, 854.0, 480.0, 0x1A10_4A90);
    spr_c(e, S_EMBLEM, 36.0, 30.0, 1.0, WHITE);
    text_edge(e, 56.0, 15.0, d(23), WHITE, ALIGN_LEFT, 2.2, "Item Lab");
    e.text_ex(186.0, 22.0, bs(12), PALE, ALIGN_LEFT, 0.0, "a sandbox: nothing here is saved to your run");
    let foe_name = if s.foe == 0 { "Random" } else { name_of(s.foe - 1) };
    let mut lv = Buf::new();
    lv.s("Level ").i(s.level);
    let rows: [(&str, &str); SROWS] = [
        ("Fighter", name_of(s.main)),
        ("Partner", name_of(s.partner)),
        ("Stage", STAGE_NAMES[s.stage]),
        ("Foes", FOE_TEXT[s.foes as usize]),
        ("Dummy does", if s.foes == FOE_DUMMY { DUMMY_TEXT[s.dummy as usize] } else { "-" }),
        ("CPU level", lv.str()),
        ("Foe fighter", if matches!(s.foes, FOE_DUMMY | FOE_CPU) { foe_name } else { "-" }),
        ("Start the Lab", ""),
        ("Back", ""),
    ];
    let mut y = 58.0;
    for (i, (label, value)) in rows.iter().enumerate() {
        let on = i == s.row;
        let big = i == SR_START;
        let h = if big { 40.0 } else { 30.0 };
        let x = 30.0 + if on { 12.0 } else { 0.0 };
        let w = if i == SR_BACK { 112.0 } else { 300.0 };
        sticker(e, if on { S_FACE_HOVER } else if big { S_FACE_SUNNY } else { S_FACE_WHITE }, x, y, w, h, if on { 0.0 } else { [-1.0, 0.8, -0.6, 0.9][i % 4] }, WHITE, |e| {
            e.text_ex(x + 12.0, y + h * 0.5 - 8.0, d(if big { 18 } else { 15 }), INK, ALIGN_LEFT, 0.0, label);
            if !value.is_empty() {
                e.text_ex(x + w - 12.0, y + h * 0.5 - 7.0, bs(12), if on { INK } else { LILAC }, ALIGN_RIGHT, 0.0, value);
            }
        });
        if on {
            spr(e, S_PTR, 6.0, y + h * 0.5 - 13.0, 0.0, 0.0, WHITE);
        }
        y += h + 6.0;
    }
    // the right card: what the session is
    let (cx, cy) = (370.0, 70.0);
    sticker(e, S_FACE_INK, cx, cy, 240.0, 250.0, 0.8, WHITE, |e| {
        e.text_ex(cx + 14.0, cy + 12.0, d(16), SUNNY, ALIGN_LEFT, 0.0, "In the Lab");
        let lines = [
            "Every item and modifier, any stack.",
            "Fire any trigger with no input.",
            "Slow motion, frame advance, freeze.",
            "Switch effects and modifiers off.",
            "Presets, random loadouts, A/B swap.",
            "99 stocks, reset, percent, coins.",
            "D-pad up: the panel, mid-fight.",
            "Start: the panel over the pause.",
        ];
        for (k, l) in lines.iter().enumerate() {
            text_fit(e, cx + 14.0, cy + 42.0 + k as f32 * 24.0, bs, 12, 9, WHITE, ALIGN_LEFT, 214.0, l);
        }
    });
    if !brawl {
        text_fit(e, 370.0, 336.0, bs, 11, 8, PALE, ALIGN_LEFT, 240.0, "Subspace enemies need your Brawl files.");
    }
    hints(e, &[("Up / Down", "Choose"), ("Left / Right", "Change"), ("A", "Start"), ("B", "Back")]);
}

// ------------------------------------------------------------------ the panel

/// The pages' names: the tabs carry them whole (uiremake: the tabs were a sixth of the bar each and carried
/// abbreviations, "Kit", "Fire", "FX", "State", with "Combos" still cut).
const PAGE_NAMES: [&str; 6] = ["Items", "Loadout", "Triggers", "Effects", "Sandbox", "Synergy"];
const PAGES: [&str; 6] = PAGE_NAMES;
const PG_ITEMS: usize = 0;
const PG_LOADOUT: usize = 1;
const PG_TRIG: usize = 2;
const PG_FX: usize = 3;
const PG_SAND: usize = 4;
const PG_SYN: usize = 5;
const PRESETS: usize = 4;
const IDS: usize = 64;

#[derive(Clone, Copy)]
struct Panel {
    open: bool,
    /// opened by D-pad up mid-fight (not the pause)
    hot: bool,
    page: usize,
    row: usize,
    scroll: usize,
    filter: usize,
    item: i32,
    time: i32,
    me_pct: i32,
    foe_pct: i32,
    coins: i32,
    lim: i32,
    stocks: i32,
    ab: usize,
    pause_preview: bool,
    freeze: bool,
    age: i32,
    said: &'static str,
    said_at: i32,
}
const P0: Panel = Panel {
    open: false, hot: false, page: 0, row: 0, scroll: 0, filter: 0, item: 0, time: 0, me_pct: 0, foe_pct: 0, coins: 0,
    lim: 4, stocks: 4, ab: 0, pause_preview: false, freeze: false, age: 0, said: "", said_at: -100,
};
static mut PANEL: Panel = P0;
static mut PRESET: [[u8; IDS]; PRESETS + 2] = [[0; IDS]; PRESETS + 2];
static mut PRESETS_READ: bool = false;
static mut CAP: u32 = 0;
static mut WAS_PAUSED: bool = false;
static mut T: i32 = 0;

fn say(p: &mut Panel, s: &'static str) {
    p.said = s;
    p.said_at = unsafe { T };
    log("", s);
}

/// The filters: everything, the modifiers, then each trigger by name (TRIGGER_TEXT order).
fn filters() -> usize {
    2 + triggers()
}
fn filter_name(f: usize) -> &'static str {
    match f {
        0 => "All items",
        1 => "Modifiers",
        _ => TRIGGER_TEXT[(f - 2).min(TRIGGER_TEXT.len() - 1)],
    }
}
fn passes(it: &Item, f: usize) -> bool {
    match f {
        0 => true,
        1 => it.trigger == TR_MODIFIER,
        _ => it.trigger as usize == f - 2,
    }
}
fn listed(f: usize, out: &mut [usize; ITEMS_LEN]) -> usize {
    let mut n = 0;
    for (i, it) in ITEMS.iter().enumerate() {
        if passes(it, f) {
            out[n] = i;
            n += 1;
        }
    }
    n
}

/// The Effects page's rows: time, step, freeze, the four categories, each modifier, break me / foe, crowd, shift,
/// pause preview.
fn modifiers(out: &mut [usize; ITEMS_LEN]) -> usize {
    let mut n = 0;
    for (i, it) in ITEMS.iter().enumerate() {
        if it.trigger == TR_MODIFIER {
            out[n] = i;
            n += 1;
        }
    }
    n
}
fn fx_rows() -> usize {
    let mut m = [0usize; ITEMS_LEN];
    3 + CAT_TEXT.len() + modifiers(&mut m) + 5
}
const SAND_ROWS: usize = 7 + 7;
fn trig_rows() -> usize {
    triggers() + ITEMS_LEN
}
const LOAD_ROWS: usize = 4 + PRESETS + 1;

fn rows_of(page: usize) -> usize {
    match page {
        PG_ITEMS => 1 + {
            let mut l = [0usize; ITEMS_LEN];
            listed(unsafe { PANEL.filter }, &mut l)
        },
        PG_LOADOUT => LOAD_ROWS,
        PG_TRIG => trig_rows(),
        PG_FX => fx_rows(),
        PG_SAND => SAND_ROWS,
        _ => 0,
    }
}

fn inv(v: &[i32; VW], id: i32) -> i32 {
    // player 1's copies; ids 48+ (the synergy lane) live in V_INV2 (inv_word)
    if (1..ITEM_IDS as i32).contains(&id) { v[inv_word(0, id as usize)] } else { 0 }
}
fn off(v: &[i32; VW], id: i32) -> bool {
    (0..64).contains(&id) && (v[VL_OFF + id as usize / 32] as u32) >> (id % 32) & 1 != 0
}

unsafe fn presets_load() {
    if PRESETS_READ {
        return;
    }
    PRESETS_READ = true;
    let n = b"presets.bin";
    let mut b = [0u8; IDS * (PRESETS + 2)];
    let got = gd_ui_data_read(n.as_ptr(), n.len() as u32, b.as_mut_ptr(), b.len() as u32);
    if got as usize == b.len() {
        for k in 0..PRESETS + 2 {
            PRESET[k].copy_from_slice(&b[k * IDS..(k + 1) * IDS]);
        }
        log("presets loaded", "");
    }
}
unsafe fn presets_save() {
    let n = b"presets.bin";
    let mut b = [0u8; IDS * (PRESETS + 2)];
    for k in 0..PRESETS + 2 {
        b[k * IDS..(k + 1) * IDS].copy_from_slice(&PRESET[k]);
    }
    let rc = gd_ui_data_write(n.as_ptr(), n.len() as u32, b.as_ptr(), b.len() as u32);
    logi("presets saved, rc ", rc);
}
unsafe fn snapshot(v: &[i32; VW], k: usize) {
    for id in 0..IDS {
        PRESET[k][id] = inv(v, id as i32).clamp(0, 99) as u8;
    }
}
unsafe fn apply(k: usize) {
    send(cmd(LC_CLEAR, 0, 0));
    for it in ITEMS.iter() {
        let c = PRESET[k][(it.id as usize).min(IDS - 1)];
        if c > 0 {
            send(cmd(LC_SET, it.id, c as i32));
        }
    }
}

/// Input every present in a Lab match: the D-pad up hotkey, the panel's navigation and the D-pad strip. `paused`:
/// Melee's pause is up. Returns true while the panel is drawn.
pub unsafe fn input(v: &[i32; VW], paused: bool) -> bool {
    T += 1;
    presets_load();
    let p = &mut PANEL;
    if paused && !WAS_PAUSED {
        p.open = true;
        p.hot = false;
        p.age = 0;
    } else if !paused && WAS_PAUSED && !p.hot {
        p.open = false;
    }
    WAS_PAUSED = paused;
    if CAP == 0 {
        CAP = gd_ui_capture(0x1, CAP_SHARED);
    }
    let mut o = [0u8; 16];
    let edge = if gd_ui_pad(0, o.as_mut_ptr()) == 0 { u16::from_le_bytes([o[12], o[13]]) as u32 } else { 0 };
    // D-pad up: the panel mid-fight (it is also the freeze frame's way back)
    if !paused && edge & 0x8 != 0 && !(p.open && p.row > 0) {
        if p.freeze {
            p.freeze = false;
            gd_ui_envoy_lab_time(p.time, 0);
        } else {
            p.open = !p.open;
            p.hot = p.open;
            p.age = 0;
            p.row = 0;
        }
        log(if p.open { "panel open, page " } else { "panel closed" }, if p.open { PAGE_NAMES[p.page] } else { "" });
    } else if p.open && p.age > 2 {
        nav(p, v, edge & 0xF);
    }
    p.age += 1;
    // the D-pad is the panel's while it is open; D-pad up (the hotkey) never reaches the game in a Lab
    gd_ui_strip(0, if p.open { 0xF } else { 0x8 });
    p.open && !p.freeze
}

pub fn pause_preview() -> bool {
    unsafe { PANEL.pause_preview }
}

unsafe fn nav(p: &mut Panel, v: &[i32; VW], e: u32) {
    if e == 0 {
        return;
    }
    let (left, right, down, up) = (e & 1 != 0, e & 2 != 0, e & 4 != 0, e & 8 != 0);
    let n = rows_of(p.page);
    if p.row == 0 {
        if left || right {
            p.page = (p.page + if right { 1 } else { PAGES.len() - 1 }) % PAGES.len();
            p.scroll = 0;
            log("page ", PAGE_NAMES[p.page]);
        } else if down && n > 0 {
            p.row = 1;
        } else if up && p.hot {
            p.open = false;
            log("panel closed", "");
        }
        return;
    }
    if up {
        p.row -= 1;
        return;
    }
    if down {
        p.row = (p.row + 1).min(n);
        return;
    }
    let d = if right { 1 } else { -1 };
    let r = p.row - 1;
    match p.page {
        PG_ITEMS => {
            if r == 0 {
                p.filter = ((p.filter as i32 + d).rem_euclid(filters() as i32)) as usize;
                log("filter ", filter_name(p.filter));
                return;
            }
            let mut l = [0usize; ITEMS_LEN];
            let cnt = listed(p.filter, &mut l);
            if r - 1 < cnt {
                let it = &ITEMS[l[r - 1]];
                p.item = it.id;
                send(cmd(if right { LC_ADD } else { LC_SUB }, it.id, 0));
            }
        }
        PG_LOADOUT => match r {
            0 => {
                send(cmd(LC_ALL, 0, 1));
                say(p, "Every item, one copy each");
            }
            1 => {
                send(cmd(LC_CLEAR, 0, 0));
                say(p, "Loadout cleared");
            }
            2 => {
                send(cmd(LC_RANDOM, T & 0xFF, 5));
                say(p, "A random loadout of five");
            }
            3 => {
                let id = if p.item > 0 { p.item } else { ITEMS[0].id };
                send(cmd(LC_SET, id, 10));
                say(p, "Ten copies of the item you last chose");
            }
            k if k < 4 + PRESETS => {
                let s = k - 4;
                if right {
                    snapshot(v, s);
                    presets_save();
                    say(p, ["Saved to preset 1", "Saved to preset 2", "Saved to preset 3", "Saved to preset 4"][s]);
                } else {
                    // said first: apply logs one command per item, and the host keeps only so many log lines a present
                    say(p, ["Preset 1 loaded", "Preset 2 loaded", "Preset 3 loaded", "Preset 4 loaded"][s]);
                    apply(s);
                }
            }
            _ => {
                // A / B: the loadout now goes to its slot, the other slot's comes in
                snapshot(v, PRESETS + p.ab);
                p.ab ^= 1;
                apply(PRESETS + p.ab);
                presets_save();
                say(p, if p.ab == 0 { "Swapped to loadout A" } else { "Swapped to loadout B" });
            }
        },
        PG_TRIG => {
            if !right {
                return;
            }
            if r < triggers() {
                send(cmd(LC_TRIGGER, r as i32, 0));
                log("trigger fired: ", TRIGGER_TEXT[r]);
            } else if r - triggers() < ITEMS_LEN {
                let it = &ITEMS[r - triggers()];
                send(cmd(LC_FIRE, it.id, 0));
                log("item fired: ", it.key);
            }
        }
        PG_FX => fx_row(p, v, r, d),
        PG_SAND => sand_row(p, v, r, d, right),
        _ => {}
    }
}

unsafe fn fx_row(p: &mut Panel, v: &[i32; VW], r: usize, d: i32) {
    let mut m = [0usize; ITEMS_LEN];
    let nm = modifiers(&mut m);
    let cats = CAT_TEXT.len();
    if r == 0 {
        p.time = (p.time + d).rem_euclid(4);
        gd_ui_envoy_lab_time(p.time, 0);
        log("time ", TIME_TEXT[p.time as usize]);
    } else if r == 1 {
        if d > 0 {
            p.time = TIME_PAUSED;
            let q = gd_ui_envoy_lab_time(TIME_PAUSED, 1);
            logi("frame advance, queued ", q);
        }
    } else if r == 2 {
        // the freeze frame: the logic stops and the panel hides (Melee's pause camera still moves); D-pad up again
        p.freeze = true;
        gd_ui_envoy_lab_time(TIME_PAUSED, 0);
        say(p, "Frozen: pause for the camera, D-pad up to go on");
    } else if r < 3 + cats {
        let bit = 1 << (r - 3);
        let now = v[VL_CATS] ^ bit;
        send(cmd(LC_CATS, now, 0));
        log(if now & bit != 0 { "category off: " } else { "category on: " }, CAT_TEXT[r - 3]);
    } else if r < 3 + cats + nm {
        let it = &ITEMS[m[r - 3 - cats]];
        let was = off(v, it.id);
        send(cmd(LC_OFF, it.id, (!was) as i32));
        log(if was { "modifier on: " } else { "modifier off: " }, it.key);
    } else {
        match r - 3 - cats - nm {
            0 => send(cmd(LC_BREAK, 0, 0)),
            1 => send(cmd(LC_BREAK, 1, 0)),
            2 => send(cmd(LC_CROWD, (v[VL_CROWD] == 0) as i32, 0)),
            3 => {
                send(cmd(LC_SHIFT, 0, 0));
                say(p, "Stage transition: the rift sweep");
            }
            _ => {
                p.pause_preview = !p.pause_preview;
                log("pause preview ", if p.pause_preview { "on" } else { "off" });
            }
        }
    }
}

unsafe fn sand_row(p: &mut Panel, v: &[i32; VW], r: usize, d: i32, right: bool) {
    match r {
        0 => {
            p.me_pct = (p.me_pct + 10 * d).clamp(0, 300);
            send(cmd(LC_PCT_ME, p.me_pct & 0xFF, p.me_pct >> 8));
        }
        1 => {
            p.foe_pct = (p.foe_pct + 10 * d).clamp(0, 300);
            send(cmd(LC_PCT_FOE, p.foe_pct & 0xFF, p.foe_pct >> 8));
        }
        2 => {
            if right {
                send(cmd(LC_DAMAGE, 10, 0));
            }
        }
        3 => {
            if right {
                send(cmd(LC_RESET, 0, 0));
                p.me_pct = 0;
                p.foe_pct = 0;
                say(p, "Reset: places and percents");
            }
        }
        4 => {
            p.coins = (v[V_COINS] + 50 * d).clamp(0, 9999);
            send(cmd(LC_COINS, p.coins & 0xFF, p.coins >> 8));
        }
        5 => {
            p.lim = (v[V_POOL_LIM].max(1) + d).clamp(1, 8);
            send(cmd(LC_LIMIT, p.lim, 0));
        }
        6 => {
            p.stocks = (v[V_POOL].max(1) + d).clamp(1, 99);
            send(cmd(LC_LIMIT, v[V_POOL_LIM].max(1), p.stocks));
        }
        k => {
            let room = (k - 6) as i32;
            if right {
                if v[VL_ROOMS] >> room & 1 != 0 {
                    send(cmd(LC_ROOM, room, 0));
                    log("room preview: ", ROOM_TEXT[room as usize]);
                } else {
                    say(p, "That room comes with slice 2");
                }
            }
        }
    }
}

// ------------------------------------------------------------------ drawing (band space, 640 x 480)

const PX: f32 = 14.0;
/// The panel's size this present (`draw` sets them): 300 wide on a 4:3 screen, where the six tab names take two
/// rows; up to 380 on a wide window, where they fit one.
static mut PWV: f32 = 300.0;
static mut PHV: f32 = 392.0;
static mut PYV: f32 = 40.0;
static mut TABH: f32 = 28.0;
fn pw() -> f32 {
    unsafe { PWV }
}
fn ph() -> f32 {
    unsafe { PHV }
}
const TAB_PX: u32 = 10;
/// The panel's right edge (band units) while it is open: what the HUD's centred cards keep clear of.
pub fn right_edge(ex: f32) -> f32 {
    PX - ex + pw() + 8.0
}
const VISIBLE: usize = 9;

pub unsafe fn draw(e: &mut E, v: &[i32; VW], ex: f32) {
    let p = PANEL;
    e.layer(LAYER_MENU, 5);
    let (sc, a) = enter(p.age);
    PWV = (300.0 + ex).min(380.0);
    TABH = tab_h(tab_rows(PWV - 16.0, TAB_PX, &PAGES), 22.0);
    PHV = 392.0 + TABH - 28.0;
    PYV = 40.0 - (TABH - 28.0) * 0.5;
    let x = PX - ex;
    claim_left(right_edge(ex));
    let y = PYV + (1.0 - sc) * 30.0;
    sticker(e, S_FACE_INK, x, y, pw(), ph(), -0.6, alpha(WHITE, a), |e| {
        face(e, S_FACE_INK, x, y, pw(), ph(), alpha(WHITE, a));
        text_edge(e, x + 12.0, y + 8.0, d(18), SUNNY, ALIGN_LEFT, 1.2, "ITEM LAB");
        let mut hd = Buf::new();
        hd.i(v[V_POOL]).s(" stocks   ").i(v[V_COINS]).s(" coins");
        e.text_ex(x + pw() - 12.0, y + 13.0, bs(11), PALE, ALIGN_RIGHT, 0.0, hd.str());
        tabs(e, &p, x + 8.0, y + 38.0);
        let cy = y + 42.0 + TABH;
        match p.page {
            PG_ITEMS => items_page(e, v, &p, x + 10.0, cy),
            PG_LOADOUT => loadout_page(e, v, &p, x + 10.0, cy),
            PG_TRIG => trig_page(e, v, &p, x + 10.0, cy),
            PG_FX => fx_page(e, v, &p, x + 10.0, cy),
            PG_SAND => sand_page(e, v, &p, x + 10.0, cy),
            _ => synergy_page(e, v, x + 10.0, cy),
        }
        let hint = if T - p.said_at < 120 && !p.said.is_empty() {
            p.said
        } else if p.row == 0 {
            "D-pad: left / right page, down to open"
        } else {
            match p.page {
                PG_ITEMS => "Up / down: row. Left / right: fewer / more",
                PG_TRIG => "Up / down: row. Right: fire now",
                PG_LOADOUT => "Right: do it (presets: left load, right save)",
                _ => "Up / down: row. Left / right: change",
            }
        };
        text_fit(e, x + pw() * 0.5, y + ph() - 20.0, bs, 11, 8, if T - p.said_at < 120 { SUNNY } else { PALE }, ALIGN_CENTER, pw() - 16.0, hint);
    });
}

fn tabs(e: &mut E, p: &Panel, x: f32, y: f32) {
    // every tab as wide as its word (kit::tab_row): two rows where one cannot hold the six names
    tab_row(e, x, y, pw() - 16.0, 22.0, TAB_PX, &PAGES, p.page, p.row == 0);
}

/// A list row: label left, value right; hovered rows move right on the hover face.
fn row(e: &mut E, x: f32, y: f32, w: f32, on: bool, label: &str, value: &str, dim: bool) {
    let dx = if on { 6.0 } else { 0.0 };
    face(e, if on { S_FACE_HOVER } else { S_FACE_WHITE }, x + dx, y, w - 6.0, 20.0, if dim { alpha(WHITE, 0.55) } else { WHITE });
    // the label takes what the value leaves (set smaller when it must be: never cut)
    let vw = if value.is_empty() { 0.0 } else { tw(value, bs(11)) + 10.0 };
    text_fit(e, x + dx + 8.0, y + 3.0, bs, 11, 8, INK, ALIGN_LEFT, w - 6.0 - 8.0 - 14.0 - vw, label);
    if !value.is_empty() {
        e.text_ex(x + dx + w - 14.0, y + 3.0, bs(11), if on { INK } else { LILAC }, ALIGN_RIGHT, 0.0, value);
    }
}

/// Rows `first..` of a page that fit, scrolled so the cursor shows: the index range drawn.
fn window(p: &Panel, n: usize, vis: usize) -> (usize, usize) {
    let cur = p.row.saturating_sub(1);
    let start = if cur >= vis { cur + 1 - vis } else { 0 };
    (start, (start + vis).min(n))
}

fn icon(e: &mut E, it: &Item, cx: f32, cy: f32, r: f32, dim: bool) {
    spr_c(e, S_DISC, cx, cy, r / 32.0, INK);
    spr_c(e, S_DISC, cx, cy, (r - 4.0) / 32.0, if dim { alpha(it.rgba, 0.35) } else { it.rgba });
    if it.trigger == TR_MODIFIER {
        spr_c(e, S_SPARK, cx, cy, 0.6, WHITE);
    }
}

unsafe fn items_page(e: &mut E, v: &[i32; VW], p: &Panel, x: f32, y: f32) {
    let w = pw() - 20.0;
    let mut f = Buf::new();
    f.s("< ").s(filter_name(p.filter)).s(" >");
    row(e, x, y, w, p.row == 1, "Show", f.str(), false);
    let mut l = [0usize; ITEMS_LEN];
    let n = listed(p.filter, &mut l);
    let (a, b) = window(&Panel { row: p.row.saturating_sub(1), ..*p }, n, 6);
    for (k, i) in (a..b).enumerate() {
        let it = &ITEMS[l[i]];
        let on = p.row == i + 2;
        let ry = y + 24.0 + k as f32 * 22.0;
        let c = inv(v, it.id);
        let mut val = Buf::new();
        if c > 0 {
            val.s("x").i(c);
        } else {
            val.s("-");
        }
        row(e, x + 18.0, ry, w - 18.0, on, it.name, val.str(), off(v, it.id));
        icon(e, it, x + 8.0, ry + 10.0, 16.0, c == 0);
    }
    // the card: the hovered item, else the first listed
    let sel = if p.row >= 2 && p.row - 2 < n { l[p.row - 2] } else if n > 0 { l[0] } else { return };
    card(e, v, &ITEMS[sel], x, y + 24.0 + 6.0 * 22.0 + 4.0, w);
}

unsafe fn card(e: &mut E, v: &[i32; VW], it: &Item, x: f32, y: f32, w: f32) {
    let c = inv(v, it.id);
    let h = 136.0;
    sticker(e, S_FACE_WHITE, x, y, w, h, 0.5, WHITE, |e| {
        e.rect(x + 6.0, y + 6.0, 5.0, h - 12.0, it.rgba);
        text_fit(e, x + 18.0, y + 6.0, d, 15, 10, INK, ALIGN_LEFT, w - 30.0 - it.quality.min(4) as f32 * 13.0, it.name);
        for q in 0..it.quality.min(4) {
            spr_c(e, S_STAR_ON, x + w - 14.0 - q as f32 * 13.0, y + 14.0, 1.0, WHITE);
        }
        let mut t = Buf::new();
        t.s(if it.trigger == TR_MODIFIER { "Modifier" } else { "Trigger: " });
        if it.trigger != TR_MODIFIER {
            t.s(TRIGGER_TEXT[(it.trigger as usize).min(TRIGGER_TEXT.len() - 1)]);
        }
        face(e, S_FACE_INK, x + 18.0, y + 28.0, tw(t.str(), bs(10)) + 12.0, 16.0, WHITE);
        e.text_ex(x + 24.0, y + 29.0, bs(10), SUNNY, ALIGN_LEFT, 0.0, t.str());
        let lines = rich(e, x + 18.0, y + 47.0, w - 26.0, 10, 11.5, INK, false, item_text(it), 3);
        // two lines of card text leave the old places; a third moves what follows down a line
        let y = y + (lines - 2).max(0) as f32 * 11.5;
        let mut nb = Buf::new();
        nb.s("Held ").i(c).s(" / cap ").i(it.cap as i32).s("   copies: ").s(STACK_TEXT[(it.stack as usize).min(4)]);
        text_fit(e, x + 18.0, y + 72.0, bs, 10, 7, LILAC, ALIGN_LEFT, w - 26.0, nb.str());
        let m = stack_mul(it.id, c.max(1));
        let mut mb = Buf::new();
        mb.s("Strength now x").i(m / 1000).s(".").i(m % 1000 / 100).s("   price ").i(item_price(it.id)).s("   id ").i(it.id);
        text_fit(e, x + 18.0, y + 86.0, bs, 10, 7, LILAC, ALIGN_LEFT, w - 26.0, mb.str());
        let mut tg = Buf::new();
        let cats = if it.trigger == TR_MODIFIER { 0 } else { item_cats(it) };
        tg.s(if it.trigger == TR_MODIFIER { "Changes: " } else { "Makes: " });
        let mask = if it.trigger == TR_MODIFIER { cats_of_tags(it.mods) } else { cats };
        let mut any = false;
        for (k, n) in CAT_TEXT.iter().enumerate() {
            if mask >> k & 1 != 0 {
                if any {
                    tg.s(", ");
                }
                tg.s(n);
                any = true;
            }
        }
        if !any {
            tg.s("nothing on screen");
        }
        text_fit(e, x + 18.0, y + 100.0, bs, 10, 7, INK, ALIGN_LEFT, w - 26.0, tg.str());
        if off(v, it.id) {
            // beside the trigger pill (the card's foot belongs to its last line)
            e.text_ex(x + w - 12.0, y + 29.0 - (lines - 2).max(0) as f32 * 11.5, bs(10), CORAL, ALIGN_RIGHT, 0.0, "switched off");
        }
    });
}

const fn cats_of_tags(t: u8) -> i32 {
    (if t & (TG_PROJECTILE | TG_SPARK) != 0 { CAT_PROJECTILE } else { 0 })
        | (if t & (TG_BURST | TG_LAUNCH) != 0 { CAT_BURST } else { 0 })
        | (if t & TG_TRAIL != 0 { CAT_TRAIL } else { 0 })
        | (if t & TG_ORBITAL != 0 { CAT_AURA } else { 0 })
}

/// The Loadout page's row pitch.
const LROW: f32 = 22.0;

unsafe fn loadout_page(e: &mut E, v: &[i32; VW], p: &Panel, x: f32, y: f32) {
    let w = pw() - 20.0;
    let labels = ["Add all", "Clear", "Random (5)", "Stack 10 of the last item"];
    for (k, l) in labels.iter().enumerate() {
        row(e, x, y + k as f32 * LROW, w, p.row == k + 1, l, "", false);
    }
    for s in 0..PRESETS {
        let n: i32 = PRESET[s].iter().map(|&c| (c > 0) as i32).sum();
        let mut val = Buf::new();
        if n == 0 { val.s("empty") } else { val.i(n).s(" items") };
        let name = ["Preset 1", "Preset 2", "Preset 3", "Preset 4"][s];
        row(e, x, y + (4 + s) as f32 * LROW, w, p.row == 5 + s, name, val.str(), false);
    }
    row(e, x, y + (4 + PRESETS) as f32 * LROW, w, p.row == 5 + PRESETS, "A / B swap", if p.ab == 0 { "on A" } else { "on B" }, false);
    // the loadout now: an icon per held item
    let iy = y + (5 + PRESETS) as f32 * LROW + 6.0;
    e.text_ex(x, iy, bs(10), PALE, ALIGN_LEFT, 0.0, "Equipped now");
    let mut k = 0;
    for it in ITEMS.iter() {
        let c = inv(v, it.id);
        if c > 0 {
            // every held item stays inside the panel: rows of 24-unit cells across its width, 21 apart (a full
            // loadout is four rows at 300 wide; it ran off the panel's foot and onto the HUD at 26 a row)
            let per = (((w - 4.0) / 24.0) as usize).max(1);
            let cx = x + 12.0 + (k % per) as f32 * 24.0;
            let cy = iy + 22.0 + (k / per) as f32 * 21.0;
            icon(e, it, cx, cy, 20.0, off(v, it.id));
            if c > 1 {
                let mut s = Buf::new();
                s.i(c);
                e.text_ex(cx + 9.0, cy + 2.0, d(8), SUNNY, ALIGN_CENTER, 0.0, s.str());
            }
            k += 1;
        }
    }
}

unsafe fn trig_page(e: &mut E, v: &[i32; VW], p: &Panel, x: f32, y: f32) {
    let w = pw() - 20.0;
    let n = trig_rows();
    let (a, b) = window(p, n, 12);
    for (k, r) in (a..b).enumerate() {
        let ry = y + k as f32 * 22.0;
        let on = p.row == r + 1;
        if r < triggers() {
            let held = ITEMS.iter().filter(|it| it.trigger as usize == r && inv(v, it.id) > 0).count() as i32;
            let mut val = Buf::new();
            val.i(held).s(" held  >");
            let mut l = Buf::new();
            l.s("On ").s(TRIGGER_TEXT[r]);
            row(e, x, ry, w, on, l.str(), val.str(), held == 0);
        } else {
            let it = &ITEMS[r - triggers()];
            row(e, x + 18.0, ry, w - 18.0, on, it.name, "fire  >", false);
            icon(e, it, x + 8.0, ry + 10.0, 16.0, inv(v, it.id) == 0);
        }
    }
}

unsafe fn fx_page(e: &mut E, v: &[i32; VW], p: &Panel, x: f32, y: f32) {
    let w = pw() - 20.0;
    let mut m = [0usize; ITEMS_LEN];
    let nm = modifiers(&mut m);
    let n = fx_rows();
    let (a, b) = window(p, n, 12);
    let cats = CAT_TEXT.len();
    for (k, r) in (a..b).enumerate() {
        let ry = y + k as f32 * 22.0;
        let on = p.row == r + 1;
        if r == 0 {
            row(e, x, ry, w, on, "Time", TIME_TEXT[p.time as usize], false);
        } else if r == 1 {
            row(e, x, ry, w, on, "Frame advance", "+1  >", false);
        } else if r == 2 {
            row(e, x, ry, w, on, "Freeze frame", "  >", false);
        } else if r < 3 + cats {
            let offc = v[VL_CATS] >> (r - 3) & 1 != 0;
            row(e, x, ry, w, on, CAT_TEXT[r - 3], if offc { "off" } else { "on" }, offc);
        } else if r < 3 + cats + nm {
            let it = &ITEMS[m[r - 3 - cats]];
            let o = off(v, it.id);
            row(e, x + 18.0, ry, w - 18.0, on, it.name, if o { "off" } else { "on" }, o);
            icon(e, it, x + 8.0, ry + 10.0, 16.0, o);
        } else {
            let (l, val) = match r - 3 - cats - nm {
                0 => ("Combo breaker: you", "break  >"),
                1 => ("Combo breaker: the foe", "break  >"),
                2 => ("Crowd knockback", if v[VL_CROWD] != 0 { "on" } else { "off" }),
                3 => ("Stage transition", "rift  >"),
                _ => ("Pause overlay preview", if p.pause_preview { "on" } else { "off" }),
            };
            row(e, x, ry, w, on, l, val, false);
        }
    }
}

unsafe fn sand_page(e: &mut E, v: &[i32; VW], p: &Panel, x: f32, y: f32) {
    let w = pw() - 20.0;
    let (a, b) = window(p, SAND_ROWS, 12);
    for (k, r) in (a..b).enumerate() {
        let ry = y + k as f32 * 22.0;
        let on = p.row == r + 1;
        let mut val = Buf::new();
        let label = match r {
            0 => {
                val.i(p.me_pct).s("%");
                "Your percent"
            }
            1 => {
                val.i(p.foe_pct).s("%");
                "Foes' percent"
            }
            2 => {
                val.s("+10%  >");
                "Damage the foes"
            }
            3 => {
                val.s("  >");
                "Reset places and percents"
            }
            4 => {
                val.i(v[V_COINS]);
                "Coins"
            }
            5 => {
                val.i(v[V_POOL_LIM]);
                "Stock limit"
            }
            6 => {
                val.i(v[V_POOL]);
                "Stocks"
            }
            k => {
                let room = (k - 6) as i32;
                let ok = v[VL_ROOMS] >> room & 1 != 0;
                val.s(if ok { "preview  >" } else { "slice 2" });
                ROOM_TEXT[room as usize]
            }
        };
        let dim = r >= 7 && v[VL_ROOMS] >> (r - 6) & 1 == 0;
        row(e, x, ry, w, on, label, val.str(), dim);
    }
}

/// Every held modifier and the held items it changes (their tags meet), and the modifiers not held that would
/// change something held: the synergies to look for.
unsafe fn synergy_page(e: &mut E, v: &[i32; VW], x: f32, y: f32) {
    let w = pw() - 20.0;
    let mut yy = y;
    let mut any = false;
    for m in ITEMS.iter().filter(|m| m.trigger == TR_MODIFIER) {
        let held = inv(v, m.id) > 0;
        let mut hits = [0usize; ITEMS_LEN];
        let mut n = 0;
        for (i, it) in ITEMS.iter().enumerate() {
            if it.trigger != TR_MODIFIER && it.tags & m.mods != 0 && inv(v, it.id) > 0 {
                hits[n] = i;
                n += 1;
            }
        }
        if !held && n == 0 {
            continue;
        }
        any = true;
        let hot = held && n > 0 && !off(v, m.id);
        let h = if n == 0 { 34.0 } else { 24.0 + ((n + 1) / 2) as f32 * 20.0 };
        face(e, if hot { S_FACE_SUNNY } else { S_FACE_WHITE }, x, yy, w, h, if held { WHITE } else { alpha(WHITE, 0.6) });
        icon(e, m, x + 12.0, yy + 11.0, 16.0, !held);
        let mut t = Buf::new();
        t.s(m.name).s(if held { "" } else { " (not held)" }).s(if hot { "  -  active" } else { "" });
        text_fit(e, x + 24.0, yy + 4.0, bs, 11, 8, INK, ALIGN_LEFT, w - 32.0, t.str());
        for k in 0..n {
            let it = &ITEMS[hits[k]];
            let cx = x + 16.0 + (k % 2) as f32 * (w * 0.5);
            let cy = yy + 32.0 + (k / 2) as f32 * 20.0;
            icon(e, it, cx, cy, 16.0, !hot);
            text_fit(e, cx + 12.0, cy - 7.0, bs, 10, 7, INK, ALIGN_LEFT, w * 0.5 - 32.0, it.name);
        }
        if n == 0 {
            e.text_ex(x + 24.0, yy + 4.0 + 13.0, bs(9), LILAC, ALIGN_LEFT, 0.0, "nothing held it changes");
        }
        yy += h + 6.0;
        if yy > y + 280.0 {
            break;
        }
    }
    if !any {
        e.text_ex(x + w * 0.5, y + 40.0, bs(12), PALE, ALIGN_CENTER, 0.0, "No modifier held, and none that");
        e.text_ex(x + w * 0.5, y + 58.0, bs(12), PALE, ALIGN_CENTER, 0.0, "would change what you hold.");
    }
}

/// The live trigger log, bottom left: the last triggers, newest first (an on-demand fire shows the item).
pub unsafe fn trigger_log(e: &mut E, v: &[i32; VW], ex: f32) {
    let n = v[VL_TLOG_N].max(0) as usize;
    let play = v[V_PLAY];
    e.layer(LAYER_MENU, 2);
    // right-aligned in the bottom right corner, over player 2's corner in co-op
    let xr = 626.0 + ex;
    let mut y = if v[V_HUMANS] >= 2 { 346.0 } else { 440.0 };
    for k in 0..n.min(6) {
        let i = (n - 1 - k) % TLOG;
        let b = VL_TLOG + i * TL_W;
        let (at, p, tr, fired) = (v[b], v[b + 1], v[b + 2], v[b + 3]);
        let age = play - at;
        if age > 600 {
            break;
        }
        let a = if age < 420 { 1.0 } else { 1.0 - (age - 420) as f32 / 180.0 };
        let mut s = Buf::new();
        if p > 0 {
            s.s("P2 ");
        }
        if tr >= LT_FIRE {
            s.s("fire ").s(item(tr - LT_FIRE).map_or("?", |it| it.name));
        } else {
            s.s(TRIGGER_TEXT[(tr.max(0) as usize).min(TRIGGER_TEXT.len() - 1)]);
            if fired > 0 {
                s.s("  ->  ").i(fired).s(if fired == 1 { " item" } else { " items" });
            }
        }
        let wd = tw(s.str(), bs(11)) + 16.0;
        let x = xr - wd;
        face(e, if k == 0 && age < 30 { S_FACE_SUNNY } else { S_FACE_INK }, x, y, wd, 18.0, alpha(WHITE, a));
        e.text_ex(x + 8.0, y + 2.0, bs(11), if k == 0 && age < 30 { INK } else { WHITE }, ALIGN_LEFT, 0.0, s.str());
        y -= 21.0;
    }
    let _ = ck_name;
}
