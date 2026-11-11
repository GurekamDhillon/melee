//! The fight HUD, H2 + M1 (UI-SPEC 2.5), drawn over the retail HUD: the retail percent and stock icons stay at the
//! bottom (hiding them would touch game objects), so each player's corner sits just above them: tag and fighter,
//! the drive row with charge rings and level pips, the Heat bar (marks, flame, no digits) and the bench sticker.
//! Shared things go to the top: keystones and coins at the top left, the stock pool, foe plates top right, the boss
//! bar top centre.

use crate::kit::*;
use crate::party::*;
use crate::{ck_name, Buf, VW};
use envoy_data::view::*;
use envoy_data::*;
use gd_ui_enc::*;

/// V3's Heat words (envoy-v3-view.md, port/v3sim): present when `V_HEAT_ON` (29) is 1.
pub mod v3 {
    pub const V_COINS: usize = 25;
    pub const V_HEAT_ON: usize = 29;
    pub const P_HEAT: usize = 80;
    pub const P_HEAT_CAP: usize = 81;
    pub const P_HEAT_MARKS: usize = 82;
    pub const P_HEAT_STATE: usize = 85;
    pub const P_HEAT_FULL: usize = 86;
    pub const P_ACT_ID: usize = 92;
    pub const P_PIPS: usize = 93;
    pub const P_PIPS_NEED: usize = 94;
    pub const P_ACT_READY: usize = 106;
}

/// Heat as the HUD draws it: value, cap, the three marks, state, full.
#[derive(Clone, Copy)]
pub struct Heat {
    pub v: f32,
    pub cap: f32,
    pub marks: [f32; 3],
    pub state: usize,
    pub full: bool,
}

/// Layout check knob: draw a moving Heat bar from a stub when the sim has no Heat (V2 rules). Off in the demo.
pub const STUB_HEAT: bool = false;

pub fn heat_of(v: &[i32; VW], p: usize) -> Option<Heat> {
    let pbk = V_P0 + p * V_PW;
    if v.len() > v3::V_HEAT_ON && v[v3::V_HEAT_ON] == 1 {
        let cap = (v[pbk + v3::P_HEAT_CAP] as f32 / 1000.0).max(1.0);
        return Some(Heat {
            v: v[pbk + v3::P_HEAT] as f32 / 1000.0,
            cap,
            marks: [v[pbk + v3::P_HEAT_MARKS] as f32 / 1000.0, v[pbk + v3::P_HEAT_MARKS + 1] as f32 / 1000.0, v[pbk + v3::P_HEAT_MARKS + 2] as f32 / 1000.0],
            state: v[pbk + v3::P_HEAT_STATE].clamp(0, 3) as usize,
            full: v[pbk + v3::P_HEAT_FULL] != 0,
        });
    }
    if STUB_HEAT {
        let t = (v[V_PLAY] % 1200) as f32 / 1200.0;
        let h = 225.0 * t;
        let marks = [40.0, 90.0, 165.0];
        let state = marks.iter().filter(|&&m| h >= m).count();
        return Some(Heat { v: h, cap: 225.0, marks, state, full: false });
    }
    None
}

pub static mut PREV_HEAT: [f32; 2] = [0.0; 2];
pub static mut GAIN_AT: [i32; 2] = [-100; 2];
pub static mut STATE_AT: [(usize, i32); 2] = [(0, -100); 2];
static mut T: i32 = 0;

/// The Heat bar: track (ink ring, white ring), fill in the state's colour, three marks, the flame at its foot.
fn heat_bar(e: &mut E, x: f32, y: f32, w: f32, h: &Heat, p: usize) {
    let t = unsafe { T };
    let f = (h.v / h.cap).clamp(0.0, 1.0);
    // rings
    let ring = if h.full { SUNNY } else { INK };
    e.rect(x - 4.5, y - 4.5, w + 9.0, 20.0, ring);
    e.rect(x - 2.0, y - 2.0, w + 4.0, 15.0, WHITE);
    e.rect(x, y, w, 11.0, TRACK);
    e.rect(x, y, w * f, 11.0, HEAT_COL[h.state]);
    unsafe {
        if h.v > PREV_HEAT[p] + 0.01 {
            GAIN_AT[p] = t;
        }
        let gained = t - GAIN_AT[p] < 6;
        if gained {
            e.rect(x + w * f - 4.0, y, 4.0, 11.0, WHITE);
        }
        if h.state != STATE_AT[p].0 {
            STATE_AT[p] = (h.state, t);
        }
        PREV_HEAT[p] = h.v;
    }
    if h.state == 3 {
        e.rect(x, y + 2.0, w * f, 2.6, alpha(WHITE, 0.85));
    }
    for (k, &m) in h.marks.iter().enumerate() {
        let mx = x + w * (m / h.cap).clamp(0.0, 1.0);
        e.rect(mx - 1.0, y, 2.0, 11.0, INK);
        e.rect(mx - 1.0, y - 7.0, 2.0, 5.5, WHITE);
        let (st, at) = unsafe { STATE_AT[p] };
        if st == k + 1 && t - at < 20 {
            burst(e, S_BURST_SUNNY, mx, y + 5.0, 26.0, t - at, "", 10);
        }
    }
    // the flame at the foot: bigger with the state
    let sc = [0.55, 0.75, 0.95, 1.15][h.state];
    spr_c(e, S_FLAME + h.state, x - 12.0, y + 2.0, sc, WHITE);
}

/// One HUD drive chip: ink disc, charge ring, badge, pips; a sunny burst when it just fired.
fn hud_chip(e: &mut E, cx: f32, cy: f32, id: usize, lv: i32, charge: f32, fired: i32) {
    if fired >= 0 && fired < 10 {
        spr_rot(e, S_BURST16, cx, cy, 0.8 + 0.4 * (1.0 - fired as f32 / 10.0), fired as f32 * 4.0, WHITE);
    }
    spr_c(e, S_DISC, cx, cy, 26.8 / 32.0, INK);
    ring(e, cx, cy, 11.6, 3.2, charge, SUNNY, alpha(PALE, 0.55));
    let s = if fired >= 0 && fired < 6 { 1.25 - 0.25 * fired as f32 / 6.0 } else { 1.0 };
    spr_c(e, badge_of(id), cx, cy, 19.0 / 28.5 * s, WHITE);
    for k in 0..3 {
        let px = cx - 8.0 + k as f32 * 8.0;
        e.rect(px - 3.5, cy + 15.0, 7.0, 5.0, INK);
        if (k as i32) < lv {
            e.rect(px - 2.5, cy + 16.0, 5.0, 3.0, SUNNY);
        }
    }
}

/// Frames since this set's drive in slot k last fired (from the fire log), or -1.
fn fired_ago(v: &[i32; VW], owner: i32, id: i32) -> i32 {
    let n = v[V_LOG_N].max(0) as usize;
    let now = v[V_PLAY];
    let mut best = -1;
    for i in 0..LOG.min(n) {
        let e = &v[V_LOG + i * LOG_W..V_LOG + (i + 1) * LOG_W];
        if e[1] == id && e[2] == owner {
            let ago = now - e[0];
            if ago >= 0 && (best < 0 || ago < best) {
                best = ago;
            }
        }
    }
    best
}

/// A player's corner. `right`: anchored to the right edge (player 2).
unsafe fn corner(e: &mut E, v: &[i32; VW], p: usize, x0: f32, x1: f32) {
    let pbk = V_P0 + p * V_PW;
    let act = v[pbk + P_ACTIVE].clamp(0, 1) as usize;
    let set = if v[V_SHARED] != 0 { 0 } else { act };
    let ids = &v[pbk + P_DRIVES + set * SLOTS..pbk + P_DRIVES + (set + 1) * SLOTS];
    let n = ids.iter().filter(|&&d| (1..=POOL_LEN as i32).contains(&d)).count().max(1);
    let wrow = 30.0 * SLOTS as f32;
    let width = 160.0f32.max(wrow + 10.0).max(230.0);
    let x = if p == 0 { x0 } else { x1 - width };
    let y = 268.0;
    let _ = n;
    // tag and name
    let col = if p == 0 { S_FACE_CORAL } else { S_FACE_LAGOON };
    face(e, col, x, y, 26.0, 18.0, WHITE);
    e.text_ex(x + 13.0, y + 2.0, d(12), WHITE, ALIGN_CENTER, 0.0, if p == 0 { "P1" } else { "P2" });
    let fk = if act == 0 { v[pbk + P_MAIN] } else { v[pbk + P_PARTNER] };
    let name = ck_name(fk);
    let nw = tw(name, d(12)) + 14.0;
    face(e, S_FACE_WHITE, x + 30.0, y, nw, 18.0, WHITE);
    e.text_ex(x + 37.0, y + 2.0, d(12), INK, ALIGN_LEFT, 0.0, name);
    // the drive row
    for k in 0..SLOTS {
        let cx = x + 14.0 + k as f32 * 30.0;
        let cy = y + 38.0;
        let id = ids[k];
        if (1..=POOL_LEN as i32).contains(&id) {
            let lv = v[pbk + P_LEVELS + set * SLOTS + k];
            let ch = v[pbk + P_CHARGE + set * SLOTS + k] as f32 / 1000.0;
            let fired = fired_ago(v, (p * 2 + set) as i32, id);
            hud_chip(e, cx, cy, id as usize - 1, lv, ch, fired);
        } else {
            spr_c(e, S_DISC, cx, cy, 19.0 / 32.0, alpha(INK, 0.55));
            spr_c(e, S_DISC, cx, cy, 15.0 / 32.0, alpha(PALE, 0.35));
        }
    }
    // the active drive (Volley): a square badge after the row, its pips, D-pad right
    if v[pbk + P_ACT_ID] > 0 {
        let ax = x + 14.0 + SLOTS as f32 * 30.0 + 8.0;
        let ay = y + 38.0;
        let ready = v[pbk + P_ACT_READY] != 0;
        if ready {
            spr_rot(e, S_BURST_SUNNY, ax, ay, 0.62, (T as f32) * 0.5, WHITE);
        }
        face(e, S_FACE_INK, ax - 12.0, ay - 12.0, 24.0, 24.0, WHITE);
        spr_c(e, S_BADGE + 3 + 1, ax, ay, 17.0 / 28.5, WHITE);
        spr_c(e, if ready { S_DPAD_RIGHT_LIT } else { S_DPAD_RIGHT }, ax + 12.0, ay - 12.0, 0.75, WHITE);
        let need = v[pbk + P_PIPS_NEED].clamp(1, 4);
        for k in 0..need {
            let px = ax - (need - 1) as f32 * 4.0 + k as f32 * 8.0;
            spr_c(e, S_DISC, px, ay + 17.0, 6.0 / 32.0, if k < v[pbk + P_PIPS] { SUNNY } else { alpha(PALE, 0.5) });
        }
    }
    // Heat bar and the bench sticker
    let hy = y + 66.0;
    if let Some(h) = heat_of(v, p) {
        heat_bar(e, x + 14.0, hy, 122.0 * (h.cap / 150.0).min(1.7), &h, p);
    }
    let bench = if act == 0 { v[pbk + P_PARTNER] } else { v[pbk + P_MAIN] };
    if v[pbk + P_PARTNER_KIND] >= 0 || (0..26).contains(&bench) {
        let (bx, by) = (x + 30.0 + nw + 8.0, y - 2.0);
        let cd = v[pbk + P_TAG_CD];
        let mx = v[pbk + P_TAG_MAX].max(1);
        let ready = cd <= 0;
        face(e, if ready { S_FACE_WHITE } else { S_FACE_FADED }, bx, by, 92.0, 22.0, WHITE);
        if !ready {
            let f = 1.0 - (cd as f32 / mx as f32).clamp(0.0, 1.0);
            e.rect(bx + 2.0, by + 2.0, 88.0 * f, 18.0, alpha(WHITE, 0.9));
        }
        spr_c(e, S_HEAD, bx + 11.0, by + 11.0, 0.7, WHITE);
        let s = ck_name(bench);
        e.text_ex(bx + 21.0, by + 4.0, d(11), INK, ALIGN_LEFT, 56.0, s);
        spr_c(e, S_DPAD_DOWN, bx + 83.0, by + 11.0, 0.8, WHITE);
    }
}

/// The whole HUD.
pub unsafe fn draw(e: &mut E, v: &[i32; VW], ex: f32, petey: Option<&[i32; 12]>) {
    T += 1;
    let humans = v[V_HUMANS].clamp(1, 2) as usize;
    e.layer(LAYER_HUD, 5);
    corner(e, v, 0, 14.0 - ex, 0.0);
    if humans == 2 {
        corner(e, v, 1, 0.0, 626.0 + ex);
    }
    // top left: keystones (mini wedges, emblem only) and coins, then the shared stock pool
    let p = party_from_view(v);
    let (ks, kn) = party_keys(&p);
    let mut x = 14.0 - ex;
    for i in 0..kn {
        face(e, S_FACE_INK, x, 12.0, 30.0, 27.0, WHITE);
        spr_c(e, kem_of(ks[i]), x + 15.0, 25.5, 19.0 / 34.0, WHITE);
        x += 34.0;
    }
    if v[v3::V_HEAT_ON] == 1 {
        let mut s = Buf::new();
        s.i(v[v3::V_COINS]);
        let w = tw(s.str(), d(13)) + 28.0;
        face(e, S_FACE_WHITE, x + 4.0, 16.0, w, 19.0, WHITE);
        spr_c(e, S_COIN, x + 15.0, 25.5, 0.9, WHITE);
        e.text_ex(x + 25.0, 18.0, d(13), INK, ALIGN_LEFT, 0.0, s.str());
        x += w + 10.0;
    }
    // the pool: white dots, the last one coral on a burst
    let pool = v[V_POOL].max(0);
    let total = if humans == 2 { 5 } else { 4 };
    let px = x + 10.0;
    for i in 0..total.max(pool) {
        let cx = px + i as f32 * 18.0;
        if i < pool {
            if pool == 1 {
                let pulse = 1.0 + 0.15 * (sin(T as f32 * 0.105)).max(0.0);
                spr_rot(e, S_BURST_SUNNY, cx, 26.0, 0.42 * pulse, 8.0, WHITE);
                spr_c(e, S_STOCK_LAST, cx, 26.0, 0.85, WHITE);
            } else {
                spr_c(e, S_STOCK, cx, 26.0, 0.85, WHITE);
            }
        } else {
            spr_c(e, S_STOCK_SPENT, cx, 26.0, 0.85, WHITE);
        }
    }
    // fight label under the pool
    let mut s = Buf::new();
    s.s("Fight ").i(v[V_FIGHT] + 1).s(" of ").i(v[V_FIGHTS]);
    e.text_ex(14.0 - ex, 44.0, bs(12), WHITE, ALIGN_LEFT, 0.0, s.str());
    // foes, top right: an elite or boss plate with its drives
    let kind = v[V_FIGHT_KIND];
    let mut fy = 12.0;
    if petey.is_none() {
        for sl in 0..4 {
            let bb = V_S0 + sl * V_SW;
            if v[bb + S_FOE] == 0 || v[bb + S_ALIVE] == 0 {
                continue;
            }
            let big = v[bb + S_DRIVES] > 0;
            let w = if big { 150.0 } else { 90.0 };
            let h = if big { 38.0 } else { 28.0 };
            let fx = 624.0 + ex - w;
            face(e, S_FACE_INK, fx, fy, w, h, WHITE);
            let label = if kind == FK_BOSS { "Boss" } else if big { "Elite" } else { "Foe" };
            e.text_ex(fx + 10.0, fy + 4.0, d(13), if big { SUNNY } else { WHITE }, ALIGN_LEFT, 0.0, label);
            if big {
                for k in 0..2 {
                    let id = v[bb + S_DRIVES + k];
                    if (1..=POOL_LEN as i32).contains(&id) {
                        let cx = fx + 70.0 + k as f32 * 26.0;
                        spr_c(e, S_DISC, cx, fy + 19.0, 24.0 / 32.0, INK);
                        ring(e, cx, fy + 19.0, 10.0, 2.6, v[bb + S_CHARGE + k] as f32 / 1000.0, SUNNY, alpha(PALE, 0.4));
                        spr_c(e, badge_of(id as usize - 1), cx, fy + 19.0, 16.0 / 28.5, WHITE);
                    }
                }
            }
            // statuses: burning dots, chilled, marked
            let mut sx = fx + w - 10.0;
            for _ in 0..v[bb + S_BURN].clamp(0, 4) {
                spr_c(e, S_FLAME + 2, sx, fy + h - 2.0, 0.35, WHITE);
                sx -= 9.0;
            }
            if v[bb + S_CHILL] > 0 {
                spr_c(e, S_BADGE + 2 * 3, sx, fy + h - 2.0, 0.3, WHITE);
                sx -= 10.0;
            }
            if v[bb + S_MARK] > 0 {
                spr_c(e, S_BURST_CORAL, sx, fy + h - 2.0, 0.22, WHITE);
            }
            fy += h + 8.0;
        }
    }
    if let Some(pv) = petey {
        boss_bar(e, pv);
    }
    moments(e, v, ex);
    if humans == 2 && crate::pad_present(1) == 0 {
        sticker(e, S_FACE_LAGOON, 180.0, 150.0, 280.0, 34.0, -1.0, WHITE, |e| {
            e.text_ex(320.0, 158.0, d(15), WHITE, ALIGN_CENTER, 0.0, "P2: connect a second controller");
        });
    }
}

/// The boss bar (Petey): top centre, 60 px right of centre, cut in quarters.
unsafe fn boss_bar(e: &mut E, p: &[i32; 12]) {
    let (x, y, w) = (224.0, 15.0, 340.0);
    spr_c(e, S_DISC, 201.0, 21.0, 34.0 / 32.0, INK);
    spr_c(e, S_DISC, 201.0, 21.0, 28.0 / 32.0, CORAL);
    spr_c(e, S_STAR_ON, 201.0, 21.0, 1.4, WHITE);
    e.rect(x - 3.0, y - 3.0, w + 6.0, 21.0, INK);
    e.rect(x, y, w, 15.0, TRACK);
    let f = (p[1] as f32 / p[2].max(1) as f32).clamp(0.0, 1.0);
    e.rect(x, y, w * f, 15.0, if p[7] != 0 { CORAL } else { SUNNY });
    for q in 1..4 {
        e.rect(x + w * q as f32 / 4.0 - 1.0, y, 2.0, 15.0, INK);
    }
    let name = if p[11] != 0 { "Petey Piranha" } else { "Petey (stand-in)" };
    text_edge(e, x + 2.0, y + 19.0, d(14), WHITE, ALIGN_LEFT, 1.5, name);
    let state = if p[6] == 7 { if p[8] == 2 { "Cage broken: partner free" } else { "Defeated" } } else if p[6] == 6 { "Stunned" } else if p[7] != 0 { "Phase 2" } else { "" };
    if !state.is_empty() {
        text_edge(e, x + w, y + 19.0, bs(13), SUNNY, ALIGN_RIGHT, 1.5, state);
    }
    // the two cages
    let cw = w * 0.5 - 6.0;
    for k in 0..2 {
        let cx = x + k as f32 * (cw + 12.0);
        e.rect(cx, y + 40.0, cw, 6.0, INK);
        let cf = (p[3 + k] as f32 / p[5].max(1) as f32).clamp(0.0, 1.0);
        e.rect(cx + 1.0, y + 41.0, (cw - 2.0) * cf, 4.0, PALE);
    }
}

/// After a pick, a sticker says what went where (fades after 2.5 s).
pub unsafe fn picked(e: &mut E, v: &[i32; VW]) {
    let since = v[V_SINCE_PICK];
    if since >= 150 {
        return;
    }
    let a = 1.0 - ease((since as f32 - 110.0) / 40.0);
    for p in 0..v[V_HUMANS].clamp(1, 2) as usize {
        let pbk = V_P0 + p * V_PW;
        let id = v[pbk + P_LAST_PICK];
        if id < 0 {
            continue;
        }
        let mut s = Buf::new();
        let mut kid = None;
        if id == OFFER_STOCK {
            s.s("+1 stock in the pool");
        } else {
            let dr = &POOL[id.clamp(0, POOL_LEN as i32 - 1) as usize];
            if dr.kind == K_KEYSTONE {
                s.s(dr.name).s(" is the rule");
                kid = Some(kem_of(id as usize));
            } else {
                let w = v[pbk + P_LAST_WEARER].clamp(0, 1);
                s.s(dr.name).s(" goes to ").s(ck_name(if w == 0 { v[pbk + P_MAIN] } else { v[pbk + P_PARTNER] }));
                kid = Some(badge_of(id as usize));
            }
        }
        let w = tw(s.str(), d(16)) + 60.0;
        let (x, y) = (320.0 - w * 0.5, 76.0 + p as f32 * 44.0);
        let (sc, _) = enter(since);
        let _ = sc;
        sticker(e, S_FACE_WHITE, x, y, w, 36.0, if p == 0 { -1.5 } else { 1.5 }, alpha(WHITE, a), |e| {
            if let Some(k) = kid {
                spr_c(e, k, x + 22.0, y + 18.0, if k >= S_KEM { 0.8 } else { 1.1 }, alpha(WHITE, a));
            }
            e.text_ex(x + 42.0, y + 9.0, d(16), alpha(INK, a), ALIGN_LEFT, 0.0, s.str());
        });
        if v[V_HUMANS] == 2 {
            face(e, if p == 0 { S_FACE_CORAL } else { S_FACE_LAGOON }, x - 10.0, y - 8.0, 26.0, 18.0, alpha(WHITE, a));
            e.text_ex(x + 3.0, y - 6.0, d(12), alpha(WHITE, a), ALIGN_CENTER, 0.0, if p == 0 { "P1" } else { "P2" });
        }
    }
}

/// The loud moments from the event ring: cash-out ("Cashed out!" burst and coins flying to the purse), a coin
/// payout, a technique spark, a boss break. Each shows for 40 frames after its frame.
unsafe fn moments(e: &mut E, v: &[i32; VW], ex: f32) {
    let n = v[V_EV_N].max(0) as usize;
    let now = v[V_PLAY];
    for i in 0..EV.min(n) {
        let ev = &v[V_EV + i * EV_W..V_EV + (i + 1) * EV_W];
        let age = now - ev[0];
        if !(0..40).contains(&age) {
            continue;
        }
        let who = ev[2].clamp(0, 1) as usize;
        let cx = if who == 0 { 80.0 - ex } else { 560.0 + ex };
        match ev[1] {
            EV_CASHOUT => {
                burst(e, S_BURST_SUNNY, cx, 300.0, 84.0, age, "Cashed out!", 11);
                for k in 0..3 {
                    let t = ((age - k * 4) as f32 / 24.0).clamp(0.0, 1.0);
                    if t > 0.0 && t < 1.0 {
                        let (x, y) = (cx + (60.0 - ex - cx) * t, 300.0 + (26.0 - 300.0) * t - 60.0 * sin(t * 3.14159));
                        spr_c(e, S_COIN, x, y, 1.3, WHITE);
                    }
                }
            }
            EV_PAY | EV_COINS => {
                let mut s = Buf::new();
                s.s("+").i(ev[3]);
                text_edge(e, 100.0 - ex, 40.0 - age as f32 * 0.4, d(16), SUNNY, ALIGN_LEFT, 1.6, s.str());
            }
            EV_SPARK => {
                let word = match ev[3] { 1 => "L-cancel!", 2 => "Wavedash!", 3 => "Tech!", _ => "Offstage!" };
                let px = f32::from_bits(v[V_P0 + who * V_PW + P_X] as u32);
                let _ = px;
                burst(e, if who == 0 { S_BURST_CORAL } else { S_BURST_LAGOON }, cx + 40.0, 250.0, 64.0, age, word, 10);
            }
            EV_STATE_UP => {
                if ev[3] == 3 {
                    burst(e, S_BURST_CORAL, cx + 20.0, 240.0, 80.0, age, "Blazing!", 13);
                }
            }
            EV_BREAK => {
                burst(e, S_BURST_CORAL, 400.0, 30.0, 80.0, age, "Break!", 14);
            }
            _ => {}
        }
    }
}
