//! Swapchain + present (platform step 3, first part): melee_gx's renderer on a winit window.
//!
//! The GX draws into its 640x480 EFB (offscreen, as on the GameCube and in Aurora); a frame the
//! game finished with GXCopyDisp ([`melee_gx::shim::GxShim::take_frame`]) is rendered there and
//! then scaled onto the window's swapchain image ([`melee_gx::render::Renderer::blit_to`]).
//! A frame with no new copy re-presents the last EFB (the VI shows the last XFB).
use melee_gx::draw::DrawCmd;
use melee_gx::render::Renderer;
use melee_gx::state::GxState;
use std::sync::Arc;
use winit::window::Window;

/// EFB size (GXNtsc480*: fbWidth x efbHeight).
pub const EFB_WIDTH: u32 = 640;
pub const EFB_HEIGHT: u32 = 480;

pub struct Presenter {
    pub renderer: Renderer,
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
    /// Frames presented / swapchain images that could not be acquired (skipped).
    pub presented: u64,
    pub skipped: u64,
    /// The internal scale follows the window (no `MELEE_RENDER_SCALE`).
    auto_scale: bool,
    /// The content aspect of the frames being presented (4:3, or the window's with widescreen in a wide scene).
    aspect: f32,
    /// The last two sim frames (render-only interpolation: never fed back into the game).
    prev: Option<melee_gx::interp::Frame>,
    cur: Option<melee_gx::interp::Frame>,
    /// `cur` is the renderer's prepared frame (geometry uploaded, pipelines looked up).
    prepared: bool,
    /// The EFB holds a blend, not `cur` itself.
    blended: bool,
    /// A new frame waiting for the next present (see `present_frames`).
    pending: Option<melee_gx::interp::Frame>,
    solver: melee_gx::interp::Solver,
    true_wide: bool,
    /// Per-present µs of prepare / draw / swap (fresh frames only for prepare), for the MELEE_SHOW_FPS exit line.
    pub times: [Vec<u32>; 3],
    /// The HUD overlay's lines (None: off), drawn over every present (overlay.rs).
    pub hud: Option<Vec<crate::overlay::Line>>,
    /// Per re-posed present: us of the solver's evaluation and of the uniform patching (MELEE_SHOW_FPS exit line).
    pub interp_us: [Vec<u32>; 2],
    interp_total: melee_gx::interp::Stats,
    /// A screenshot of an in-between: adopt the new frame at once and draw it at this phase (MELEE_SHOT_PHASE).
    pub force_phase: Option<f64>,
    /// The EFB copies of `cur` have been made from its exact pose (see `present_posed`).
    copies_exact: bool,
}

impl Drop for Presenter {
    fn drop(&mut self) {
        if std::env::var_os("MELEE_SHOW_FPS").is_none() {
            return;
        }
        let q = |v: &mut Vec<u32>, f: f64| {
            if v.is_empty() {
                return 0.0;
            }
            v.sort_unstable();
            v[((v.len() - 1) as f64 * f) as usize] as f64 / 1e3
        };
        let [a, b, c] = &mut self.times;
        eprintln!(
            "gw: perf: present prepare p50 {:.2} p99 {:.2} ms ({}), draw p50 {:.2} p99 {:.2} ms, swap p50 {:.2} p99 {:.2} ms",
            q(a, 0.5), q(a, 0.99), a.len(), q(b, 0.5), q(b, 0.99), q(c, 0.5), q(c, 0.99)
        );
        let st = self.interp_total;
        let [e, f] = &mut self.interp_us;
        eprintln!(
            "gw: perf: interp evaluate p50 {:.3} p99 {:.3} ms, patch p50 {:.3} p99 {:.3} ms ({} re-posed presents); draws patched {} posed {} camera-carried {} held {} widened {}, cuts {}",
            q(e, 0.5), q(e, 0.99), q(f, 0.5), q(f, 0.99), e.len(), st.draws, st.posed, st.carried, st.held, st.widened, st.cuts
        );
    }
}

/// `MELEE_VSYNC=off|on` (default on).
pub fn vsync_from_env() -> bool {
    !matches!(std::env::var("MELEE_VSYNC").as_deref().map(str::to_ascii_lowercase).as_deref(), Ok("off" | "0" | "false" | "no"))
}

impl Presenter {
    /// A surface on `window` and a renderer on an adapter that can present to it.
    pub fn new(window: Arc<Window>) -> Result<Self, String> {
        let instance = Renderer::instance();
        let size = window.inner_size();
        let surface = instance.create_surface(window).map_err(|e| format!("create_surface: {e}"))?;
        let mut renderer = Renderer::with_instance(&instance, Some(&surface), EFB_WIDTH, EFB_HEIGHT)?;
        // Render the 3D scene at the window's resolution (the EFB's 4:3 coordinates scaled), not at 640x480 upscaled.
        let auto_scale = Renderer::env_scale().is_none();
        if auto_scale {
            renderer.set_scale(Renderer::scale_for_window(size.width, size.height));
        }
        crate::plog!("gw: video: render scale {} ({}), MSAA {}x", renderer.scale, if auto_scale { "auto" } else { "MELEE_RENDER_SCALE" }, renderer.samples);
        let caps = surface.get_capabilities(&renderer.adapter);
        // The EFB holds display-space values already (no sRGB decode on the GC), so prefer a
        // non-sRGB swapchain format; the blit copies the bytes through.
        let format = caps
            .formats
            .iter()
            .copied()
            .find(|f| !f.is_srgb())
            .or_else(|| caps.formats.first().copied())
            .ok_or("surface reports no formats")?;
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: if vsync_from_env() {
                wgpu::PresentMode::AutoVsync
            } else if caps.present_modes.contains(&wgpu::PresentMode::Immediate) {
                wgpu::PresentMode::Immediate
            } else if caps.present_modes.contains(&wgpu::PresentMode::Mailbox) {
                wgpu::PresentMode::Mailbox
            } else {
                wgpu::PresentMode::AutoNoVsync
            },
            // MELEE_FRAME_LATENCY=1|2|3 (frames queued ahead of the display; 1 = the waitable low-latency swapchain)
            desired_maximum_frame_latency: std::env::var("MELEE_FRAME_LATENCY").ok().and_then(|v| v.parse::<u32>().ok()).unwrap_or(2).clamp(1, 3),
            alpha_mode: caps.alpha_modes.first().copied().unwrap_or(wgpu::CompositeAlphaMode::Auto),
            view_formats: vec![],
        };
        surface.configure(&renderer.device, &config);
        let ai = renderer.adapter.get_info();
        crate::plog!("gw: video: present mode {:?}, frame latency {}, {:?} on {}", config.present_mode, config.desired_maximum_frame_latency, ai.backend, ai.name);
        Ok(Self { renderer, surface, config, auto_scale, aspect: 4.0 / 3.0, presented: 0, skipped: 0, prev: None, cur: None, prepared: false, blended: false, pending: None, solver: melee_gx::interp::Solver::with_mode(std::env::var("MELEE_INTERP_MODE").ok().and_then(|v| melee_gx::interp::Mode::parse(&v)).unwrap_or_default()), true_wide: std::env::var("MELEE_WIDESCREEN_MODE").map_or(true, |v| v != "camera") && std::env::var("MELEE_WIDESCREEN").is_ok_and(|v| v == "1"), times: Default::default(), hud: None, interp_us: Default::default(), interp_total: Default::default(), force_phase: None, copies_exact: false })
    }

    pub fn format(&self) -> wgpu::TextureFormat {
        self.config.format
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return; // minimised: keep the old configuration until it comes back
        }
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.renderer.device, &self.config);
        if self.auto_scale {
            self.renderer.set_scale(Renderer::scale_for_window_aspect(width, height, self.aspect));
            self.blended = true; // the targets were recreated: re-render the current frame
        }
    }

    /// The content aspect of the next frames (melee_engine `view::aspect`): reshapes the EFB target when it changed.
    /// The blit then letterboxes / pillarboxes that aspect inside the window.
    pub fn set_content_aspect(&mut self, aspect: f32) {
        if aspect.to_bits() == self.aspect.to_bits() {
            return;
        }
        self.aspect = aspect;
        let scale = if self.auto_scale { Renderer::scale_for_window_aspect(self.config.width, self.config.height, aspect) } else { self.renderer.scale };
        if self.renderer.set_target(scale, aspect) {
            self.blended = true;
            crate::plog!("gw: video: content aspect {aspect:.4}: EFB target {}x{}", self.renderer.width, self.renderer.height);
        }
    }

    /// Render `frame` (if the game finished one) into the EFB, then present the EFB.
    pub fn present(&mut self, state: &GxState, frame: Option<&[DrawCmd]>) {
        if let Some(draws) = frame {
            self.renderer.render_frame(state, draws);
            self.prepared = false;
        }
        self.swap_present();
    }

    /// Decoupled presentation: a new sim frame (if any) becomes current, the last becomes previous.
    /// `alpha = Some(t)` draws the frame re-posed at phase `t` between the two ticks (melee_gx `interp`: authored
    /// joints re-sampled, the rest interpolated, or extrapolated past the newest in the predictive mode); `None`
    /// draws the newest frame as is. With renderer widescreen on, perspective draws are widened either way.
    /// Everything here is render-only: the frames are copies the game no longer reads.
    pub fn present_frames(&mut self, state: &GxState, frame: Option<Vec<DrawCmd>>, alpha: Option<f64>) {
        self.present_posed(state, frame, None, alpha)
    }

    /// [`Presenter::present_frames`] with the frame's pose record.
    pub fn present_posed(&mut self, state: &GxState, frame: Option<Vec<DrawCmd>>, pose: Option<std::sync::Arc<melee_core::pose::PoseFrame>>, mut alpha: Option<f64>) {
        if self.renderer.gpu_fault().is_some() {
            return; // the caller stops on `gpu_fault`
        }
        let interpolate = self.solver.mode == melee_gx::interp::Mode::Interpolate;
        // Interpolating: a frame that arrives with a tick is adopted at the NEXT present, so the tick and the new
        // frame's prepare never land in one frame interval (the frame-time p99 was their sum). This present shows
        // the current frame itself (t = 1: never a step back toward the previous one).
        let mut frame = frame.map(|d| melee_gx::interp::Frame { draws: d, pose });
        if let Some(ph) = self.force_phase {
            alpha = Some(ph);
        }
        if interpolate && alpha.is_some() && self.cur.is_some() && self.force_phase.is_none() {
            if let Some(f) = frame.take() {
                let _late = self.pending.replace(f); // two ticks without a present: the older one is skipped
                alpha = Some(1.0);
            } else if let Some(f) = self.pending.take() {
                frame = Some(f);
            }
        } else if let Some(p) = self.pending.take() {
            if frame.is_none() {
                frame = Some(p);
            }
        }
        let fresh = frame.is_some();
        if let Some(f) = frame {
            self.prev = self.cur.replace(f);
            self.prepared = false;
            self.copies_exact = false;
            let widen = self.widen().is_some();
            let (solver, prev, cur) = (&mut self.solver, self.prev.as_ref(), self.cur.as_ref().unwrap());
            solver.plan(prev, cur, widen);
        }
        let Some(cur) = self.cur.as_ref() else {
            self.swap_present();
            return;
        };
        let widen = self.widen();
        // the phase on the solver's timeline; t = 1 (interpolate) / 0 (extrapolate) is the newest frame itself
        let t = alpha.map(|a| a.clamp(0.0, 1.0) as f32);
        let moving = match t {
            Some(t) if !self.solver.is_cut() && self.prev.is_some() => if interpolate { t < 0.9999 } else { t > 1e-4 },
            _ => false,
        };
        if moving || fresh || self.blended || (widen.is_some() && !self.prepared) {
            if !self.prepared {
                let t0 = std::time::Instant::now();
                self.renderer.prepare(state, &cur.draws);
                self.times[0].push(t0.elapsed().as_micros() as u32);
                self.prepared = true;
            }
            let t0 = std::time::Instant::now();
            // EFB copies (Pokemon Stadium's screen, shadows) hold a sim frame's exact pose, as on the console: a re-posed
            // in-between leaves the copy targets alone, else what a later frame samples depends on the wall-clock phase of
            // the last present before it (run-to-run render variance). A fresh frame first re-posed is drawn once exactly.
            let has_copies = cur.draws.iter().any(|d| d.copy.is_some());
            if moving && has_copies && !self.copies_exact {
                let solver = &mut self.solver;
                let mut f = |i: usize, out: &mut [u8]| solver.patch(cur, i, out, widen, false);
                self.renderer.draw_prepared(state, &cur.draws, if widen.is_some() { Some(&mut f) } else { None });
            }
            self.renderer.skip_copies = moving;
            if moving || widen.is_some() {
                let prev = self.prev.as_ref();
                if moving {
                    let te = std::time::Instant::now();
                    self.solver.evaluate(prev, cur, t.unwrap());
                    self.interp_us[0].push(te.elapsed().as_micros() as u32);
                }
                let solver = &mut self.solver;
                let mut f = |i: usize, out: &mut [u8]| solver.patch(cur, i, out, widen, moving);
                self.renderer.draw_prepared(state, &cur.draws, Some(&mut f));
                if moving {
                    self.interp_us[1].push(self.renderer.last_fill_us);
                }
            } else {
                self.renderer.draw_prepared(state, &cur.draws, None);
            }
            self.renderer.skip_copies = false;
            self.copies_exact = true;
            self.blended = moving;
            let s = std::mem::take(&mut self.solver.stats);
            self.interp_total.add(&s);
            self.times[1].push(t0.elapsed().as_micros() as u32);
        }
        let t0 = std::time::Instant::now();
        self.swap_present();
        self.times[2].push(t0.elapsed().as_micros() as u32);
    }

    /// The renderer-side widening for the current content aspect (None: off or a 4:3 frame).
    fn widen(&self) -> Option<melee_gx::interp::Widen> {
        if !self.true_wide {
            return None;
        }
        melee_gx::interp::Widen::for_aspect(self.aspect)
    }

    /// Interpolation timeline (interpolate / extrapolate) and whether a match is on screen (its HUD cameras held).
    pub fn set_interp_mode(&mut self, mode: melee_gx::interp::Mode) {
        self.solver.mode = mode;
    }
    pub fn set_match_scene(&mut self, on: bool) {
        self.solver.match_scene = on;
    }
    /// Renderer-side widescreen (the content aspect comes from `set_content_aspect`).
    pub fn set_true_widescreen(&mut self, on: bool) {
        self.true_wide = on;
    }
    /// The solver's counters since the last call.
    pub fn take_interp_stats(&mut self) -> melee_gx::interp::Stats {
        let s = std::mem::take(&mut self.solver.stats);
        self.interp_total.add(&s);
        s
    }

    fn swap_present(&mut self) {
        let tex = match self.surface.get_current_texture() {
            Ok(t) => t,
            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                self.surface.configure(&self.renderer.device, &self.config);
                match self.surface.get_current_texture() {
                    Ok(t) => t,
                    Err(_) => {
                        self.skipped += 1;
                        return;
                    }
                }
            }
            Err(_) => {
                self.skipped += 1;
                return;
            }
        };
        let view = tex.texture.create_view(&Default::default());
        self.renderer.blit_to(&view, self.config.format, self.config.width, self.config.height);
        let hud = self.hud.as_ref().map_or(Vec::new(), |l| crate::overlay::layout(l, self.config.width, self.config.height));
        self.renderer.overlay_to(&view, self.config.format, self.config.width, self.config.height, &hud);
        tex.present();
        self.presented += 1;
    }

    /// DXGI's frame statistics of the swapchain (D3D12 backend only): frames shown (PresentCount), refreshes
    /// (SyncRefreshCount) and the time of the last refresh in seconds (SyncQPCTime). None: another backend, or DXGI
    /// has no statistics for this swapchain yet.
    #[cfg(windows)]
    pub fn frame_stats(&self) -> Option<(u64, u64, f64)> {
        use windows::Win32::Graphics::Dxgi::DXGI_FRAME_STATISTICS;
        let hal = unsafe { self.surface.as_hal::<wgpu::hal::api::Dx12>() }?;
        let sc = hal.swap_chain()?;
        let mut st = DXGI_FRAME_STATISTICS::default();
        unsafe { sc.GetFrameStatistics(&mut st) }.ok()?;
        let mut f = 0i64;
        unsafe { windows::Win32::System::Performance::QueryPerformanceFrequency(&mut f) }.ok()?;
        Some((st.PresentCount as u64, st.SyncRefreshCount as u64, st.SyncQPCTime as f64 / f.max(1) as f64))
    }
    #[cfg(not(windows))]
    pub fn frame_stats(&self) -> Option<(u64, u64, f64)> {
        None
    }

    /// The swapchain size (window pixels).
    pub fn size(&self) -> (u32, u32) {
        (self.config.width, self.config.height)
    }

    /// What the window shows at the window's size (EFB blit, gd_ui batch, HUD overlay), alpha opaque: the shot for
    /// window-space UI (gd-ui-v1 GAP-0), which an EFB-sized shot would cut off.
    pub fn screenshot_window(&mut self) -> (u32, u32, Vec<u8>) {
        let (w, h) = (self.config.width, self.config.height);
        let hud = self.hud.as_ref().map_or(Vec::new(), |l| crate::overlay::layout(l, w, h));
        let mut px = self.renderer.read_composed_size(w, h, &hud);
        for p in px.chunks_mut(4) {
            p[3] = 255;
        }
        (w, h, px)
    }

    /// The EFB as RGBA8 with alpha forced opaque (what the window shows, before scaling).
    /// With the HUD overlay on, the shot shows it too (drawn over the EFB at its size; the EFB itself is untouched).
    pub fn screenshot(&mut self) -> Vec<u8> {
        let mut px = if let Some(l) = self.hud.as_ref() {
            let hud = crate::overlay::layout(l, self.renderer.width, self.renderer.height);
            self.renderer.read_composed(&hud)
        } else {
            self.renderer.read_color()
        };
        for p in px.chunks_mut(4) {
            p[3] = 255;
        }
        px
    }
}

/// Write RGBA8 pixels as a PNG.
pub fn write_png(path: &std::path::Path, width: u32, height: u32, rgba: &[u8]) -> Result<(), String> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    }
    let f = std::fs::File::create(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut e = png::Encoder::new(std::io::BufWriter::new(f), width, height);
    e.set_color(png::ColorType::Rgba);
    e.set_depth(png::BitDepth::Eight);
    e.write_header()
        .and_then(|mut w| w.write_image_data(rgba))
        .map_err(|e| format!("{}: {e}", path.display()))
}
