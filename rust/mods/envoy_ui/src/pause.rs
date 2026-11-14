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
use crate::ly::*;
use alloc::vec::Vec;
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

// ------------------------------------------------------------------ drawing (band space; the boxes come from ly.rs)
//
// The panel is one flexbox tree (ly.rs, Taffy), rebuilt every present it is up: a column of the title row, the
// header sticker, the tab bar, the page and the hint line. Every box is as large as its words and its children
// need; the page takes the height left over. Nothing here is a typed coordinate, so a longer word, a larger text
// size (Options) or a narrower panel (two players on a 4:3 screen) moves the boxes instead of breaking them.

/// The panel's width this present (`draw` sets it): 312 for one player; two players get what half the screen
/// leaves each (4:3: 298, a wide window: 312).
static mut PWV: f32 = 312.0;
/// The least the panel is high (it grows with a second tab row or a long page).
const PANEL_MIN_H: f32 = 318.0;

// what the screen draws itself in a box the layout gives it (`P::Own` tags)
const OWN_SLOT: u8 = 1;
const OWN_MAP: u8 = 2;
const OWN_LEGEND: u8 = 3;
const OWN_STARS: u8 = 4;
const OWN_PICK: u8 = 5;
const OWN_COUNT: u8 = 6;
const OWN_METER: u8 = 7;

/// The overlay over the retail pause: a panel per human player.
pub unsafe fn draw(e: &mut E, v: &[i32; VW], ex: f32, boss_petey: bool) {
    let humans = v[V_HUMANS].clamp(1, 2) as usize;
    e.layer(LAYER_MENU, 4);
    PWV = if humans == 2 { ((640.0 + 2.0 * ex - 44.0) * 0.5).clamp(232.0, 312.0) } else { 312.0 };
    for p in 0..humans {
        let x = if p == 0 { 14.0 - ex } else { 626.0 + ex - PWV };
        // the HUD's centred cards keep clear of the panel (kit::clear_span)
        if p == 0 {
            claim_left(x + PWV + 10.0);
        } else {
            claim_right(x - 10.0);
        }
        panel(e, v, p, x, if p == 0 { -0.8 } else { 0.8 }, boss_petey, humans);
    }
}

unsafe fn panel(e: &mut E, v: &[i32; VW], p: usize, x: f32, deg: f32, petey: bool, humans: usize) {
    let pn = PANELS[p];
    let (sc, a) = enter(pn.age);
    let y = 52.0 + (1.0 - sc) * 40.0;
    let inner = PWV - 20.0;
    let mut l = Lay::new(SLOT_PAUSE + p);
    // the title row: ENVOY (+ the player tag), "paused" at the right
    let mut t = Buf::new();
    t.s("ENVOY");
    if humans == 2 {
        t.s(if p == 0 { "  P1" } else { "  P2" });
    }
    let k_title = l.words(t.str(), d, 18, 1, false, St::row().fixed(), P::TextEdge(SUNNY, ALIGN_LEFT, 1.2));
    let k_sp = l.spring();
    let k_paused = l.text("paused", bs, 11, PALE);
    let title = l.node(St::row().align(A::Center), &[k_title, k_sp, k_paused]);
    let head = header(&mut l, v, p);
    let tab_bar = tabs(&mut l, &pn, inner);
    // the player's colour under the tab bar
    let rule = l.leaf(St::row().h(2.0).fixed(), P::Fill(alpha(PCOL[p], 0.9), 0.0));
    let page = match pn.page {
        PG_ITEMS => items_page(&mut l, v, p, &pn),
        PG_RUN => run_page(&mut l, v, petey),
        PG_COMBOS => combos_page(&mut l, v, p, &pn),
        PG_FIGHT => fight_page(&mut l, v),
        PG_WORDS => words_page(&mut l, &pn),
        _ => options_page(&mut l, &pn),
    };
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
    let k_hint = l.words(hint, bs, 11, 1, false, St::row().min_w(0.0).fixed(), P::Text(if pn.abandoned > 0 { CORAL } else { PALE }, ALIGN_CENTER));
    let root = l.node(St::col().w(PWV).min_h(PANEL_MIN_H).pad4(10.0, 8.0, 10.0, 8.0).gap(7.0), &[title, head, tab_bar, rule, page, k_hint]);
    l.compute(root, x, y, None, None);
    let r = l.rect(root);
    panel_shadow(e, r.x, r.y, r.w, r.h, a);
    // the ink sticker twice: the face is see-through at its middle, and the panel must read over any stage
    sticker(e, S_FACE_INK, r.x, r.y, r.w, r.h, deg, alpha(WHITE, a), |e| {
        face(e, S_FACE_INK, r.x, r.y, r.w, r.h, alpha(WHITE, a));
        l.draw(e, root, a, &mut |e, tag, data, r| own(e, v, tag, data, r, a));
    });
}

/// The boxes the layout leaves to the screen.
unsafe fn own(e: &mut E, v: &[i32; VW], tag: u8, data: i32, r: R, a: f32) {
    match tag {
        // on the white sticker the held stocks are ink discs (the HUD's hearts would be lost), the empty slots rings
        OWN_SLOT => crate::icons::slot_ink(e, r.cx(), r.cy(), r.w, data & 1 != 0, data & 2 != 0, a),
        OWN_MAP => crate::map::draw(e, v, r.x + 10.0, r.y + 4.0, r.w - 20.0, r.h - 6.0, 8.0),
        OWN_LEGEND => crate::map::legend(e, v, r.x, r.y, r.w),
        OWN_STARS => {
            for q in 0..data {
                spr_c(e, S_STAR_ON, r.right() - 6.5 - q as f32 * 13.0, r.cy(), 1.0, alpha(WHITE, a));
            }
        }
        // the hovered item: a sunny burst behind its icon
        OWN_PICK => {
            if data != 0 {
                spr_c(e, S_BURST_SUNNY, r.cx(), r.cy(), 0.56, alpha(WHITE, a));
            }
        }
        OWN_COUNT => count_chip(e, r.x, r.y, 22.0, data),
        OWN_METER => {
            e.round_rect(r.x, r.y, r.w, r.h, 3.0, alpha(TRACK, a));
            e.round_rect(r.x, r.y, r.w * (data.clamp(0, 1000) as f32 / 1000.0), r.h, 3.0, alpha(SUNNY, a));
        }
        _ => {}
    }
}

/// Stocks against the limit, and coins: one white sticker; the limit and the purse at its right end.
unsafe fn header(l: &mut Lay, v: &[i32; VW], p: usize) -> N {
    let (pool, lim) = crate::hud::stocks_of(v, p);
    let lim = if lim > 0 { lim } else { pool.max(4) };
    let n = lim.max(pool).clamp(1, 8);
    let mut dots = Vec::new();
    for i in 0..n {
        dots.push(l.leaf(St::boxed(13.0, 13.0), P::Own(OWN_SLOT, (i < pool) as i32 | ((pool == 1) as i32) << 1)));
    }
    let dot_row = l.node(St::row().gap(4.0), &dots);
    let k_s = l.text("STOCKS", bs, 10, LILAC);
    let left = l.node(St::col().gap(3.0), &[k_s, dot_row]);
    let mut s = Buf::new();
    s.i(pool).s("/").i(lim);
    let k_l = l.text("LIMIT", bs, 10, LILAC);
    let k_v = l.text(s.str(), d, 14, INK);
    let mid = l.node(St::col(), &[k_l, k_v]);
    let mut c = Buf::new();
    c.i(v[V_COINS]);
    let coin = l.leaf(St::boxed(16.0, 16.0), P::Spr(S_COIN));
    let k_c = l.text(c.str(), d, 13, INK);
    let purse = l.node(St::row().gap(4.0).align(A::Center), &[coin, k_c]);
    let sp = l.spring();
    l.boxed(St::row().pad4(9.0, 4.0, 10.0, 6.0).gap(16.0).align(A::End).fixed(), P::Face(S_FACE_WHITE), &[left, sp, mid, purse])
}

/// The tab bar: every tab as wide as its word plus its share of the room left; two rows of three where one row
/// cannot hold the six words (two panels on a 4:3 screen).
unsafe fn tabs(l: &mut Lay, pn: &Panel, inner: f32) -> N {
    let mut row = |l: &mut Lay, from: usize, to: usize| {
        let mut ks = Vec::new();
        for (i, name) in PAGES.iter().enumerate().take(to).skip(from) {
            let on = i == pn.page;
            let id = if on && pn.row == 0 { S_FACE_HOVER } else if on { S_FACE_SUNNY } else { S_FACE_LILAC };
            let k = l.words(name, d, 10, 1, false, St::row().fixed(), P::Text(if on { INK } else { WHITE }, ALIGN_CENTER));
            ks.push(l.boxed(St::row().h(22.0).pad4(7.0, 0.0, 7.0, 0.0).grow(1.0).justify(A::Center).align(A::Center), P::Face(id), &[k]));
        }
        l.node(St::row().gap(6.0).fixed(), &ks)
    };
    if tab_rows(inner, 10, &PAGES) == 1 {
        row(l, 0, PAGES.len())
    } else {
        let (a, b) = (row(l, 0, 3), row(l, 3, PAGES.len()));
        l.node(St::col().gap(7.0).fixed(), &[a, b])
    }
}

unsafe fn items_page(l: &mut Lay, v: &[i32; VW], p: usize, pn: &Panel) -> N {
    let (list, n) = held_items(v, p);
    if n == 0 {
        let a = l.words("No items yet.", bs, ts(13), 1, false, St::row().fixed(), P::Text(PALE, ALIGN_CENTER));
        let b = l.words("The item room, the shop and the boss hand them out.", bs, ts(11), 3, false, St::row().min_w(0.0), P::Text(PALE, ALIGN_CENTER));
        return l.node(St::col().grow(1.0).gap(8.0).pad4(10.0, 30.0, 10.0, 0.0), &[a, b]);
    }
    let sel = pn.item.min(n - 1);
    // the icon strip: a wrapping row of 24-unit cells; the hovered one has a burst behind it, a stack its count
    let mut cells = Vec::new();
    for k in 0..n {
        let it = &ITEMS[list[k]];
        let cnt = v[inv_word(p, it.id as usize)];
        let icon = l.leaf(St::boxed(24.0, 24.0), P::Item(it.id));
        let mut kids = Vec::new();
        kids.push(icon);
        if cnt > 1 {
            kids.push(l.leaf(St::boxed(0.0, 0.0).at(12.0, 12.0), P::Own(OWN_COUNT, cnt)));
        }
        cells.push(l.boxed(St::row().fixed(), P::Own(OWN_PICK, (k == sel && pn.row == 1) as i32), &kids));
    }
    let strip = l.node(St::row().wrap().gap(2.0).fixed(), &cells);
    // the card of the hovered item (the first one while the tab bar has the cursor)
    let it = &ITEMS[list[sel]];
    let cnt = v[inv_word(p, it.id as usize)];
    let name = l.words(it.name, d, ts(17), 1, false, St::row().min_w(0.0).grow(1.0), P::Text(INK, ALIGN_LEFT));
    let q = it.quality.min(4) as i32;
    let stars = l.leaf(St::boxed(q as f32 * 13.0, 13.0), P::Own(OWN_STARS, q));
    let top = l.node(St::row().gap(6.0).align(A::Center), &[name, stars]);
    // the card in the keyword language (its trigger words lead), then the combos it is part of and the stack
    let body = l.rich(envoy_data::keywords::card(it.id), ts(12), INK, 6);
    let sp = l.spring();
    let mut kids = Vec::new();
    kids.extend_from_slice(&[top, body, sp]);
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
        kids.push(l.para(c.str(), bs, ts(10), CORAL, 2));
    }
    let mut s = Buf::new();
    s.s("Stack ").i(cnt).s("/").i(it.cap as i32);
    if cnt >= it.cap as i32 {
        s.s("  (at its cap)");
    }
    kids.push(l.text_soft(s.str(), bs, ts(11), LILAC));
    let bar = l.leaf(St::row().w(6.0).fixed(), P::Fill(it.rgba, 3.0));
    let words = l.node(St::col().grow(1.0).min_w(0.0).gap(4.0), &kids);
    let card = l.boxed(St::row().grow(1.0).pad4(7.0, 7.0, 9.0, 7.0).gap(8.0).min_h(110.0), P::Face(S_FACE_WHITE), &[bar, words]);
    l.node(St::col().grow(1.0).gap(8.0), &[strip, card])
}

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

unsafe fn run_page(l: &mut Lay, v: &[i32; VW], petey: bool) -> N {
    let mut s = Buf::new();
    s.s("Loop ").i(v[V_LOOP].max(1)).s("  -  ");
    if v[V_ROOM] == 0 {
        s.s("Boss fight");
    } else {
        s.s("Room ").i(v[V_ROOM]).s(" of ").i(v[V_ROOMS]);
    }
    let k1 = l.text_soft(s.str(), d, ts(15), WHITE);
    let mut s2 = Buf::new();
    s2.s(kind_word(v[V_FIGHT_KIND]));
    if v[V_WAVES] > 1 {
        s2.s("  -  wave ").i(v[V_WAVE]).s(" of ").i(v[V_WAVES]);
    }
    let rw = road_word(v[V_ROAD_TAKEN]);
    if !rw.is_empty() {
        s2.s("  -  ").s(rw);
    }
    let k2 = l.text_soft(s2.str(), bs, ts(12), PALE);
    if crate::map::has(v) {
        // the act map takes the page's height (rows up to the boss, as in Slay the Spire), its legend under it
        let map = l.leaf(St::row().grow(1.0).min_h(150.0), P::Own(OWN_MAP, 0));
        let legend = l.leaf(St::row().h(14.0).fixed(), P::Own(OWN_LEGEND, 0));
        return l.node(St::col().grow(1.0).gap(3.0), &[k1, k2, map, legend]);
    }
    // no map published: what follows in this session (V_SESS_KINDS: kind + 1 per 4 bits), and the boss
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
    let k3 = l.para(q.str(), bs, ts(12), WHITE, 2);
    let b1 = l.text("THE BOSS", bs, ts(10), WHITE);
    let b2 = l.words(if petey { "Petey Piranha" } else { "Ganondorf, level 9" }, d, ts(16), 1, false, St::row().min_w(0.0), P::TextEdge(SUNNY, ALIGN_LEFT, 1.2));
    let mut b = Buf::new();
    b.s("Bosses beaten this run: ").i(v[V_BOSSES]);
    let b3 = l.text_soft(b.str(), bs, ts(10), WHITE);
    let boss = l.boxed(St::col().pad4(12.0, 6.0, 12.0, 8.0).gap(2.0), P::Face(S_FACE_CORAL), &[b1, b2, b3]);
    let sp = l.spring();
    l.node(St::col().grow(1.0).gap(6.0), &[k1, k2, k3, sp, boss])
}

unsafe fn fight_page(l: &mut Lay, v: &[i32; VW]) -> N {
    let fh = &*core::ptr::addr_of!(crate::hud::FH);
    let kind = v[V_FIGHT_KIND];
    let mut rows = Vec::new();
    let mut foes = 0;
    for sl in 0..4 {
        let b = V_S0 + sl * V_SW;
        let foe = v[b + S_FOE] != 0;
        if fh[sl * 3] == 0 && !foe {
            continue;
        }
        let big = foe && (v[b + S_BOSS] != 0 || v[b + S_DRIVES] > 0);
        let alive = v[b + S_ALIVE] != 0;
        let face_id = if !alive { S_FACE_FADED } else if !foe { S_FACE_WHITE } else if big { S_FACE_CORAL } else { S_FACE_LILAC };
        let ink = if foe && alive { WHITE } else { INK };
        let sub = if foe && alive { WHITE } else { LILAC };
        let label = if !foe {
            "You"
        } else if kind == FK_BOSS && big {
            "Boss"
        } else if big {
            "Elite"
        } else {
            "Foe"
        };
        let k_label = l.text(label, d, ts(12), ink);
        let mut s = Buf::new();
        s.i(fh[sl * 3 + 1].clamp(0, 999)).s("%");
        let k_pct = l.text(s.str(), d, ts(17), ink);
        let left = l.node(St::col().min_w(52.0).fixed(), &[k_label, k_pct]);
        let mut st = Buf::new();
        st.i(fh[sl * 3 + 2].max(0)).s(if fh[sl * 3 + 2] == 1 { " stock" } else { " stocks" });
        let mut right = Vec::new();
        right.push(l.text_soft(st.str(), bs, ts(10), sub));
        if v[b + S_HP_MAX] > 0 {
            let f = (v[b + S_HP] as f32 / v[b + S_HP_MAX] as f32).clamp(0.0, 1.0);
            right.push(l.leaf(St::row().h(6.0).fixed(), P::Own(OWN_METER, (f * 1000.0) as i32)));
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
        if words.str().is_empty() && foe {
            words.s("no status");
        }
        if !words.str().is_empty() {
            right.push(l.text_soft(words.str().trim_end(), bs, ts(10), sub));
        }
        let rcol = l.node(St::col().grow(1.0).min_w(0.0).gap(3.0), &right);
        rows.push(l.boxed(St::row().pad4(10.0, 4.0, 10.0, 6.0).gap(12.0).align(A::Center).fixed(), P::Face(face_id), &[left, rcol]));
        if foe {
            foes += 1;
        }
        if rows.len() >= 4 {
            break;
        }
    }
    if foes == 0 {
        rows.push(l.words("No foes left standing.", bs, ts(12), 1, false, St::row().min_w(0.0), P::Text(PALE, ALIGN_CENTER)));
    }
    l.node(St::col().grow(1.0).gap(8.0), &rows)
}

unsafe fn options_page(l: &mut Lay, pn: &Panel) -> N {
    let fx_txt = ["Off", "25%", "50%", "75%", "Full"];
    let rows: [(&str, &str); OPT_ROWS] = [
        ("Text size", ["Small", "Normal", "Large"][TEXT.min(2)]),
        ("Effects", fx_txt[FX.clamp(0, 4) as usize]),
        ("Screen shake", if SHAKE { "On" } else { "Off" }),
        ("Abandon run", if pn.confirm { "Sure? Right again" } else { "" }),
    ];
    let mut ks = Vec::new();
    for (i, (label, value)) in rows.iter().enumerate() {
        let on = pn.row == 1 && pn.opt == i;
        let abandon = i == OPT_ROWS - 1;
        let id = if abandon && pn.confirm { S_FACE_CORAL } else if on { S_FACE_HOVER } else { S_FACE_WHITE };
        let tc = if abandon && pn.confirm { WHITE } else if abandon { DEEP } else { INK };
        let k_l = l.text(label, d, ts(13), tc);
        let sp = l.spring();
        let k_v = l.text_soft(value, bs, ts(12), if abandon { WHITE } else { LILAC });
        if abandon {
            // Abandon stands apart from the settings
            ks.push(l.leaf(St::row().h(6.0).fixed(), P::None));
        }
        ks.push(l.boxed(St::row().min_h(32.0).pad4(10.0, 4.0, 12.0, 4.0).gap(8.0).align(A::Center).fixed(), P::Face(id), &[k_l, sp, k_v]));
    }
    l.node(St::col().grow(1.0).gap(8.0), &ks)
}

fn item_name(id: i32) -> &'static str {
    item(id).map_or("?", |it| it.name)
}

/// The glossary: every keyword of the item language, in its colour, and what it means.
unsafe fn words_page(l: &mut Lay, pn: &Panel) -> N {
    use envoy_data::keywords::*;
    let top = if pn.page == PG_WORDS { pn.item.min(KEYWORDS.len() - WORDS_ROWS) } else { 0 };
    let mut ks = Vec::new();
    for kw in KEYWORDS.iter().skip(top).take(WORDS_ROWS) {
        let col = KC_DARK[kw.cat as usize];
        let bar = l.leaf(St::row().w(3.0).fixed(), P::Fill(col, 1.5));
        let k_w = l.text(kw.word, d, ts(12), col);
        let sp = l.spring();
        let k_c = l.text(KC_NAME[kw.cat as usize], bs, ts(9), PALE);
        let head = l.node(St::row().align(A::Center), &[k_w, sp, k_c]);
        let k_d = l.para(kw.def, bs, ts(10), WHITE, 2);
        let words = l.node(St::col().grow(1.0).min_w(0.0), &[head, k_d]);
        ks.push(l.node(St::row().gap(6.0).fixed(), &[bar, words]));
    }
    let mut b = Buf::new();
    b.i(top as i32 + 1).s("-").i((top + WORDS_ROWS) as i32).s(" of ").i(KEYWORDS.len() as i32);
    ks.push(l.spring());
    ks.push(l.words(b.str(), bs, ts(10), 1, false, St::row().min_w(0.0).fixed(), P::Text(PALE, ALIGN_RIGHT)));
    l.node(St::col().grow(1.0).gap(4.0), &ks)
}

/// The Combos page: every synergy and evolution (lit when the player holds it), the selected one's card, then the
/// keyword glossary of the item language.
unsafe fn combos_page(l: &mut Lay, v: &[i32; VW], p: usize, pn: &Panel) -> N {
    let bits = v[V_SYN + p];
    let sel = pn.opt.min(COMBO_ROWS - 1);
    // two columns: each entry half the page wide
    let mut ks = Vec::new();
    for k in 0..COMBOS {
        let (name, rgba, on) = if k < procs::SYNERGIES.len() {
            let sy = &procs::SYNERGIES[k];
            (sy.name, sy.rgba, bits >> k & 1 != 0)
        } else {
            let j = k - procs::SYNERGIES.len();
            (procs::EVOLUTIONS[j].name, procs::EVOLUTIONS[j].rgba, bits >> (16 + j) & 1 != 0)
        };
        let evo = k >= procs::SYNERGIES.len();
        let dot = l.leaf(St::boxed(9.0, 9.0), P::Disc(if on { rgba } else { alpha(PALE, 0.5) }));
        let col = if on { WHITE } else if evo { alpha(SUNNY, 0.6) } else { PALE };
        let k_n = l.words(name, bs, ts(10), 1, false, St::row().min_w(0.0).grow(1.0), P::Text(col, ALIGN_LEFT));
        let hover = if k == sel && pn.row == 1 { P::Fill(alpha(SUNNY, 0.35), 3.0) } else { P::None };
        ks.push(l.boxed(St::row().w_pct(0.5).pad4(2.0, 1.0, 4.0, 1.0).gap(4.0).align(A::Center), hover, &[dot, k_n]));
    }
    let grid = l.node(St::row().wrap().fixed(), &ks);
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
    // the selected entry's card: as high as its line needs
    let mut cs = Vec::new();
    cs.push(l.text_soft(title, d, ts(13), INK));
    if !recipe_a.is_empty() {
        let mut r = Buf::new();
        r.s(recipe_a);
        if sel >= procs::SYNERGIES.len() {
            r.s(" (at cap)");
        }
        r.s(" + ").s(recipe_b);
        cs.push(l.text_soft(r.str(), bs, ts(10), LILAC));
    }
    cs.push(l.para(line, bs, ts(10), INK, 4));
    let bar = l.leaf(St::row().w(5.0).fixed(), P::Fill(rgba, 2.5));
    let words = l.node(St::col().grow(1.0).min_w(0.0).gap(2.0), &cs);
    let card = l.boxed(St::row().pad4(6.0, 6.0, 9.0, 7.0).gap(7.0).min_h(60.0).fixed(), P::Face(S_FACE_WHITE), &[bar, words]);
    l.node(St::col().grow(1.0).gap(9.0), &[grid, card])
}
