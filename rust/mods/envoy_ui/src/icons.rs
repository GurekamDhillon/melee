//! Item icons (envoy slice 2): one distinct sticker icon per item, drawn from the kit's primitives (discs, rects,
//! sprites) so it reads at HUD size (16-24 px) and up. An ink rim, the item's colour as the face, a white glyph.
//! No rotation is used (a sticker's own tilt stays intact when an icon is drawn inside it).
use crate::kit::*;
use envoy_data::items::*;

/// A disc of diameter `dm` centred on (x, y).
fn dot(e: &mut E, x: f32, y: f32, dm: f32, c: u32) {
    spr_c(e, S_DISC, x, y, dm / 32.0, c);
}

/// A thick round-capped line from (x0, y0) to (x1, y1), `w` thick: one pill quad (kit::seg), not a chain of discs.
pub fn ln(e: &mut E, x0: f32, y0: f32, x1: f32, y1: f32, w: f32, c: u32) {
    seg(e, x0, y0, x1, y1, w, c);
}

/// A triangle pointing up (apex at (x, top), base `w` wide at `bot`) from stacked rects.
fn tri_up(e: &mut E, x: f32, top: f32, bot: f32, w: f32, c: u32) {
    let h = bot - top;
    let n = ceil(h / 1.2).clamp(2.0, 24.0) as i32;
    for i in 0..n {
        let t0 = i as f32 / n as f32;
        let ww = w * (t0 + 0.5 / n as f32);
        e.rect(x - ww * 0.5, top + h * t0, ww, h / n as f32 + 0.4, c);
    }
}

/// A triangle pointing down (apex at (x, bot)).
fn tri_down(e: &mut E, x: f32, top: f32, bot: f32, w: f32, c: u32) {
    let h = bot - top;
    let n = ceil(h / 1.2).clamp(2.0, 24.0) as i32;
    for i in 0..n {
        let t0 = i as f32 / n as f32;
        let ww = w * (1.0 - t0 - 0.5 / n as f32);
        e.rect(x - ww * 0.5, top + h * t0, ww, h / n as f32 + 0.4, c);
    }
}

fn ringd(e: &mut E, x: f32, y: f32, r: f32, w: f32, c: u32, n: i32) {
    for i in 0..n {
        let a = i as f32 / n as f32 * 6.283_185;
        dot(e, x + r * cos(a), y + r * sin(a), w, c);
    }
}

/// The item's icon centred on (x, y), `size` across (the ink rim included), faded by `a`.
pub fn icon(e: &mut E, id: i32, x: f32, y: f32, size: f32) {
    icon_a(e, id, x, y, size, 1.0);
}

pub fn icon_a(e: &mut E, id: i32, x: f32, y: f32, size: f32, a: f32) {
    let col = item(id).map_or(PALE, |it| it.rgba);
    let s = size / 24.0; // the glyphs are laid out on a 24 px icon
    let w = alpha(WHITE, a);
    let k = alpha(INK, a);
    let c = alpha(col, a);
    dot(e, x, y, size, k);
    dot(e, x, y, size - 4.0 * s.max(0.6), c);
    let p = |dx: f32, dy: f32| (x + dx * s, y + dy * s);
    let l = |e: &mut E, x0: f32, y0: f32, x1: f32, y1: f32, wd: f32, cc: u32| {
        let (a0, b0) = p(x0, y0);
        let (a1, b1) = p(x1, y1);
        ln(e, a0, b0, a1, b1, wd * s, cc);
    };
    let dd = |e: &mut E, dx: f32, dy: f32, dm: f32, cc: u32| {
        let (a0, b0) = p(dx, dy);
        dot(e, a0, b0, dm * s, cc);
    };
    let r = |e: &mut E, dx: f32, dy: f32, ww: f32, hh: f32, cc: u32| {
        let (a0, b0) = p(dx, dy);
        e.rect(a0, b0, ww * s, hh * s, cc);
    };
    match id {
        IT_SPARK_RING => {
            for i in 0..6 {
                let an = i as f32 / 6.0 * 6.283_185;
                dd(e, 6.0 * cos(an), 6.0 * sin(an), 3.4, w);
            }
            dd(e, 0.0, 0.0, 3.0, w);
        }
        IT_GLIDE_WAKE => {
            l(e, -7.0, -3.0, 4.0, -3.0, 2.2, w);
            l(e, -8.0, 1.0, 6.0, 1.0, 2.6, w);
            l(e, -6.0, 5.0, 3.0, 5.0, 2.0, w);
            dd(e, 6.0, 4.0, 4.5, alpha(0xFFE0_70FF, a));
        }
        IT_SHINE_BELL => {
            dd(e, 0.0, -2.0, 9.0, w);
            r(e, -4.5, -2.0, 9.0, 6.0, w);
            r(e, -6.5, 3.5, 13.0, 2.2, w);
            dd(e, 0.0, 6.5, 3.0, w);
            dd(e, 0.0, -7.5, 2.4, w);
        }
        IT_HOP_SHOT => {
            l(e, -7.0, 6.0, -4.0, 1.0, 1.6, w);
            l(e, -4.0, 1.0, -1.0, 5.0, 1.6, w);
            spr_c(e, S_STAR_ON, x + 3.0 * s, y - 2.0 * s, 1.05 * s, w);
        }
        IT_LEDGE_SPIKES => {
            tri_up(e, x - 5.0 * s, y - 4.0 * s, y + 6.0 * s, 5.0 * s, w);
            tri_up(e, x, y - 8.0 * s, y + 6.0 * s, 6.0 * s, w);
            tri_up(e, x + 5.0 * s, y - 3.0 * s, y + 6.0 * s, 5.0 * s, w);
            r(e, -8.0, 6.0, 16.0, 1.6, w);
        }
        IT_WHIFF_ECHO => {
            // a ghost with its echo behind
            dd(e, 2.5, -2.5, 8.0, alpha(WHITE, 0.45 * a));
            r(e, -1.5, -2.5, 8.0, 7.0, alpha(WHITE, 0.45 * a));
            dd(e, -2.0, -1.0, 9.0, w);
            r(e, -6.5, -1.0, 9.0, 7.0, w);
            for i in 0..3 {
                dd(e, -5.0 + i as f32 * 3.0, 6.0, 3.0, w);
            }
            dd(e, -3.5, -1.5, 1.8, k);
            dd(e, -0.5, -1.5, 1.8, k);
        }
        IT_ROLL_MINES => {
            for i in 0..8 {
                let an = i as f32 / 8.0 * 6.283_185;
                l(e, 0.0, 1.0, 7.0 * cos(an), 1.0 + 7.0 * sin(an), 1.4, k);
            }
            dd(e, 0.0, 1.0, 10.0, k);
            dd(e, 0.0, 1.0, 4.0, alpha(0xFF40_40FF, a));
            dd(e, -2.0, -1.0, 1.8, w);
        }
        IT_SMASH_QUAKE => {
            r(e, -8.0, 4.0, 16.0, 2.2, w);
            l(e, -6.0, -6.0, -2.0, -1.0, 1.7, w);
            l(e, -2.0, -1.0, 1.0, -5.0, 1.7, w);
            l(e, 1.0, -5.0, 5.0, 1.0, 1.7, w);
            l(e, 5.0, 1.0, 7.0, -3.0, 1.7, w);
        }
        IT_MOON_DROPS => {
            dd(e, -1.0, 0.0, 13.0, w);
            dd(e, 2.5, -2.0, 11.0, c);
            dd(e, 5.0, 5.0, 4.0, w);
        }
        IT_COIN_MAGNET => {
            // a horseshoe magnet with white tips over a coin
            dd(e, 0.0, 1.0, 14.0, w);
            dd(e, 0.0, 1.0, 6.5, c);
            r(e, -7.0, -6.0, 14.0, 7.0, c);
            r(e, -7.0, -5.0, 3.6, 6.0, w);
            r(e, 3.4, -5.0, 3.6, 6.0, w);
            r(e, -7.0, -7.0, 3.6, 2.5, k);
            r(e, 3.4, -7.0, 3.6, 2.5, k);
        }
        IT_HOMING_INK => {
            let pts = [(-7.0, 5.0), (-4.0, 0.0), (0.0, -2.0), (4.0, -1.0), (6.0, 3.0)];
            for i in 0..4 {
                l(e, pts[i].0, pts[i].1, pts[i + 1].0, pts[i + 1].1, 1.8, w);
            }
            dd(e, 6.0, 4.5, 5.0, alpha(0x5A1A_A0FF, a));
            dd(e, -7.0, 5.0, 3.0, w);
        }
        IT_FIRE_LACE => {
            spr_c(e, S_FLAME + 2, x, y, 0.62 * s, w);
        }
        IT_LASER_PRISM => {
            tri_up(e, x - 2.5 * s, y - 6.0 * s, y + 5.0 * s, 11.0 * s, w);
            l(e, -9.0, 1.0, -4.0, 1.0, 1.4, alpha(0xFF30_30FF, a));
            l(e, 2.0, 0.0, 8.0, -5.0, 1.4, alpha(0xFF50_50FF, a));
            l(e, 2.0, 1.0, 9.0, 1.0, 1.4, alpha(0x60FF_80FF, a));
            l(e, 2.0, 2.0, 8.0, 7.0, 1.4, alpha(0x60A0_FFFF, a));
        }
        IT_CROUCH_THORNS => {
            dd(e, 0.0, 4.0, 13.0, w);
            r(e, -7.0, 4.0, 14.0, 5.0, c);
            tri_up(e, x - 4.5 * s, y - 6.0 * s, y + 0.5 * s, 3.4 * s, w);
            tri_up(e, x, y - 9.0 * s, y - 1.0 * s, 3.6 * s, w);
            tri_up(e, x + 4.5 * s, y - 6.0 * s, y + 0.5 * s, 3.4 * s, w);
        }
        IT_PERFECT_GUARD => {
            ringd(e, x - 3.0 * s, y, 6.0 * s, 2.4 * s, w, 14);
            l(e, 0.0, 0.0, 9.0, 0.0, 3.0, alpha(0xFFF0_A0FF, a));
            dd(e, -3.0, 0.0, 3.0, w);
        }
        IT_BRIMSTONE_JAB => {
            l(e, -2.0, 0.0, 9.0, 0.0, 5.0, alpha(0x8A00_10FF, a));
            l(e, -2.0, 0.0, 9.0, 0.0, 2.0, alpha(0xFF60_50FF, a));
            dd(e, -5.0, 0.0, 8.0, w);
            r(e, -8.0, -3.0, 3.0, 6.0, w);
        }
        IT_DEMON_WINGS => {
            for i in 0..4 {
                let t = i as f32;
                dd(e, -2.0 - t * 1.8, -4.0 + t * 2.4, 5.0 - t * 0.6, k);
                dd(e, 2.0 + t * 1.8, -4.0 + t * 2.4, 5.0 - t * 0.6, k);
            }
            l(e, -2.0, -5.0, -9.0, -6.0, 2.0, k);
            l(e, 2.0, -5.0, 9.0, -6.0, 2.0, k);
            dd(e, 0.0, -2.0, 2.4, w);
        }
        IT_CRIMSON_COUNT => {
            ringd(e, x, y, 7.0 * s, 1.8 * s, w, 18);
            l(e, 0.0, -5.0, 4.5, 3.0, 1.4, w);
            l(e, 4.5, 3.0, -4.5, 3.0, 1.4, w);
            l(e, -4.5, 3.0, 0.0, -5.0, 1.4, w);
            dd(e, 0.0, 0.5, 2.6, k);
        }
        IT_BLOODLUST => {
            dd(e, 0.0, 2.5, 9.0, w);
            tri_up(e, x, y - 8.0 * s, y + 1.5 * s, 7.6 * s, w);
            dd(e, 0.0, 3.0, 4.0, alpha(0x9A10_20FF, a));
            l(e, 4.0, 4.0, 8.0, 1.0, 1.2, w);
            l(e, -4.0, 4.0, -8.0, 1.0, 1.2, w);
        }
        IT_HALO => {
            for i in 0..18 {
                let an = i as f32 / 18.0 * 6.283_185;
                dd(e, 7.0 * cos(an), -1.0 + 3.0 * sin(an), 2.6, w);
            }
            dd(e, 0.0, 5.5, 5.0, alpha(WHITE, 0.6 * a));
        }
        IT_GUARDIAN_FEATHER => {
            l(e, -6.0, 7.0, 6.0, -7.0, 1.2, k);
            for i in 0..5 {
                let t = i as f32 / 4.0;
                let (cx, cy) = (-4.0 + 9.0 * t, 5.0 - 10.5 * t);
                dd(e, cx - 1.2, cy - 1.2, 5.0 - 2.0 * (t - 0.4).abs() * 2.0, w);
            }
        }
        IT_ICE_GLASS => {
            l(e, 0.0, -8.0, 0.0, 8.0, 1.8, w);
            l(e, -7.0, -4.0, 7.0, 4.0, 1.8, w);
            l(e, -7.0, 4.0, 7.0, -4.0, 1.8, w);
            dd(e, 0.0, 0.0, 4.0, w);
        }
        IT_CHAIN_SPARK => {
            l(e, 2.0, -8.0, -3.0, 0.0, 2.2, w);
            l(e, -3.0, 0.0, 3.0, 0.0, 2.2, w);
            l(e, 3.0, 0.0, -2.0, 8.0, 2.2, w);
            dd(e, 6.0, -5.0, 3.0, w);
            dd(e, -6.0, 5.0, 3.0, w);
        }
        IT_TWIN_SHOT => {
            l(e, -8.0, -3.5, 1.0, -3.5, 1.4, alpha(WHITE, 0.6 * a));
            l(e, -8.0, 3.5, 1.0, 3.5, 1.4, alpha(WHITE, 0.6 * a));
            dd(e, 3.0, -3.5, 6.0, w);
            dd(e, 3.0, 3.5, 6.0, w);
        }
        IT_GRAVITY_WELL => {
            for i in 0..14 {
                let t = i as f32 / 13.0;
                let an = t * 9.0;
                let rr = 1.0 + 7.0 * t;
                dd(e, rr * cos(an), rr * sin(an), 1.4 + 1.4 * t, w);
            }
            dd(e, 0.0, 0.0, 3.4, k);
        }
        IT_BIG_BANG => {
            spr_c(e, S_BURST16, x, y, 0.42 * s, w);
            dd(e, 0.0, 0.0, 6.0, c);
            dd(e, 0.0, 0.0, 3.0, w);
        }
        IT_BOUNCY => {
            r(e, -8.0, 6.0, 16.0, 1.6, w);
            for i in 0..5 {
                let t = i as f32 / 4.0;
                let xx = -7.0 + 9.0 * t;
                let yy = 5.0 - 10.0 * (1.0 - (2.0 * t - 1.0) * (2.0 * t - 1.0));
                dd(e, xx, yy, 1.5, w);
            }
            dd(e, 5.0, 1.0, 7.0, w);
            dd(e, 5.0, 1.0, 3.0, c);
        }
        IT_SPARE_HEART => {
            dd(e, -3.2, -2.0, 8.0, w);
            dd(e, 3.2, -2.0, 8.0, w);
            tri_down(e, x, y - 1.5 * s, y + 7.5 * s, 14.8 * s, w);
            r(e, -1.0, -4.0, 2.0, 6.0, alpha(CORAL, a));
            r(e, -3.0, -2.0, 6.0, 2.0, alpha(CORAL, a));
        }
        IT_DODGE_ORB => {
            // a dodging figure's after-images and the orb leading it
            dd(e, -6.0, 2.0, 5.0, alpha(WHITE, 0.35 * a));
            dd(e, -2.5, 1.0, 5.5, alpha(WHITE, 0.6 * a));
            dd(e, 4.0, 0.0, 8.0, w);
            dd(e, 4.0, 0.0, 3.6, c);
            l(e, -8.0, 6.0, 8.0, 6.0, 1.4, w);
        }
        IT_NAIR_RING => {
            ringd(e, x, y, 7.0 * s, 1.0 * s, alpha(WHITE, 0.5 * a), 16);
            for i in 0..4 {
                let an = i as f32 / 4.0 * 6.283_185 + 0.4;
                dd(e, 7.0 * cos(an), 7.0 * sin(an), 4.2, w);
            }
            dd(e, 0.0, 0.0, 4.0, k);
        }
        IT_DASH_ORB => {
            l(e, -8.0, -3.0, -1.0, -3.0, 1.4, w);
            l(e, -9.0, 1.0, 0.0, 1.0, 1.4, w);
            l(e, -8.0, 5.0, -1.0, 5.0, 1.4, w);
            dd(e, 4.0, 1.0, 9.0, w);
            dd(e, 4.0, 1.0, 4.0, c);
        }
        IT_SHIELD_ORBS => {
            dd(e, 0.0, 0.0, 11.0, alpha(WHITE, 0.45 * a));
            ringd(e, x, y, 5.5 * s, 1.4 * s, w, 16);
            for i in 0..3 {
                let an = i as f32 / 3.0 * 6.283_185 - 1.57;
                dd(e, 9.0 * cos(an), 9.0 * sin(an), 3.6, w);
            }
        }
        _ => dd(e, 0.0, 0.0, 6.0, w),
    }
}

/// A stock slot icon (the stock limit's heart container): full (`full`) or an empty outline, faded by `a`.
pub fn slot(e: &mut E, x: f32, y: f32, size: f32, full: bool, last: bool, a: f32) {
    if full {
        spr_c(e, if last { S_STOCK_LAST } else { S_STOCK }, x, y, size / 18.0, alpha(WHITE, a));
    } else {
        spr_c(e, S_STOCK_SPENT, x, y, size / 18.0, alpha(WHITE, a));
    }
}

/// A stock slot on a white sticker: a full slot an ink (coral when the last) disc, an empty one a pale ring.
pub fn slot_ink(e: &mut E, x: f32, y: f32, size: f32, full: bool, last: bool, a: f32) {
    if full {
        dot(e, x, y, size, alpha(if last { CORAL } else { INK }, a));
        dot(e, x - size * 0.18, y - size * 0.18, size * 0.3, alpha(WHITE, 0.6 * a));
    } else {
        dot(e, x, y, size, alpha(PALE, 0.9 * a));
        dot(e, x, y, size * 0.6, alpha(WHITE, a));
    }
}

/// Square root (no_std): a bit-trick guess and three Newton steps (presentation only).
pub fn sqrt(x: f32) -> f32 {
    if x <= 0.0 {
        return 0.0;
    }
    let mut g = f32::from_bits((x.to_bits() >> 1) + 0x1FBD_1DF5);
    for _ in 0..3 {
        g = 0.5 * (g + x / g);
    }
    g
}

pub fn ceil(x: f32) -> f32 {
    let t = x as i32 as f32;
    if t < x { t + 1.0 } else { t }
}
