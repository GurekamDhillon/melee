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
use envoy_data::view::*;
use gd_ui_enc::*;

#[link(wasm_import_module = "gd_ui")]
extern "C" {
    fn gd_ui_strip(port: u32, bits: u32) -> i32;
    fn gd_ui_sim_signal(port: u32, bits: u32, frames: u32) -> i32;
}

const DPAD: u32 = 0x000F;
const ABANDON_BIT: u32 = 0x2000;
const PAGES: [&str; 4] = ["Items", "Run", "Fight", "Options"];
const PG_ITEMS: usize = 0;
const PG_RUN: usize = 1;
const PG_FIGHT: usize = 2;
const PG_OPTIONS: usize = 3;
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
        if v[V_INV + p * ITEM_IDS + it.id as usize] > 0 {
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
        if left {
            pn.page = (pn.page + PAGES.len() - 1) % PAGES.len();
            log("page ", PAGES[pn.page]);
        } else if right {
            pn.page = (pn.page + 1) % PAGES.len();
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
        _ => {
            if up {
                pn.row = 0;
            }
        }
    }
}

// ------------------------------------------------------------------ drawing (band space, 640 x 480)

const PW: f32 = 232.0;
const PY: f32 = 52.0;
const PH: f32 = 318.0;

/// The overlay over the retail pause: a panel per human player.
pub unsafe fn draw(e: &mut E, v: &[i32; VW], ex: f32, boss_petey: bool) {
    let humans = v[V_HUMANS].clamp(1, 2) as usize;
    e.layer(LAYER_MENU, 4);
    for p in 0..humans {
        let x = if p == 0 { 14.0 - ex } else { 626.0 + ex - PW };
        panel(e, v, p, x, if p == 0 { -0.8 } else { 0.8 }, boss_petey, humans);
    }
}

unsafe fn panel(e: &mut E, v: &[i32; VW], p: usize, x: f32, deg: f32, petey: bool, humans: usize) {
    let pn = PANELS[p];
    let (sc, a) = enter(pn.age);
    let y = PY + (1.0 - sc) * 40.0;
    let col = PCOL[p];
    // the ink sticker twice: the baked face is see-through at its middle, and the panel must read over any stage
    sticker(e, S_FACE_INK, x, y, PW, PH, deg, alpha(WHITE, a), |e| {
        face(e, S_FACE_INK, x, y, PW, PH, alpha(WHITE, a));
        // title strip: ENVOY + the player tag
        let mut t = Buf::new();
        t.s("ENVOY");
        if humans == 2 {
            t.s(if p == 0 { "  P1" } else { "  P2" });
        }
        text_edge(e, x + 12.0, y + 8.0, d(18), SUNNY, ALIGN_LEFT, 1.2, t.str());
        e.text_ex(x + PW - 12.0, y + 13.0, bs(11), PALE, ALIGN_RIGHT, 0.0, "paused");
        header(e, v, p, x + 10.0, y + 36.0);
        tabs(e, &pn, x + 8.0, y + 84.0, col);
        let cy = y + 116.0;
        match pn.page {
            PG_ITEMS => items_page(e, v, p, &pn, x + 10.0, cy),
            PG_RUN => run_page(e, v, p, x + 10.0, cy, petey),
            PG_FIGHT => fight_page(e, v, x + 10.0, cy),
            _ => options_page(e, &pn, x + 10.0, cy),
        }
        // the hint line at the panel's foot
        let hint = if pn.abandoned > 0 {
            "Run abandoned. Ending the match..."
        } else if pn.row == 0 {
            "D-pad: left / right page, down to open"
        } else if pn.page == PG_OPTIONS {
            "D-pad: up / down row, left / right set"
        } else if pn.page == PG_ITEMS {
            "D-pad: left / right item, up to the tabs"
        } else {
            "D-pad up: back to the tabs"
        };
        e.text_ex(x + PW * 0.5, y + PH - 22.0, bs(11), if pn.abandoned > 0 { CORAL } else { PALE }, ALIGN_CENTER, PW - 16.0, hint);
    });
}

/// Stocks against the limit, and coins.
unsafe fn header(e: &mut E, v: &[i32; VW], p: usize, x: f32, y: f32) {
    let (pool, lim) = crate::hud::stocks_of(v, p);
    let lim = if lim > 0 { lim } else { pool.max(4) };
    face(e, S_FACE_WHITE, x, y, PW - 20.0, 40.0, WHITE);
    e.text_ex(x + 8.0, y + 3.0, bs(10), LILAC, ALIGN_LEFT, 0.0, "STOCKS");
    let n = lim.max(pool).clamp(1, 8);
    for i in 0..n {
        // on the white sticker the held stocks are ink discs (the HUD's white icons would vanish), the empty slots rings
        let cx = x + 16.0 + i as f32 * 17.0;
        crate::icons::slot_ink(e, cx, y + 26.0, 13.0, i < pool, pool == 1, 1.0);
    }
    let mut s = Buf::new();
    s.i(pool).s("/").i(lim);
    e.text_ex(x + 136.0, y + 3.0, bs(10), LILAC, ALIGN_LEFT, 0.0, "LIMIT");
    e.text_ex(x + 136.0, y + 16.0, d(14), INK, ALIGN_LEFT, 0.0, s.str());
    spr_c(e, S_COIN, x + 180.0, y + 26.0, 1.0, WHITE);
    let mut c = Buf::new();
    c.i(v[V_COINS]);
    e.text_ex(x + 190.0, y + 18.0, d(13), INK, ALIGN_LEFT, 0.0, c.str());
}

unsafe fn tabs(e: &mut E, pn: &Panel, x: f32, y: f32, col: u32) {
    let w = (PW - 16.0) / 4.0;
    for (i, name) in PAGES.iter().enumerate() {
        let on = i == pn.page;
        let tx = x + i as f32 * w;
        let id = if on && pn.row == 0 { S_FACE_HOVER } else if on { S_FACE_SUNNY } else { S_FACE_LILAC };
        let ty = if on { y - 2.0 } else { y };
        face(e, id, tx + 1.0, ty, w - 4.0, 24.0, WHITE);
        e.text_ex(tx + w * 0.5 - 1.0, ty + 5.0, d(11), if on { INK } else { WHITE }, ALIGN_CENTER, w - 6.0, name);
    }
    // the player's colour under the tab bar
    e.rect(x + 2.0, y + 26.0, PW - 20.0, 2.0, alpha(col, 0.9));
}

fn trigger_word(it: &Item) -> &'static str {
    TRIGGER_TEXT[(it.trigger as usize).min(TRIGGER_TEXT.len() - 1)]
}

unsafe fn items_page(e: &mut E, v: &[i32; VW], p: usize, pn: &Panel, x: f32, y: f32) {
    let (list, n) = held_items(v, p);
    let w = PW - 20.0;
    if n == 0 {
        e.text_ex(x + w * 0.5, y + 40.0, tb(13), PALE, ALIGN_CENTER, w, "No items yet.");
        e.text_ex(x + w * 0.5, y + 62.0, tb(11), PALE, ALIGN_CENTER, w, "The item room, the shop");
        e.text_ex(x + w * 0.5, y + 76.0, tb(11), PALE, ALIGN_CENTER, w, "and the boss hand them out.");
        return;
    }
    let sel = pn.item.min(n - 1);
    // the icon strip: 7 a row
    for k in 0..n {
        let it = &ITEMS[list[k]];
        let cx = x + 14.0 + (k % 7) as f32 * 30.0;
        let cy = y + 14.0 + (k / 7) as f32 * 30.0;
        let on = k == sel && pn.row == 1;
        if on {
            spr_rot(e, S_BURST_SUNNY, cx, cy, 0.52, 8.0, WHITE);
        }
        crate::icons::icon(e, it.id, cx, cy, 24.0);
        let cnt = v[V_INV + p * ITEM_IDS + it.id as usize];
        if cnt > 1 {
            let mut s = Buf::new();
            s.s("x").i(cnt);
            face(e, S_FACE_INK, cx + 3.0, cy + 3.0, 16.0, 11.0, WHITE);
            e.text_ex(cx + 11.0, cy + 3.0, d(8), SUNNY, ALIGN_CENTER, 0.0, s.str());
        }
    }
    // the card of the hovered item (the first one while the tab bar has the cursor)
    let it = &ITEMS[list[sel]];
    let cnt = v[V_INV + p * ITEM_IDS + it.id as usize];
    let cy = y + 16.0 + ((n + 6) / 7) as f32 * 30.0;
    let ch = 128.0;
    sticker(e, S_FACE_WHITE, x, cy, w, ch, 0.6, WHITE, |e| {
        e.rect(x + 6.0, cy + 6.0, 6.0, ch - 12.0, it.rgba);
        e.text_ex(x + 20.0, cy + 8.0, td(17), INK, ALIGN_LEFT, w - 28.0, it.name);
        let mut t = Buf::new();
        t.s("Trigger: ").s(trigger_word(it));
        face(e, S_FACE_INK, x + 20.0, cy + 34.0, tw(t.str(), tb(11)) + 14.0, 18.0, WHITE);
        e.text_ex(x + 27.0, cy + 36.0, tb(11), SUNNY, ALIGN_LEFT, 0.0, t.str());
        line_wrap(e, x + 20.0, cy + 58.0, tb(12), INK, w - 30.0, it.line);
        let mut s = Buf::new();
        s.s("Stack ").i(cnt).s(" / ").i(it.cap as i32);
        e.text_ex(x + 20.0, cy + ch - 40.0, td(13), INK, ALIGN_LEFT, 0.0, s.str());
        let mut r = Buf::new();
        if cnt >= it.cap as i32 {
            r.s("At its cap: more copies add nothing");
        } else {
            r.s("Each copy: ").s(STACK_TEXT[(it.stack as usize).min(4)]);
        }
        e.text_ex(x + 20.0, cy + ch - 21.0, tb(11), LILAC, ALIGN_LEFT, w - 28.0, r.str());
        for q in 0..it.quality.min(4) {
            spr_c(e, S_STAR_ON, x + w - 14.0 - q as f32 * 13.0, cy + 16.0, 1.0, WHITE);
        }
    });
}

/// Text over at most two lines of width `w` (split at the last space that fits).
fn line_wrap(e: &mut E, x: f32, y: f32, role: i32, rgba: u32, w: f32, s: &str) {
    if tw(s, role) <= w {
        e.text_ex(x, y, role, rgba, ALIGN_LEFT, 0.0, s);
        return;
    }
    let mut cut = 0;
    for (i, c) in s.char_indices() {
        if c == ' ' && tw(&s[..i], role) <= w {
            cut = i;
        }
    }
    if cut == 0 {
        e.text_ex(x, y, role, rgba, ALIGN_LEFT, w, s);
        return;
    }
    e.text_ex(x, y, role, rgba, ALIGN_LEFT, 0.0, &s[..cut]);
    e.text_ex(x, y + 16.0, role, rgba, ALIGN_LEFT, w, &s[cut + 1..]);
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

unsafe fn run_page(e: &mut E, v: &[i32; VW], _p: usize, x: f32, y: f32, petey: bool) {
    let w = PW - 20.0;
    let mut s = Buf::new();
    s.s("Loop ").i(v[V_LOOP].max(1)).s("  -  ");
    if v[V_ROOM] == 0 {
        s.s("Boss fight");
    } else {
        s.s("Room ").i(v[V_ROOM]).s(" of ").i(v[V_ROOMS]);
    }
    e.text_ex(x, y, td(15), WHITE, ALIGN_LEFT, w, s.str());
    let mut s2 = Buf::new();
    s2.s(kind_word(v[V_FIGHT_KIND]));
    if v[V_WAVES] > 1 {
        s2.s("  -  wave ").i(v[V_WAVE]).s(" of ").i(v[V_WAVES]);
    }
    let rw = road_word(v[V_ROAD_TAKEN]);
    if !rw.is_empty() {
        s2.s("  -  ").s(rw);
    }
    e.text_ex(x, y + 22.0, tb(12), PALE, ALIGN_LEFT, w, s2.str());
    // the floor map (the sim's map words); the old room dots only when no map is published
    let my = y + 58.0;
    if crate::map::has(v) {
        face(e, S_FACE_INK, x - 2.0, my - 24.0, w + 4.0, 50.0, WHITE);
        crate::map::draw(e, v, x, my - 24.0, w, 50.0, 8.5);
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
    e.text_ex(x, my + 22.0, tb(12), WHITE, ALIGN_LEFT, w, q.str());
    // the boss card
    let by = my + 46.0;
    sticker(e, S_FACE_CORAL, x, by, w, 60.0, -0.7, WHITE, |e| {
        e.text_ex(x + 12.0, by + 5.0, tb(10), WHITE, ALIGN_LEFT, 0.0, "THE BOSS");
        let name = if petey { "Petey Piranha" } else { "Ganondorf, level 9" };
        text_edge(e, x + 12.0, by + 18.0, td(16), SUNNY, ALIGN_LEFT, 1.2, name);
        let mut b = Buf::new();
        b.s("Bosses beaten this run: ").i(v[V_BOSSES]);
        e.text_ex(x + 12.0, by + 41.0, tb(10), WHITE, ALIGN_LEFT, w - 20.0, b.str());
    });
}

unsafe fn fight_page(e: &mut E, v: &[i32; VW], x: f32, y: f32) {
    let w = PW - 20.0;
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
            e.rect(x + 70.0, yy + 21.0, 120.0, 6.0, TRACK);
            e.rect(x + 70.0, yy + 21.0, 120.0 * f, 6.0, SUNNY);
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
        e.text_ex(x + 70.0, yy + 29.0, tb(10), if foe { WHITE } else { LILAC }, ALIGN_LEFT, w - 76.0, words.str());
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
        e.text_ex(x + w * 0.5, yy + 10.0, tb(12), PALE, ALIGN_CENTER, w, "No foes left standing.");
    }
}

unsafe fn options_page(e: &mut E, pn: &Panel, x: f32, y: f32) {
    let w = PW - 20.0;
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
