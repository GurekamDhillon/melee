//! `melee view-golden` on the consolidated engine: the real scene loop (`melee_engine::Engine`) stepped once per tick,
//! its render pass drawn into melee_gx through the GX bridge and presented in a winit window. The frame is the game's
//! own: stage, fighters, camera, HUD, effects. Screenshots (`--shots`) are written from the presenter's EFB.
use melee_engine::Engine;
use melee_gx::draw::DrawCmd;
use melee_platform::gfx::{write_png, Presenter};
use melee_platform::{run_windowed, Game, PadSystem, WindowConfig};
use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::Arc;
use winit::window::Window;

struct EngineView {
    e: Engine,
    presenter: Option<Presenter>,
    frame: Option<Vec<DrawCmd>>,
    shots: Vec<u32>,
    shot_dir: PathBuf,
    max_frames: Option<u32>,
    quit: bool,
    /// a `--shots` frame was ticked: the next present writes it
    pending_shot: Option<u32>,
}

impl Game for EngineView {
    fn tick(&mut self, _pads: &mut PadSystem) {
        if self.quit {
            return;
        }
        if !self.e.tick() {
            eprintln!("view-golden: the scene ended after {} frames", self.e.frames);
            self.quit = true;
        }
        if let Some(f) = melee_engine::gxbridge::take_frame() {
            self.frame = Some(f);
        }
        let n = self.e.frames.saturating_sub(1);
        if self.shots.contains(&n) {
            self.pending_shot = Some(n);
        }
        if self.max_frames.is_some_and(|n| self.e.frames >= n) {
            self.quit = true;
        }
    }
    fn present(&mut self, window: &Arc<Window>, _alpha: f64) {
        if self.presenter.is_none() {
            match Presenter::new(window.clone()) {
                Ok(p) => self.presenter = Some(p),
                Err(e) => {
                    eprintln!("view-golden: {e}");
                    self.quit = true;
                    return;
                }
            }
        }
        let p = self.presenter.as_mut().unwrap();
        let frame = self.frame.take();
        p.present(&melee_engine::gxbridge::shim().gx.state, frame.as_deref());
        if let (Some(n), true) = (self.pending_shot.take(), frame.is_some()) {
            let px = p.screenshot();
            let path = self.shot_dir.join(format!("engine_{n:04}.png"));
            match write_png(&path, p.renderer.width, p.renderer.height, &px) {
                Ok(()) => eprintln!("view-golden: frame {n} -> {}", path.display()),
                Err(e) => eprintln!("view-golden: screenshot: {e}"),
            }
        }
    }
    fn resized(&mut self, w: u32, h: u32) {
        if let Some(p) = self.presenter.as_mut() {
            p.resize(w, h);
        }
    }
    fn should_quit(&self) -> bool {
        self.quit
    }
}

pub fn view_golden(args: &[String]) -> ExitCode {
    let get = |k: &str| args.iter().position(|a| a == k).and_then(|i| args.get(i + 1)).cloned();
    let Some(iso) = get("--iso").or_else(|| std::env::var("GW_ISO_VANILLA").ok()) else {
        eprintln!("usage: melee view-golden --iso DISC.iso [--scenario S] [--frames N] [--shots 30,60] [--shot-dir DIR]");
        return ExitCode::from(2);
    };
    let scenario = get("--scenario").unwrap_or_else(|| "pad_falcon_fox_fd".into());
    unsafe { melee_engine::engine::RENDER = true };
    let e = match crate::boot_engine_scenario(&iso, &scenario) {
        Ok(e) => e,
        Err(er) => {
            eprintln!("view-golden: {er}");
            return ExitCode::FAILURE;
        }
    };
    let shot_dir = PathBuf::from(get("--shot-dir").unwrap_or_else(|| "target/shots".into()));
    let _ = std::fs::create_dir_all(&shot_dir);
    let view = EngineView {
        e,
        presenter: None,
        frame: None,
        shots: get("--shots").map_or(vec![], |s| s.split(',').filter_map(|x| x.trim().parse().ok()).collect()),
        shot_dir,
        max_frames: get("--frames").and_then(|v| v.parse().ok()),
        quit: false,
        pending_shot: None,
    };
    let cfg = WindowConfig { title: format!("melee view-golden - {scenario}"), width: 1280, height: 960, active: std::env::var_os("MELEE_PAD_SCRIPT").is_none() && std::env::var_os("MELEE_WINDOW_UNFOCUSED").is_none(), ..Default::default() };
    match run_windowed(view, PadSystem::headless(), cfg) {
        Ok((ticks, _)) => {
            eprintln!("view-golden: closed after {ticks} ticks");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("view-golden: event loop: {e}");
            ExitCode::FAILURE
        }
    }
}
