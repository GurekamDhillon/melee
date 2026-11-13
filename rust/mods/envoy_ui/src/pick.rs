//! The pick of three (R2) and the keystone pick (K1) in the Party pack look, over the frozen fight. The sim owns the
//! input (left / right moves, A takes, then left / right picks the fighter); this only draws what the view says.

use crate::camp::fighter_sticker;
use crate::kit::*;
use crate::party::*;
use crate::{ck_name, Buf, VW};
use envoy_data::view::*;
use envoy_data::*;
use gd_ui_enc::*;

static mut T: i32 = 0;
static mut OPEN_AT: i32 = 0;
static mut DONE_AT: [i32; 2] = [-1; 2];
static mut LAST_RW: i32 = 0;

fn level_of(v: &[i32; VW], p: usize, f: usize, id: i32) -> i32 {
    let pbk = V_P0 + p * V_PW;
    (0..SLOTS).find(|&k| v[pbk + P_DRIVES + f * SLOTS + k] == id + 1).map_or(0, |k| v[pbk + P_LEVELS + f * SLOTS + k])
}

/// Word-wrap `s` into lines of at most `w` px (role), drawn from (x, y), `lh` apart. Returns the lines drawn.
pub fn wrap(e: &mut E, x: f32, y: f32, w: f32, role: i32, lh: f32, rgba: u32, align: i32, s: &str, max_lines: usize) -> usize {
    let bytes = s.as_bytes();
    let mut start = 0;
    let mut line = 0;
    while start < bytes.len() && line < max_lines {
        // the longest run of words from `start` that fits
        let mut end = start;
        let mut last_ok = start;
        while end <= bytes.len() {
            if end == bytes.len() || bytes[end] == b' ' {
                let piece = unsafe { core::str::from_utf8_unchecked(&bytes[start..end]) };
                if tw(piece, role) <= w || last_ok == start {
                    last_ok = end;
                } else {
                    break;
                }
                if end == bytes.len() {
                    break;
                }
            }
            end += 1;
        }
        if last_ok == start {
            last_ok = bytes.len();
        }
        let piece = unsafe { core::str::from_utf8_unchecked(&bytes[start..last_ok]) };
        e.text_ex(x, y + line as f32 * lh, role, rgba, align, if line + 1 == max_lines { w } else { 0.0 }, piece);
        line += 1;
        start = last_ok;
        while start < bytes.len() && bytes[start] == b' ' {
            start += 1;
        }
    }
    line
}

/// A drive offer card (w x h at x, y): badge, name, family, rule, what it does for each fighter.
fn offer(e: &mut E, v: &[i32; VW], p: usize, i: usize, x: f32, y: f32, w: f32, h: f32, on: bool, faded: bool) {
    let pbk = V_P0 + p * V_PW;
    let id = v[pbk + P_OFFERS + i];
    let deg = if on { 0.0 } else { [-1.5, 1.0, -0.8][i] };
    let y = if on { y - 8.0 } else { y };
    let a = if faded { 0.34 } else { 1.0 };
    let face_id = if on { S_FACE_HOVER } else { S_FACE_WHITE };
    sticker(e, face_id, x, y, w, h, deg, alpha(WHITE, a), |e| {
        let cx = x + w * 0.5;
        if id == OFFER_STOCK || !(0..POOL_LEN as i32).contains(&id) {
            spr_c(e, S_STOCK, cx, y + 40.0, 2.4, alpha(WHITE, a));
            e.text_ex(cx, y + 74.0, d(19), alpha(INK, a), ALIGN_CENTER, w - 12.0, "+1 stock");
            let mut s = Buf::new();
            s.s("Pool ").i(v[V_POOL]).s(" to ").i(v[V_POOL] + 1);
            e.text_ex(cx, y + 100.0, bs(13), alpha(LILAC, a), ALIGN_CENTER, w - 12.0, s.str());
            return;
        }
        let dr = &POOL[id as usize];
        let keystone = dr.kind == K_KEYSTONE;
        if keystone {
            spr_c(e, kem_of(id as usize), cx, y + 34.0, 1.3, alpha(WHITE, a));
        } else {
            spr_c(e, badge_of(id as usize), cx, y + 32.0, 38.0 / 28.5, alpha(WHITE, a));
        }
        e.text_ex(cx, y + 66.0, d(17), alpha(INK, a), ALIGN_CENTER, w - 10.0, dr.name);
        let fam = if keystone { "Keystone" } else { crate::party::FAM_WORD[FAM_BADGE.get(dr.family as usize).copied().unwrap_or(5)] };
        e.text_ex(cx, y + 88.0, bs(11), alpha(LILAC, a), ALIGN_CENTER, 0.0, fam);
        wrap(e, cx, y + 106.0, w - 16.0, bs(12), 14.0, alpha(INK, a), ALIGN_CENTER, dr.rule, 5);
        if !keystone {
            let sets = if v[V_SHARED] != 0 { 1 } else { 2 };
            for f in 0..sets {
                let lv = level_of(v, p, f, id);
                let ok = v[pbk + P_CAN_TAKE] >> (i * 2 + f) & 1 != 0;
                let fy = y + h - 22.0 - (sets - 1 - f) as f32 * 18.0;
                let who = if sets == 1 { "Pair" } else { ck_name(if f == 0 { v[pbk + P_MAIN] } else { v[pbk + P_PARTNER] }) };
                let mut s = Buf::new();
                s.s(who).s(": ");
                if !ok {
                    s.s(if lv >= 3 { "top level" } else { "row full" });
                } else if lv == 0 {
                    s.s("new");
                } else {
                    s.s("level ").i(lv + 1);
                }
                let col = if !ok { DEEP } else if lv > 0 { CORAL } else { INK };
                e.text_ex(cx, fy, bs(11), alpha(col, a), ALIGN_CENTER, w - 10.0, s.str());
                if ok && lv > 0 && on {
                    burst(e, S_BURST_CORAL, x + w - 6.0, y + 6.0, 46.0, unsafe { T - OPEN_AT }, "Level up!", 9);
                }
            }
        } else {
            let pr = PRICES[dr.price as usize];
            let mut s = Buf::new();
            s.s("Price: ").s(pr);
            e.text_ex(cx, y + h - 24.0, d(12), alpha(CORAL, a), ALIGN_CENTER, w - 10.0, s.str());
        }
    });
}

/// What fills a drive's charge, in words ("Every 35% you deal"), or "" when the rule says it.
fn trigger(dr: &Drive, out: &mut Buf) {
    let n = dr.need / 1000;
    match dr.charge {
        C_DEALT => { out.s("Every ").i(n).s("% you deal: "); }
        C_AERIAL_DEALT => { out.s("Every ").i(n).s("% from aerials: "); }
        C_CHILLED_DEALT => { out.s("Every ").i(n).s("% to Chilled foes: "); }
        C_OFFSTAGE_DEALT => { out.s("Every ").i(n).s("% offstage: "); }
        C_TIME => { out.s("Every ").i(n).s(" s: "); }
        _ => {}
    }
}

/// One offer as a D2 row: the round family badge over the tag's left edge, name, tier and family, the trigger and
/// rule, and what it does for each fighter (new / level n / row full). Keystones: emblem and price.
fn offer_row(e: &mut E, v: &[i32; VW], p: usize, i: usize, x: f32, y: f32, w: f32, on: bool, faded: bool, a0: f32) {
    let pbk = V_P0 + p * V_PW;
    let id = v[pbk + P_OFFERS + i];
    let a = if faded { 0.34 } else { a0 };
    let deg = if on { 0.0 } else { [-0.8, 0.6, -0.5][i] };
    let x = if on { x + 6.0 } else { x };
    let h = 70.0;
    let (bx, by) = (x + 2.0, y + 32.0);
    let stock = id == OFFER_STOCK || !(0..POOL_LEN as i32).contains(&id);
    // level-up: rays behind the badge, loud
    let mut lvl_up = false;
    if !stock && POOL[id as usize].kind != K_KEYSTONE {
        let sets = if v[V_SHARED] != 0 { 1 } else { 2 };
        for f in 0..sets {
            if level_of(v, p, f, id) > 0 && v[pbk + P_CAN_TAKE] >> (i * 2 + f) & 1 != 0 {
                lvl_up = true;
            }
        }
    }
    if lvl_up && !faded {
        spr_rot(e, S_RAYS, bx, by, if on { 1.2 } else { 0.8 }, unsafe { T } as f32 * 0.6, alpha(SUNNY, if on { 0.9 } else { 0.45 } * a));
    }
    if !stock && !faded {
        rarity_fx(POOL[id as usize].tier);
    }
    sticker(e, if on { S_FACE_HOVER } else { S_FACE_WHITE }, x + 18.0, y, w - 18.0, h, deg, alpha(WHITE, a), |e| {
        let tx = x + 50.0;
        let tr = x + w - 10.0;
        if stock {
            e.text_ex(tx, y + 6.0, d(17), alpha(INK, a), ALIGN_LEFT, 0.0, "+1 stock");
            let mut s = Buf::new();
            s.s("One stock back in the shared pool: ").i(v[V_POOL]).s(" to ").i(v[V_POOL] + 1).s(".");
            wrap(e, tx, y + 28.0, w - 64.0, bs(12), 14.0, alpha(INK, a), ALIGN_LEFT, s.str(), 2);
            return;
        }
        let dr = &POOL[id as usize];
        let keystone = dr.kind == K_KEYSTONE;
        e.text_ex(tx, y + 5.0, d(17), alpha(INK, a), ALIGN_LEFT, w - 150.0, dr.name);
        let mut s = Buf::new();
        if keystone {
            s.s("Keystone");
        } else {
            s.s(["Common", "Rare", "Epic"][(dr.tier as usize).min(2)]).s(" ").s(fam_word(id as usize));
        }
        e.text_ex(tr, y + 9.0, bs(11), alpha(LILAC, a), ALIGN_RIGHT, 0.0, s.str());
        let mut r = Buf::new();
        if !keystone && !dr.rule.starts_with("Every") {
            trigger(dr, &mut r);
        }
        r.s(dr.rule);
        wrap(e, tx, y + 26.0, w - 66.0, bs(12), 13.5, alpha(INK, a), ALIGN_LEFT, r.str(), 2);
        let mut s = Buf::new();
        let mut col = LILAC;
        if keystone {
            s.s("Price: ").s(PRICES[dr.price as usize]);
            col = CORAL;
        } else {
            let sets = if v[V_SHARED] != 0 { 1 } else { 2 };
            for f in 0..sets {
                let lv = level_of(v, p, f, id);
                let ok = v[pbk + P_CAN_TAKE] >> (i * 2 + f) & 1 != 0;
                if f > 0 {
                    s.s("   ");
                }
                s.s(if sets == 1 { "Pair" } else { ck_name(if f == 0 { v[pbk + P_MAIN] } else { v[pbk + P_PARTNER] }) }).s(": ");
                if !ok {
                    s.s(if lv >= 3 { "top level" } else { "row full" });
                    col = DEEP;
                } else if lv == 0 {
                    s.s("new");
                } else {
                    s.s("level ").i(lv + 1);
                    col = CORAL;
                }
            }
        }
        e.text_ex(tx, y + 50.0, bs(11), alpha(col, a), ALIGN_LEFT, w - 66.0, s.str());
    });
    // the badge over the tag's left edge, with its tier stars
    spr_c(e, S_DISC, bx, by, 58.0 / 32.0, alpha(INK, a));
    if stock {
        spr_c(e, S_STOCK, bx, by, 2.2, alpha(WHITE, a));
    } else if POOL[id as usize].kind == K_KEYSTONE {
        spr_c(e, kem_of(id as usize), bx, by, 50.0 / 34.0, alpha(WHITE, a));
    } else {
        spr_c(e, badge_of(id as usize), bx, by, 52.0 / 28.5, alpha(WHITE, a));
        let tier = POOL[id as usize].tier.min(2) as usize;
        face(e, S_FACE_WHITE, bx - 22.0, by + 22.0, 44.0, 13.0, alpha(WHITE, a));
        for k in 0..3 {
            spr_c(e, if k <= tier { S_STAR_ON } else { S_STAR_OFF }, bx - 12.0 + k as f32 * 12.0, by + 28.5, 0.5, alpha(WHITE, a));
        }
    }
    if lvl_up && on {
        burst(e, S_BURST_CORAL, x + w - 14.0, y - 2.0, 54.0, unsafe { T - OPEN_AT }, "Level up!", 10);
    }
}

/// envoy-fx.md: Epic offers shimmer (foil), Unique ones (tier 3) get the holographic face.
fn rarity_fx(tier: u8) {
    match tier {
        2 => face_fx_next(FX_FOIL, 0xC9BF_F5FF),
        t if t >= 3 => face_fx_next(FX_HOLO, 0xFFFF_FFFF),
        _ => {}
    }
}

/// One player's side: three cards, then (step 1) the fighters; `x0`, `w0` the side's box.
unsafe fn side(e: &mut E, v: &[i32; VW], p: usize, x0: f32, w0: f32, coop: bool) {
    let pbk = V_P0 + p * V_PW;
    let done = v[pbk + P_RW_DONE] != 0;
    let step = v[pbk + P_STEP];
    let cur = v[pbk + P_CURSOR].clamp(0, 2) as usize;
    if v[V_RW] == 2 && !coop {
        // K1: three wedges
        for i in 0..3 {
            let (_, a) = enter(T - OPEN_AT - i as i32 * 3);
            if a <= 0.0 {
                continue;
            }
            let id = v[pbk + P_OFFERS + i];
            let on = i == cur && !done;
            let faded = done && i != cur;
            let (wx, wy) = (28.0 + 199.0 * i as f32, if i == 1 { 106.0 } else { 118.0 } - if on { 8.0 } else { 0.0 });
            let deg = if on { 0.0 } else if done && faded { [-24.0, 0.0, 24.0][i] } else { [-4.0, 0.0, 4.0][i] };
            let al = if faded { 0.3 } else { a };
            let (cx, cy) = (wx + 93.0, wy + 138.0);
            rot(e, deg, cx, cy);
            spr(e, S_WEDGE + on as usize, wx, wy, 186.0, 276.0, alpha(WHITE, al));
            let ink = if on { INK } else { WHITE };
            if (0..POOL_LEN as i32).contains(&id) {
                let dr = &POOL[id as usize];
                spr_c(e, kem_of(id as usize), cx, wy + 2.0, 62.0 / 34.0, alpha(WHITE, al));
                wrap(e, cx, wy + 44.0, 150.0, d(24), 26.0, alpha(if on { INK } else { SUNNY }, al), ALIGN_CENTER, dr.name, 2);
                wrap(e, cx, wy + 112.0, 128.0, bs(15), 18.0, alpha(ink, al), ALIGN_CENTER, dr.rule, 5);
                let mut s = Buf::new();
                s.s("Price: ").s(PRICES[dr.price as usize]);
                e.text_ex(cx, wy + 236.0, d(13), alpha(if on { DEEP } else { CORAL }, al), ALIGN_CENTER, 150.0, s.str());
            } else {
                e.text_ex(cx, wy + 100.0, d(22), alpha(ink, al), ALIGN_CENTER, 0.0, "+1 stock");
            }
            unrot(e);
            if on {
                spr(e, S_PTRDOWN, wx + 80.0, wy - 44.0, 0.0, 0.0, WHITE);
            }
        }
        if done {
            if DONE_AT[p] < 0 {
                DONE_AT[p] = T;
            }
            let wx = 28.0 + 199.0 * cur as f32;
            burst(e, S_BURST_CORAL, wx + 170.0, 110.0, 90.0, T - DONE_AT[p], "Set!", 22);
        } else {
            DONE_AT[p] = -1;
        }
        return;
    }
    // R2 + D2: three rows (round badge, wide paper tag), Skip as a fourth row (drives)
    let rx = x0 + 26.0;
    let rw = w0 - 30.0;
    let cur4 = v[pbk + P_CURSOR].clamp(0, 3) as usize;
    for i in 0..3 {
        let (_, a) = enter(T - OPEN_AT - i as i32 * 3);
        if a <= 0.0 {
            continue;
        }
        let y = 50.0 + i as f32 * 76.0;
        let on = i == cur4 && !done;
        let faded = (step >= 1 || done) && i != cur;
        offer_row(e, v, p, i, rx, y, rw, on, faded, a);
        if on && step == 0 {
            spr(e, S_PTR + p, rx - 58.0, y + 20.0, 0.0, 0.0, WHITE);
        }
    }
    if v[V_RW] == 1 && step == 0 {
        let on = cur4 == 3 && !done && step == 0;
        let (sx, sy) = (rx + 8.0, 282.0);
        sticker(e, if on { S_FACE_HOVER } else { S_FACE_WHITE }, sx, sy, 86.0, 24.0, if on { 0.0 } else { -1.0 }, WHITE, |e| {
            e.text_ex(sx + 43.0, sy + 4.0, d(14), INK, ALIGN_CENTER, 0.0, "Skip");
        });
        if on {
            spr(e, S_PTR + p, sx - 40.0, sy - 2.0, 0.0, 0.0, WHITE);
        }
    }
    // the fighters (step 1: who wears it)
    let party = party_from_view(v);
    let sets = if v[V_SHARED] != 0 { 1 } else { 2 };
    let fw = if coop { (w0 - 8.0) / 2.0 } else { 140.0 };
    for f in 0..sets {
        let fx = if coop { x0 + f as f32 * (fw + 8.0) } else { x0 + w0 * 0.5 - 146.0 + f as f32 * 152.0 };
        let fy = 318.0;
        let on = step == 1 && v[pbk + P_FCURSOR] == f as i32 && !done;
        fighter_sticker_w(e, fx, fy, fw, if f == 0 { -0.6 } else { 0.5 }, &party, p * 2 + f, f == 0 && coop, on);
        if on {
            spr(e, S_PTRDOWN + p, fx + fw * 0.5 - 13.0, fy - 26.0, 0.0, 0.0, WHITE);
        }
        // step 2: the row is full, the slot cursor walks it (the drive there is replaced)
        if step == 2 && v[pbk + P_FCURSOR] == f as i32 && !done {
            let k = v[pbk + P_SCURSOR].clamp(0, SLOTS as i32 - 1) as f32;
            let stepw = if fw >= 140.0 { 26.0 } else { ((fw - 12.0) / SLOTS as f32).min(26.0) };
            let cx = fx + if fw >= 140.0 { 17.0 } else { 6.0 + stepw * 0.5 } + k * stepw;
            ring(e, cx, fy + 48.0, 13.0, 3.2, 1.0, SUNNY, SUNNY);
            spr(e, S_PTRDOWN + p, cx - 13.0, fy + 8.0, 0.0, 0.0, WHITE);
            face(e, S_FACE_CORAL, fx + fw - 64.0, fy - 12.0, 62.0, 18.0, WHITE);
            e.text_ex(fx + fw - 33.0, fy - 10.0, d(11), WHITE, ALIGN_CENTER, 0.0, "Replace");
        }
    }
    if done {
        if DONE_AT[p] < 0 {
            DONE_AT[p] = T;
        }
        let tx = x0 + w0 * 0.5;
        burst(e, S_BURST_CORAL, tx, 200.0, 90.0, T - DONE_AT[p], "Set!", 22);
    } else {
        DONE_AT[p] = -1;
    }
}

/// A fighter sticker squeezed to `w` (co-op halves); `on`: sunny.
fn fighter_sticker_w(e: &mut E, x: f32, y: f32, w: f32, deg: f32, p: &Party, set: usize, first: bool, on: bool) {
    if w >= 140.0 && !on {
        fighter_sticker(e, x, y, deg, p, set, first, WHITE);
        return;
    }
    sticker(e, if on { S_FACE_HOVER } else { S_FACE_WHITE }, x, y, w, 80.0, if on { 0.0 } else { deg }, WHITE, |e| {
        spr_c(e, S_HEAD, x + 16.0, y + 14.0, 1.0, WHITE);
        e.text_ex(x + 31.0, y + 6.0, d(14), INK, ALIGN_LEFT, w - 36.0, ck_name(p.pairs[set]));
        let step = ((w - 12.0) / SLOTS as f32).min(26.0);
        for k in 0..SLOTS {
            let cx = x + 6.0 + step * 0.5 + k as f32 * step;
            let id = p.ids[set][k];
            if (1..=POOL_LEN as i32).contains(&id) {
                chip(e, cx, y + 48.0, id as usize - 1, p.lvs[set][k], 19.0);
            } else {
                spr_c(e, S_DISC, cx, y + 48.0, 17.0 / 32.0, alpha(PALE, 0.5));
            }
        }
    });
}

/// The pick screen (V_RW 1 drives, 2 keystones). `ex`: extra width beyond the 4:3 band.
pub unsafe fn draw(e: &mut E, v: &[i32; VW], ex: f32) {
    T += 1;
    if LAST_RW != v[V_RW] {
        LAST_RW = v[V_RW];
        OPEN_AT = T;
    }
    let keys = v[V_RW] == 2;
    let coop = v[V_HUMANS] == 2;
    e.layer(LAYER_MODAL, 0);
    e.rect(-ex, 0.0, 640.0 + 2.0 * ex, 480.0, 0x1A10_4AF2);
    if keys {
        spr_rot(e, S_RAYS, 320.0, 200.0, 2.6, T as f32 * 0.07, alpha(WHITE, 0.10));
    }
    // sessions: the banked picks one after another ("Pick 1 of 3")
    let mut tb = Buf::new();
    if keys {
        tb.s("Choose a keystone");
    } else if v[V_BANK_N] > 1 {
        tb.s(if v[V_BANK_KIND] == RK_LEVEL { "Level-up: reward " } else { "Reward " }).i(v[V_BANK_I].max(1)).s(" of ").i(v[V_BANK_N]);
    } else {
        tb.s("Pick a drive");
    }
    let centre = coop || !keys;
    text_edge(e, if centre { 320.0 } else { 16.0 }, 6.0, d(24), WHITE, if centre { ALIGN_CENTER } else { ALIGN_LEFT }, 2.2, tb.str());
    if !coop && keys {
        let sub = if keys { "One rule for the whole run." } else { "One of three, for one of your two fighters." };
        e.text_ex(22.0, 36.0, bs(13), WHITE, ALIGN_LEFT, 0.0, sub);
    }
    if v[V_TIMER] > 0 {
        let secs = (v[V_TIMER] + 59) / 60;
        let (tx, ty) = (606.0 + ex.min(20.0), 26.0);
        if secs <= 5 {
            spr_rot(e, S_BURST_CORAL, tx, ty, 0.95, 8.0, WHITE);
        } else {
            spr_c(e, S_DISC, tx, ty, 38.0 / 32.0, WHITE);
            ring(e, tx, ty, 17.0, 3.0, v[V_TIMER] as f32 / v[V_RW_TIME].max(1) as f32, CORAL, alpha(PALE, 0.4));
        }
        let mut s = Buf::new();
        s.i(secs);
        e.text_ex(tx, ty - 9.0, d(16), if secs <= 5 { WHITE } else { INK }, ALIGN_CENTER, 0.0, s.str());
    }
    if coop {
        // the seam
        for k in 0..20 {
            e.rect(318.0, 52.0 + k as f32 * 18.0, 4.0, 10.0, alpha(WHITE, 0.5));
        }
        for p in 0..2 {
            let x0 = 12.0 + p as f32 * 320.0;
            face(e, if p == 0 { S_FACE_CORAL } else { S_FACE_LAGOON }, x0, 30.0, 36.0, 22.0, WHITE);
            e.text_ex(x0 + 18.0, 32.0, d(15), WHITE, ALIGN_CENTER, 0.0, if p == 0 { "P1" } else { "P2" });
            side(e, v, p, x0, 296.0, true);
        }
    } else {
        if keys {
            side(e, v, 0, 40.0, 560.0, false);
        } else {
            side(e, v, 0, 150.0, 350.0, false);
        }
    }
    // strip and hints
    let pbk = V_P0;
    let cur = v[pbk + P_CURSOR].clamp(0, 2) as usize;
    let id = v[pbk + P_OFFERS + cur];
    let mut s = Buf::new();
    if v[pbk + P_CURSOR] == 3 && v[pbk + P_RW_DONE] == 0 {
        s.s("Skip: no drive this time.");
    } else if v[pbk + P_RW_DONE] != 0 {
        if coop { s.s("Waiting for the other player.") } else { s.s("Taken.") };
    } else if id == OFFER_STOCK {
        s.s("One stock back in the shared pool.");
    } else if (0..POOL_LEN as i32).contains(&id) {
        let dr = &POOL[id as usize];
        if v[pbk + P_STEP] == 2 {
            let f = v[pbk + P_FCURSOR].clamp(0, 1) as usize;
            let k = v[pbk + P_SCURSOR].clamp(0, SLOTS as i32 - 1) as usize;
            let old = v[pbk + P_DRIVES + f * SLOTS + k];
            s.s("Replace ").s(if old > 0 { POOL[old as usize - 1].name } else { "the empty slot" }).s(" with ").s(dr.name).s(" for good?");
        } else if v[pbk + P_STEP] == 1 {
            let f = v[pbk + P_FCURSOR].clamp(0, 1);
            s.s("Who wears ").s(dr.name).s("? ").s(ck_name(if f == 0 { v[pbk + P_MAIN] } else { v[pbk + P_PARTNER] })).s(".");
        } else if dr.kind == K_KEYSTONE {
            s.s(dr.name).s(" is the rule for the whole run.");
        } else {
            s.s(dr.name).s(": ").s(dr.rule);
        }
    }
    if !coop {
        strip(e, 24.0, 590.0, s.str());
    }
    let step1 = (0..v[V_HUMANS].clamp(1, 2) as usize).any(|p| v[V_P0 + p * V_PW + P_STEP] == 1);
    let step2 = (0..v[V_HUMANS].clamp(1, 2) as usize).any(|p| v[V_P0 + p * V_PW + P_STEP] == 2);
    if confirm_up(v) {
        // the replace confirm draws its own hint line over this screen: one footer, never two
    } else if step2 {
        hints(e, &[("Left / Right", "Choose a slot"), ("A", "Replace"), ("B", "Back")]);
    } else if step1 {
        hints(e, &[("Left / Right", "Choose a fighter"), ("A", "Take"), ("B", "Back")]);
    } else {
        hints(e, &[(if keys && !coop { "Left / Right" } else { "Up / Down" }, "Choose"), ("A", "Take it")]);
    }
}

/// A replace / scrap confirm is waiting for some player (`confirm` draws it and the only hint line).
pub fn confirm_up(v: &[i32; VW]) -> bool {
    (0..v[V_HUMANS].clamp(1, 2) as usize).any(|p| v[V_P0 + p * V_PW + P_CONFIRM] != 0)
}

/// The replace confirm (any screen that overwrites a slot: the pick, the shop, a level-up token): "Replace X with Y
/// for good?", Keep first (B keeps). One card per player that has one waiting, over that player's side.
pub unsafe fn confirm(e: &mut E, v: &[i32; VW]) {
    let humans = v[V_HUMANS].clamp(1, 2) as usize;
    for p in 0..humans {
        let b = V_P0 + p * V_PW;
        if v[b + P_CONFIRM] == 0 {
            continue;
        }
        let old = v[b + P_CONFIRM_OLD];
        let new = v[b + P_CONFIRM_NEW];
        let name = |id: i32| if (1..=POOL_LEN as i32).contains(&id) { POOL[id as usize - 1].name } else { "?" };
        let w = if humans == 2 { 290.0 } else { 380.0 };
        let x = if humans == 2 { 18.0 + p as f32 * 314.0 } else { 130.0 };
        let y = 150.0;
        e.rect(-200.0, -20.0, 1040.0, 520.0, 0x0C06_2A90);
        sticker(e, S_FACE_WHITE, x, y, w, 150.0, -0.5, WHITE, |e| {
            let mut s = Buf::new();
            let at = v[b + P_CONFIRM_AT];
            if new == 0 {
                let price = v[b + P_SCRAP_PRICE + ((at / 8).clamp(0, 1) * SLOTS as i32 + (at % 8).clamp(0, SLOTS as i32 - 1)) as usize];
                s.s("Scrap ").s(name(old)).s(" for ").i(price).s(" coins?");
            } else {
                s.s("Replace ").s(name(old)).s(" with ").s(name(new)).s(" for good?");
            }
            wrap(e, x + 16.0, y + 14.0, w - 32.0, d(17), 22.0, INK, ALIGN_LEFT, s.str(), 2);
            let mut s = Buf::new();
            s.s(if at / 8 == 0 { "Main" } else { "Partner" }).s(", slot ").i(at % 8 + 1).s(". The old drive is gone.");
            e.text_ex(x + 16.0, y + 64.0, bs(12), LILAC, ALIGN_LEFT, w - 32.0, s.str());
            let cur = v[b + P_CONFIRM_CUR];
            let words = if new == 0 { ["Keep", "Scrap"] } else { ["Keep", "Replace"] };
            for (i, word) in words.iter().enumerate() {
                let bx = x + 16.0 + i as f32 * 118.0;
                let on = cur == i as i32;
                face(e, if on { if i == 0 { S_FACE_SUNNY } else { S_FACE_CORAL } } else { S_FACE_CREAM }, bx, y + 96.0, 108.0, 34.0, WHITE);
                e.text_ex(bx + 54.0, y + 103.0, d(15), if on && i == 1 { WHITE } else { INK }, ALIGN_CENTER, 0.0, word);
                if on {
                    spr(e, S_PTR + p, bx - 22.0, y + 100.0, 0.0, 0.0, WHITE);
                }
            }
        });
        if humans == 2 {
            face(e, if p == 0 { S_FACE_CORAL } else { S_FACE_LAGOON }, x - 4.0, y - 10.0, 28.0, 18.0, WHITE);
            e.text_ex(x + 10.0, y - 8.0, d(12), WHITE, ALIGN_CENTER, 0.0, if p == 0 { "P1" } else { "P2" });
        }
    }
    if confirm_up(v) {
        hints(e, &[("Stick", "Choose"), ("A", "Confirm"), ("B", "Keep")]);
    }
}
