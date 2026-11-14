//! The host's built-in effects (docs/rust-port/notes/envoy-fx.md), behind small helpers: the Heat screen treatment,
//! the cash-out and boss-break bursts, and drive materials. Presentation only: the host clamps everything, never
//! draws over the centre of the picture, and turns it all down or off with the player's intensity setting.

use crate::hud;
use crate::VW;
use envoy_data::view::*;

#[link(wasm_import_module = "gd_ui")]
extern "C" {
    fn gd_fx_post(effect: u32, strength: f32, x: f32, y: f32, rgba: u32) -> i32;
    fn gd_ui_world_model_ex(model: i32, x: f32, y: f32, z: f32, yaw: f32, scale: f32, alpha: f32, material: i32, p0: f32, p1: f32, p2: f32, p3: f32, rgba: u32) -> i32;
}

pub const POST_VIGNETTE: u32 = 1;

pub const MAT_BODY: i32 = 0;
pub const MAT_GLASS: i32 = 1;
pub const MAT_GLOW: i32 = 2;
pub const MAT_FOIL: i32 = 3;
pub const MAT_DISSOLVE: i32 = 4;
pub const MAT_TRAIL: i32 = 0x80;
/// The ink edge (`melee_gx::fx::MAT_OUTLINE`): drive bodies only, never tier rings, the ascended overlay or glass.
pub const MAT_OUTLINE: i32 = 0x40;

/// The one screen post left (the edge vignette), through the Envoy settings (pause overlay, Options): the effect
/// intensity scales its strength.
unsafe fn post(effect: u32, strength: f32, x: f32, y: f32, rgba: u32) -> i32 {
    let k = crate::pause::fx_scale();
    if k <= 0.0 {
        return 0;
    }
    gd_fx_post(effect, strength * k, x, y, rgba)
}

/// A drive model with a material (see `gd_ui_world_model_ex`).
pub fn model(m: i32, x: f32, y: f32, z: f32, yaw: f32, scale: f32, alpha: f32, mat: i32, p: [f32; 4], rgba: u32) {
    unsafe { gd_ui_world_model_ex(m, x, y, z, yaw, scale, alpha, mat, p[0], p[1], p[2], p[3], rgba) };
}

/// The wearer's Heat state (0..3) of player `p`, 0 without Heat.
pub fn heat_state(v: &[i32; VW], p: usize) -> usize {
    hud::heat_of(v, p).map_or(0, |h| h.state)
}

/// Heat has no screen treatment any more (the owner, 2026-11-13: the full-screen heat effect goes with the flash).
/// The Heat meter left play with the items, so nothing stands in for it; a fighter-local look is `vfx::AURA`.
pub fn heat_screen(_v: &[i32; VW]) {}

/// The moments from the event ring, drawn for as long as each lasts straight from its age (no state: a rollback
/// that replays an event draws the same frames). All of it happens where the fighter is; nothing covers the
/// screen. A combo break: a cracking circle under the player, a white shockwave that bends the air, lightning
/// thrown outward, and one soft pulse of the player's colour at the screen's edge. A cash-out: a gold ring and a
/// shower of gold. A boss break: a coral shockwave and sparks from the boss.
pub fn moments(v: &[i32; VW]) {
    use crate::vfx;
    let n = v[V_EV_N].max(0) as usize;
    let now = v[V_PLAY];
    for i in 0..EV.min(n) {
        let ev = &v[V_EV + i * EV_W..V_EV + (i + 1) * EV_W];
        let age = now - ev[0];
        if !(0..48).contains(&age) || !(ev[1] == EV_CASHOUT || ev[1] == EV_BREAK || ev[1] == EV_COMBOBREAK) {
            continue;
        }
        let Some((x, y)) = (if ev[1] == EV_CASHOUT || ev[1] == EV_COMBOBREAK {
            let b = V_P0 + ev[2].clamp(0, 1) as usize * V_PW;
            if v[b + P_HAVE_POS] != 0 { Some((f32::from_bits(v[b + P_X] as u32), f32::from_bits(v[b + P_Y] as u32))) } else { None }
        } else {
            let b = V_S0 + ev[2].clamp(0, 3) as usize * V_SW;
            if v[b + S_ALIVE] != 0 { Some((f32::from_bits(v[b + S_X] as u32), f32::from_bits(v[b + S_Y] as u32))) } else { None }
        }) else {
            continue;
        };
        let af = age as f32;
        let k = af / 48.0;
        let sd = (ev[0] % 83) as f32;
        if ev[1] == EV_COMBOBREAK {
            let pc = crate::PLAYER_COL[ev[2].clamp(0, 1) as usize];
            if age == 0 {
                crate::sfx("combobreak");
            }
            // the circle draws itself in at once, then cracks open and dies
            let fade = 1.0 - ((k - 0.6) / 0.4).clamp(0.0, 1.0);
            vfx::ground_circle(x, y + 0.8, 30.0 + 6.0 * k, (k * 6.0).min(1.0), ((k - 0.12) * 1.6).clamp(0.0, 1.0), 1.0, af, crate::kit::alpha(pc, fade), sd);
            // two rings: the white one bends the air, the player's follows it
            vfx::fx(vfx::SHOCK, x, y + 10.0, 150.0, 150.0).cols(0xFFFF_FFB0, pc).p((k * 2.2).min(1.0), 0.03, 1.0, 2.0).age(af).seed(sd).bend().emit();
            vfx::fx(vfx::SHOCK, x, y + 10.0, 110.0, 110.0).cols(pc, pc).p(((k - 0.08) * 2.4).clamp(0.0, 1.0), 0.025, 0.0, 0.0).age(af).seed(sd + 1.0).emit();
            for j in 0..5 {
                let ang = j as f32 * 1.2566 + sd;
                let (dx, dy) = (crate::kit::cos(ang), crate::kit::sin(ang));
                vfx::bolt(x + dx * 5.0, y + 10.0 + dy * 5.0, x + dx * 44.0, y + 10.0 + dy * 34.0, 13.0, af, 14.0, 0xFFFF_FFFF, sd + j as f32 * 5.0);
            }
            vfx::fx(vfx::SPARKS, x, y + 10.0, 110.0, 110.0).cols(0xFFFF_FFFF, pc).p((k * 1.6).min(1.0), 34.0, 0.5, 0.0).seed(sd).emit();
            vfx::pop(x, y + 10.0, 11.0, af, 5.0, 0xFFFF_FFC0);
            if age < 14 {
                // the one thing at the screen's edge: a soft pulse in the player's colour, scaled by the setting
                unsafe { post(POST_VIGNETTE, 0.55 * (1.0 - af / 14.0), 0.0, 0.0, pc) };
            }
        } else if ev[1] == EV_CASHOUT {
            vfx::burst_ring(x, y + 10.0, 34.0, (k * 1.8).min(1.0), 0xFFD8_4AFF, 0.0, af);
            vfx::fx(vfx::SPARKS, x, y + 12.0, 70.0, 70.0).rot(1.5708).cols(0xFFF4_D0FF, 0xFFD8_4AFF).p((k * 1.4).min(1.0), 26.0, 1.0, 0.6).seed(sd).emit();
            vfx::fx(vfx::STAR, x, y + 12.0, 30.0 * (1.0 - k) + 4.0, 30.0 * (1.0 - k) + 4.0).cols(0xFFF4_D0FF, 0xFFD8_4AFF).p(1.0, 2.0, 0.7, 0.0).age(af).a(1.0 - k).emit();
        } else {
            vfx::fx(vfx::SHOCK, x, y + 10.0, 130.0, 130.0).cols(0xFF4D_6DE0, 0xC020_40FF).p((k * 1.8).min(1.0), 0.035, 0.5, 2.0).age(af).seed(sd).bend().emit();
            vfx::fx(vfx::SPARKS, x, y + 10.0, 100.0, 100.0).cols(0xFFFF_FFFF, 0xFF4D_6DFF).p((k * 1.5).min(1.0), 32.0, 0.8, 0.0).seed(sd).emit();
            vfx::pop(x, y + 10.0, 11.0, af, 5.0, 0xFFD0_D8C0);
        }
    }
}

/// Drops: a fly-in dissolve when one appears, a pickup dissolve (a ghost where it was) when it goes.
struct Drop {
    fam: i32,
    yaw: f32,
    x: f32,
    y: f32,
    born: u32,
    gone: u32,
}
static mut DROPS: [[Drop; 8]; 2] = unsafe { core::mem::zeroed() };

/// Draw drop `k` of player `p` this present: model `m` (a family body, or a Party drive's own model) at `yaw`, in its
/// glass shell when `glass`, outlined when `outline` (drive bodies); call `drop_ghosts` once after.
pub fn drop_model(t: u32, p: usize, k: usize, fam: i32, x: f32, y: f32, yaw: f32, glass: bool, outline: bool) {
    let d = unsafe { &mut (*core::ptr::addr_of_mut!(DROPS))[p.min(1)][k.min(7)] };
    if d.born == 0 || d.gone != 0 || d.fam != fam {
        d.born = t.max(1);
    }
    d.gone = 0;
    d.fam = fam;
    d.x = x;
    d.y = y;
    d.yaw = yaw;
    let pr = 1.0 - (t.wrapping_sub(d.born) as f32 / 24.0).min(1.0);
    model(fam, x, y, 0.0, yaw, 5.0, 1.0, MAT_DISSOLVE | if outline { MAT_OUTLINE } else { 0 }, [1.0, 1.0, 0.0, pr], 0xFFE0_A0FF);
    if glass {
        model(6, x, y, 0.0, yaw, 7.5, 0.3, MAT_GLASS, [0.5, 0.0, 0.0, 0.0], 0xFFFF_FFFF);
    }
}

/// Mark drops not drawn this present (`seen[p]` bit k) as gone and draw their pickup dissolve for 20 frames.
pub fn drop_ghosts(t: u32, seen: [u32; 2]) {
    for p in 0..2 {
        for k in 0..8 {
            let d = unsafe { &mut (*core::ptr::addr_of_mut!(DROPS))[p][k] };
            if d.born == 0 || seen[p] >> k & 1 != 0 {
                continue;
            }
            if d.gone == 0 {
                d.gone = t.max(1);
            }
            let a = t.wrapping_sub(d.gone) as f32 / 20.0;
            if a >= 1.0 {
                d.born = 0;
                d.gone = 0;
                continue;
            }
            model(d.fam, d.x, d.y + a * 6.0, 0.0, d.yaw, 5.0 * (1.0 + a * 0.3), 1.0, MAT_DISSOLVE, [1.6, 1.0, 0.0, a], 0xFFF4_D0FF);
        }
    }
}
