//! Envoy items on screen (envoy-items-design.md 6, the first slice): every item's signature effect in the world
//! (sparks, a burning wake, the shine bell, stars, ice spikes, a ghost, mines, quakes, moons, coins; Homing Ink's
//! trail and Fire Lace's embers on everything), the pedestals of the item room and the boss drop, the new-item flash,
//! the item room / boss drop / campfire banners and the item strip. Presentation only: it reads the sim's view.
use crate::kit::*;
use crate::vfx;
use crate::{FX_BEAM, FX_BOLT, FX_DISC, FX_LINE, FX_RING, FX_SPIKES};
const TAU: f32 = 6.283_185_3;

/// The older shape calls (slice 2 looks), drawn as shader quads through lib.rs's shim (vfx lane).
unsafe fn fx(kind: i32, x: f32, y: f32, a: f32, b: f32, c: f32, rgba: u32) {
    crate::fx(kind, x, y, 0.0, a, b, c, rgba);
}
use crate::{Buf, VW};
use envoy_data::items::*;
use envoy_data::procs;
use envoy_data::view::*;
use gd_ui_enc::*;

const INK_COL: u32 = 0xB04A_FFFF;
const EMBER: u32 = 0xFF7A_20FF;

fn col(id: i32) -> u32 {
    if (VFX_SYN..VFX_SYN + procs::SYNERGIES.len() as i32).contains(&id) {
        return procs::SYNERGIES[(id - VFX_SYN) as usize].rgba;
    }
    if (VFX_EVO..VFX_EVO + procs::EVOLUTIONS.len() as i32).contains(&id) {
        return procs::EVOLUTIONS[(id - VFX_EVO) as usize].rgba;
    }
    item(id).map_or(WHITE, |it| it.rgba)
}

/// The sim's burst ids for the synergy lane's own visuals (envoy_sim items.rs VFX_SYN / VFX_EVO).
const VFX_SYN: i32 = 100;
const VFX_EVO: i32 = 120;

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
            PJ_TETHER => {
                // Kin Tether: a light line between the partners with beads running along it
                let (bx, by) = (a as f32 / 100.0, bb as f32 / 100.0);
                let c = col(IT_KIN_TETHER);
                fx(FX_LINE, x, y, bx, by, 1.1 + 0.4 * sin(t * 0.2), alpha(c, 0.55));
                for j in 0..3 {
                    let u = ((t * 0.012 + j as f32 / 3.0) % 1.0).abs();
                    fx(FX_DISC, x + (bx - x) * u, y + (by - y) * u, 1.6, 0.0, 0.0, alpha(WHITE, 0.8));
                }
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
            _ => proj2(v, i, kind, flags, x, y, age, life, a, bb, fade),
        }
        if kind <= PJ_MOON {
            look_flags(flags, x, y, vx, vy, fade, kind);
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
            // ---- the synergy lane
            IT_EMBER_SEED => {
                // a seed that splits into five petals of flame
                fx(FX_DISC, x, y, 2.5 * a, 0.0, 0.0, alpha(0xFFE0_80FF, a));
                for j in 0..5 {
                    let ang = j as f32 * TAU / 5.0 + k * 1.5;
                    let r = size * ease(k * 2.0);
                    fx(FX_SPIKES, x + cos(ang) * r, y + sin(ang) * r + k * 4.0, 0.5, 2.6 * a + 0.5, ang, alpha(c, a));
                }
            }
            IT_STATIC_COIL => {
                fx(FX_SPIKES, x, y, 0.8, size * (0.8 + 0.6 * k), T as f32 * 0.7, alpha(c, a));
                fx(FX_RING, x, y, 2.0 + size * 0.6 * k, 0.7, 0.0, alpha(WHITE, a));
            }
            IT_FROST_BUD => {
                // a frost flower: six ice spikes and a pale ring
                for j in 0..6 {
                    let ang = j as f32 * TAU / 6.0;
                    let r = size * 0.7 * ease(k * 3.0);
                    fx(FX_LINE, x, y, x + cos(ang) * r, y + sin(ang) * r, 1.0, alpha(c, a));
                }
                fx(FX_RING, x, y, size * ease(k * 2.0), 1.2, 0.0, alpha(0xE8F8_FFFF, a));
            }
            IT_KIN_TETHER => fx(FX_RING, x, y, 2.0 + size * k, 0.9, 0.0, alpha(c, a)),
            IT_RALLY_BANNER => {
                // a pennant raised over the partner
                let h = 10.0 * ease(k * 3.0);
                fx(FX_LINE, x, y - 6.0, x, y - 6.0 + h, 0.8, alpha(WHITE, a));
                fx(FX_BEAM, x + 3.0, y - 6.0 + h - 2.0, 4.0, 6.0, 0.0, alpha(c, a));
            }
            IT_KINDRED_SPARK => {
                fx(FX_SPIKES, x, y, 1.0, size * (0.6 + k), -(T as f32) * 0.3, alpha(c, a));
                fx(FX_RING, x, y, size * 0.5 * (1.0 + k), 0.8, 0.0, alpha(WHITE, 0.7 * a));
            }
            _ if (VFX_SYN..VFX_SYN + 20).contains(&id) => {
                // a synergy: a double ring in the synergy's colour, a white core flash, rays
                fx(FX_RING, x, y, size * ease(k * 2.0), 1.8, 0.0, alpha(c, a));
                fx(FX_RING, x, y, size * 0.6 * ease(k * 1.4), 1.0, 0.0, alpha(WHITE, 0.8 * a));
                fx(FX_SPIKES, x, y, size * 0.2, size * 0.8 * ease(k * 2.0), k * 2.0, alpha(c, 0.5 * a));
            }
            _ if (VFX_EVO..VFX_EVO + 20).contains(&id) => {
                // an evolution: a crown of eight rays, gold rimmed, and a slow ring
                for j in 0..8 {
                    let ang = j as f32 * TAU / 8.0 + k;
                    let r0 = size * 0.3;
                    let r1 = size * (0.5 + 0.6 * ease(k * 2.0));
                    fx(FX_LINE, x + cos(ang) * r0, y + sin(ang) * r0, x + cos(ang) * r1, y + sin(ang) * r1, 1.4, alpha(c, a));
                }
                fx(FX_RING, x, y, size * (0.4 + k), 2.0, 0.0, alpha(0xFFD8_60FF, a));
                fx(FX_DISC, x, y, size * 0.25 * a, 0.0, 0.0, alpha(WHITE, a));
            }
            _ => {}
        }
        burst2(id, x, y, size, fl, age, k, a);
        if fl & PF_LACE != 0 {
            // Fire Lace on a burst: tongues of fire and embers rising off it
            vfx::fx(vfx::FLAME, x, y + size * 0.45, size * 0.9, size * 1.5).cols(0xFFB0_40FF, EMBER).p(0.5 + 0.5 * a, 0.0, 0.3, 0.0).age(af).seed(sd + 2.0).a(a).emit();
            vfx::fx(vfx::EMBERS, x, y + size * 0.7, size * 1.6, size * 2.0).cols(0xFFC0_60FF, EMBER).p(5.0, 1.5, 0.0, 0.0).age(af).seed(sd).a(a).emit();
        }
    }
    // a partner's Rally Banner: a gold pennant over the rallied fighter while it lasts
    let humans = v[V_HUMANS].clamp(1, 2) as usize;
    for p in 0..humans {
        let left = v[V_RALLY + p];
        if left <= 0 || v[pb(p) + P_HAVE_POS] == 0 {
            continue;
        }
        let (fx_, fy) = (f32::from_bits(v[pb(p) + P_X] as u32), f32::from_bits(v[pb(p) + P_Y] as u32));
        let a = (left as f32 / 30.0).min(1.0);
        let wave = sin(t * 0.25) * 1.2;
        fx(FX_LINE, fx_, fy + 20.0, fx_, fy + 30.0, 0.7, alpha(WHITE, 0.8 * a));
        fx(FX_BEAM, fx_ + 3.0 + wave * 0.3, fy + 28.0, 5.0, 5.0 + wave, 0.0, alpha(col(IT_RALLY_BANNER), 0.85 * a));
    }
    let _ = FX_BOLT;
}

/// The pedestals (the item room and the boss drop) and their fly-in: a circle of the item colour turning on the
/// floor, a pillar of light in the owner colour (co-op), the item floating over it as a ball of its own light.
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
            if !moving {
                DOOR_SUM.0 += bx;
                DOOR_SUM.1 = DOOR_SUM.1.min(by);
                DOOR_SUM.2 += 1.0;
                ped_cost(v, p, k, bx, by + 22.0);
            }
            let m = item(id).map_or(-1, |it| crate::fx::em_model(it.key));
            if m >= 0 {
                // the item's own model (envoy-models1.md) rocking in the ring, ink edged
                vfx::fx(vfx::RING, bx, fy, 17.0, 17.0).cols(c, c).p(0.8, 0.03, 8.0, 0.8).age(t).a(0.8).emit();
                let yaw = 0.35 * sin(t * 0.045 + k as f32 * 1.3);
                crate::fx::model(m, bx, fy, 0.0, yaw, 16.0, 1.0, crate::fx::MAT_BODY | crate::fx::MAT_OUTLINE, [0.3, 0.0, 0.0, 0.0], 0xFFFF_FFFF);
            } else {
                vfx::fx(vfx::ORB, bx, fy, 15.0, 15.0).cols(vfx::pale(c, 0.15), c).p(1.0, 0.0, 3.0, 0.0).age(t).seed(sd).emit();
                vfx::fx(vfx::RING, bx, fy, 17.0, 17.0).cols(c, c).p(0.8, 0.03, 8.0, 0.8).age(t).a(0.8).emit();
                vfx::fx(vfx::STAR, bx - 2.4, fy + 2.6, 7.0, 7.0).cols(WHITE, c).p(1.5, 0.0, 0.0, 0.0).age(t).seed(sd).a(0.6 + 0.4 * sin(t * 0.2 + sd)).emit();
            }
        }
    }
}

/// The items player `p` holds, in id order: a coloured chip each, x2 on a stack (band space).
pub unsafe fn strip(e: &mut E, v: &[i32; VW], p: usize, x: f32, y: f32, w: f32) {
    let mut cx = x + 11.0;
    let mut row_y = y + 11.0;
    let mut any = false;
    for it in ITEMS.iter() {
        let n = v[inv_word(p, it.id as usize)];
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

/// The synergy / evolution banner: the first time a synergy fires (or an evolution forms) this run, its name and
/// keyword line slide in under the top for 2.5 s (a sticker in its colour; never a full-screen flash).
unsafe fn combo_banner(e: &mut E, v: &[i32; VW]) {
    let humans = v[V_HUMANS].clamp(1, 2) as usize;
    for p in 0..humans {
        let code = v[V_BANNER + p];
        let age = v[V_PLAY] - v[V_BANNER_AT + p];
        if code == 0 || v[V_BANNER_AT + p] <= 0 || !(0..150).contains(&age) {
            continue;
        }
        let (kind, k) = (code >> 8, (code & 0xFF) as usize);
        // the emblem: em_syn_<key>.png / em_evo_<key>.png (envoy-models1.md)
        let (head, name, line, rgba, pre, key) = if kind == BANNER_EVO {
            let Some(ev) = procs::EVOLUTIONS.get(k) else { continue };
            ("EVOLUTION", ev.name, ev.line, ev.rgba, "evo_", ev.key)
        } else {
            let Some(sy) = procs::SYNERGIES.get(k) else { continue };
            ("SYNERGY", sy.name, sy.line, sy.rgba, "syn_", sy.key)
        };
        let a = if age > 120 { (150 - age) as f32 / 30.0 } else { 1.0 };
        let slide = (1.0 - ease(age as f32 / 12.0)) * 40.0;
        let em = em_tex(pre, key) != 0;
        let ex = if em { 44.0 } else { 0.0 }; // room for the emblem left of the words
        let w = tw(name, d(20)).max(tw(line, bs(12))) + 60.0 + ex;
        let (x, y) = (320.0 - w * 0.5 + if p == 0 { -slide } else { slide }, 236.0 + p as f32 * 54.0);
        sticker(e, S_FACE_WHITE, x, y, w, 48.0, if p == 0 { 1.0 } else { -1.0 }, alpha(WHITE, a), |e| {
            e.rect(x + 2.0, y + 2.0, 8.0, 44.0, alpha(rgba, a));
            em_spr(e, pre, key, x + 34.0, y + 24.0, 44.0, alpha(WHITE, a));
            e.text_ex(x + 18.0 + ex, y + 4.0, bs(10), alpha(LILAC, a), ALIGN_LEFT, 0.0, head);
            e.text_ex(x + 18.0 + ex + tw(head, bs(10)) + 8.0, y + 1.0, d(20), alpha(INK, a), ALIGN_LEFT, 0.0, name);
            e.text_ex(x + 18.0 + ex, y + 27.0, bs(12), alpha(INK, a), ALIGN_LEFT, 0.0, line);
        });
    }
}

/// The room banners: the item room and the boss drop while their pedestals wait, the campfire after the boss.
pub unsafe fn banners(e: &mut E, v: &[i32; VW]) {
    combo_banner(e, v);
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
            // the campfire's own sprite, four frames (em_node_campfire_anim.png); the three flame stickers without it
            if !em_anim(e, "node_campfire_anim", 4, (age / 7) as u32, 320.0 - w * 0.5 + 24.0, 208.0, 36.0, alpha(WHITE, a)) {
                for j in 0..3 {
                    spr_c(e, S_FLAME + 1 + j.min(2), 320.0 - w * 0.5 + 24.0, 210.0 - j as f32 * 3.0, 0.5 + 0.1 * sin(age as f32 * 0.3 + j as f32), alpha(WHITE, a));
                }
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
            IT_EMBER_SEED => "fire_red",
            IT_STATIC_COIL => "fire_bolt",
            IT_FROST_BUD => "fire_purple",
            IT_KIN_TETHER => "fire_green",
            IT_RALLY_BANNER => "levelup",
            IT_KINDRED_SPARK => "fire_spire",
            id if (VFX_SYN..VFX_SYN + 20).contains(&id) => "keystone",
            id if (VFX_EVO..VFX_EVO + 20).contains(&id) => "fire_heal",
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
    let (w, h) = (18.0, 30.0);
    let devil = kind == PED_DEVIL;
    let (body, edge, glow) = if devil { (0x7A08_18FF, 0xFF30_40FF, 0xFF20_30FF) } else { (0xC8C8_D8FF, WHITE, HOLY) };
    // the door (vfx lane quads): a glow column, the doorway's haze, its edges and the arch
    vfx::fx(vfx::BEAM, cx, fy + h * 0.5, w + 14.0, h + 14.0).cols(glow, glow).p(0.35, 1.0, 0.0, 0.0).age(t).a(0.12 + 0.04 * sin(t * 0.08)).back().emit();
    vfx::fx(vfx::HAZE, cx, fy + h * 0.5, w, h).cols(body, body).age(t).a(0.4).emit();
    vfx::fx(vfx::BEAM, cx - w * 0.5, fy + h * 0.5, 1.8, h).cols(edge, edge).p(0.35, 1.0, 0.0, 0.0).age(t).emit();
    vfx::fx(vfx::BEAM, cx + w * 0.5, fy + h * 0.5, 1.8, h).cols(edge, edge).p(0.35, 1.0, 0.0, 0.0).age(t).emit();
    vfx::fx(vfx::RING, cx, fy + h, w, w).cols(edge, edge).p(0.8, 0.03, 8.0, 0.8).age(t).emit();
    fx(FX_LINE, cx - w * 0.5 - 3.0, fy, cx + w * 0.5 + 3.0, fy, 2.0, edge);
    if devil {
        fx(FX_LINE, cx - 9.0, fy + h + 8.0, cx - 14.0, fy + h + 17.0, 2.2, edge);
        fx(FX_LINE, cx + 9.0, fy + h + 8.0, cx + 14.0, fy + h + 17.0, 2.2, edge);
        
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
        crate::map::draw(e, v, x + 8.0, y + 8.0, 168.0, h - 16.0, 9.5);
        let cur_kind = crate::map::cursor(v).map_or(0, |c| c.1);
        let (title, say, fid) = match cur_kind {
            MN_SHOP => ("Shop", "Spend coins on items and stocks.", S_FACE_LAGOON),
            MN_CAMPFIRE => ("Campfire", "Rest: stocks back.", S_FACE_CORAL),
            MN_ITEM => ("Item room", "A free item on a pedestal.", S_FACE_WHITE),
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
            e.text_ex(bx + 66.0, by + 14.0, d(22), if fid == S_FACE_WHITE { INK } else { WHITE }, ALIGN_LEFT, 0.0, title);
            e.text_ex(bx + 66.0, by + 50.0, bs(13), if fid == S_FACE_WHITE { INK } else { WHITE }, ALIGN_LEFT, bw - 74.0, say);
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
            // the feather's own model, quill down, rocking as it falls (envoy-models1.md); the old strokes without it
            if !crate::fx::mesh(crate::fx::ME_FEATHER, x + sway, y, 2.2 + 0.1 * sway, 0.0, 10.0, fade) {
                fx(FX_LINE, x + sway - 2.5, y + 3.0, x + sway + 2.5, y - 3.0, 2.6, alpha(0x6A20_58FF, 0.9 * fade));
                fx(FX_LINE, x + sway - 2.5, y + 3.0, x + sway + 2.5, y - 3.0, 0.6, alpha(0xE060_80FF, fade));
            }
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
            // the splinter's own model, point along its flight
            if !crate::fx::mesh(crate::fx::ME_SHARD, x, y, crate::fx::atan2(vy, vx), 0.0, 9.0, fade) {
                fx(FX_SPIKES, x, y, 0.8, 4.0, t * 0.4 + i as f32, alpha(ICE, fade));
                fx(FX_DISC, x, y, 2.4, 0.0, 0.0, alpha(WHITE, 0.8 * fade));
            }
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
            // the sigil's own model, turning
            if !crate::fx::mesh(crate::fx::ME_SIGIL, sx, sy, 0.0, t * 0.06 + a as f32, 8.5, 1.0) {
                fx(FX_RING, sx, sy, 3.2, 1.0, 0.0, 0xFF30_40FF);
                fx(FX_SPIKES, sx, sy, 1.0, 3.2, -t * 0.08, alpha(0xFF60_70FF, 0.9));
            }
            fx(FX_DISC, sx, sy, 4.0, 0.0, 0.0, alpha(BLOOD, 0.5));
        }
        PJ_GUARD => {
            let (cx, cy) = player_pos(v, owner).unwrap_or((x, y));
            let spent = bb == 1;
            let ang = -t * 0.04 + a as f32 * 2.1;
            let (sx, sy) = (cx + cos(ang) * 12.0, cy + 12.0 + sin(ang) * 6.0);
            let c = if spent { alpha(PALE, 0.3) } else { HOLY };
            // the Guardian Feather's own model (faint once spent)
            if !crate::fx::mesh(crate::fx::ME_GUARD, sx, sy, 0.0, 0.4 * sin(t * 0.05), 10.0, if spent { 0.35 } else { 1.0 }) {
                fx(FX_LINE, sx - 2.5, sy - 2.0, sx + 2.5, sy + 2.0, 2.4, c);
            }
            fx(FX_DISC, sx, sy, 4.0, 0.0, 0.0, alpha(c, 0.5));
        }
        PJ_HALO => {
            let (cx, cy) = player_pos(v, owner).unwrap_or((x, y));
            let sz = (a as f32 / 100.0).max(4.0);
            let hy = cy + 22.0 + 0.8 * sin(t * 0.08);
            // the halo's own model (a ring lying flat)
            if !crate::fx::mesh(crate::fx::ME_HALO, cx, hy, 0.0, 0.0, sz * 2.3, 1.0) {
                for j in 0..14 {
                    let ang = j as f32 / 14.0 * TAU;
                    fx(FX_DISC, cx + cos(ang) * sz, hy + sin(ang) * sz * 0.3, 1.8, 0.0, 0.0, alpha(0xFFF0_A0FF, 0.95));
                }
            }
            fx(FX_DISC, cx, hy, sz * 1.4, 0.0, 0.0, alpha(HOLY, 0.25));
        }
        PJ_WINGS => {
            let (cx, cy) = player_pos(v, owner).unwrap_or((x, y));
            let face = if a < 0 { -1.0 } else { 1.0 };
            let flap = 2.0 * sin(t * 0.15);
            let c = if bb == 1 { alpha(0x6A30_70FF, 0.5) } else { 0x9A20_78FF };
            // the wings' own model, beating (a small turn about the vertical); the old strokes without it
            let winged = crate::fx::mesh(crate::fx::ME_WINGS, cx - face * 2.0, cy + 14.0, 0.0, 0.45 * sin(t * 0.15), 24.0, if bb == 1 { 0.5 } else { 1.0 });
            for side in [-1.0f32, 1.0] {
                if winged {
                    break;
                }
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
            // an orb (vfx lane shader quads): each family its own look; shattered (life word 1): faint ice shards
            let fam = bb;
            let c = col(fam);
            let sd = (i as f32) * 1.7 + a as f32;
            let broken = v[V_PROJ + i * PJ_W + 5] == 1;
            if broken {
                vfx::fx(vfx::ICE, x, y, 7.0, 7.0).cols(vfx::pale(c, 0.4), c).age(t).seed(sd).a(0.3).emit();
            } else {
                match fam {
                    IT_DODGE_ORB => {
                        if !crate::fx::mesh(crate::fx::ME_ORB_DODGE, x, y, 0.0, 0.5 * sin(t * 0.07 + sd), 10.0, 1.0) {
                            vfx::fx(vfx::ORB, x, y, 10.0, 10.0).cols(vfx::pale(c, 0.2), c).p(1.0, 0.0, 3.0, 0.0).age(t).seed(sd).emit();
                        }
                        vfx::fx(vfx::TRAIL, x - 8.0, y, 0.0, 0.0).line(x, y, 5.0).cols(vfx::pale(c, 0.3), c).age(t).seed(sd).a(0.6).emit();
                    }
                    IT_NAIR_RING => {
                        if !crate::fx::mesh(crate::fx::ME_ORB_NAIR, x, y, t * 0.09 + sd, 0.0, 9.0, 1.0) {
                            vfx::fx(vfx::ORB, x, y, 8.0, 8.0).cols(0xFFF0_C0FF, c).p(1.0, 0.0, 3.0, 0.0).age(t).seed(sd).emit();
                        }
                        vfx::fx(vfx::SPARKS, x, y, 10.0, 10.0).cols(WHITE, c).p(0.6, 0.0, 0.0, 0.0).age(t).seed(sd).a(0.8).emit();
                    }
                    IT_DASH_ORB => {
                        if !crate::fx::mesh(crate::fx::ME_ORB_DASH, x, y, 0.0, 0.0, 10.0, 1.0) {
                            vfx::fx(vfx::ORB, x, y, 9.0, 9.0).cols(vfx::pale(c, 0.2), c).p(1.0, 0.0, 3.0, 0.0).age(t).seed(sd).emit();
                        }
                        vfx::fx(vfx::FLAME, x - 5.0, y, 10.0, 6.0).cols(c, vfx::deep(c, 0.4)).age(t).seed(sd).a(0.6).rot(1.571).emit();
                    }
                    IT_SHIELD_ORBS => {
                        if !crate::fx::mesh(crate::fx::ME_ORB_SHIELD, x, y, 0.0, t * 0.05 + sd, 8.5, 1.0) {
                            vfx::fx(vfx::DOME, x, y, 8.0, 8.0).cols(0xE8D8_FFFF, c).age(t).seed(sd).a(0.85).emit();
                        }
                    }
                    _ => {
                        let m = col(IT_MOON_DROPS);
                        if !crate::fx::mesh(crate::fx::ME_MOON, x, y, 0.0, t * 0.04 + sd, 8.0, 1.0) {
                            vfx::fx(vfx::ORB, x, y, 8.0, 8.0).cols(vfx::pale(m, 0.3), m).p(1.0, 0.0, 3.0, 0.0).age(t).seed(sd).emit();
                        }
                        vfx::fx(vfx::GLOW, x, y, 14.0, 14.0).cols(0xA0B8_FFFF, m).age(t).a(0.5).emit();
                    }
                }
            }
        }
        PJ_THORNS => {
            let face = if a < 0 { -1.0 } else { 1.0 };
            let sz = (bb as f32 / 100.0).max(3.0) * ease(k * 4.0);
            // the thorns' own model, fanning up and back from the fighter's back
            if !crate::fx::mesh(crate::fx::ME_THORNS, x - face * 3.0, y + 7.0, face * 0.85, 0.0, sz * 1.7, fade) {
                for j in 0..4 {
                    let ang = 1.2 + j as f32 * 0.35;
                    let (tx, ty) = (x - face * cos(ang) * sz, y + 6.0 + sin(ang) * sz);
                    fx(FX_LINE, x - face * 2.0, y + 6.0 + j as f32 * 1.5, tx, ty, 1.6, alpha(0x40A0_40FF, fade));
                    fx(FX_SPIKES, tx, ty, 0.3, 1.6, 0.0, alpha(0xC0FF_A0FF, fade));
                }
            }
        }
        PJ_PRISM => {
            // Laser Prism's bolts: the bolt's own model along its flight, a thin pink streak behind it
            fx(FX_LINE, x, y, x - vx * 3.0, y - vy * 3.0, 0.9, alpha(0xFF5A_8AFF, 0.55 * fade));
            if !crate::fx::mesh(crate::fx::ME_BOLT, x, y, crate::fx::atan2(vy, vx), 0.0, 9.0, fade) {
                fx(FX_DISC, x, y, 2.0, 0.0, 0.0, alpha(0xFFB0_C8FF, fade));
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
            // the shatter: an impact with ice shards in the family's colour
            vfx::impact(x, y, 12.0, age as f32, c, ev[2] as f32 + 3.0);
            vfx::fx(vfx::ICE, x, y, 16.0 * (0.6 + k), 16.0 * (0.6 + k)).cols(WHITE, c).age(age as f32).a(1.0 - k).emit();
        } else {
            // the block: a hex shield flash and a pop
            vfx::fx(vfx::DOME, x, y, 18.0, 18.0).cols(WHITE, c).age(age as f32).a(0.8 * (1.0 - k)).emit();
            vfx::pop(x, y, 10.0, age as f32, 18.0, c);
        }
    }
}
