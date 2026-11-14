//! The floor map (envoy-items-design.md 11.4): the loop's rooms as a branching map, drawn only from the sim's map
//! words (V_MAP_*: kind, column / row, state bits, links), so it stays right when the sim changes the layout. One
//! widget for the camp's road screen, the in-match fork and the pause Run page.
use crate::icons::ln;
use crate::kit::*;
use crate::{Buf, VW};
use envoy_data::view::*;
use gd_ui_enc::*;

static mut T: i32 = 0;

const RED: u32 = 0xD8203AFF;
const FLAME: u32 = 0xFF8A_1FFF;

fn node(v: &[i32; VW], i: usize) -> (i32, i32, i32, i32, i32) {
    let b = V_MAP0 + i * MAP_W;
    let pos = v[b + MP_POS];
    (v[b + MP_KIND], pos & 0xFF, (pos >> 8) & 0xFF, v[b + MP_STATE], v[b + MP_LINKS])
}

/// The centre of node `i` in the box (x, y, w, h).
fn at(v: &[i32; VW], i: usize, x: f32, y: f32, w: f32, h: f32, cols: i32) -> (f32, f32) {
    let (_, c, row, _, _) = node(v, i);
    let step = w / cols.max(1) as f32;
    let cx = x + step * (c as f32 + 0.5);
    let dy = h * 0.32;
    let cy = y + h * 0.5 + if row == 255 { -dy } else if row == 1 { dy } else { 0.0 };
    (cx, cy)
}

/// Whether the sim has published a map.
pub fn has(v: &[i32; VW]) -> bool {
    v[V_MAP_N] > 1
}

/// The floor map in the box (x, y, w, h), node radius `r`. Band space; draws nothing without a map.
pub unsafe fn draw(e: &mut E, v: &[i32; VW], x: f32, y: f32, w: f32, h: f32, r: f32) {
    if !has(v) {
        return;
    }
    T += 1;
    let n = (v[V_MAP_N] as usize).min(MAP_N);
    let mut cols = 1;
    for i in 0..n {
        cols = cols.max(node(v, i).1 + 1);
    }
    // the roads first
    for i in 0..n {
        let (_, _, _, st, links) = node(v, i);
        let (ax, ay) = at(v, i, x, y, w, h, cols);
        for j in 0..n {
            if links & (1 << j) == 0 {
                continue;
            }
            let (_, _, _, sj, _) = node(v, j);
            let (bx, by) = at(v, j, x, y, w, h, cols);
            let secret = st & MS_WALL != 0 && node(v, j).0 == MN_SECRET;
            let walked = st & (MS_DONE | MS_HERE) != 0 && sj & (MS_DONE | MS_HERE | MS_CHOSEN) != 0;
            let skipped = sj & MS_SKIPPED != 0;
            let col = if walked { SUNNY } else if skipped { alpha(PALE, 0.35) } else { alpha(WHITE, 0.85) };
            let (dx, dy) = (bx - ax, by - ay);
            let len = (dx * dx + dy * dy).sqrt().max(1.0);
            if secret || sj & MS_UNKNOWN != 0 {
                // dashed: a hidden way
                let k = (len / (r * 0.8)) as i32;
                for s in 0..k {
                    if s % 2 == 0 {
                        let (t0, t1) = (s as f32 / k as f32, (s as f32 + 0.6) / k as f32);
                        ln(e, ax + dx * t0, ay + dy * t0, ax + dx * t1, ay + dy * t1, r * 0.22, alpha(PALE, 0.8));
                    }
                }
            } else {
                ln(e, ax, ay, bx, by, r * 0.5, INK);
                ln(e, ax, ay, bx, by, r * 0.28, col);
            }
        }
    }
    for i in 0..n {
        let (kind, _, _, st, _) = node(v, i);
        let (cx, cy) = at(v, i, x, y, w, h, cols);
        node_icon(e, v, kind, st, cx, cy, r);
    }
}

/// One room's sticker and mark.
unsafe fn node_icon(e: &mut E, v: &[i32; VW], kind: i32, st: i32, cx: f32, cy: f32, r: f32) {
    let t = T as f32;
    let a = if st & MS_SKIPPED != 0 { 0.4 } else { 1.0 };
    if st & MS_HERE != 0 {
        spr_c(e, S_BURST_SUNNY, cx, cy, r * 2.9 / 58.0 * (1.0 + 0.06 * sin(t * 0.12)), WHITE);
    }
    if st & MS_CURSOR != 0 {
        spr_c(e, S_BURST_CORAL, cx, cy, r * 3.1 / 58.0 * (1.0 + 0.1 * sin(t * 0.2)), WHITE);
    } else if st & MS_NEXT != 0 {
        spr_c(e, S_DISC, cx, cy, (r * 2.0 + 5.0 + 2.0 * sin(t * 0.15)) / 32.0, alpha(WHITE, 0.55));
    }
    let face_id = if st & MS_UNKNOWN != 0 {
        S_FACE_LILAC
    } else if st & MS_DONE != 0 && st & MS_HERE == 0 {
        S_FACE_FADED
    } else {
        match kind {
            MN_BOSS | MN_DEVIL => S_FACE_CORAL,
            MN_ITEM => S_FACE_SUNNY,
            MN_SHOP | MN_START => S_FACE_LAGOON,
            MN_CAMPFIRE => S_FACE_CREAM,
            MN_ELITE => S_FACE_INK,
            _ => S_FACE_WHITE,
        }
    };
    let rr = if kind == MN_BOSS { r * 1.2 } else { r };
    face(e, face_id, cx - rr, cy - rr, rr * 2.0, rr * 2.0, alpha(WHITE, a));
    let g = alpha(INK, a);
    let s = rr / 10.0;
    if st & MS_UNKNOWN != 0 {
        text(e, cx, cy - 7.0 * s, d((12.0 * s) as u32), alpha(WHITE, a), ALIGN_CENTER, "?");
        return;
    }
    match kind {
        MN_START => {
            e.rect(cx - 3.0 * s, cy - 6.0 * s, 1.6 * s, 12.0 * s, alpha(WHITE, a));
            e.rect(cx - 1.5 * s, cy - 6.0 * s, 6.0 * s, 4.5 * s, alpha(SUNNY, a));
        }
        MN_DUEL => {
            // crossed swords
            ln(e, cx - 5.0 * s, cy - 5.0 * s, cx + 5.0 * s, cy + 5.0 * s, 1.8 * s, g);
            ln(e, cx + 5.0 * s, cy - 5.0 * s, cx - 5.0 * s, cy + 5.0 * s, 1.8 * s, g);
            e.rect(cx - 5.5 * s, cy + 2.5 * s, 3.0 * s, 1.4 * s, g);
            e.rect(cx + 2.5 * s, cy + 2.5 * s, 3.0 * s, 1.4 * s, g);
        }
        MN_CROWD => {
            for k in 0..3 {
                let hx = cx + (k as f32 - 1.0) * 4.6 * s;
                let hy = cy + if k == 1 { -1.5 } else { 0.5 } * s;
                spr_c(e, S_DISC, hx, hy - 2.0 * s, 3.8 * s / 32.0 * 1.0, g);
                e.rect(hx - 2.0 * s, hy + 0.5 * s, 4.0 * s, 4.0 * s, g);
            }
        }
        MN_ELITE => {
            // a crown
            e.rect(cx - 5.5 * s, cy + 0.5 * s, 11.0 * s, 3.5 * s, alpha(SUNNY, a));
            for k in 0..3 {
                let px = cx + (k as f32 - 1.0) * 4.5 * s;
                ln(e, px, cy + 1.0 * s, px, cy - 4.5 * s, 2.0 * s, alpha(SUNNY, a));
                spr_c(e, S_DISC, px, cy - 5.0 * s, 2.6 * s / 32.0, alpha(SUNNY, a));
            }
        }
        MN_BOSS => {
            // a skull
            spr_c(e, S_DISC, cx, cy - 1.0 * s, 12.0 * s / 32.0, alpha(WHITE, a));
            e.rect(cx - 3.5 * s, cy + 2.0 * s, 7.0 * s, 4.0 * s, alpha(WHITE, a));
            spr_c(e, S_DISC, cx - 2.6 * s, cy - 1.0 * s, 3.6 * s / 32.0, g);
            spr_c(e, S_DISC, cx + 2.6 * s, cy - 1.0 * s, 3.6 * s / 32.0, g);
            e.rect(cx - 0.6 * s, cy + 3.0 * s, 1.2 * s, 3.0 * s, g);
        }
        MN_ITEM => spr_c(e, S_STAR_ON, cx, cy, 1.25 * s, alpha(WHITE, a)),
        MN_SHOP => spr_c(e, S_COIN, cx, cy, 0.85 * s, alpha(WHITE, a)),
        MN_CAMPFIRE => {
            spr_c(e, S_FLAME + 2, cx, cy - 1.0 * s, 0.55 * s, alpha(WHITE, a));
            e.rect(cx - 5.0 * s, cy + 5.0 * s, 10.0 * s, 1.8 * s, alpha(0x7A4A_20FF, a));
            let _ = FLAME;
        }
        MN_DOOR | MN_DEVIL | MN_ANGEL => {
            let dc = match kind { MN_DEVIL => RED, MN_ANGEL => WHITE, _ => PALE };
            e.rect(cx - 4.0 * s, cy - 3.0 * s, 8.0 * s, 10.0 * s, alpha(INK, a));
            spr_c(e, S_DISC, cx, cy - 3.0 * s, 8.0 * s / 32.0, alpha(INK, a));
            e.rect(cx - 3.0 * s, cy - 3.0 * s, 6.0 * s, 9.0 * s, alpha(dc, a));
            spr_c(e, S_DISC, cx, cy - 3.0 * s, 6.0 * s / 32.0, alpha(dc, a));
            if kind == MN_DOOR {
                text(e, cx, cy - 6.5 * s, d((9.0 * s) as u32), alpha(INK, a), ALIGN_CENTER, "?");
                let pct = v[V_DOOR_PCT];
                if pct > 0 {
                    let mut b = Buf::new();
                    b.i(pct).s("%");
                    text_edge(e, cx, cy + rr + 1.0, bs((10.0 * s).max(9.0) as u32), WHITE, ALIGN_CENTER, 1.2, b.str());
                }
            } else {
                spr_c(e, S_DISC, cx + 1.5 * s, cy + 2.0 * s, 1.6 * s / 32.0, alpha(INK, a));
            }
        }
        MN_SECRET => {
            spr_c(e, S_STAR_ON, cx, cy, 1.0 * s, alpha(WHITE, a));
            ln(e, cx - 6.0 * s, cy - 6.0 * s, cx - 2.0 * s, cy - 1.0 * s, 1.2 * s, g);
        }
        _ => {}
    }
    if st & MS_WALL != 0 {
        // the cracked wall mark on the fight: a small stone block, cracked (lit once broken)
        let (wx, wy) = (cx + rr * 0.85, cy + rr * 0.75);
        let found = st & MS_FOUND != 0;
        face(e, if found { S_FACE_SUNNY } else { S_FACE_FADED }, wx - 5.0 * s, wy - 5.0 * s, 10.0 * s, 10.0 * s, WHITE);
        ln(e, wx - 2.5 * s, wy - 4.0 * s, wx + 0.5 * s, wy, 1.1 * s, INK);
        ln(e, wx + 0.5 * s, wy, wx - 1.0 * s, wy + 4.0 * s, 1.1 * s, INK);
    }
    if st & (MS_STEEP | MS_SHIFT) != 0 {
        let tag = if st & MS_STEEP != 0 { "+" } else { "~" };
        let (tx, ty) = (cx - rr * 0.9, cy - rr * 0.95);
        spr_c(e, S_DISC, tx, ty, 9.0 * s / 32.0, CORAL);
        text(e, tx, ty - 5.5 * s, d((9.0 * s) as u32), WHITE, ALIGN_CENTER, tag);
    }
    if st & MS_SKIPPED != 0 {
        ln(e, cx - rr * 0.7, cy - rr * 0.7, cx + rr * 0.7, cy + rr * 0.7, 1.6 * s, alpha(CORAL, 0.9));
    }
    if st & MS_CHOSEN != 0 && st & MS_DONE == 0 && st & MS_HERE == 0 {
        spr_c(e, S_DISC, cx + rr * 0.8, cy - rr * 0.8, 6.0 * s / 32.0, SUNNY);
    }
}

/// The word for a room kind (fork choices, callouts).
pub fn kind_word(k: i32) -> &'static str {
    match k {
        MN_START => "Start",
        MN_DUEL => "Duels",
        MN_CROWD => "Crowd",
        MN_ELITE => "Elite",
        MN_BOSS => "Boss",
        MN_ITEM => "Item room",
        MN_SHOP => "Shop",
        MN_CAMPFIRE => "Campfire",
        MN_DOOR => "Door",
        MN_DEVIL => "Devil door",
        MN_ANGEL => "Angel door",
        MN_SECRET => "Secret room",
        _ => "",
    }
}

/// A legend line under a large map: what each mark means (only the kinds on this map).
pub unsafe fn legend(e: &mut E, v: &[i32; VW], x: f32, y: f32, max_w: f32) {
    if !has(v) {
        return;
    }
    let mut b = Buf::new();
    b.s("Floor ").i(v[V_MAP_LOOP].max(1));
    let at = v[V_MAP_AT];
    if (0..v[V_MAP_N]).contains(&at) {
        let (k, ..) = node(v, at as usize);
        b.s("  -  at the ").s(kind_word(k));
    }
    if v[V_DOOR_PCT] > 0 {
        b.s("  -  door after the boss: ").i(v[V_DOOR_PCT]).s("%");
    }
    text_edge(e, x, y, bs(12), WHITE, ALIGN_LEFT, 1.3, b.str());
    let _ = max_w;
}
