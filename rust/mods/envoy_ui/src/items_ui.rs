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
            _ => proj2(v, i, kind, flags, x, y, age, life, a, bb, fade),
        }
        if kind <= PJ_MOON {
            look_flags(flags, x, y, vx, vy, fade, kind);
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
        burst2(id, x, y, size, fl, age, k, a);
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
static mut DOOR_SUM: (f32, f32, f32) = (0.0, 0.0, 0.0);
pub unsafe fn pedestals(v: &[i32; VW]) {
    wall(v);
    DOOR_SUM = (0.0, 1.0e4, 0.0);
    // the door behind the room's pedestals (drawn first: behind them)
    door_last(v);
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
            if !moving {
                DOOR_SUM.0 += bx;
                DOOR_SUM.1 = DOOR_SUM.1.min(by);
                DOOR_SUM.2 += 1.0;
                ped_cost(v, p, k, bx, by + 22.0);
            }
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
        crate::icons::icon(e, it.id, cx, row_y, 22.0);
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
        let card = envoy_data::keywords::card(it.id);
        let w = (tw(s.str(), d(18)) + 70.0).max(400.0);
        let (x, y) = (320.0 - w * 0.5, 64.0 + p as f32 * 72.0);
        let (sc, _) = enter(age);
        let _ = sc;
        sticker(e, S_FACE_WHITE, x, y, w, 64.0, if p == 0 { -1.2 } else { 1.2 }, alpha(WHITE, a), |e| {
            crate::icons::icon_a(e, it.id, x + 26.0, y + 25.0, 36.0, a);
            e.text_ex(x + 52.0, y + 5.0, d(18), alpha(INK, a), ALIGN_LEFT, 0.0, s.str());
            rich(e, x + 52.0, y + 27.0, w - 62.0, 13, 16.0, alpha(INK, a), false, card, 2);
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
    if (kind == PED_ITEM_ROOM || kind == PED_BOSS) && v[V_RW] == 0 {
        let title = if kind == PED_BOSS { "Boss drop: take one" } else { "Item room" };
        let say = if kind == PED_BOSS { "Walk into the item you want. The other one stays behind." } else { "Walk to the pedestal to take the item." };
        let w = tw(say, bs(13)).max(tw(title, d(22))) + 40.0;
        sticker(e, S_FACE_SUNNY, 320.0 - w * 0.5, 120.0, w, 58.0, -1.0, WHITE, |e| {
            e.text_ex(320.0, 124.0, d(22), INK, ALIGN_CENTER, 0.0, title);
            e.text_ex(320.0, 154.0, bs(13), INK, ALIGN_CENTER, 0.0, say);
        });
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

// ---------------------------------------------------------------- slice 2: rooms, the wall, the fork, new effects

const BLOOD: u32 = 0xC010_1EFF;
const HOLY: u32 = 0xFFF8_E8FF;
const ICE: u32 = 0xA8E8_FFFF;
static mut DOOR_AT: (f32, f32, f32) = (0.0, 0.0, 0.0);

/// The devil / angel door behind the pedestals: placed from last frame's pedestal centre (pedestals stand still).
unsafe fn door_last(v: &[i32; VW]) {
    let kind = v[V_PED_KIND];
    if kind != PED_DEVIL && kind != PED_ANGEL {
        DOOR_AT = (0.0, 0.0, 0.0);
        return;
    }
    if DOOR_AT.2 == 0.0 {
        return;
    }
    let t = T as f32;
    let (cx, fy) = (DOOR_AT.0, DOOR_AT.1);
    let (w, h) = (24.0, 38.0);
    let devil = kind == PED_DEVIL;
    let (body, edge, glow) = if devil { (0x7A08_18FF, 0xFF30_40FF, 0xFF20_30FF) } else { (0xC8C8_D8FF, WHITE, HOLY) };
    fx(FX_BEAM, cx, fy - 2.0, h + 22.0, w + 26.0, 0.0, alpha(glow, 0.22 + 0.06 * sin(t * 0.08)));
    fx(FX_LINE, cx, fy, cx, fy + h, w, alpha(body, 0.85));
    fx(FX_DISC, cx, fy + h, w * 0.5, 0.0, 0.0, alpha(body, 0.9));
    fx(FX_LINE, cx - w * 0.5, fy, cx - w * 0.5, fy + h, 1.8, edge);
    fx(FX_LINE, cx + w * 0.5, fy, cx + w * 0.5, fy + h, 1.8, edge);
    fx(FX_RING, cx, fy + h, w * 0.5, 1.8, 0.0, edge);
    fx(FX_LINE, cx - w * 0.5 - 3.0, fy, cx + w * 0.5 + 3.0, fy, 2.0, edge);
    if devil {
        fx(FX_LINE, cx - 9.0, fy + h + 8.0, cx - 14.0, fy + h + 17.0, 2.2, edge);
        fx(FX_LINE, cx + 9.0, fy + h + 8.0, cx + 14.0, fy + h + 17.0, 2.2, edge);
        fx(FX_SPIKES, cx, fy + h * 0.55, 1.5, 7.0, t * 0.01, alpha(0xFF60_70FF, 0.8));
        for j in 0..3 {
            let ph = (t * 0.04 + j as f32 * 2.1) % 1.0;
            fx(FX_DISC, cx - 10.0 + j as f32 * 10.0, fy + ph * 30.0, 2.0, 0.0, 0.0, alpha(0xFF40_20FF, 1.0 - ph));
        }
    } else {
        fx(FX_RING, cx, fy + h + 10.0, 6.0, 1.4, 0.0, alpha(HOLY, 0.9));
        fx(FX_SPIKES, cx, fy + h * 0.5, 4.0, 22.0 + 2.0 * sin(t * 0.1), t * 0.004, alpha(HOLY, 0.35));
    }
}

/// A devil deal's price over its pedestal: a red slot per stock slot; crossed grey ones when locked.
unsafe fn ped_cost(v: &[i32; VW], p: usize, k: usize, x: f32, y: f32) {
    if v[V_PED_KIND] != PED_DEVIL {
        return;
    }
    let w = v[V_PED_COST + p * DROPS + k];
    let n = (w & 0xFF).clamp(0, 4);
    let locked = w & COST_LOCKED != 0;
    for j in 0..n {
        let sx = x + (j as f32 - (n - 1) as f32 * 0.5) * 6.0;
        if locked {
            fx(FX_RING, sx, y, 2.4, 0.8, 0.0, alpha(0x9090_A0FF, 0.9));
            fx(FX_LINE, sx - 2.2, y - 2.2, sx + 2.2, y + 2.2, 0.8, 0xFF40_40FF);
            fx(FX_LINE, sx - 2.2, y + 2.2, sx + 2.2, y - 2.2, 0.8, 0xFF40_40FF);
        } else {
            fx(FX_DISC, sx, y, 3.6, 0.0, 0.0, alpha(0xFF20_30FF, 0.8));
            fx(FX_RING, sx, y, 2.4, 0.9, 0.0, 0xFF50_60FF);
        }
    }
}

/// The cracked wall in a fight (world): a stone block, cracks growing with its hits; then the crumble.
static mut WALL_SEEN: i32 = 0;
static mut WALL_BROKE: u32 = 0;
unsafe fn wall(v: &[i32; VW]) {
    // the door's place comes from the pedestals the previous frame drew
    if DOOR_SUM.2 > 0.0 {
        DOOR_AT = (DOOR_SUM.0 / DOOR_SUM.2, DOOR_SUM.1, 1.0);
    }
    orb_events(v);
    let st = v[V_SECRET];
    if st == 2 && WALL_SEEN == 1 {
        WALL_BROKE = T.max(1);
    }
    WALL_SEEN = st;
    let (x, y) = (v[V_SECRET_X] as f32 / 100.0, v[V_SECRET_Y] as f32 / 100.0);
    if st == 1 {
        let (w, h) = (14.0, 26.0);
        let frac = (v[V_SECRET_HITS] as f32 / v[V_SECRET_HP].max(1) as f32).clamp(0.0, 1.0);
        fx(FX_LINE, x, y, x, y + h, w, alpha(0x8A7C_9AFF, 0.75));
        for r in 0..4 {
            let yy = y + r as f32 * h / 4.0;
            fx(FX_LINE, x - w * 0.5, yy, x + w * 0.5, yy, 0.6, alpha(0x3A30_48FF, 0.9));
            let off = if r % 2 == 0 { -2.5 } else { 2.5 };
            fx(FX_LINE, x + off, yy, x + off, yy + h / 4.0, 0.6, alpha(0x3A30_48FF, 0.9));
        }
        fx(FX_LINE, x - w * 0.5, y, x - w * 0.5, y + h, 1.2, 0xD8D0_E8FF);
        fx(FX_LINE, x + w * 0.5, y, x + w * 0.5, y + h, 1.2, 0xD8D0_E8FF);
        // the crack: always one; more, longer and brighter with each hit
        let cracks = 1 + (frac * 4.0) as i32;
        for c in 0..cracks {
            let ang = -1.2 + c as f32 * 0.8;
            let len = 5.0 + 8.0 * frac;
            fx(FX_BOLT, x, y + h * 0.55, x + cos(ang) * len, y + h * 0.55 + sin(ang) * len, 0.9, alpha(0xFFE8_A0FF, 0.5 + 0.5 * frac));
        }
        if frac > 0.6 {
            fx(FX_DISC, x, y + h * 0.55, 6.0, 0.0, 0.0, alpha(0xFFD0_60FF, 0.4 * (0.5 + 0.5 * sin(T as f32 * 0.3))));
        }
    }
    let age = T.wrapping_sub(WALL_BROKE) as i32;
    if WALL_BROKE != 0 && (0..50).contains(&age) {
        let k = age as f32 / 50.0;
        for j in 0..8 {
            let ang = j as f32 * 0.785 + 0.3;
            let d = 4.0 + 26.0 * ease(k * 1.5);
            let rx = x + cos(ang) * d;
            let ry = y + 13.0 + sin(ang) * d - 30.0 * k * k;
            fx(FX_DISC, rx, ry, 2.6 * (1.0 - k) + 0.6, 0.0, 0.0, alpha(0xB8A8_C8FF, 1.0 - k));
        }
        fx(FX_RING, x, y + 13.0, 6.0 + 24.0 * ease(k * 2.0), 2.0, 0.0, alpha(SUNNY, 1.0 - k));
        if age < 8 {
            fx(FX_DISC, x, y + 13.0, 18.0, 0.0, 0.0, alpha(WHITE, 0.8 * (1.0 - age as f32 / 8.0)));
        }
    }
}

/// Band-space callouts of the slice 2 events (deals, slots, the wall, the doors, the feather, the campfire) and the
/// devil / angel / secret room banners (the devil's price list with the slot icons).
pub unsafe fn callouts(e: &mut E, v: &[i32; VW]) {
    let n = v[V_EV_N].max(0) as usize;
    let now = v[V_PLAY];
    let mut row = 0;
    for i in 0..EV.min(n) {
        let ev = &v[V_EV + i * EV_W..V_EV + (i + 1) * EV_W];
        let age = now - ev[0];
        if !(0..100).contains(&age) {
            continue;
        }
        let who = ev[2].clamp(0, 1) as usize;
        let mut s = crate::Buf::new();
        let (face_id, icon_id) = match ev[1] {
            EV_DEAL => {
                let id = ev[3] & 0xFF;
                let paid = ev[3] >> 8 & 0xFF;
                s.s("Deal: ").s(item(id).map_or("?", |it| it.name)).s("  -").i(paid).s(if paid == 1 { " stock slot" } else { " stock slots" });
                (S_FACE_CORAL, id)
            }
            EV_SLOT => {
                s.s("Stock limit ").i(ev[3]);
                (S_FACE_WHITE, 0)
            }
            EV_SECRET => {
                s.s("The wall broke! A secret room after this fight");
                (S_FACE_SUNNY, 0)
            }
            EV_DOOR => {
                s.s(if ev[3] == DOOR_DEVIL { "A red door opened..." } else if ev[3] == DOOR_ANGEL { "A white door opened" } else { "A door opened" });
                (if ev[3] == DOOR_DEVIL { S_FACE_CORAL } else { S_FACE_WHITE }, 0)
            }
            EV_GUARD => {
                s.s("Guardian Feather saved you!");
                if ev[3] > 0 {
                    s.s("  (").i(ev[3]).s(" left)");
                }
                (S_FACE_WHITE, IT_GUARDIAN_FEATHER)
            }
            EV_CAMPFIRE => {
                s.s("Campfire: +").i(ev[3]).s(if ev[3] == 1 { " stock" } else { " stocks" });
                (S_FACE_SUNNY, 0)
            }
            _ => continue,
        };
        let a = if age > 80 { (100 - age) as f32 / 20.0 } else { 1.0 };
        let (sc, _) = enter(age);
        let w = tw(s.str(), d(15)) + if icon_id != 0 { 48.0 } else { 24.0 };
        let x = 320.0 - w * 0.5;
        let y = 230.0 + row as f32 * 36.0 - (1.0 - sc) * 20.0;
        let ink = if face_id == S_FACE_CORAL { WHITE } else { INK };
        let deg = if who == 0 { -1.4 } else { 1.4 };
        sticker(e, face_id, x, y, w, 30.0, deg, alpha(WHITE, a), |e| {
            let mut tx = x + 12.0;
            if icon_id != 0 {
                crate::icons::icon_a(e, icon_id, x + 18.0, y + 15.0, 22.0, a);
                tx = x + 34.0;
            }
            e.text_ex(tx, y + 6.0, d(15), alpha(ink, a), ALIGN_LEFT, 0.0, s.str());
        });
        if ev[1] == EV_SECRET && age < 40 {
            burst(e, S_BURST_SUNNY, 320.0, 190.0, 120.0, age, "Secret room!", 15);
        }
        row += 1;
        if row >= 3 {
            break;
        }
    }
    let kind = v[V_PED_KIND];
    if v[V_RW] == 0 && matches!(kind, PED_DEVIL | PED_ANGEL | PED_SECRET) {
        let (title, say, fid) = match kind {
            PED_DEVIL => ("Devil deal", "Walk into an item to take it: it costs stock slots. D-pad down: leave.", S_FACE_CORAL),
            PED_ANGEL => ("Angel room", "A gift. Walk to the pedestal to take it.", S_FACE_WHITE),
            _ => ("Secret room", "Walk to the pedestal to take the item.", S_FACE_SUNNY),
        };
        let w = tw(say, bs(13)).max(tw(title, d(22))) + 40.0;
        let ink = if kind == PED_DEVIL { WHITE } else { INK };
        sticker(e, fid, 320.0 - w * 0.5, 60.0, w, 58.0, -1.0, WHITE, |e| {
            e.text_ex(320.0, 64.0, d(22), ink, ALIGN_CENTER, 0.0, title);
            e.text_ex(320.0, 94.0, bs(13), ink, ALIGN_CENTER, 0.0, say);
        });
        if kind == PED_DEVIL {
            // the price list: each pedestal's item and its price in stock slots (locked: crossed, and why)
            let humans = v[V_HUMANS].clamp(1, 2) as usize;
            let mut y = 126.0;
            for p in 0..humans {
                for k in 0..DROPS {
                    let dd = pb(p) + P_DROPS + k * 4;
                    if v[dd] != 1 {
                        continue;
                    }
                    let id = v[dd + 1];
                    let cw = v[V_PED_COST + p * DROPS + k];
                    let cost = (cw & 0xFF).clamp(0, 4);
                    let locked = cw & COST_LOCKED != 0;
                    let name = item(id).map_or("?", |it| it.name);
                    let w = tw(name, d(14)) + 60.0 + cost as f32 * 18.0;
                    let x = 320.0 - w * 0.5;
                    face(e, S_FACE_INK, x, y, w, 26.0, WHITE);
                    crate::icons::icon(e, id, x + 14.0, y + 13.0, 20.0);
                    e.text_ex(x + 28.0, y + 5.0, d(14), if locked { PALE } else { WHITE }, ALIGN_LEFT, 0.0, name);
                    for j in 0..cost {
                        let sx = x + w - 14.0 - j as f32 * 18.0;
                        crate::icons::slot(e, sx, y + 13.0, 15.0, true, false, if locked { 0.4 } else { 1.0 });
                        if locked {
                            crate::icons::ln(e, sx - 6.0, y + 7.0, sx + 6.0, y + 19.0, 2.0, CORAL);
                        }
                    }
                    if locked {
                        text_edge(e, x + w * 0.5, y + 28.0, bs(11), CORAL, ALIGN_CENTER, 1.0, "can't pay your last slot");
                        y += 14.0;
                    }
                    y += 32.0;
                }
            }
        }
    }
}

/// The fork after the item room (V_FORK 1): "Shop or Campfire?", the map, two choices, P1's cursor, the clock.
pub unsafe fn fork(e: &mut E, v: &[i32; VW]) {
    if v[V_FORK] != 1 {
        return;
    }
    let cur = v[V_FORK_CUR].clamp(0, 1);
    let (x, y, w, h) = (90.0, 90.0, 460.0, 236.0);
    e.rect(-200.0, 0.0, 1040.0, 480.0, 0x1A10_4A90);
    sticker(e, S_FACE_INK, x, y, w, h, -0.6, WHITE, |e| {
        face(e, S_FACE_INK, x, y, w, h, WHITE);
        text_edge(e, x + 180.0, y + 10.0, d(22), SUNNY, ALIGN_LEFT, 1.8, "Choose the next room");
        crate::map::draw(e, v, x + 14.0, y + 12.0, 150.0, h - 24.0, 8.0);
        let cur_kind = crate::map::cursor(v).map_or(0, |c| c.1);
        let (title, say, fid) = match cur_kind {
            MN_SHOP => ("Shop", "Spend coins on items and stocks.", S_FACE_LAGOON),
            MN_CAMPFIRE => ("Campfire", "Rest: stocks back.", S_FACE_CORAL),
            MN_ITEM => ("Item room", "A free item on a pedestal.", S_FACE_SUNNY),
            _ => ("", "", S_FACE_WHITE),
        };
        if !title.is_empty() {
            let (bx, by, bw) = (x + 180.0, y + 56.0, 260.0);
            face(e, S_FACE_HOVER, bx - 3.0, by - 3.0, bw + 6.0, 96.0, WHITE);
            face(e, fid, bx, by, bw, 90.0, WHITE);
            match cur_kind {
                MN_SHOP => spr_c(e, S_COIN, bx + 34.0, by + 45.0, 2.4, WHITE),
                MN_CAMPFIRE => spr_c(e, S_FLAME + 3, bx + 34.0, by + 43.0, 1.3, WHITE),
                _ => spr_c(e, S_STAR_ON, bx + 34.0, by + 45.0, 3.0, WHITE),
            }
            e.text_ex(bx + 66.0, by + 14.0, d(22), if fid == S_FACE_SUNNY { INK } else { WHITE }, ALIGN_LEFT, 0.0, title);
            e.text_ex(bx + 66.0, by + 50.0, bs(13), if fid == S_FACE_SUNNY { INK } else { WHITE }, ALIGN_LEFT, bw - 74.0, say);
        }
        let _ = cur;
        e.text_ex(x + 310.0, y + h - 30.0, bs(13), PALE, ALIGN_CENTER, 0.0, "P1: Left / Right to choose, A to go.");
    });
}

unsafe fn player_pos(v: &[i32; VW], p: usize) -> Option<(f32, f32)> {
    if v[pb(p) + P_HAVE_POS] == 0 {
        return None;
    }
    Some((f32::from_bits(v[pb(p) + P_X] as u32), f32::from_bits(v[pb(p) + P_Y] as u32)))
}

fn slot_pos(v: &[i32; VW], s: i32) -> Option<(f32, f32)> {
    if !(0..4).contains(&s) {
        return None;
    }
    let b = V_S0 + s as usize * V_SW;
    Some((v[b + S_X] as f32 / 100.0, v[b + S_Y] as f32 / 100.0))
}

/// Slice 2's projectiles and published orbitals (from `world`'s loop).
#[allow(clippy::too_many_arguments)]
unsafe fn proj2(v: &[i32; VW], i: usize, kind: i32, flags: i32, x: f32, y: f32, age: i32, life: i32, a: i32, bb: i32, fade: f32) {
    let t = T as f32;
    let (vx, vy) = (a as f32 / 100.0, bb as f32 / 100.0);
    let owner = v[V_PROJ + i * PJ_W + 1].clamp(0, 1) as usize;
    let k = age as f32 / life.max(1) as f32;
    match kind {
        PJ_BEAM => {
            let len = a as f32 / 100.0;
            let grow = ease(age as f32 / 5.0);
            let end = x + len * grow;
            let flick = 1.0 + 0.15 * sin(t * 0.9);
            match bb {
                BEAM_GOLD => {
                    fx(FX_LINE, x, y, end, y, 7.0 * flick, alpha(0xFFC8_30FF, 0.55 * fade));
                    fx(FX_LINE, x, y, end, y, 3.0, alpha(0xFFF4_C0FF, fade));
                    fx(FX_SPIKES, end, y, 1.5, 6.0, t * 0.3, alpha(0xFFE0_60FF, fade));
                }
                BEAM_LASER => {
                    fx(FX_LINE, x, y, end, y, 1.0, alpha(0xFF40_40FF, fade));
                    fx(FX_LINE, x, y, end, y, 2.6, alpha(0xFF80_80FF, 0.35 * fade));
                }
                _ => {
                    // Brimstone: a thick blood-red beam, a pale core, a pulsing edge and a splash at the end
                    fx(FX_LINE, x, y, end, y, 14.0 * flick, alpha(0x9A00_12FF, 0.75 * fade));
                    fx(FX_LINE, x, y, end, y, 8.0, alpha(BLOOD, fade));
                    fx(FX_LINE, x, y, end, y, 2.4, alpha(0xFFA0_A0FF, 0.8 * fade));
                    fx(FX_DISC, x, y, 9.0, 0.0, 0.0, alpha(0xFF20_30FF, 0.7 * fade));
                    fx(FX_SPIKES, end, y, 3.0, 11.0 * flick, t * 0.2, alpha(0xE010_20FF, fade));
                }
            }
        }
        PJ_FEATHER => {
            let sway = 3.0 * sin(t * 0.12 + i as f32);
            fx(FX_LINE, x + sway - 2.5, y + 3.0, x + sway + 2.5, y - 3.0, 2.6, alpha(0x6A20_58FF, 0.9 * fade));
            fx(FX_LINE, x + sway - 2.5, y + 3.0, x + sway + 2.5, y - 3.0, 0.6, alpha(0xE060_80FF, fade));
            fx(FX_DISC, x + sway, y, 3.0, 0.0, 0.0, alpha(0x8A20_60FF, 0.5 * fade));
        }
        PJ_TENDRIL => {
            if let Some((tx, ty)) = slot_pos(v, a) {
                let strike = bb.max(1);
                let reach = if age >= strike { 1.0 } else { ease(age as f32 / strike as f32) };
                let (ex, ey) = (x + (tx - x) * reach, y + 8.0 + (ty - y) * reach);
                fx(FX_BOLT, x, y + 8.0, ex, ey, 2.4, alpha(BLOOD, fade));
                fx(FX_LINE, x, y + 8.0, ex, ey, 0.8, alpha(0xFF80_90FF, fade));
                if age >= strike && age < strike + 8 {
                    fx(FX_SPIKES, ex, ey, 2.0, 9.0, 0.3, alpha(0xFF20_30FF, fade));
                }
            }
        }
        PJ_SHARD => {
            fx(FX_LINE, x, y, x - vx * 4.0, y - vy * 4.0, 1.4, alpha(ICE, 0.6 * fade));
            fx(FX_SPIKES, x, y, 0.8, 4.0, t * 0.4 + i as f32, alpha(ICE, fade));
            fx(FX_DISC, x, y, 2.4, 0.0, 0.0, alpha(WHITE, 0.8 * fade));
        }
        PJ_WELL => {
            let r = (a as f32 / 100.0).max(4.0);
            if age < life - 6 {
                for j in 0..10 {
                    let ph = (t * 0.03 + j as f32 * 0.1) % 1.0;
                    let rr = r * (1.0 - ph);
                    let ang = j as f32 * 0.628 + ph * 5.0;
                    fx(FX_DISC, x + cos(ang) * rr, y + sin(ang) * rr, 1.8, 0.0, 0.0, alpha(0xB080_FFFF, 0.9 * ph + 0.1));
                }
                fx(FX_RING, x, y, r * (1.0 - 0.1 * sin(t * 0.3)), 1.2, 0.0, alpha(0x8050_E0FF, 0.6));
                fx(FX_DISC, x, y, 5.0, 0.0, 0.0, alpha(0x6030_C0FF, 0.9));
            } else {
                let kk = (age - (life - 6)) as f32 / 6.0;
                fx(FX_RING, x, y, r * (0.3 + kk * 1.2), 3.0, 0.0, alpha(0xD0B0_FFFF, 1.0 - kk));
                fx(FX_DISC, x, y, r * 0.8, 0.0, 0.0, alpha(WHITE, 0.7 * (1.0 - kk)));
            }
        }
        PJ_ARC => {
            let jit = if (T / 2) % 2 == 0 { 1.0 } else { -1.0 };
            fx(FX_BOLT, x, y, vx, vy + jit, 2.2, alpha(0xC0E0_FFFF, fade));
            fx(FX_LINE, x, y, vx, vy, 5.0, alpha(0x6090_FFFF, 0.3 * fade));
            fx(FX_DISC, vx, vy, 5.0, 0.0, 0.0, alpha(WHITE, 0.8 * fade));
        }
        PJ_SIGIL => {
            let n = bb.max(1) as f32;
            let (cx, cy) = player_pos(v, owner).unwrap_or((x, y));
            let ang = t * 0.05 + a as f32 / n * TAU;
            let (sx, sy) = (cx + cos(ang) * 14.0, cy + 10.0 + sin(ang) * 14.0);
            fx(FX_RING, sx, sy, 3.2, 1.0, 0.0, 0xFF30_40FF);
            fx(FX_SPIKES, sx, sy, 1.0, 3.2, -t * 0.08, alpha(0xFF60_70FF, 0.9));
            fx(FX_DISC, sx, sy, 4.0, 0.0, 0.0, alpha(BLOOD, 0.5));
        }
        PJ_GUARD => {
            let (cx, cy) = player_pos(v, owner).unwrap_or((x, y));
            let spent = bb == 1;
            let ang = -t * 0.04 + a as f32 * 2.1;
            let (sx, sy) = (cx + cos(ang) * 12.0, cy + 12.0 + sin(ang) * 6.0);
            let c = if spent { alpha(PALE, 0.3) } else { HOLY };
            fx(FX_LINE, sx - 2.5, sy - 2.0, sx + 2.5, sy + 2.0, 2.4, c);
            fx(FX_DISC, sx, sy, 4.0, 0.0, 0.0, alpha(c, 0.5));
        }
        PJ_HALO => {
            let (cx, cy) = player_pos(v, owner).unwrap_or((x, y));
            let sz = (a as f32 / 100.0).max(4.0);
            let hy = cy + 22.0 + 0.8 * sin(t * 0.08);
            for j in 0..14 {
                let ang = j as f32 / 14.0 * TAU;
                fx(FX_DISC, cx + cos(ang) * sz, hy + sin(ang) * sz * 0.3, 1.8, 0.0, 0.0, alpha(0xFFF0_A0FF, 0.95));
            }
            fx(FX_DISC, cx, hy, sz * 1.4, 0.0, 0.0, alpha(HOLY, 0.25));
        }
        PJ_WINGS => {
            let (cx, cy) = player_pos(v, owner).unwrap_or((x, y));
            let face = if a < 0 { -1.0 } else { 1.0 };
            let flap = 2.0 * sin(t * 0.15);
            let c = if bb == 1 { alpha(0x6A30_70FF, 0.5) } else { 0x9A20_78FF };
            for side in [-1.0f32, 1.0] {
                let bx = cx - face * 2.0;
                let by = cy + 13.0;
                for j in 0..4 {
                    let tt = j as f32 / 3.0;
                    let wx = bx + side * (3.0 + 9.0 * tt);
                    let wy = by + 6.0 * tt + flap * tt - 5.0 * tt * tt;
                    fx(FX_LINE, bx, by, wx, wy - 4.0 * tt, 2.0 - tt, c);
                }
                fx(FX_LINE, bx, by, bx + side * 12.0, by + 5.0 + flap, 0.8, alpha(0xE060_A0FF, 0.8));
            }
        }
        PJ_ORB => {
            // an orb: each family its own look; shattered (life word 1): faint shards waiting to come back
            let fam = bb;
            let c = col(fam);
            let broken = v[V_PROJ + i * PJ_W + 5] == 1;
            if broken {
                for j in 0..3 {
                    let ang = j as f32 * 2.094 + t * 0.02;
                    fx(FX_SPIKES, x + cos(ang) * 2.5, y + sin(ang) * 2.5, 0.3, 1.4, ang, alpha(c, 0.3));
                }
            } else {
                match fam {
                    IT_DODGE_ORB => {
                        fx(FX_DISC, x, y, 5.0, 0.0, 0.0, alpha(c, 0.55));
                        fx(FX_RING, x, y, 3.2, 1.0, 0.0, alpha(WHITE, 0.9));
                        fx(FX_LINE, x, y, x - 6.0, y, 2.0, alpha(c, 0.4));
                    }
                    IT_NAIR_RING => {
                        fx(FX_DISC, x, y, 3.6, 0.0, 0.0, alpha(c, 0.8));
                        fx(FX_SPIKES, x, y, 1.0, 4.0, t * 0.3 + a as f32, alpha(0xFFE0_A0FF, 0.9));
                    }
                    IT_DASH_ORB => {
                        fx(FX_DISC, x, y, 4.2, 0.0, 0.0, alpha(c, 0.7));
                        fx(FX_RING, x, y, 2.8, 1.2, 0.0, alpha(WHITE, 0.8));
                        fx(FX_LINE, x - 3.0, y + 1.5, x - 9.0, y + 1.5, 0.8, alpha(c, 0.6));
                        fx(FX_LINE, x - 3.0, y - 1.5, x - 8.0, y - 1.5, 0.8, alpha(c, 0.6));
                    }
                    IT_SHIELD_ORBS => {
                        fx(FX_DISC, x, y, 3.4, 0.0, 0.0, alpha(c, 0.7));
                        fx(FX_RING, x, y, 3.0, 0.8, 0.0, alpha(0xE8D8_FFFF, 0.95));
                        fx(FX_SPIKES, x, y, 0.6, 2.4, 0.785, alpha(WHITE, 0.7));
                    }
                    _ => {
                        fx(FX_DISC, x, y, 3.0, 0.0, 0.0, alpha(col(IT_MOON_DROPS), 0.95));
                        fx(FX_RING, x, y, 3.6, 0.7, 0.0, alpha(0xA0B8_FFFF, 0.7));
                    }
                }
            }
        }
        PJ_THORNS => {
            let face = if a < 0 { -1.0 } else { 1.0 };
            let sz = (bb as f32 / 100.0).max(3.0) * ease(k * 4.0);
            for j in 0..4 {
                let ang = 1.2 + j as f32 * 0.35;
                let (tx, ty) = (x - face * cos(ang) * sz, y + 6.0 + sin(ang) * sz);
                fx(FX_LINE, x - face * 2.0, y + 6.0 + j as f32 * 1.5, tx, ty, 1.6, alpha(0x40A0_40FF, fade));
                fx(FX_SPIKES, tx, ty, 0.3, 1.6, 0.0, alpha(0xC0FF_A0FF, fade));
            }
        }
        _ => {}
    }
    look_flags(flags, x, y, vx, vy, fade, kind);
}

/// The modifiers' looks on any projectile.
#[allow(clippy::too_many_arguments)]
unsafe fn look_flags(flags: i32, x: f32, y: f32, vx: f32, vy: f32, fade: f32, kind: i32) {
    if matches!(kind, PJ_SIGIL | PJ_GUARD | PJ_HALO | PJ_WINGS | PJ_COIN | PJ_ARC | PJ_TENDRIL) {
        return;
    }
    let t = T as f32;
    if flags & PF_ICE != 0 {
        fx(FX_SPIKES, x, y, 1.0, 4.5, 0.4, alpha(ICE, 0.85 * fade));
        fx(FX_RING, x, y, 3.0, 0.7, 0.0, alpha(WHITE, 0.7 * fade));
    }
    if flags & PF_CHAIN != 0 && (T / 3) % 2 == 0 {
        let a = t * 1.7;
        fx(FX_BOLT, x, y, x + cos(a) * 6.0, y + sin(a) * 6.0, 0.7, alpha(0xC0E0_FFFF, fade));
    }
    if flags & PF_TWIN != 0 {
        fx(FX_DISC, x - vy * 1.5, y + vx * 1.5 + 2.0, 1.6, 0.0, 0.0, alpha(WHITE, 0.6 * fade));
    }
    if flags & PF_BOUNCY != 0 {
        let sq = 1.0 + 0.4 * sin(t * 0.6);
        fx(FX_LINE, x - 3.0 * sq, y, x + 3.0 * sq, y, 2.0 / sq, alpha(0x80FF_A0FF, 0.7 * fade));
    }
    if flags & PF_WELL != 0 {
        let a = t * 0.3;
        fx(FX_DISC, x + cos(a) * 4.0, y + sin(a) * 4.0, 1.2, 0.0, 0.0, alpha(0xB080_FFFF, fade));
    }
    if flags & PF_BIG != 0 {
        fx(FX_RING, x, y, 6.0, 1.2, 0.0, alpha(0xFFA0_40FF, 0.6 * fade));
    }
    if flags & PF_HALO != 0 {
        fx(FX_DISC, x, y, 6.0, 0.0, 0.0, alpha(HOLY, 0.55 * fade));
        fx(FX_RING, x, y, 4.0, 0.6, 0.0, alpha(0xFFF0_A0FF, fade));
    }
}

/// Slice 2's bursts (by item id) and the modifier looks on any burst.
#[allow(clippy::too_many_arguments)]
unsafe fn burst2(id: i32, x: f32, y: f32, size: f32, fl: i32, age: i32, k: f32, a: f32) {
    let c = col(id);
    match id {
        IT_LASER_PRISM => {
            for j in 0..3 {
                let ang = -0.35 + j as f32 * 0.35;
                let d = size * 1.5 * ease(k * 3.0);
                fx(FX_LINE, x, y, x + cos(ang) * d, y + sin(ang) * d, 1.2, alpha([0xFF50_50FF, 0x60FF_80FF, 0x60A0_FFFF][j], a));
            }
            fx(FX_SPIKES, x, y, 1.0, 5.0, 0.0, alpha(WHITE, a));
        }
        IT_CROUCH_THORNS => fx(FX_SPIKES, x, y, 2.0, size * (0.6 + 0.6 * ease(k * 3.0)), 0.2, alpha(c, a)),
        IT_PERFECT_GUARD => {
            fx(FX_RING, x, y, size * (0.4 + 0.6 * ease(k * 3.0)), 2.0, 0.0, alpha(0xFFE0_60FF, a));
            if age < 6 {
                fx(FX_DISC, x, y, size * 0.8, 0.0, 0.0, alpha(WHITE, 0.8));
            }
        }
        IT_BRIMSTONE_JAB => {
            fx(FX_DISC, x, y, size * (0.5 + k), 0.0, 0.0, alpha(BLOOD, 0.7 * a));
            fx(FX_SPIKES, x, y, 2.0, size * 1.3 * ease(k * 2.0), 0.0, alpha(0xFF30_40FF, a));
        }
        IT_DEMON_WINGS => fx(FX_RING, x, y, 4.0 + size * k, 1.6, 0.0, alpha(0xB030_90FF, a)),
        IT_CRIMSON_COUNT => {
            fx(FX_RING, x, y, size * ease(k * 2.0), 2.2, 0.0, alpha(0xFF30_40FF, a));
            fx(FX_SPIKES, x, y, size * 0.3, size * 0.9, k * 3.0, alpha(BLOOD, a));
        }
        IT_BLOODLUST => {
            for j in 0..5 {
                let ang = j as f32 * 1.256 + 0.4;
                let d = size * 1.4 * ease(k * 2.5);
                fx(FX_BOLT, x, y, x + cos(ang) * d, y + sin(ang) * d, 1.4, alpha(BLOOD, a));
            }
        }
        IT_HALO => fx(FX_RING, x, y, size * (0.5 + k), 2.0, 0.0, alpha(HOLY, a)),
        IT_GUARDIAN_FEATHER => {
            fx(FX_DISC, x, y, size * (1.0 + k), 0.0, 0.0, alpha(HOLY, 0.8 * a));
            fx(FX_RING, x, y, size * (0.6 + 1.2 * ease(k * 2.0)), 2.6, 0.0, alpha(WHITE, a));
            fx(FX_SPIKES, x, y, size * 0.4, size * 1.6, 0.0, alpha(0xFFF0_C0FF, a));
        }
        IT_ICE_GLASS => {
            for j in 0..3 {
                let ang = j as f32 * 2.094 + 0.5;
                let d = size * ease(k * 3.0);
                fx(FX_SPIKES, x + cos(ang) * d, y + sin(ang) * d, 0.6, 3.5, ang, alpha(ICE, a));
            }
            fx(FX_RING, x, y, size * 0.5 * (1.0 + k), 1.0, 0.0, alpha(WHITE, a));
        }
        IT_CHAIN_SPARK => fx(FX_SPIKES, x, y, 1.0, size * (0.5 + k), k * 6.0, alpha(0xC0E0_FFFF, a)),
        IT_TWIN_SHOT => {
            fx(FX_DISC, x - 3.0, y, size * 0.5 * a, 0.0, 0.0, alpha(c, a));
            fx(FX_DISC, x + 3.0, y, size * 0.5 * a, 0.0, 0.0, alpha(c, a));
        }
        IT_GRAVITY_WELL => {
            let r = if k < 0.4 { size * 1.4 * (1.0 - k / 0.4) } else { size * 1.6 * ease((k - 0.4) / 0.6) };
            fx(FX_RING, x, y, r.max(1.0), 2.0, 0.0, alpha(0xB080_FFFF, a));
            fx(FX_DISC, x, y, 4.0, 0.0, 0.0, alpha(0x6030_C0FF, a));
        }
        IT_BIG_BANG => {
            fx(FX_DISC, x, y, size * (0.8 + 1.2 * k), 0.0, 0.0, alpha(0xFFB0_40FF, 0.8 * a));
            fx(FX_RING, x, y, size * 2.2 * ease(k * 2.0), 3.0, 0.0, alpha(WHITE, a));
            fx(FX_SPIKES, x, y, size * 0.5, size * 2.6 * ease(k * 2.0), 0.2, alpha(0xFF70_20FF, a));
        }
        IT_BOUNCY => fx(FX_LINE, x - size * (0.5 + k), y, x + size * (0.5 + k), y, 3.0 * a + 0.2, alpha(0x80FF_A0FF, a)),
        IT_SPARE_HEART => {
            fx(FX_RING, x, y + 8.0, 6.0 + 20.0 * ease(k), 2.4, 0.0, alpha(CORAL, a));
            fx(FX_DISC, x - 2.0, y + 14.0 + 10.0 * k, 3.0, 0.0, 0.0, alpha(0xFF60_80FF, a));
            fx(FX_DISC, x + 2.0, y + 14.0 + 10.0 * k, 3.0, 0.0, 0.0, alpha(0xFF60_80FF, a));
        }
        _ => {}
    }
    if fl & PF_ICE != 0 {
        fx(FX_SPIKES, x, y, size * 0.3, size * 0.9, 0.4, alpha(ICE, 0.7 * a));
    }
    if fl & PF_CHAIN != 0 && age % 4 < 2 {
        fx(FX_BOLT, x - size, y, x + size, y + 2.0, 1.0, alpha(0xC0E0_FFFF, a));
    }
    if fl & PF_TWIN != 0 {
        fx(FX_RING, x + 4.0, y + 2.0, size * 0.6 * (0.5 + k), 1.0, 0.0, alpha(c, 0.6 * a));
    }
    if fl & PF_BOUNCY != 0 {
        fx(FX_LINE, x - size * 0.9, y - 1.0, x + size * 0.9, y - 1.0, 1.5, alpha(0x80FF_A0FF, a));
    }
    if fl & PF_WELL != 0 && k < 0.4 {
        fx(FX_RING, x, y, size * 1.5 * (1.0 - k / 0.4) + 1.0, 1.4, 0.0, alpha(0xB080_FFFF, a));
    }
    if fl & PF_BIG != 0 {
        fx(FX_RING, x, y, size * 2.0 * ease(k * 2.0), 2.4, 0.0, alpha(0xFFA0_40FF, a));
    }
    if fl & PF_HALO != 0 {
        fx(FX_DISC, x, y, size * 0.9 * a, 0.0, 0.0, alpha(HOLY, 0.6 * a));
    }
}

/// Orb shatter pops and block flashes (EV_ORB_BREAK / EV_ORB_BLOCK) round the owner, in the family's colour.
unsafe fn orb_events(v: &[i32; VW]) {
    let n = v[V_EV_N].max(0) as usize;
    for i in 0..EV.min(n) {
        let ev = &v[V_EV + i * EV_W..V_EV + (i + 1) * EV_W];
        let age = v[V_PLAY] - ev[0];
        if !(0..18).contains(&age) || !(ev[1] == EV_ORB_BREAK || ev[1] == EV_ORB_BLOCK) {
            continue;
        }
        let Some((x, y)) = player_pos(v, ev[2].clamp(0, 1) as usize) else { continue };
        let (y, k) = (y + 10.0, age as f32 / 18.0);
        let c = col(ev[3]);
        if ev[1] == EV_ORB_BREAK {
            for j in 0..6 {
                let ang = j as f32 * 1.047 + 0.3;
                let d = 4.0 + 14.0 * ease(k * 2.0);
                fx(FX_SPIKES, x + cos(ang) * d, y + sin(ang) * d - 6.0 * k * k, 0.3, 2.0 * (1.0 - k) + 0.4, ang, alpha(c, 1.0 - k));
            }
            fx(FX_RING, x, y, 4.0 + 10.0 * ease(k * 2.0), 1.4, 0.0, alpha(WHITE, 1.0 - k));
        } else {
            fx(FX_DISC, x, y, 12.0 * (1.0 - k) + 2.0, 0.0, 0.0, alpha(WHITE, 0.7 * (1.0 - k)));
            fx(FX_RING, x, y, 8.0 + 6.0 * k, 2.4, 0.0, alpha(c, 1.0 - k));
        }
    }
}
