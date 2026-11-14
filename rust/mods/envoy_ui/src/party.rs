//! Envoy V3 screens in the Party pack look (envoy-v3-ui.md; menu/out_envoy/UI-SPEC.md). Band space: 640 x 480.

use crate::kit::*;
use crate::{ck_name, Buf, VW};
use envoy_data::view::*;
use envoy_data::*;
use gd_ui_enc::*;

/// V2 families (RED, GREEN, BLUE, YELLOW, PURPLE, WHITE) -> the spec's eight badge families (fire, bolt, frost, wind,
/// tech, util, clay, rose).
/// Words by badge row (the baked rows: fire, bolt (unused), frost, wind, tech, util, stone, rose).
pub const FAM_WORD: [&str; 8] = ["Fire", "Spark", "Frost", "Sky", "Tempo", "Utility", "Guard", "Rose"];
/// The rules' family (F_RED 0, F_GREEN 1, F_BLUE 2, F_YELLOW 3, F_PURPLE 4, F_WHITE 5) -> badge row. One mapping
/// everywhere (sets are by family): RED Fire (tangerine flame), GREEN Tempo (orchid bullseye), BLUE Guard (clay
/// shield), YELLOW Sky (lime gust), PURPLE Frost (sky-blue snowflake), WHITE Utility (lilac-grey rings).
pub const FAM_BADGE: [usize; 6] = [0, 4, 6, 3, 2, 5];
/// The family's word for a pool index.
pub fn fam_word(id: usize) -> &'static str {
    FAM_WORD[FAM_BADGE.get(POOL[id].family as usize).copied().unwrap_or(5)]
}

/// The badge sprite for a pool index.
pub fn badge_of(id: usize) -> usize {
    let d = &POOL[id];
    let f = FAM_BADGE.get(d.family as usize).copied().unwrap_or(5);
    S_BADGE + f * 3 + (d.tier as usize).min(2)
}

/// The keystone emblem sprite for a pool index (by name; a generic one otherwise).
pub fn kem_of(id: usize) -> usize {
    match POOL[id].name {
        "Plague Bearer" => S_KEM,
        "Echo Oath" => S_KEM + 1,
        "Grand Finale" => S_KEM + 2,
        "Overclock" => S_KEM + 3,
        "Twin Soul" => S_KEM + 1,
        "Winter Oath" => S_KEM + 4,
        "Glass Core" => S_KEM + 5,
        _ => S_KEM + 2,
    }
}

/// A keystone pill (25 high, ink, emblem 21 + name 12.5 sunny).
pub fn kpill(e: &mut E, x: f32, y: f32, id: usize, deg: f32) {
    let name = POOL[id].name;
    let w = tw(name, d(13)) + 40.0;
    sticker(e, S_FACE_INK, x, y, w, 25.0, deg, WHITE, |e| {
        spr_c(e, kem_of(id), x + 14.0, y + 12.5, 21.0 / 34.0, WHITE);
        e.text_ex(x + 28.0, y + 5.0, d(13), SUNNY, ALIGN_LEFT, 0.0, name);
    });
}

/// Stocks sticker: label and the pool's dots in an ink capsule (UI-SPEC 3.10).
pub fn stocks(e: &mut E, x: f32, y: f32, w: f32, left: i32, total: i32, label: &str, deg: f32) {
    sticker(e, S_FACE_WHITE, x, y, w, 50.0, deg, WHITE, |e| {
        e.text_ex(x + 10.0, y + 6.0, bs(12), LILAC, ALIGN_LEFT, 0.0, label);
        let mut s = Buf::new();
        s.i(left.max(0)).s(" of ").i(total);
        e.text_ex(x + w - 10.0, y + 6.0, d(13), INK, ALIGN_RIGHT, 0.0, s.str());
        let n = total.clamp(1, 6);
        let cw = n as f32 * 16.0 + 6.0;
        e.rect(x + 10.0, y + 25.0, cw, 18.0, INK);
        spr_c(e, S_DISC, x + 10.0, y + 34.0, 18.0 / 32.0, INK);
        spr_c(e, S_DISC, x + 10.0 + cw, y + 34.0, 18.0 / 32.0, INK);
        for i in 0..n {
            let cx = x + 18.0 + i as f32 * 16.0;
            if heart(e, i < left, cx, y + 34.0, 16.0, WHITE) {
            } else if i < left {
                spr_c(e, S_DISC, cx, y + 34.0, 12.0 / 32.0, WHITE);
            } else {
                spr_c(e, S_DISC, cx, y + 34.0, 12.0 / 32.0, alpha(PALE, 0.35));
            }
        }
    });
}

/// A run strip's marks: `n` stops, `done` finished, the next one coral (UI-SPEC 3.10).
pub fn marks(e: &mut E, x: f32, y: f32, n: i32, done: i32, gap: f32, r: f32) {
    for i in 0..n {
        let cx = x + i as f32 * gap;
        let boss = i == n - 1;
        let sz = r * 2.0 / 32.0 * if boss { 1.25 } else { 1.0 };
        spr_c(e, S_DISC, cx, y, sz * 1.35, INK);
        let col = if i < done { INK } else if i == done { CORAL } else { WHITE };
        spr_c(e, S_DISC, cx, y, sz, col);
        if boss {
            spr_c(e, S_STAR_ON, cx, y, r / 7.0, WHITE);
        }
    }
}

/// A drive chip (badge + level pips) centred at (cx, cy), badge `size` px.
pub fn chip(e: &mut E, cx: f32, cy: f32, id: usize, lv: i32, size: f32) {
    spr_c(e, badge_of(id), cx, cy, size / 28.5, WHITE);
    for k in 0..3 {
        let px = cx - 7.0 + k as f32 * 7.0;
        let col = if (k as i32) < lv { INK } else { alpha(PALE, 0.6) };
        e.rect(px - 2.5, cy + size * 0.5 + 2.0, 5.0, 4.0, col);
    }
    if lv > MAX_LEVEL {
        // past level 3 (endless loops): the level as a number by the pips
        let mut s = Buf::new();
        s.s("+").i(lv - MAX_LEVEL);
        e.text_ex(cx + 13.0, cy + size * 0.5 - 3.0, bs(10), CORAL, ALIGN_LEFT, 0.0, s.str());
    }
}

// ------------------------------------------------------------------ the main menu's Envoy row (UI-SPEC 2.1)

pub static mut MENU_AGE: i32 = 0;

/// What the camp, the menu and the map show of a run: the live view's or the saved record's.
pub struct Party {
    pub have: bool,
    pub players: usize,
    pub fight: i32,
    pub fights: i32,
    /// The loop (1 first; endless NG+ loops after a clear)
    pub lp: i32,
    pub pool: i32,
    pub pairs: [i32; 4],
    pub ids: [[i32; SLOTS]; 4],
    pub lvs: [[i32; SLOTS]; 4],
    pub keys: [[i32; KEYSTONES]; 2],
    pub shared: bool,
    /// Each player's item copies by id (the build the cards show off; envoy-items-design.md 11.11)
    pub items: [[u8; envoy_data::items::ITEM_IDS]; 2],
}

/// A fight kind's word (FK_*).
pub fn kind_word(k: i32) -> &'static str {
    match k {
        FK_PACK => "Pack",
        FK_ELITE => "Elite",
        FK_BOSS => "Boss",
        _ => "Duel",
    }
}

/// "Elite, then Boss": the session's fights from the next one on (view `V_SESS_KINDS`).
pub fn session_line(b: &mut Buf, kinds: i32) {
    let mut k = kinds;
    let mut first = true;
    while k & 15 != 0 {
        if !first {
            b.s(", then ");
        }
        b.s(kind_word((k & 15) - 1));
        first = false;
        k >>= 4;
    }
}

/// Sessions: the banner between fights of a session and over a fight's first seconds: "Fight 2 - Pack", then the
/// loop and the stocks left. `a`: opacity; `y`: its top.
pub fn session_banner(e: &mut E, v: &[i32; VW], a: f32, y: f32) {
    if a <= 0.0 {
        return;
    }
    let (x, w, h) = (170.0, 300.0, 86.0);
    sticker(e, S_FACE_SUNNY, x, y, w, h, -1.0, alpha(WHITE, a), |e| {
        let mut t = Buf::new();
        t.s("Fight ").i(v[V_FIGHT] + 1).s(" - ").s(kind_word(v[V_FIGHT_KIND]));
        e.text_ex(x + w * 0.5, y + 10.0, d(28), alpha(INK, a), ALIGN_CENTER, 0.0, t.str());
        let mut s = Buf::new();
        if v[V_LOOP] > 1 {
            s.s("Loop ").i(v[V_LOOP]).s(" - ");
        }
        s.i(v[V_POOL]).s(if v[V_POOL] == 1 { " stock left" } else { " stocks left" });
        e.text_ex(x + w * 0.5, y + 50.0, bs(14), alpha(INK, a), ALIGN_CENTER, 0.0, s.str());
    });
}

/// "Loop 3 - Fight 2 of 5" (loop 1: "Fight 2 of 5"); `fight` 0-based.
pub fn loop_fight(b: &mut Buf, lp: i32, fight: i32, fights: i32) {
    if lp > 1 {
        b.s("Loop ").i(lp).s(" - ");
    }
    b.s("Fight ").i(fight + 1).s(" of ").i(fights);
}

pub fn party_from_record(r: &[i32]) -> Party {
    let mut p = Party { have: true, players: if r[R_COOP] != 0 { 2 } else { 1 }, fight: r[R_FIGHT], fights: 4, lp: rec_loop(r[R_LOOP]).max(1), pool: r[R_POOL], pairs: [-1; 4],
        ids: [[0; SLOTS]; 4], lvs: [[0; SLOTS]; 4], keys: [[0; KEYSTONES]; 2], shared: r[R_SHARED] != 0, items: [[0; envoy_data::items::ITEM_IDS]; 2] };
    for set in 0..4 {
        for k in 0..SLOTS {
            (p.ids[set][k], p.lvs[set][k]) = rec_drive(r[R_DRIVES + set * SLOTS + k], r[1]);
        }
    }
    for q in 0..2 {
        p.keys[q].copy_from_slice(&r[R_KEYS + q * KEYSTONES..R_KEYS + (q + 1) * KEYSTONES]);
    }
    p.pairs.copy_from_slice(&r[R_PAIRS..R_PAIRS + 4]);
    // version 9: R_DRIVES holds item copies, a byte per id (envoy_sim items::rec_put), not drives
    if r[1] >= 9 {
        p.ids = [[0; SLOTS]; 4];
        p.keys = [[0; KEYSTONES]; 2];
        for q in 0..2 {
            for id in 1..envoy_data::items::ITEM_IDS {
                let w = if id < 40 { R_DRIVES + q * 10 + id / 4 } else { R_KEYS + 1 + q * 2 + (id - 40) / 4 };
                p.items[q][id] = (r[w] as u32 >> (8 * (id % 4)) & 0xFF) as u8;
            }
        }
    }
    p
}

pub fn party_from_view(v: &[i32; VW]) -> Party {
    let pb = |p: usize| V_P0 + p * V_PW;
    let mut q = Party { have: true, players: v[V_HUMANS].clamp(1, 2) as usize, fight: v[V_FIGHT], fights: v[V_FIGHTS].max(1), lp: v[V_LOOP].max(1), pool: v[V_POOL],
        pairs: [-1; 4], ids: [[0; SLOTS]; 4], lvs: [[0; SLOTS]; 4], keys: [[0; KEYSTONES]; 2], shared: v[V_SHARED] != 0, items: [[0; envoy_data::items::ITEM_IDS]; 2] };
    for set in 0..4 {
        let b = pb(set / 2) + (set % 2) * SLOTS;
        q.ids[set].copy_from_slice(&v[b + P_DRIVES..b + P_DRIVES + SLOTS]);
        q.lvs[set].copy_from_slice(&v[b + P_LEVELS..b + P_LEVELS + SLOTS]);
    }
    for p in 0..2 {
        q.keys[p].copy_from_slice(&v[pb(p) + P_KEYS..pb(p) + P_KEYS + KEYSTONES]);
        q.pairs[p * 2] = v[pb(p) + P_MAIN];
        q.pairs[p * 2 + 1] = v[pb(p) + P_PARTNER];
        for id in 1..envoy_data::items::ITEM_IDS {
            q.items[p][id] = v[envoy_data::view::inv_word(p, id)].clamp(0, 255) as u8;
        }
    }
    q
}

/// A build row: each item held as a chip of its colour (x2 on a stack), left to right from (x, cy), at most `w` wide.
/// The camp cards, the run-end summary and the pause header all show a build with it.
pub fn build_row(e: &mut E, items: &[u8; envoy_data::items::ITEM_IDS], x: f32, cy: f32, w: f32, size: f32) -> usize {
    let step = size + 4.0;
    let mut cx = x + size * 0.5;
    let mut n = 0;
    for it in envoy_data::items::ITEMS.iter() {
        let c = items[it.id as usize];
        if c == 0 {
            continue;
        }
        if cx > x + w - size * 0.5 {
            // no room left: a "+n" for the rest
            let more = envoy_data::items::ITEMS.iter().filter(|j| items[j.id as usize] > 0).count() - n;
            let mut s = Buf::new();
            s.s("+").i(more as i32);
            e.text_ex(cx - size * 0.4, cy - 6.0, bs(11), INK, ALIGN_LEFT, 0.0, s.str());
            break;
        }
        item_icon(e, it.id, cx, cy, size, 1.0);
        if c > 1 {
            let mut s = Buf::new();
            s.s("x").i(c as i32);
            e.text_ex(cx + size * 0.45, cy + size * 0.1, bs(9), INK, ALIGN_CENTER, 0.0, s.str());
        }
        cx += step;
        n += 1;
    }
    n
}

/// The held keystones of the party (pool indices), deduplicated, at most 3.
pub fn party_keys(p: &Party) -> ([usize; 3], usize) {
    let mut out = [0usize; 3];
    let mut n = 0;
    for q in 0..p.players {
        for &k in &p.keys[q] {
            if (1..=POOL_LEN as i32).contains(&k) && n < 3 && !out[..n].contains(&(k as usize - 1)) {
                out[n] = k as usize - 1;
                n += 1;
            }
        }
    }
    (out, n)
}

/// One fighter line of a run card: tag, head, name, chips.
fn fighter_line(e: &mut E, x: f32, y: f32, p: &Party, set: usize, first: bool) {
    let pl = set / 2;
    if first && p.players == 2 {
        face(e, if pl == 0 { S_FACE_CORAL } else { S_FACE_LAGOON }, x, y + 8.0, 22.0, 16.0, WHITE);
        e.text_ex(x + 11.0, y + 10.0, d(11), WHITE, ALIGN_CENTER, 0.0, if pl == 0 { "P1" } else { "P2" });
    }
    spr_c(e, S_HEAD, x + 36.0, y + 16.0, 0.9, WHITE);
    e.text_ex(x + 50.0, y + 10.0, d(12), INK, ALIGN_LEFT, 74.0, ck_name(p.pairs[set]));
    let mut cx = x + 136.0;
    for k in 0..SLOTS {
        let id = p.ids[set][k];
        if (1..=POOL_LEN as i32).contains(&id) {
            chip(e, cx, y + 14.0, id as usize - 1, p.lvs[set][k], 16.0);
            cx += 22.0;
        }
    }
}

/// The main menu's Envoy row and, while it is hovered, the loose stickers in the right-hand space.
/// The fallback Envoy row's pill (no native plate): below Data, on the 640x480 menu.
const FB_X: f32 = 66.0;
const FB_Y: f32 = 365.0;
pub unsafe fn menu(e: &mut E, st: i32, rec: Option<[i32; REC_WORDS]>) {
    let hover = st & 4 != 0;
    let miss = st & 32 != 0;
    // the row itself (plate, "Envoy" label, description line) is the host's, native (mnmain envoy_text_make); on a
    // menu model without the sixth plate anchor (state bit 8 clear) the row is plateless below Data and this draws it
    e.layer(LAYER_OVERLAY, 10);
    if st & 256 == 0 {
        e.rect(FB_X, FB_Y, 214.0, 30.0, if hover { 0xFFC8_1CFF } else { 0x1A12_2CFF });
        spr(e, S_LABEL + if miss { 2 } else if hover { 1 } else { 0 }, FB_X + 38.0, FB_Y - 4.0, 222.0, 38.0, WHITE);
    }
    // the Item Lab cell, right of the Envoy row (mnmain: Right / Left move to it while the row is hovered)
    if hover {
        let lab_on = st & (1 << 10) != 0;
        let (lx, ly) = (FB_X + 222.0, FB_Y + 1.0);
        face(e, if lab_on { S_FACE_HOVER } else { S_FACE_INK }, lx, ly, 104.0, 28.0, WHITE);
        e.text_ex(lx + 52.0, ly + 6.0, d(13), if lab_on { INK } else { SUNNY }, ALIGN_CENTER, 98.0, "Item Lab");
        if !lab_on {
            e.text_ex(lx + 98.0, ly + 7.0, bs(10), PALE, ALIGN_RIGHT, 0.0, ">");
        }
    }
    if !hover {
        MENU_AGE = 0;
        return;
    }
    MENU_AGE += 1;
    let t = MENU_AGE;
    let (ox, oy) = (272.0f32, 76.0f32);
    let pop = |e: &mut E, k: i32, x: f32, y: f32, w: f32, h: f32, deg: f32, inner: &dyn Fn(&mut E)| {
        let (s, a) = enter(t - k * 2);
        if a <= 0.0 {
            return;
        }
        let (cx, cy) = (x + w * 0.5, y + h * 0.5);
        rot(e, deg, cx, cy);
        // scale about the centre by drawing into a scaled rectangle: faces only (text keeps its size while it pops)
        let (sw, sh) = (w * s, h * s);
        let _ = (sw, sh, a);
        inner(e);
        unrot(e);
    };
    let party = rec.as_ref().map(|r| party_from_record(r));
    let mut bar = Buf::new();
    if miss {
        // Brawl disc missing: a dashed cream card with the three steps, the punch card faded
        let (x, y, w, h) = (ox + 2.0, oy + 8.0, 246.0, 176.0);
        pop(e, 0, x, y, w, h, -2.0, &|e| {
            face(e, S_FACE_CREAM, x, y, w, h, WHITE);
            spr_c(e, S_NODISC, x + 30.0, y + 32.0, 1.4, WHITE);
            e.text_ex(x + 52.0, y + 18.0, d(19), INK, ALIGN_LEFT, w - 60.0, "Needs your Brawl disc");
            e.text_ex(x + 14.0, y + 56.0, bs(13), LILAC, ALIGN_LEFT, w - 28.0, "Envoy opens once the disc is added.");
            for (i, s) in ["Open the launcher.", "Add your Brawl disc.", "Restart the game."].iter().enumerate() {
                let yy = y + 84.0 + i as f32 * 28.0;
                spr_c(e, S_DISC, x + 24.0, yy + 9.0, 20.0 / 32.0, INK);
                let mut n = Buf::new();
                n.i(i as i32 + 1);
                e.text_ex(x + 24.0, yy + 2.0, d(13), WHITE, ALIGN_CENTER, 0.0, n.str());
                e.text_ex(x + 42.0, yy + 1.0, bs(14), INK, ALIGN_LEFT, 0.0, s);
            }
        });
        bar.s("Envoy needs your Brawl disc. Add it in the launcher.");
    } else if let Some(p) = party.as_ref() {
        let sets = if p.shared { 1 } else { 2 };
        let lines = p.players * sets;
        let (x, y, w) = (ox + 2.0, oy + 8.0, 246.0);
        let h = 60.0 + lines as f32 * 32.0 + 10.0;
        pop(e, 0, x, y, w, h, -2.0, &|e| {
            face(e, S_FACE_WHITE, x, y, w, h, WHITE);
            e.rect(x + 1.5, y + 1.5, w - 3.0, 52.0, SUNNY);
            e.rect(x + 1.5, y + 53.5, w - 3.0, 2.5, INK);
            e.text_ex(x + 14.0, y + 6.0, d(23), INK, ALIGN_LEFT, 0.0, "Continue run");
            let mut s = Buf::new();
            loop_fight(&mut s, p.lp, p.fight, p.fights);
            s.s(". Saved after fight ").i(p.fight.max(1)).s(".");
            e.text_ex(x + 14.0, y + 33.0, bs(12), INK, ALIGN_LEFT, w - 28.0, s.str());
            let mut yy = y + 62.0;
            for q in 0..p.players {
                for f in 0..sets {
                    fighter_line(e, x + 8.0, yy, p, q * 2 + f, f == 0);
                    yy += 32.0;
                }
            }
        });
        // punch card: the run's fights as marks
        let (px, py) = (ox + 245.0, oy + 0.0);
        pop(e, 1, px, py, 97.0, 96.0, 3.0, &|e| {
            face(e, S_FACE_SUNNY, px, py, 97.0, 96.0, WHITE);
            e.text_ex(px + 48.0, py + 8.0, d(13), INK, ALIGN_CENTER, 0.0, "Run");
            marks(e, px + 16.0, py + 50.0, p.fights.min(5), p.fight, 16.0, 5.5);
            let mut s = Buf::new();
            s.i(p.fight).s(" of ").i(p.fights);
            e.text_ex(px + 48.0, py + 66.0, bs(12), INK, ALIGN_CENTER, 0.0, s.str());
            if p.lp > 1 {
                let mut s = Buf::new();
                s.s("Loop ").i(p.lp);
                e.text_ex(px + 48.0, py + 26.0, bs(12), CORAL, ALIGN_CENTER, 0.0, s.str());
            }
        });
        let sy = oy + 8.0 + h + 14.0;
        pop(e, 2, ox + 8.0, sy, 126.0, 50.0, -3.0, &|e| {
            stocks(e, ox + 8.0, sy, 126.0, p.pool, if p.players == 2 { 5 } else { 4 }, "Team stocks", 0.0);
        });
        let (ks, kn) = party_keys(p);
        for i in 0..kn {
            let kx = ox + 150.0 + if i == 1 { 8.0 } else { 0.0 };
            let ky = sy + 2.0 + i as f32 * 29.0;
            pop(e, 3 + i as i32, kx, ky, 120.0, 25.0, 0.0, &|e| kpill(e, kx, ky, ks[i], if i % 2 == 0 { -2.0 } else { 2.0 }));
        }
        bar.s("Continue your run from ");
        if p.lp > 1 {
            bar.s("loop ").i(p.lp).s(", ");
        }
        bar.s("fight ").i(p.fight + 1).s(".");
    } else {
        let (x, y, w, h) = (ox + 2.0, oy + 8.0, 246.0, 164.0);
        pop(e, 0, x, y, w, h, -2.0, &|e| {
            face(e, S_FACE_WHITE, x, y, w, h, WHITE);
            e.rect(x + 1.5, y + 1.5, w - 3.0, 52.0, SUNNY);
            e.rect(x + 1.5, y + 53.5, w - 3.0, 2.5, INK);
            e.text_ex(x + 14.0, y + 6.0, d(23), INK, ALIGN_LEFT, 0.0, "Start a run");
            e.text_ex(x + 14.0, y + 33.0, bs(12), INK, ALIGN_LEFT, w - 28.0, "Three rooms of waves, then a boss.");
            for (i, s) in ["Pick two fighters at the camp.", "Player 2 joins at the camp."].iter().enumerate() {
                let yy = y + 72.0 + i as f32 * 40.0;
                spr_c(e, S_DOT_INK, x + 20.0, yy + 8.0, 1.0, WHITE);
                e.text_ex(x + 34.0, yy, bs(14), INK, ALIGN_LEFT, w - 44.0, s);
            }
        });
        let (px, py) = (ox + 245.0, oy + 0.0);
        pop(e, 1, px, py, 97.0, 96.0, 3.0, &|e| {
            face(e, S_FACE_SUNNY, px, py, 97.0, 96.0, WHITE);
            e.text_ex(px + 48.0, py + 8.0, d(13), INK, ALIGN_CENTER, 0.0, "Run");
            marks(e, px + 24.0, py + 50.0, 4, -1, 16.0, 5.5);
            e.text_ex(px + 48.0, py + 66.0, bs(12), INK, ALIGN_CENTER, 0.0, "4 stops");
        });
        let (nx, ny) = (ox + 176.0, oy + 160.0);
        pop(e, 2, nx, ny, 150.0, 70.0, -3.0, &|e| {
            face(e, S_FACE_WHITE, nx, ny, 150.0, 70.0, WHITE);
            e.text_ex(nx + 12.0, ny + 10.0, d(17), INK, ALIGN_LEFT, 0.0, "One stock pool");
            e.text_ex(nx + 12.0, ny + 38.0, bs(13), LILAC, ALIGN_LEFT, 0.0, "5 together, 4 solo");
        });
        let (bx, by) = (ox + 60.0, oy + 210.0);
        let tb = t - 8;
        if tb > 0 {
            burst(e, S_BURST_CORAL, bx, by, 70.0, tb, "New!", 15);
        }
        bar.s("A co-op run for 1 or 2 players: rooms of waves and a boss.");
    }
    // the description bar's line is the host's own SIS line now (mnmain mn_80229A7C)
    let _ = bar.str();
    if st & 64 != 0 {
        // A without the disc: the message
        e.rect(0.0, 0.0, 640.0, 480.0, 0x1A10_4AA8);
        sticker(e, S_FACE_WHITE, 148.0, 150.0, 344.0, 150.0, -1.0, WHITE, |e| {
            spr_c(e, S_NODISC, 186.0, 190.0, 1.6, WHITE);
            e.text_ex(214.0, 174.0, d(21), INK, ALIGN_LEFT, 0.0, "Needs your Brawl disc");
            for (i, s) in ["1  Open the launcher.", "2  Add your Brawl disc.", "3  Restart the game."].iter().enumerate() {
                e.text_ex(214.0, 210.0 + i as f32 * 24.0, bs(15), INK, ALIGN_LEFT, 0.0, s);
            }
        });
    }
}
