//! The shop, S1 (UI-SPEC 2.7) on V3's shop words (envoy-v3-view.md): one list per player, the purse, the hovered
//! drive's tag, the fighter the shop acts on. The sim reads the pads (up / down, A, B, X / Y switches the fighter).

use crate::camp::fighter_sticker;
use crate::kit::*;
use crate::party::*;
use crate::pick::wrap;
use crate::{ck_name, Buf, VW};
use envoy_data::view::*;
use envoy_data::*;
use gd_ui_enc::*;

static mut T: i32 = 0;

const MSG: [&str; 13] = ["", "Bought.", "Not enough coins.", "Levelled up!", "Stock restored.", "Rerolled.", "Moved.", "Row full: choose a slot to replace.", "Replaced.", "Can't do that.", "Scrapped for coins.", "Pin changed.", "Kept."];

unsafe fn side(e: &mut E, v: &[i32; VW], p: usize, x0: f32, w: f32) {
    let pbk = V_P0 + p * V_PW;
    let coins = v[V_COINS];
    let cur = v[pbk + P_SHOP_CURSOR];
    let step = v[pbk + P_SHOP_STEP];
    let done = v[pbk + P_SHOP_DONE] != 0;
    let mut y = 50.0;
    for row in 0..SHOP_ROWS {
        let on = row == cur && !done && step == 0;
        let mut label = Buf::new();
        let mut price = 0;
        let mut badge = None;
        let mut slot_row = false;
        match row {
            0..=2 => {
                let id = v[pbk + P_SHOP_OFFERS + row as usize];
                if let Some(it) = envoy_data::items::item(id) {
                    label.s(it.name);
                    let held = v[V_INV + p * envoy_data::items::ITEM_IDS + id as usize];
                    if held > 0 {
                        label.s("  (x").i(held + 1).s(")");
                    }
                    price = v[pbk + P_SHOP_PRICE + row as usize];
                    badge = Some(it.id);
                } else {
                    label.s("Sold");
                }
            }
            SHOP_STOCK => {
                let (now, lim) = crate::hud::stocks_of(v, p);
                label.s("Refill a stock  ").i(now).s("/").i(lim);
                if now < lim {
                    price = v[V_STOCK_PRICE];
                }
            }
            SHOP_SLOT => {
                label.s("Stock slot: +1 limit, +1 stock");
                price = v[V_SHOP_SLOT_PRICE];
                slot_row = true;
            }
            SHOP_REROLL => {
                label.s("New items on the counter");
                price = v[V_REROLL_COST];
            }
            SHOP_REORDER => {
                label.s("Reorder drives");
            }
            _ => {
                label.s("Back on the road");
            }
        }
        let gap = if row == SHOP_STOCK || row == SHOP_LEAVE { 8.0 } else { 0.0 };
        y += gap;
        let (_, a) = enter(T - row * 2);
        let a = if done || step != 0 { a * 0.36 } else { a };
        let x = x0 + if on { 10.0 } else { 0.0 };
        let deg = if on { 0.0 } else if row % 2 == 0 { -0.7 } else { 0.6 };
        let rw = w - 20.0;
        sticker(e, if on { S_FACE_HOVER } else if row == SHOP_LEAVE { S_FACE_LILAC } else { S_FACE_WHITE }, x, y, rw, 26.0, deg, alpha(WHITE, a), |e| {
            let mut tx = x + 10.0;
            if slot_row {
                crate::icons::slot_ink(e, x + 12.0, y + 13.0, 12.0, false, false, a);
                crate::icons::slot_ink(e, x + 24.0, y + 13.0, 12.0, true, false, a);
                tx = x + 36.0;
            }
            if let Some(col) = badge {
                // the item's icon: its colour in an ink ring
                crate::icons::icon_a(e, col, x + 16.0, y + 13.0, 22.0, a);
                tx = x + 30.0;
            }
            e.text_ex(tx, y + 5.0, d(14), alpha(INK, a), ALIGN_LEFT, rw - 90.0, label.str());
            if price > 0 {
                let poor = price > coins;
                spr_c(e, S_COIN, x + rw - 48.0, y + 13.0, 0.85, alpha(WHITE, a));
                let mut s = Buf::new();
                s.i(price);
                e.text_ex(x + rw - 10.0, y + 5.0, d(14), alpha(if poor { DEEP } else { INK }, a), ALIGN_RIGHT, 0.0, s.str());
            } else if slot_row {
                e.text_ex(x + rw - 10.0, y + 6.0, bs(12), alpha(LILAC, a), ALIGN_RIGHT, 0.0, "not today");
            } else if row == SHOP_STOCK {
                e.text_ex(x + rw - 10.0, y + 6.0, bs(12), alpha(LILAC, a), ALIGN_RIGHT, 0.0, "full");
            }
        });
        if on {
            spr(e, S_PTR + p, x - 30.0, y, 0.0, 0.0, WHITE);
        }
        y += 31.0;
    }
    // the items this player holds (no slots: a strip, copies as x2)
    let fy = y + 10.0;
    crate::items_ui::strip(e, v, p, x0, fy + 6.0, w - 20.0);
    let _ = (fighter_sticker as unsafe fn(_, _, _, _, _, _, _, _), reorder as unsafe fn(_, _, _, _, _));
    let msg = v[pbk + P_SHOP_MSG].clamp(0, 12) as usize;
    if msg != 0 && !done {
        let good = matches!(msg, 1 | 3 | 4 | 5 | 6 | 8 | 10 | 11 | 12);
        let mw = tw(MSG[msg], d(14)) + 24.0;
        sticker(e, if good { S_FACE_SUNNY } else { S_FACE_CORAL }, x0 + 246.0, fy + 22.0, mw, 26.0, -2.0, WHITE, |e| {
            e.text_ex(x0 + 258.0, fy + 27.0, d(14), INK, ALIGN_LEFT, 0.0, MSG[msg]);
        });
    }
    if done {
        burst(e, S_BURST_SUNNY, x0 + w * 0.5, 160.0, 90.0, 10, "Done!", 18);
    }
    // the hovered drive's tag (solo: right side)
    if (0..3).contains(&cur) && step == 0 && !done {
        let id = v[pbk + P_SHOP_OFFERS + cur as usize];
        if let Some(it) = envoy_data::items::item(id).filter(|_| w > 300.0) {
            let (tx, ty) = (x0 + w + 18.0, 56.0);
            sticker(e, S_FACE_WHITE, tx, ty, 260.0, 100.0, 1.0, WHITE, |e| {
                crate::icons::icon(e, it.id, tx + 8.0, ty + 8.0, 38.0);
                e.text_ex(tx + 34.0, ty + 8.0, d(17), INK, ALIGN_LEFT, 0.0, it.name);
                let mut s = Buf::new();
                s.s("on ").s(envoy_data::items::TRIGGER_TEXT[it.trigger as usize]).s(" / stack: ").s(envoy_data::items::STACK_TEXT[it.stack as usize]);
                e.text_ex(tx + 250.0, ty + 10.0, bs(11), LILAC, ALIGN_RIGHT, 0.0, s.str());
                wrap(e, tx + 34.0, ty + 34.0, 216.0, bs(13), 16.0, INK, ALIGN_LEFT, it.line, 4);
            });
        }
    }
}

/// The shop over the frozen fight (band space for ours; `sc` to go back to window space).
pub unsafe fn draw(e: &mut E, v: &[i32; VW], ex: f32, _vw: f32, _vh: f32, sc: f32) {
    T += 1;
    e.space(SPACE_BAND, 1.0);
    e.layer(LAYER_MODAL, 0);
    e.rect(-ex, 0.0, 640.0 + 2.0 * ex, 480.0, 0x2C1E_70F4);
    text_edge(e, 16.0, 4.0, d(24), WHITE, ALIGN_LEFT, 2.2, "Rest stop");
    // the purse
    let mut s = Buf::new();
    s.i(v[V_COINS]);
    sticker(e, S_FACE_WHITE, 160.0, 6.0, 128.0, 34.0, 1.0, WHITE, |e| {
        spr_c(e, S_COIN, 180.0, 23.0, 1.3, WHITE);
        e.text_ex(196.0, 11.0, d(20), INK, ALIGN_LEFT, 0.0, s.str());
        e.text_ex(278.0, 16.0, bs(11), LILAC, ALIGN_RIGHT, 0.0, "one purse");
    });
    if v[V_TIMER] > 0 {
        let mut s = Buf::new();
        s.i((v[V_TIMER] + 59) / 60).s(" s");
        e.text_ex(620.0, 12.0, d(15), WHITE, ALIGN_RIGHT, 0.0, s.str());
    }
    let coop = v[V_HUMANS] == 2;
    if coop {
        for p in 0..2 {
            side(e, v, p, 36.0 + p as f32 * 316.0, 286.0);
        }
    } else {
        side(e, v, 0, 40.0, 316.0);
    }
    let step = v[V_P0 + P_SHOP_STEP];
    let _ = step;
    let say = match v[V_P0 + P_SHOP_CURSOR] {
        0..=2 => "Buy it: it is yours at once. A second copy stacks.",
        SHOP_STOCK => "One stock back, up to your stock limit.",
        SHOP_REROLL => "Three new items on the counter.",
        _ => "Leave the shop.",
    };
    strip(e, 24.0, 590.0, say);
    if !crate::pick::confirm_up(v) {
        hints(e, &[("Up / Down", "Choose"), ("A", "Buy")]);
    }
    let _ = ck_name;
    e.space(SPACE_WINDOW, sc);
}

/// The reorder screen (the stop's last row): both of a player's rows, a pin on pinned chips, each drive's scrap
/// price; the lifted drive rides above its slot and moves with left / right, up / down to the other fighter.
unsafe fn reorder(e: &mut E, v: &[i32; VW], p: usize, x: f32, y: f32) {
    let pbk = V_P0 + p * V_PW;
    let party = party_from_view(v);
    let row = v[pbk + P_REORDER_ROW].clamp(0, 1) as usize;
    let k = v[pbk + P_SCURSOR].clamp(0, SLOTS as i32 - 1) as usize;
    let lift = v[pbk + P_REORDER_LIFT] != 0;
    let pins = v[pbk + P_PINS];
    let shared = v[V_SHARED] != 0;
    let rows = if shared { 1 } else { 2 };
    let h = 30.0 + rows as f32 * 58.0;
    sticker(e, S_FACE_WHITE, x, y, 300.0, h, -0.4, WHITE, |e| {
        e.text_ex(x + 12.0, y + 6.0, d(14), INK, ALIGN_LEFT, 0.0, "Order, pins and scrap");
        for f in 0..rows {
            let ry = y + 30.0 + f as f32 * 58.0;
            let set = if shared { 0 } else { f };
            let ck = party.pairs[p * 2 + f];
            e.text_ex(x + 12.0, ry + 2.0, bs(11), LILAC, ALIGN_LEFT, 0.0, if f == 0 { "Main" } else { "Partner" });
            e.text_ex(x + 60.0, ry + 2.0, bs(11), INK, ALIGN_LEFT, 0.0, crate::ck_name(ck));
            for s in 0..SLOTS {
                let cx = x + 26.0 + s as f32 * 54.0;
                let cy = ry + 28.0;
                let here = f == row && s == k;
                let id = v[pbk + P_DRIVES + set * SLOTS + s];
                let dy = if here && lift { -8.0 } else { 0.0 };
                if (1..=POOL_LEN as i32).contains(&id) {
                    chip(e, cx, cy + dy, id as usize - 1, v[pbk + P_LEVELS + set * SLOTS + s], 24.0);
                    if pins >> (f * 8 + s) & 1 != 0 {
                        face(e, S_FACE_INK, cx + 6.0, cy - 16.0 + dy, 14.0, 14.0, WHITE);
                        e.text_ex(cx + 13.0, cy - 15.0 + dy, d(9), SUNNY, ALIGN_CENTER, 0.0, "P");
                    } else {
                        let price = v[pbk + P_SCRAP_PRICE + f * SLOTS + s];
                        if price > 0 {
                            let mut b = Buf::new();
                            b.i(price);
                            spr_c(e, S_COIN, cx - 6.0, cy + 19.0, 0.6, WHITE);
                            e.text_ex(cx + 1.0, cy + 14.0, bs(10), LILAC, ALIGN_LEFT, 0.0, b.str());
                        }
                    }
                } else {
                    spr_c(e, S_DISC, cx, cy, 19.0 / 32.0, alpha(PALE, 0.5));
                }
                if here {
                    ring(e, cx, cy + dy, 15.0, 3.0, 1.0, if lift { CORAL } else { SUNNY }, 0);
                }
            }
        }
    });
    let say = if lift { "Moving: left / right, up / down to the other fighter. A puts it down." } else { "A lifts. Y pins. X scraps for the coins shown. B is done." };
    strip(e, x, 600.0, say);
}
