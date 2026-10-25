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
use winit::keyboard::PhysicalKey;
use winit::window::{Window, WindowId};

pub const TICK_HZ: u64 = 60;
const NS: u64 = 1_000_000_000;
/// Most ticks run per wake before the clock drops the backlog (a stall, a debugger break).
pub const MAX_CATCH_UP: u32 = 5;

/// What the platform drives.
pub trait Game {
    /// One 60 Hz logic frame.
    fn tick(&mut self, pads: &mut PadSystem);
    /// Draw. `alpha` in [0,1): how far into the next tick real time is.
    fn present(&mut self, _window: &Arc<Window>, _alpha: f64) {}
    /// The window's client area changed (physical pixels); reconfigure a swapchain here.
    fn resized(&mut self, _width: u32, _height: u32) {}
    /// Return true to end the loop (checked after every tick).
    fn should_quit(&self) -> bool {
        false
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
    /// Time until the next tick is due.
    pub fn until_next(&self) -> Duration {
        Duration::from_nanos((NS - self.acc).div_ceil(TICK_HZ))
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

pub struct WindowConfig {
    pub title: String,
    pub width: u32,
    pub height: u32,
    /// false: create the window without taking focus (scripted / test runs, so the user's keyboard stays where it is).
    pub active: bool,
}
impl Default for WindowConfig {
    fn default() -> Self {
        Self { title: "GD's Melee".into(), width: 1280, height: 960, active: true }
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
}

impl<G: Game> App<G> {
    fn run_due_ticks(&mut self) {
        let now = Instant::now();
        let dt = self.last.map(|l| now - l).unwrap_or_default();
        self.last = Some(now);
        for _ in 0..self.clock.advance(dt) {
            self.game.tick(&mut self.pads);
            if self.game.should_quit() {
                self.quit = true;
                break;
            }
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
        match el.create_window(attrs) {
            Ok(w) => self.window = Some(Arc::new(w)),
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
            WindowEvent::KeyboardInput { event, .. } => {
                if let PhysicalKey::Code(k) = event.physical_key {
                    self.pads.key_event(k, event.state == ElementState::Pressed);
                }
            }
            WindowEvent::RedrawRequested => {
                if let Some(w) = self.window.clone() {
                    self.game.present(&w, self.clock.alpha());
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
        el.set_control_flow(ControlFlow::WaitUntil(Instant::now() + self.clock.until_next()));
    }
}

/// Open a window and run until it closes or the game quits. Returns (ticks run, ticks dropped).
pub fn run_windowed<G: Game>(game: G, pads: PadSystem, cfg: WindowConfig) -> Result<(u64, u64), winit::error::EventLoopError> {
    let el = EventLoop::new()?;
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
    };
    el.run_app(&mut app)?;
    Ok((app.clock.ticks, app.clock.dropped))
}
