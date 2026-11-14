//! Envoy items on screen (envoy-items-design.md 6, the first slice): every item's signature effect in the world
//! (sparks, a burning wake, the shine bell, stars, ice spikes, a ghost, mines, quakes, moons, coins; Homing Ink's
//! trail and Fire Lace's embers on everything), the pedestals of the item room and the boss drop, the new-item flash,
//! the item room / boss drop / campfire banners and the item strip. Presentation only: it reads the sim's view.
use crate::kit::*;
use crate::{Buf, VW};
use envoy_data::items::*;
use envoy_data::view::*;
use gd_ui_enc::*;

const FX_RING: i32 = 1;
const FX_DISC: i32 = 2;
const FX_SPIKES: i32 = 3;
const FX_BOLT: i32 = 4;
const FX_BEAM: i32 = 5;
const FX_LINE: i32 = 6;
const TAU: f32 = 6.283_185_5;
const INK_COL: u32 = 0xB04A_FFFF;
const EMBER: u32 = 0xFF7A_20FF;

unsafe fn fx(kind: i32, x: f32, y: f32, a: f32, b: f32, c: f32, rgba: u32) {
    // the Envoy settings' effect intensity (pause overlay, Options) fades every item effect in the world
    let k = crate::pause::fx_scale();
    if k <= 0.0 {
        return;
    }
    crate::gd_ui_world_fx(kind, x, y, 0.0, a, b, c, alpha(rgba, k));
}

fn col(id: i32) -> u32 {
    item(id).map_or(WHITE, |it| it.rgba)
}

fn pb(p: usize) -> usize {
    V_P0 + p * V_PW
}

static mut T: u32 = 0;

/// Every item effect this frame, from the view's projectile list and burst ring.
pub unsafe fn world(v: &[i32; VW]) {
    if v[V_ITEMS_ON] != 1 {
        return;
    }
    T = T.wrapping_add(1);
    let t = T as f32;
    let play = v[V_PLAY];
    for i in 0..PROJ {
        let b = V_PROJ + i * PJ_W;
        let kind = v[b];
        if kind == 0 {
            break;
        }
        let flags = v[b + 1] & PF_MASK;
        let (x, y) = (v[b + 2] as f32 / 100.0, v[b + 3] as f32 / 100.0);
        let (age, life, a, bb) = (v[b + 4], v[b + 5].max(1), v[b + 6], v[b + 7]);
        let fade = 1.0 - ((age as f32 - life as f32 + 8.0) / 8.0).clamp(0.0, 1.0);
        let (vx, vy) = (a as f32 / 100.0, bb as f32 / 100.0);
        match kind {
            PJ_SPARK => {
                fx(FX_LINE, x, y, x - vx * 3.5, y - vy * 3.5, 0.9, alpha(0xFFF6_B0FF, 0.8 * fade));
                fx(FX_DISC, x, y, 2.2, 0.0, 0.0, alpha(col(IT_SPARK_RING), fade));
                fx(FX_SPIKES, x, y, 0.6, 2.6, t * 0.5 + i as f32, alpha(WHITE, 0.8 * fade));
            }
            PJ_STAR => {
                fx(FX_LINE, x, y, x - vx * 5.0, y - vy * 5.0, 1.6, alpha(col(IT_HOP_SHOT), 0.6 * fade));
                fx(FX_DISC, x, y, 4.2, 0.0, 0.0, alpha(0xFFF0_A0FF, 0.7 * fade));
                fx(FX_SPIKES, x, y, 1.2, 4.6, t * 0.25, alpha(col(IT_HOP_SHOT), fade));
            }
            PJ_WAKE => {
                let half = a as f32 / 100.0;
                let k = (life - age) as f32 / life as f32;
                fx(FX_LINE, x - half, y + 0.4, x + half, y + 0.4, 1.4, alpha(0xFF5A_10FF, 0.8 * k));
                let n = (half / 3.0) as i32;
                for j in -n..=n {
                    let fx_ = x + j as f32 * 3.0;
                    let flick = 2.5 + 1.8 * sin(t * 0.6 + j as f32 * 1.7);
                    fx(FX_BEAM, fx_, y, flick * (0.6 + 0.4 * k) * 2.0, 2.4, 0.0, alpha(col(IT_GLIDE_WAKE), 0.75 * k));
                }
            }
            PJ_GHOST => {
                let face = if a < 0 { -1.0 } else { 1.0 };
                let k = fade * (0.55 + 0.15 * sin(t * 0.4));
                fx(FX_BEAM, x, y, 16.0, 7.5, 0.0, alpha(col(IT_WHIFF_ECHO), 0.45 * k));
                fx(FX_DISC, x, y + 15.0, 3.6, 0.0, 0.0, alpha(col(IT_WHIFF_ECHO), 0.6 * k));
                fx(FX_RING, x, y + 9.0, 7.0, 0.8, 0.0, alpha(col(IT_WHIFF_ECHO), 0.5 * k));
                // the arm of the swing to come: a line toward where it lands
                fx(FX_LINE, x, y + 10.0, x + face * 8.0, y + 9.0, 1.2, alpha(WHITE, 0.5 * k));
            }
            PJ_MINE => {
                let blink = if (age / 6) % 2 == 0 || age > life - 18 && (age / 3) % 2 == 0 { 1.0 } else { 0.35 };
                fx(FX_DISC, x, y + 1.0, 2.8, 0.0, 0.0, alpha(col(IT_ROLL_MINES), blink));
                fx(FX_RING, x, y + 1.0, 4.0 + 2.5 * (age as f32 / life as f32), 0.7, 0.0, alpha(0xFFB0_B0FF, 0.7));
                fx(FX_SPIKES, x, y + 1.0, 1.0, 3.6, 0.0, alpha(0x5020_20FF, 0.9));
            }
            PJ_QUAKE => {
                let size = bb as f32 / 100.0;
                let k = fade;
                fx(FX_SPIKES, x, y + 1.0, 1.0, size * 1.2, t * 0.2, alpha(col(IT_SMASH_QUAKE), k));
                fx(FX_BEAM, x, y, size * 1.6, size * 0.9, 0.0, alpha(0xFFE8_A0FF, 0.7 * k));
                fx(FX_LINE, x - a as f32 * 8.0, y + 0.3, x, y + 0.3, 1.0, alpha(0x8A5A_20FF, 0.9 * k));
            }
            PJ_COIN => {
                fx(FX_DISC, x, y, 2.0, 0.0, 0.0, alpha(col(IT_COIN_MAGNET), 1.0));
                fx(FX_RING, x, y, 2.4, 0.6, 0.0, alpha(0xFFF4_C0FF, 0.9));
                fx(FX_LINE, x, y, x - vx * 2.0, y - vy * 2.0, 0.8, alpha(col(IT_COIN_MAGNET), 0.5));
            }
            PJ_MOON => {
                fx(FX_DISC, x, y, 3.0, 0.0, 0.0, alpha(col(IT_MOON_DROPS), 0.95));
                fx(FX_RING, x, y, 3.6, 0.7, 0.0, alpha(0xA0B8_FFFF, 0.7));
            }
            _ => {}
        }
        if flags & PF_INK != 0 && matches!(kind, PJ_SPARK | PJ_STAR) {
            fx(FX_LINE, x, y, x - vx * 9.0, y - vy * 9.0, 2.4, alpha(INK_COL, 0.55 * fade));
        }
        if flags & PF_LACE != 0 && kind != PJ_COIN {
            fx(FX_SPIKES, x + 1.5 * sin(t * 0.9 + i as f32), y + 2.0 + 1.5 * cos(t * 0.7), 0.4, 2.2, t * 0.3, alpha(EMBER, 0.75));
        }
    }
    // the one-shot visuals
    let n = v[V_BURST_N].max(0) as usize;
    for e in 0..BURSTS.min(n) {
        let b = V_BURST + e * BU_W;
        let age = play - v[b];
        if !(0..32).contains(&age) {
            continue;
        }
        let (id, x, y, size, fl) = (v[b + 1], v[b + 3] as f32 / 100.0, v[b + 4] as f32 / 100.0, v[b + 5] as f32 / 100.0, v[b + 6]);
        let k = age as f32 / 32.0;
        let a = 1.0 - k;
        let c = col(id);
        match id {
            IT_SPARK_RING => {
                fx(FX_RING, x, y, 2.0 + size * 1.6 * ease(k * 2.0), 1.6, 0.0, alpha(c, a));
                if age < 6 {
                    fx(FX_DISC, x, y, 6.0, 0.0, 0.0, alpha(WHITE, 0.8 * (1.0 - age as f32 / 6.0)));
                }
            }
            IT_GLIDE_WAKE => fx(FX_DISC, x, y + 2.0, 6.0 * a, 0.0, 0.0, alpha(c, 0.7 * a)),
            IT_SHINE_BELL => {
                fx(FX_RING, x, y, size * ease(k * 2.5), 2.4, 0.0, alpha(c, a));
                fx(FX_RING, x, y, size * 0.7 * ease(k * 1.6), 1.2, 0.0, alpha(WHITE, a));
                fx(FX_SPIKES, x, y, size * 0.5, size * 1.1 * ease(k * 2.0), T as f32 * 0.05, alpha(c, 0.6 * a));
            }
            IT_HOP_SHOT => fx(FX_SPIKES, x, y, 1.0, size * (0.5 + k), 0.4, alpha(c, a)),
            IT_LEDGE_SPIKES => {
                let h = size * ease(k * 4.0);
                for j in 0..4 {
                    let yy = y + h * j as f32 / 4.0;
                    fx(FX_SPIKES, x, yy, 1.5, 5.0 + j as f32, 0.785 + j as f32 * 0.3, alpha(c, a));
                }
                fx(FX_BEAM, x, y, h, 7.0, 0.0, alpha(0xD8FF_FFFF, 0.7 * a));
            }
            IT_WHIFF_ECHO => {
                let dir = if fl & 3 == 0 { -1.0 } else { 1.0 };
                fx(FX_RING, x, y, size * (0.5 + 0.6 * ease(k * 3.0)), 2.2, 0.0, alpha(c, a));
                fx(FX_LINE, x - dir * size * 0.7, y + size * 0.5, x + dir * size * 0.7, y - size * 0.5, 2.0, alpha(WHITE, a));
            }
            IT_ROLL_MINES => {
                fx(FX_DISC, x, y, size * (0.6 + 0.6 * ease(k * 3.0)), 0.0, 0.0, alpha(0xFFB0_40FF, a));
                fx(FX_SPIKES, x, y, 2.0, size * 1.5 * ease(k * 2.5), 0.2, alpha(c, a));
            }
            IT_SMASH_QUAKE => {
                fx(FX_DISC, x, y + 2.0, size * (0.6 + k), 0.0, 0.0, alpha(0xE0C0_90FF, 0.6 * a));
                fx(FX_BOLT, x - size * 2.0 * ease(k * 3.0), y + 0.2, x + size * 2.0 * ease(k * 3.0), y + 0.2, 1.0, alpha(0x7A4A_10FF, a));
            }
            IT_MOON_DROPS => fx(FX_RING, x, y, 3.0 + 6.0 * k, 1.0, 0.0, alpha(c, a)),
            IT_COIN_MAGNET => {
                fx(FX_SPIKES, x, y, 2.0, size * (0.6 + k), 0.0, alpha(c, a));
                fx(FX_DISC, x, y, size * 0.6 * a, 0.0, 0.0, alpha(0xFFF4_C0FF, a));
            }
            _ => {}
        }
        if fl & PF_LACE != 0 {
            for j in 0..3 {
                let ang = j as f32 * TAU / 3.0 + k * 3.0;
                fx(FX_SPIKES, x + cos(ang) * size * 0.6 * k, y + sin(ang) * size * 0.6 * k + k * 6.0, 0.4, 2.0, 0.0, alpha(EMBER, a));
            }
        }
    }
    let _ = FX_BOLT;
}

/// The pedestals (the item room and the boss drop) and their fly-in: a stone, the item floating over it in its own
/// colour, a light beam in the owner's colour (co-op).
static mut LAST: [[(f32, f32); DROPS]; 2] = [[(0.0, 0.0); DROPS]; 2];
pub unsafe fn pedestals(v: &[i32; VW]) {
    let humans = v[V_HUMANS].clamp(1, 2) as usize;
    let t = T as f32;
    for p in 0..humans {
        for k in 0..DROPS {
            let d = pb(p) + P_DROPS + k * 4;
            if v[d] != 1 {
                LAST[p][k] = (0.0, 0.0);
                continue;
            }
            let id = v[d + 1];
            let c = col(id);
            let (bx, by) = (v[d + 2] as f32 / 100.0, v[d + 3] as f32 / 100.0);
            let moving = LAST[p][k].0 != 0.0 && ((LAST[p][k].0 - bx).abs() + (LAST[p][k].1 - by).abs()) > 0.3;
            if moving {
                fx(FX_LINE, LAST[p][k].0, LAST[p][k].1 + 9.0, bx, by + 9.0, 3.0, alpha(c, 0.9));
            } else {
                // the stone
                fx(FX_BEAM, bx, by, 5.0, 11.0, 0.0, alpha(0xB0A8_D0FF, 0.6));
                fx(FX_LINE, bx - 6.0, by + 5.0, bx + 6.0, by + 5.0, 1.4, alpha(0xE8E0_FFFF, 0.8));
                fx(FX_BEAM, bx, by + 5.0, 60.0, 9.0, 0.0, alpha(if humans == 2 { crate::PLAYER_COL[p] } else { c }, 0.28));
            }
            LAST[p][k] = (bx, by);
            let fy = by + 12.0 + 1.5 * sin(t * 0.07 + k as f32 * 1.3);
            fx(FX_DISC, bx, fy, 7.5, 0.0, 0.0, alpha(c, 0.55));
            fx(FX_DISC, bx, fy, 4.0, 0.0, 0.0, alpha(WHITE, 0.8));
            fx(FX_RING, bx, fy, 6.0, 1.4, 0.0, alpha(c, 1.0));
            fx(FX_SPIKES, bx, fy, 4.5, 8.5 + sin(t * 0.2), t * 0.02, alpha(c, 0.7));
        }
    }
}

/// The items player `p` holds, in id order: a coloured chip each, x2 on a stack (band space).
pub unsafe fn strip(e: &mut E, v: &[i32; VW], p: usize, x: f32, y: f32, w: f32) {
    let mut cx = x + 11.0;
    let mut row_y = y + 11.0;
    let mut any = false;
    for it in ITEMS.iter() {
        let n = v[V_INV + p * ITEM_IDS + it.id as usize];
        if n <= 0 {
            continue;
        }
        any = true;
        spr_c(e, S_DISC, cx, row_y, 22.0 / 32.0, INK);
        spr_c(e, S_DISC, cx, row_y, 17.0 / 32.0, it.rgba);
        if n > 1 {
            let mut s = Buf::new();
            s.s("x").i(n);
            face(e, S_FACE_INK, cx + 3.0, row_y + 2.0, 16.0, 11.0, WHITE);
            e.text_ex(cx + 11.0, row_y + 2.0, d(8), SUNNY, ALIGN_CENTER, 0.0, s.str());
        }
        cx += 26.0;
        if cx > x + w - 12.0 {
            cx = x + 11.0;
            row_y += 26.0;
        }
    }
    if !any {
        e.text_ex(x, y + 4.0, bs(12), PALE, ALIGN_LEFT, 0.0, "No items yet");
    }
}

/// Seen gain counts (the flash starts when one moves) and the frame each flash started.
static mut SEEN: [i32; 2] = [-1; 2];
static mut AT: [u32; 2] = [0; 2];

/// The new-item flash (2 s): the item's chip, name and one line over the top of the screen, and a ring of its
/// colour round the fighter in the world. Band space.
pub unsafe fn flash(e: &mut E, v: &[i32; VW]) {
    let humans = v[V_HUMANS].clamp(1, 2) as usize;
    for p in 0..humans {
        let g = pb(p) + P_GAIN;
        let key = v[g + 5];
        if SEEN[p] < 0 {
            SEEN[p] = key; // the first view: what was gained before is not news
        } else if key != SEEN[p] {
            SEEN[p] = key;
            if key > 0 && item(v[g + 1]).is_some() {
                AT[p] = T.max(1);
                crate::sfx("pickup");
            }
        }
        let age = T.wrapping_sub(AT[p]) as i32;
        let Some(it) = item(v[g + 1]) else { continue };
        if AT[p] == 0 || !(0..120).contains(&age) {
            continue;
        }
        let a = if age > 100 { (120 - age) as f32 / 20.0 } else { 1.0 };
        let copies = v[g + 3];
        let mut s = Buf::new();
        s.s(it.name);
        if copies > 1 {
            s.s("  x").i(copies);
        }
        let w = tw(s.str(), d(18)).max(tw(it.line, bs(13))) + 70.0;
        let (x, y) = (320.0 - w * 0.5, 64.0 + p as f32 * 58.0);
        let (sc, _) = enter(age);
        let _ = sc;
        sticker(e, S_FACE_WHITE, x, y, w, 50.0, if p == 0 { -1.2 } else { 1.2 }, alpha(WHITE, a), |e| {
            spr_c(e, S_DISC, x + 26.0, y + 25.0, 34.0 / 32.0, alpha(INK, a));
            spr_c(e, S_DISC, x + 26.0, y + 25.0, 27.0 / 32.0, alpha(it.rgba, a));
            e.text_ex(x + 52.0, y + 5.0, d(18), alpha(INK, a), ALIGN_LEFT, 0.0, s.str());
            e.text_ex(x + 52.0, y + 28.0, bs(13), alpha(LILAC, a), ALIGN_LEFT, 0.0, it.line);
        });
        // in the world: rings of the item's colour round the fighter
        if v[pb(p) + P_HAVE_POS] != 0 && age < 40 {
            let (fx_, fy) = (f32::from_bits(v[pb(p) + P_X] as u32), f32::from_bits(v[pb(p) + P_Y] as u32));
            let k = age as f32 / 40.0;
            fx(FX_RING, fx_, fy + 9.0, 6.0 + 22.0 * ease(k), 2.6, 0.0, alpha(it.rgba, 1.0 - k));
            fx(FX_BEAM, fx_, fy, 40.0 * (1.0 - k), 12.0, 0.0, alpha(it.rgba, 0.4 * (1.0 - k)));
        }
    }
}

/// The room banners: the item room and the boss drop while their pedestals wait, the campfire after the boss.
pub unsafe fn banners(e: &mut E, v: &[i32; VW]) {
    let kind = v[V_PED_KIND];
    if kind != 0 && v[V_RW] == 0 {
        let title = if kind == PED_BOSS { "Boss drop: take one" } else { "Item room" };
        let say = if kind == PED_BOSS { "Walk into the item you want. The other one stays behind." } else { "Walk to the pedestal to take the item." };
        let w = tw(say, bs(13)).max(tw(title, d(22))) + 40.0;
        sticker(e, S_FACE_SUNNY, 320.0 - w * 0.5, 120.0, w, 58.0, -1.0, WHITE, |e| {
            e.text_ex(320.0, 124.0, d(22), INK, ALIGN_CENTER, 0.0, title);
            e.text_ex(320.0, 154.0, bs(13), INK, ALIGN_CENTER, 0.0, say);
        });
        let mut s = Buf::new();
        s.i((v[V_PED_LEFT] + 59) / 60).s(" s");
        e.text_ex(320.0 + w * 0.5 - 8.0, 182.0, bs(12), WHITE, ALIGN_RIGHT, 0.0, s.str());
    }
    let at = v[V_CAMP_AT];
    let age = v[V_PLAY] - at;
    if at > 0 && (0..240).contains(&age) {
        let a = if age > 200 { (240 - age) as f32 / 40.0 } else { 1.0 };
        let mut s = Buf::new();
        s.s("Campfire: stocks ").i(v[V_POOL]).s(" / ").i(v[V_POOL_LIM]);
        if v[V_CAMP_GAIN] > 0 {
            s.s("  (+").i(v[V_CAMP_GAIN]).s(")");
        }
        let w = tw(s.str(), d(18)) + 70.0;
        sticker(e, S_FACE_CORAL, 320.0 - w * 0.5, 190.0, w, 40.0, 1.0, alpha(WHITE, a), |e| {
            for j in 0..3 {
                spr_c(e, S_FLAME + 1 + j.min(2), 320.0 - w * 0.5 + 24.0, 210.0 - j as f32 * 3.0, 0.5 + 0.1 * sin(age as f32 * 0.3 + j as f32), alpha(WHITE, a));
            }
            e.text_ex(320.0 - w * 0.5 + 46.0, 199.0, d(18), alpha(WHITE, a), ALIGN_LEFT, 0.0, s.str());
        });
    }
}

/// Item sounds: one per new burst (keyed on its frame and slot, so a rollback replay does not double it).
static mut PLAYED: [i32; 16] = [-1; 16];
pub unsafe fn sounds(v: &[i32; VW]) {
    if v[V_ITEMS_ON] != 1 {
        return;
    }
    let n = v[V_BURST_N].max(0) as usize;
    let play = v[V_PLAY];
    let mut k = 0;
    for e in 0..BURSTS.min(n) {
        let b = V_BURST + e * BU_W;
        if !(0..3).contains(&(play - v[b])) || k >= 2 {
            continue;
        }
        let key = v[b].wrapping_mul(32) ^ v[b + 1] ^ (e as i32) << 24;
        if PLAYED.contains(&key) {
            continue;
        }
        PLAYED[e % 16] = key;
        k += 1;
        crate::sfx(match v[b + 1] {
            IT_SPARK_RING => "fire_bolt",
            IT_SHINE_BELL => "fire_blue",
            IT_HOP_SHOT => "fire_white",
            IT_LEDGE_SPIKES => "fire_shatter",
            IT_WHIFF_ECHO => "fire_green",
            IT_ROLL_MINES | IT_SMASH_QUAKE => "fire_yellow",
            IT_COIN_MAGNET => "pickup",
            IT_GLIDE_WAKE => "fire_red",
            _ => "",
        });
    }
}
