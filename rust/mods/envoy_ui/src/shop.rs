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
        match row {
            0..=2 => {
                let id = v[pbk + P_SHOP_OFFERS + row as usize];
                if id >= 0 {
                    label.s(POOL[id as usize].name);
                    price = v[pbk + P_SHOP_PRICE + row as usize];
                    badge = Some(badge_of(id as usize));
                } else {
                    label.s("Sold");
                }
            }
            SHOP_LEVEL => {
                // one per visit: every price 0 once it is used (or nothing can level)
                let open = (0..SLOTS).any(|k| v[pbk + P_SHOP_LEVEL_PRICE + k] > 0);
                label.s(if open { "Level up a drive" } else { "Level-up: sold" });
            }
            SHOP_STOCK => {
                label.s("Restore a stock");
                price = v[pbk + P_SHOP_STOCK_PRICE];
            }
            SHOP_REROLL => {
                label.s("Swap the drives on sale");
                price = v[V_REROLL_COST];
            }
            SHOP_REORDER => {
                label.s("Reorder drives");
            }
            _ => {
                label.s("Back on the road");
            }
        }
        let gap = if row == SHOP_LEVEL || row == SHOP_REROLL || row == SHOP_LEAVE { 8.0 } else { 0.0 };
        y += gap;
        let (_, a) = enter(T - row * 2);
        let a = if done || step != 0 { a * 0.36 } else { a };
        let x = x0 + if on { 10.0 } else { 0.0 };
        let deg = if on { 0.0 } else if row % 2 == 0 { -0.7 } else { 0.6 };
        let rw = w - 20.0;
        sticker(e, if on { S_FACE_HOVER } else if row == SHOP_LEAVE { S_FACE_LILAC } else { S_FACE_WHITE }, x, y, rw, 26.0, deg, alpha(WHITE, a), |e| {
            let mut tx = x + 10.0;
            if let Some(bd) = badge {
                spr_c(e, bd, x + 16.0, y + 13.0, 18.0 / 28.5, alpha(WHITE, a));
                tx = x + 30.0;
            }
            e.text_ex(tx, y + 5.0, d(14), alpha(INK, a), ALIGN_LEFT, rw - 90.0, label.str());
            if price > 0 {
                let poor = price > coins;
                spr_c(e, S_COIN, x + rw - 48.0, y + 13.0, 0.85, alpha(WHITE, a));
                let mut s = Buf::new();
                s.i(price);
                e.text_ex(x + rw - 10.0, y + 5.0, d(14), alpha(if poor { DEEP } else { INK }, a), ALIGN_RIGHT, 0.0, s.str());
            } else if row == SHOP_REORDER {
                e.text_ex(x + rw - 10.0, y + 6.0, bs(12), alpha(LILAC, a), ALIGN_RIGHT, 0.0, "free");
            }
        });
        if on {
            spr(e, S_PTR + p, x - 30.0, y, 0.0, 0.0, WHITE);
        }
        y += 31.0;
    }
    // the fighter the shop acts on, its row (slot cursor in the slot steps)
    let party = party_from_view(v);
    let f = v[pbk + P_SHOP_FIGHTER].clamp(0, 1) as usize;
    let fy = y + 10.0;
    let set = p * 2 + f;
    fighter_sticker(e, x0, fy, -0.6, &party, set, false, WHITE);
    face(e, S_FACE_INK, x0 + 146.0, fy + 26.0, 90.0, 22.0, WHITE);
    e.text_ex(x0 + 191.0, fy + 29.0, d(11), WHITE, ALIGN_CENTER, 0.0, "X / Y: switch");
    if step >= 1 && !done {
        let k = v[pbk + P_SCURSOR].clamp(0, SLOTS as i32 - 1) as f32;
        let cx = x0 + 17.0 + k * 26.0;
        ring(e, cx, fy + 48.0, 13.0, 3.0, 1.0, SUNNY, SUNNY);
        spr(e, S_PTRDOWN + p, cx - 13.0, fy + 12.0, 0.0, 0.0, WHITE);
        if step == 3 {
            spr_c(e, S_PTR + p, cx - 20.0, fy + 48.0, 0.6, WHITE);
        }
    }
    if cur == SHOP_REORDER && step >= 2 && !done {
        reorder(e, v, p, x0, fy - 6.0);
    }
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
        if id >= 0 && w > 300.0 {
            let (tx, ty) = (x0 + w + 18.0, 56.0);
            let dr = &POOL[id as usize];
            sticker(e, S_FACE_WHITE, tx, ty, 260.0, 100.0, 1.0, WHITE, |e| {
                spr_c(e, badge_of(id as usize), tx + 6.0, ty + 6.0, 44.0 / 28.5, WHITE);
                e.text_ex(tx + 34.0, ty + 8.0, d(17), INK, ALIGN_LEFT, 0.0, dr.name);
                let mut s = Buf::new();
                s.s(["Common", "Rare", "Epic"][(dr.tier as usize).min(2)]).s(" ").s(FAM_WORD[FAM_BADGE.get(dr.family as usize).copied().unwrap_or(5)]);
                e.text_ex(tx + 250.0, ty + 10.0, bs(11), LILAC, ALIGN_RIGHT, 0.0, s.str());
                wrap(e, tx + 34.0, ty + 34.0, 216.0, bs(13), 16.0, INK, ALIGN_LEFT, dr.rule, 4);
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
    let say = match step {
        1 => "Choose a slot: left / right, then A.",
        2 => "Pick the drive to move.",
        3 => "Move it left or right, then put it down.",
        _ => match v[V_P0 + P_SHOP_CURSOR] {
            0..=2 => "Buy it, then choose who wears it.",
            SHOP_LEVEL => "Pick one of your drives to raise a level.",
            SHOP_STOCK => "One stock back in the shared pool.",
            SHOP_REROLL => "Three new drives on the counter.",
            SHOP_REORDER => "Change the order your drives fire in.",
            _ => "Leave the shop.",
        },
    };
    strip(e, 24.0, 590.0, say);
    let a = match step {
        1 => "Choose",
        2 => "Lift",
        3 => "Put it down",
        _ => "Buy",
    };
    if !crate::pick::confirm_up(v) {
        hints(e, &[("Up / Down", "Choose"), ("A", a), ("B", "Back"), ("X / Y", "Switch fighter")]);
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
