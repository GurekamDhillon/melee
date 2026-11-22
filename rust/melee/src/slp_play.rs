//! `melee play --replay FILE.slp`: a Slippi replay played in the normal game window (docs/rust-port/notes/spectate-replay.md).
//!
//! The match is the one `melee golden --slp` simulates, set up by the same code: `slp_open` (the Game Start block
//! converted by `slpcheck::convert`, the seed, the replay's Gecko-derived codes: widescreen, UCF, Pokemon Stadium,
//! DbLevel, Neutral Spawn), `replay_modes_active`, `Engine::boot_vs` and `install_replay`; every frame runs
//! [`replay_frame`] (the player's `Replay_Tick` and the recorded seed) and then `Engine::tick`, as golden's loop does.
//! The only additions are presentation: the GX frames go to a `Presenter` in a `run_windowed` window, the sound plays,
//! and the replay controls (replay_ctl.rs: pause, step, speed, camera) pace the ticks.
//!
//! `--hash-out FILE` writes golden's CSV (`frame,hash,seed,...`, one row per engine tick, [`csv_row`]) so a windowed
//! playback can be compared with `melee golden --slp FILE --out GOLDEN.csv --frames N` row for row.
//!
//! The sim view: a console replay names none, and golden then runs the C recordings' fixed 16:9 camera scale (engine.rs
//! `boot_host`). A rendered run takes the view hooks instead, so the playback sets the session's sim view to that same
//! 16:9 (the projection scale the VS scene answers is then bit-identical) and the presenter draws 16:9 content.
use std::process::ExitCode;
use std::sync::Arc;

use melee_platform::gfx::Presenter;
use melee_platform::pad::*;
use melee_platform::{run_windowed, Game, PadSystem};
use winit::window::Window;

fn log(m: &str) {
    eprintln!("slpplay: {m}");
}

/// A Slippi replay: the `.slp` extension, else the file's UBJSON start (`{U\x03raw[$U#l`).
pub fn is_slp(path: &str) -> bool {
    if std::path::Path::new(path).extension().is_some_and(|e| e.eq_ignore_ascii_case("slp")) {
        return true;
    }
    use std::io::Read;
    let mut head = [0u8; 6];
    std::fs::File::open(path).and_then(|mut f| f.read_exact(&mut head)).is_ok() && &head == b"{U\x03raw"
}

/// golden's CSV header (main.rs `golden_engine`).
pub(crate) const CSV_HEADER: &str = "frame,hash,seed,items,itemsum,s0,s1,s2,s3,s4,s5,s6,s7,s8,s9,s10,s11\n";

/// One golden CSV row: engine tick `f` and its `Engine::neutral_row`.
pub(crate) fn csv_row(f: usize, r: &[u32; 16]) -> String {
    let mut s = format!("{f},{:08X},{:08X},{},{:08X}", r[0], r[1], r[2], r[3]);
    for w in &r[4..16] {
        s.push_str(&format!(",{w:08X}"));
    }
    s.push('\n');
    s
}

/// Before each engine tick of a replay (gmscene.c: `Replay_Tick`, then the forced seed): the fighters read the
/// recorded inputs through the `Replay_*` platform hooks `install_replay` set.
pub(crate) fn replay_frame() {
    let pl = unsafe { crate::r_in() };
    pl.tick();
    let sd = pl.resync_seed();
    if sd != 0 {
        melee_runtime::tu::random::global::state().set_logic_seed(sd);
    }
}

struct SlpPlay {
    e: melee_engine::Engine,
    presenter: Option<Presenter>,
    frame: Option<Vec<melee_gx::draw::DrawCmd>>,
    packed: Option<melee_gx::render::Packed>,
    audio: Option<crate::audio::Audio>,
    ctl: crate::replay_ctl::Ctl,
    tick_no: u64,
    /// render ticks since the match scene ended (the window closes after a moment unless `--replay-stay`)
    end_age: Option<u32>,
    stay: bool,
    /// the player's last recorded frame: a replay whose scene never ends stops this long after it
    last_frame: i32,
    hash_out: Option<std::io::BufWriter<std::fs::File>>,
    interp: bool,
    quit: bool,
    failed: Option<String>,
}

impl SlpPlay {
    /// One engine tick of the replay (golden's loop body: replay frame, tick, the hash row).
    fn step(&mut self) -> bool {
        let f = self.e.frames;
        replay_frame();
        let alive = self.e.tick();
        if let Some(fr) = melee_engine::gxbridge::take_frame() {
            self.frame = Some(fr);
            self.packed = melee_engine::gxbridge::shim().gx.packed.take();
        }
        // golden's loop: the tick on which the scene ends writes no row
        if let (true, Some(w)) = (alive, self.hash_out.as_mut()) {
            use std::io::Write;
            let _ = w.write_all(csv_row(f as usize, &self.e.neutral_row()).as_bytes());
        }
        let past_end = unsafe { crate::r_in() }.frame > self.last_frame + 600;
        if !alive || past_end {
            log(&format!("the replay ended at engine tick {} (replay frame {}){}", self.e.frames, unsafe { crate::r_in() }.frame, if past_end { ": past its last frame" } else { "" }));
            if let Some(w) = self.hash_out.as_mut() {
                use std::io::Write;
                let _ = w.flush();
            }
            self.end_age = Some(0);
            return false;
        }
        true
    }
}

impl Game for SlpPlay {
    fn tick(&mut self, pads: &mut PadSystem) {
        let mut st = [pad_disconnected(); PAD_CHANMAX];
        pads.read(&mut st); // the replay's own inputs drive the match; the pads are read only to keep the queue drained
        crate::hang::beat();
        self.tick_no += 1;
        // the window's keys always go to the replay controls (no UI here holds them)
        match pads.ui_keys.as_mut() {
            Some(q) => {
                let keys = std::mem::take(q);
                self.ctl.keys(&keys);
            }
            None => pads.ui_keys = Some(Vec::with_capacity(64)),
        }
        self.ctl.on_tick(self.tick_no);
        if let Some(t) = self.end_age.as_mut() {
            *t += 1;
            if *t >= 120 && !self.stay {
                self.quit = true;
            }
        } else {
            if let Some(s) = self.ctl.seek.take() {
                use crate::replay_ctl::Seek;
                // forward only: a .slp playback keeps no snapshots; the skipped ticks run unpaced (and are hashed)
                let now = self.e.frames as usize;
                let target = match s {
                    Seek::Tick(t) => Some(t),
                    Seek::MatchFrame(m) => Some(m as usize + 124),
                    Seek::Rel(d) if d > 0 => Some(now + d as usize),
                    _ => None,
                };
                match target.filter(|&t| t > now) {
                    Some(t) => {
                        log(&format!("seek forward to engine tick {t}"));
                        while (self.e.frames as usize) < t && self.step() {}
                    }
                    None => log("seek back: a Slippi replay plays forward only"),
                }
            }
            self.ctl.apply_camera();
            for _ in 0..self.ctl.steps() {
                if !self.step() {
                    break;
                }
            }
        }
        if let Some(a) = self.audio.as_mut() {
            a.tick();
        }
    }

    fn set_interp(&mut self, on: bool) {
        self.interp = on;
        melee_runtime::pose_capture::ON.store(on, std::sync::atomic::Ordering::Relaxed);
    }

    fn present(&mut self, window: &Arc<Window>, alpha: f64) {
        if self.presenter.is_none() {
            match Presenter::new(window.clone()) {
                Ok(mut p) => {
                    melee_engine::gxbridge::shim().gx.set_pipe_factory(p.renderer.pipe_factory());
                    p.offthread_drop = Some(|q| !melee_engine::arena::contains(q));
                    self.presenter = Some(p)
                }
                Err(e) => {
                    self.failed = Some(e);
                    self.quit = true;
                    return;
                }
            }
        }
        let frame = self.frame.take();
        let packed = self.packed.take();
        let p = self.presenter.as_mut().unwrap();
        p.set_true_widescreen(melee_engine::view::renderer_widen());
        p.set_content_aspect(melee_engine::view::aspect());
        let pose = if frame.is_some() { melee_engine::gxbridge::shim().gx.pose.take() } else { None };
        p.set_match_scene(true);
        p.present_posed(&melee_engine::gxbridge::shim().gx.state, frame, pose, packed, self.interp.then_some(alpha));
        if let Some(e) = p.renderer.gpu_fault() {
            self.failed = Some(format!("GPU error, stopping: {e}"));
            self.quit = true;
        }
    }

    fn resized(&mut self, w: u32, h: u32) {
        melee_engine::view::set_window(w, h);
        if let Some(p) = self.presenter.as_mut() {
            p.resize(w, h);
        }
    }

    fn should_quit(&self) -> bool {
        self.quit
    }

    fn display_stats(&mut self) -> Option<(u64, u64, f64)> {
        self.presenter.as_ref().and_then(|p| p.frame_stats())
    }

    fn set_hud(&mut self, lines: Option<Vec<melee_platform::overlay::Line>>) {
        if let Some(p) = self.presenter.as_mut() {
            p.hud = lines;
        }
    }
}

/// `melee play --replay FILE.slp [--hash-out CSV] [--replay-stay] [--replay-paused | --replay-speed X | --replay-ctl ..]
/// [--no-audio] [window options]`.
pub fn play(args: &[String], path: &str) -> ExitCode {
    let get = |k: &str| args.iter().position(|a| a == k).and_then(|i| args.get(i + 1)).cloned();
    let has = |k: &str| args.iter().any(|a| a == k);
    match crate::display::setup(args) {
        Ok(d) => crate::profile::start(&d, args),
        Err(e) => {
            eprintln!("play: {e}");
            return ExitCode::from(2);
        }
    }
    let Ok(iso) = crate::iso_path(args) else {
        eprintln!("play: --replay {path}: no disc image (--iso or GW_ISO_VANILLA)");
        return ExitCode::from(2);
    };
    crate::crash::note_disc(&iso);
    // golden's setup, in golden's order: the mods, the sound, the replay (codes, seed, scene), the boot, the player
    if let Err(er) = crate::set_engine_mods(args) {
        eprintln!("play: mods: {er}");
        return ExitCode::FAILURE;
    }
    let audio = if !has("--no-audio") && crate::cfg::flag("audio", true) {
        match crate::audio::Audio::boot(&iso, false) {
            Ok(a) => {
                crate::engine_audio::attach(false);
                // the stage BGM pick below makes the RNG draw that golden's pre-draw stands for (golden --audio)
                // SAFETY: single-threaded start-up, before the engine reads it.
                unsafe { std::env::set_var("MELEE_ENGINE_PRE_DRAWS", "0") };
                Some(a)
            }
            Err(e) => {
                log(&format!("audio off: {e}"));
                None
            }
        }
    } else {
        None
    };
    let (pl, scn, seed) = match crate::slp_open(path, None) {
        Ok(x) => x,
        Err(e) => {
            eprintln!("play: --replay: {e}");
            return ExitCode::from(2);
        }
    };
    let last_frame = pl.r.last;
    // SAFETY: single-threaded start-up, before the engine reads them.
    unsafe {
        std::env::set_var("MELEE_TEST_SEED", seed.to_string());
        if std::env::var_os("MELEE_ENGINE_PRE_DRAWS").is_none() {
            std::env::set_var("MELEE_ENGINE_PRE_DRAWS", "1");
        }
        melee_engine::engine::RENDER = true;
    }
    if melee_engine::view::sim_view().is_none() {
        // golden's fixed camera scale for a replay without a sim view (see the module doc)
        melee_engine::view::set_sim_view((1920.0f32 / 1080.0f32).to_bits());
    }
    crate::replay_modes_active(pl.slippi_version());
    let booted = crate::engine_disc(&iso).and_then(|disc| melee_engine::Engine::boot_vs(disc, 64 << 20, PadSystem::headless(), scn));
    crate::install_replay(pl);
    let e = match booted {
        Ok(e) => e,
        Err(er) => {
            eprintln!("play: --replay {path}: {er}");
            return ExitCode::FAILURE;
        }
    };
    if audio.is_some() {
        crate::engine_audio::start_stage_bgm();
    }
    let hash_out = match get("--hash-out") {
        Some(p) => match std::fs::File::create(&p) {
            Ok(f) => {
                let mut w = std::io::BufWriter::new(f);
                use std::io::Write;
                let _ = w.write_all(CSV_HEADER.as_bytes());
                Some(w)
            }
            Err(er) => {
                eprintln!("play: --hash-out {p}: {er}");
                return ExitCode::from(2);
            }
        },
        None => None,
    };
    let ctl = crate::replay_ctl::Ctl::from_args(args).unwrap_or_else(crate::replay_ctl::Ctl::new);
    log(&format!("{} frames, controls: {}", last_frame + 124, ctl.state()));
    let game = SlpPlay {
        e,
        presenter: None,
        frame: None,
        packed: None,
        audio,
        ctl,
        tick_no: 0,
        end_age: None,
        stay: has("--replay-stay"),
        last_frame,
        hash_out,
        interp: false,
        quit: false,
        failed: None,
    };
    let title = std::env::var("MELEE_WINDOW_TITLE").unwrap_or_else(|_| format!("{} replay", crate::play::window_title(None)));
    let scripted = std::env::var_os("MELEE_PAD_SCRIPT").is_some_and(|v| !v.is_empty());
    let cfg = crate::display::current().window_config(title, !scripted);
    let mut pads = PadSystem::interactive();
    if scripted {
        pads.set_ignore_adapter(true);
        pads.gamepads = melee_platform::pad_read::GamepadSource::Fixed(Default::default());
    }
    match run_windowed(game, pads, cfg) {
        Ok((ticks, _)) => {
            log(&format!("closed after {ticks} ticks"));
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("play: event loop: {e}");
            ExitCode::FAILURE
        }
    }
}
