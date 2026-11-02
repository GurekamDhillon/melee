//! `melee play`: the live game. A window at 60 Hz, real input (keyboard, GameCube adapter,
//! gamepads, or a `MELEE_PAD_SCRIPT`), and the scene flow of the C game's mode loop
//! (`gm_801A4510` -> `runGameMode`): opening -> title -> main menu -> VS menu -> character select
//! -> stage select -> loading -> live VS match (pause) -> results -> back to the character select.
//!
//! What is real: the pad layer (melee_platform's `PadSystem::read` = `gw_PADRead`), the title's
//! timings and exits (`gmTitle` / `gmTitleMode`), the pause trigger (`gm_DefaultVSGetPauser`:
//! Start on a player's pad), and the match itself (`vs_match::VsMatch`, stepped once per tick with
//! the live pads as `HSD_PadRenewStatus`'s input), drawn through the ported HSD draw path.
//!
//! What is a stand-in: the screens other than the match are drawn by `ui.rs` (rectangles and a
//! bitmap font), not by the retail menu models (MnTitle / MnMain / MnCharSel DAT scenes are not
//! ported) and not by the frontend kit; the opening movie (THP) is a card. Match rules are fixed
//! (a time match of `--time-secs`, the rules the golden sim runs under). The match ends when the time is up or when
//! the simulation reaches code that is not ported yet (`VsMatch::stopped`), reported on the
//! results screen. The camera is an approximation (cm/camera.c is not ported), no HUD models.
//!
//! ```text
//! melee play [--iso DISC.iso] [--time-secs N] [--shot-dir DIR] [--shots] [--quit-at-results]
//!            [--max-frames N] [--skip-intro] [--mods DIR] [--golden-pads]
//! ```
//! `--shots` writes a PNG per screen (30 ticks after entering it) into `--shot-dir`.
use crate::pads::RawPad;
use crate::ui::{self, Ui};
use melee_gx::api::GxColor;
use melee_gx::shim::GxShim;
use melee_platform::gfx::{write_png, Presenter, EFB_HEIGHT, EFB_WIDTH};
use melee_platform::pad::*;
use melee_platform::{run_headless, run_windowed, Game, PadSystem, WindowConfig};
use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::Arc;
use winit::window::Window;

const HSD_PAD_A: u32 = 0x100;
const HSD_PAD_B: u32 = 0x200;
const HSD_PAD_START: u32 = 0x1000;
const HSD_PAD_UP: u32 = 0x8;
const HSD_PAD_DOWN: u32 = 0x4;
const HSD_PAD_LEFT: u32 = 0x1;
const HSD_PAD_RIGHT: u32 = 0x2;
const HSD_PAD_L: u32 = 0x40;
const HSD_PAD_R: u32 = 0x20;

/// Fighters the native sim can load (name, display).
const ROSTER: &[(&str, &str)] = &[
    ("falcon", "C. FALCON"),
    ("fox", "FOX"),
    ("falco", "FALCO"),
    ("marth", "MARTH"),
    ("sheik", "SHEIK"),
    ("peach", "PEACH"),
    ("samus", "SAMUS"),
];
/// Stages the sim can load (file, display).
const STAGES: &[(&str, &str)] = &[
    ("GrNLa.dat", "FINAL DESTINATION"),
    ("GrNBa.dat", "BATTLEFIELD"),
    ("GrGr.dat", "GREEN GREENS"),
    ("GrSt.dat", "YOSHI'S STORY"),
];
const MAIN_ITEMS: &[(&str, bool)] = &[("1P MODE", false), ("VS MODE", true), ("TROPHIES", false), ("OPTIONS", false), ("DATA", false)];
const VS_ITEMS: &[(&str, bool)] = &[("MELEE", true), ("SPECIAL MELEE", false), ("CUSTOM RULES", false), ("NAMES", false)];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Scene {
    Opening,
    Title,
    MainMenu,
    VsMenu,
    Css,
    Sss,
    Loading,
    Match,
    Results,
}

impl Scene {
    fn name(self) -> &'static str {
        match self {
            Scene::Opening => "opening",
            Scene::Title => "title",
            Scene::MainMenu => "mainmenu",
            Scene::VsMenu => "vsmenu",
            Scene::Css => "css",
            Scene::Sss => "sss",
            Scene::Loading => "loading",
            Scene::Match => "match",
            Scene::Results => "results",
        }
    }
}

/// One controller port as the menus see it: this tick's sample, the buttons newly down
/// (`HSD_PadStatus.trigger`) and the stick as a repeating direction.
#[derive(Clone, Copy, Default)]
struct PortIn {
    connected: bool,
    raw: RawPad,
    held: u32,
    trig: u32,
    nav_x: i32,
    nav_y: i32,
    prev_x: i32,
    prev_y: i32,
    rep_x: u32,
    rep_y: u32,
}

/// Menu stick repeat: the first step at once, then after 20 ticks every 5.
fn repeat_step(dir: i32, prev: &mut i32, rep: &mut u32) -> i32 {
    if dir == 0 {
        *prev = 0;
        *rep = 0;
        return 0;
    }
    let fire = if dir != *prev {
        *rep = 0;
        true
    } else {
        *rep += 1;
        *rep >= 20 && (*rep - 20) % 5 == 0
    };
    *prev = dir;
    if fire { dir } else { 0 }
}

impl PortIn {
    fn update(&mut self, st: &PADStatus) {
        self.connected = st.err == 0;
        let mut raw = RawPad::default();
        if self.connected {
            raw = RawPad {
                button: st.button as u32,
                stick_x: st.stickX,
                stick_y: st.stickY,
                sub_x: st.substickX,
                sub_y: st.substickY,
                analog_l: st.triggerLeft,
                analog_r: st.triggerRight,
            };
        }
        self.trig = raw.button & !self.held;
        self.held = raw.button;
        self.raw = raw;
        let dir = |v: i8| if v >= 40 { 1 } else if v <= -40 { -1 } else { 0 };
        self.nav_x = repeat_step(dir(raw.stick_x), &mut self.prev_x, &mut self.rep_x);
        self.nav_y = repeat_step(dir(raw.stick_y), &mut self.prev_y, &mut self.rep_y);
        if self.trig & HSD_PAD_LEFT != 0 { self.nav_x = -1 }
        if self.trig & HSD_PAD_RIGHT != 0 { self.nav_x = 1 }
        if self.trig & HSD_PAD_UP != 0 { self.nav_y = 1 }
        if self.trig & HSD_PAD_DOWN != 0 { self.nav_y = -1 }
    }
}

struct MatchSetup {
    fighters: [usize; 2],
    stage: usize,
}

/// A match on the consolidated engine (`melee_engine`): the game's own scene loop, camera, HUD and pause, drawn through
/// the GX bridge. One per process: the engine boots its host and the TU statics once (consolidation.md step 18).
struct EngMatch {
    e: melee_engine::Engine,
    frame: Option<Vec<melee_gx::draw::DrawCmd>>,
    names: [&'static str; 2],
    end_timer: Option<u32>,
    timed_out: bool,
    quit_hold: u32,
}

/// The tick the HUD's GO appears on (the legacy sim's `go_frame` on every stage in the roster).
const ENGINE_GO_FRAME: i32 = 84;


#[derive(Default, Clone)]
struct Outcome {
    names: [String; 2],
    falls: [i32; 2],
    percent: [f32; 2],
    frames: i32,
    winner: Option<usize>,
    why: String,
}

pub struct Play {
    iso: String,
    mods_dir: Option<String>,
    /// Match length in seconds (the sim is a time match: the golden's rules, `match_kind` 0).
    time_secs: i32,
    /// `--golden-pads`: the match reads the golden harness's pad cycles (pad_fixed.lua) instead of
    /// the live pads, so a scripted run goes through exactly the states the oracle verified.
    golden_pads: bool,
    scene: Scene,
    age: u32,
    tick_no: u64,
    port: [PortIn; 4],
    title_count: u32,
    main_sel: usize,
    vs_sel: usize,
    toast: Option<(String, u32)>,
    cursor: [usize; 2],
    pick: [Option<usize>; 2],
    sss_sel: usize,
    setup: Option<MatchSetup>,
    /// The match (the consolidated engine).
    em: Option<EngMatch>,
    engine_used: bool,
    headless: bool,
    outcome: Outcome,
    matches_played: u32,
    shim: GxShim,
    ui: Ui,
    presenter: Option<Presenter>,
    shot_dir: Option<PathBuf>,
    shots: bool,
    shot_pending: Option<String>,
    pause_age: u32,
    quit_at_results: bool,
    max_frames: Option<u64>,
    skip_intro: bool,
    quit: bool,
    pub failed: Option<String>,
    audio: Option<crate::audio::Audio>,
    /// The retail title scene (retail.rs), alive while `scene == Title`.
    title: Option<crate::retail::Title>,
    menu: Option<crate::retail::mainmenu::MainMenu>,
    /// `--shot-ages a,b,c`: a screenshot at those scene ages of every scene (`play_<scene>_<age>.png`).
    shot_ages: Vec<u32>,
    /// `--shot-ticks a,b`: screenshots at those timeline ticks (tick_no - 1: the title's age under --skip-intro), named `tl_<n>`.
    shot_ticks: Vec<u64>,
    kit: Box<dyn crate::retail::MenuHook>,
    /// Render-only interpolation of the match frames (set by the platform loop).
    interp: bool,
    /// A tick ran since the last present (the stand-in / menu scenes redraw only then).
    dirty: bool,
}

fn log(msg: &str) {
    eprintln!("play: {msg}");
}

impl Play {
    fn goto(&mut self, s: Scene) {
        log(&format!("tick {}: {} -> {}{}", self.tick_no, self.scene.name(), s.name(), self.audio.as_ref().map_or(String::new(), |a| format!(" (audio underruns {})", a.underruns()))));
        self.audio_enter(s);
        self.scene = s;
        self.age = 0;
        self.title = None;
        self.menu = None;
        if s == Scene::MainMenu || s == Scene::VsMenu {
            let (kind, hov) = if s == Scene::VsMenu { (crate::retail::mainmenu::KIND_VS, 0) } else { (crate::retail::mainmenu::KIND_MAIN, self.main_sel as u16) };
            match self.load_menu(kind, hov) {
                Ok(m) => self.menu = Some(m),
                Err(e) => {
                    log(&format!("main menu: {e}"));
                    self.failed = Some(e);
                    self.quit = true;
                }
            }
        }
        if s == Scene::Title {
            match self.load_title() {
                Ok(t) => self.title = Some(t),
                Err(e) => {
                    log(&format!("title scene: {e}"));
                    self.failed = Some(e);
                    self.quit = true;
                }
            }
        }
    }

    /// `mnMain_Scene_OnEnter`: MnMaAll.usd (US) or .dat from the disc.
    fn load_menu(&self, kind: u8, hovered: u16) -> Result<crate::retail::mainmenu::MainMenu, String> {
        let mut disc = crate::disc::Disc::open(&self.iso)?;
        let bytes = disc.read("MnMaAll.usd").or_else(|_| disc.read("MnMaAll.dat"))?;
        let sis = disc.read("SdMenu.usd").or_else(|_| disc.read("SdMenu.dat")).ok();
        let atlas = disc.dol_bytes(crate::retail::sis::ATLAS_ADDR, crate::retail::sis::ATLAS_BYTES)?;
        crate::retail::mainmenu::MainMenu::new(&bytes, sis.as_deref().map(|s| (s, atlas)), kind, hovered, true)
    }

    /// `gm_Scene_Title_OnEnter`: GmTtAll.usd (US) or .dat from the disc.
    fn load_title(&self) -> Result<crate::retail::Title, String> {
        let mut disc = crate::disc::Disc::open(&self.iso)?;
        let bytes = disc.read("GmTtAll.usd").or_else(|_| disc.read("GmTtAll.dat"))?;
        crate::retail::Title::new(&bytes, true)
    }

    /// The scene's audio start (see audio.rs), before `self.scene` changes.
    fn audio_enter(&mut self, s: Scene) {
        let Some(a) = self.audio.as_ref() else { return };
        let from = self.scene;
        a.scene_start();
        match s {
            Scene::Opening => a.stop_music(),
            Scene::Title => a.title(),
            Scene::MainMenu | Scene::VsMenu => a.menu_music(),
            Scene::Css => {
                if from != Scene::Sss {
                    a.css();
                } else {
                    a.menu_music();
                }
            }
            Scene::Sss => a.menu_music(),
            Scene::Loading => {}
            Scene::Match => {
                if let (Some(su), None) = (self.setup.as_ref(), self.em.as_ref()) {
                    let ck = [crate::audio::ckind_of(ROSTER[su.fighters[0]].0), crate::audio::ckind_of(ROSTER[su.fighters[1]].0)];
                    let (st, gr) = crate::audio::stage_kinds(STAGES[su.stage].0);
                    // the engine's own stage BGM pick is a sound seam for now: the model's pick (or track 0)
                    a.match_start(&ck, st, gr, 0);
                }
            }
            Scene::Results => {
                if from == Scene::Match {
                    a.match_leave();
                }
                let w = self.outcome.winner.map(|w| crate::audio::ckind_of(self.em.as_ref().map_or("falcon", |m| m.names[w])));
                a.results(w);
            }
        }
    }

    fn any_trig(&self) -> u32 {
        self.port.iter().filter(|p| p.connected).fold(0, |a, p| a | p.trig)
    }
    fn any_nav(&self) -> (i32, i32) {
        let mut n = (0, 0);
        for p in self.port.iter().filter(|p| p.connected) {
            if n.0 == 0 { n.0 = p.nav_x }
            if n.1 == 0 { n.1 = p.nav_y }
        }
        n
    }

    fn back_to_css(&mut self) {
        if self.scene == Scene::Match {
            if let Some(a) = &self.audio { a.match_leave() }
        }
        self.em = None;
        self.pick = [None, None];
        self.cursor = [0, 1];
        self.goto(Scene::Css);
    }

    fn tick_scene(&mut self) {
        let trig = self.any_trig();
        let (nx, ny) = self.any_nav();
        match self.scene {
            Scene::Opening => {
                // gmopening: START or A skips the movie; otherwise it plays to its end.
                if self.skip_intro || trig & (HSD_PAD_START | HSD_PAD_A) != 0 || self.age > 600 {
                    self.title_count = 0;
                    self.goto(Scene::Title);
                }
            }
            Scene::Title => {
                // gm_Scene_Title_OnFrame (retail.rs `Title`): a 20-tick lockout, then START -> menu;
                // 600 ticks -> the opening movie; Y / B (PC build) -> the port's own menu.
                use crate::retail::Exit;
                let exit = match self.title.as_mut() {
                    Some(t) => {
                        t.tick(trig);
                        t.exit
                    }
                    None => Exit::None,
                };
                match exit {
                    Exit::TitleTimeout => self.goto(Scene::Opening),
                    Exit::TitleStart(_) => {
                        // gmTitleMode onExit: START -> GM_MENU (the menu opens on 1-P Mode, gmMenuMode's default)
                        self.main_sel = 0;
                        if let Some(a) = &self.audio { a.title_start() }
                        self.goto(Scene::MainMenu);
                    }
                    Exit::TitleKit(b) => {
                        // The kit menus are the user's: a hook, not a port. Nothing plugged in -> stay.
                        if !self.kit.open_kit("title", b) {
                            if let Some(t) = self.title.as_mut() { t.exit = Exit::None }
                        }
                    }
                    Exit::None => {}
                }
            }
            Scene::MainMenu | Scene::VsMenu => {
                use crate::retail::mainmenu::Exit;
                let raws: [Option<RawPad>; 4] = std::array::from_fn(|i| self.port[i].connected.then_some(self.port[i].raw));
                let exit = match self.menu.as_mut() {
                    Some(m) => {
                        m.tick(&raws);
                        m.exit
                    }
                    None => Exit::None,
                };
                match exit {
                    Exit::Title => {
                        self.title_count = 0;
                        self.goto(Scene::Title);
                    }
                    Exit::Mode("Vs") => {
                        self.cursor = [0, 1];
                        self.pick = [None, None];
                        self.goto(Scene::Css);
                    }
                    Exit::None => {}
                    other => {
                        self.toast = Some((format!("{other:?} IS NOT IN THE NATIVE PORT YET"), 120));
                        if let Some(m) = self.menu.as_mut() { m.exit = Exit::None }
                    }
                }
            }
            Scene::Css => self.tick_css(),
            Scene::Sss => {
                if nx != 0 || ny != 0 {
                    if let Some(a) = &self.audio { a.moved() }
                }
                if nx != 0 {
                    self.sss_sel = (self.sss_sel as i32 + nx).rem_euclid(STAGES.len() as i32) as usize;
                }
                if ny != 0 {
                    self.sss_sel = (self.sss_sel + 2) % STAGES.len();
                }
                if trig & (HSD_PAD_A | HSD_PAD_START) != 0 {
                    if let Some(a) = &self.audio { a.forward() }
                    self.setup = Some(MatchSetup { fighters: [self.pick[0].unwrap_or(0), self.pick[1].unwrap_or(1)], stage: self.sss_sel });
                    self.goto(Scene::Loading);
                } else if trig & HSD_PAD_B != 0 {
                    if let Some(a) = &self.audio { a.back() }
                    self.pick = [None, None];
                    self.goto(Scene::Css);
                }
            }
            Scene::Loading => {
                // Let the card reach the screen, then build the match (disc reads + model loads).
                if self.age == 3 {
                    self.start_match();
                }
            }
            Scene::Match => self.tick_match(),
            Scene::Results => {
                if self.quit_at_results && self.age >= 90 {
                    self.quit = true;
                }
                if trig & (HSD_PAD_A | HSD_PAD_START) != 0 {
                    if let Some(a) = &self.audio { a.forward() }
                }
                if trig & HSD_PAD_A != 0 {
                    self.back_to_css();
                } else if trig & HSD_PAD_START != 0 {
                                self.goto(Scene::MainMenu);
                }
            }
        }
        if let Some((_, t)) = &mut self.toast {
            *t = t.saturating_sub(1);
            if *t == 0 {
                self.toast = None;
            }
        }
    }

    /// Character select. Slot 0 is chosen with port 1's pad; slot 1 with port 2's pad when one is
    /// plugged in, otherwise with port 1's after slot 0 has picked. A confirms, B takes back, START
    /// or A (both picked) goes on to the stage select.
    fn tick_css(&mut self) {
        let slot = if self.pick[0].is_none() { 0 } else { 1 };
        let pi = if slot == 1 && self.port[1].connected { 1 } else { 0 };
        let first = self.port[pi];
        let src = if first.connected { first } else { *self.port.iter().find(|q| q.connected).unwrap_or(&first) };
        if self.pick[1].is_none() {
            let cols = 4i32;
            let rows = (ROSTER.len() as i32 + cols - 1) / cols;
            let c = &mut self.cursor[slot];
            let (mut cx, mut cy) = ((*c as i32) % cols, (*c as i32) / cols);
            cx = (cx + src.nav_x).rem_euclid(cols);
            cy = (cy - src.nav_y).rem_euclid(rows);
            let old = *c;
            *c = ((cy * cols + cx) as usize).min(ROSTER.len() - 1);
            if *c != old {
                if let Some(a) = &self.audio { a.moved() }
            }
            if src.trig & HSD_PAD_A != 0 {
                if let Some(a) = &self.audio { a.css_pick(crate::audio::ckind_of(ROSTER[*c].0), *c) }
                self.pick[slot] = Some(*c);
                log(&format!("css: slot {} picked {}", slot + 1, ROSTER[*c].0));
                return;
            }
        } else if src.trig & (HSD_PAD_START | HSD_PAD_A) != 0 {
            if let Some(a) = &self.audio { a.forward() }
            self.sss_sel = 0;
            self.goto(Scene::Sss);
            return;
        }
        if src.trig & HSD_PAD_B != 0 {
            if let Some(a) = &self.audio { a.back() }
            if self.pick[1].is_some() {
                self.pick[1] = None;
            } else if self.pick[0].is_some() {
                self.pick[0] = None;
            } else {
                self.goto(Scene::VsMenu);
            }
        }
    }

    fn start_match(&mut self) {
        let Some(su) = self.setup.as_ref() else { return };
        let names = [ROSTER[su.fighters[0]].0, ROSTER[su.fighters[1]].0];
        let (stage_file, stage_name) = STAGES[su.stage];
        if self.engine_used {
            // The engine boots its host and the TU statics (the C globals) once per process.
            self.toast = Some(("ONE ENGINE MATCH PER RUN: RESTART, OR --engine legacy".into(), 300));
            self.goto(Scene::Sss);
            return;
        }
        log(&format!("loading {} v {} on {} on the engine ({} s time match)", names[0], names[1], stage_name, self.time_secs));
        let stage = match stage_file {
            "GrNLa.dat" => "fd",
            "GrNBa.dat" => "bf",
            "GrGr.dat" => "dl",
            "GrSt.dat" => "ys",
            _ => "fd",
        };
        let scene = format!("mode=vs;at=match;p1={}/human;p2={}/human;p3=none;p4=none;stage={stage};match=time;minutes=0;items=off", names[0], names[1]);
        // MELEE_PLAY_SCENE=<scene string>: any scenario's match instead (e.g. a 4-player CPU free-for-all for profiling).
        let scene = std::env::var("MELEE_PLAY_SCENE").unwrap_or(scene);
        let built = (|| -> Result<EngMatch, String> {
            let scn = melee_engine::scenario::Scenario::parse(&scene)?;
            let margs: Vec<String> = self.mods_dir.iter().flat_map(|d| ["--mods".to_string(), d.clone()]).collect();
            crate::set_engine_mods(&margs)?;
            let disc = crate::engine_disc(&self.iso)?;
            unsafe {
                melee_engine::engine::RENDER = !self.headless;
                std::env::set_var("MELEE_TEST_SEED", (12345 + self.matches_played).to_string());
                if std::env::var_os("MELEE_ENGINE_PRE_DRAWS").is_none() {
                    std::env::set_var("MELEE_ENGINE_PRE_DRAWS", "1");
                }
            }
            if self.audio.is_some() {
                // the game's own lbAudioAx calls (banks, announcer, entry sounds, pause) drive the sound the menus already booted
                crate::engine_audio::attach(false);
            }
            let e = melee_engine::Engine::boot_vs(disc, 64 << 20, PadSystem::headless(), scn)?;
            if self.audio.is_some() {
                crate::engine_audio::start_stage_bgm();
            }
            Ok(EngMatch { e, frame: None, names, end_timer: None, timed_out: false, quit_hold: 0 })
        })();
        match built {
            Ok(em) => {
                self.matches_played += 1;
                self.engine_used = true;
                self.em = Some(em);
                self.goto(Scene::Match);
            }
            Err(e) => {
                log(&format!("match load failed: {e}"));
                self.toast = Some((format!("LOAD FAILED: {}", e.chars().take(60).collect::<String>()), 300));
                self.goto(Scene::Sss);
            }
        }
    }


    fn tick_match(&mut self) {
        let ports = self.port;
        let Some(m) = self.em.as_mut() else { return };
        // L+R+A+START held: end the match and return to the character select (the pad layer, as in the model).
        const COMBO: u32 = HSD_PAD_L | HSD_PAD_R | HSD_PAD_A | HSD_PAD_START;
        let quit_combo = ports.iter().take(2).any(|p| p.connected && p.held & COMBO == COMBO);
        m.quit_hold = if quit_combo { m.quit_hold + 1 } else { 0 };
        if m.quit_hold >= 30 {
            log("match quit with L+R+A+START");
            self.back_to_css();
            return;
        }
        if quit_combo {
            return;
        }
        if let Some(t) = &mut m.end_timer {
            *t += 1;
            if *t >= 150 {
                self.finish_match();
            }
            return;
        }
        // The frame: the live pads in (START included: the pause is the game's own), the scene loop stepped once.
        let mut pads = [melee_types::PADStatus::zeroed(); 4];
        for (i, st) in pads.iter_mut().enumerate() {
            let raw = if self.golden_pads { crate::pads::pad_fixed(i, m.e.frames as i32) } else { ports[i].raw };
            st.button = raw.button as u16;
            st.stickX = raw.stick_x;
            st.stickY = raw.stick_y;
            st.substickX = raw.sub_x;
            st.substickY = raw.sub_y;
            st.triggerLeft = raw.analog_l;
            st.triggerRight = raw.analog_r;
            st.err = if ports[i].connected || self.golden_pads { 0 } else { -1 };
        }
        m.e.set_pads(Some(pads));
        let alive = m.e.tick();
        if let Some(f) = melee_engine::gxbridge::take_frame() {
            m.frame = Some(f);
        }
        let frame = m.e.frames as i32 - 1;
        if !alive {
            log("the engine's scene ended");
            m.end_timer = Some(0);
        } else if frame >= crate::audio::clock_end(ENGINE_GO_FRAME, self.time_secs) - 1 {
            log("time up: GameSet");
            m.timed_out = true;
            m.end_timer = Some(0);
            if let Some(a) = &self.audio { a.match_leave() }
        }
    }

    fn finish_match(&mut self) {
        let Some(m) = self.em.as_ref() else { return };
        let mut o = Outcome { frames: m.e.frames as i32, ..Default::default() };
        for i in 0..2usize {
            o.names[i] = ROSTER.iter().find(|r| r.0 == m.names[i]).map_or(m.names[i], |r| r.1).to_string();
            o.falls[i] = melee_fighter::tu::player::Player_GetFalls(i as i32);
            o.percent[i] = unsafe {
                let g = melee_fighter::player::get_entity(i as i32);
                if g.is_null() || (*g).user_data.is_null() { 0.0 } else { (*((*g).user_data as *const melee_types::Fighter)).dmg.x1830_percent }
            };
        }
        o.winner = if o.falls[0] < o.falls[1] { Some(0) } else if o.falls[1] < o.falls[0] { Some(1) } else { None };
        o.why = "TIME!".into();
        log(&format!("results: falls {:?} percent {:?} frames {} winner {:?}", o.falls, o.percent, o.frames, o.winner));
        self.outcome = o;
        self.goto(Scene::Results);
    }



    // ---------------------------------------------------------------- drawing

    fn backdrop(&mut self, top: ui::Rgba, bottom: ui::Rgba) {
        self.ui.gradient(0.0, 0.0, ui::W, ui::H, top, bottom, 48);
    }

    fn draw_ui(&mut self) {
        self.ui.clear();
        let t = self.age;
        match self.scene {
            Scene::Opening => {
                self.backdrop([8, 10, 24, 255], [0, 0, 0, 255]);
                let u = &mut self.ui;
                u.text_centered_shadow(320.0, 190.0, 3.0, "SUPER SMASH BROS.", ui::WHITE);
                u.text_centered_shadow(320.0, 225.0, 5.0, "MELEE", ui::YELLOW);
                u.text_centered(320.0, 330.0, 1.0, "OPENING MOVIE (THP) NOT DECODED IN THE NATIVE PORT", ui::DIM);
                if t / 30 % 2 == 0 {
                    u.text_centered(320.0, 400.0, 2.0, "PRESS START", ui::GREY);
                }
            }
            Scene::Title => {
                self.backdrop([20, 30, 70, 255], [90, 20, 30, 255]);
                let u = &mut self.ui;
                u.text_centered_shadow(320.0, 120.0, 3.0, "SUPER SMASH BROS.", ui::WHITE);
                u.text_centered_shadow(320.0, 160.0, 7.0, "MELEE", ui::YELLOW);
                if t >= 20 && t / 30 % 2 == 0 {
                    u.text_centered_shadow(320.0, 360.0, 2.0, "PRESS START", ui::WHITE);
                }
                u.text_centered(320.0, 440.0, 1.0, "GD'S MELEE - NATIVE RUST PORT", ui::GREY);
            }
            Scene::MainMenu => {
                self.backdrop([14, 40, 36, 255], [6, 12, 24, 255]);
                let sel = self.main_sel;
                let u = &mut self.ui;
                u.text_centered_shadow(320.0, 50.0, 3.0, "MAIN MENU", ui::WHITE);
                for (i, (name, on)) in MAIN_ITEMS.iter().enumerate() {
                    let y = 110.0 + i as f32 * 60.0;
                    let s = i == sel;
                    u.rect(150.0, y, 340.0, 48.0, if s { [40, 120, 90, 230] } else { [20, 30, 40, 200] });
                    if s { u.frame(150.0, y, 340.0, 48.0, 3.0, ui::YELLOW) }
                    u.text_centered(320.0, y + 14.0, 3.0, name, if *on { ui::WHITE } else { ui::DIM });
                }
                u.text(20.0, 452.0, 1.0, "STICK: MOVE   A: OK   B: BACK", ui::GREY);
            }
            Scene::VsMenu => {
                self.backdrop([40, 20, 50, 255], [6, 8, 22, 255]);
                let sel = self.vs_sel;
                let u = &mut self.ui;
                u.text_centered_shadow(320.0, 50.0, 3.0, "VS MODE", ui::WHITE);
                for (i, (name, on)) in VS_ITEMS.iter().enumerate() {
                    let y = 130.0 + i as f32 * 70.0;
                    let s = i == sel;
                    u.rect(150.0, y, 340.0, 52.0, if s { [120, 50, 110, 230] } else { [24, 24, 40, 200] });
                    if s { u.frame(150.0, y, 340.0, 52.0, 3.0, ui::YELLOW) }
                    u.text_centered(320.0, y + 16.0, 3.0, name, if *on { ui::WHITE } else { ui::DIM });
                }
                u.text(20.0, 452.0, 1.0, "STICK: MOVE   A: OK   B: BACK", ui::GREY);
            }
            Scene::Css => self.draw_css(),
            Scene::Sss => {
                self.backdrop([20, 40, 70, 255], [6, 10, 20, 255]);
                let sel = self.sss_sel;
                let u = &mut self.ui;
                u.text_centered_shadow(320.0, 36.0, 3.0, "STAGE SELECT", ui::WHITE);
                for (i, (_, name)) in STAGES.iter().enumerate() {
                    let (cx, cy) = (i % 2, i / 2);
                    let (x, y) = (60.0 + cx as f32 * 280.0, 110.0 + cy as f32 * 150.0);
                    u.rect(x, y, 240.0, 120.0, if i == sel { [60, 100, 150, 235] } else { [24, 30, 44, 220] });
                    if i == sel { u.frame(x, y, 240.0, 120.0, 4.0, ui::YELLOW) }
                    u.text_centered(x + 120.0, y + 50.0, 1.0, name, ui::WHITE);
                }
                u.text(20.0, 452.0, 1.0, "STICK: MOVE   A: OK   B: BACK", ui::GREY);
            }
            Scene::Loading => {
                self.backdrop([0, 0, 0, 255], [10, 10, 30, 255]);
                self.ui.text_centered_shadow(320.0, 220.0, 3.0, "LOADING", ui::WHITE);
            }
            Scene::Match => {}
            Scene::Results => self.draw_results(),
        }
        if let Some((msg, _)) = &self.toast {
            let w = Ui::text_width(msg, 1.0) + 24.0;
            self.ui.rect(320.0 - w / 2.0, 410.0, w, 28.0, [0, 0, 0, 220]);
            self.ui.text_centered(320.0, 419.0, 1.0, msg, ui::YELLOW);
        }
    }

    fn draw_css(&mut self) {
        self.backdrop([60, 30, 30, 255], [8, 8, 24, 255]);
        let (cursor, pick) = (self.cursor, self.pick);
        let u = &mut self.ui;
        u.text_centered_shadow(320.0, 24.0, 3.0, "CHARACTER SELECT", ui::WHITE);
        for (i, (_, name)) in ROSTER.iter().enumerate() {
            let (cx, cy) = (i % 4, i / 4);
            let (x, y) = (40.0 + cx as f32 * 148.0, 90.0 + cy as f32 * 110.0);
            u.rect(x, y, 136.0, 96.0, [26, 30, 46, 230]);
            u.text_centered(x + 68.0, y + 44.0, 1.0, name, ui::WHITE);
            for s in 0..2 {
                let col = if s == 0 { ui::RED } else { ui::CYAN };
                let sel = pick[s] == Some(i);
                if sel || (pick[s].is_none() && cursor[s] == i && (s == 0 || pick[0].is_some())) {
                    let inset = 2.0 + s as f32 * 5.0;
                    u.frame(x - 2.0 + inset, y - 2.0 + inset, 140.0 - 2.0 * inset, 100.0 - 2.0 * inset, if sel { 4.0 } else { 2.0 }, col);
                }
            }
        }
        for s in 0..2 {
            let y = 330.0;
            let x = 60.0 + s as f32 * 280.0;
            let col = if s == 0 { ui::RED } else { ui::CYAN };
            u.rect(x, y, 240.0, 70.0, [16, 16, 28, 230]);
            u.frame(x, y, 240.0, 70.0, 3.0, col);
            u.text(x + 12.0, y + 10.0, 2.0, &format!("P{}", s + 1), col);
            let label = match pick[s] {
                Some(i) => ROSTER[i].1.to_string(),
                None => {
                    if s == 0 || pick[0].is_some() { format!("{}?", ROSTER[cursor[s]].1) } else { "...".into() }
                }
            };
            u.text(x + 12.0, y + 40.0, 2.0, &label, ui::WHITE);
        }
        let hint = if pick[1].is_some() { "A / START: STAGE SELECT   B: BACK" } else { "STICK: MOVE   A: PICK   B: BACK" };
        u.text(20.0, 452.0, 1.0, hint, ui::GREY);
    }


    fn draw_results(&mut self) {
        self.backdrop([30, 20, 60, 255], [4, 4, 14, 255]);
        let o = self.outcome.clone();
        let u = &mut self.ui;
        u.text_centered_shadow(320.0, 30.0, 4.0, "RESULTS", ui::WHITE);
        let head = match o.winner {
            Some(w) => format!("{} WINS", o.names[w]),
            None => "DRAW".into(),
        };
        u.text_centered_shadow(320.0, 90.0, 3.0, &head, ui::YELLOW);
        for i in 0..2 {
            let x = 60.0 + i as f32 * 280.0;
            let col = if i == 0 { ui::RED } else { ui::CYAN };
            u.rect(x, 150.0, 240.0, 170.0, [16, 16, 30, 230]);
            u.frame(x, 150.0, 240.0, 170.0, 3.0, if o.winner == Some(i) { ui::YELLOW } else { col });
            u.text(x + 14.0, 164.0, 2.0, &format!("P{} {}", i + 1, o.names[i]), col);
            u.text(x + 14.0, 210.0, 2.0, &format!("FALLS   {}", o.falls[i]), ui::WHITE);
            u.text(x + 14.0, 245.0, 2.0, &format!("DAMAGE  {}%", o.percent[i].max(0.0) as i32), ui::WHITE);
        }
        let secs = o.frames.max(0) / 60;
        u.text_centered(320.0, 345.0, 2.0, &format!("TIME  {}:{:02}", secs / 60, secs % 60), ui::GREY);
        if o.why != "TIME!" {
            u.text_centered(320.0, 385.0, 1.0, &o.why.chars().take(78).collect::<String>(), ui::DIM);
        }
        u.text(20.0, 452.0, 1.0, "A: CHARACTER SELECT   START: MAIN MENU", ui::GREY);
    }

    /// Write the pending screenshot (the presenter's last frame), if any.
    fn emit_shot(&mut self) {
        let (Some(name), Some(dir), Some(p)) = (self.shot_pending.take(), self.shot_dir.as_ref(), self.presenter.as_mut()) else { return };
        let px = p.screenshot();
        let path = dir.join(format!("play_{name}.png"));
        match write_png(&path, p.renderer.width, p.renderer.height, &px) {
            Ok(()) => log(&format!("screenshot {}", path.display())),
            Err(e) => log(&format!("screenshot: {e}")),
        }
    }

    /// Draw and present the current state: the engine's own frame during an engine match, else the stand-in scene.
    /// `alpha`: Some(t) for a render-only interpolated present between the last two sim frames.
    fn present_now(&mut self, alpha: Option<f64>) {
        if self.scene == Scene::Match && self.em.is_some() {
            let frame = self.em.as_mut().and_then(|m| m.frame.take());
            let p = self.presenter.as_mut().unwrap();
            p.present_frames(&melee_engine::gxbridge::shim().gx.state, frame, alpha);
            return;
        }
        if !std::mem::take(&mut self.dirty) {
            // no tick since the last present: show the last frame again
            let p = self.presenter.as_mut().unwrap();
            p.present_frames(&self.shim.gx.state, None, None);
            return;
        }
        self.draw_frame();
        let frame = self.shim.take_frame();
        let p = self.presenter.as_mut().unwrap();
        p.present_frames(&self.shim.gx.state, frame, None);
        self.shim.log.clear();
    }

    fn draw_frame(&mut self) {
        self.shim.set_copy_clear(GxColor::new(0, 0, 0, 255), 0xFF_FFFF);
        if self.scene == Scene::Match {
            // the engine's own frame is presented in `present_now`; nothing of the stand-in UI during a match
            self.ui.clear();
        } else if let (Scene::Title, Some(t)) = (self.scene, self.title.as_mut()) {
            t.draw(&mut self.shim);
        } else if let (Scene::MainMenu | Scene::VsMenu, Some(m)) = (self.scene, self.menu.as_mut()) {
            m.draw(&mut self.shim);
        } else {
            self.draw_ui();
        }
        self.ui.flush(&mut self.shim);
        self.shim.copy_disp(true);
    }
}

impl Game for Play {
    fn tick(&mut self, pads: &mut PadSystem) {
        let mut st = [pad_disconnected(); PAD_CHANMAX];
        pads.read(&mut st);
        for i in 0..4 {
            self.port[i].update(&st[i]);
        }
        self.tick_no += 1;
        self.age += 1;
        self.tick_scene();
        self.dirty = true;
        if let Some(a) = self.audio.as_mut() {
            a.tick();
        }
        if self.max_frames.is_some_and(|n| self.tick_no >= n) {
            log(&format!("max frames reached in {}", self.scene.name()));
            self.quit = true;
        }
        if self.shots && (self.shot_ages.contains(&self.age) || self.shot_ticks.contains(&(self.tick_no - 1))) && self.presenter.is_some() {
            // Age-exact: draw this tick's state now (the display may have skipped ticks), then write it.
            self.shot_pending = Some(if self.shot_ticks.contains(&(self.tick_no - 1)) { format!("tl_{:04}", self.tick_no - 1) } else { format!("{}_{:04}", self.scene.name(), self.age) });
            self.present_now(None);
            self.emit_shot();
        }
        if self.shots && self.shot_ages.is_empty() && self.age == 30 && self.shot_pending.is_none() {
            self.shot_pending = Some(format!("m{}_{}", self.matches_played, self.scene.name()));
        }
        // Match: one shot well into the fight and one 10 ticks into a pause.
        let paused = false;
        self.pause_age = if paused { self.pause_age + 1 } else { 0 };
        if self.shots && self.shot_pending.is_none() {
            if paused && self.pause_age == 10 {
                self.shot_pending = Some(format!("m{}_pause", self.matches_played));
            } else if self.scene == Scene::Match && self.age == 420 {
                self.shot_pending = Some(format!("m{}_live", self.matches_played));
            }
        }
    }

    fn set_interp(&mut self, on: bool) {
        log(&format!("render interpolation {}", if on { "on" } else { "off" }));
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
        // A pending screenshot is of the newest sim frame, never an interpolated one.
        let a = (self.interp && self.shot_pending.is_none()).then_some(alpha);
        self.present_now(a);
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
}

pub fn play(args: &[String]) -> ExitCode {
    let get = |k: &str| args.iter().position(|a| a == k).and_then(|i| args.get(i + 1)).cloned();
    let has = |k: &str| args.iter().any(|a| a == k);
    let Some(iso) = get("--iso").or_else(|| std::env::var("GW_ISO_VANILLA").ok()) else {
        eprintln!("usage: melee play --iso DISC.iso [--time-secs N] [--shots] [--shot-dir DIR] [--quit-at-results] [--max-frames N] [--skip-intro]");
        return ExitCode::from(2);
    };
    crate::crash::note_disc(&iso);
    if let Err(e) = crate::disc::Disc::open(&iso) {
        eprintln!("play: {e}");
        return ExitCode::FAILURE;
    }
    let shot_dir = get("--shot-dir").map(PathBuf::from).or_else(|| has("--shots").then(|| PathBuf::from("target/shots")));
    if let Some(d) = &shot_dir {
        if let Err(e) = std::fs::create_dir_all(d) {
            eprintln!("play: {}: {e}", d.display());
            return ExitCode::FAILURE;
        }
    }
    let game = Play {
        iso,
        mods_dir: get("--mods"),
        time_secs: get("--time-secs").and_then(|s| s.parse().ok()).unwrap_or(180),
        golden_pads: has("--golden-pads"),
        scene: Scene::Opening,
        age: 0,
        tick_no: 0,
        port: [PortIn::default(); 4],
        title_count: 0,
        main_sel: 1,
        vs_sel: 0,
        toast: None,
        cursor: [0, 1],
        pick: [None, None],
        sss_sel: 0,
        setup: None,
        em: None,
        engine_used: false,
        headless: false,
        outcome: Outcome::default(),
        matches_played: 0,
        shim: GxShim::new(),
        ui: Ui::new(),
        presenter: None,
        shots: shot_dir.is_some(),
        shot_dir,
        shot_pending: None,
        pause_age: 0,
        quit_at_results: has("--quit-at-results"),
        max_frames: get("--max-frames").and_then(|s| s.parse().ok()),
        skip_intro: has("--skip-intro") || crate::cfg::flag("skip_intro", false),
        quit: false,
        failed: None,
        audio: None,
        title: None,
        menu: None,
        kit: Box::new(crate::retail::NoKit),
        interp: false,
        dirty: true,
        shot_ticks: get("--shot-ticks").map(|v| v.split(',').filter_map(|x| x.trim().parse().ok()).collect()).unwrap_or_default(),
        shot_ages: get("--shot-ages").map(|v| v.split(',').filter_map(|x| x.trim().parse().ok()).collect()).unwrap_or_default(),
    };
    let mut game = game;
    // `--match falcon,fox,GrNLa.dat`: skip the menus (a hands-off test of the match path)
    if let Some(spec) = get("--match") {
        let parts: Vec<&str> = spec.split(',').collect();
        let f = |n: &str| ROSTER.iter().position(|r| r.0 == n);
        let st = parts.get(2).and_then(|n| STAGES.iter().position(|s| s.0 == *n)).unwrap_or(0);
        match (parts.first().and_then(|n| f(n)), parts.get(1).and_then(|n| f(n))) {
            (Some(a), Some(b)) => {
                game.setup = Some(MatchSetup { fighters: [a, b], stage: st });
                game.scene = Scene::Loading;
            }
            _ => {
                eprintln!("play: --match NAME,NAME[,STAGE.dat]: unknown fighter");
                return ExitCode::from(2);
            }
        }
    }
    // --headless: no window, no device, no gamepads; the pad script (MELEE_PAD_SCRIPT) drives
    // the ports and --max-frames / --quit-at-results end the run. --audio-wav keeps the mix.
    let headless = has("--headless");
    game.headless = headless;
    let trace_path = get("--audio-trace");
    if !has("--no-audio") && crate::cfg::flag("audio", true) {
        match crate::audio::Audio::boot(&game.iso, headless) {
            Ok(a) => game.audio = Some(a),
            Err(e) => log(&format!("audio off: {e}")),
        }
    }
    if let Some(a) = &game.audio {
        if trace_path.is_some() {
            melee_audio::sim::with(|s| s.trace = Some(Vec::new()));
        }
        a.scene_start();
        a.stop_music();
    }
    if headless {
        let mut pads = PadSystem::headless();
        let limit = game.max_frames.unwrap_or(60 * 600);
        let ticks = run_headless(&mut game, &mut pads, limit);
        log(&format!("headless: {ticks} ticks, {}", game.scene.name()));
        if let (Some(path), Some(cap)) = (get("--audio-wav"), game.audio.as_ref().and_then(|a| a.capture.as_ref())) {
            if let Err(e) = crate::write_wav(&path, cap) {
                eprintln!("play: {path}: {e}");
                return ExitCode::FAILURE;
            }
            log(&format!("audio mix of {} sub-frames written to {path}", cap.len() / 320));
        }
        if let Some(p) = &trace_path {
            let t = melee_audio::sim::with(|s| s.trace.take()).flatten().unwrap_or_default();
            if let Err(e) = std::fs::write(p, t.join("
") + "
") {
                eprintln!("play: {p}: {e}");
            }
            log(&format!("{} audio calls written to {p}", t.len()));
        }
        return if game.failed.is_some() { ExitCode::FAILURE } else { ExitCode::SUCCESS };
    }
    let scripted = std::env::var_os("MELEE_PAD_SCRIPT").is_some_and(|v| !v.is_empty());
    let cfg = WindowConfig { title: "GD's Melee (native)".into(), width: 1280, height: 960, active: !scripted };
    if scripted {
        std::env::set_var("MELEE_PAD_IGNORE_ADAPTER", "1");
    }
    let mut pads = PadSystem::interactive();
    if scripted {
        // Deterministic and hands-off: no keyboard, adapter or gamepad can leak into a scripted run, and the window never takes focus.
        pads.set_ignore_adapter(true);
        pads.gamepads = melee_platform::pad_read::GamepadSource::Fixed(Default::default());
    }
    // Port 1 on the keyboard (WASD stick, IJKL C-stick, Space A, Shift B, F/G X/Y, Q/E L/R, Z, Enter START, arrows d-pad)
    // unless MELEE_KEYBOARD=0; an adapter or gamepad on port 1 still wins, a pad script wins over both.
    if !scripted && std::env::var("MELEE_KEYBOARD").map_or(true, |v| v != "0") {
        pads.keyboard[0] = melee_platform::aurora::KeyboardBindings::preset_default();
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
