//! The presenter's half of the Envoy effects layer (melee_gx::fx, docs/rust-port/notes/envoy-fx.md): each present,
//! the drives the UI mod queued (melee_modes::tu::drive_draw), the fight camera the fighter pass recorded and the
//! queued post effects become one `FxFrame` for the renderer. Arms the layer when Envoy needs it and disarms it when
//! nothing does; with Envoy off nothing here creates or draws anything.

use melee_gx::fx::{FxFrame, FxMesh, WorldDraw, MAT_BODY, MAT_GLASS, MAT_TRAIL};
use melee_modes::tu::drive_draw;

/// Reused across presents (steady state allocates nothing).
#[derive(Default)]
pub struct EnvoyFx {
    frame: FxFrame,
    tris: Vec<drive_draw::Tri>,
    idle: u32,
    demo_n: u32,
    vfx_sig: u64,
    presents: u32,
    /// The effect pipelines were built (pre-warmed at start-up, or by the first run).
    warmed: bool,
}

impl EnvoyFx {
    /// `ui_mods`: a UI mod is loaded (Envoy can run): the effect pipelines are then built once, on the third present
    /// of the process (the swapchain format is known by then), so a fight never waits on a shader compile.
    pub fn present(&mut self, r: &mut melee_gx::render::Renderer, netplay: bool, ui_mods: bool) {
        self.presents += 1;
        let prewarm = ui_mods && !self.warmed && self.presents >= 3 && !r.fx_armed();
        let posts = melee_gx::fx::posts_pending() || std::env::var_os("MELEE_FX_DEMO").is_some();
        let want = melee_modes::tu::envoy::armed() != 0 || drive_draw::shown() > 0 || melee_gx::vfx::shown() > 0 || posts || prewarm;
        if !want {
            // a second of nothing: disarm (every pass off; objects kept for the next run)
            if r.fx_armed() {
                self.idle += 1;
                if self.idle > 60 {
                    r.fx_disarm();
                }
            }
            if !r.fx_armed() {
                return;
            }
        } else {
            self.idle = 0;
        }
        if !r.fx_armed() {
            let ms = drive_draw::models();
            let meshes: Vec<FxMesh<'_>> = ms.iter().map(|m| FxMesh { pos: &m.pos, nrm: &m.nrm, base: &m.base, glow: &m.glow }).collect();
            let t = std::time::Instant::now();
            r.fx_arm(&meshes);
            self.warmed = true;
            eprintln!("play: envoy fx: armed ({:.1} ms to build the effect pipelines{})", t.elapsed().as_secs_f64() * 1e3, if prewarm { ", pre-warmed at start-up" } else { "" });
        }
        let f = &mut self.frame;
        f.intensity = melee_gx::fx::intensity_from_env(netplay);
        r.fx_intensity = f.intensity;
        f.cam = drive_draw::camera().map(|c| melee_gx::fx::Camera { view: c.view, proj: c.proj, vp: c.vp });
        f.draws.clear();
        let glass_a = melee_gx::fx::glass_params().0;
        let draws = &mut f.draws;
        drive_draw::world_frame(
            |model, at, yaw, scale, alpha, mat, p, rgba| {
                // the old import's glass shell: the glass material at the material json's opacity
                let (mat, alpha) = if model as usize == drive_draw::M_GLASS && mat & melee_gx::fx::MAT_ID == MAT_BODY { (MAT_GLASS | (mat & MAT_TRAIL), alpha.min(glass_a)) } else { (mat, alpha) };
                // the ink edge only on drive bodies (never tier rings, the ascended overlay or glass)
                let mat = if mat & melee_gx::fx::MAT_OUTLINE != 0 && !drive_draw::is_drive_body(model as usize) { mat & !melee_gx::fx::MAT_OUTLINE } else { mat };
                draws.push(WorldDraw { model, mat, at, yaw, scale, alpha, p, rgba });
            },
            &mut self.tris,
        );
        f.tris.clear();
        f.tris.extend_from_slice(&self.tris);
        melee_gx::fx::take_posts(&mut f.posts);
        melee_gx::vfx::copy_shown(&mut f.vfx);
        vfx_log(&f.vfx, &mut self.vfx_sig, self.presents);
        demo(&mut f.posts, &mut self.demo_n);
        r.set_fx(f);
    }
}

/// `MELEE_VFX_LOG=1`: a line whenever the set of effect kinds on screen changes (present number, count per kind), so
/// a capture script can find the frames an effect is up. Diagnostics only.
fn vfx_log(list: &[melee_gx::vfx::VfxInst], sig: &mut u64, presents: u32) {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    if !*ON.get_or_init(|| std::env::var("MELEE_VFX_LOG").is_ok_and(|v| v == "1")) {
        return;
    }
    let mut n = [0u8; melee_gx::vfx::K_COUNT as usize];
    for i in list {
        let k = &mut n[(i.kind as usize).min(n.len() - 1)];
        *k = k.saturating_add(1);
    }
    let s = n.iter().fold(0xcbf2_9ce4_8422_2325u64, |h, &c| (h ^ c as u64).wrapping_mul(0x100_0000_01b3));
    if s != *sig {
        *sig = s;
        let kinds: Vec<String> = n.iter().enumerate().filter(|(_, &c)| c > 0).map(|(k, c)| format!("{k}x{c}")).collect();
        eprintln!("vfx: present {presents} quads {} [{}]", list.len(), kinds.join(" "));
    }
}

/// `MELEE_FX_DEMO` = comma list of warm | hot | blazing | cashout | break: the screenshot harness's stand-in posts
/// (each Heat level held; a cash-out / boss break every 90 presents, off the centre). Test aid only.
fn demo(posts: &mut Vec<melee_gx::fx::Post>, n: &mut u32) {
    use melee_gx::fx::*;
    let Ok(v) = std::env::var("MELEE_FX_DEMO") else { return };
    *n += 1;
    let mut add = |id, s, x, y, c| {
        if posts.len() < MAX_POST {
            posts.push(Post { id, strength: s, x, y, rgba: c });
        }
    };
    for w in v.split(',') {
        match w.trim() {
            "warm" => add(POST_VIGNETTE, 0.5, 0.0, 0.0, 0xFFBE_2EFF),
            "hot" => {
                add(POST_VIGNETTE, 0.7, 0.0, 0.0, 0xFF8A_1FFF);
                add(POST_EMBER, 0.6, 0.0, 0.0, 0xFF8A_1FFF);
            }
            "blazing" => {
                add(POST_EMBER, 0.8, 0.0, 0.0, 0xFF4A_1CFF);
                add(POST_HEAT_HAZE, 1.0, 0.0, 0.0, 0xFF4A_1CFF);
            }
            "cashout" if *n % 90 == 1 => {
                add(POST_RADIAL_WAVE, 1.0, 140.0, 330.0, 0xFFD8_4AFF);
                add(POST_FLASH, 0.8, 0.0, 0.0, 0xFFF4_D0FF);
            }
            "break" if *n % 90 == 45 => {
                add(POST_SHOCKWAVE, 1.0, 470.0, 120.0, 0xFF4D_6DFF);
                add(POST_FLASH, 1.0, 0.0, 0.0, 0xFFFF_FFFF);
            }
            _ => {}
        }
    }
}

/// The menu camera for the port's own menu screens (retail mn_8022BA1C: the first controller whose C-stick is out of
/// the 0.4 dead zone turns the menu camera up to 30 degrees per axis, linear past the dead zone). Ours adds a
/// spring-damper (it lags, overshoots a little and wobbles back to rest) and parallax in the UI shader. Display only:
/// it reads the pads the UI host already copied for this present; nothing reaches the sim.
#[derive(Default)]
pub struct MenuCam {
    ang: [f32; 2],
    vel: [f32; 2],
    last: Option<std::time::Instant>,
}

/// Whether one of our full-screen menus is up: a custom UI scene (the Envoy camp, the party picker), or an armed
/// Envoy run's reward / shop screen or its between-fights screens. Never the fight HUD.
pub fn our_menu_up(ui: &melee_modhost::ui::PresHost) -> bool {
    if !ui.shared.borrow().stack.is_empty() {
        return true;
    }
    let v = unsafe { &*core::ptr::addr_of!(melee_modes::tu::envoy::VIEW) };
    const VIEW_MAGIC: i32 = 0x454E_5633;
    melee_modes::tu::envoy::armed() != 0 && v[0] == VIEW_MAGIC && v[1] != 0 && (v[8] != 0 || v[5] == 0)
}

impl MenuCam {
    /// This present's tilt (yaw, pitch radians) for `Renderer::ui_tilt`: exactly [0, 0] at rest or with no menu up.
    pub fn update(&mut self, ui: Option<&melee_modhost::ui::PresHost>, intensity: f32) -> [f32; 2] {
        let now = std::time::Instant::now();
        let dt = self.last.replace(now).map_or(0.0, |l| (now - l).as_secs_f32()).min(0.05);
        let up = intensity > 0.0 && ui.is_some_and(our_menu_up);
        let mut target = [0.0f32; 2];
        if up {
            // the larger deflection of any controller, per axis (retail takes the first one out of the dead zone)
            let pads = ui.unwrap().shared.borrow().pads;
            let tilt = |raw: i8| {
                let v = (raw as f32 / 80.0).clamp(-1.0, 1.0);
                if v.abs() < 0.4 { 0.0 } else { 30f32.to_radians() * (v.abs() - 0.4) / 0.6 * v.signum() }
            };
            for p in pads.iter().filter(|p| p.err != -1) {
                let (x, y) = (tilt(p.substickX), tilt(p.substickY));
                if x.abs() > target[0].abs() {
                    target[0] = x;
                }
                if y.abs() > target[1].abs() {
                    target[1] = y;
                }
            }
            // Subtle: half the turn
            target = [target[0] * intensity, target[1] * intensity];
        }
        // spring-damper: stiff enough to follow in ~0.15 s, under-damped (zeta 0.42) for a small overshoot and wobble
        const K: f32 = 260.0;
        let c = 2.0 * K.sqrt() * 0.42;
        let steps = (dt / 0.004).ceil().max(1.0) as u32;
        let h = dt / steps as f32;
        for _ in 0..steps {
            for i in 0..2 {
                let a = K * (target[i] - self.ang[i]) - c * self.vel[i];
                self.vel[i] += a * h;
                self.ang[i] += self.vel[i] * h;
            }
        }
        // rest is exact: text pixel-crisp the moment it settles
        for i in 0..2 {
            if target[i] == 0.0 && self.ang[i].abs() < 2e-4 && self.vel[i].abs() < 2e-3 {
                self.ang[i] = 0.0;
                self.vel[i] = 0.0;
            }
        }
        if !up && intensity <= 0.0 {
            *self = MenuCam { last: Some(now), ..Default::default() };
        }
        self.ang
    }
}
