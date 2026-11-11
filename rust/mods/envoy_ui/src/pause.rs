//! Envoy's pause (UI-SPEC pause.png): Resume / Read drives / Quit to camp, the party's fighters with their drive
//! rows, the keystones with their rules, the D-pad card. The engine hides the retail panel while a run is armed and
//! takes our choices through `gd_ui_envoy_pause` (1 resume, 2 quit the fight as no contest; the run is kept).

use crate::camp::{fighter_sticker, read_pad};
use crate::kit::*;
use crate::party::*;
use crate::pick::wrap;
use crate::{Buf, VW};
use envoy_data::view::*;
use envoy_data::*;
use gd_ui_enc::*;

#[link(wasm_import_module = "gd_ui")]
extern "C" {
    fn gd_ui_envoy_pause(op: i32) -> i32;
}

static mut SEL: usize = 0;
static mut READ: bool = false;
static mut CONFIRM: bool = false;
static mut AGE: i32 = 0;
static mut SEEN: i32 = -100;
static mut T: i32 = 0;

/// Draw the pause and take its input; called every present while the game is paused and Envoy owns the pause.
pub unsafe fn draw(e: &mut E, v: &[i32; VW], ex: f32) {
    T += 1;
    // a new pause: start on Resume
    if T - SEEN > 2 {
        SEL = 0;
        READ = false;
        CONFIRM = false;
        AGE = 0;
    }
    SEEN = T;
    AGE += 1;
    // input (P1's pad; the press that paused must not also choose)
    if AGE > 6 {
        let p = read_pad(0);
        if READ {
            if p.a || p.b {
                READ = false;
            }
        } else if CONFIRM {
            if p.b {
                CONFIRM = false;
            } else if p.a {
                CONFIRM = false;
                gd_ui_envoy_pause(2);
            }
        } else {
            if p.up {
                SEL = (SEL + 2) % 3;
            }
            if p.down {
                SEL = (SEL + 1) % 3;
            }
            if p.a {
                match SEL {
                    0 => {
                        gd_ui_envoy_pause(1);
                    }
                    1 => READ = true,
                    _ => CONFIRM = true,
                }
            } else if p.b {
                gd_ui_envoy_pause(1);
            }
        }
    }
    e.layer(LAYER_MODAL, 0);
    e.rect(-ex, 0.0, 640.0 + 2.0 * ex, 480.0, 0x2C1E_70E0);
    text_edge(e, 16.0, 4.0, d(24), WHITE, ALIGN_LEFT, 2.2, "Paused");
    let party = party_from_view(v);
    if READ {
        read_drives(e, v, &party);
        hints(e, &[("A / B", "Back")]);
        return;
    }
    // the list
    let rows = ["Resume", "Read drives", "Quit to camp"];
    let mut y = 44.0;
    for (i, r) in rows.iter().enumerate() {
        let on = i == SEL && !CONFIRM;
        let (w, h, role) = if i == 0 { (226.0, 44.0, d(22)) } else { (216.0, 24.0, d(14)) };
        let (_, a) = enter(AGE - i as i32 * 2);
        sticker(e, if on { S_FACE_HOVER } else { S_FACE_WHITE }, 34.0, y, w, h, if on { 0.0 } else { [-0.6, 0.5, -0.4][i] }, alpha(WHITE, a), |e| {
            e.text_ex(46.0, y + if i == 0 { 9.0 } else { 4.0 }, role, alpha(INK, a), ALIGN_LEFT, 0.0, r);
            if i == 2 {
                e.text_ex(34.0 + w - 10.0, y + 6.0, bs(11), alpha(LILAC, a), ALIGN_RIGHT, 0.0, "the run is kept");
            }
        });
        if on {
            spr(e, S_PTR, 4.0, y + h * 0.5 - 13.0, 0.0, 0.0, WHITE);
        }
        y += h + 10.0;
    }
    // the D-pad card
    sticker(e, S_FACE_WHITE, 30.0, 190.0, 230.0, 74.0, -0.8, WHITE, |e| {
        e.text_ex(42.0, 197.0, d(14), INK, ALIGN_LEFT, 0.0, "The D-pad in a fight");
        spr_c(e, S_DPAD_DOWN, 50.0, 228.0, 0.9, WHITE);
        e.text_ex(62.0, 220.0, bs(12), INK, ALIGN_LEFT, 0.0, "Down: tag in your other fighter");
        spr_c(e, S_DPAD_RIGHT, 50.0, 248.0, 0.9, WHITE);
        e.text_ex(62.0, 240.0, bs(12), INK, ALIGN_LEFT, 0.0, "Right: fire your active drive");
    });
    // the party: fighter stickers, then the keystones with their rules
    let sets = if party.shared { 1 } else { 2 };
    let mut fy = 44.0;
    for q in 0..party.players {
        for f in 0..sets {
            let set = q * 2 + f;
            let fx = 300.0 + f as f32 * 150.0;
            fighter_sticker(e, fx, fy, if f == 0 { -0.6 } else { 0.5 }, &party, set, f == 0, WHITE);
        }
        fy += 88.0;
    }
    let (ks, kn) = party_keys(&party);
    let mut ky = fy + 4.0;
    for i in 0..kn {
        let dr = &POOL[ks[i]];
        sticker(e, S_FACE_INK, 300.0, ky, 314.0, 42.0, if i % 2 == 0 { -0.5 } else { 0.4 }, WHITE, |e| {
            spr_c(e, kem_of(ks[i]), 322.0, ky + 21.0, 30.0 / 34.0, WHITE);
            e.text_ex(344.0, ky + 4.0, d(14), SUNNY, ALIGN_LEFT, 0.0, dr.name);
            e.text_ex(344.0, ky + 22.0, bs(11), WHITE, ALIGN_LEFT, 262.0, dr.rule);
        });
        ky += 50.0;
    }
    if CONFIRM {
        sticker(e, S_FACE_CORAL, 150.0, 300.0, 340.0, 60.0, -1.2, WHITE, |e| {
            e.text_ex(320.0, 308.0, d(17), WHITE, ALIGN_CENTER, 0.0, "Quit this fight?");
            e.text_ex(320.0, 334.0, bs(12), WHITE, ALIGN_CENTER, 0.0, "Back to camp. The run is kept.");
        });
        hints(e, &[("A", "Quit the fight"), ("B", "Stay")]);
        return;
    }
    let say = match SEL {
        0 => "Back to the fight.",
        1 => "What every drive and keystone does.",
        _ => "Leave this fight. The run is saved at the camp.",
    };
    strip(e, 24.0, 590.0, say);
    hints(e, &[("Up / Down", "Choose"), ("A", rows[SEL]), ("Start", "Resume")]);
}

/// Every worn drive: badge, name, level, rule.
unsafe fn read_drives(e: &mut E, v: &[i32; VW], party: &Party) {
    let _ = v;
    let mut y = 44.0;
    let sets = if party.shared { 1 } else { 2 };
    let mut col = 0;
    for q in 0..party.players {
        for f in 0..sets {
            let set = q * 2 + f;
            let x = 16.0 + col as f32 * 310.0;
            let mut yy = y;
            let mut s = Buf::new();
            s.s(crate::ck_name(party.pairs[set]));
            text_edge(e, x, yy, d(14), WHITE, ALIGN_LEFT, 1.4, s.str());
            yy += 22.0;
            for k in 0..SLOTS {
                let id = party.ids[set][k];
                if !(1..=POOL_LEN as i32).contains(&id) {
                    continue;
                }
                let dr = &POOL[id as usize - 1];
                sticker(e, S_FACE_WHITE, x + 14.0, yy, 284.0, 40.0, 0.0, WHITE, |e| {
                    let mut n = Buf::new();
                    n.s(dr.name).s("  Lv ").i(party.lvs[set][k]);
                    e.text_ex(x + 36.0, yy + 3.0, d(12), INK, ALIGN_LEFT, 0.0, n.str());
                    wrap(e, x + 36.0, yy + 18.0, 254.0, bs(10), 11.0, INK, ALIGN_LEFT, dr.rule, 2);
                });
                spr_c(e, badge_of(id as usize - 1), x + 14.0, yy + 20.0, 26.0 / 28.5, WHITE);
                yy += 44.0;
            }
            col += 1;
            if col == 2 {
                col = 0;
                y = yy + 8.0;
            }
        }
    }
}
