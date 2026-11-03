//! Window + main loop. Logic runs at exactly 60 Hz (the C build's pacing: "the simulation stays
//! at exactly 60 Hz", shim_vi.c) on a fixed-step clock; presentation runs whenever winit redraws
//! and gets the fraction of a tick elapsed (`alpha`) for interpolation. Headless mode runs ticks
//! back to back with no window and no timing (the C's turbo).
//!
//! The game calls `PADRead` itself (as `HSD_PadRenewRawStatus` does), through the
//! [`PadSystem`] handed to [`Game::tick`]; the platform never reads pads behind its back.
use crate::pad_read::PadSystem;
use crate::plog;
use std::sync::Arc;
use std::time::{Duration, Instant};
use winit::application::ApplicationHandler;
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, ModifiersState, PhysicalKey};
use winit::window::{Fullscreen, Window, WindowId};

pub const TICK_HZ: u64 = 60;
const NS: u64 = 1_000_000_000;
/// Most ticks run per wake before the clock drops the backlog (a stall, a debugger break).
pub const MAX_CATCH_UP: u32 = 5;
/// A tick at least this long is a load (see `run_due_ticks`): the time it took is not caught up.
pub const LOAD_TICK: Duration = Duration::from_millis(50);

/// What the platform drives.
pub trait Game {
    /// One 60 Hz logic frame.
    fn tick(&mut self, pads: &mut PadSystem);
    /// Draw. `alpha` in [0,1): how far into the next tick real time is.
    fn present(&mut self, _window: &Arc<Window>, _alpha: f64) {}
    /// Render-only interpolation switched on/off (MELEE_INTERP, or auto above 60 fps).
    fn set_interp(&mut self, _on: bool) {}
    /// The window's client area changed (physical pixels); reconfigure a swapchain here.
    fn resized(&mut self, _width: u32, _height: u32) {}
    /// Return true to end the loop (checked after every tick).
    fn should_quit(&self) -> bool {
        false
    }
    /// Return true to present before the next tick runs (a screenshot of this tick's frame is pending).
    fn present_now(&self) -> bool {
        false
    }
    /// Extra state for the MELEE_SHOW_FPS title line (netplay: ping, rollbacks), asked once a second.
    fn status_line(&mut self) -> Option<String> {
        None
    }
    /// The game's lines for the HUD overlay (netplay: ping, delay, rollbacks, health), asked a few times a second
    /// while it is on. Read-only: nothing here may change the simulation.
    fn overlay_lines(&mut self) -> Vec<crate::overlay::Line> {
        Vec::new()
    }
    /// The HUD overlay's lines to draw from now on (None: off); a game with a Presenter puts them in `Presenter::hud`.
    fn set_hud(&mut self, _lines: Option<Vec<crate::overlay::Line>>) {}
}

/// Fixed 60 Hz step clock in exact integer arithmetic (no drift: time is kept in ns * 60).
#[derive(Clone, Debug, Default)]
pub struct FixedStep {
    acc: u64, // ns * TICK_HZ
    pub ticks: u64,
    pub dropped: u64,
}

impl FixedStep {
    /// Add elapsed real time; return how many ticks are due (at most [`MAX_CATCH_UP`]).
    pub fn advance(&mut self, dt: Duration) -> u32 {
        self.acc += dt.as_nanos() as u64 * TICK_HZ;
        let due = self.acc / NS;
        self.acc %= NS;
        let run = due.min(MAX_CATCH_UP as u64);
        self.dropped += due - run;
        self.ticks += run;
        run as u32
    }
    /// Fraction of a tick accumulated.
    pub fn alpha(&self) -> f64 {
        self.acc as f64 / NS as f64
    }
    /// Fraction of a tick accumulated plus `extra` real time not yet added, clamped below 1.
    pub fn alpha_after(&self, extra: Duration) -> f64 {
        let a = (self.acc as u128 + extra.as_nanos() * TICK_HZ as u128) as f64 / NS as f64;
        a.min(0.999_999)
    }
    /// Drop the fraction accumulated (a long load tick: its wall time is not owed to the sim).
    pub fn forgive(&mut self) {
        self.acc = 0;
    }
    /// Time until the next tick is due.
    pub fn until_next(&self) -> Duration {
        Duration::from_nanos((NS - self.acc).div_ceil(TICK_HZ))
    }
}

#[cfg(test)]
mod display_tests {
    use super::*;

    #[test]
    fn fullscreen_parse_and_toggle() {
        assert_eq!(FullscreenMode::parse("exclusive"), Some(FullscreenMode::Exclusive));
        assert_eq!(FullscreenMode::parse("1"), Some(FullscreenMode::Borderless));
        assert_eq!(FullscreenMode::parse("Off"), Some(FullscreenMode::Off));
        assert_eq!(FullscreenMode::parse("maybe"), None);
        use FullscreenMode::*;
        assert_eq!(FullscreenMode::toggled(Off, Off), Borderless);
        assert_eq!(FullscreenMode::toggled(Off, Exclusive), Exclusive);
        assert_eq!(FullscreenMode::toggled(Borderless, Borderless), Off);
        assert_eq!(FullscreenMode::toggled(Exclusive, Exclusive), Off);
    }

    #[test]
    fn exclusive_mode_choice() {
        let m = [(1920, 1080, 60000, 32), (2560, 1440, 59950, 32), (2560, 1440, 143998, 32), (1280, 720, 240000, 32)];
        assert_eq!(choose_video_mode(&m), Some(2));
        assert_eq!(choose_video_mode(&[]), None);
    }
}

/// Run `frames` ticks with no window (the C's turbo / headless test mode). Returns ticks run.
pub fn run_headless<G: Game>(game: &mut G, pads: &mut PadSystem, frames: u64) -> u64 {
    let mut n = 0;
    while n < frames {
        game.tick(pads);
        n += 1;
        if game.should_quit() {
            break;
        }
    }
    n
}

/// How the window fills the screen (`--fullscreen`, `MELEE_FULLSCREEN`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FullscreenMode {
    /// A normal window.
    #[default]
    Off,
    /// Borderless fullscreen on the window's monitor (the desktop's mode; no mode switch).
    Borderless,
    /// Exclusive fullscreen: the monitor's largest video mode (highest refresh at that size). Falls back to
    /// borderless when the monitor reports no modes.
    Exclusive,
}

impl FullscreenMode {
    /// `0/off/no/false/windowed` -> Off, `1/on/yes/true/borderless` -> Borderless, `exclusive` -> Exclusive.
    pub fn parse(v: &str) -> Option<Self> {
        Some(match v.trim().to_ascii_lowercase().as_str() {
            "" | "0" | "off" | "no" | "false" | "windowed" => Self::Off,
            "1" | "on" | "yes" | "true" | "borderless" => Self::Borderless,
            "2" | "exclusive" => Self::Exclusive,
            _ => return None,
        })
    }

    /// The mode Alt+Enter switches to from `current` (`preferred`: the configured fullscreen kind, borderless when Off).
    pub fn toggled(current: Self, preferred: Self) -> Self {
        match current {
            Self::Off => if preferred == Self::Off { Self::Borderless } else { preferred },
            _ => Self::Off,
        }
    }
}

/// The video mode exclusive fullscreen picks from `modes` (width, height, refresh in mHz, bit depth): the largest area,
/// then the highest refresh, then the deepest colour. Its index.
pub fn choose_video_mode(modes: &[(u32, u32, u32, u16)]) -> Option<usize> {
    (0..modes.len()).max_by_key(|&i| {
        let (w, h, r, b) = modes[i];
        (w as u64 * h as u64, r, b)
    })
}

pub struct WindowConfig {
    pub title: String,
    pub width: u32,
    pub height: u32,
    /// false: create the window without taking focus (scripted / test runs, so the user's keyboard stays where it is).
    pub active: bool,
    /// Start fullscreen (Alt+Enter toggles at run time). Ignored under MELEE_WINDOW_UNFOCUSED (test runs never take
    /// the user's screen).
    pub fullscreen: FullscreenMode,
    /// The HUD overlay is on at start (F3 toggles it at run time).
    pub overlay: bool,
}
impl Default for WindowConfig {
    fn default() -> Self {
        Self { title: "GD's Melee".into(), width: 1280, height: 960, active: true, fullscreen: FullscreenMode::Off, overlay: false }
    }
}

/// The winit fullscreen value for `mode` on `w`'s current monitor.
fn winit_fullscreen(w: &Window, mode: FullscreenMode) -> Option<Fullscreen> {
    match mode {
        FullscreenMode::Off => None,
        FullscreenMode::Borderless => Some(Fullscreen::Borderless(None)),
        FullscreenMode::Exclusive => {
            let mon = w.current_monitor().or_else(|| w.primary_monitor())?;
            let modes: Vec<_> = mon.video_modes().collect();
            let keys: Vec<_> = modes.iter().map(|m| (m.size().width, m.size().height, m.refresh_rate_millihertz(), m.bit_depth())).collect();
            match choose_video_mode(&keys) {
                Some(i) => {
                    let (a, b, r, _) = keys[i];
                    plog!("gw: video: exclusive fullscreen {a}x{b} @ {:.2} Hz", r as f64 / 1000.0);
                    Some(Fullscreen::Exclusive(modes[i].clone()))
                }
                None => Some(Fullscreen::Borderless(Some(mon))),
            }
        }
    }
}

struct App<G: Game> {
    game: G,
    pads: PadSystem,
    cfg: WindowConfig,
    window: Option<Arc<Window>>,
    clock: FixedStep,
    last: Option<Instant>,
    focus_lost_at: Option<Instant>,
    focus_released: bool,
    quit: bool,
    /// Render as often as the swapchain allows instead of once per tick.
    decoupled: bool,
    /// MELEE_INTERP: Some(forced) or None (auto: on when rendering above 60 fps).
    interp_env: Option<bool>,
    interp: bool,
    show_fps: bool,
    fps_t0: Option<Instant>,
    fps_frames: u64,
    fps_ticks0: u64,
    /// Alt held (Alt+Enter toggles fullscreen).
    mods: ModifiersState,
    /// The current fullscreen state (what the last switch asked for).
    fullscreen: FullscreenMode,
    /// Rendered frames over the whole run (for the exit log).
    pub frames: u64,
    run_t0: Option<Instant>,
    tick_time: Duration,
    present_time: Duration,
    /// MELEE_FRAME_TIMES=<csv>: one row per tick (T) and per present (P): kind, index, ms since the loop
    /// started, ms the call took (benchmarks; off by default).
    trace: Option<std::io::BufWriter<std::fs::File>>,
    /// MELEE_SHOW_FPS: every tick's duration and every frame's interval (µs), for the exit percentiles.
    tick_us: Vec<u32>,
    frame_us: Vec<u32>,
    last_frame: Option<Instant>,
    /// HUD overlay (F3): on/off, its refresh clock, and the frame intervals / frames / ticks since the last refresh.
    hud_on: bool,
    hud_t0: Option<Instant>,
    hud_frames: crate::overlay::FrameWindow,
    hud_last_frame: Option<Instant>,
    hud_n: (u64, u64),
    hud_shown: bool,
}

/// The `q` quantile of `v` (sorted in place).
fn quantile(v: &mut [u32], q: f64) -> f64 {
    if v.is_empty() {
        return 0.0;
    }
    v.sort_unstable();
    v[((v.len() - 1) as f64 * q).round() as usize] as f64 / 1e3
}

fn env_bool(k: &str) -> Option<bool> {
    let v = std::env::var(k).ok()?.to_ascii_lowercase();
    Some(!matches!(v.as_str(), "" | "0" | "off" | "false" | "no"))
}

impl<G: Game> App<G> {
    fn run_due_ticks(&mut self) {
        let now = Instant::now();
        let dt = self.last.map(|l| now - l).unwrap_or_default();
        self.last = Some(now);
        for _ in 0..self.clock.advance(dt) {
            let t = Instant::now();
            self.game.tick(&mut self.pads);
            let el = t.elapsed();
            self.tick_time += el;
            if let (Some(f), Some(t0)) = (self.trace.as_mut(), self.run_t0) {
                use std::io::Write;
                let _ = writeln!(f, "T,{},{:.3},{:.3}", self.clock.ticks, (t - t0).as_secs_f64() * 1e3, el.as_secs_f64() * 1e3);
            }
            if self.show_fps {
                self.tick_us.push(el.as_micros().min(u32::MAX as u128) as u32);
            }
            if self.game.should_quit() {
                self.quit = true;
                break;
            }
            if self.game.present_now() {
                break; // the rest of the due time stays in the clock
            }
            if el >= LOAD_TICK {
                // A scene load (archive parse, device open): the real time it took is not owed to the sim. Without
                // this the next advance runs MAX_CATCH_UP ticks back to back (a fast-forward burst right after the
                // load, no presents in between). Pacing only: the ticks themselves are unchanged.
                self.last = Some(Instant::now());
                self.clock.forgive();
                break;
            }
        }
    }

    /// Once a second: measure render fps / sim tps, apply the MELEE_INTERP auto rule, show MELEE_SHOW_FPS.
    fn fps_tick(&mut self, w: &Arc<Window>) {
        let now = Instant::now();
        let t0 = *self.fps_t0.get_or_insert(now);
        let dt = (now - t0).as_secs_f64();
        if dt < 1.0 {
            return;
        }
        let fps = self.fps_frames as f64 / dt;
        let tps = (self.clock.ticks - self.fps_ticks0) as f64 / dt;
        self.fps_t0 = Some(now);
        self.fps_frames = 0;
        self.fps_ticks0 = self.clock.ticks;
        let (sim_ms, ren_ms) = (std::mem::take(&mut self.tick_time).as_secs_f64() * 1e3, std::mem::take(&mut self.present_time).as_secs_f64() * 1e3);
        let want = self.interp_env.unwrap_or(fps > 65.0);
        if want != self.interp {
            self.interp = want;
            self.game.set_interp(want);
        }
        if self.show_fps {
            plog!("gw: fps: render {fps:.1} sim {tps:.1} tps (interp {}; sim {:.0} ms, render {:.0} ms of {:.0})", if self.interp { "on" } else { "off" }, sim_ms, ren_ms, dt * 1e3);
            let extra = self.game.status_line().map_or(String::new(), |x| format!(" - {x}"));
            if !extra.is_empty() {
                plog!("gw: status:{extra}");
            }
            w.set_title(&format!("{} - {fps:.0} fps / {tps:.1} tps{extra}", self.cfg.title));
        }
    }

    /// Twice a second while the overlay is on: fps, frame time (mean / worst), sim rate, then the game's lines.
    fn hud_tick(&mut self) {
        let now = Instant::now();
        if let Some(l) = self.hud_last_frame.replace(now) {
            self.hud_frames.push((now - l).as_micros().min(u32::MAX as u128) as u32);
        }
        let t0 = *self.hud_t0.get_or_insert(now);
        let dt = (now - t0).as_secs_f64();
        if dt < 0.5 && self.hud_shown {
            return;
        }
        self.hud_t0 = Some(now);
        let (frames, ticks) = (self.frames - self.hud_n.0, self.clock.ticks - self.hud_n.1);
        self.hud_n = (self.frames, self.clock.ticks);
        let (mean, worst) = self.hud_frames.take();
        let (fps, tps) = if dt > 0.0 { (frames as f64 / dt, ticks as f64 / dt) } else { (0.0, 0.0) };
        use crate::overlay::*;
        let mut lines = vec![
            (format!("{fps:.0} fps  {mean:.1} ms (worst {worst:.1})"), if worst > 34.0 { WARN } else { WHITE }),
            (format!("sim {tps:.1} tps{}", if self.interp { "  interp" } else { "" }), if dt > 0.0 && (tps - 60.0).abs() > 1.5 { WARN } else { DIM }),
        ];
        lines.extend(self.game.overlay_lines());
        self.hud_shown = true;
        self.game.set_hud(Some(lines));
    }

    fn hud_toggle(&mut self) {
        self.hud_on = !self.hud_on;
        plog!("gw: overlay {}", if self.hud_on { "on" } else { "off" });
        if !self.hud_on {
            self.hud_shown = false;
            self.game.set_hud(None);
        } else {
            self.hud_t0 = None;
            self.hud_last_frame = None;
            self.hud_frames.take();
            self.hud_n = (self.frames, self.clock.ticks);
        }
    }

    /// `gw_pad_focus_tick`: a loss acts after 100 ms, a gain at once.
    fn focus_tick(&mut self) {
        let want_release = self.focus_lost_at.is_some_and(|t| t.elapsed() >= Duration::from_millis(100));
        if want_release && !self.focus_released {
            self.focus_released = true;
            self.pads.set_focus(false);
        } else if !want_release && self.focus_released {
            self.focus_released = false;
            self.pads.set_focus(true);
        }
    }
}

impl<G: Game> ApplicationHandler for App<G> {
    fn resumed(&mut self, el: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let mut attrs = Window::default_attributes()
            .with_title(self.cfg.title.clone())
            .with_active(self.cfg.active)
            .with_inner_size(winit::dpi::LogicalSize::new(self.cfg.width, self.cfg.height));
        // MELEE_WINDOW_UNFOCUSED=1 (test harnesses): never take the user's keyboard focus.
        // MELEE_WINDOW_UNFOCUSED=hidden also leaves the window invisible (EFB shots still work).
        if let Ok(v) = std::env::var("MELEE_WINDOW_UNFOCUSED") {
            attrs = attrs.with_active(false);
            if v == "hidden" {
                attrs = attrs.with_visible(false);
            } else {
                attrs = attrs.with_window_level(winit::window::WindowLevel::AlwaysOnBottom);
            }
        }
        let unattended = std::env::var_os("MELEE_WINDOW_UNFOCUSED").is_some();
        if self.cfg.fullscreen != FullscreenMode::Off && unattended {
            plog!("gw: video: fullscreen ignored under MELEE_WINDOW_UNFOCUSED (a test run never takes the screen)");
        }
        match el.create_window(attrs) {
            Ok(w) => {
                if self.cfg.fullscreen != FullscreenMode::Off && !unattended {
                    w.set_fullscreen(winit_fullscreen(&w, self.cfg.fullscreen));
                    self.fullscreen = self.cfg.fullscreen;
                    plog!("gw: video: fullscreen {:?}", self.fullscreen);
                }
                let sz = w.inner_size();
                self.game.resized(sz.width, sz.height);
                self.window = Some(Arc::new(w));
            }
            Err(e) => {
                plog!("gw: video: window creation failed: {e}");
                el.exit();
            }
        }
        self.last = Some(Instant::now());
    }

    fn window_event(&mut self, el: &ActiveEventLoop, _id: WindowId, ev: WindowEvent) {
        match ev {
            WindowEvent::CloseRequested => el.exit(),
            WindowEvent::Resized(sz) => self.game.resized(sz.width, sz.height),
            WindowEvent::Focused(f) => {
                if f {
                    self.focus_lost_at = None;
                    self.pads.focused = true;
                } else {
                    self.focus_lost_at = Some(Instant::now());
                    self.pads.focused = false;
                }
            }
            WindowEvent::ModifiersChanged(m) => self.mods = m.state(),
            WindowEvent::KeyboardInput { event, .. } => {
                if let PhysicalKey::Code(k) = event.physical_key {
                    let enter = matches!(k, KeyCode::Enter | KeyCode::NumpadEnter);
                    if enter && self.mods.alt_key() {
                        // Alt+Enter: fullscreen <-> window (the key never reaches the pads)
                        if event.state == ElementState::Pressed && !event.repeat {
                            if let Some(w) = self.window.as_ref() {
                                let next = FullscreenMode::toggled(self.fullscreen, self.cfg.fullscreen);
                                w.set_fullscreen(winit_fullscreen(w, next));
                                self.fullscreen = next;
                                plog!("gw: video: Alt+Enter -> fullscreen {next:?}");
                            }
                        }
                        return;
                    }
                    if k == KeyCode::F3 {
                        // F3: the HUD overlay on/off (the key never reaches the pads)
                        if event.state == ElementState::Pressed && !event.repeat {
                            self.hud_toggle();
                        }
                        return;
                    }
                    self.pads.key_event(k, event.state == ElementState::Pressed);
                }
            }
            WindowEvent::RedrawRequested => {
                if let Some(w) = self.window.clone() {
                    let extra = self.last.map(|l| l.elapsed()).unwrap_or_default();
                    let t = Instant::now();
                    self.game.present(&w, self.clock.alpha_after(extra));
                    let d = t.elapsed();
                    self.present_time += d;
                    if let (Some(f), Some(t0)) = (self.trace.as_mut(), self.run_t0) {
                        use std::io::Write;
                        let _ = writeln!(f, "P,{},{:.3},{:.3}", self.frames, (t - t0).as_secs_f64() * 1e3, d.as_secs_f64() * 1e3);
                    }
                    if self.show_fps {
                        let now = Instant::now();
                        if let Some(l) = self.last_frame {
                            self.frame_us.push((now - l).as_micros().min(u32::MAX as u128) as u32);
                        }
                        self.last_frame = Some(now);
                    }
                    self.frames += 1;
                    self.fps_frames += 1;
                    self.fps_tick(&w);
                    if self.hud_on {
                        self.hud_tick();
                    }
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, el: &ActiveEventLoop) {
        self.focus_tick();
        self.run_due_ticks();
        if self.quit {
            el.exit();
            return;
        }
        if let Some(w) = self.window.as_ref() {
            w.request_redraw();
        }
        if self.decoupled {
            // Render as fast as presentation allows (vsync on: blocks in present at the refresh
            // rate; off: uncapped). The fixed-step clock still runs exactly 60 ticks a second.
            el.set_control_flow(ControlFlow::Poll);
        } else {
            el.set_control_flow(ControlFlow::WaitUntil(Instant::now() + self.clock.until_next()));
        }
    }
}

/// Open a window and run until it closes or the game quits. Returns (ticks run, ticks dropped).
pub fn run_windowed<G: Game>(game: G, pads: PadSystem, cfg: WindowConfig) -> Result<(u64, u64), winit::error::EventLoopError> {
    let el = EventLoop::new()?;
    let hud_on = cfg.overlay || env_bool("MELEE_OVERLAY").unwrap_or(false);
    let mut app = App {
        game,
        pads,
        cfg,
        window: None,
        clock: FixedStep::default(),
        last: None,
        focus_lost_at: None,
        focus_released: false,
        quit: false,
        // MELEE_RENDER_DECOUPLED=0 restores the old once-per-tick redraw.
        decoupled: env_bool("MELEE_RENDER_DECOUPLED").unwrap_or(true),
        interp_env: env_bool("MELEE_INTERP"),
        interp: false,
        show_fps: env_bool("MELEE_SHOW_FPS").unwrap_or(false),
        fps_t0: None,
        fps_frames: 0,
        fps_ticks0: 0,
        mods: ModifiersState::empty(),
        fullscreen: FullscreenMode::Off,
        frames: 0,
        run_t0: Some(Instant::now()),
        tick_time: Duration::ZERO,
        present_time: Duration::ZERO,
        trace: std::env::var_os("MELEE_FRAME_TIMES").and_then(|p| std::fs::File::create(p).ok()).map(|f| std::io::BufWriter::with_capacity(1 << 20, f)),
        tick_us: Vec::new(),
        frame_us: Vec::new(),
        last_frame: None,
        hud_on,
        hud_t0: None,
        hud_frames: Default::default(),
        hud_last_frame: None,
        hud_n: (0, 0),
        hud_shown: false,
    };
    if let Some(f) = app.trace.as_mut() {
        use std::io::Write;
        let unix = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default();
        let _ = writeln!(f, "S,0,{:.3},0", unix.as_secs_f64() * 1e3);
    }
    if app.interp_env == Some(true) {
        app.interp = true;
        app.game.set_interp(true);
    }
    el.run_app(&mut app)?;
    if let Some(f) = app.trace.as_mut() {
        use std::io::Write;
        let _ = f.flush();
    }
    if app.show_fps {
        let secs = app.run_t0.map(|t| t.elapsed().as_secs_f64()).unwrap_or(1.0).max(1e-9);
        plog!("gw: fps: run total: {} frames, {} ticks in {secs:.2}s = render {:.1} fps, sim {:.2} tps", app.frames, app.clock.ticks, app.frames as f64 / secs, app.clock.ticks as f64 / secs);
        let (mut t, mut f) = (std::mem::take(&mut app.tick_us), std::mem::take(&mut app.frame_us));
        // 1% low: the frame rate of the slowest 1% of frames (their mean interval)
        let low = {
            let mut g = f.clone();
            g.sort_unstable_by(|a, b| b.cmp(a));
            let n = (g.len() / 100).max(1).min(g.len());
            let mean = g[..n].iter().map(|&x| x as f64).sum::<f64>() / n.max(1) as f64;
            if mean > 0.0 { 1e6 / mean } else { 0.0 }
        };
        plog!(
            "gw: perf: tick p50 {:.2} ms p99 {:.2} ms max {:.2} ms ({} ticks); frame p50 {:.2} ms p99 {:.2} ms max {:.2} ms, 1% low {:.0} fps ({} frames)",
            quantile(&mut t, 0.5), quantile(&mut t, 0.99), quantile(&mut t, 1.0), t.len(),
            quantile(&mut f, 0.5), quantile(&mut f, 0.99), quantile(&mut f, 1.0), low, f.len()
        );
    }
    Ok((app.clock.ticks, app.clock.dropped))
}
