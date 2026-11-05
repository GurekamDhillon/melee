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
use melee_modhost::ui::{self as mui, PresHost};

#[derive(Default)]
pub struct State {
    filled: bool,
    modset: Option<melee_data::modreg::ModSet>,
    replays: Vec<(std::path::PathBuf, &'static str)>,
    /// Catalog ids of the modes, in list order: what LAUNCH does for each.
    modes: Vec<(String, Launch)>,
}

#[derive(Clone)]
enum Launch {
    Retail(u32),
    Scene(&'static str),
    ModMode(String),
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
    }

    fn browser_fill_all(&mut self) {
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
        push(&mut v, "envoy", "Online", "Envoy", "Best-of sets with an Envoy build per game. Needs the Envoy mod.", Launch::Unavailable("Envoy is not in this build yet."));
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

    fn ui_status(&mut self, code: u8, text: &str) {
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

    fn relaunch(&mut self, extra: &[String], scene: Option<&str>, to_menu: bool, what: &str) {
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
            return;
        }
        let Ok(exe) = std::env::current_exe() else { return self.ui_status(1, "cannot find the game executable") };
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
        match cmd.spawn() {
            Ok(_) => {
                self.ui_status(2, &format!("relaunching for {what}:{env} melee {}", shown.join(" ")));
                self.quit = true;
            }
            Err(e) => {
                self.ui_status(1, &format!("relaunch failed: {e}"));
                if to_menu {
                    self.quit = true;
                }
            }
        }
    }
}
