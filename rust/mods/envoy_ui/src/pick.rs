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

/// One player's side: three cards, then (step 1) the fighters; `x0`, `w0` the side's box.
unsafe fn side(e: &mut E, v: &[i32; VW], p: usize, x0: f32, w0: f32, coop: bool) {
    let pbk = V_P0 + p * V_PW;
    let done = v[pbk + P_RW_DONE] != 0;
    let step = v[pbk + P_STEP];
    let cur = v[pbk + P_CURSOR].clamp(0, 2) as usize;
    let gap = if coop { 6.0 } else { 14.0 };
    let cw = (w0 - 2.0 * gap) / 3.0;
    let ch = if coop { 232.0 } else { 250.0 };
    let y = 64.0;
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
    for i in 0..3 {
        let (_, a) = enter(T - OPEN_AT - i as i32 * 3);
        if a <= 0.0 {
            continue;
        }
        let x = x0 + i as f32 * (cw + gap);
        let on = i == cur && !done;
        let faded = (step >= 1 || done) && i != cur;
        offer(e, v, p, i, x, y, cw, ch, on, faded);
        if on && step == 0 {
            spr(e, S_PTRDOWN + p, x + cw * 0.5 - 13.0, y - 40.0, 0.0, 0.0, WHITE);
        }
    }
    // the fighters (step 1: who wears it)
    let party = party_from_view(v);
    let sets = if v[V_SHARED] != 0 { 1 } else { 2 };
    let fw = if coop { (w0 - 8.0) / 2.0 } else { 140.0 };
    for f in 0..sets {
        let fx = if coop { x0 + f as f32 * (fw + 8.0) } else { x0 + w0 * 0.5 - 146.0 + f as f32 * 152.0 };
        let fy = 316.0;
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
    let title = if keys { "Choose a keystone" } else { "Pick a drive" };
    text_edge(e, if coop { 320.0 } else { 16.0 }, 6.0, d(24), WHITE, if coop { ALIGN_CENTER } else { ALIGN_LEFT }, 2.2, title);
    if !coop {
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
        side(e, v, 0, 40.0, 560.0, false);
    }
    // strip and hints
    let pbk = V_P0;
    let cur = v[pbk + P_CURSOR].clamp(0, 2) as usize;
    let id = v[pbk + P_OFFERS + cur];
    let mut s = Buf::new();
    if v[pbk + P_RW_DONE] != 0 {
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
    if step2 {
        hints(e, &[("Left / Right", "Choose a slot"), ("A", "Replace"), ("B", "Back")]);
    } else if step1 {
        hints(e, &[("Left / Right", "Choose a fighter"), ("A", "Take"), ("B", "Back")]);
    } else {
        hints(e, &[("Left / Right", "Choose"), ("A", "Take it")]);
    }
}
