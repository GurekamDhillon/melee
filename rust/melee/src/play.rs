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
use crate::vs_match::input::RawPad;
use crate::ui::{self, Ui};
use crate::view::MatchRender;
use crate::vs_match::VsMatch;
use crate::Sim;
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

struct Match {
    sim: VsMatch,
    mr: MatchRender,
    setup_names: [&'static str; 2],
    paused: Option<usize>,
    /// Ticks since a stock count reached 0 / the sim stopped (the GameSet beat before the results).
    end_timer: Option<u32>,
    /// The clock ran out (the TIME! call comes with the end timer).
    timed_out: bool,
    quit_hold: u32,
}

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
    m: Option<Match>,
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
    /// `--shot-ages a,b,c`: a screenshot at those scene ages of every scene (`play_<scene>_<age>.png`).
    shot_ages: Vec<u32>,
    kit: Box<dyn crate::retail::MenuHook>,
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
                if let (Some(su), Some(m)) = (self.setup.as_ref(), self.m.as_ref()) {
                    let ck = [crate::audio::ckind_of(ROSTER[su.fighters[0]].0), crate::audio::ckind_of(ROSTER[su.fighters[1]].0)];
                    let (st, gr) = crate::audio::stage_kinds(STAGES[su.stage].0);
                    a.match_start(&ck, st, gr, m.sim.bgm);
                }
            }
            Scene::Results => {
                if from == Scene::Match {
                    a.match_leave();
                }
                let w = self.outcome.winner.map(|w| crate::audio::ckind_of(self.m.as_ref().map_or("falcon", |m| m.setup_names[w])));
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
        self.m = None;
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
                        // gmTitleMode onExit: START -> GM_MENU
                        self.main_sel = 1;
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
            Scene::MainMenu => {
                if ny != 0 {
                    self.main_sel = (self.main_sel as i32 - ny).rem_euclid(MAIN_ITEMS.len() as i32) as usize;
                    if let Some(a) = &self.audio { a.moved() }
                }
                if trig & HSD_PAD_A != 0 {
                    if let Some(a) = &self.audio { if MAIN_ITEMS[self.main_sel].1 { a.forward() } else { a.denied() } }
                    if MAIN_ITEMS[self.main_sel].1 {
                        self.vs_sel = 0;
                        self.goto(Scene::VsMenu);
                    } else {
                        self.toast = Some((format!("{} IS NOT IN THE NATIVE PORT YET", MAIN_ITEMS[self.main_sel].0), 120));
                    }
                } else if trig & HSD_PAD_B != 0 {
                    if let Some(a) = &self.audio { a.back() }
                    self.title_count = 0;
                    self.goto(Scene::Title);
                }
            }
            Scene::VsMenu => {
                if ny != 0 {
                    self.vs_sel = (self.vs_sel as i32 - ny).rem_euclid(VS_ITEMS.len() as i32) as usize;
                    if let Some(a) = &self.audio { a.moved() }
                }
                if trig & HSD_PAD_A != 0 {
                    if let Some(a) = &self.audio { if VS_ITEMS[self.vs_sel].1 { a.forward() } else { a.denied() } }
                    if VS_ITEMS[self.vs_sel].1 {
                        self.cursor = [0, 1];
                        self.pick = [None, None];
                        self.goto(Scene::Css);
                    } else {
                        self.toast = Some((format!("{} IS NOT IN THE NATIVE PORT YET", VS_ITEMS[self.vs_sel].0), 120));
                    }
                } else if trig & HSD_PAD_B != 0 {
                    if let Some(a) = &self.audio { a.back() }
                    self.goto(Scene::MainMenu);
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
                    self.m = None;
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
        log(&format!("loading {} v {} on {} ({} s time match)", names[0], names[1], stage_name, self.time_secs));
        let built = (|| -> Result<Match, String> {
            let mut disc = crate::disc::Disc::open(&self.iso)?;
            if let Some(d) = &self.mods_dir {
                disc.mods = Some(crate::mods::Mods::open(std::path::Path::new(d), None)?);
            }
            let seed = 12345 + self.matches_played;
            let mut sim = VsMatch::new(&mut disc, stage_file, &names, seed, 3)?;
            let mr = MatchRender::new(&mut disc, stage_file, &names, &mut sim)?;
            Ok(Match { sim, mr, setup_names: names, paused: None, end_timer: None, timed_out: false, quit_hold: 0 })
        })();
        match built {
            Ok(m) => {
                self.matches_played += 1;
                self.m = Some(m);
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
        let Some(m) = self.m.as_mut() else { return };
        let ports = self.port;
        // gm_DefaultVSGetPauser: START on a connected pad pauses; the pauser's START resumes.
        let pauser = ports.iter().take(2).position(|p| p.connected && p.trig & HSD_PAD_START != 0 && p.held & (HSD_PAD_L | HSD_PAD_R | HSD_PAD_A) != (HSD_PAD_L | HSD_PAD_R | HSD_PAD_A));
        match (m.paused, pauser) {
            (None, Some(p)) if m.end_timer.is_none() && m.sim.go_reached() => {
                m.paused = Some(p);
                if let Some(a) = self.audio.as_mut() { a.pause(true) }
                log(&format!("paused by port {}", p + 1));
            }
            (Some(q), Some(p)) if q == p => {
                m.paused = None;
                if let Some(a) = self.audio.as_mut() { a.pause(false) }
                log("resumed");
            }
            _ => {}
        }
        // L+R+A+START held (gmvs): end the match and return to the character select.
        const COMBO: u32 = HSD_PAD_L | HSD_PAD_R | HSD_PAD_A | HSD_PAD_START;
        let quit_combo = ports.iter().take(2).any(|p| p.connected && p.held & COMBO == COMBO);
        m.quit_hold = if quit_combo { m.quit_hold + 1 } else { 0 };
        if m.quit_hold >= 30 {
            log("match quit with L+R+A+START");
            self.back_to_css();
            return;
        }
        // The combo is read at the pad layer: the fighters do not see it (the sim holds meanwhile).
        if m.paused.is_some() || quit_combo {
            return;
        }
        if let Some(t) = &mut m.end_timer {
            // The GameSet beat (a halted sim holds its last frame).
            *t += 1;
            if *t == crate::audio::TIME_CALL_DELAY as u32 && m.timed_out {
                if let Some(a) = &self.audio { a.match_end_call(true) }
            }
            if *t >= 150 {
                self.finish_match();
            }
            return;
        }
        // The frame: the live pads in, the sim stepped once.
        let mut raws = [RawPad::default(); 4];
        for i in 0..4 {
            raws[i] = if self.golden_pads { crate::vs_match::input::pad_fixed(i, m.sim.frame + 1) } else { ports[i].raw };
        }
        // START belongs to the pause layer (gm_DefaultVSGetPauser); no fighter check reads it.
        for r in raws.iter_mut() {
            r.button &= !HSD_PAD_START;
        }
        m.sim.pad_feed = Some(raws);
        let alive = m.sim.step();
        if alive {
            m.mr.step(&mut m.sim);
            if let Some(a) = &self.audio {
                a.match_frame(m.sim.frame, m.sim.go_frame_for_hud(), self.time_secs);
            }
        }
        if !alive {
            log(&format!("simulation halted: {:?}", m.sim.stopped));
            if let Some(a) = &self.audio { a.match_end_call(false) }
            m.end_timer = Some(0);
        } else if m.sim.frame >= crate::audio::clock_end(m.sim.go_frame_for_hud(), self.time_secs) - 1 {
            log("time up: GameSet");
            m.timed_out = true;
            m.end_timer = Some(0);
            // gmVs: the end of the clock stops every track, then the TIME! call.
            if let Some(a) = &self.audio { a.match_leave() }
        }
    }

    fn finish_match(&mut self) {
        let Some(m) = self.m.as_ref() else { return };
        let mut o = Outcome { frames: m.sim.frame, ..Default::default() };
        let main: Vec<_> = m.sim.players.iter().filter(|p| !p.follower).collect();
        for (i, p) in main.iter().take(2).enumerate() {
            o.names[i] = ROSTER.iter().find(|r| r.0 == m.setup_names[i]).map_or(m.setup_names[i], |r| r.1).to_string();
            o.falls[i] = p.falls as i32;
            o.percent[i] = p.fp.dmg.x1830_percent;
        }
        match m.sim.stopped.as_ref() {
            Some(why) => o.why = format!("simulation reached unported code: {why}"),
            None => {
                // Time-match score: -1 per fall / self-destruct, no KOs are possible in the ported sim.
                o.winner = if o.falls[0] < o.falls[1] {
                    Some(0)
                } else if o.falls[1] < o.falls[0] {
                    Some(1)
                } else {
                    None
                };
                o.why = "TIME!".into();
            }
        }
        log(&format!("results: falls {:?} percent {:?} frames {} winner {:?} ({})", o.falls, o.percent, o.frames, o.winner, o.why));
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

    fn draw_match_hud(&mut self) {
        let Some(m) = &self.m else { return };
        let u = &mut self.ui;
        let main: Vec<_> = m.sim.players.iter().filter(|p| !p.follower).collect();
        for (i, p) in main.iter().take(2).enumerate() {
            let x = 110.0 + i as f32 * 300.0;
            let col = if i == 0 { ui::RED } else { ui::CYAN };
            u.rect(x, 396.0, 160.0, 70.0, [0, 0, 0, 150]);
            u.frame(x, 396.0, 160.0, 70.0, 2.0, col);
            let pct = p.fp.dmg.x1830_percent.max(0.0) as i32;
            u.text_shadow(x + 10.0, 406.0, 4.0, &format!("{pct}%"), if pct > 100 { ui::RED } else { ui::WHITE });
            u.text(x + 10.0, 450.0, 1.0, &format!("FALLS {}", p.falls), col);
            u.text(x + 100.0, 450.0, 1.0, &m.setup_names[i].to_uppercase(), ui::GREY);
        }
        let secs = (m.sim.frame.max(0) / 60) as u32;
        u.text_centered_shadow(320.0, 12.0, 2.0, &format!("{}:{:02}", secs / 60, secs % 60), ui::WHITE);
        if !m.sim.go_reached() {
            u.text_centered_shadow(320.0, 200.0, 6.0, "READY", ui::YELLOW);
        } else if m.sim.frame < m.sim.go_frame_for_hud() + 50 && m.end_timer.is_none() {
            u.text_centered_shadow(320.0, 200.0, 6.0, "GO!", ui::GREEN);
        }
        if m.end_timer.is_some() {
            let msg = if m.sim.stopped.is_some() { "SIMULATION HALTED" } else { "TIME!" };
            u.text_centered_shadow(320.0, 190.0, 6.0, msg, ui::YELLOW);
            if let Some(why) = &m.sim.stopped {
                u.text_centered(320.0, 250.0, 1.0, &why.chars().take(76).collect::<String>(), ui::GREY);
            }
        }
        if let Some(p) = m.paused {
            u.rect(0.0, 0.0, ui::W, ui::H, [0, 0, 0, 140]);
            u.text_centered_shadow(320.0, 180.0, 5.0, "PAUSE", ui::WHITE);
            u.text_centered(320.0, 250.0, 2.0, &format!("PLAYER {}   START: RESUME", p + 1), ui::GREY);
            u.text_centered(320.0, 280.0, 1.0, "HOLD L+R+A+START: QUIT TO CHARACTER SELECT", ui::GREY);
        }
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
        match write_png(&path, EFB_WIDTH, EFB_HEIGHT, &px) {
            Ok(()) => log(&format!("screenshot {}", path.display())),
            Err(e) => log(&format!("screenshot: {e}")),
        }
    }

    fn draw_frame(&mut self) {
        self.shim.set_copy_clear(GxColor::new(0, 0, 0, 255), 0xFF_FFFF);
        if self.scene == Scene::Match {
            if let Some(m) = self.m.as_mut() {
                m.mr.draw(&mut self.shim, &m.sim);
            }
            self.ui.clear();
            self.draw_match_hud();
        } else if let (Scene::Title, Some(t)) = (self.scene, self.title.as_mut()) {
            t.draw(&mut self.shim);
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
        if let Some(a) = self.audio.as_mut() {
            a.tick();
        }
        if self.max_frames.is_some_and(|n| self.tick_no >= n) {
            log(&format!("max frames reached in {}", self.scene.name()));
            self.quit = true;
        }
        if self.shots && self.shot_ages.contains(&self.age) && self.presenter.is_some() {
            // Age-exact: draw this tick's state now (the display may have skipped ticks), then write it.
            self.shot_pending = Some(format!("{}_{:04}", self.scene.name(), self.age));
            self.draw_frame();
            let frame = self.shim.take_frame();
            let p = self.presenter.as_mut().unwrap();
            p.present(&self.shim.gx.state, frame.as_deref());
            self.shim.log.clear();
            self.emit_shot();
        }
        if self.shots && self.shot_ages.is_empty() && self.age == 30 && self.shot_pending.is_none() {
            self.shot_pending = Some(format!("m{}_{}", self.matches_played, self.scene.name()));
        }
        // Match: one shot well into the fight and one 10 ticks into a pause.
        let paused = self.scene == Scene::Match && self.m.as_ref().is_some_and(|m| m.paused.is_some());
        self.pause_age = if paused { self.pause_age + 1 } else { 0 };
        if self.shots && self.shot_pending.is_none() {
            if paused && self.pause_age == 10 {
                self.shot_pending = Some(format!("m{}_pause", self.matches_played));
            } else if self.scene == Scene::Match && self.age == 420 {
                self.shot_pending = Some(format!("m{}_live", self.matches_played));
            }
        }
    }

    fn present(&mut self, window: &Arc<Window>, _alpha: f64) {
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
        self.draw_frame();
        let frame = self.shim.take_frame();
        let p = self.presenter.as_mut().unwrap();
        p.present(&self.shim.gx.state, frame.as_deref());
        self.shim.log.clear();
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
        m: None,
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
        kit: Box::new(crate::retail::NoKit),
        shot_ages: get("--shot-ages").map(|v| v.split(',').filter_map(|x| x.trim().parse().ok()).collect()).unwrap_or_default(),
    };
    let mut game = game;
    // --headless: no window, no device, no gamepads; the pad script (MELEE_PAD_SCRIPT) drives
    // the ports and --max-frames / --quit-at-results end the run. --audio-wav keeps the mix.
    let headless = has("--headless");
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
