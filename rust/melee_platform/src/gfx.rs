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
    /// The fx world list that arrived with `pending` (melee_gx::fx::FxSnap): adopted with it.
    pending_fx: Option<Box<melee_gx::fx::FxSnap>>,
    /// Whether `pending` is more than one tick after the frame before it (see `arrive_gap`).
    pending_cut: bool,
    /// Set by the caller before a present that hands over a frame: ticks since the frame it handed over before (1
    /// normally; more after a stall, when ticks ran in a burst and only the newest frame was kept). A pair more than
    /// one tick apart is not blended.
    pub arrive_gap: u32,
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
    /// The next present is a screenshot (see `present_posed`: its EFB copies are restored to exact poses first).
    pub shot: bool,
    /// The last present's µs of prepare / draw / swap (0: not run), and whether it adopted a new frame (spike logs).
    pub last: [u32; 3],
    pub last_fresh: bool,
    pub adopt_us: u32,
    pub drop_us: u32,
    /// Drop retired frames on a helper thread when this says their draw list is not game-arena memory (the GX decode
    /// thread's frames live on the system heap): freeing ~2500 draws took 1.0-1.5 ms of every fresh present (the
    /// frame-time p99). None: drop in place. The pose record stays on this thread (it may be arena memory).
    pub offthread_drop: Option<fn(*const u8) -> bool>,
    dropper: Option<std::sync::mpsc::Sender<Vec<DrawCmd>>>,
    /// The surface's present modes (live vsync switch).
    modes: Vec<wgpu::PresentMode>,
    /// Present pacing seen here: the last present's time and a running mean of the interval (ms), for `split`.
    last_present: Option<std::time::Instant>,
    interval_ms: f32,
    /// MELEE_PRESENT_SPLIT (default on): when presents come far faster than ticks, a new frame is prepared at the
    /// present that adopts it and drawn at the next, so one present interval never holds prepare + draw + swap
    /// of a fresh frame (that sum was the frame-time p99 at uncapped rates). Off: prepare and draw in one present.
    split: bool,
    /// Presents that only prepared (see `split`), and fresh frames in all.
    pub deferred: u64,
    pub fresh: u64,
    /// The current frame was prepared by a deferring present and is not on the EFB yet.
    undrawn: bool,
}

/// Mean present interval (ms) under which a fresh frame's prepare and draw go to separate presents: the one present
/// of extra latency is then under 2.5 ms, and the interval saved is the fresh present's ~3 ms. At vsync rates (16.7 /
/// 6.9 ms) a fresh present fits its interval anyway and the frame is drawn at once, as before.
const SPLIT_BELOW_MS: f32 = 2.5;

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
            "gw: perf: present prepare p50 {:.2} p99 {:.2} ms ({}), draw p50 {:.2} p99 {:.2} ms, swap p50 {:.2} p99 {:.2} ms; {} fresh frames, {} prepared at one present and drawn at the next, {} packed on this thread, {} drawn unprepared (prepared on the spot)",
            q(a, 0.5), q(a, 0.99), a.len(), q(b, 0.5), q(b, 0.99), q(c, 0.5), q(c, 0.99), self.fresh, self.deferred, melee_gx::render::main_packs(0), melee_gx::render::repairs(0)
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

/// The present mode for `vsync` among `modes` (off: immediate, else mailbox, else whatever does not wait).
fn present_mode(vsync: bool, modes: &[wgpu::PresentMode]) -> wgpu::PresentMode {
    if vsync {
        wgpu::PresentMode::AutoVsync
    } else if modes.contains(&wgpu::PresentMode::Immediate) {
        wgpu::PresentMode::Immediate
    } else if modes.contains(&wgpu::PresentMode::Mailbox) {
        wgpu::PresentMode::Mailbox
    } else {
        wgpu::PresentMode::AutoNoVsync
    }
}

impl Presenter {
    /// Live vsync switch (the settings overlay): reconfigures the swapchain. Presentation only.
    pub fn set_vsync(&mut self, on: bool) {
        let m = present_mode(on, &self.modes);
        if m != self.config.present_mode {
            self.config.present_mode = m;
            self.surface.configure(&self.renderer.device, &self.config);
            crate::plog!("gw: video: present mode {m:?} (live)");
        }
    }

    /// Live internal resolution (None: follow the window). Recreates the EFB targets; the frame is re-rendered.
    pub fn set_render_scale(&mut self, scale: Option<u32>) {
        self.auto_scale = scale.is_none();
        let s = scale.unwrap_or_else(|| Renderer::scale_for_window_aspect(self.config.width, self.config.height, self.aspect));
        if self.renderer.set_target(s, self.aspect) {
            self.blended = true;
        }
        crate::plog!("gw: video: render scale {} ({}, live)", self.renderer.scale, if self.auto_scale { "auto" } else { "set" });
    }

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
            present_mode: present_mode(vsync_from_env(), &caps.present_modes),
            // MELEE_FRAME_LATENCY=1|2|3 (frames queued ahead of the display; 1 = the waitable low-latency swapchain)
            desired_maximum_frame_latency: std::env::var("MELEE_FRAME_LATENCY").ok().and_then(|v| v.parse::<u32>().ok()).unwrap_or(2).clamp(1, 3),
            alpha_mode: caps.alpha_modes.first().copied().unwrap_or(wgpu::CompositeAlphaMode::Auto),
            view_formats: vec![],
        };
        surface.configure(&renderer.device, &config);
        let ai = renderer.adapter.get_info();
        crate::plog!("gw: video: present mode {:?}, frame latency {}, {:?} on {}", config.present_mode, config.desired_maximum_frame_latency, ai.backend, ai.name);
        Ok(Self { renderer, surface, config, auto_scale, aspect: 4.0 / 3.0, presented: 0, skipped: 0, prev: None, cur: None, prepared: false, blended: false, pending: None, pending_fx: None, pending_cut: false, arrive_gap: 1, solver: melee_gx::interp::Solver::with_mode(std::env::var("MELEE_INTERP_MODE").ok().and_then(|v| melee_gx::interp::Mode::parse(&v)).unwrap_or_default()), true_wide: std::env::var("MELEE_WIDESCREEN_MODE").map_or(true, |v| v != "camera") && std::env::var("MELEE_WIDESCREEN").is_ok_and(|v| v == "1"), times: Default::default(), hud: None, interp_us: Default::default(), interp_total: Default::default(), force_phase: None, shot: false, last: [0; 3], last_fresh: false, adopt_us: 0, drop_us: 0, offthread_drop: None, dropper: None, modes: caps.present_modes.clone(), last_present: None, interval_ms: 16.7, split: std::env::var("MELEE_PRESENT_SPLIT").map_or(true, |v| !matches!(v.trim(), "0" | "off" | "false" | "no")), deferred: 0, fresh: 0, undrawn: false })
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
        self.present_posed(state, frame, None, None, alpha)
    }

    /// Prepare the current frame (its own decode-thread packing when it still has one), timed into `times[0]`.
    fn prepare_cur(&mut self, state: &GxState) {
        let Some(cur) = self.cur.as_mut() else { return };
        let t0 = std::time::Instant::now();
        if let Some(pk) = cur.packed.take() {
            self.renderer.set_packed(pk);
        }
        self.renderer.prepare(state, &cur.draws);
        self.renderer.pin(&cur.draws);
        self.last[0] = t0.elapsed().as_micros() as u32;
        self.times[0].push(self.last[0]);
        self.prepared = true;
    }

    /// [`Presenter::present_frames`] with the frame's pose record and the decode thread's packing of its draws.
    pub fn present_posed(&mut self, state: &GxState, frame: Option<Vec<DrawCmd>>, pose: Option<std::sync::Arc<melee_core::pose::PoseFrame>>, packed: Option<melee_gx::render::Packed>, mut alpha: Option<f64>) {
        if self.renderer.gpu_fault().is_some() {
            return; // the caller stops on `gpu_fault`
        }
        self.last = [0; 3];
        self.adopt_us = 0;
        let now = std::time::Instant::now();
        if let Some(l) = self.last_present.replace(now) {
            let dt = (now - l).as_secs_f32() * 1e3;
            self.interval_ms = if dt > 50.0 { dt.min(100.0) } else { self.interval_ms * 0.9 + dt * 0.1 };
        }
        let interpolate = self.solver.mode == melee_gx::interp::Mode::Interpolate;
        // the fx layer's world camera follows the true-widescreen widening (melee_gx::fx)
        self.renderer.fx_widen = self.widen().map_or(1.0, |w| w.factor);
        // Interpolating: a frame that arrives with a tick is adopted at the NEXT present, so the tick and the new
        // frame's prepare never land in one frame interval (the frame-time p99 was their sum). This present shows
        // the current frame itself (t = 1: never a step back toward the previous one).
        let mut frame = frame.map(|d| melee_gx::interp::Frame { draws: d, pose, packed });
        // the fx world list handed over this present is of the tick that arrived with `frame`: it travels with it
        let mut frame_fx = if frame.is_some() { self.renderer.fx_snapshot() } else { None };
        let mut frame_cut = frame.is_some() && self.arrive_gap > 1;
        let mut skipped = None;
        if let Some(ph) = self.force_phase {
            alpha = Some(ph);
        }
        let shot = std::mem::take(&mut self.shot);
        if interpolate && alpha.is_some() && self.cur.is_some() && self.force_phase.is_none() {
            if let Some(f) = frame.take() {
                // two ticks without a present: the older one is skipped, but its EFB copies (GXCopyTex, e.g. the results
                // cards' portraits, copied once) still run, as every frame's do in the C build
                let mut late_skipped = false;
                if let Some(mut late) = self.pending.replace(f) {
                    let pk = late.packed.take();
                    self.render_copies(state, &late.draws, pk);
                    self.retire(Some(late));
                    late_skipped = true;
                }
                self.pending_fx = frame_fx.take();
                // a skipped (late) frame in between makes the pair non-consecutive too
                self.pending_cut = std::mem::take(&mut frame_cut) || late_skipped;
                alpha = Some(1.0);
            } else if let Some(f) = self.pending.take() {
                frame = Some(f);
                frame_fx = self.pending_fx.take();
                frame_cut = std::mem::take(&mut self.pending_cut);
            }
        } else if let Some(p) = self.pending.take() {
            if frame.is_none() {
                frame = Some(p);
                frame_fx = self.pending_fx.take();
                frame_cut = std::mem::take(&mut self.pending_cut);
            } else {
                skipped = Some(p);
                self.pending_fx = None;
                self.pending_cut = false;
                frame_cut = true;
            }
        }
        let fresh = frame.is_some();
        // A screenshot is the newest sim frame drawn exactly, but the EFB copies it samples from earlier presents
        // (Pokemon Stadium's screen shows the previous frame's copy) were made by re-posed in-between presents at a
        // wall-clock phase: the shot differed run to run. Redraw the previous sim frame exactly first, so its copies
        // hold that frame's exact pose as on the console (shots only: in-between presents are phase-dependent anyway).
        if shot && fresh && frame.as_ref().is_some_and(|f| f.draws.iter().any(|d| d.copy.is_some())) {
            if let Some(p) = skipped.as_mut().or(self.cur.as_mut()) {
                if let Some(pk) = p.packed.take() {
                    self.renderer.set_packed(pk);
                }
                self.renderer.prepare(state, &p.draws);
                self.renderer.draw_prepared(state, &p.draws, None);
            }
        }
        // Adopting a fresh frame at a present that has in-between presents before the next tick (uncapped rates):
        // prepare it now, draw it at the next present (see `split`). The EFB keeps showing the previous frame at its
        // newest pose, which the adopting present drew anyway (alpha = 1 at the deferral above).
        let defer = fresh && self.split && interpolate && alpha.is_some() && self.force_phase.is_none() && !shot && self.cur.is_some() && self.interval_ms < SPLIT_BELOW_MS;
        if let Some(f) = frame {
            let ta = std::time::Instant::now();
            self.renderer.fx_adopt(frame_fx.take());
            let old = std::mem::replace(&mut self.prev, self.cur.replace(f));
            self.retire(old);
            self.drop_us = ta.elapsed().as_micros() as u32;
            self.prepared = false;
            let widen = self.widen().is_some();
            let (solver, prev, cur) = (&mut self.solver, self.prev.as_ref(), self.cur.as_ref().unwrap());
            solver.plan(prev, cur, widen);
            if frame_cut {
                solver.mark_cut();
            }
            self.adopt_us = ta.elapsed().as_micros() as u32;
            self.fresh += 1;
        }
        if defer {
            self.deferred += 1;
            self.undrawn = true;
            self.prepare_cur(state);
            self.last_fresh = true;
            let t0 = std::time::Instant::now();
            self.swap_present();
            self.last[2] = t0.elapsed().as_micros() as u32;
            self.times[2].push(self.last[2]);
            return;
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
        // a deferred frame (prepared at the last present, not drawn yet) is drawn now whatever the phase
        if moving || fresh || self.undrawn || self.blended || (widen.is_some() && !self.prepared) {
            if !self.prepared {
                self.prepare_cur(state);
            }
            let cur = self.cur.as_ref().unwrap();
            self.undrawn = false;
            let t0 = std::time::Instant::now();
            if moving || widen.is_some() {
                let prev = self.prev.as_ref();
                if moving {
                    let te = std::time::Instant::now();
                    self.solver.evaluate(prev, cur, t.unwrap());
                    self.interp_us[0].push(te.elapsed().as_micros() as u32);
                }
                // the fx world layer follows the same phase and the same camera
                self.renderer.fx_phase = if moving {
                    t.map(|t| melee_gx::fx::FxPhase { t, extrapolate: !interpolate, view: self.solver.primary_view() })
                } else {
                    None
                };
                let solver = &mut self.solver;
                let mut f = |i: usize, out: &mut [u8]| solver.patch(cur, i, out, widen, moving);
                self.renderer.draw_prepared(state, &cur.draws, Some(&mut f));
                if moving {
                    self.interp_us[1].push(self.renderer.last_fill_us);
                }
            } else {
                self.renderer.fx_phase = None;
                self.renderer.draw_prepared(state, &cur.draws, None);
            }
            self.blended = moving;
            let s = std::mem::take(&mut self.solver.stats);
            self.interp_total.add(&s);
            self.last[1] = t0.elapsed().as_micros() as u32;
            self.times[1].push(self.last[1]);
        }
        self.last_fresh = fresh;
        let t0 = std::time::Instant::now();
        self.swap_present();
        self.last[2] = t0.elapsed().as_micros() as u32;
        self.times[2].push(self.last[2]);
    }

    /// Free a frame the presenter no longer needs (see `offthread_drop`).
    fn retire(&mut self, f: Option<melee_gx::interp::Frame>) {
        let Some(f) = f else { return };
        self.renderer.forget(&f.draws);
        let Some(ok) = self.offthread_drop else { return drop(f) };
        if f.draws.capacity() == 0 || !ok(f.draws.as_ptr() as *const u8) {
            return drop(f);
        }
        let melee_gx::interp::Frame { draws, pose, packed } = f;
        drop(pose);
        drop(packed);
        let tx = self.dropper.get_or_insert_with(|| {
            let (tx, rx) = std::sync::mpsc::channel::<Vec<DrawCmd>>();
            let _ = std::thread::Builder::new().name("gx frame drop".into()).spawn(move || {
                while let Ok(v) = rx.recv() {
                    drop(v);
                }
            });
            tx
        });
        if let Err(e) = tx.send(draws) {
            drop(e.0);
        }
    }

    /// A sim frame that will never be presented (a newer one replaced it): when it holds EFB copies, render it
    /// offscreen so the copy targets later frames sample exist. `packed` is the decode thread's packing of `draws`.
    pub fn render_copies(&mut self, state: &GxState, draws: &[DrawCmd], packed: Option<melee_gx::render::Packed>) {
        if !draws.iter().any(|d| d.copy.is_some()) {
            return;
        }
        if let Some(p) = packed {
            self.renderer.set_packed(p);
        }
        self.renderer.render_frame(state, draws);
        // the renderer's other slot took this frame (the current one stays prepared); the EFB now holds it
        self.blended = true;
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
