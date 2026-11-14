//! The Envoy pause overlay (owner request 2026-11-13): over Melee's own pause, never instead of it. The retail pause
//! keeps its camera (sticks, X / Y), Start and the L + R + A + Start quit; the overlay is driven by the D-pad alone,
//! and the D-pad is taken off the game's pad while the overlay is up (`gd_ui_strip`) so a press meant for it never
//! reaches the sim (a classic tag, Volley) or the pause camera. A D-pad direction still held when the pause ends
//! stays stripped until it is let go (no press edge on the first live frame).
//!
//! Each human player's pad drives that player's own panel (co-op: P1 left, P2 right). A panel has a tab bar
//! (row 0: left / right change the page, down enters it) and four pages:
//! * ITEMS: the icon strip of the items you hold; left / right hover one, its card says name, trigger, effect and
//!   the stack (copies, cap, what a copy adds).
//! * RUN: the loop, the room (or the boss), the road taken, the map of the rooms ahead and the boss at its end.
//! * FIGHT: every foe in the match (foe / elite / boss, percent, stocks, a boss's HP) and its statuses (burning,
//!   chilled, marked, haste, shield); your fighters' percent.
//! * OPTIONS: Envoy settings (text size, effect intensity, screen shake) and Abandon run (right, then right again).
//! The header on every page: stocks against the stock limit, and the coins.
//!
//! Presentation only: the overlay reads the published view. The one exception is Abandon, which goes to the sim as
//! synced input (`gd_ui_sim_signal`, envoy_sim `abandon_check`), so rollback and netplay stay deterministic.
use crate::kit::*;
use crate::{ck_name, Buf, VW};
use envoy_data::items::*;
use envoy_data::procs;
use envoy_data::view::*;
use gd_ui_enc::*;

#[link(wasm_import_module = "gd_ui")]
extern "C" {
    fn gd_ui_strip(port: u32, bits: u32) -> i32;
    fn gd_ui_sim_signal(port: u32, bits: u32, frames: u32) -> i32;
}

const DPAD: u32 = 0x000F;
const ABANDON_BIT: u32 = 0x2000;
const PAGES: [&str; 6] = ["Items", "Run", "Fight", "Options", "Words", "Combos"];
const PG_ITEMS: usize = 0;
const PG_RUN: usize = 1;
const PG_FIGHT: usize = 2;
const PG_OPTIONS: usize = 3;
const PG_WORDS: usize = 4;
const PG_COMBOS: usize = 5;
/// The Combos page's entries: the synergies, the evolutions, then the proc glossary.
const COMBOS: usize = procs::SYNERGIES.len() + procs::EVOLUTIONS.len();
const COMBO_ROWS: usize = COMBOS + procs::KEYWORDS.len();
const OPT_ROWS: usize = 4;

// ------------------------------------------------------------------ Envoy settings (presentation only)

/// Text size: 0 small, 1 normal, 2 large (the overlay's text).
pub static mut TEXT: usize = 1;
/// Effect intensity in quarters: 0 off .. 4 full (screen effects and the items' world effects).
pub static mut FX: i32 = 4;
/// Screen shake: the shockwave / radial wave distortions and the HUD's bar shake.
pub static mut SHAKE: bool = true;

/// The effect intensity as a factor 0..1.
pub fn fx_scale() -> f32 {
    unsafe { FX.clamp(0, 4) as f32 * 0.25 }
}
pub fn shake_on() -> bool {
    unsafe { SHAKE }
}

/// A text size scaled by the setting.
fn ts(px: u32) -> u32 {
    let k = unsafe { [0.85f32, 1.0, 1.15][TEXT.min(2)] };
    ((px as f32 * k) + 0.5) as u32
}
fn td(px: u32) -> i32 {
    d(ts(px))
}
fn tb(px: u32) -> i32 {
    bs(ts(px))
}

// ------------------------------------------------------------------ per-player panel state

#[derive(Clone, Copy)]
struct Panel {
    page: usize,
    /// 0: the tab bar; 1: inside the page.
    row: usize,
    item: usize,
    opt: usize,
    confirm: bool,
    /// Presents since the abandon signal was sent (0: none).
    abandoned: i32,
    /// Presents since this pause opened (the panel's enter animation).
    age: i32,
    moved: i32,
}
const P0: Panel = Panel { page: 0, row: 0, item: 0, opt: 0, confirm: false, abandoned: 0, age: 0, moved: -100 };
static mut PANELS: [Panel; 2] = [P0; 2];
static mut CAP: u32 = 0;
/// D-pad bits still taken off each port (the pause, then until they are let go).
static mut LATCH: [u32; 2] = [0; 2];
static mut WAS_PAUSED: bool = false;

fn log(a: &str, b: &str) {
    let mut m = Buf::new();
    m.s("envoy pause: ").s(a).s(b);
    unsafe { gd_ui_log(m.str().as_ptr(), m.str().len() as u32) };
}

unsafe fn raw(port: usize) -> Option<(u32, u32)> {
    let mut o = [0u8; 16];
    if gd_ui_pad(port as u32, o.as_mut_ptr()) != 0 {
        return None;
    }
    Some((u16::from_le_bytes([o[0], o[1]]) as u32, u16::from_le_bytes([o[12], o[13]]) as u32))
}

/// Input and stripping, every present (also outside the pause: the latch lets go of a held D-pad). `paused`: the
/// retail pause is up in a live Envoy match. Returns true while the overlay should be drawn.
pub unsafe fn input(v: &[i32; VW], paused: bool) -> bool {
    let humans = v[V_HUMANS].clamp(1, 2) as usize;
    if paused && !WAS_PAUSED {
        for p in 0..2 {
            let keep = PANELS[p];
            PANELS[p] = P0;
            // the page a player was on is remembered between pauses; the confirm is not
            PANELS[p].page = keep.page;
            PANELS[p].item = keep.item;
        }
    }
    if paused != WAS_PAUSED {
        log(if paused { "open, page " } else { "closed" }, if paused { PAGES[PANELS[0].page] } else { "" });
    }
    WAS_PAUSED = paused;
    let need_cap = paused || LATCH.iter().any(|&l| l != 0);
    if need_cap && CAP == 0 {
        CAP = gd_ui_capture(0x3, CAP_SHARED);
    }
    for p in 0..2 {
        let (held, edge) = if CAP != 0 { raw(p).unwrap_or((0, 0)) } else { (0, 0) };
        if paused && p < humans {
            LATCH[p] = DPAD;
            let pn = &mut PANELS[p];
            pn.age += 1;
            if pn.abandoned > 0 {
                pn.abandoned += 1;
            } else if pn.age > 2 {
                nav(pn, v, p, edge & DPAD);
            }
        } else {
            LATCH[p] &= held;
        }
        gd_ui_strip(p as u32, LATCH[p]);
    }
    if !need_cap && CAP != 0 {
        gd_ui_release(CAP);
        CAP = 0;
    }
    paused
}

fn held_items(v: &[i32; VW], p: usize) -> ([usize; ITEMS_LEN], usize) {
    let mut out = [0usize; ITEMS_LEN];
    let mut n = 0;
    for (i, it) in ITEMS.iter().enumerate() {
        if v[inv_word(p, it.id as usize)] > 0 {
            out[n] = i;
            n += 1;
        }
    }
    (out, n)
}

unsafe fn nav(pn: &mut Panel, v: &[i32; VW], p: usize, e: u32) {
    if e == 0 {
        return;
    }
    let (left, right, down, up) = (e & 1 != 0, e & 2 != 0, e & 4 != 0, e & 8 != 0);
    pn.moved = pn.age;
    if pn.row == 0 {
        // L / R walk the four main pages (Items, Run, Fight, Options; the order the pad checks rely on); up on the tab
        // row walks the extra pages: Words (the keyword glossary), Combos (synergies, evolutions), then back to Items;
        // from an extra page, left / right leave it
        const CYCLE: usize = 4;
        let extra = pn.page >= CYCLE;
        if up {
            pn.page = match pn.page {
                PG_WORDS => PG_COMBOS,
                PG_COMBOS => PG_ITEMS,
                _ => PG_WORDS,
            };
            pn.opt = 0;
            log("page ", PAGES[pn.page]);
        } else if left {
            pn.page = if extra { PG_OPTIONS } else { (pn.page + CYCLE - 1) % CYCLE };
            pn.opt = 0;
            log("page ", PAGES[pn.page]);
        } else if right {
            pn.page = if extra { PG_ITEMS } else { (pn.page + 1) % CYCLE };
            pn.opt = 0;
            log("page ", PAGES[pn.page]);
        } else if down {
            pn.row = 1;
        }
        return;
    }
    match pn.page {
        PG_ITEMS => {
            let (_, n) = held_items(v, p);
            if up {
                pn.row = 0;
            } else if n > 0 && left {
                pn.item = (pn.item + n - 1) % n;
            } else if n > 0 && right {
                pn.item = (pn.item + 1) % n;
            }
        }
        PG_COMBOS => {
            if up {
                pn.row = 0;
            } else if left {
                pn.opt = (pn.opt + COMBO_ROWS - 1) % COMBO_ROWS;
            } else if right || down {
                pn.opt = (pn.opt + 1) % COMBO_ROWS;
            }
        }
        PG_OPTIONS => {
            if up && pn.opt == 0 {
                pn.row = 0;
                pn.confirm = false;
            } else if up {
                pn.opt -= 1;
                pn.confirm = false;
            } else if down {
                pn.opt = (pn.opt + 1).min(OPT_ROWS - 1);
                pn.confirm = false;
            } else {
                let d = if right { 1 } else { -1 };
                match pn.opt {
                    0 => {
                        TEXT = (TEXT as i32 + d).clamp(0, 2) as usize;
                        log("option text size ", ["small", "normal", "large"][TEXT]);
                    }
                    1 => {
                        FX = (FX + d).clamp(0, 4);
                        log("option effects ", ["off", "25%", "50%", "75%", "full"][FX as usize]);
                    }
                    2 => {
                        SHAKE = !SHAKE;
                        log("option screen shake ", if SHAKE { "on" } else { "off" });
                    }
                    _ => {
                        if left {
                            pn.confirm = false;
                        } else if !pn.confirm {
                            pn.confirm = true;
                            log("abandon asked", "");
                        } else {
                            // the one action that changes the run: synced input, held a few logic frames
                            gd_ui_sim_signal(p as u32, ABANDON_BIT, 6);
                            pn.abandoned = 1;
                            log("abandon sent, player ", if p == 0 { "1" } else { "2" });
                            pn.confirm = false;
                        }
                    }
                }
            }
        }
        PG_WORDS => {
            // the glossary scrolls a keyword at a time (pn.item is the top row)
            if up && pn.item == 0 {
                pn.row = 0;
            } else if up {
                pn.item -= 1;
            } else if down {
                pn.item = (pn.item + 1).min(envoy_data::keywords::KEYWORDS.len() - WORDS_ROWS);
            }
        }
        _ => {
            if up {
                pn.row = 0;
            }
        }
    }
}

const WORDS_ROWS: usize = 6;

/// The glossary: every keyword of the item language, in its colour, and what it means.
unsafe fn words_page(e: &mut E, pn: &Panel, x: f32, y: f32) {
    use envoy_data::keywords::*;
    let w = pw() - 20.0;
    let top = if pn.page == PG_WORDS { pn.item.min(KEYWORDS.len() - WORDS_ROWS) } else { 0 };
    for (r, kw) in KEYWORDS.iter().enumerate().skip(top).take(WORDS_ROWS) {
        let ry = y + (r - top) as f32 * 27.0;
        let col = KC_DARK[kw.cat as usize];
        e.rect(x, ry + 2.0, 3.0, 22.0, col);
        e.text_ex(x + 8.0, ry, td(12), col, ALIGN_LEFT, 0.0, kw.word);
        let cat = KC_NAME[kw.cat as usize];
        e.text_ex(x + w, ry + 1.0, tb(9), PALE, ALIGN_RIGHT, 0.0, cat);
        text_fit(e, x + 8.0, ry + 13.0, tb, 10, 7, WHITE, ALIGN_LEFT, w - 10.0, kw.def);
    }
    let mut b = Buf::new();
    b.i(top as i32 + 1).s("-").i((top + WORDS_ROWS) as i32).s(" of ").i(KEYWORDS.len() as i32);
    e.text_ex(x + w, y + WORDS_ROWS as f32 * 27.0 + 2.0, tb(10), PALE, ALIGN_RIGHT, 0.0, b.str());
}

// ------------------------------------------------------------------ drawing (band space, 640 x 480)

/// The panel's size this present (`draw` sets them): wide enough for the six tabs' words on one row wherever the
/// screen has the room (uiremake: at 232 wide the tabs were 36 units each and their words were cut, "Optio",
/// "Comb"); two panels on a 4:3 screen have less, and there the tabs take two rows and the panel grows by a row.
static mut PWV: f32 = 312.0;
static mut PHV: f32 = 318.0;
static mut PYV: f32 = 52.0;
/// The height of the tab bar this present (one row or two).
static mut TABH: f32 = 28.0;
fn pw() -> f32 {
    unsafe { PWV }
}
fn ph() -> f32 {
    unsafe { PHV }
}
const TAB_PX: u32 = 10;

/// The overlay over the retail pause: a panel per human player.
pub unsafe fn draw(e: &mut E, v: &[i32; VW], ex: f32, boss_petey: bool) {
    let humans = v[V_HUMANS].clamp(1, 2) as usize;
    e.layer(LAYER_MENU, 4);
    // one panel: 312 wide; two: what half the screen leaves each (4:3: 298, a wide window: 312)
    PWV = if humans == 2 { ((640.0 + 2.0 * ex - 44.0) * 0.5).clamp(232.0, 312.0) } else { 312.0 };
    let rows = tab_rows(PWV - 16.0, TAB_PX, &PAGES);
    TABH = tab_h(rows, 22.0);
    PHV = 318.0 + TABH - 28.0;
    PYV = 52.0 - (TABH - 28.0);
    for p in 0..humans {
        let x = if p == 0 { 14.0 - ex } else { 626.0 + ex - pw() };
        // the HUD's centred cards keep clear of the panel (kit::clear_span)
        if p == 0 {
            claim_left(x + pw() + 10.0);
        } else {
            claim_right(x - 10.0);
        }
        panel(e, v, p, x, if p == 0 { -0.8 } else { 0.8 }, boss_petey, humans);
    }
}

unsafe fn panel(e: &mut E, v: &[i32; VW], p: usize, x: f32, deg: f32, petey: bool, humans: usize) {
    let pn = PANELS[p];
    let (sc, a) = enter(pn.age);
    let y = PYV + (1.0 - sc) * 40.0;
    let col = PCOL[p];
    // the ink sticker twice: the baked face is see-through at its middle, and the panel must read over any stage
    sticker(e, S_FACE_INK, x, y, pw(), ph(), deg, alpha(WHITE, a), |e| {
        face(e, S_FACE_INK, x, y, pw(), ph(), alpha(WHITE, a));
        // title strip: ENVOY + the player tag
        let mut t = Buf::new();
        t.s("ENVOY");
        if humans == 2 {
            t.s(if p == 0 { "  P1" } else { "  P2" });
        }
        text_edge(e, x + 12.0, y + 8.0, d(18), SUNNY, ALIGN_LEFT, 1.2, t.str());
        e.text_ex(x + pw() - 12.0, y + 13.0, bs(11), PALE, ALIGN_RIGHT, 0.0, "paused");
        header(e, v, p, x + 10.0, y + 36.0);
        tabs(e, &pn, x + 8.0, y + 84.0, col);
        let cy = y + 88.0 + TABH;
        match pn.page {
            PG_ITEMS => items_page(e, v, p, &pn, x + 10.0, cy),
            PG_RUN => run_page(e, v, p, x + 10.0, cy, petey),
            PG_COMBOS => combos_page(e, v, p, &pn, x + 10.0, cy),
            PG_FIGHT => fight_page(e, v, x + 10.0, cy),
            PG_WORDS => words_page(e, &pn, x + 10.0, cy),
            _ => options_page(e, &pn, x + 10.0, cy),
        }
        // the hint line at the panel's foot
        let hint = if pn.abandoned > 0 {
            "Run abandoned. Ending the match..."
        } else if pn.row == 0 {
            "D-pad: left / right page, down to open"
        } else if pn.page == PG_OPTIONS {
            "D-pad: up / down row, left / right set"
        } else if pn.page == PG_WORDS {
            "D-pad: up / down to scroll"
        } else if pn.page == PG_ITEMS {
            "D-pad: left / right item, up to the tabs"
        } else if pn.page == PG_COMBOS {
            "D-pad: left / right entry, up to the tabs"
        } else {
            "D-pad up: back to the tabs"
        };
        text_fit(e, x + pw() * 0.5, y + ph() - 22.0, bs, 11, 8, if pn.abandoned > 0 { CORAL } else { PALE }, ALIGN_CENTER, pw() - 16.0, hint);
    });
}

/// Stocks against the limit, and coins.
unsafe fn header(e: &mut E, v: &[i32; VW], p: usize, x: f32, y: f32) {
    let (pool, lim) = crate::hud::stocks_of(v, p);
    let lim = if lim > 0 { lim } else { pool.max(4) };
    face(e, S_FACE_WHITE, x, y, pw() - 20.0, 40.0, WHITE);
    e.text_ex(x + 8.0, y + 3.0, bs(10), LILAC, ALIGN_LEFT, 0.0, "STOCKS");
    let n = lim.max(pool).clamp(1, 8);
    for i in 0..n {
        // on the white sticker the held stocks are ink discs (the HUD's white icons would vanish), the empty slots rings
        let cx = x + 16.0 + i as f32 * 17.0;
        crate::icons::slot_ink(e, cx, y + 26.0, 13.0, i < pool, pool == 1, 1.0);
    }
    let mut s = Buf::new();
    s.i(pool).s("/").i(lim);
    let mut c = Buf::new();
    c.i(v[V_COINS]);
    // from the right: the coins (their number never runs off the sticker), then the limit
    let w = pw() - 20.0;
    let cw = tw(c.str(), d(13));
    let coin_x = x + w - 12.0 - cw - 12.0;
    spr_c(e, S_COIN, coin_x, y + 26.0, 1.0, WHITE);
    e.text_ex(coin_x + 11.0, y + 18.0, d(13), INK, ALIGN_LEFT, 0.0, c.str());
    let lx = (coin_x - 20.0 - tw(s.str(), d(14)).max(tw("LIMIT", bs(10)))).max(x + 16.0 + n as f32 * 17.0);
    e.text_ex(lx, y + 3.0, bs(10), LILAC, ALIGN_LEFT, 0.0, "LIMIT");
    e.text_ex(lx, y + 16.0, d(14), INK, ALIGN_LEFT, 0.0, s.str());
}

unsafe fn tabs(e: &mut E, pn: &Panel, x: f32, y: f32, col: u32) {
    // every tab as wide as its word (kit::tab_row): no word is cut to a share of the bar
    let h = tab_row(e, x, y, pw() - 16.0, 22.0, TAB_PX, &PAGES, pn.page, pn.row == 0);
    // the player's colour under the tab bar
    e.rect(x + 2.0, y + h - 2.0, pw() - 20.0, 2.0, alpha(col, 0.9));
}

fn trigger_word(it: &Item) -> &'static str {
    TRIGGER_TEXT[(it.trigger as usize).min(TRIGGER_TEXT.len() - 1)]
}

unsafe fn items_page(e: &mut E, v: &[i32; VW], p: usize, pn: &Panel, x: f32, y: f32) {
    let (list, n) = held_items(v, p);
    let w = pw() - 20.0;
    if n == 0 {
        e.text_ex(x + w * 0.5, y + 40.0, tb(13), PALE, ALIGN_CENTER, 0.0, "No items yet.");
        e.text_ex(x + w * 0.5, y + 62.0, tb(11), PALE, ALIGN_CENTER, 0.0, "The item room, the shop");
        e.text_ex(x + w * 0.5, y + 76.0, tb(11), PALE, ALIGN_CENTER, 0.0, "and the boss hand them out.");
        return;
    }
    let sel = pn.item.min(n - 1);
    // the icon strip: as many a row as the panel's width takes (26 units each)
    let per = (((w - 4.0) / 26.0) as usize).max(1);
    for k in 0..n {
        let it = &ITEMS[list[k]];
        let cx = x + 12.0 + (k % per) as f32 * 26.0;
        let cy = y + 11.0 + (k / per) as f32 * 25.0;
        let on = k == sel && pn.row == 1;
        if on {
            spr_rot(e, S_BURST_SUNNY, cx, cy, 0.52, 8.0, WHITE);
        }
        // the same icons as everywhere else (the pool frame and the item's sprite; they were the older glyph discs)
        item_icon(e, it.id, cx, cy, 18.0, 1.0);
        let cnt = v[inv_word(p, it.id as usize)];
        if cnt > 1 {
            count_chip(e, cx, cy, 22.0, cnt);
        }
    }
    // the card of the hovered item (the first one while the tab bar has the cursor)
    let it = &ITEMS[list[sel]];
    let cnt = v[inv_word(p, it.id as usize)];
    let cy = y + 4.0 + n.div_ceil(per) as f32 * 25.0;
    let ch = (PYV + ph() - 30.0 - cy).clamp(70.0, 150.0);
    sticker(e, S_FACE_WHITE, x, cy, w, ch, 0.6, WHITE, |e| {
        e.rect(x + 6.0, cy + 6.0, 6.0, ch - 12.0, it.rgba);
        let stars = it.quality.min(4) as f32 * 13.0;
        text_fit(e, x + 20.0, cy + 8.0, td, 17, 11, INK, ALIGN_LEFT, w - 34.0 - stars, it.name);
        // the card in the keyword language (its trigger words lead), then the stack
        rich(e, x + 20.0, cy + 30.0, w - 30.0, ts(12), 14.0, INK, false, envoy_data::keywords::card(it.id), 4);
        // the item language's combos: the synergies and evolutions it is part of
        let mut names = [""; 6];
        let k = procs::partners(it.id, &mut names);
        if k > 0 {
            let mut c = Buf::new();
            c.s("Combos: ");
            for (i, nm) in names[..k].iter().enumerate() {
                if i > 0 {
                    c.s(", ");
                }
                c.s(nm);
            }
            let lines = wrap_lines(c.str(), w - 28.0, tb(10)).min(2);
            wrap(e, x + 20.0, cy + ch - 25.0 - lines as f32 * 11.0, w - 28.0, tb, 10, 11.0, CORAL, ALIGN_LEFT, c.str(), 2);
        }
        let mut s = Buf::new();
        s.s("Stack ").i(cnt).s("/").i(it.cap as i32);
        if cnt >= it.cap as i32 {
            s.s("  (at its cap)");
        }
        e.text_ex(x + 20.0, cy + ch - 20.0, tb(11), LILAC, ALIGN_LEFT, 0.0, s.str());
        for q in 0..it.quality.min(4) {
            spr_c(e, S_STAR_ON, x + w - 14.0 - q as f32 * 13.0, cy + 16.0, 1.0, WHITE);
        }
    });
}

/// Text over at most two lines of width `w` (split at the last space that fits).

fn kind_word(k: i32) -> &'static str {
    match k {
        FK_PACK => "Pack",
        FK_ELITE => "Elite",
        FK_BOSS => "Boss",
        _ => "Duel",
    }
}

fn road_word(k: i32) -> &'static str {
    match k {
        ROAD_STEEP => "Steep road",
        ROAD_SWARM => "Swarm road",
        ROAD_SHIFT => "Shifting road",
        ROAD_OPEN => "Open road",
        _ => "",
    }
}

unsafe fn run_page(e: &mut E, v: &[i32; VW], _p: usize, x: f32, y: f32, petey: bool) {
    let w = pw() - 20.0;
    let mut s = Buf::new();
    s.s("Loop ").i(v[V_LOOP].max(1)).s("  -  ");
    if v[V_ROOM] == 0 {
        s.s("Boss fight");
    } else {
        s.s("Room ").i(v[V_ROOM]).s(" of ").i(v[V_ROOMS]);
    }
    text_fit(e, x, y, td, 15, 10, WHITE, ALIGN_LEFT, w, s.str());
    let mut s2 = Buf::new();
    s2.s(kind_word(v[V_FIGHT_KIND]));
    if v[V_WAVES] > 1 {
        s2.s("  -  wave ").i(v[V_WAVE]).s(" of ").i(v[V_WAVES]);
    }
    let rw = road_word(v[V_ROAD_TAKEN]);
    if !rw.is_empty() {
        s2.s("  -  ").s(rw);
    }
    text_fit(e, x, y + 22.0, tb, 12, 8, PALE, ALIGN_LEFT, w, s2.str());
    // the floor map (the sim's map words); the old room dots only when no map is published
    let my = y + 58.0;
    if crate::map::has(v) {
        // the act map fills the page (rows up to the boss, as in Slay the Spire)
        // the page's whole height (what the panel leaves over the hint line), so the rows stand apart
        let mh = PYV + ph() - 44.0 - (y + 40.0);
        crate::map::draw(e, v, x + 20.0, y + 40.0, w - 40.0, mh, 8.0);
        crate::map::legend(e, v, x, y + 40.0 + mh + 2.0, w);
        return;
    } else {
        let rooms = v[V_ROOMS].clamp(1, 6);
        let here = if v[V_ROOM] == 0 { rooms + 1 } else { v[V_ROOM] };
        let step = (w - 40.0) / rooms as f32;
        for i in 0..=rooms {
            let cx = x + 18.0 + i as f32 * step;
            if i < rooms {
                let mut k = 0;
                while k < 5 {
                    e.rect(cx + 12.0 + k as f32 * (step - 24.0) / 5.0, my - 1.0, (step - 24.0) / 9.0, 3.0, alpha(PALE, 0.7));
                    k += 1;
                }
            }
            let node = i + 1;
            let boss = i == rooms;
            if node == here {
                spr_rot(e, S_BURST_SUNNY, cx, my, if boss { 0.6 } else { 0.5 }, 8.0, WHITE);
            }
            let id = if node < here { S_FACE_INK } else if boss { S_FACE_CORAL } else { S_FACE_WHITE };
            let r = if boss { 14.0 } else { 10.0 };
            face(e, id, cx - r, my - r, r * 2.0, r * 2.0, WHITE);
            if boss {
                spr_c(e, S_STAR_ON, cx, my, 1.2, WHITE);
            } else {
                let mut n = Buf::new();
                n.i(node);
                e.text_ex(cx, my - 7.0, d(11), if node < here { PALE } else { INK }, ALIGN_CENTER, 0.0, n.str());
            }
        }
    }
    // what follows in this session (V_SESS_KINDS: kind + 1 per 4 bits)
    let mut q = Buf::new();
    q.s("Ahead: ");
    let ks = v[V_SESS_KINDS];
    let mut any = false;
    for k in 1..4 {
        let kk = (ks >> (4 * k)) & 15;
        if kk == 0 {
            break;
        }
        if any {
            q.s(", ");
        }
        q.s(kind_word(kk - 1));
        any = true;
    }
    if !any {
        q.s(if v[V_ROOM] == 0 { "the campfire" } else { "a stop: item room / shop" });
    }
    text_fit(e, x, my + 22.0, tb, 12, 8, WHITE, ALIGN_LEFT, w, q.str());
    // the boss card
    let by = my + 46.0;
    sticker(e, S_FACE_CORAL, x, by, w, 60.0, -0.7, WHITE, |e| {
        e.text_ex(x + 12.0, by + 5.0, tb(10), WHITE, ALIGN_LEFT, 0.0, "THE BOSS");
        let name = if petey { "Petey Piranha" } else { "Ganondorf, level 9" };
        text_edge(e, x + 12.0, by + 18.0, td(16), SUNNY, ALIGN_LEFT, 1.2, name);
        let mut b = Buf::new();
        b.s("Bosses beaten this run: ").i(v[V_BOSSES]);
        text_fit(e, x + 12.0, by + 41.0, tb, 10, 8, WHITE, ALIGN_LEFT, w - 24.0, b.str());
    });
}

unsafe fn fight_page(e: &mut E, v: &[i32; VW], x: f32, y: f32) {
    let w = pw() - 20.0;
    let fh = &*core::ptr::addr_of!(crate::hud::FH);
    let kind = v[V_FIGHT_KIND];
    let mut yy = y;
    let mut foes = 0;
    for sl in 0..4 {
        let b = V_S0 + sl * V_SW;
        let foe = v[b + S_FOE] != 0;
        if fh[sl * 3] == 0 && !foe {
            continue;
        }
        let big = foe && (v[b + S_BOSS] != 0 || v[b + S_DRIVES] > 0);
        let alive = v[b + S_ALIVE] != 0;
        let h = 44.0;
        let face_id = if !foe { S_FACE_WHITE } else if big { S_FACE_CORAL } else { S_FACE_LILAC };
        face(e, face_id, x, yy, w, h, if alive { WHITE } else { alpha(WHITE, 0.5) });
        let label = if !foe {
            "You"
        } else if kind == FK_BOSS && big {
            "Boss"
        } else if big {
            "Elite"
        } else {
            "Foe"
        };
        e.text_ex(x + 10.0, yy + 4.0, td(12), if foe { WHITE } else { INK }, ALIGN_LEFT, 0.0, label);
        let mut s = Buf::new();
        s.i(fh[sl * 3 + 1].clamp(0, 999)).s("%");
        e.text_ex(x + 10.0, yy + 19.0, td(17), if foe { WHITE } else { INK }, ALIGN_LEFT, 0.0, s.str());
        let mut st = Buf::new();
        st.i(fh[sl * 3 + 2].max(0)).s(if fh[sl * 3 + 2] == 1 { " stock" } else { " stocks" });
        e.text_ex(x + 70.0, yy + 6.0, tb(10), if foe { WHITE } else { LILAC }, ALIGN_LEFT, 0.0, st.str());
        if v[b + S_HP_MAX] > 0 {
            let f = (v[b + S_HP] as f32 / v[b + S_HP_MAX] as f32).clamp(0.0, 1.0);
            e.rect(x + 70.0, yy + 21.0, w - 82.0, 6.0, TRACK);
            e.rect(x + 70.0, yy + 21.0, (w - 82.0) * f, 6.0, SUNNY);
        }
        // statuses
        let mut words = Buf::new();
        let burn = v[b + S_BURN];
        if burn > 0 {
            words.s("Burning ").i(burn).s("  ");
        }
        if v[b + S_CHILL] > 0 {
            words.s("Chilled  ");
        }
        if v[b + S_MARK] > 0 {
            words.s("Marked  ");
        }
        if v[b + S_HASTE] > 0 {
            words.s("Haste  ");
        }
        if v[b + S_SHIELD] > 0 {
            words.s("Shield  ");
        }
        if !alive && foe {
            words.s("Down");
        }
        if words.str().is_empty() {
            words.s(if foe { "no status" } else { "" });
        }
        text_fit(e, x + 70.0, yy + 29.0, tb, 10, 7, if foe { WHITE } else { LILAC }, ALIGN_LEFT, w - 78.0, words.str().trim_end());
        for k in 0..burn.clamp(0, 4) {
            spr_c(e, S_FLAME + 2, x + w - 10.0 - k as f32 * 9.0, yy + 10.0, 0.35, WHITE);
        }
        if foe {
            foes += 1;
        }
        yy += h + 6.0;
        if yy > y + 140.0 {
            break;
        }
    }
    if foes == 0 {
        e.text_ex(x + w * 0.5, yy + 10.0, tb(12), PALE, ALIGN_CENTER, 0.0, "No foes left standing.");
    }
}

unsafe fn options_page(e: &mut E, pn: &Panel, x: f32, y: f32) {
    let w = pw() - 20.0;
    let fx_txt = ["Off", "25%", "50%", "75%", "Full"];
    let rows: [(&str, &str); OPT_ROWS] = [
        ("Text size", ["Small", "Normal", "Large"][TEXT.min(2)]),
        ("Effects", fx_txt[FX.clamp(0, 4) as usize]),
        ("Screen shake", if SHAKE { "On" } else { "Off" }),
        ("Abandon run", if pn.confirm { "Sure? Right again" } else { "" }),
    ];
    for (i, (label, value)) in rows.iter().enumerate() {
        let on = pn.row == 1 && pn.opt == i;
        let abandon = i == OPT_ROWS - 1;
        let ry = y + i as f32 * 38.0 + if abandon { 10.0 } else { 0.0 };
        let id = if abandon && pn.confirm { S_FACE_CORAL } else if on { S_FACE_HOVER } else { S_FACE_WHITE };
        let dx = if on { 6.0 } else { 0.0 };
        let deg = if on { 0.0 } else { [-0.9, 0.7, -0.6, 0.9][i] };
        let tc = if abandon && pn.confirm { WHITE } else if abandon { DEEP } else { INK };
        sticker(e, id, x + dx, ry, w - 6.0, 32.0, deg, WHITE, |e| {
            e.text_ex(x + dx + 10.0, ry + 8.0, td(13), tc, ALIGN_LEFT, 0.0, label);
            if !value.is_empty() {
                let vx = x + dx + w - 16.0;
                e.text_ex(vx, ry + 9.0, tb(12), if abandon { WHITE } else { LILAC }, ALIGN_RIGHT, 0.0, value);
            }
        });
    }
    let _ = ck_name;
}

fn item_name(id: i32) -> &'static str {
    item(id).map_or("?", |it| it.name)
}

/// The Combos page: every synergy and evolution (lit when the player holds it), the selected one's card, then the
/// keyword glossary of the item language.
unsafe fn combos_page(e: &mut E, v: &[i32; VW], p: usize, pn: &Panel, x: f32, y: f32) {
    let w = pw() - 20.0;
    let bits = v[V_SYN + p];
    let sel = pn.opt.min(COMBO_ROWS - 1);
    for k in 0..COMBOS {
        let (name, rgba, on) = if k < procs::SYNERGIES.len() {
            let sy = &procs::SYNERGIES[k];
            (sy.name, sy.rgba, bits >> k & 1 != 0)
        } else {
            let j = k - procs::SYNERGIES.len();
            (procs::EVOLUTIONS[j].name, procs::EVOLUTIONS[j].rgba, bits >> (16 + j) & 1 != 0)
        };
        let cx = x + (k % 2) as f32 * (w * 0.5);
        let cy = y + (k / 2) as f32 * 15.0;
        if k == sel && pn.row == 1 {
            e.rect(cx - 2.0, cy - 1.0, w * 0.5 - 2.0, 14.0, alpha(SUNNY, 0.35));
        }
        spr_c(e, S_DISC, cx + 5.0, cy + 6.0, 9.0 / 32.0, if on { rgba } else { alpha(PALE, 0.5) });
        let evo = k >= procs::SYNERGIES.len();
        text_fit(e, cx + 13.0, cy, tb, 10, 7, if on { WHITE } else if evo { alpha(SUNNY, 0.6) } else { PALE }, ALIGN_LEFT, w * 0.5 - 18.0, name);
    }
    let cy = y + ((COMBOS + 1) / 2) as f32 * 15.0 + 6.0;
    let (title, recipe_a, recipe_b, line, rgba) = if sel < procs::SYNERGIES.len() {
        let sy = &procs::SYNERGIES[sel];
        (sy.name, item_name(sy.a), item_name(sy.b), sy.line, sy.rgba)
    } else if sel < COMBOS {
        let ev = &procs::EVOLUTIONS[sel - procs::SYNERGIES.len()];
        (ev.name, item_name(ev.base), item_name(ev.partner), ev.line, ev.rgba)
    } else {
        let (kw, gl) = procs::KEYWORDS[sel - COMBOS];
        (kw, "", "", gl, SUNNY)
    };
    // the card grows with its line (two lines at most at this width, three on a narrow panel)
    let ly = if recipe_a.is_empty() { 22.0 } else { 34.0 };
    let lines = wrap_lines(line, w - 26.0, tb(10)).min(3);
    let chh = (ly + lines as f32 * 13.0 + 6.0).max(64.0);
    sticker(e, S_FACE_WHITE, x, cy, w, chh, -0.4, WHITE, |e| {
        e.rect(x + 5.0, cy + 5.0, 5.0, chh - 10.0, rgba);
        text_fit(e, x + 16.0, cy + 4.0, td, 13, 9, INK, ALIGN_LEFT, w - 24.0, title);
        if !recipe_a.is_empty() {
            let mut r = Buf::new();
            r.s(recipe_a);
            if sel >= procs::SYNERGIES.len() {
                r.s(" (at cap)");
            }
            r.s(" + ").s(recipe_b);
            text_fit(e, x + 16.0, cy + 20.0, tb, 10, 7, LILAC, ALIGN_LEFT, w - 24.0, r.str());
        }
        wrap(e, x + 16.0, cy + ly, w - 26.0, tb, 10, 13.0, INK, ALIGN_LEFT, line, 3);
    });
}
