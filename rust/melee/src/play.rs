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
//! ported) and not by the frontend kit; the opening movie (THP) is a card. The CSS sets the slots (HMN / CPU 1-9 / NA,
//! fighter of all 26, costume or team) and the rules (stock or time, items, teams), which reach the engine through its
//! launcher (`SceneLaunch_SeedVs`), as in the C build's VS mode. The match ends the game's own way (GAME! / TIME!), or with
//! L+R+A+START; after the results the same engine runs the next match (`Engine::next_vs`), any number of times.
//!
//! ```text
//! melee play [--iso DISC.iso] [--time-secs N | --stocks N] [--items OFF|LOW|..] [--teams] [--shot-dir DIR] [--shots]
//!            [--quit-at-results [--quit-after-matches N]] [--max-frames N] [--skip-intro] [--mods DIR] [--golden-pads]
//!            [--match a,b[/cpuN],..[,STAGE]] [--headless]
//! ```
//! `--time-secs` presets the time rule (the game counts whole minutes: GameRules rounds it up), `--stocks N` a stock match.
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

const HSD_PAD_X: u32 = 0x400;
const HSD_PAD_Y: u32 = 0x800;
const HSD_PAD_Z: u32 = 0x10;

/// The whole VS roster in the retail CSS's order (scene name, display, costume count, note). The scene name is
/// `melee_engine::scenario::ckind_by_name`'s; the costume counts are `gm_GetNumCostumesForCKind`'s (the launcher
/// takes the colour modulo the real count anyway). Sheik has her own icon here (retail reaches her through Zelda).
const ROSTER: &[(&str, &str, u8, &str)] = &[
    ("drmario", "DR. MARIO", 5, ""),
    ("mario", "MARIO", 5, ""),
    ("luigi", "LUIGI", 4, ""),
    ("bowser", "BOWSER", 4, ""),
    ("peach", "PEACH", 5, ""),
    ("yoshi", "YOSHI", 6, ""),
    ("dk", "DK", 5, ""),
    ("falcon", "C. FALCON", 6, ""),
    ("ganon", "GANONDORF", 5, ""),
    ("falco", "FALCO", 4, ""),
    ("fox", "FOX", 4, ""),
    ("ness", "NESS", 4, ""),
    ("ics", "ICE CLIMBERS", 4, ""),
    ("kirby", "KIRBY", 6, ""),
    ("samus", "SAMUS", 5, ""),
    ("zelda", "ZELDA", 5, ""),
    ("sheik", "SHEIK", 5, ""),
    ("link", "LINK", 5, ""),
    ("younglink", "YOUNG LINK", 4, ""),
    ("pichu", "PICHU", 4, ""),
    ("pikachu", "PIKACHU", 4, ""),
    ("jigglypuff", "JIGGLYPUFF", 5, "SPECIALS INCOMPLETE (BLOCKER D)"),
    ("mewtwo", "MEWTWO", 4, ""),
    ("gw", "MR. G&W", 4, ""),
    ("marth", "MARTH", 5, ""),
    ("roy", "ROY", 5, ""),
];
const CSS_COLS: usize = 9;
/// Every retail VS stage by its external StKind (gw_sl_stage_names; Akaneia 21 is not on the disc and Icetop 26
/// asserts in the C build too), with the display name and a known problem (docs/rust-port/notes/parity.md section 4).
pub(crate) const STAGES: &[(i32, &str, &str)] = &[
    (32, "FINAL DESTINATION", ""),
    (31, "BATTLEFIELD", ""),
    (8, "YOSHI'S STORY", ""),
    (2, "FOUNTAIN OF DREAMS", ""),
    (3, "POKEMON STADIUM", ""),
    (28, "DREAM LAND N64", ""),
    (4, "PEACH'S CASTLE", ""),
    (5, "KONGO JUNGLE", ""),
    (6, "BRINSTAR", ""),
    (7, "CORNERIA", "CRASHES (ARWINGS)"),
    (9, "ONETT", ""),
    (10, "MUTE CITY", ""),
    (11, "RAINBOW CRUISE", ""),
    (12, "JUNGLE JAPES", ""),
    (13, "GREAT BAY", "CRASHES (TINGLE)"),
    (14, "HYRULE TEMPLE", ""),
    (15, "BRINSTAR DEPTHS", ""),
    (16, "YOSHI'S ISLAND", ""),
    (17, "GREEN GREENS", ""),
    (18, "FOURSIDE", ""),
    (19, "MUSHROOM KINGDOM", ""),
    (20, "MUSHROOM KINGDOM II", "CRASHES"),
    (22, "VENOM", "CRASHES"),
    (23, "POKE FLOATS", ""),
    (24, "BIG BLUE", ""),
    (25, "ICICLE MOUNTAIN", "SCROLL DIVERGES"),
    (27, "FLAT ZONE", ""),
    (29, "YOSHI'S ISLAND N64", ""),
    (30, "KONGO JUNGLE N64", ""),
];
const SSS_COLS: usize = 5;
/// The extra SSS cell after the stages: a random stage (play's own pick, never the game's RNG).
const SSS_RANDOM: usize = STAGES.len();
const ITEM_NAMES: [&str; 6] = ["OFF", "VERY LOW", "LOW", "MEDIUM", "HIGH", "VERY HIGH"];
const TEAM_NAMES: [&str; 3] = ["RED", "BLUE", "GREEN"];
const SLOT_COLS: [ui::Rgba; 4] = [ui::RED, ui::CYAN, ui::YELLOW, ui::GREEN];
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

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum SlotKind {
    Hmn,
    Cpu,
    Off,
}

/// One CSS slot (`MnCharSel`'s player panel): who plays it, the fighter, costume, CPU level and team.
#[derive(Clone, Copy, Debug)]
struct Slot {
    kind: SlotKind,
    cursor: usize,
    pick: Option<usize>,
    color: u8,
    level: u8,
    team: u8,
}

impl Slot {
    fn new(kind: SlotKind, cursor: usize, team: u8) -> Slot {
        Slot { kind, cursor, pick: None, color: 0, level: 9, team }
    }
    /// The fighter this slot plays (a CPU plays its cursor's fighter without an explicit pick).
    fn fighter(&self) -> Option<usize> {
        match self.kind {
            SlotKind::Off => None,
            SlotKind::Cpu => Some(self.pick.unwrap_or(self.cursor)),
            SlotKind::Hmn => self.pick,
        }
    }
}

/// The rules the CSS sets (the retail Rules screen's VS subset), mapped onto `GameRules` / `StartMeleeRules`
/// by the engine's launcher (`SceneLaunch_SeedVs`), as the C build's VS mode does.
#[derive(Clone, Copy, Debug)]
struct Rules {
    stock: bool,
    stocks: i32,
    /// time limit in seconds (time mode; the menu steps whole minutes, `--time-secs` may set any value)
    time_secs: i32,
    /// a stock match's time limit in seconds (`GameRules.stock_time_limit`): `--time-secs` with `--stocks`; 0 = none
    stock_time_secs: i32,
    /// 0 off, 1..5 = item frequency 0..4 (very low .. very high)
    items: i32,
    teams: bool,
}
const RULE_ROWS: usize = 4;

#[derive(Clone)]
struct MatchSetup {
    slots: [Slot; 4],
    rules: Rules,
    /// index into STAGES
    stage: usize,
}

/// A match on the consolidated engine (`melee_engine`): the game's own scene loop, camera, HUD and pause, drawn through
/// the GX bridge. One per process: the engine boots its host and the TU statics once (consolidation.md step 18).
struct EngMatch {
    e: melee_engine::Engine,
    frame: Option<Vec<melee_gx::draw::DrawCmd>>,
    /// The decode thread's packing of `frame` (renderer CPU work done off the main thread).
    packed: Option<melee_gx::render::Packed>,
    setup: MatchSetup,
    end_timer: Option<u32>,
    timed_out: bool,
    quit_hold: u32,
}



#[derive(Default, Clone)]
struct Outcome {
    /// per slot: None = not in the match
    rows: [Option<OutRow>; 4],
    frames: i32,
    /// the winning slot, or the first slot of the winning team
    winner: Option<usize>,
    teams: bool,
    stock: bool,
    why: String,
}

#[derive(Default, Clone)]
struct OutRow {
    name: String,
    ckind: i32,
    cpu: bool,
    team: u8,
    kos: i32,
    falls: i32,
    stocks: i32,
    percent: f32,
}

pub struct Play {
    iso: String,
    mods_dir: Option<String>,
    /// The CSS rules (the `--time-secs` / `--stocks` / `--items` options preset them).
    rules: Rules,
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
    slots: [Slot; 4],
    /// What port 1 edits on the CSS: 0..3 a slot, 4 the rules bar (Z cycles it).
    focus: usize,
    rule_row: usize,
    sss_sel: usize,
    setup: Option<MatchSetup>,
    /// The match (the consolidated engine).
    em: Option<EngMatch>,
    /// The engine between matches: booted once per process, each next match is `Engine::next_vs`.
    spare: Option<melee_engine::Engine>,
    /// play's own random source (the SSS random stage), never the game's RNG
    menu_rng: u32,
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
    /// `--quit-after-matches N` (with `--quit-at-results`): quit on the Nth results screen (default 1).
    quit_after: u32,
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
            // the engine's own lbAudioAx calls (engine_audio) load the banks and start the stage music
            Scene::Match => {}
            Scene::Results => {
                if from == Scene::Match {
                    a.match_leave();
                }
                let w = self.outcome.winner.and_then(|w| self.outcome.rows[w].as_ref()).map(|r| r.ckind);
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

    /// Leave the match (or the results) for the character select. The picks and rules stay, as on the retail CSS
    /// after a match, so START goes straight to a rematch. The engine leaves its scene and is kept for the next one.
    fn back_to_css(&mut self) {
        if self.scene == Scene::Match {
            if let Some(a) = &self.audio { a.match_leave() }
        }
        if let Some(mut m) = self.em.take() {
            m.e.leave_scene();
            self.spare = Some(m.e);
        }
        self.frame_mem("back to css");
        self.focus = 0;
        self.goto(Scene::Css);
    }

    /// The process's memory (working set, private bytes) in the log: checks that match after match does not grow.
    fn frame_mem(&self, what: &str) {
        mem_line(self.matches_played, what);
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
                        for s in self.slots.iter_mut() {
                            s.pick = None;
                        }
                        self.focus = 0;
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
                let n = STAGES.len() + 1;
                if nx != 0 {
                    self.sss_sel = (self.sss_sel as i32 + nx).rem_euclid(n as i32) as usize;
                }
                if ny != 0 {
                    let v = self.sss_sel as i32 - ny * SSS_COLS as i32;
                    if (0..n as i32).contains(&v) {
                        self.sss_sel = v as usize;
                    }
                }
                if trig & (HSD_PAD_A | HSD_PAD_START) != 0 {
                    if let Some(a) = &self.audio { a.forward() }
                    let stage = if self.sss_sel == SSS_RANDOM {
                        self.menu_rng = self.menu_rng.wrapping_mul(1103515245).wrapping_add(12345 + self.tick_no as u32);
                        (self.menu_rng >> 16) as usize % STAGES.len()
                    } else {
                        self.sss_sel
                    };
                    self.setup = Some(MatchSetup { slots: self.slots, rules: self.rules, stage });
                    self.goto(Scene::Loading);
                } else if trig & HSD_PAD_B != 0 {
                    if let Some(a) = &self.audio { a.back() }
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
                if self.quit_at_results && self.matches_played >= self.quit_after && self.age >= 90 {
                    self.quit = true;
                }
                if trig & (HSD_PAD_A | HSD_PAD_START) != 0 {
                    if let Some(a) = &self.audio { a.forward() }
                }
                if trig & HSD_PAD_A != 0 {
                    self.back_to_css();
                } else if trig & HSD_PAD_START != 0 {
                    if let Some(mut m) = self.em.take() {
                        m.e.leave_scene();
                        self.spare = Some(m.e);
                    }
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

    /// The port that edits slot `i`: its own pad when one is plugged in (ports 2-4), else port 1 when it has the focus.
    fn slot_port(&self, i: usize) -> Option<usize> {
        if i > 0 && self.port[i].connected {
            Some(i)
        } else if self.focus == i {
            Some(self.first_port())
        } else {
            None
        }
    }
    /// Port 1, or the first connected port when port 1 is unplugged.
    fn first_port(&self) -> usize {
        if self.port[0].connected { 0 } else { self.port.iter().position(|q| q.connected).unwrap_or(0) }
    }

    /// Why the CSS cannot go on yet, or None when it can.
    fn css_blocker(&self) -> Option<&'static str> {
        let active: Vec<&Slot> = self.slots.iter().filter(|s| s.kind != SlotKind::Off).collect();
        if active.len() < 2 {
            return Some("NEED 2 OR MORE PLAYERS");
        }
        if active.iter().any(|s| s.fighter().is_none()) {
            return Some("EVERY HMN SLOT MUST PICK");
        }
        if self.rules.teams && active.iter().all(|s| s.team == active[0].team) {
            return Some("TEAMS: NEED 2 TEAMS");
        }
        None
    }

    /// Character select (the stand-in for `MnCharSel`): four slots, each HMN / CPU (level 1-9) / off, a fighter and a
    /// costume (or a team in a team match), and the rules bar. Port 1 edits the slot it has the focus on (Z moves the
    /// focus P1 -> P2 -> P3 -> P4 -> RULES); ports 2-4 edit their own slot when plugged in. In a slot: the stick moves the
    /// cursor, A picks, B takes back (B with nothing picked: the focus steps back; on P1, back to the VS menu), X cycles
    /// HMN/CPU/OFF, Y cycles the costume (the team in a team match), L/R lower/raise the CPU level. On the rules bar:
    /// up/down picks a rule, left/right (or A) changes it, B returns to P1. START (or A on a picked slot, everyone
    /// ready) goes on to the stage select.
    fn tick_css(&mut self) {
        let p1 = self.first_port();
        if self.port[p1].trig & HSD_PAD_Z != 0 {
            self.focus = (self.focus + 1) % 5;
            if let Some(a) = &self.audio { a.moved() }
            return;
        }
        let ready = self.css_blocker().is_none();
        let start = self.port.iter().any(|q| q.connected && q.trig & HSD_PAD_START != 0);
        if start {
            if ready {
                if let Some(a) = &self.audio { a.forward() }
                self.goto(Scene::Sss);
            } else if let Some(why) = self.css_blocker() {
                self.toast = Some((why.into(), 90));
            }
            return;
        }
        if self.focus == 4 {
            self.tick_rules(p1);
            return;
        }
        for i in 0..4 {
            let Some(pi) = self.slot_port(i) else { continue };
            let src = self.port[pi];
            let teams = self.rules.teams;
            let mut moved = false;
            let mut sl = self.slots[i];
            if src.trig & HSD_PAD_X != 0 {
                sl.kind = match sl.kind {
                    SlotKind::Hmn => SlotKind::Cpu,
                    SlotKind::Cpu => SlotKind::Off,
                    SlotKind::Off => SlotKind::Hmn,
                };
                if sl.kind == SlotKind::Off {
                    sl.pick = None;
                }
                self.slots[i] = sl;
                if let Some(a) = &self.audio { a.moved() }
                continue;
            }
            let back = src.trig & HSD_PAD_B != 0;
            if sl.kind == SlotKind::Off {
                // A on a closed slot opens it as a human (the retail HMN / CPU / NA button)
                if src.trig & HSD_PAD_A != 0 {
                    sl.kind = SlotKind::Hmn;
                    self.slots[i] = sl;
                    if let Some(a) = &self.audio { a.moved() }
                } else if back && pi == p1 {
                    self.css_back(i);
                    return;
                }
                continue;
            }
            if src.trig & HSD_PAD_Y != 0 {
                if teams {
                    sl.team = (sl.team + 1) % 3;
                } else {
                    sl.color = (sl.color + 1) % ROSTER[sl.pick.unwrap_or(sl.cursor)].2;
                }
                moved = true;
            }
            if sl.kind == SlotKind::Cpu {
                if src.trig & HSD_PAD_L != 0 && sl.level > 1 { sl.level -= 1; moved = true }
                if src.trig & HSD_PAD_R != 0 && sl.level < 9 { sl.level += 1; moved = true }
            }
            if sl.pick.is_none() {
                let cols = CSS_COLS as i32;
                let rows = (ROSTER.len() as i32 + cols - 1) / cols;
                let (mut cx, mut cy) = ((sl.cursor as i32) % cols, (sl.cursor as i32) / cols);
                cx = (cx + src.nav_x).rem_euclid(cols);
                cy = (cy - src.nav_y).rem_euclid(rows);
                let old = sl.cursor;
                sl.cursor = ((cy * cols + cx) as usize).min(ROSTER.len() - 1);
                if sl.cursor != old {
                    sl.color %= ROSTER[sl.cursor].2;
                    moved = true;
                }
                if src.trig & HSD_PAD_A != 0 {
                    let c = sl.cursor;
                    sl.pick = Some(c);
                    self.slots[i] = sl;
                    if let Some(a) = &self.audio { a.css_pick(melee_engine::scenario::ckind_by_name(ROSTER[c].0).unwrap_or(0), c) }
                    log(&format!("css: slot {} picked {}", i + 1, ROSTER[c].0));
                    // port 1 moves on to the next open human slot nobody else drives (the old two-player flow)
                    if pi == p1 && self.focus == i {
                        if let Some(n) = (i + 1..4).find(|&n| self.slots[n].kind == SlotKind::Hmn && self.slots[n].pick.is_none() && !self.port[n].connected) {
                            self.focus = n;
                        }
                    }
                    return;
                }
                if back && pi == p1 {
                    self.slots[i] = sl;
                    self.css_back(i);
                    return;
                }
            } else if src.trig & HSD_PAD_A != 0 && pi == p1 && ready {
                if let Some(a) = &self.audio { a.forward() }
                self.goto(Scene::Sss);
                return;
            } else if back {
                sl.pick = None;
                if let Some(a) = &self.audio { a.back() }
            }
            self.slots[i] = sl;
            if moved {
                if let Some(a) = &self.audio { a.moved() }
            }
        }
    }

    /// B with nothing to take back on slot `i` (port 1): the focus steps back to the previous slot and takes back its
    /// pick; on P1, back to the VS menu.
    fn css_back(&mut self, i: usize) {
        if let Some(a) = &self.audio { a.back() }
        if i == 0 {
            self.goto(Scene::VsMenu);
            return;
        }
        self.focus = i - 1;
        self.slots[i - 1].pick = None;
    }

    fn tick_rules(&mut self, p1: usize) {
        let src = self.port[p1];
        if src.nav_y != 0 {
            self.rule_row = (self.rule_row as i32 - src.nav_y).rem_euclid(RULE_ROWS as i32) as usize;
            if let Some(a) = &self.audio { a.moved() }
        }
        if src.trig & HSD_PAD_B != 0 {
            self.focus = 0;
            if let Some(a) = &self.audio { a.back() }
            return;
        }
        let d = if src.nav_x != 0 { src.nav_x } else if src.trig & HSD_PAD_A != 0 { 1 } else { 0 };
        if d == 0 {
            return;
        }
        let r = &mut self.rules;
        match self.rule_row {
            0 => r.stock = !r.stock,
            1 if r.stock => r.stocks = (r.stocks - 1 + d).rem_euclid(99) + 1,
            1 => {
                let m = ((r.time_secs + 59) / 60 - 1 + d).rem_euclid(99) + 1;
                r.time_secs = m * 60;
            }
            2 => r.items = (r.items + d).rem_euclid(ITEM_NAMES.len() as i32),
            _ => {
                r.teams = !r.teams;
                if r.teams {
                    // the retail default split: P1 / P3 red, P2 / P4 blue
                    for (i, s) in self.slots.iter_mut().enumerate() {
                        s.team = (i % 2) as u8;
                    }
                }
            }
        }
        if let Some(a) = &self.audio { a.moved() }
    }

    /// The setup as the engine's launcher reads it (`SceneLaunch_*`, melee_engine::scenario): the fields the C build's
    /// CSS and Rules screens leave in `VsModeData` / `GameRules`, seeded by `SceneLaunch_SeedVs` in the VS mode's on_load.
    fn scenario_of(su: &MatchSetup) -> melee_engine::scenario::Scenario {
        use melee_engine::scenario::{PlayerSpec, Scenario};
        let r = su.rules;
        let mut players = [None; 6];
        for (i, sl) in su.slots.iter().enumerate() {
            let Some(f) = sl.fighter() else { continue };
            let cpu = sl.kind == SlotKind::Cpu;
            players[i] = Some(PlayerSpec {
                ckind: melee_engine::scenario::ckind_by_name(ROSTER[f].0).unwrap_or(0),
                slot_type: cpu as i32,
                cpu_level: if cpu { sl.level as i32 } else { -1 },
                color: sl.color as i32,
                team: if r.teams { sl.team as i32 } else { -1 },
                stocks: -1,
                handicap: -1,
                cpu_idle: false,
            });
        }
        Scenario {
            stage: STAGES[su.stage].0,
            players,
            rule_match: r.stock as i32,
            rule_minutes: -1,
            item_freq: if r.items == 0 { -2 } else { r.items - 1 },
            teams: r.teams as i32,
            // a time match counts its seconds; a stock match runs the stock time limit when one was given
            // (the C VS mode's GameRules.stock_time_limit: the HUD clock and the Stadium screen show it)
            time_limit: if r.stock { r.stock_time_secs } else { r.time_secs },
            rule_stocks: if r.stock { r.stocks } else { -1 },
            onep_mode: -1,
            onep_step: -1,
            onep_level: -1, game_mode: -1, entry_state: -1,
        }
    }

    fn start_match(&mut self) {
        let Some(su) = self.setup.clone() else { return };
        let (_, stage_name, _) = STAGES[su.stage];
        let who: Vec<String> = su.slots.iter().enumerate().filter_map(|(i, s)| s.fighter().map(|f| format!("P{} {}{}", i + 1, ROSTER[f].0, if s.kind == SlotKind::Cpu { format!("/cpu{}", s.level) } else { String::new() }))).collect();
        let r = su.rules;
        log(&format!("loading {} on {} ({}, items {}, teams {}), engine match {}", who.join(" v "), stage_name,
            if r.stock { format!("{} stock", r.stocks) } else { format!("{} s time", r.time_secs) }, ITEM_NAMES[r.items as usize], r.teams, self.matches_played + 1));
        // MELEE_PLAY_SCENE=<scene string>: any scenario's match instead (e.g. a 4-player CPU free-for-all for profiling).
        let scn = match std::env::var("MELEE_PLAY_SCENE") {
            Ok(s) => match melee_engine::scenario::Scenario::parse(&s) {
                Ok(x) => x,
                Err(e) => { log(&format!("MELEE_PLAY_SCENE: {e}")); Self::scenario_of(&su) }
            },
            Err(_) => Self::scenario_of(&su),
        };
        let load_t0 = std::time::Instant::now();
        let built = if let Some(mut e) = self.spare.take() {
            // the next match of the run: the engine stays booted (Engine::next_vs, the C mode loop's next scene)
            e.next_vs(scn).map(|()| e)
        } else {
            (|| -> Result<melee_engine::Engine, String> {
                let margs: Vec<String> = self.mods_dir.iter().flat_map(|d| ["--mods".to_string(), d.clone()]).collect();
                crate::set_engine_mods(&margs)?;
                let disc = crate::engine_disc(&self.iso)?;
                unsafe {
                    melee_engine::engine::RENDER = !self.headless;
                    std::env::set_var("MELEE_TEST_SEED", "12345");
                    if std::env::var_os("MELEE_ENGINE_PRE_DRAWS").is_none() {
                        std::env::set_var("MELEE_ENGINE_PRE_DRAWS", "1");
                    }
                }
                if self.audio.is_some() {
                    // the game's own lbAudioAx calls (banks, announcer, entry sounds, pause) drive the sound the menus already booted
                    crate::engine_audio::attach(false);
                }
                melee_engine::Engine::boot_vs(disc, 64 << 20, PadSystem::headless(), scn)
            })()
        };
        match built {
            Ok(e) => {
                if self.audio.is_some() {
                    crate::engine_audio::start_stage_bgm();
                }
                self.matches_played += 1;
                // GX decode on its own thread (render-only; needs a run that never snapshots the arena, whose blocks
                // the thread's frames hold): MELEE_GX_SYNC=1 keeps it in the tick
                if melee_engine::arena::trim() && std::env::var_os("MELEE_GX_SYNC").is_none() {
                    melee_engine::gxbridge::shim().gx.enable_async(melee_engine::arena::mark_foreign_thread);
                    // screenshots show exactly their tick: take each frame when it is decoded
                    melee_engine::gxbridge::shim().wait_frame = self.shots && std::env::var_os("MELEE_SHOT_NOWAIT").is_none();
                }
                self.em = Some(EngMatch { e, frame: None, packed: None, setup: su, end_timer: None, timed_out: false, quit_hold: 0 });
                if std::env::var_os("MELEE_TICK_SPIKE").is_some() {
                    let pn = melee_engine::archive::PARSE_NS.swap(0, std::sync::atomic::Ordering::Relaxed);
                    let n = melee_engine::archive::PARSE_N.swap(0, std::sync::atomic::Ordering::Relaxed);
                    log(&format!("match load: {:.1} ms, {n} file parse(s) {:.1} ms", load_t0.elapsed().as_secs_f64() * 1e3, pn as f64 / 1e6));
                }
                self.frame_mem("match start");
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
        let quit_combo = ports.iter().any(|p| p.connected && p.held & COMBO == COMBO);
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
            m.packed = melee_engine::gxbridge::shim().gx.packed.take();
        }
        if !alive {
            // GAME! / TIME! are the game's own (the match scene ends itself); the results follow after a moment
            log(&format!("the engine's scene ended at frame {}", m.e.frames));
            m.timed_out = !m.setup.rules.stock;
            m.end_timer = Some(0);
        }
    }

    fn finish_match(&mut self) {
        let Some(m) = self.em.as_ref() else { return };
        let su = &m.setup;
        let mut o = Outcome { frames: m.e.frames as i32, teams: su.rules.teams, stock: su.rules.stock, ..Default::default() };
        for (i, sl) in su.slots.iter().enumerate() {
            let Some(f) = sl.fighter() else { continue };
            let slot = i as i32;
            let percent = unsafe {
                let g = melee_fighter::player::get_entity(slot);
                if g.is_null() || (*g).user_data.is_null() { 0.0 } else { (*((*g).user_data as *const melee_types::Fighter)).dmg.x1830_percent }
            };
            let kos = (0..6).filter(|&k| k != slot).map(|k| melee_fighter::tu::player::Player_GetKOsByPlayerIndex(slot, k)).sum();
            o.rows[i] = Some(OutRow {
                name: ROSTER[f].1.to_string(),
                ckind: melee_engine::scenario::ckind_by_name(ROSTER[f].0).unwrap_or(0),
                cpu: sl.kind == SlotKind::Cpu,
                team: sl.team,
                kos,
                falls: melee_fighter::tu::player::Player_GetFalls(slot),
                stocks: unsafe { melee_fighter::tu::player::player_slots[i].stocks as i32 },
                percent,
            });
        }
        // the winner: most stocks left (stock), most KOs minus falls (time); summed per team in a team match
        let stock = o.stock;
        let score = |r: &OutRow| if stock { r.stocks } else { r.kos - r.falls };
        let key = |i: usize| if o.teams { o.rows[i].as_ref().unwrap().team as usize } else { i };
        let mut best: Option<(i32, usize)> = None;
        let mut tie = false;
        let mut seen = Vec::new();
        for i in (0..4).filter(|&i| o.rows[i].is_some()) {
            let k = key(i);
            if seen.contains(&k) { continue }
            seen.push(k);
            let v: i32 = (0..4).filter(|&j| o.rows[j].is_some() && key(j) == k).map(|j| score(o.rows[j].as_ref().unwrap())).sum();
            match best {
                Some((b, _)) if v < b => {}
                Some((b, _)) if v == b => tie = true,
                _ => { best = Some((v, i)); tie = false; }
            }
        }
        o.winner = if tie { None } else { best.map(|b| b.1) };
        o.why = if m.timed_out { "TIME!".into() } else { "GAME!".into() };
        let rows: Vec<String> = o.rows.iter().enumerate().filter_map(|(i, r)| r.as_ref().map(|r| format!("P{} {} kos {} falls {} stocks {} {:.0}%", i + 1, r.name, r.kos, r.falls, r.stocks, r.percent))).collect();
        log(&format!("results of match {}: {} | frames {} winner {:?}", self.matches_played, rows.join(" | "), o.frames, o.winner.map(|w| w + 1)));
        self.outcome = o;
        self.frame_mem("results");
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
                u.text_centered_shadow(320.0, 20.0, 3.0, "STAGE SELECT", ui::WHITE);
                let (cw, ch) = (120.0, 44.0);
                for i in 0..=STAGES.len() {
                    let (name, note) = if i == SSS_RANDOM { ("RANDOM", "") } else { (STAGES[i].1, STAGES[i].2) };
                    let (cx, cy) = (i % SSS_COLS, i / SSS_COLS);
                    let (x, y) = (16.0 + cx as f32 * (cw + 4.0), 60.0 + cy as f32 * (ch + 4.0));
                    u.rect(x, y, cw, ch, if i == sel { [60, 100, 150, 235] } else { [24, 30, 44, 220] });
                    if i == sel { u.frame(x, y, cw, ch, 3.0, ui::YELLOW) }
                    // two lines of up to 14 characters
                    let mut lines = vec![String::new()];
                    for w in name.split(' ') {
                        let l = lines.last_mut().unwrap();
                        if !l.is_empty() && l.len() + 1 + w.len() > 14 {
                            lines.push(w.to_string());
                        } else {
                            if !l.is_empty() { l.push(' ') }
                            l.push_str(w);
                        }
                    }
                    for (k, l) in lines.iter().take(2).enumerate() {
                        u.text_centered(x + cw / 2.0, y + 10.0 + k as f32 * 12.0, 1.0, l, if note.is_empty() { ui::WHITE } else { ui::GREY });
                    }
                    if !note.is_empty() {
                        u.text_centered(x + cw / 2.0, y + 34.0, 1.0, "*", ui::RED);
                    }
                }
                if sel < STAGES.len() && !STAGES[sel].2.is_empty() {
                    u.text_centered(320.0, 432.0, 1.0, &format!("* KNOWN ISSUE: {}", STAGES[sel].2), ui::RED);
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
        let (slots, focus, rules, row) = (self.slots, self.focus, self.rules, self.rule_row);
        let blocker = self.css_blocker();
        let u = &mut self.ui;
        u.text_centered_shadow(320.0, 8.0, 2.0, "CHARACTER SELECT", ui::WHITE);
        // the rules bar (focus 4)
        let items = [
            ("MODE", if rules.stock { "STOCK".to_string() } else { "TIME".to_string() }),
            (if rules.stock { "STOCKS" } else { "TIME" }, if rules.stock { rules.stocks.to_string() } else if rules.time_secs % 60 == 0 { format!("{} MIN", rules.time_secs / 60) } else { format!("{} SEC", rules.time_secs) }),
            ("ITEMS", ITEM_NAMES[rules.items as usize].to_string()),
            ("TEAMS", if rules.teams { "ON".to_string() } else { "OFF".to_string() }),
        ];
        u.rect(16.0, 28.0, 608.0, 22.0, if focus == 4 { [70, 60, 20, 235] } else { [16, 16, 28, 220] });
        if focus == 4 { u.frame(16.0, 28.0, 608.0, 22.0, 2.0, ui::YELLOW) }
        for (k, (name, val)) in items.iter().enumerate() {
            let x = 24.0 + k as f32 * 150.0;
            let on = focus == 4 && row == k;
            u.text(x, 35.0, 1.0, &format!("{name}: {val}"), if on { ui::YELLOW } else { ui::WHITE });
        }
        // the roster grid
        let (cw, ch) = (64.0, 56.0);
        for (i, (_, name, _, note)) in ROSTER.iter().enumerate() {
            let (cx, cy) = (i % CSS_COLS, i / CSS_COLS);
            let (x, y) = (24.0 + cx as f32 * (cw + 2.0), 58.0 + cy as f32 * (ch + 2.0));
            u.rect(x, y, cw, ch, [26, 30, 46, 230]);
            // the icon label: words on up to two lines of 8 characters (GANONDORF, JIGGLYPUFF: the short names)
            let label = match *name { "GANONDORF" => "GANON", "JIGGLYPUFF" => "PUFF", "C. FALCON" => "FALCON", "MR. G&W" => "G&W", n => n };
            let mut lines: Vec<String> = vec![String::new()];
            for w in label.split(' ') {
                let l = lines.last_mut().unwrap();
                if !l.is_empty() && l.len() + 1 + w.len() > 8 {
                    lines.push(w.chars().take(8).collect());
                } else {
                    if !l.is_empty() { l.push(' ') }
                    l.push_str(w);
                }
            }
            for (k, l) in lines.iter().take(2).enumerate() {
                u.text_centered(x + cw / 2.0, y + 18.0 + 10.0 * k as f32, 1.0, l, if note.is_empty() { ui::WHITE } else { ui::GREY });
            }
            if !note.is_empty() {
                u.text_centered(x + cw / 2.0, y + 42.0, 1.0, "*", ui::RED);
            }
            for (s, sl) in slots.iter().enumerate() {
                if sl.kind == SlotKind::Off { continue }
                let picked = sl.pick == Some(i);
                if picked || (sl.pick.is_none() && sl.cursor == i) {
                    let inset = 1.0 + s as f32 * 3.0;
                    u.frame(x + inset, y + inset, cw - 2.0 * inset, ch - 2.0 * inset, if picked { 3.0 } else { 1.0 }, SLOT_COLS[s]);
                }
            }
        }
        // the four slot panels
        for (s, sl) in slots.iter().enumerate() {
            let (x, y, w, h) = (16.0 + s as f32 * 154.0, 248.0, 146.0, 150.0);
            let col = if rules.teams && sl.kind != SlotKind::Off { [ui::RED, ui::CYAN, ui::GREEN][sl.team as usize] } else { SLOT_COLS[s] };
            u.rect(x, y, w, h, if sl.kind == SlotKind::Off { [10, 10, 16, 200] } else { [16, 16, 28, 230] });
            u.frame(x, y, w, h, if focus == s { 4.0 } else { 2.0 }, if focus == s { ui::WHITE } else { col });
            let kind = match sl.kind { SlotKind::Hmn => "HMN".to_string(), SlotKind::Cpu => format!("CPU LV{}", sl.level), SlotKind::Off => "NA".to_string() };
            u.text(x + 8.0, y + 8.0, 2.0, &format!("P{}", s + 1), col);
            u.text(x + 46.0, y + 12.0, 1.0, &kind, if sl.kind == SlotKind::Off { ui::DIM } else { ui::WHITE });
            if sl.kind == SlotKind::Off {
                u.text(x + 8.0, y + 60.0, 1.0, "A: JOIN", ui::DIM);
                continue;
            }
            let f = sl.pick.unwrap_or(sl.cursor);
            let ready = sl.pick.is_some() || sl.kind == SlotKind::Cpu;
            u.text(x + 8.0, y + 40.0, 1.0, ROSTER[f].1, if ready { ui::WHITE } else { ui::GREY });
            u.text(x + 8.0, y + 56.0, 1.0, if sl.pick.is_some() { "READY" } else if sl.kind == SlotKind::Cpu { "(CURSOR)" } else { "CHOOSING..." }, if ready { ui::GREEN } else { ui::DIM });
            if rules.teams {
                u.text(x + 8.0, y + 76.0, 1.0, &format!("TEAM {}", TEAM_NAMES[sl.team as usize]), col);
            } else {
                u.text(x + 8.0, y + 76.0, 1.0, &format!("COSTUME {}", sl.color + 1), ui::GREY);
            }
            if !ROSTER[f].3.is_empty() {
                // e.g. Jigglypuff: two lines of the note
                let n = ROSTER[f].3;
                let (a, b) = n.split_at(n.find(" (").unwrap_or(n.len()));
                u.text(x + 8.0, y + 100.0, 1.0, a, ui::RED);
                u.text(x + 8.0, y + 112.0, 1.0, b.trim(), ui::RED);
            }
        }
        let hint = if focus == 4 {
            "RULES: UP/DOWN PICK  LEFT/RIGHT CHANGE  B: SLOTS  Z: NEXT"
        } else {
            "A PICK  B BACK  X HMN/CPU/NA  Y COSTUME/TEAM  L/R CPU LV  Z: NEXT"
        };
        u.text(16.0, 410.0, 1.0, hint, ui::GREY);
        match blocker {
            None => u.text(16.0, 428.0, 1.0, "START: STAGE SELECT", ui::YELLOW),
            Some(b) => u.text(16.0, 428.0, 1.0, b, ui::DIM),
        }
    }

    fn draw_results(&mut self) {
        self.backdrop([30, 20, 60, 255], [4, 4, 14, 255]);
        let o = self.outcome.clone();
        let u = &mut self.ui;
        u.text_centered_shadow(320.0, 20.0, 4.0, "RESULTS", ui::WHITE);
        let head = match o.winner.and_then(|w| o.rows[w].as_ref().map(|r| (w, r))) {
            Some((_, r)) if o.teams => format!("{} TEAM WINS", TEAM_NAMES[r.team as usize]),
            Some((_, r)) => format!("{} WINS", r.name),
            None => "NO CONTEST".into(),
        };
        u.text_centered_shadow(320.0, 70.0, 3.0, &head, ui::YELLOW);
        u.text_centered(320.0, 104.0, 1.0, &o.why, ui::GREY);
        for (i, r) in o.rows.iter().enumerate() {
            let x = 16.0 + i as f32 * 154.0;
            let col = SLOT_COLS[i];
            let Some(r) = r else {
                u.rect(x, 130.0, 146.0, 190.0, [10, 10, 16, 180]);
                continue;
            };
            let winner = o.winner.is_some_and(|w| if o.teams { o.rows[w].as_ref().is_some_and(|q| q.team == r.team) } else { w == i });
            u.rect(x, 130.0, 146.0, 190.0, [16, 16, 30, 230]);
            u.frame(x, 130.0, 146.0, 190.0, 3.0, if winner { ui::YELLOW } else { col });
            u.text(x + 10.0, 142.0, 2.0, &format!("P{}", i + 1), col);
            u.text(x + 46.0, 146.0, 1.0, if r.cpu { "CPU" } else { "HMN" }, ui::GREY);
            u.text(x + 10.0, 172.0, 1.0, &r.name, ui::WHITE);
            if o.teams {
                u.text(x + 10.0, 188.0, 1.0, &format!("TEAM {}", TEAM_NAMES[r.team as usize]), ui::GREY);
            }
            u.text(x + 10.0, 210.0, 1.0, &format!("KOS      {}", r.kos), ui::WHITE);
            u.text(x + 10.0, 230.0, 1.0, &format!("FALLS    {}", r.falls), ui::WHITE);
            if o.stock {
                u.text(x + 10.0, 250.0, 1.0, &format!("STOCKS   {}", r.stocks), ui::WHITE);
            }
            u.text(x + 10.0, 270.0, 1.0, &format!("DAMAGE   {}%", r.percent.max(0.0) as i32), ui::WHITE);
        }
        let secs = o.frames.max(0) / 60;
        u.text_centered(320.0, 345.0, 2.0, &format!("TIME  {}:{:02}", secs / 60, secs % 60), ui::GREY);
        u.text(20.0, 452.0, 1.0, "A: CHARACTER SELECT (REMATCH: START THERE)   START: MAIN MENU", ui::GREY);
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
            let packed = self.em.as_mut().and_then(|m| m.packed.take());
            let p = self.presenter.as_mut().unwrap();
            if let Some(pk) = packed {
                p.renderer.set_packed(pk);
            }
            p.set_content_aspect(melee_engine::view::aspect());
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
        crate::prof::gate(true);
        self.tick_scene();
        crate::prof::gate(false);
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
                Ok(p) => {
                    melee_engine::gxbridge::shim().gx.set_pipe_factory(p.renderer.pipe_factory());
                    self.presenter = Some(p)
                }
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
        if let Some(e) = self.presenter.as_ref().and_then(|p| p.renderer.gpu_fault()) {
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

    fn set_hud(&mut self, lines: Option<Vec<melee_platform::overlay::Line>>) {
        if let Some(p) = self.presenter.as_mut() {
            p.hud = lines;
        }
    }
}

/// The window title: `MELEE_WINDOW_TITLE`, else "melee [<worktree> <hash>] <stage> <fighters>" (build identity from build.rs,
/// the scenario from `--match`), so several windows on one desktop can be told apart. The fps readout is appended later.
pub(crate) fn window_title(spec: Option<&str>) -> String {
    if let Ok(t) = std::env::var("MELEE_WINDOW_TITLE") {
        return t;
    }
    let id = format!("melee [{} {}]", env!("MELEE_GIT_WHERE"), env!("MELEE_GIT_HASH"));
    let Some(spec) = spec else { return id };
    // "a/cpu9,b/cpu9,STAGE": fighters without their slot kind, then the stage first
    let parts: Vec<&str> = spec.split(',').map(str::trim).collect();
    let (stage, fighters) = match parts.split_last() {
        Some((s, f)) if !s.contains('/') => (Some(*s), f),
        _ => (None, &parts[..]),
    };
    let f: Vec<&str> = fighters.iter().map(|x| x.split('/').next().unwrap_or(x)).collect();
    match stage {
        Some(s) => format!("{id} {s} {}", f.join(",")),
        None => format!("{id} {}", f.join(",")),
    }
}

const USAGE: &str = "usage: melee play --iso DISC.iso [--time-secs N | --stocks N] [--items OFF|LOW|..] [--teams] [--match a,b/cpu9,..,STAGE] [--quit-after-matches N] [--shots] [--shot-dir DIR] [--quit-at-results] [--max-frames N] [--skip-intro] [--headless] [--no-audio]
       [--widescreen] [--fullscreen[=exclusive]] [--window WxH] [--render-scale N|auto] [--msaa N] [--vsync on|off] [--save-display]";

pub fn play(args: &[String]) -> ExitCode {
    // `--replay FILE`: the file's play options decide the boot (the command line's window options still apply)
    let mut feed = None;
    let merged: Vec<String>;
    // `--spectate HOST:PORT | relay:HOST:PORT/CODE`: a read-only spectator of a netplay match (spectate.rs): the host's
    // confirmed stream is played back like a replay that grows
    let mut spec_client = None;
    let spec_target = args.iter().position(|a| a == "--spectate").and_then(|i| args.get(i + 1)).cloned();
    let replay_src = args.iter().position(|a| a == "--replay").and_then(|i| args.get(i + 1)).cloned().or(spec_target.clone());
    let args: &[String] = match replay_src.as_ref() {
        Some(path) => {
            let r = if let Some(t) = &spec_target {
                let mods = match crate::play_net::mods_identity(args) {
                    Ok(m) => m,
                    Err(e) => {
                        eprintln!("play: --spectate: mods: {e}");
                        return ExitCode::FAILURE;
                    }
                };
                let gv = |k: &str| args.iter().position(|a| a == k).and_then(|i| args.get(i + 1)).cloned();
                match crate::spectate::SpecClient::connect(t, crate::play_net::build_hash(), mods, std::time::Duration::from_secs(gv("--spec-timeout").and_then(|v| v.parse().ok()).unwrap_or(30))) {
                    Ok((mut c, r)) => {
                        c.delay = gv("--spec-delay").and_then(|v| v.parse().ok()).unwrap_or(6);
                        c.catch_up = gv("--spec-catch-up").and_then(|v| v.parse().ok()).unwrap_or(16).max(1);
                        spec_client = Some(c);
                        Ok(r)
                    }
                    Err(e) => Err(e),
                }
            } else {
                std::fs::read(path).map_err(|e| format!("{path}: {e}")).and_then(|d| crate::play_net::Replay::parse(&d))
            };
            let r = match r {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("play: {}: {e}", if spec_target.is_some() { "--spectate" } else { "--replay" });
                    return ExitCode::from(2);
                }
            };
            eprintln!("play: replay {path}: {} ticks, options {:?}, {}", r.ticks.len(), r.args, r.meta.iter().map(|m| format!("{}={}", m.0, m.1)).collect::<Vec<_>>().join(" "));
            let mut a: Vec<String> = Vec::new();
            let mut i = 0;
            while i < args.len() {
                if args[i] == "--replay" || args[i] == "--record" || args[i] == "--netplay" || args[i] == "--spectate" || args[i] == "--np-spectate" || args[i] == "--np-spectate-relay" {
                    i += if args[i] == "--netplay" && args.get(i + 1).map_or(false, |m| m == "join") { 3 } else { 2 };
                    continue;
                }
                a.push(args[i].clone());
                i += 1;
            }
            let mut it = r.args.iter();
            while let Some(x) = it.next() {
                if x == "--match" && r.meta("np_scene").is_some() {
                    it.next(); // a netplay match comes from its agreed scene
                    continue;
                }
                if x == "--mods" && spec_target.is_some() {
                    it.next(); // the host's mods path: the spectator's own --mods (handshake-checked to be the same set) applies
                    continue;
                }
                a.push(x.clone());
            }
            merged = a;
            feed = Some(crate::play_net::ReplayFeed::new(r));
            &merged
        }
        None => args,
    };
    crate::prof::set_gated(std::env::var_os("MELEE_SAMPLE_GATE").map_or(true, |v| v != "0"));
    // `play` never snapshots the game arena (no rollback offline): big free blocks may give their pages back.
    melee_engine::arena::set_trim(std::env::var_os("MELEE_ARENA_NO_TRIM").is_none());
    // images whose raw bytes a later parse overwrote are freed from the first match on (not only after a rematch)
    melee_engine::archive::set_reclaim(std::env::var_os("MELEE_NO_RECLAIM").is_none());
    let get = |k: &str| args.iter().position(|a| a == k).and_then(|i| args.get(i + 1)).cloned();
    let has = |k: &str| args.iter().any(|a| a == k);
    if has("--help") || has("-h") {
        println!("{USAGE}

{}", crate::display::HELP);
        return ExitCode::SUCCESS;
    }
    if let Err(e) = crate::display::setup(args) {
        eprintln!("play: {e}");
        return ExitCode::from(2);
    }
    let Some(iso) = get("--iso").or_else(|| std::env::var("GW_ISO_VANILLA").ok()) else {
        eprintln!("{USAGE}");
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
        rules: Rules {
            stock: get("--stocks").is_some(),
            stocks: get("--stocks").and_then(|s| s.parse().ok()).unwrap_or(4).clamp(1, 99),
            time_secs: get("--time-secs").and_then(|s| s.parse().ok()).unwrap_or(120).max(1),
            stock_time_secs: get("--time-secs").and_then(|s| s.parse::<i32>().ok()).map_or(0, |s| s.max(1)),
            items: get("--items").map_or(0, |v| ITEM_NAMES.iter().position(|n| n.eq_ignore_ascii_case(&v.replace('_', " "))).unwrap_or(3) as i32),
            teams: has("--teams"),
        },
        golden_pads: has("--golden-pads"),
        scene: Scene::Opening,
        age: 0,
        tick_no: 0,
        port: [PortIn::default(); 4],
        title_count: 0,
        main_sel: 1,
        vs_sel: 0,
        toast: None,
        slots: [Slot::new(SlotKind::Hmn, 7, 0), Slot::new(SlotKind::Hmn, 10, 1), Slot::new(SlotKind::Off, 9, 0), Slot::new(SlotKind::Off, 24, 1)],
        focus: 0,
        rule_row: 0,
        sss_sel: 0,
        setup: None,
        em: None,
        spare: None,
        menu_rng: 0x2545_F491,
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
        quit_after: get("--quit-after-matches").and_then(|s| s.parse().ok()).unwrap_or(1),
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
    // `--match falcon,fox[,fox,...][,STAGE]`: skip the menus (a hands-off test of the match path). Fighters as the scene
    // names them (`falcon`, `jigglypuff`, ...; `name/cpuN` for a CPU), the stage by its external id or display name.
    let netplay = has("--netplay");
    if let Some(spec) = get("--match").filter(|_| !netplay) {
        let mut slots = [Slot::new(SlotKind::Off, 0, 0); 4];
        let mut stage = 0;
        let mut n = 0;
        for part in spec.split(',') {
            let (name, kind) = part.split_once('/').unwrap_or((part, ""));
            if let Some(f) = ROSTER.iter().position(|r| r.0 == name) {
                if n < 4 {
                    slots[n] = Slot::new(SlotKind::Hmn, f, (n % 2) as u8);
                    slots[n].pick = Some(f);
                    if let Some(l) = kind.strip_prefix("cpu") {
                        slots[n].kind = SlotKind::Cpu;
                        slots[n].level = l.parse().unwrap_or(9).clamp(1, 9);
                    }
                    n += 1;
                }
            } else if let Some(st) = STAGES.iter().position(|s| s.0.to_string() == part || s.1.eq_ignore_ascii_case(part) || (part == "GrNLa.dat" && s.0 == 32) || (part == "GrNBa.dat" && s.0 == 31)) {
                stage = st;
            } else {
                eprintln!("play: --match: unknown fighter or stage '{part}'");
                return ExitCode::from(2);
            }
        }
        if n < 2 {
            eprintln!("play: --match NAME,NAME[,NAME..][,STAGE]: two fighters or more");
            return ExitCode::from(2);
        }
        game.slots = slots;
        game.setup = Some(MatchSetup { slots, rules: game.rules, stage });
        game.scene = Scene::Loading;
    }
    // The retail scenes (retail_flow.rs) unless MELEE_STANDIN_MENUS=1 keeps the stand-in flow below.
    if std::env::var("MELEE_STANDIN_MENUS").map_or(true, |v| v != "1") {
        let presets = crate::retail_flow::Presets {
            stocks: get("--stocks").and_then(|s| s.parse().ok()),
            time_secs: get("--time-secs").and_then(|s| s.parse().ok()),
            items: get("--items").map(|_| game.rules.items),
            scenario: game.setup.as_ref().map(Play::scenario_of),
        };
        // netplay: connect first (blocking: the room agrees the match), then boot straight into it
        let mut net = None;
        let mut np_scene = feed.as_ref().and_then(|f: &crate::play_net::ReplayFeed| f.r.meta("np_scene").map(str::to_string));
        let mut seed = feed.as_ref().and_then(|f| f.r.meta("seed").and_then(|s| s.parse::<u32>().ok()));
        // the guest plays by the host's rules (sent in the agreed scene): its own --stocks / --time-secs / --items are
        // replaced, for the presets and for the replay's boot options
        let mut eff_args: Vec<String> = args.to_vec();
        let mut np_rules: Option<crate::play_net::HostRules> = None;
        if netplay {
            let ck_of = |n: &str| melee_engine::scenario::ckind_by_name(n);
            let stage_of = |p: &str| STAGES.iter().find(|s| s.0.to_string() == p || s.1.eq_ignore_ascii_case(p) || (p.eq_ignore_ascii_case("fd") && s.0 == 32) || (p.eq_ignore_ascii_case("bf") && s.0 == 31)).map(|s| s.0);
            match crate::play_net::NetDriver::connect(args, &ck_of, &stage_of) {
                Ok(n) => {
                    np_scene = Some(n.scene_string());
                    seed = Some(n.agreed.seed);
                    if !n.host {
                        let r = n.host_rules.clone();
                        let mut a = Vec::new();
                        let mut i = 0;
                        while i < eff_args.len() {
                            if matches!(eff_args[i].as_str(), "--stocks" | "--time-secs" | "--items") {
                                i += 2;
                                continue;
                            }
                            a.push(eff_args[i].clone());
                            i += 1;
                        }
                        if let Some(v) = r.stocks {
                            a.extend(["--stocks".to_string(), v.to_string()]);
                        }
                        if let Some(v) = r.time_secs {
                            a.extend(["--time-secs".to_string(), v.to_string()]);
                        }
                        if let Some(v) = &r.items {
                            a.extend(["--items".to_string(), v.clone()]);
                        }
                        eff_args = a;
                        np_rules = Some(r);
                    }
                    net = Some(n);
                }
                Err(e) => {
                    eprintln!("play: {e}");
                    return ExitCode::FAILURE;
                }
            }
        }
        let sync_k = std::env::var("MELEE_PLAY_SYNCTEST").ok().and_then(|v| v.parse::<i32>().ok()).filter(|&k| k > 0);
        // --replay-seekable / a seek control: the playback snapshots the match (the rollback allocator modes)
        let seekable = feed.is_some() && (has("--replay-seekable") || get("--replay-seek").is_some() || get("--replay-ctl").is_some_and(|c| c.contains("seek") || c.contains("back")));
        // a Lab mod (gd-lab-v1 savestates and rewind) snapshots the engine like rollback does
        let lab_mod = melee_modhost::lab::lab_mod_in_env();
        if has("--np-lab") && !lab_mod {
            eprintln!("play: --np-lab needs a Lab mod in MELEE_WASM_MODS");
            return ExitCode::from(2);
        }
        let rollback_mode = net.is_some() || sync_k.is_some() || seekable || feed.as_ref().map_or(false, |f| f.r.flags & 1 != 0) || lab_mod;
        if rollback_mode {
            crate::play_net::prepare_rollback_run();
            // the GX shim (a static the snapshots restore) with all its buffers and Arcs in the arena from the start
            let _s = melee_engine::arena::GameScope::enter();
            let _ = melee_engine::gxbridge::shim();
        }
        let seed = seed.unwrap_or_else(|| std::env::var("MELEE_TEST_SEED").ok().and_then(|v| v.parse().ok()).unwrap_or(12345));
        std::env::set_var("MELEE_TEST_SEED", seed.to_string());
        let mut presets = presets;
        if let Some(r) = &np_rules {
            presets.stocks = r.stocks;
            presets.time_secs = r.time_secs;
            presets.items = r.items.as_ref().map(|v| ITEM_NAMES.iter().position(|n| n.eq_ignore_ascii_case(&v.replace('_', " "))).unwrap_or(3) as i32);
        }
        // --np-css (or a replay of such a run): no agreed scene, the players pick on the retail character select. Both
        // peers boot straight into VS mode with no memory card (a card's unlocks and settings would differ per machine).
        let np_css = net.as_ref().map_or(false, |n| n.css()) || rp_feed_css(&feed);
        if np_css {
            std::env::set_var("MELEE_RETAIL_FIRST", (melee_types::GM_VS as u8).to_string());
            std::env::set_var("MELEE_CARD", "0");
            np_scene = None;
        }
        if let Some(sc) = &np_scene {
            match crate::play_net::scenario_of_scene(sc) {
                Ok(s) => presets.scenario = Some(s),
                Err(e) => {
                    eprintln!("play: {e}");
                    return ExitCode::FAILURE;
                }
            }
        }
        let quit_after = has("--quit-after-matches").then_some(game.quit_after).or(game.quit_at_results.then_some(1));
        let mut rp = crate::retail_flow::RetailPlay::new(game.iso.clone(), game.mods_dir.clone(), presets, has("--headless"), game.shot_dir.clone(), game.shot_ages.clone(), quit_after, game.max_frames);
        rp.skip_intro = game.skip_intro;
        rp.audio_on = !has("--no-audio") && crate::cfg::flag("audio", true);
        rp.rollback_mode = rollback_mode;
        rp.lab_seed = seed;
        rp.practice = Some(crate::practice::Practice::new(crate::practice::Opts::from_args(args)));
        rp.delay_line = crate::input_delay::DelayLine::from_args(&args);
        if rp.delay_line.frames > 0 {
            log(&format!("offline input delay {} frames (matches only)", rp.delay_line.frames));
        }
        rp.feed = feed;
        rp.spec_client = spec_client;
        if rp.feed.is_some() {
            rp.ctl = crate::replay_ctl::Ctl::from_args(args);
            if let Some(c) = &rp.ctl {
                log(&format!("replay controls: {}{}", c.state(), if rp.rollback_mode { ", seek snapshots on" } else { ", seek forward only (--replay-seekable for snapshots)" }));
            }
        }
        // `--np-spectate PORT [--np-spectate-bind ADDR]` / `--np-spectate-relay HOST:PORT [--np-spectate-code C]`: the
        // match host serves read-only spectators (spectate.rs); the stream is the confirmed replay
        if net.as_ref().is_some_and(|n| n.host) && (get("--np-spectate").is_some() || get("--np-spectate-relay").is_some()) {
            let port: u16 = get("--np-spectate").and_then(|v| v.parse().ok()).unwrap_or(0);
            let bind = get("--np-spectate-bind").unwrap_or_else(|| "127.0.0.1".into());
            let mods = crate::play_net::mods_identity(args).map(|m| m.0).unwrap_or(0);
            match crate::spectate::SpecServer::open(&bind, port, get("--np-spectate-relay").as_deref(), get("--np-spectate-code").as_deref(), crate::play_net::build_hash(), mods) {
                Ok(s) => rp.spec_server = Some(s),
                Err(e) => {
                    eprintln!("play: {e}");
                    return ExitCode::FAILURE;
                }
            }
            if rp.rec.is_none() {
                let mut meta = vec![("seed".to_string(), seed.to_string()), ("build".to_string(), env!("MELEE_GIT_HASH").to_string())];
                if np_css {
                    meta.push(("np_css".into(), "1".into()));
                }
                if let Some(sc) = &np_scene {
                    meta.push(("np_scene".into(), sc.clone()));
                }
                meta.push(("np_role".into(), "host".into()));
                rp.rec = Some((String::new(), crate::play_net::Replay { flags: rollback_mode as u16, args: crate::play_net::replay_args(&eff_args), meta, ticks: Vec::new() }));
            }
        }
        rp.sync = sync_k.map(|k| crate::play_net::PlaySync { st: melee_net::session::SyncTest::new(k), g: None });
        if let Some(path) = get("--record") {
            let path = if path == "auto" {
                let t = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_secs());
                format!("Replays/melee-{t}.mrpl")
            } else {
                path
            };
            let mut meta = vec![("seed".to_string(), seed.to_string()), ("build".to_string(), env!("MELEE_GIT_HASH").to_string())];
            if np_css {
                meta.push(("np_css".into(), "1".into()));
            }
            if let Some(sc) = &np_scene {
                meta.push(("np_scene".into(), sc.clone()));
            }
            if let Some(n) = &net {
                meta.push(("np_role".into(), n.tag().into()));
            }
            let r = crate::play_net::Replay { flags: rollback_mode as u16, args: crate::play_net::replay_args(&eff_args), meta, ticks: Vec::new() };
            rp.rec = Some((path, r));
        }
        // `--slp FILE|auto`: Slippi export. Live only when nothing rolls back; otherwise from the .mrpl playback (play_net).
        if let Some(spec) = get("--slp") {
            if rollback_mode && rp.feed.is_none() {
                if net.is_some() && rp.rec.is_some() {
                    crate::play_net::SLP_AFTER.with(|c| *c.borrow_mut() = Some(spec.clone()));
                } else {
                    eprintln!("play: --slp with netplay or synctest needs --record (the .slp is made from its playback)");
                    return ExitCode::from(2);
                }
            } else {
                // playedOn: readers take only Slippi's values (py-slippi: console, dolphin, network, nintendont)
                let online = net.is_some() || rp.feed.as_ref().map_or(false, |f| f.r.meta("np_role").is_some());
                crate::slp_export::install(&spec, if online { "network" } else { "console" }, ["P1".into(), "P2".into(), "P3".into(), "P4".into()]);
            }
        }
        rp.net = net;
        return crate::retail_flow::run(rp, has("--headless"));
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
    let cfg: WindowConfig = crate::display::current().window_config(window_title(get("--match").as_deref()), !scripted);
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
    // the launcher's own controls, when it started us (launcher_compat.rs)
    crate::launcher_compat::apply_input(&mut pads);
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

fn rp_feed_css(feed: &Option<crate::play_net::ReplayFeed>) -> bool {
    feed.as_ref().map_or(false, |f| f.r.meta("np_css").is_some())
}

/// (working set, private bytes) of this process (`K32GetProcessMemoryInfo`); zeros elsewhere.
#[cfg(windows)]
fn proc_mem() -> (usize, usize) {
    #[repr(C)]
    #[derive(Default)]
    struct Counters {
        cb: u32,
        page_faults: u32,
        peak_ws: usize,
        ws: usize,
        peak_paged: usize,
        paged: usize,
        peak_nonpaged: usize,
        nonpaged: usize,
        pagefile: usize,
        peak_pagefile: usize,
        private: usize,
    }
    extern "system" {
        fn GetCurrentProcess() -> *mut core::ffi::c_void;
        fn K32GetProcessMemoryInfo(p: *mut core::ffi::c_void, c: *mut Counters, cb: u32) -> i32;
    }
    let mut c = Counters { cb: core::mem::size_of::<Counters>() as u32, ..Default::default() };
    unsafe {
        if K32GetProcessMemoryInfo(GetCurrentProcess(), &mut c, c.cb) == 0 {
            return (0, 0);
        }
    }
    (c.ws, c.private)
}
#[cfg(not(windows))]
fn proc_mem() -> (usize, usize) {
    (0, 0)
}

/// The process's memory (working set, private bytes, arena, system heap, caches) as a `mem after` log line, with a
/// heap-sampler section when MELEE_HEAP_SAMPLE is on. Also the retail flow's MELEE_MEM_EVERY line.
pub(crate) fn mem_line(matches: u32, what: &str) {
    let (ws, private) = proc_mem();
    crate::heapprof::dump(&format!("after {matches} matches ({what})"));
    if std::env::var_os("MELEE_ARENA_STATS").is_some() {
        let (bumped, free, top) = melee_engine::arena::stats();
        log(&format!("arena: bumped {} KB, on free lists {} KB, top free classes (size, blocks) {:?}", bumped >> 10, free >> 10, top));
    }
    log(&format!("mem after {} matches ({what}): working set {} KB, private {} KB, arena used {} KB (live {} KB, {} allocations so far), system heap {} KB, gx data cache {:?}, archives (live, retired, raw KB, arena KB) {:?}", matches, ws / 1024, private / 1024, melee_engine::arena::used_range().1 / 1024, melee_engine::arena::live_bytes() / 1024, melee_engine::arena::allocations(), crate::SYS_LIVE.load(std::sync::atomic::Ordering::Relaxed) / 1024, { let (n, b) = melee_engine::gxbridge::data_cache_stats(); (n, b >> 20) }, { let (a, r, raw, c) = melee_engine::archive::stats(); (a, r, raw >> 10, c >> 10) }));
}
