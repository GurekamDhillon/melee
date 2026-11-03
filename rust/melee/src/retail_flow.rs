//! `melee play` on the retail scenes: the game's own mode loop (`gm_801A4510` -> `runGameMode` -> `gm_801A4014`,
//! `Engine::boot_modes` / `Engine::tick_modes`) runs from the boot on, so the opening movie, the title, the main menu,
//! the VS character select (`mnCharSel`), the stage select (`mnStageSel`), the match, the results (`gmResult`) and the
//! rules / name entry screens are the retail scenes, in the retail order. `MELEE_STANDIN_MENUS=1` keeps the older
//! stand-in flow (play.rs).
//!
//! Presets: `--stocks N` / `--time-secs N` / `--items F` are written into the saved VS rules (`GameRules` /
//! `GamePrefs`) the rules menu edits; `--match a,b/cpuN,..[,STAGE]` seeds the VS mode the way the C build's scene
//! launcher does (`SceneLaunch_SeedVs`: the first match starts at once), and after its results the retail CSS follows.
use melee_platform::gfx::{write_png, Presenter};
use melee_platform::pad::*;
use melee_platform::{Game, PadSystem};
use std::path::PathBuf;
use std::sync::Arc;
use winit::window::Window;

fn log(msg: &str) {
    eprintln!("play: {msg}");
}

pub fn scene_name(kind: u8) -> &'static str {
    match kind as u32 {
        0 => "title",
        1 => "menu",
        2 => "vs",
        3 => "suddendeath",
        5 => "results",
        8 => "css",
        9 => "sss",
        28 => "opening",
        40 => "progscan",
        42 => "memcard",
        _ => "scene",
    }
}

pub struct Presets {
    pub stocks: Option<i32>,
    pub time_secs: Option<i32>,
    /// 0 off, 1..5 very low..very high
    pub items: Option<i32>,
    pub scenario: Option<melee_engine::scenario::Scenario>,
}

pub struct RetailPlay {
    iso: String,
    mods_dir: Option<String>,
    presets: Presets,
    e: Option<melee_engine::Engine>,
    frame: Option<Vec<melee_gx::draw::DrawCmd>>,
    /// The decode thread's packed vertex/index data for `frame` (async GX, see `boot`).
    packed: Option<melee_gx::render::Packed>,
    presenter: Option<Presenter>,
    headless: bool,
    tick_no: u64,
    age: u32,
    /// (mode, scene kind) of the current scene
    cur: (u8, u8),
    /// scenes entered so far (the shot index)
    scene_no: u32,
    matches: u32,
    quit_after: Option<u32>,
    max_frames: Option<u64>,
    shot_dir: Option<PathBuf>,
    shot_ages: Vec<u32>,
    shot_pending: Option<String>,
    quit: bool,
    pub failed: Option<String>,
    interp: bool,
    pub skip_intro: bool,
    /// Sound on (no --no-audio, audio.cfg on): melee_audio's sim, as the match (port/sfxparity), the menus' calls routed by name.
    pub audio_on: bool,
    audio: Option<crate::audio::Audio>,
}

impl RetailPlay {
    pub fn new(iso: String, mods_dir: Option<String>, presets: Presets, headless: bool, shot_dir: Option<PathBuf>, shot_ages: Vec<u32>, quit_after: Option<u32>, max_frames: Option<u64>) -> RetailPlay {
        RetailPlay {
            iso, mods_dir, presets, e: None, frame: None, packed: None, presenter: None, headless, tick_no: 0, age: 0, cur: (0xFF, 0xFF), scene_no: 0,
            matches: 0, quit_after, max_frames, shot_dir,
            shot_ages: if shot_ages.is_empty() { vec![90] } else { shot_ages },
            shot_pending: None, quit: false, failed: None, interp: false, skip_intro: false, audio_on: false, audio: None,
        }
    }

    fn boot(&mut self) -> Result<melee_engine::Engine, String> {
        let margs: Vec<String> = self.mods_dir.iter().flat_map(|d| ["--mods".to_string(), d.clone()]).collect();
        crate::set_engine_mods(&margs)?;
        // HSD_SisLib_FontAtlas (main.dol) is loaded by Engine::boot_host.
        let disc = crate::engine_disc(&self.iso)?;
        unsafe {
            melee_engine::engine::RENDER = !self.headless;
        }
        if self.audio_on {
            match crate::audio::Audio::boot(&self.iso, self.headless) {
                Ok(au) => {
                    // the TU hooks (post-link) and the scene's per-frame update, before the engine links
                    crate::engine_audio::attach(false);
                    unsafe { melee_modes::tu::ext_hook::AUDIO_ROUTE = Some(crate::engine_audio::route_by_name) };
                    self.audio = Some(au);
                }
                Err(e) => log(&format!("audio off: {e}")),
            }
        }
        // The boot mode (gmboot.c) is the memory-card scene, which the port skips with no card (the C build's unattended
        // runs: SceneLaunch_SkipMemcard); its bootOnLoad then picks the mode: the opening movie, or the title when the
        // intro is skipped. MELEE_RETAIL_FIRST=<GameModeKind> starts elsewhere.
        let boot_pick = if self.skip_intro { melee_types::GM_TITLE } else { melee_types::GM_OPENING_MV } as u8;
        let mut first = std::env::var("MELEE_RETAIL_FIRST").ok().and_then(|v| v.parse::<u8>().ok()).unwrap_or(boot_pick);
        if let Some(scn) = self.presets.scenario.take() {
            melee_engine::scenario::install(scn);
            first = melee_types::GM_VS as u8;
        }
        let e = melee_engine::Engine::boot_modes(disc, 64 << 20, PadSystem::headless(), first)?;
        if self.audio.is_some() {
            // the menu TUs' sound hooks, over the seams' silent defaults (which the link above installs)
            crate::engine_audio_gen::install_menus();
        }
        self.apply_rules();
        // GX decode on its own thread, as play.rs does for its engine match (port/memfix): the windowed retail flow
        // never snapshots the arena (play turns trim on), so the thread's frames may hold its blocks. Without this the
        // FIFO decode and draw packing ran inside every tick (Temple 4P: tick p50 5.7 ms instead of ~2).
        // MELEE_GX_SYNC=1 keeps decoding in the tick; headless runs (golden, gate) never decode on a thread.
        if !self.headless && melee_engine::arena::trim() && std::env::var_os("MELEE_GX_SYNC").is_none() {
            melee_engine::gxbridge::shim().gx.enable_async(melee_engine::arena::mark_foreign_thread);
            // screenshots show exactly their tick: take each frame when it is decoded
            melee_engine::gxbridge::shim().wait_frame = self.shot_dir.is_some() && std::env::var_os("MELEE_SHOT_NOWAIT").is_none();
        }
        Ok(e)
    }

    /// The presets into the saved VS rules (what the rules menu edits and `gm_InitVsMode` reads).
    fn apply_rules(&self) {
        let p = &self.presets;
        unsafe {
            let gr = melee_modes::tu::gmmain_lib::gmMainLib_GetGameRules();
            let gp = melee_modes::tu::gmmain_lib::gmMainLib_GetGamePrefs();
            if let Some(n) = p.stocks {
                (*gr).mode = 1;
                (*gr).stock_count = n.clamp(1, 99) as u8;
            }
            if let Some(s) = p.time_secs {
                if p.stocks.is_none() {
                    (*gr).mode = 0;
                }
                let minutes = ((s.max(1) + 59) / 60) as u8;
                (*gr).time_limit = minutes;
                (*gr).stock_time_limit = minutes;
            }
            if let Some(i) = p.items {
                (*gp).item_freq = if i == 0 { 0xFF } else { (i - 1) as u8 };
            }
        }
    }

    fn emit_shot(&mut self) {
        let (Some(name), Some(dir), Some(p)) = (self.shot_pending.take(), self.shot_dir.as_ref(), self.presenter.as_mut()) else { return };
        let px = p.screenshot();
        let path = dir.join(format!("retail_{name}.png"));
        match write_png(&path, p.renderer.width, p.renderer.height, &px) {
            Ok(()) => log(&format!("screenshot {}", path.display())),
            Err(e) => log(&format!("screenshot: {e}")),
        }
    }
}

impl Game for RetailPlay {
    fn tick(&mut self, pads: &mut PadSystem) {
        if self.quit {
            return;
        }
        if self.e.is_none() {
            match self.boot() {
                Ok(e) => self.e = Some(e),
                Err(err) => {
                    log(&format!("retail boot failed: {err}"));
                    self.failed = Some(err);
                    self.quit = true;
                    return;
                }
            }
        }
        let mut st = [pad_disconnected(); PAD_CHANMAX];
        pads.read(&mut st);
        let mut four = [melee_types::PADStatus::zeroed(); 4];
        four.copy_from_slice(&st[..4]);
        let e = self.e.as_mut().unwrap();
        e.set_pads(Some(four));
        crate::prof::gate(true);
        let entered = e.tick_modes();
        crate::prof::gate(false);
        if let Some(au) = self.audio.as_mut() {
            au.tick();
        }
        if let Some(f) = melee_engine::gxbridge::take_frame() {
            self.frame = Some(f);
            self.packed = melee_engine::gxbridge::shim().gx.packed.take();
        }
        // MELEE_RETAIL_CSSDBG=N: every N ticks on the CSS, port 1's cursor (world units) and the four CPU levels
        if let Some(n) = std::env::var("MELEE_RETAIL_CSSDBG").ok().and_then(|v| v.parse::<u64>().ok()) {
            if self.cur.1 == melee_types::GS_CSS as u8 && n > 0 && self.tick_no % n == 0 {
                unsafe {
                    let c = melee_modes::tu::mncharsel::mnCharSel_804A0BC0[0];
                    let d = melee_modes::tu::mncharsel::mnCharSel_804D6CB0;
                    if !c.is_null() && !d.is_null() {
                        let lv: Vec<String> = (0..4).map(|i| format!("{}/{}/{}", (*d).vs.start.players[i].slot_type, (*d).vs.start.players[i].ckind, (*d).vs.start.players[i].cpu_level)).collect();
                        let mut knobs = Vec::new();
                        for i in 0..4 {
                            let mut j: *mut melee_types::HSD_JObj = core::ptr::null_mut();
                            let jt = melee_modes::tu::mncharsel::mnCharSel_803F0DFC.doors[i].cpuslider_joint as i32;
                            melee_runtime::tu::lbspdisplay::lb_80011E24(melee_modes::tu::mncharsel::mnCharSel_804D6CC0, &mut j, &[jt, -1]);
                            if !j.is_null() {
                                let mut p = melee_types::Vec3 { x: 0.0, y: 0.0, z: 0.0 };
                                melee_runtime::tu::lb_00B0::lb_8000B1CC(j as _, core::ptr::null_mut(), &mut p as *mut _ as _);
                                knobs.push(format!("{:.2},{:.2}", -2.9 + p.x, 1.7 + p.y));
                            }
                        }
                        log(&format!("css knobs {}", knobs.join(" ")));
                        log(&format!("css age {} cursor {:.2},{:.2} x5 {} x6 {} slots {}", self.age, (*c).xC, (*c).x10, (*c).x5, (*c).x6, lv.join(" ")));
                    }
                }
            }
        }
        // a once-a-second heartbeat (tick, scene, draw commands of the last frame): a stall shows as a stopped tick,
        // a render-only fault as a running tick with no draws
        if self.tick_no % 60 == 0 {
            log(&format!("heartbeat tick {} scene {} age {} draws {}", self.tick_no, scene_name(self.cur.1), self.age, self.frame.as_ref().map_or(0, |f| f.len())));
            if std::env::var_os("MELEE_RETAIL_DRAWDBG").is_some() {
                if let Some(f) = self.frame.as_ref() {
                    for (i, d) in f.iter().take(6).enumerate() {
                        log(&format!("  draw {i}: prim {:#x} verts {} vp {:?} sc {:?} pn {}", d.prim, d.vtx_count, d.render_viewport, d.render_scissor, d.current_pn_mtx));
                    }
                }
            }
        }
        self.tick_no += 1;
        self.age += 1;
        let now = e.mode_scene();
        if entered || now != self.cur {
            let prev = self.cur;
            self.cur = now;
            self.age = 0;
            self.scene_no += 1;
            log(&format!("tick {}: scene {} (mode {}, scene kind {})", self.tick_no, scene_name(now.1), now.0, now.1));
            if prev.1 == melee_types::GS_RESULTS as u8 {
                self.matches += 1;
                if self.quit_after.is_some_and(|n| self.matches >= n) {
                    log(&format!("quit after {} matches", self.matches));
                    self.quit = true;
                }
            }
        }
        if self.max_frames.is_some_and(|n| self.tick_no >= n) {
            log(&format!("max frames reached in {}", scene_name(self.cur.1)));
            self.quit = true;
        }
        // MELEE_RETAIL_SHOT_TICKS="a-b,c": timeline shots (tick numbers from the first tick), for the transition captures
        if self.shot_dir.is_some() {
            if let Ok(v) = std::env::var("MELEE_RETAIL_SHOT_TICKS") {
                let t = self.tick_no as u64;
                let hit = v.split(',').any(|r| {
                    let mut it = r.split('-').map(|x| x.trim().parse::<u64>().unwrap_or(u64::MAX));
                    let a = it.next().unwrap_or(u64::MAX);
                    let b = it.next().unwrap_or(a);
                    t >= a && t <= b
                });
                if hit {
                    self.shot_pending = Some(format!("tl_{:05}", t));
                }
            }
        }
        if self.shot_dir.is_some() && self.shot_ages.contains(&self.age) {
            self.shot_pending = Some(format!("{:02}_{}_{:04}", self.scene_no, scene_name(self.cur.1), self.age));
        }
    }

    fn set_interp(&mut self, on: bool) {
        self.interp = on;
    }

    fn present(&mut self, window: &Arc<Window>, alpha: f64) {
        if self.presenter.is_none() {
            match Presenter::new(window.clone()) {
                Ok(p) => self.presenter = Some(p),
                Err(e) => {
                    self.failed = Some(e);
                    self.quit = true;
                    return;
                }
            }
        }
        let a = (self.interp && self.shot_pending.is_none()).then_some(alpha);
        let frame = self.frame.take();
        let p = self.presenter.as_mut().unwrap();
        if let Some(pk) = self.packed.take() {
            p.renderer.set_packed(pk);
        }
        p.present_frames(&melee_engine::gxbridge::shim().gx.state, frame, a);
        self.emit_shot();
    }

    fn resized(&mut self, w: u32, h: u32) {
        if let Some(p) = self.presenter.as_mut() {
            p.resize(w, h);
        }
    }

    fn should_quit(&self) -> bool {
        self.quit
    }

    fn present_now(&self) -> bool {
        self.shot_pending.is_some()
    }
}

pub fn run(mut rp: RetailPlay, headless: bool) -> std::process::ExitCode {
    use std::process::ExitCode;
    if headless {
        let mut pads = PadSystem::headless();
        let limit = rp.max_frames.unwrap_or(60 * 600);
        let ticks = melee_platform::run_headless(&mut rp, &mut pads, limit);
        log(&format!("headless: {ticks} ticks, {}", scene_name(rp.cur.1)));
        return if rp.failed.is_some() { ExitCode::FAILURE } else { ExitCode::SUCCESS };
    }
    let scripted = std::env::var_os("MELEE_PAD_SCRIPT").is_some_and(|v| !v.is_empty());
    // play.rs window_title: MELEE_WINDOW_TITLE, else the build id ("melee [where hash]") and "retail menus"
    let title = std::env::var("MELEE_WINDOW_TITLE").unwrap_or_else(|_| format!("{} retail menus", crate::play::window_title(None)));
    let cfg = melee_platform::WindowConfig { title, width: 1280, height: 960, active: !scripted };
    if scripted {
        std::env::set_var("MELEE_PAD_IGNORE_ADAPTER", "1");
    }
    let mut pads = PadSystem::interactive();
    if scripted {
        pads.set_ignore_adapter(true);
        pads.gamepads = melee_platform::pad_read::GamepadSource::Fixed(Default::default());
    }
    if !scripted && std::env::var("MELEE_KEYBOARD").map_or(true, |v| v != "0") {
        pads.keyboard[0] = melee_platform::aurora::KeyboardBindings::preset_default();
    }
    match melee_platform::run_windowed(rp, pads, cfg) {
        Ok((ticks, g)) => {
            log(&format!("closed after {ticks} ticks"));
            let _ = g;
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("play: event loop: {e}");
            ExitCode::FAILURE
        }
    }
}
