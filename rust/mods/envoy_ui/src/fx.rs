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
pub const POST_EMBER: u32 = 2;
pub const POST_HEAT_HAZE: u32 = 3;
pub const POST_FLASH: u32 = 4;
pub const POST_RADIAL_WAVE: u32 = 5;
pub const POST_SHOCKWAVE: u32 = 7;
pub const POST_WORLD: u32 = 0x100;

pub const MAT_BODY: i32 = 0;
pub const MAT_GLASS: i32 = 1;
pub const MAT_GLOW: i32 = 2;
pub const MAT_FOIL: i32 = 3;
pub const MAT_DISSOLVE: i32 = 4;
pub const MAT_TRAIL: i32 = 0x80;
/// The ink edge (`melee_gx::fx::MAT_OUTLINE`): drive bodies only, never tier rings, the ascended overlay or glass.
pub const MAT_OUTLINE: i32 = 0x40;

/// Every screen post through the Envoy settings (pause overlay, Options): the effect intensity scales the strength;
/// with screen shake off the distortions (shockwave, radial wave) are skipped.
unsafe fn post(effect: u32, strength: f32, x: f32, y: f32, rgba: u32) -> i32 {
    let k = crate::pause::fx_scale();
    let kind = effect & 0xFF;
    if k <= 0.0 || (!crate::pause::shake_on() && (kind == POST_SHOCKWAVE || kind == POST_RADIAL_WAVE)) {
        return 0;
    }
    gd_fx_post(effect, strength * k, x, y, rgba)
}

const HEAT_RGBA: [u32; 4] = [0, 0xFFBE_2EFF, 0xFF8A_1FFF, 0xFF4A_1CFF];

/// A drive model with a material (see `gd_ui_world_model_ex`).
pub fn model(m: i32, x: f32, y: f32, z: f32, yaw: f32, scale: f32, alpha: f32, mat: i32, p: [f32; 4], rgba: u32) {
    unsafe { gd_ui_world_model_ex(m, x, y, z, yaw, scale, alpha, mat, p[0], p[1], p[2], p[3], rgba) };
}

/// The wearer's Heat state (0..3) of player `p`, 0 without Heat.
pub fn heat_state(v: &[i32; VW], p: usize) -> usize {
    hud::heat_of(v, p).map_or(0, |h| h.state)
}

/// The Heat screen treatment for the hottest player: nothing at Cold, a warm vignette at Warm, an ember tint at Hot,
/// edge heat-haze with embers at Blazing. Held effects: posted every present.
pub fn heat_screen(v: &[i32; VW]) {
    let humans = v[V_HUMANS].clamp(1, 2) as usize;
    let st = (0..humans).map(|p| heat_state(v, p)).max().unwrap_or(0);
    let c = HEAT_RGBA[st];
    unsafe {
        match st {
            1 => { post(POST_VIGNETTE, 0.5, 0.0, 0.0, c); }
            2 => {
                post(POST_VIGNETTE, 0.7, 0.0, 0.0, c);
                post(POST_EMBER, 0.6, 0.0, 0.0, c);
            }
            3 => {
                post(POST_EMBER, 0.8, 0.0, 0.0, c);
                post(POST_HEAT_HAZE, 1.0, 0.0, 0.0, c);
            }
            _ => {}
        }
    }
}

static mut SEEN: [i32; 16] = [i32::MIN; 16];
static mut SEEN_N: usize = 0;

/// One-shots from the event ring, each once (a rollback that replays an event does not fire it twice): a cash-out
/// rings out from the player with a short flash and colour split; a boss break sends a shockwave from the boss.
pub fn moments(v: &[i32; VW]) {
    let n = v[V_EV_N].max(0) as usize;
    let now = v[V_PLAY];
    for i in 0..EV.min(n) {
        let ev = &v[V_EV + i * EV_W..V_EV + (i + 1) * EV_W];
        if !(0..6).contains(&(now - ev[0])) || !(ev[1] == EV_CASHOUT || ev[1] == EV_BREAK) {
            continue;
        }
        let key = ev[0].wrapping_mul(64) ^ (ev[1] << 3) ^ ev[2];
        unsafe {
            let seen = &mut *core::ptr::addr_of_mut!(SEEN);
            if seen.contains(&key) {
                continue;
            }
            seen[SEEN_N % 16] = key;
            SEEN_N += 1;
        }
        let (x, y, world) = if ev[1] == EV_CASHOUT {
            let b = V_P0 + ev[2].clamp(0, 1) as usize * V_PW;
            if v[b + P_HAVE_POS] != 0 { (f32::from_bits(v[b + P_X] as u32), f32::from_bits(v[b + P_Y] as u32) + 10.0, true) } else { (320.0, 240.0, false) }
        } else {
            let b = V_S0 + ev[2].clamp(0, 3) as usize * V_SW;
            if v[b + S_ALIVE] != 0 { (f32::from_bits(v[b + S_X] as u32), f32::from_bits(v[b + S_Y] as u32) + 10.0, true) } else { (320.0, 240.0, false) }
        };
        let w = if world { POST_WORLD } else { 0 };
        unsafe {
            if ev[1] == EV_CASHOUT {
                post(POST_RADIAL_WAVE | w, 1.0, x, y, 0xFFD8_4AFF);
                post(POST_FLASH, 0.8, 0.0, 0.0, 0xFFF4_D0FF);
            } else {
                post(POST_SHOCKWAVE | w, 1.0, x, y, 0xFF4D_6DFF);
                post(POST_FLASH, 1.0, 0.0, 0.0, 0xFFFF_FFFF);
            }
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
