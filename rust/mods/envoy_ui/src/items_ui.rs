//! Envoy items on screen (envoy-items-design.md 6, the first slice): every item's signature effect in the world
//! (sparks, a burning wake, the shine bell, stars, ice spikes, a ghost, mines, quakes, moons, coins; Homing Ink's
//! trail and Fire Lace's embers on everything), the pedestals of the item room and the boss drop, the new-item flash,
//! the item room / boss drop / campfire banners and the item strip. Presentation only: it reads the sim's view.
use crate::kit::*;
use crate::vfx;
use crate::{Buf, VW};
use envoy_data::items::*;
use envoy_data::view::*;
use gd_ui_enc::*;

const INK_COL: u32 = 0xB04A_FFFF;
const EMBER: u32 = 0xFF7A_20FF;

fn col(id: i32) -> u32 {
    item(id).map_or(WHITE, |it| it.rgba)
}

fn pb(p: usize) -> usize {
    V_P0 + p * V_PW
}

static mut T: u32 = 0;
/// Where each projectile slot was last present (its kind too): the direction a trail points back along.
static mut LASTP: [(i32, f32, f32); PROJ] = [(0, 0.0, 0.0); PROJ];

/// Every item effect this frame, from the view's projectile list and burst ring: shader quads (vfx.rs), each item
/// its own silhouette and colours.
pub unsafe fn world(v: &[i32; VW]) {
    if v[V_ITEMS_ON] != 1 {
        return;
    }
    T = T.wrapping_add(1);
    let t = T as f32;
    let play = v[V_PLAY];
    let mut live = true;
    for i in 0..PROJ {
        let b = V_PROJ + i * PJ_W;
        let kind = if live { v[b] } else { 0 };
        if kind == 0 {
            live = false;
            LASTP[i] = (0, 0.0, 0.0);
            continue;
        }
        let flags = v[b + 1] & PF_MASK;
        let (x, y) = (v[b + 2] as f32 / 100.0, v[b + 3] as f32 / 100.0);
        let (age, life, a, bb) = (v[b + 4], v[b + 5].max(1), v[b + 6], v[b + 7]);
        let fade = 1.0 - ((age as f32 - life as f32 + 8.0) / 8.0).clamp(0.0, 1.0);
        let (vx, vy) = (a as f32 / 100.0, bb as f32 / 100.0);
        let sd = i as f32 * 7.3 + 1.0;
        let af = age as f32;
        // how it moved since the last present (orbiters and coins carry no velocity in the view)
        let (lk, lx, ly) = LASTP[i];
        let (dx, dy) = if lk == kind && (lx - x).abs() + (ly - y).abs() < 30.0 { (x - lx, y - ly) } else { (0.0, 0.0) };
        LASTP[i] = (kind, x, y);
        match kind {
            PJ_SPARK => {
                // Spark Ring sparks: a spinning electric star on a short hot tail (the look the item had, kept)
                let c = col(IT_SPARK_RING);
                let fade = fade * (af / 5.0).min(1.0);
                vfx::fx(vfx::TRAIL, x - vx * 4.5, y - vy * 4.5, 0.0, 0.0).line(x + vx * 0.6, y + vy * 0.6, 3.4).cols(0xFFF6_B0FF, c).a(0.9 * fade).age(t).seed(sd).emit();
                vfx::fx(vfx::STAR, x, y, 9.5, 9.5).cols(vfx::pale(c, 0.4), c).p(1.0, 9.0, 0.8, 0.0).age(t).seed(sd).a(fade).emit();
                vfx::fx(vfx::BOLT, x - 3.2, y + 1.0, 0.0, 0.0).line(x + 3.2, y - 1.0, 3.0).cols(0xFFFF_FFFF, c).p(1.0, 0.0, 30.0, 0.8).age(t).seed(sd).a(0.7 * fade).emit();
            }
            PJ_STAR => {
                // Hop Shot: a big slow sparkle with a soft comet tail
                let c = col(IT_HOP_SHOT);
                vfx::fx(vfx::TRAIL, x - vx * 6.0, y - vy * 6.0, 0.0, 0.0).line(x + vx * 0.5, y + vy * 0.5, 6.0).cols(c, vfx::deep(c, 0.3)).a(0.75 * fade).age(t).seed(sd).emit();
                vfx::fx(vfx::STAR, x, y, 17.0, 17.0).cols(0xFFF0_A0FF, c).p(0.7, 2.5, 0.6, 0.0).age(t).seed(sd).a(fade).emit();
            }
            PJ_WAKE => {
                // Glide Wake: a wall of fire along the ground, embers over it, the air above it shimmering
                let half = a as f32 / 100.0;
                let k = (life - age) as f32 / life as f32;
                let c = col(IT_GLIDE_WAKE);
                let h = 9.0 * (0.55 + 0.45 * k);
                vfx::fx(vfx::FLAME, x, y + h * 0.5 - 0.3, half * 2.0 + 5.0, h).cols(0xFFA0_30FF, c).p(1.0, 0.0, 0.3 * k, 0.0).age(t).seed(sd).a(0.95 * k.min(1.0).max(0.25)).emit();
                vfx::fx(vfx::EMBERS, x, y + 8.0, half * 2.0 + 4.0, 15.0).cols(0xFFB0_40FF, c).p(5.0, 1.2, 0.0, 0.0).age(t).seed(sd).a(k).emit();
                vfx::fx(vfx::HAZE, x, y + 9.0, half * 2.0 + 10.0, 20.0).p(1.0 * k, 0.0, 0.0, 0.0).age(t).seed(sd).bend().emit();
            }
            PJ_GHOST => {
                // Whiff Echo: a spectre waiting where the swing missed, flaring as its strike comes due
                let face = if a < 0 { -1.0 } else { 1.0 };
                let c = col(IT_WHIFF_ECHO);
                let due = ((af - life as f32 + 14.0) / 14.0).clamp(0.0, 1.0);
                vfx::fx(vfx::WISP, x, y + 9.5, 13.0, 21.0).cols(c, vfx::pale(c, 0.3)).p(face, due, 0.0, 0.0).age(t).seed(sd).a(0.85 * fade.max(due)).emit();
                vfx::fx(vfx::RING, x, y + 0.6, 15.0, 4.6).cols(c, c).p(0.8, 0.05, 12.0, 0.6).age(t).a(0.5 * fade).back().emit();
            }
            PJ_MINE => {
                let blink = if (age / 6) % 2 == 0 || age > life - 18 && (age / 3) % 2 == 0 { 1.0 } else { 0.12 };
                let armed = if age > life - 18 { 1.0 } else { 0.0 };
                let ring = (age % 36) as f32 / 36.0;
                vfx::fx(vfx::MINE, x, y + 3.0, 9.6, 9.6).cols(col(IT_ROLL_MINES), 0xFFB0_B0FF).p(blink, ring, armed, 0.0).age(t).seed(sd).emit();
            }
            PJ_QUAKE => {
                // Smash Quake: the ground tearing open under a running wall of dust and thrown rock
                let size = bb as f32 / 100.0;
                let dir = if a < 0 { -1.0 } else { 1.0 };
                let c = col(IT_SMASH_QUAKE);
                vfx::fx(vfx::CRACK, x - dir * size * 1.2, y + 1.6, size * 4.6, size * 1.9).cols(0xFFA0_40FF, 0xC8A8_78FF).p(1.0, 0.9 * fade, 1.0, 0.0).age(t).seed(sd).a(fade).back().emit();
                vfx::fx(vfx::SPARKS, x, y + 1.0, size * 3.4, size * 3.4).rot(1.5708).cols(0xE0C8_A0FF, c).p((age % 14) as f32 / 14.0, 14.0, 1.6, 0.4).seed(sd + (age / 14) as f32).a(fade).emit();
            }
            PJ_COIN => {
                let c = col(IT_COIN_MAGNET);
                if dx * dx + dy * dy > 0.02 {
                    vfx::fx(vfx::TRAIL, x - dx * 5.0, y - dy * 5.0, 0.0, 0.0).line(x, y, 2.6).cols(0xFFF4_C0FF, c).a(0.6).age(t).seed(sd).emit();
                }
                vfx::fx(vfx::COIN, x, y, 5.6, 5.6).cols(c, 0xFFF4_C0FF).p(1.6, 1.0, 0.0, 0.0).age(t).seed(sd).emit();
            }
            PJ_MOON => {
                // Moon Drops: crescent moons of cold light in orbit, each trailing a wake
                let c = col(IT_MOON_DROPS);
                if dx * dx + dy * dy > 0.01 {
                    vfx::fx(vfx::TRAIL, x - dx * 7.0, y - dy * 7.0, 0.0, 0.0).line(x, y, 4.6).cols(c, 0x6080_FFFF).a(0.55).age(t).seed(sd).emit();
                }
                vfx::fx(vfx::ORB, x, y, 10.5, 10.5).cols(vfx::pale(c, 0.25), 0x7090_FFFF).p(0.8, 0.85, 4.0, 0.0).age(t).seed(sd).emit();
                vfx::fx(vfx::STAR, x - 1.4, y + 1.6, 4.5, 4.5).cols(0xFFFF_FFFF, c).p(1.6, 0.0, 0.0, 0.0).age(t).seed(sd).a(0.7).emit();
            }
            _ => {}
        }
        if flags & PF_INK != 0 && matches!(kind, PJ_SPARK | PJ_STAR) {
            // Homing Ink: a long violet ribbon that snakes behind anything it steers
            vfx::fx(vfx::TRAIL, x - vx * 11.0, y - vy * 11.0, 0.0, 0.0).line(x, y, 5.0).cols(INK_COL, 0x5A20_C0FF).p(1.0, 0.7, 0.0, 0.0).a(0.8 * fade).age(t).seed(sd + 9.0).emit();
        }
        if flags & PF_LACE != 0 && kind != PJ_COIN {
            // Fire Lace: everything burns
            vfx::fx(vfx::FLAME, x, y + 2.6, 4.6, 7.5).cols(0xFFB0_40FF, EMBER).p(1.0, -vx * 0.25, 0.4, 0.0).age(t).seed(sd + 5.0).a(0.9 * fade).emit();
            vfx::fx(vfx::EMBERS, x, y + 5.0, 6.0, 11.0).cols(0xFFC0_60FF, EMBER).p(3.0, 1.4, 0.0, 0.0).age(t).seed(sd + 5.0).a(0.8 * fade).emit();
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
        let af = age as f32;
        let k = af / 32.0;
        let a = 1.0 - k;
        let c = col(id);
        let sd = (v[b] % 97) as f32 + e as f32 * 3.0;
        match id {
            IT_SPARK_RING => {
                // the ring and its white pop (kept), now a live wire: arcs ride the ring, sparks fly, the air bends
                let r = 2.0 + size * 1.6;
                vfx::burst_ring(x, y, r, (k * 1.5).min(1.0), c, 1.0, af);
                vfx::pop(x, y, 6.0, af, 6.0, WHITE);
                vfx::sparks(x, y, r * 1.1, (k * 1.6).min(1.0), 14.0, c, sd);
            }
            IT_GLIDE_WAKE => {
                vfx::fx(vfx::FLAME, x, y + 6.0, 9.0, 13.0).cols(0xFFB0_40FF, c).p(0.4 + 0.6 * a, 0.0, 0.6 * a, 0.0).age(af).seed(sd).a(a).emit();
                vfx::fx(vfx::SPARKS, x, y + 1.5, 16.0, 16.0).rot(1.5708).cols(0xFFD0_70FF, c).p(k, 12.0, 0.6, 0.5).seed(sd).emit();
            }
            IT_SHINE_BELL => {
                // a hexagonal shield ringing out, a white ring inside it, a glint at the heart
                vfx::fx(vfx::DOME, x, y, size * 2.3, size * 2.3).cols(c, vfx::deep(c, 0.3)).p((k * 1.3).min(1.0), 5.0, 0.0, 1.0).age(af).seed(sd).bend().emit();
                vfx::fx(vfx::SHOCK, x, y, size * 1.5, size * 1.5).cols(WHITE, c).p((k * 1.6).min(1.0), 0.05, 0.0, 0.0).age(af).seed(sd).emit();
                if age < 10 {
                    vfx::fx(vfx::STAR, x, y, size * 1.6, size * 1.6).cols(WHITE, c).p(1.2, 0.0, 0.5, 0.0).a(1.0 - af / 10.0).emit();
                }
            }
            IT_HOP_SHOT => {
                vfx::fx(vfx::STAR, x, y, size * (1.0 + 2.0 * k), size * (1.0 + 2.0 * k)).cols(0xFFF0_A0FF, c).p(0.8, 3.0, 0.7, 0.0).age(af).seed(sd).a(a).emit();
                vfx::fx(vfx::SHOCK, x, y, size * 1.8, size * 0.7).cols(c, c).p((k * 2.0).min(1.0), 0.06, 0.0, 0.0).age(af).a(a).back().emit();
            }
            IT_LEDGE_SPIKES => {
                // ice bursts up the ledge, stands, then shatters
                let grow = ease(k * 4.0);
                let shatter = ((k - 0.72) / 0.28).clamp(0.0, 1.0);
                vfx::fx(vfx::ICE, x, y + size * 0.5, size * 0.95 + 6.0, size).cols(0xD8FF_FFFF, c).p(grow, shatter, 0.0, 0.0).age(af).seed(sd).emit();
                vfx::pop(x, y + 1.5, 7.0, af, 8.0, 0xD8FF_FFFF);
                if shatter > 0.0 {
                    vfx::fx(vfx::SPARKS, x, y + size * 0.55, size * 1.5, size * 1.5).cols(WHITE, c).p(shatter, 22.0, 1.4, 0.0).seed(sd).emit();
                }
            }
            IT_WHIFF_ECHO => {
                // the swing of the echo: a crescent the way it faced, and the spectre flaring out
                let dir = if fl & 3 == 0 { -1.0 } else { 1.0 };
                vfx::fx(vfx::SLASH, x, y, size * 2.3, size * 2.3).rot(if dir < 0.0 { 3.14159 } else { 0.0 }).cols(c, vfx::deep(c, 0.3)).p((k * 1.8).min(1.0), 0.42, 1.2, 0.0).age(af).seed(sd).emit();
                vfx::fx(vfx::WISP, x - dir * size * 0.3, y + 2.0 + k * 5.0, 13.0 * (1.0 + k * 0.4), 21.0 * (1.0 + k * 0.4)).cols(c, WHITE).p(dir, a, 0.0, 0.0).age(af).seed(sd).a(a * a).emit();
                vfx::sparks(x + dir * size * 0.6, y, size * 0.9, (k * 1.5).min(1.0), 12.0, c, sd);
            }
            IT_ROLL_MINES => {
                // the blast: a fireball, a ring that bends the air, shrapnel
                let r = size * (0.7 + 0.7 * ease(k * 3.0));
                vfx::fx(vfx::GLOW, x, y, r * 2.4, r * 2.4).cols(0xFFB0_40FF, c).p(2.4, 1.0, 0.0, 0.0).a(a * a).emit();
                vfx::fx(vfx::EMBERS, x, y + r * 0.6, r * 2.2, r * 2.6).cols(0xFFC0_60FF, c).p(6.0, 1.6, 0.0, 0.0).age(af).seed(sd).a(a).emit();
                vfx::burst_ring(x, y, size * 1.7, (k * 1.4).min(1.0), 0xFFD0_80FF, 0.0, af);
                vfx::fx(vfx::SPARKS, x, y, size * 3.4, size * 3.4).cols(0xFFE0_A0FF, c).p(k, 28.0, 1.2, 0.0).seed(sd).emit();
            }
            IT_SMASH_QUAKE => {
                // the slam: the floor splits both ways from the hit, dust and rock jump, a low ring rolls out
                vfx::fx(vfx::CRACK, x, y + 1.6, size * 5.2, size * 2.0).cols(0xFFA0_40FF, 0xC8A8_78FF).p(ease(k * 3.0), a, a, 0.0).age(af).seed(sd).a((a * 2.0).min(1.0)).back().emit();
                vfx::fx(vfx::SHOCK, x, y + 0.8, size * 4.4, size * 1.1).cols(0xE8D0_A0FF, 0x8A5A_20FF).p((k * 1.5).min(1.0), 0.07, 0.0, 1.0).age(af).seed(sd).bend().emit();
                vfx::fx(vfx::SPARKS, x, y + 1.0, size * 3.6, size * 3.6).rot(1.5708).cols(0xE0C8_A0FF, c).p(k, 20.0, 1.8, 0.45).seed(sd).emit();
            }
            IT_MOON_DROPS => {
                // a moon nicks a foe: a cold ripple and a glint
                vfx::fx(vfx::SHOCK, x, y, 20.0, 20.0).cols(vfx::pale(c, 0.3), 0x6080_FFFF).p(k, 0.05, 0.0, 0.0).age(af).seed(sd).emit();
                vfx::fx(vfx::STAR, x, y, 11.0 * a + 3.0, 11.0 * a + 3.0).cols(WHITE, c).p(1.2, 4.0, 0.0, 0.0).age(af).a(a).emit();
                vfx::sparks(x, y, 9.0, k, 9.0, 0xA0B8_FFFF, sd);
            }
            IT_COIN_MAGNET => {
                // the payout: gold sparks and a glint
                vfx::fx(vfx::SPARKS, x, y, size * 2.6, size * 2.6).cols(0xFFF4_C0FF, c).p(k, 18.0, 0.8, 0.0).seed(sd).emit();
                vfx::fx(vfx::STAR, x, y, size * 1.6 * a + 3.0, size * 1.6 * a + 3.0).cols(0xFFF4_C0FF, c).p(1.0, 2.0, 0.6, 0.0).age(af).a(a).emit();
                vfx::pop(x, y, size * 0.7, af, 8.0, 0xFFF4_C0FF);
            }
            _ => {}
        }
        if fl & PF_LACE != 0 {
            // Fire Lace on a burst: tongues of fire and embers rising off it
            vfx::fx(vfx::FLAME, x, y + size * 0.45, size * 0.9, size * 1.5).cols(0xFFB0_40FF, EMBER).p(0.5 + 0.5 * a, 0.0, 0.3, 0.0).age(af).seed(sd + 2.0).a(a).emit();
            vfx::fx(vfx::EMBERS, x, y + size * 0.7, size * 1.6, size * 2.0).cols(0xFFC0_60FF, EMBER).p(5.0, 1.5, 0.0, 0.0).age(af).seed(sd).a(a).emit();
        }
    }
}

/// The pedestals (the item room and the boss drop) and their fly-in: a circle of the item colour turning on the
/// floor, a pillar of light in the owner colour (co-op), the item floating over it as a ball of its own light.
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
            let sd = (p * 8 + k) as f32 + 1.0;
            let moving = LAST[p][k].0 != 0.0 && ((LAST[p][k].0 - bx).abs() + (LAST[p][k].1 - by).abs()) > 0.3;
            let fy = by + 12.0 + 1.5 * sin(t * 0.07 + k as f32 * 1.3);
            if moving {
                let (lx, ly) = LAST[p][k];
                vfx::fx(vfx::TRAIL, lx + (lx - bx) * 4.0, ly + 12.0 + (ly - by) * 4.0, 0.0, 0.0).line(bx, fy, 8.0).cols(vfx::pale(c, 0.3), c).age(t).seed(sd).emit();
            } else {
                let beam = if humans == 2 { crate::PLAYER_COL[p] } else { c };
                vfx::ground_circle(bx, by + 0.6, 13.0, 1.0, 0.0, 0.4, t, c, sd);
                vfx::fx(vfx::BEAM, bx, by + 30.0, 15.0, 60.0).cols(beam, beam).p(0.35, 1.0, 0.0, 0.0).age(t).seed(sd).a(0.55).back().emit();
            }
            LAST[p][k] = (bx, by);
            vfx::fx(vfx::ORB, bx, fy, 15.0, 15.0).cols(vfx::pale(c, 0.15), c).p(1.0, 0.0, 3.0, 0.0).age(t).seed(sd).emit();
            vfx::fx(vfx::RING, bx, fy, 17.0, 17.0).cols(c, c).p(0.8, 0.03, 8.0, 0.8).age(t).a(0.8).emit();
            vfx::fx(vfx::STAR, bx - 2.4, fy + 2.6, 7.0, 7.0).cols(WHITE, c).p(1.5, 0.0, 0.0, 0.0).age(t).seed(sd).a(0.6 + 0.4 * sin(t * 0.2 + sd)).emit();
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
        // in the world: a circle in the item colour flares under the fighter, a pillar of its light, sparks going up
        if v[pb(p) + P_HAVE_POS] != 0 && age < 48 {
            let (fx_, fy) = (f32::from_bits(v[pb(p) + P_X] as u32), f32::from_bits(v[pb(p) + P_Y] as u32));
            let k = age as f32 / 48.0;
            let af = age as f32;
            vfx::ground_circle(fx_, fy + 0.6, 10.0 + 12.0 * ease(k * 2.0), (k * 3.0).min(1.0), 0.0, 1.0, af, alpha(it.rgba, 1.0 - k), p as f32 + 3.0);
            vfx::fx(vfx::BEAM, fx_, fy + 26.0, 16.0, 52.0 * (1.0 - k * 0.5)).cols(it.rgba, it.rgba).p(0.5, 1.0, 0.0, 0.0).age(af).a(0.7 * (1.0 - k)).back().emit();
            vfx::burst_ring(fx_, fy + 9.0, 8.0 + 20.0 * ease(k), (k * 1.6).min(1.0), it.rgba, 0.0, af);
            vfx::fx(vfx::SPARKS, fx_, fy + 6.0, 40.0, 40.0).rot(1.5708).cols(WHITE, it.rgba).p((k * 1.4).min(1.0), 16.0, 0.0, 0.5).seed(p as f32).emit();
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
