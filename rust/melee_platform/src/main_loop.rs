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
    /// A request to switch the HUD overlay on / off (the game's settings overlay), taken once.
    fn take_hud_request(&mut self) -> Option<bool> {
        None
    }
    /// A live change of the frame pacing (the game's settings overlay), taken once. Presentation only.
    fn take_pacing(&mut self) -> Option<Pacing> {
        None
    }
    /// The swapchain's frame statistics (frames shown, display refreshes, seconds), for VRR detection
    /// (`Presenter::frame_stats`).
    fn display_stats(&mut self) -> Option<(u64, u64, f64)> {
        None
    }
}

/// A live pacing change: each field that is Some replaces the loop's value.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Pacing {
    /// Presented frames per second (0: no cap).
    pub fps_cap: Option<f64>,
    pub vsync: Option<bool>,
    /// Render-side interpolation: Some(forced) or None (auto above 60 fps).
    pub interp: Option<Option<bool>>,
}

/// `--vrr` / `MELEE_VRR`: whether the display follows the frames (G-Sync / FreeSync).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VrrMode {
    Auto,
    On,
    Off,
}

impl VrrMode {
    pub fn parse(v: &str) -> Option<Self> {
        match v.trim().to_ascii_lowercase().as_str() {
            "" | "auto" => Some(Self::Auto),
            "1" | "on" | "yes" | "true" => Some(Self::On),
            "0" | "off" | "no" | "false" => Some(Self::Off),
            _ => None,
        }
    }
}

/// The frame rate that keeps a variable-refresh display inside its range without ever waiting on a sync: a little
/// under the refresh, more so the faster the panel (144 Hz: 138, 240 Hz: 224).
pub fn vrr_cap(hz: f64) -> f64 {
    hz * (1.0 - hz / 3600.0)
}

/// One second of swapchain statistics while frames come slower than the refresh: a fixed-rate display keeps
/// refreshing at its full rate (showing some frames twice), a variable one refreshes once per frame.
/// Some(true): variable, Some(false): fixed, None: cannot tell (frames as fast as the refresh look the same on both).
pub fn judge_vrr(shown_per_s: f64, refreshes_per_s: f64, hz: f64) -> Option<bool> {
    if hz < 75.0 || shown_per_s < 30.0 || shown_per_s > 0.93 * hz {
        return None;
    }
    if refreshes_per_s > 0.97 * hz {
        return Some(false);
    }
    if (refreshes_per_s - shown_per_s).abs() <= 0.05 * shown_per_s {
        return Some(true);
    }
    None
}

/// Wait until `t`: a timer sleep for most of it, then yields for the last stretch (a plain sleep can wake late by
/// most of a millisecond, a whole frame at 1000 fps).
pub fn sleep_until(t: Instant) {
    loop {
        let now = Instant::now();
        if now >= t {
            return;
        }
        let left = t - now;
        if left > Duration::from_micros(1300) {
            std::thread::sleep(left - Duration::from_millis(1));
        } else {
            std::thread::yield_now();
        }
    }
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
    /// Pacing: the user's cap (0: none), vsync, VRR setting and verdict, the monitor's refresh.
    fps_cap: f64,
    vsync: bool,
    vrr_mode: VrrMode,
    vrr: Option<bool>,
    refresh_hz: f64,
    /// The next scheduled present and the time the current one is for (the phase comes from it, not from when
    /// the present call happens to run).
    next_present: Option<Instant>,
    present_at: Option<Instant>,
    /// VRR detection: the probe's end, the last statistics sample, verdicts in a row.
    vrr_probe_until: Option<Instant>,
    vrr_last: Option<(u64, u64, f64)>,
    vrr_votes: (Option<bool>, u32),
    /// Presents dropped because a tick came due first (their in-between pose was already stale).
    stale: u64,
    /// YouGame bridge: the Tab press was taken as Shift+Tab, so its release is ours too.
    yg_tab: bool,
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
    /// The present period the pacer holds to (None: present as fast as presentation allows).
    fn cap_period(&self) -> Option<Duration> {
        let mut cap = self.fps_cap;
        if self.vsync && self.refresh_hz >= 1.0 {
            if self.vrr_probe_until.is_some() {
                cap = 0.75 * self.refresh_hz; // VRR probe: frames slower than the refresh
            } else if self.vrr == Some(true) {
                let v = vrr_cap(self.refresh_hz);
                cap = if cap > 0.0 { cap.min(v) } else { v };
            }
        }
        (cap > 0.0).then(|| Duration::from_secs_f64(1.0 / cap))
    }

    /// During the VRR probe: sample the swapchain statistics once a second and vote.
    fn vrr_tick(&mut self) {
        let Some(until) = self.vrr_probe_until else { return };
        let now = Instant::now();
        if let Some(s) = self.game.display_stats() {
            match self.vrr_last {
                Some(l) if s.2 - l.2 >= 1.0 && s.0 >= l.0 && s.1 >= l.1 => {
                    let dt = s.2 - l.2;
                    let (shown, refr) = ((s.0 - l.0) as f64 / dt, (s.1 - l.1) as f64 / dt);
                    let v = judge_vrr(shown, refr, self.refresh_hz);
                    plog!("gw: video: VRR probe: {shown:.1} frames/s shown, {refr:.1} refreshes/s at {:.1} Hz -> {:?}", self.refresh_hz, v);
                    if v.is_some() && v == self.vrr_votes.0 {
                        self.vrr_votes.1 += 1;
                    } else {
                        self.vrr_votes = (v, 1);
                    }
                    self.vrr_last = Some(s);
                    if let (Some(x), 2..) = self.vrr_votes {
                        self.vrr = Some(x);
                        self.vrr_probe_until = None;
                    }
                }
                Some(l) if s.2 >= l.2 && s.0 >= l.0 && s.1 >= l.1 => {}
                _ => self.vrr_last = Some(s),
            }
        }
        if self.vrr_probe_until.is_some() && now >= until {
            self.vrr_probe_until = None;
            self.vrr = Some(false);
            plog!("gw: video: VRR probe: no verdict (statistics unavailable or inconclusive): fixed refresh assumed");
        }
        if self.vrr_probe_until.is_none() {
            let cap = self.cap_period().map_or(0.0, |p| 1.0 / p.as_secs_f64());
            plog!("gw: video: VRR {} -> {}", if self.vrr == Some(true) { "detected" } else { "not detected" }, if cap > 0.0 { format!("frame cap {cap:.1} fps") } else { "no frame cap".into() });
        }
    }

    /// A live pacing change: the cap and the vsync flag the pacer reads, the interpolation rule (the next fps
    /// sample applies an auto rule; a forced value applies now).
    fn apply_pacing(&mut self, p: Pacing) {
        if let Some(c) = p.fps_cap {
            self.fps_cap = c.max(0.0);
            self.next_present = None;
        }
        if let Some(v) = p.vsync {
            self.vsync = v;
            self.next_present = None;
        }
        if let Some(i) = p.interp {
            self.interp_env = i;
            if let Some(on) = i {
                if on != self.interp {
                    self.interp = on;
                    self.game.set_interp(on);
                }
            }
        }
        plog!("gw: video: pacing (live): fps cap {}, vsync {}, interp {}", if self.fps_cap > 0.0 { format!("{:.0}", self.fps_cap) } else { "none".into() }, if self.vsync { "on" } else { "off" }, match self.interp_env { None => "auto", Some(true) => "on", Some(false) => "off" });
    }

    fn run_due_ticks(&mut self) {
        if self.game.present_now() {
            // A present is owed first (a screenshot of the tick that asked for it): the redraw that follows a tick
            // loop calls here again, and a tick that had come due meanwhile would make the shot the NEXT frame's,
            // run to run as the load fell (Pokemon Stadium shots at one scene age differed by a tick). The time
            // stays in the clock; the ticks run right after the present.
            return;
        }
        let now = Instant::now();
        let dt = self.last.map(|l| now - l).unwrap_or_default();
        self.last = Some(now);
        for _ in 0..self.clock.advance(dt) {
            let t = Instant::now();
            self.game.tick(&mut self.pads);
            if let Some(on) = self.game.take_hud_request() {
                if on != self.hud_on {
                    self.hud_toggle();
                }
            }
            if let Some(p) = self.game.take_pacing() {
                self.apply_pacing(p);
            }
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
        if crate::yougame::embedded() {
            attrs = yougame_box_attrs(attrs);
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
                self.refresh_hz = std::env::var("MELEE_REFRESH_HZ").ok().and_then(|v| v.parse().ok()).unwrap_or_else(|| {
                    w.current_monitor().and_then(|m| m.refresh_rate_millihertz()).map_or(0.0, |r| r as f64 / 1000.0)
                });
                self.vrr = match self.vrr_mode {
                    VrrMode::On => Some(true),
                    VrrMode::Off => Some(false),
                    VrrMode::Auto => None,
                };
                if self.vrr.is_none() {
                    if self.vsync && self.refresh_hz >= 75.0 && self.decoupled {
                        self.vrr_probe_until = Some(Instant::now() + Duration::from_secs(6));
                    } else {
                        self.vrr = Some(false);
                    }
                }
                plog!("gw: video: refresh {:.2} Hz, vsync {}, VRR {:?}{}, fps cap {}", self.refresh_hz, if self.vsync { "on" } else { "off" }, self.vrr_mode,
                    if self.vrr_probe_until.is_some() { " (probing)" } else { "" }, if self.fps_cap > 0.0 { format!("{:.1}", self.fps_cap) } else { "none".into() });
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
                    crate::yougame::focus_lost();
                }
            }
            WindowEvent::ModifiersChanged(m) => self.mods = m.state(),
            WindowEvent::KeyboardInput { event, .. } => {
                if let PhysicalKey::Code(k) = event.physical_key {
                    if crate::yougame::active() {
                        // YouGame shortcuts (notes/yougame.md): Shift+Esc goes to the app, Shift+Tab opens the menu.
                        let down = event.state == ElementState::Pressed;
                        if k == KeyCode::Escape && self.mods.shift_key() {
                            if down && !event.repeat {
                                crate::yougame::relay_escape();
                            }
                            return;
                        }
                        if k == KeyCode::Tab && ((down && self.mods.shift_key()) || (!down && self.yg_tab)) {
                            self.yg_tab = down;
                            crate::yougame::shift_tab(down, event.repeat);
                            return;
                        }
                    }
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
                    if let Some(q) = self.pads.ui_keys.as_mut() {
                        // a UI mod holds the keyboard: its events, never the pads (melee_modhost::ui KEY_*)
                        let down = event.state == ElementState::Pressed;
                        let code = match k {
                            KeyCode::Enter | KeyCode::NumpadEnter => 1,
                            KeyCode::Escape => 2,
                            KeyCode::Backspace => 3,
                            KeyCode::ArrowUp => 4,
                            KeyCode::ArrowDown => 5,
                            KeyCode::ArrowLeft => 6,
                            KeyCode::ArrowRight => 7,
                            KeyCode::Tab => 8,
                            _ => 0,
                        };
                        if code != 0 && q.len() < 256 {
                            q.push((if down { 1 } else { 2 }) << 24 | code);
                        }
                        if down {
                            for ch in event.text.as_deref().unwrap_or("").chars().filter(|c| !c.is_control()) {
                                if q.len() < 256 {
                                    q.push(3 << 24 | (ch as u32 & 0xFF_FFFF));
                                }
                            }
                        }
                        return;
                    }
                    self.pads.key_event(k, event.state == ElementState::Pressed);
                }
            }
            WindowEvent::RedrawRequested => {
                if let Some(w) = self.window.clone() {
                    // A tick that came due since the present was scheduled runs first: the in-between pose of the
                    // older pair would be stale (shown after the frame it leads to exists).
                    let before = self.clock.ticks;
                    self.run_due_ticks();
                    if self.clock.ticks != before {
                        self.stale += 1;
                    }
                    if self.quit {
                        el.exit();
                        return;
                    }
                    // the phase of the time this present is scheduled for (the tick schedule), not of "now"
                    let at = self.present_at.take().unwrap_or_else(Instant::now);
                    let extra = self.last.map(|l| at.saturating_duration_since(l)).unwrap_or_default();
                    let t = Instant::now();
                    self.game.present(&w, self.clock.alpha_after(extra));
                    if !self.game.should_quit() {
                        crate::yougame::presented();
                    }
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
        if crate::yougame::active() {
            if crate::yougame::quit_requested() {
                self.quit = true;
            }
            if let Some(w) = self.window.as_ref() {
                yougame_box(w, crate::yougame::take_box_commands());
            }
        }
        self.run_due_ticks();
        if self.quit {
            el.exit();
            return;
        }
        self.vrr_tick();
        if let (Some(period), true) = (self.cap_period(), self.decoupled && self.window.is_some()) {
            // paced: present on a fixed grid; sleep to the next slot, or to the next tick if that comes first
            let now = Instant::now();
            let target = *self.next_present.get_or_insert(now);
            if now + Duration::from_micros(200) >= target {
                // the next slot is one period on, unless we fell behind by more than a period (then from now)
                let next = target + period;
                self.next_present = Some(if next + period < now { now + period } else { next });
                self.present_at = Some(target.max(now));
                self.window.as_ref().unwrap().request_redraw();
            } else {
                let tick_due = self.last.map_or(target, |l| l + self.clock.until_next());
                sleep_until(target.min(tick_due));
            }
            el.set_control_flow(ControlFlow::Poll);
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

/// The YouGame player box (YOUGAME_EMBED=1): the class and caption the app's launcher finds the window by, no frame,
/// hidden until the page says `show`; a child of YOUGAME_EMBED_PARENT when the app passes one (unverified on Windows).
fn yougame_box_attrs(mut attrs: winit::window::WindowAttributes) -> winit::window::WindowAttributes {
    attrs = attrs.with_title("Melee Unlocked").with_decorations(false).with_visible(false);
    #[cfg(windows)]
    {
        use winit::platform::windows::WindowAttributesExtWindows;
        attrs = attrs.with_class_name("MeleePortWindow");
        if let Some(h) = std::env::var("YOUGAME_EMBED_PARENT").ok().and_then(|v| v.trim().parse::<isize>().ok()).and_then(std::num::NonZeroIsize::new) {
            let raw = winit::raw_window_handle::RawWindowHandle::Win32(winit::raw_window_handle::Win32WindowHandle::new(h));
            // SAFETY: the app guarantees the parent outlives us; winit only passes it to CreateWindowEx.
            attrs = unsafe { attrs.with_parent_window(Some(raw)) };
            plog!("yougame: player box: parent window from YOUGAME_EMBED_PARENT");
        }
    }
    attrs
}

/// The player box's commands, on the window thread: `rect x y w h`, `show` / `hide`, `focus` (`blur`, `view`,
/// `volume` are logged and ignored).
fn yougame_box(w: &Window, cmds: Vec<String>) {
    for c in cmds {
        let n: Vec<i32> = c.split_whitespace().skip(1).filter_map(|v| v.parse().ok()).collect();
        match c.split_whitespace().next().unwrap_or("") {
            "rect" if n.len() == 4 && n[2] > 0 && n[3] > 0 => {
                w.set_outer_position(winit::dpi::PhysicalPosition::new(n[0], n[1]));
                let _ = w.request_inner_size(winit::dpi::PhysicalSize::new(n[2] as u32, n[3] as u32));
            }
            "show" => w.set_visible(true),
            "hide" => w.set_visible(false),
            "focus" => w.focus_window(),
            _ => plog!("yougame: player box command ignored: {}", c.chars().take(60).collect::<String>()),
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
        fps_cap: std::env::var("MELEE_FPS_CAP").ok().and_then(|v| v.trim().parse::<f64>().ok()).filter(|v| *v > 0.0).unwrap_or(0.0),
        vsync: crate::gfx::vsync_from_env(),
        vrr_mode: std::env::var("MELEE_VRR").ok().and_then(|v| VrrMode::parse(&v)).unwrap_or(VrrMode::Auto),
        vrr: None,
        refresh_hz: 0.0,
        next_present: None,
        present_at: None,
        vrr_probe_until: None,
        vrr_last: None,
        vrr_votes: (None, 0),
        stale: 0,
        yg_tab: false,
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
        plog!("gw: pacing: {} presents dropped stale (a tick came due first)", app.stale);
        plog!(
            "gw: perf: tick p50 {:.2} ms p99 {:.2} ms max {:.2} ms ({} ticks); frame p50 {:.2} ms p99 {:.2} ms max {:.2} ms, 1% low {:.0} fps ({} frames)",
            quantile(&mut t, 0.5), quantile(&mut t, 0.99), quantile(&mut t, 1.0), t.len(),
            quantile(&mut f, 0.5), quantile(&mut f, 0.99), quantile(&mut f, 1.0), low, f.len()
        );
    }
    Ok((app.clock.ticks, app.clock.dropped))
}
