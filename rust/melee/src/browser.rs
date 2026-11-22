//! The in-game browser's host side (gd-ui-v1.2 `UI_HOST`, docs/rust-port/notes/wasm-ui-lab-api.md "A6"): the play flow
//! fills the catalogs the `rust/mods/browser` wasm mod reads (`gd_ui_list_*`) and applies the requests it queues
//! (`gd_ui_request`). Nothing here runs unless a loaded presentation mod declares `UI_HOST`.
//!
//! * Catalogs are rebuilt only at the first present and after a request that changes them (never per frame).
//! * Requests:
//!   - LAUNCH a retail mode (Training, Event Match, 1P, Stadium, Special Melee): offline and only on the main menu, the
//!     same two writes the main menu makes when a mode is picked (`MenuExitData.pending_mode`, `gm_801A4B60`), checked
//!     against `menuguard` first. A menu input path, not a sim one: like any menu choice it changes the flow, so a
//!     `--record` replay of a session that used it does not reproduce (the browser is a menu, never used in a match).
//!   - LAUNCH the LAB / workbench (Geno's GM_LAB match), a replay, online host / join / spectate, and a mod game mode:
//!     these are boot-time modes of `melee play`, so the game relaunches itself with the same options plus the new ones
//!     (`--replay`, `--netplay host|join CODE --np-css`, `--spectate`, `MELEE_SCENE=mode=lab..`) and this run quits.
//!     `MELEE_BROWSER_DRY_RUN=1` logs the relaunch instead (the headless checks). The log never shows the disc path.
//!   - The Modes list has only rows that start and can be left (the settings overlay's SESSION page exits any of them);
//!     MELEE_DEV_MODES=1 adds the dev rows: the workbench (two CPUs, no human port) and the modes that cannot launch.
//!   - MOD_SET: enabled.txt for the next launch (`ModSet::set_enabled` + `save`).
//!   - SKIN_SET: `skin_<slot>=name` in settings.cfg, mounted live offline through `cosmetics::mount` (as the settings
//!     overlay does), else at the next launch.
use super::RetailPlay;

/// The in-process match's mode, waiting for its files (ticks waited).
static PENDING_GOTO: std::sync::Mutex<Option<(u8, u32)>> = std::sync::Mutex::new(None);
use melee_modhost::ui::{self as mui, PresHost};

#[derive(Default)]
pub struct State {
    filled: bool,
    modset: Option<melee_data::modreg::ModSet>,
    replays: Vec<(std::path::PathBuf, &'static str)>,
    /// Catalog ids of the modes, in list order: what LAUNCH does for each.
    modes: Vec<(String, Launch)>,
    /// The SSBM Nucleus engine (gd-ui-v1.3): threads start when the browser's Nucleus view first asks.
    nc: Option<crate::nucleus::Engine>,
    perf: Option<Perf>,
}

/// `MELEE_NUCLEUS_PERF=1`: the game-thread cost of the Nucleus pump and the present-to-present interval, logged every
/// 600 presents (p50 / p99 / max).
struct Perf {
    pump_us: Vec<f64>,
    frame_ms: Vec<f64>,
    last: std::time::Instant,
}

#[derive(Clone)]
enum Launch {
    Retail(u32),
    Scene(&'static str),
    ModMode(String),
    /// The Envoy camp scene (envoy_ui's door, envoy-redesign.md 1.2 B).
    Envoy,
    Unavailable(&'static str),
}

fn rec(fields: &[&str]) -> Vec<u8> {
    fields.join("\0").into_bytes()
}

const RETAIL_MODES: &[(&str, &str, &str, u32, &str)] = {
    use melee_types::*;
    &[
        ("training", "Training", "Training Mode", GM_TRAINING, "Retail training: pick fighters, then set the CPU, items and speed."),
        ("event", "Events", "Event Match", GM_EVENT, "The retail event matches, from the event list."),
        ("classic", "1P", "Classic", GM_CLASSIC, "Twelve stages against the CPU, then Master Hand."),
        ("adventure", "1P", "Adventure", GM_ADVENTURE, "The side-scrolling adventure through the series' worlds."),
        ("allstar", "1P", "All-Star", GM_ALLSTAR, "Every unlocked fighter, one life, heal between rounds."),
        ("target", "Stadium", "Target Test", GM_TARGET_TEST, "Break the ten targets as fast as you can."),
        ("homerun", "Stadium", "Home-Run Contest", GM_HOME_RUN_CONTEST, "Ten seconds to launch the sandbag."),
        ("multiman10", "Stadium", "10-Man Melee", GM_10MAN_VS, "Multi-Man Melee: KO ten wire frames."),
        ("multiman100", "Stadium", "100-Man Melee", GM_100MAN_VS, "Multi-Man Melee: KO a hundred."),
        ("multiman3", "Stadium", "3-Minute Melee", GM_3MIN_VS, "Multi-Man Melee: survive three minutes."),
        ("multiman15", "Stadium", "15-Minute Melee", GM_15MIN_VS, "Multi-Man Melee: survive fifteen minutes."),
        ("endless", "Stadium", "Endless Melee", GM_ENDLESS_VS, "Multi-Man Melee until you fall."),
        ("cruel", "Stadium", "Cruel Melee", GM_CRUEL_VS, "Multi-Man Melee against the cruel wire frames."),
        ("stamina", "Special Melee", "Stamina Mode", GM_STAMINA_VS, "HP instead of percent."),
        ("suddendeath", "Special Melee", "Super Sudden Death", GM_SUPER_SUDDEN_DEATH_VS, "Everyone starts at 300%."),
        ("giant", "Special Melee", "Giant Melee", GM_GIANT_VS, "Everyone is giant."),
        ("tiny", "Special Melee", "Tiny Melee", GM_TINY_VS, "Everyone is tiny."),
        ("invisible", "Special Melee", "Invisible Melee", GM_INVISIBLE_VS, "Fighters you cannot see."),
        ("lightning", "Special Melee", "Lightning Melee", GM_LIGHTNING_VS, "The match at high speed."),
        ("slomo", "Special Melee", "Slo-Mo Melee", GM_SLOMO_VS, "The match in slow motion."),
        ("singlebutton", "Special Melee", "Single-Button Melee", GM_SINGLE_BUTTON_VS, "Only the A button."),
        ("fixedcamera", "Special Melee", "Fixed-Camera Mode", GM_CAMERA_VS, "The camera stays put."),
    ]
};

fn replay_dir() -> std::path::PathBuf {
    std::path::PathBuf::from(std::env::var("MELEE_SLP_DIR").unwrap_or_else(|_| "Replays".into()))
}

fn np_server() -> Option<String> {
    std::env::var("MELEE_NP_SERVER").ok().filter(|s| !s.is_empty()).or_else(|| crate::cfg::text("np_server"))
}

fn status(u: &mut PresHost, code: u8, text: &str) {
    log(text);
    u.set_list(mui::LIST_STATUS, vec![rec(&[&code.to_string(), text])]);
}

fn log(msg: &str) {
    eprintln!("browser: {msg}");
}

fn code_ok(s: &str) -> bool {
    !s.is_empty() && s.len() <= 64 && s.bytes().all(|b| b.is_ascii_alphanumeric() || b".:-_/[]".contains(&b))
}

impl RetailPlay {
    /// After each present: the catalogs on first use, then this present's requests.
    pub(super) fn browser_tick(&mut self) {
        let Some(u) = self.ui.as_mut() else { return };
        if !u.wants_host() {
            return;
        }
        if !self.browser.filled {
            self.browser.filled = true;
            self.browser_fill_all();
        }
        let reqs = self.ui.as_mut().unwrap().take_requests();
        for r in reqs {
            self.browser_request(r.op, &r.arg);
        }
        let t0 = std::time::Instant::now();
        self.nucleus_tick();
        if let Some(p) = self.browser.perf.as_mut() {
            let now = std::time::Instant::now();
            p.pump_us.push((now - t0).as_secs_f64() * 1e6);
            p.frame_ms.push((now - p.last).as_secs_f64() * 1e3);
            p.last = now;
            if p.pump_us.len() >= 600 {
                let pct = |v: &mut Vec<f64>| {
                    v.sort_by(|a, b| a.total_cmp(b));
                    let n = v.len();
                    let r = (v[n / 2], v[n * 99 / 100], v[n - 1]);
                    v.clear();
                    r
                };
                let (a, b, c) = pct(&mut p.pump_us);
                let (d, e, f) = pct(&mut p.frame_ms);
                log(&format!("nucleus perf: game-thread pump p50 {a:.1} p99 {b:.1} max {c:.1} us; frame interval p50 {d:.2} p99 {e:.2} max {f:.2} ms"));
            }
        }
    }

    /// The Nucleus engine's per-present work on the game thread: flags, log lines, pictures, and the lists when the
    /// engine's version moved. Only `try_lock`s: never waits on a worker.
    fn nucleus_tick(&mut self) {
        let offline = self.browser_offline();
        let Some(e) = self.browser.nc.as_mut() else { return };
        e.set_online(!offline);
        for l in e.pump() {
            log(&format!("nucleus: {l}"));
        }
        let u = self.ui.as_mut().unwrap();
        let wants = u.take_pic_wants();
        e.want_pics(&wants);
        for (k, px) in e.ready_pics(2) {
            u.offer_pic(k, px);
        }
        let Some(v) = e.view() else { return };
        let fresh = e.take_installed(&v);
        u.set_list(mui::LIST_NC, v.results);
        u.set_list(mui::LIST_NC_QUEUE, v.queue);
        u.set_list(mui::LIST_NC_STATE, vec![v.state]);
        if fresh {
            // an install finished: the installed mods and the skin catalog show it now (enabled for the next launch)
            self.browser.modset = None;
            self.browser_fill_mods();
            self.browser_fill_modes();
            self.browser_fill_skins();
        }
    }

    fn nucleus(&mut self) -> &mut crate::nucleus::Engine {
        if self.browser.nc.is_none() {
            let dir = self.mods_dir.clone().or_else(|| crate::cfg::text("mods_dir"));
            self.browser.nc = Some(crate::nucleus::Engine::new(dir, self.iso.clone()));
        }
        self.browser.nc.as_mut().unwrap()
    }

    fn browser_fill_all(&mut self) {
        if self.browser.perf.is_none() && std::env::var("MELEE_NUCLEUS_PERF").is_ok_and(|v| v == "1") {
            self.browser.perf = Some(Perf { pump_us: Vec::with_capacity(600), frame_ms: Vec::with_capacity(600), last: std::time::Instant::now() });
        }
        self.browser_fill_mods();
        self.browser_fill_modes();
        self.browser_fill_replays();
        self.browser_fill_skins();
        let srv = np_server();
        let u = self.ui.as_mut().unwrap();
        u.set_list(mui::LIST_ONLINE, vec![rec(&[if srv.is_some() { "1" } else { "0" }, if srv.is_some() { "Room codes via the configured server." } else { "No server set: join and spectate take HOST:PORT." }])]);
        status(u, 0, "ready");
    }

    fn browser_fill_modes(&mut self) {
        let mut v = Vec::new();
        let mut ids = Vec::new();
        // MELEE_DEV_MODES=1 lists the dev rows too: the workbench (two CPUs, no human port: nothing to pause with) and
        // the rows that cannot launch (modes not ported yet, Envoy). A player's list has only what starts and exits.
        let dev = std::env::var("MELEE_DEV_MODES").is_ok_and(|v| v == "1");
        let mut push = |v: &mut Vec<Vec<u8>>, id: &str, group: &str, title: &str, desc: &str, l: Launch| {
            if !dev && (matches!(l, Launch::Unavailable(_)) || id == "workbench") {
                return;
            }
            let avail = !matches!(l, Launch::Unavailable(_));
            let note = match &l {
                Launch::Unavailable(n) => *n,
                Launch::Retail(_) => "Opens from the main menu.",
                Launch::Scene(_) => "Relaunches the game into the match.",
                Launch::ModMode(_) => "Enables the mod and relaunches.",
                Launch::Envoy => "Opens the Envoy camp (also 1-P > Envoy).",
            };
            v.push(rec(&[id, group, title, desc, if avail { "1" } else { "0" }, note]));
            ids.push((id.to_string(), l));
        };
        push(&mut v, "lab", "Training", "LAB", "Geno's LAB: freeze, frame step, rewind, savestates and frame data (Fox vs a Falco CPU on Final Destination).", Launch::Scene("mode=lab;p1=fox/human;p2=falco/cpu1;stage=fd"));
        for &(id, group, title, mode, desc) in RETAIL_MODES {
            let ok = melee_modes::menuguard::KNOWN_WORKING.contains(&mode);
            push(&mut v, id, group, title, desc, if ok { Launch::Retail(mode) } else { Launch::Unavailable("Not ported yet: the menu refuses it too.") });
        }
        push(&mut v, "workbench", "Geno", "Workbench", "Geno's workbench match (GM_LAB with two level-9 CPUs) for watching a define.", Launch::Scene("mode=workbench;p1=kirby/cpu9;p2=fox/cpu9;stage=fd"));
        // the Envoy camp: available while the Envoy presentation mod has registered its door (envoy_ui + envoy_sim loaded)
        let envoy = if unsafe { melee_modes::tu::envoy::DOOR_ON } != 0 { Launch::Envoy } else { Launch::Unavailable("Needs the Envoy mods (envoy_sim, envoy_ui).") };
        // the Envoy camp opens from the main menu's Envoy row only (envoy-v3-ui.md)
    let _ = envoy;
        if let Some(ms) = self.browser.modset.as_ref() {
            for m in &ms.mods {
                let k = m.kind_str().to_ascii_lowercase();
                if k == "mode" || k == "gamemode" || k == "game_mode" {
                    let id = format!("mod:{}", m.id_str());
                    let name = String::from_utf8_lossy(&m.name).into_owned();
                    push(&mut v, &id, "Mods", if name.is_empty() { &id } else { &name }, &String::from_utf8_lossy(&m.desc), Launch::ModMode(m.id_str()));
                }
            }
        }
        self.browser.modes = ids;
        self.ui.as_mut().unwrap().set_list(mui::LIST_MODES, v);
    }

    fn browser_fill_mods(&mut self) {
        if self.browser.modset.is_none() {
            let dir = self.mods_dir.clone().or_else(|| crate::cfg::text("mods_dir"));
            self.browser.modset = dir.map(|d| melee_data::modreg::ModSet::load(std::path::Path::new(&d), false));
        }
        let v = match self.browser.modset.as_ref() {
            Some(ms) => ms
                .mods
                .iter()
                .map(|m| {
                    let name = String::from_utf8_lossy(&m.name).into_owned();
                    rec(&[&m.id_str(), if name.is_empty() { "" } else { &name }, &String::from_utf8_lossy(&m.desc), &m.kind_str(), if m.enabled { "1" } else { "0" }, if m.enabled_boot { "1" } else { "0" }, &m.status_str()])
                })
                .collect(),
            None => Vec::new(),
        };
        self.ui.as_mut().unwrap().set_list(mui::LIST_MODS, v);
    }

    fn browser_fill_replays(&mut self) {
        let list = crate::replays::list(&replay_dir());
        let mut v = Vec::new();
        self.browser.replays.clear();
        for (i, e) in list.iter().take(200).enumerate() {
            let (m, s) = ((e.secs as u64) / 60, (e.secs as u64) % 60);
            let file = e.path.file_name().map_or(String::new(), |f| f.to_string_lossy().into_owned());
            v.push(rec(&[&(i + 1).to_string(), e.kind, &e.date, &e.stage, &e.chars.join(" vs "), &format!("{m}:{s:02}"), &file, &e.note]));
            self.browser.replays.push((e.path.clone(), e.kind));
        }
        self.ui.as_mut().unwrap().set_list(mui::LIST_REPLAYS, v);
    }

    fn browser_fill_skins(&mut self) {
        let skins = crate::cosmetics::list(&crate::cosmetics::dir());
        let pick = crate::cosmetics::chosen(&skins);
        let verdicts = crate::cosmetics::booted().map(|sh| sh.lock().unwrap().verdict.clone()).unwrap_or_default();
        let v = skins
            .iter()
            .map(|s| {
                let sel = pick.get(&s.slot).is_some_and(|n| n.eq_ignore_ascii_case(&s.name));
                let verdict = match (sel, verdicts.get(&s.slot)) {
                    (true, Some(Ok(n))) => format!("mounted: {n} joints match"),
                    (true, Some(Err(e))) => format!("refused: {e}"),
                    _ => String::new(),
                };
                rec(&[&s.slot, crate::cosmetics::slot_file(&s.slot).trim_end_matches(".dat"), &s.name, if sel { "1" } else { "0" }, &verdict])
            })
            .collect();
        self.ui.as_mut().unwrap().set_list(mui::LIST_SKINS, v);
    }

    fn browser_offline(&self) -> bool {
        // not `rollback_mode`: a loaded Lab mod turns that on too (the snapshot allocator), and the LAB is launched from
        // here; a replay / spectate run has a feed or a spectate client
        self.net.is_none() && self.sync.is_none() && self.feed.is_none() && self.spec_client.is_none()
    }

    fn browser_request(&mut self, op: i32, arg: &str) {
        let mut st = |rp: &mut RetailPlay, code: u8, t: &str| status(rp.ui.as_mut().unwrap(), code, t);
        match op {
            mui::REQ_LAUNCH => {
                let Some((_, l)) = self.browser.modes.iter().find(|m| m.0 == arg).cloned() else { return st(self, 1, &format!("unknown mode {arg}")) };
                match l {
                    Launch::Unavailable(why) => st(self, 1, why),
                    Launch::Retail(mode) => {
                        if !self.browser_offline() {
                            return st(self, 1, "Modes start offline only.");
                        }
                        if self.cur.0 as u32 != melee_types::GM_MENU {
                            return st(self, 1, "Open the browser from the main menu to start a mode.");
                        }
                        // the main menu's own mode pick (mnmain.rs: pending_mode, then end the scene)
                        let ok = unsafe {
                            if melee_modes::menuguard::deny(mode as i32) {
                                false
                            } else {
                                let d = melee_modes::tu::gmscene::gm_GetCurrentSceneExitData() as *mut melee_types::MenuExitData;
                                if d.is_null() {
                                    false
                                } else {
                                    (*d).pending_mode = mode as i8;
                                    melee_modes::tu::gmscene::gm_801A4B60();
                                    true
                                }
                            }
                        };
                        if ok { st(self, 2, &format!("starting mode {arg}")) } else { st(self, 1, "That mode cannot start from here.") }
                    }
                    Launch::Scene(scene) => self.browser_relaunch(&[], Some(scene), &format!("mode {arg}")),
                    Launch::Envoy => {
                        if !self.browser_offline() {
                            return st(self, 1, "Envoy starts offline only.");
                        }
                        if self.cur.0 as u32 != melee_types::GM_MENU {
                            return st(self, 1, "Open the browser from the main menu to start Envoy.");
                        }
                        unsafe { melee_modes::tu::envoy::DOOR_OPEN = 1 };
                        st(self, 2, "opening the Envoy camp")
                    }
                    Launch::ModMode(id) => {
                        if let Some(ms) = self.browser.modset.as_mut() {
                            if let Some(i) = ms.find(id.as_bytes()) {
                                ms.set_enabled(i, true);
                                ms.save();
                            }
                        }
                        self.browser_relaunch(&[], None, &format!("mod mode {id}"))
                    }
                }
            }
            mui::REQ_MOD_SET => {
                let (id, on) = arg.split_once('\0').unwrap_or((arg, "1"));
                let on = on == "1";
                let r = self.browser.modset.as_mut().and_then(|ms| ms.find(id.as_bytes()).map(|i| (ms.set_enabled(i, on), ms.save(), ms.restart_needed())));
                match r {
                    Some((_, 0, restart)) => {
                        for l in self.browser.modset.as_mut().unwrap().log.drain(..) {
                            log(&l);
                        }
                        self.browser_fill_mods();
                        self.browser_fill_modes();
                        st(self, 0, &format!("{id} {}: {}", if on { "on" } else { "off" }, if restart { "applies at the next launch" } else { "as at this launch" }));
                    }
                    Some(_) => st(self, 1, "Could not write enabled.txt."),
                    None => st(self, 1, &format!("no mod {id} (mods folder: mods_dir= in settings.cfg)")),
                }
            }
            mui::REQ_REPLAY_PLAY => {
                let i: usize = arg.parse().unwrap_or(0);
                let Some((path, kind)) = self.browser.replays.get(i.wrapping_sub(1)).cloned() else { return st(self, 1, "no such replay") };
                if kind != "mrpl" {
                    return st(self, 1, "Slippi replays play through the sim's player: melee replays --play N.");
                }
                let p = path.to_string_lossy().into_owned();
                self.browser_relaunch(&["--replay".into(), p], None, "replay")
            }
            mui::REQ_SKIN_SET => {
                let (slot, name) = arg.split_once('\0').unwrap_or((arg, "none"));
                let slot = slot.to_ascii_lowercase();
                if crate::cosmetics::slot_of(&format!("{slot}.dat")).is_none() {
                    return st(self, 1, "not a costume slot");
                }
                let path = if name == "none" { None } else { crate::cosmetics::list(&crate::cosmetics::dir()).into_iter().find(|s| s.slot == slot && s.name == name).map(|s| s.path) };
                if name != "none" && path.is_none() {
                    return st(self, 1, "that skin is not in the catalog");
                }
                crate::cfg::open().set_str(&format!("skin_{slot}"), name);
                let note = match crate::cosmetics::booted().filter(|_| self.browser_offline()) {
                    Some(sh) => {
                        let disc = &mut melee_engine::host::Host::get().disc;
                        let r = crate::cosmetics::mount(&sh, &mut |f| disc.read_vanilla(f).ok(), &slot, path.as_deref());
                        disc.forget_replaced(&crate::cosmetics::slot_file(&slot));
                        match (&r, path.is_some()) {
                            (_, false) => "vanilla from the next match".to_string(),
                            (Ok(n), _) => format!("mounted ({n} joints match): next match"),
                            (Err(e), _) => format!("refused: {e}"),
                        }
                    }
                    None => "saved: applies at the next launch".into(),
                };
                self.browser_fill_skins();
                st(self, 0, &format!("{} -> {name}: {note}", crate::cosmetics::slot_file(&slot)));
            }
            mui::REQ_NET_HOST => self.browser_relaunch(&["--netplay".into(), "host".into(), "--np-css".into()], None, "online host"),
            mui::REQ_NET_JOIN => {
                if !code_ok(arg) {
                    return st(self, 1, "Type the room code (or HOST:PORT).");
                }
                self.browser_relaunch(&["--netplay".into(), "join".into(), arg.into(), "--np-css".into()], None, "online join")
            }
            mui::REQ_NET_SPECTATE => {
                if !code_ok(arg) {
                    return st(self, 1, "Type the spectate code (or HOST:PORT).");
                }
                let target = if arg.contains(':') { arg.to_string() } else if let Some(s) = np_server() { format!("relay:{s}/{arg}") } else { return st(self, 1, "A spectate code needs the server (MELEE_NP_SERVER); or type HOST:PORT.") };
                self.browser_relaunch(&["--spectate".into(), target], None, "spectate")
            }
            mui::REQ_NC_SEARCH => {
                if !self.browser_offline() {
                    return st(self, 1, "SSBM Nucleus is offline-only: finish the online session first.");
                }
                let mut f = arg.split('\0');
                let text: String = f.next().unwrap_or("").chars().take(47).collect();
                let tab = f.next().and_then(|t| t.parse::<usize>().ok()).filter(|&t| t < crate::nucleus::catalog::TAB_LABEL.len());
                let sort = f.next().and_then(|t| t.parse::<usize>().ok()).filter(|&t| t < crate::nucleus::catalog::SORT_LABEL.len()).unwrap_or(0);
                let e = self.nucleus();
                e.open();
                e.search(crate::nucleus::catalog::Query { text: text.clone(), tab, sort, ..Default::default() });
                log(&format!("nucleus: search '{text}' tab {} sort {}", tab.map_or("all", |t| crate::nucleus::catalog::TAB_LABEL[t]), crate::nucleus::catalog::SORT_LABEL[sort]));
            }
            mui::REQ_NC_QUEUE => {
                let Ok(id) = arg.parse::<i64>() else { return st(self, 1, "no such mod") };
                if !self.browser_offline() {
                    return st(self, 1, "Mods cannot be installed while you are online.");
                }
                let m = self.nucleus().toggle(id);
                st(self, 0, &m);
            }
            mui::REQ_NC_PROCESS => {
                if !self.browser_offline() {
                    return st(self, 1, "Mods cannot be installed while you are online.");
                }
                let e = self.nucleus();
                e.open();
                let m = e.set_processing(arg == "1");
                st(self, 0, &m);
            }
            mui::REQ_NC_SYNC => {
                if !self.browser_offline() {
                    return st(self, 1, "SSBM Nucleus is offline-only.");
                }
                let e = self.nucleus();
                e.open();
                e.refresh();
                st(self, 0, "Checking SSBM Nucleus for changes.");
            }
            mui::REQ_REFRESH => {
                match arg {
                    "1" => self.browser_fill_mods(),
                    "2" => self.browser_fill_replays(),
                    "3" => self.browser_fill_skins(),
                    _ => {
                        self.browser.modset = None;
                        return self.browser_fill_all();
                    }
                }
                st(self, 0, "refreshed");
            }
            _ => {}
        }
    }

    /// Relaunch `melee play` with this run's options (minus the boot-mode ones) plus `extra`, then quit this run.
    fn browser_relaunch(&mut self, extra: &[String], scene: Option<&str>, what: &str) {
        if !self.browser_offline() {
            return self.ui_status(1, "Finish the online session or replay first.");
        }
        self.relaunch(extra, scene, false, what);
    }

    pub(super) fn ui_status(&mut self, code: u8, text: &str) {
        match self.ui.as_mut() {
            Some(u) => status(u, code, text),
            None => log(text),
        }
    }

    /// The settings overlay's SESSION exits (after the step, so never inside a scene's frame). "Quit game" ends the
    /// run; "Return to main menu" switches the scene in place (`Engine::goto_mode`) in an ordinary offline run, and
    /// relaunches to the menus (MELEE_BOOT_MENU=1, no boot-mode options, no MELEE_SCENE) from anything a scene launch,
    /// a browser relaunch, a replay, netplay, a spectate or a recording set up: those carry run-wide state (the LAB's
    /// rollback mode, a feed, a session) that a scene switch cannot undo.
    /// The Slippi online submenu's pick (`melee_modes::tu::slippi_online`, plan step 8): hand the run to our Slippi
    /// netplay client, paired through matchmaking (`melee slippi-netplay --mm`). The matchmaking URL must be a local
    /// server (the client refuses any other without `--mm-real`, which this path never passes). The account is a local
    /// test identity from MELEE_SLIPPI_TEST_CODE: no Slippi user.json is read here. Until the connect-code screen
    /// (step 7) is ported, Direct's opponent code comes from MELEE_SLIPPI_DIRECT_CODE.
    pub(super) fn slippi_online_pick(&mut self) {
        const SLIPPI_MM_REAL: &str = "wss://matchmaking.slippi.gg/v1/queue";
        // a set's next game: the match process relaunched this window into the menus; on to the online select
        static REMATCH_DONE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
        if !REMATCH_DONE.load(std::sync::atomic::Ordering::Relaxed) && self.e.is_some() && self.cur.0 as u32 == melee_types::GM_MENU && melee_modes::tu::slippi_online::enabled() {
            REMATCH_DONE.store(true, std::sync::atomic::Ordering::Relaxed);
            if crate::slippi_css_host::start_rematch().is_some() {
                log("online select: the set goes on, back to the online select");
                self.e.as_mut().unwrap().goto_mode(melee_types::GM_HANYU_CSS as u8);
                return;
            }
        }
        // after an Unranked game: the match process relaunched this window into the menus; the same mode's online
        // select, idle (Slippi / Hero: the results, then the CSS, where Start searches again)
        static BACK_DONE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
        if !BACK_DONE.load(std::sync::atomic::Ordering::Relaxed) && self.e.is_some() && self.cur.0 as u32 == melee_types::GM_MENU && melee_modes::tu::slippi_online::enabled() {
            BACK_DONE.store(true, std::sync::atomic::Ordering::Relaxed);
            let back = std::env::var("MELEE_SLIPPI_BACK").ok().and_then(|v| v.parse::<i32>().ok());
            std::env::remove_var("MELEE_SLIPPI_BACK");
            if let Some(mode) = back.filter(|&m| m != melee_modes::tu::slippi_online::MODE_DIRECT && melee_modes::tu::slippi_online::row_unlocked(melee_modes::tu::slippi_online::row_of(m) as i32)) {
                log(&format!("online select: back from the match, {} again (Start searches)", melee_modes::tu::slippi_online::mode_name(mode)));
                melee_modes::tu::slippi_online::repick(mode);
                melee_modes::tu::slippi_css::begin(mode, "");
                self.e.as_mut().unwrap().goto_mode(melee_types::GM_HANYU_CSS as u8);
            }
        }
        let Some(mode) = melee_modes::tu::slippi_online::take_pick() else { return };
        let name = melee_modes::tu::slippi_online::mode_name(mode);
        let typed = melee_modes::tu::slippi_online::take_direct_code();
        let env = |k: &str| if k == "MELEE_SLIPPI_DIRECT_CODE" && typed.is_some() { typed.clone() } else { std::env::var(k).ok().filter(|v| !v.is_empty()) };
        // Slippi's real matchmaking only with a login the YouGame app granted (YOUGAME_SLIPPI_USER_JSON: the path of
        // the player's Slippi sign-in file, Hero's `account::app_sign_in`), never in an agent or scripted run (hidden
        // window, pad script, or a local mock named). Its fields are read by `establish` and never printed.
        let agent = std::env::var_os("MELEE_PAD_SCRIPT").is_some() || env("MELEE_WINDOW_HIDDEN").is_some_and(|v| v == "1") || env("MELEE_SLIPPI_MM_URL").is_some();
        if let Some(grant) = env("YOUGAME_SLIPPI_USER_JSON").filter(|_| !agent) {
            let port = env("YOUGAME_SLIPPI_UDP_PORT").unwrap_or_else(|| "51441".into());
            let mut args: Vec<String> = ["--slippi-mm", SLIPPI_MM_REAL, "--slippi-mm-real", "--slippi-user-json", &grant, "--slippi-port", &port].iter().map(|s| s.to_string()).collect();
            match mode {
                melee_modes::tu::slippi_online::MODE_DIRECT => match env("MELEE_SLIPPI_DIRECT_CODE") {
                    Some(theirs) => args.extend(["--slippi-mm-mode".into(), "direct".into(), "--slippi-search".into(), theirs]),
                    None => return self.ui_status(1, "online menu: Direct needs the code screen (not ported yet)"),
                },
                melee_modes::tu::slippi_online::MODE_UNRANKED => {
                    args.extend(["--slippi-mm-mode".into(), "unranked".into()]);
                    // Hero's yougame_mode_policy: in YouGame mode Unranked with a Slippi login searches both pools
                    if melee_platform::yougame::active() && melee_platform::yougame_mm::matchmaking_route(mode, true, true, false, false, true) == melee_platform::yougame_mm::Route::Blend {
                        args.extend(["--slippi-route".into(), "blend".into()]);
                    }
                }
                _ => return self.ui_status(1, &format!("online menu: {name} is not offered yet")),
            }
            log(&format!("online menu: {name}: Slippi matchmaking with the granted login, the online select meanwhile"));
            return crate::slippi_css_host::start(args, mode);
        }
        // Hero's yougame_mode_policy (melee_platform::yougame_mm::matchmaking_route): in YouGame mode without a granted
        // login Direct / Teams / Party are Locked, and Unranked goes to YouGame's own queue (mpTicket through the app
        // bridge, on a thread: the answer can take 30 s). Never Slippi's servers from here.
        // Tests: MELEE_YOUGAME_TEST_SLIPPI_LOGIN=1 stands in for a granted login, with the local Slippi mock
        // (MELEE_SLIPPI_MM_URL; loopback only) as Slippi's pool: an agent run may then take the YouGame route too, whose
        // rooms URL must also be loopback (yougame_search::target).
        let test_login = env("MELEE_YOUGAME_TEST_SLIPPI_LOGIN").is_some_and(|v| v == "1") && env("MELEE_SLIPPI_MM_URL").is_some();
        let yg_route = if melee_platform::yougame::active() && (!agent || test_login) {
            use melee_platform::yougame_mm as ym;
            let route = ym::matchmaking_route(mode, true, test_login, false, false, true);
            if route == ym::Route::Locked || (route == ym::Route::Slippi && !test_login) {
                return self.ui_status(1, &format!("online menu: {name} needs your Slippi login (allow it on the game's page)"));
            }
            log(&format!("online menu: {name}: {:?} ({})", route, if route == ym::Route::Blend { ym::SEARCH_BOTH } else { ym::SEARCH_YOUGAME_ONLY }));
            Some(match route {
                ym::Route::Blend => "blend",
                ym::Route::YouGame => "yougame",
                _ => "slippi",
            })
        } else {
            None
        };
        if let (Some(r), None) = (yg_route, env("MELEE_SLIPPI_MM_URL")) {
            // YouGame's rooms alone: no Slippi pool to name
            let port = env("YOUGAME_SLIPPI_UDP_PORT").or_else(|| env("MELEE_SLIPPI_NP_PORT")).unwrap_or_else(|| "51441".into());
            let args: Vec<String> = ["--slippi-port", &port, "--slippi-route", r].iter().map(|s| s.to_string()).collect();
            return crate::slippi_css_host::start(args, mode);
        }
        let (Some(url), Some(port), Some(code)) = (env("MELEE_SLIPPI_MM_URL"), env("MELEE_SLIPPI_NP_PORT"), env("MELEE_SLIPPI_TEST_CODE")) else {
            return self.ui_status(1, &format!("online menu: {name} needs a matchmaking server (MELEE_SLIPPI_MM_URL, MELEE_SLIPPI_NP_PORT, MELEE_SLIPPI_TEST_CODE)"));
        };
        // the match runs in this game's window (play --slippi-mm: renderer, audio, pads), over the Slippi wire
        let mut args: Vec<String> = ["--slippi-mm", &url, "--slippi-port", &port, "--slippi-code", &code].iter().map(|s| s.to_string()).collect();
        match mode {
            melee_modes::tu::slippi_online::MODE_DIRECT => {
                let Some(theirs) = env("MELEE_SLIPPI_DIRECT_CODE") else {
                    return self.ui_status(1, "online menu: Direct needs the opponent's code (MELEE_SLIPPI_DIRECT_CODE until the code screen is ported)");
                };
                args.extend(["--slippi-mm-mode".into(), "direct".into(), "--slippi-search".into(), theirs]);
            }
            // unranked: a loopback matchmaking server only (the client refuses any other address without --mm-real)
            _ => args.extend(["--slippi-mm-mode".into(), "unranked".into()]),
        }
        if let Some(v) = env("MELEE_SLIPPI_NP_MAX_SECONDS") {
            args.extend(["--slippi-max-seconds".to_string(), v]);
        }
        if let Some(r) = yg_route {
            args.extend(["--slippi-route".into(), r.into()]);
        }
        log(&format!("online menu: {name}: the online select as {code}; Start with a fighter chosen searches"));
        crate::slippi_css_host::start(args, mode);
    }

    /// The online select was left with both fighters locked in (`slippi_css::take_go`): the match runs in this game's
    /// window (play --slippi-peer: renderer, audio, pads), against the opponent the select's session linked to.
    pub(super) fn slippi_online_match(&mut self) {
        // the in-process match's scene change, once its files are read (at most half a second)
        let due = PENDING_GOTO.lock().ok().and_then(|mut g| {
            let (mode, n) = g.as_mut()?;
            *n += 1;
            let m = *mode;
            (!melee_engine::asset::hints_busy() || *n >= 30).then(|| (g.take(), m).1)
        });
        if let Some(mode) = due {
            if let Some(e) = self.e.as_mut() {
                e.goto_mode(mode);
            }
        }
        // the select can leave a moment before the session has closed its link: wait for the handoff
        static WAITING: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
        use std::sync::atomic::Ordering::Relaxed;
        if melee_modes::tu::slippi_css::take_go() {
            WAITING.store(true, Relaxed);
        }
        if !WAITING.load(Relaxed) {
            return;
        }
        if melee_modes::tu::slippi_css::with(|o| o.conn == melee_modes::tu::slippi_css::CONN_ERROR) {
            WAITING.store(false, Relaxed);
            return self.ui_status(1, "online select: the opponent's link failed before the match");
        }
        let Some(mut args) = crate::slippi_css_host::take_handoff() else { return };
        WAITING.store(false, Relaxed);
        // test hooks: the match length, the per-frame checksum file, a hard time limit, generated pads
        for (k, e) in [("--slippi-frames", "MELEE_SLIPPI_NP_FRAMES"), ("--slippi-out", "MELEE_SLIPPI_NP_OUT"), ("--slippi-max-seconds", "MELEE_SLIPPI_NP_MAX_SECONDS"), ("--slippi-padgen", "MELEE_SLIPPI_NP_PADGEN")] {
            if let Ok(v) = std::env::var(e) {
                if !v.is_empty() {
                    args.extend([k.to_string(), v]);
                }
            }
        }
        // a set's later games write their own checksum file (<out>-g<n>.csv)
        let game = crate::slippi_css_host::handoff_game();
        if game > 0 {
            if let Some(i) = args.iter().position(|a| a == "--slippi-out") {
                if let Some(p) = args.get_mut(i + 1) {
                    let pb = std::path::PathBuf::from(&*p);
                    let stem = pb.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
                    *p = pb.with_file_name(format!("{stem}-g{}.csv", game + 1)).to_string_lossy().into_owned();
                }
            }
        }
        if crate::slippi_link::inprocess() && !crate::room_keeper::active() {
            return self.slippi_match_inprocess(args);
        }
        log("online select: both locked in, the match in the game window");
        if crate::room_keeper::active() {
            // a YouGame room: this process keeps it through the match, the match process reports to it
            let report = crate::room_keeper::report_path();
            let _ = std::fs::remove_file(&report);
            self.room_report = Some(report.clone());
            if let Some(child) = self.relaunch(&args, None, false, "the Slippi match") {
                crate::room_keeper::attach(child, report);
            } else {
                crate::room_keeper::close();
            }
            self.room_report = None;
            return;
        }
        self.relaunch(&args, None, false, "the Slippi match");
    }

    /// The match in this process and window (`slippi_link::inprocess`): the select's link is taken over here (from this
    /// process's own keeper, `slippi_link::take`), the match scene is installed and the engine goes to it, as a scene
    /// change; the session takes the engine on the match's first frame (`step`, `arm`). Nothing closes or reopens.
    fn slippi_match_inprocess(&mut self, args: Vec<String>) {
        let t0 = std::time::Instant::now();
        crate::slippi_link::save_offline_codes();
        match crate::play_net::NetDriver::connect_slippi(&args) {
            Ok((n, scn)) => {
                log(&format!("online select: both locked in, the match in this window (no relaunch; link taken in {:.1} ms)", t0.elapsed().as_secs_f64() * 1e3));
                self.net = Some(n);
                // the match process had no pad script (generated pads, MELEE_SLIPPI_NP_PADGEN, in tests) and no frame limit
                super::set_pad_swap(None);
                super::transition_begin("select->match");
                self.max_frames = None;
                let mode = melee_engine::scenario::boot_mode(&scn);
                // the match's files to the asset worker first (the relaunched process had them at its boot): the
                // select stays up a few ticks while they are read, so the match's first tick applies ready plans
                let picks: Vec<(i32, u8)> = scn.players.iter().flatten().map(|p| (p.ckind, p.color.max(0) as u8)).collect();
                melee_engine::asset::hint_vs_match(&picks, 0);
                melee_engine::scenario::install(scn);
                *PENDING_GOTO.lock().unwrap() = Some((mode, 0));
            }
            Err(e) => {
                crate::slippi_link::restore_offline_codes();
                crate::play_net::clear_name_tags();
                crate::slippi_css_host::reset_for_select();
                self.ui_status(1, &format!("online select: the match could not start: {e}"));
                melee_modes::tu::slippi_css::with(|o| {
                    o.conn = melee_modes::tu::slippi_css::CONN_ERROR;
                    o.error = e;
                });
            }
        }
    }

    /// The online match of this process is over (`slippi_link::inprocess`): the session, its codes, name tags and
    /// scene launch go; the menus are the offline ones again.
    fn slippi_match_cleanup(&mut self) {
        self.net = None;
        melee_core::slippi_online::disable();
        crate::slippi_link::restore_offline_codes();
        crate::play_net::clear_name_tags();
        melee_engine::scenario::uninstall();
        crate::slippi_css_host::reset_for_select();
        std::env::remove_var("MELEE_SLIPPI_SET");
        std::env::remove_var("MELEE_SLIPPI_AFTER");
        std::env::remove_var("MELEE_SLIPPI_TAGS");
    }

    /// Back to the online select in this window (`slippi_relaunch_select`'s in-process form): `kv` is the environment
    /// the relaunched menus would have read (`MELEE_SLIPPI_REMATCH`: the set's next game, `MELEE_SLIPPI_BACK`: the
    /// mode's select, idle).
    pub(super) fn slippi_back_inprocess(&mut self, kv: (&str, &str), pad: Option<String>) -> bool {
        self.slippi_match_cleanup();
        super::set_pad_swap(pad);
        super::transition_begin("match->select");
        if let Ok(n) = std::env::var("MELEE_SLIPPI_SET_MAX_FRAMES").map(|v| v.parse::<u64>()) {
            if let Ok(n) = n {
                self.max_frames = Some(self.tick_no as u64 + n);
            }
        }
        let Some(e) = self.e.as_mut() else { return false };
        if kv.0 == "MELEE_SLIPPI_REMATCH" {
            std::env::set_var(kv.0, kv.1);
            if crate::slippi_css_host::start_rematch().is_none() {
                return false;
            }
            log("online select: the set goes on, back to the online select (this window)");
            e.goto_mode(melee_types::GM_HANYU_CSS as u8);
            return true;
        }
        let Ok(mode) = kv.1.parse::<i32>() else { return false };
        log(&format!("online select: back from the match, {} again (Start searches; this window)", melee_modes::tu::slippi_online::mode_name(mode)));
        melee_modes::tu::slippi_online::repick(mode);
        melee_modes::tu::slippi_css::begin(mode, "");
        e.goto_mode(melee_types::GM_HANYU_CSS as u8);
        true
    }

    /// After a game of an online-select set (Hero: the select comes back, connected, for the next game; the loser
    /// picks its stage): relaunch the window into the menus' online select with the same opponent. False when this
    /// match is not part of one (the window quits as before). `link`: the match's link (its game ended, kept open by
    /// `play_net` when `slippi_css_host::set_goes_on`), kept for the select to take over (`slippi_link::hand_back`).
    pub(super) fn slippi_set_back(&mut self, link: Option<melee_net::slippi::Client>) -> bool {
        let cur: Vec<String> = std::env::args().skip(1).collect();
        let won = crate::play_net::LAST_WINNER.load(std::sync::atomic::Ordering::Relaxed);
        if crate::slippi_css_host::rematch_env(&cur, won, "").is_none() {
            return false;
        }
        if !crate::slippi_css_host::set_goes_on() {
            log("online select: the set's games are played");
            return false;
        }
        // the one link goes on into the next game's select (Slippi Dolphin takes no second connection)
        let kept = match link.map(crate::slippi_link::hand_back) {
            Some(Ok(spec)) => spec,
            Some(Err(e)) => {
                log(&format!("online select: the set's link could not be kept: {e}"));
                return false;
            }
            None => String::new(),
        };
        let Some(env) = crate::slippi_css_host::rematch_env(&cur, won, &kept) else { return false };
        let game: u32 = env.split(';').nth(7).and_then(|g| g.parse().ok()).unwrap_or(1);
        let pad = std::env::var(format!("MELEE_SLIPPI_SET_PAD_SCRIPT_G{}", game + 1)).ok();
        if self.slippi_relaunch_select(("MELEE_SLIPPI_REMATCH", &env), pad, &format!("select-g{}", game + 1)) {
            log(&format!("online select: game {game} over (winner port {won}), back to the select for game {}", game + 1));
            true
        } else {
            false
        }
    }

    /// After an Unranked game (its results shown): relaunch the window into the menus' online select of the same mode,
    /// idle; Start searches again (Slippi Dolphin and Hero: the results, then the CSS). False when this match did not
    /// come from the online select (`MELEE_SLIPPI_AFTER`, set at its handoff).
    pub(super) fn slippi_after_back(&mut self) -> bool {
        let Some(mode) = std::env::var("MELEE_SLIPPI_AFTER").ok().filter(|v| !v.is_empty()) else { return false };
        // tests: the select's pad script after the match
        let pad = std::env::var("MELEE_SLIPPI_BACK_PAD_SCRIPT").ok();
        if self.slippi_relaunch_select(("MELEE_SLIPPI_BACK", &mode), pad, "select") {
            log("online select: the results are over, back to the online select");
            true
        } else {
            false
        }
    }

    /// Relaunch the window into the menus (`MELEE_BOOT_MENU=1`) with `kv` set, the match's own arguments dropped.
    fn slippi_relaunch_select(&mut self, kv: (&str, &str), pad: Option<String>, title: &str) -> bool {
        if crate::slippi_link::inprocess() {
            return self.slippi_back_inprocess(kv, pad);
        }
        let cur: Vec<String> = std::env::args().skip(1).collect();
        // the match's own arguments go; the menus' stay (`--slippi-menus on`)
        let mut args: Vec<String> = Vec::new();
        let mut i = 0;
        while i < cur.len() {
            let a = cur[i].as_str();
            if a == "--slippi-host" || a == "--slippi-stage-unset" {
                i += 1;
                continue;
            }
            if a.starts_with("--slippi-") && a != "--slippi-menus" || a == "--max-frames" {
                i += 2;
                continue;
            }
            args.push(cur[i].clone());
            i += 1;
        }
        if args.first().is_none_or(|a| a != "play") {
            args.insert(0, "play".into());
        }
        if let Ok(n) = std::env::var("MELEE_SLIPPI_SET_MAX_FRAMES") {
            args.extend(["--max-frames".into(), n]);
        }
        let Ok(exe) = std::env::current_exe() else { return false };
        let mut cmd = std::process::Command::new(exe);
        cmd.args(&args).env("MELEE_BOOT_MENU", "1").env(kv.0, kv.1).env_remove("MELEE_SCENE").env_remove("MELEE_SLIPPI_SET").env_remove("MELEE_SLIPPI_AFTER");
        match pad {
            Some(p) => cmd.env("MELEE_PAD_SCRIPT", p),
            None => cmd.env_remove("MELEE_PAD_SCRIPT"),
        };
        if let Ok(t) = std::env::var("MELEE_WINDOW_TITLE") {
            cmd.env("MELEE_WINDOW_TITLE", format!("{}:{title}", t.split(":select").next().unwrap_or(&t)));
        }
        match cmd.spawn() {
            Ok(_) => {
                self.quit = true;
                true
            }
            Err(e) => {
                log(&format!("online select: relaunch failed: {e}"));
                false
            }
        }
    }

    pub(super) fn session_exit(&mut self) {
        if self.boot_menu && self.e.is_some() && matches!(self.cur.0 as u32, melee_types::GM_TITLE | melee_types::GM_OPENING_MV) {
            self.boot_menu = false;
            log("session: MELEE_BOOT_MENU: the boot's title -> main menu");
            self.e.as_mut().unwrap().goto_mode(melee_types::GM_MENU as u8);
        }
        let Some(menu) = self.exit_req.take() else { return };
        if !menu {
            log("session: quit game (settings overlay)");
            self.quit = true;
            return;
        }
        let plain = self.browser_offline() && !self.lab_launch && self.rec.is_none() && std::env::var_os("MELEE_SCENE").is_none() && self.e.is_some();
        if plain {
            log(&format!("session: return to main menu (scene switch from mode {} scene {})", self.cur.0, self.cur.1));
            if let Some(a) = self.audio.as_mut() {
                a.pause(false);
                a.match_leave();
            }
            self.e.as_mut().unwrap().goto_mode(melee_types::GM_MENU as u8);
            return;
        }
        log("session: return to main menu (relaunch to the menus)");
        self.relaunch(&[], None, true, "the main menu");
    }

    fn relaunch(&mut self, extra: &[String], scene: Option<&str>, to_menu: bool, what: &str) -> Option<std::process::Child> {
        let mut args: Vec<String> = Vec::new();
        let cur: Vec<String> = std::env::args().skip(1).collect();
        let mut i = 0;
        while i < cur.len() {
            let a = cur[i].as_str();
            let skip = match a {
                "--netplay" => {
                    if cur.get(i + 1).is_some_and(|m| m == "join") { 3 } else { 2 }
                }
                "--replay" | "--spectate" | "--max-frames" | "--quit-after-matches" | "--record" => 2,
                "--np-css" | "--np-lab" | "--quit-at-results" => 1,
                _ => 0,
            };
            if skip > 0 {
                i += skip;
                continue;
            }
            args.push(cur[i].clone());
            i += 1;
        }
        if args.first().is_none_or(|a| a != "play") {
            args.insert(0, "play".into());
        }
        args.extend_from_slice(extra);
        // the log never shows the disc path (or any path after --iso)
        let shown: Vec<&str> = args.iter().enumerate().map(|(k, a)| if k > 0 && args[k - 1] == "--iso" { "<iso>" } else if extra.iter().any(|e| e == a) && a.contains(['/', '\\']) { "<file>" } else { a.as_str() }).collect();
        let env = scene.map(|s| format!(" MELEE_SCENE={s}")).unwrap_or_default();
        if std::env::var("MELEE_BROWSER_DRY_RUN").is_ok_and(|v| v == "1") {
            self.ui_status(0, &format!("relaunch (dry run) for {what}:{env} melee {}", shown.join(" ")));
            if to_menu {
                self.quit = true;
            }
            return None;
        }
        let Ok(exe) = std::env::current_exe() else {
            self.ui_status(1, "cannot find the game executable");
            return None;
        };
        let mut cmd = std::process::Command::new(exe);
        cmd.args(&args).env_remove("MELEE_PAD_SCRIPT");
        if to_menu {
            cmd.env("MELEE_BOOT_MENU", "1");
        } else {
            cmd.env_remove("MELEE_BOOT_MENU");
        }
        match scene {
            Some(s) => cmd.env("MELEE_SCENE", s),
            None => cmd.env_remove("MELEE_SCENE"),
        };
        if let Some(r) = &self.room_report {
            cmd.env("MELEE_ROOM_REPORT", r);
        }
        match cmd.spawn() {
            Ok(c) => {
                self.ui_status(2, &format!("relaunching for {what}:{env} melee {}", shown.join(" ")));
                self.quit = true;
                Some(c)
            }
            Err(e) => {
                self.ui_status(1, &format!("relaunch failed: {e}"));
                if to_menu {
                    self.quit = true;
                }
                None
            }
        }
    }
}
