//! `melee.exe --settings-window`: the Melee Unlocked launcher's Settings button with the Rust Port picked
//! (docs/rust-port/notes/launcher-seam.md §7). Our in-game settings overlay (settings_overlay.rs), alone in a small
//! window of its own: no disc, no game, no engine. Its text is the host's bundled fonts (melee_gx ui_font), so nothing
//! from the disc is needed.
//!
//! The launcher's contract (his launcher.cpp `open_settings` / `settings_standby_start`, main.cpp, settings_window.cpp;
//! behaviour only, reimplemented):
//! * `--settings-window --settings-path <port-settings.ini> [--settings-tab video|audio|game|controls|overlays|
//!   customize|gecko|mods]`: open now; closing the panel ends the process (exit 0).
//! * `--settings-standby <launcher pid>`: start hidden and wait for the named events
//!   `Local\MeleeUnlockedSettingsShow-<pid>` (show: re-read the files, open) or `...Quit-<pid>`, or the launcher's
//!   exit (both: exit 0). We wait before any window or GPU device exists, so a standby copy costs nothing while it
//!   waits (his keeps a drawn window ready; ours opens in the time a window and a device take).
//!
//! Settings: on open his `port-settings.ini` is read and its display / audio values copied into our settings.cfg
//! (and audio.cfg's volume), so the rows show what both engines will use. Each change is written to settings.cfg by
//! the overlay as in game, and the keys his file shares (port_ini.rs) to his file at once, every other line of it
//! untouched. The controls tables are not edited here (the panel has no bindings page) and stay as they are.
//!
//! The pads: nothing reads the GC adapter while waiting in standby (a game beside the launcher owns it); once shown,
//! the controllers and the keyboard (arrows, Enter, Escape, Tab) drive the panel.
//!
//! Test hooks: `MELEE_SETTINGS_SCRIPT=tab,down,right,...,esc` feeds those keys (one every 6 ticks after 20) through the
//! same path as the keyboard; `MELEE_SETTINGS_SHOT=<png>` writes the window as presented (tick 40 or the script's
//! end); `MELEE_SETTINGS_MAX_TICKS` ends a run that never closes (exit 4).
use crate::port_ini::{self, PortIni};
use crate::settings_overlay::Overlay;
use melee_platform::gfx::{write_png, Presenter};
use melee_platform::{run_windowed, Game, PadSystem, WindowConfig};
use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::Arc;
use winit::window::Window;

pub struct Opts {
    pub ini: Option<PathBuf>,
    pub tab: Option<String>,
    pub standby: Option<u32>,
}

pub fn parse(args: &[String]) -> Opts {
    let get = |k: &str| args.iter().position(|a| a == k).and_then(|i| args.get(i + 1)).cloned();
    Opts {
        ini: get("--settings-path").or_else(|| std::env::var("MELEE_PORT_SETTINGS").ok()).filter(|s| !s.is_empty()).map(PathBuf::from),
        tab: get("--settings-tab"),
        standby: get("--settings-standby").and_then(|v| v.parse().ok()).filter(|&p| p != 0),
    }
}

/// Copy his file's shared values into settings.cfg / audio.cfg (only where they differ). Returns the volume.
pub fn sync_from_port(ini: &PortIni) -> i32 {
    let mut vol = melee_audio::output::load_volume();
    let mut s = melee_data::settings::Settings::new(melee_data::settings::default_path());
    for (k, v) in port_ini::from_port(ini) {
        if k == "volume" {
            let n: i32 = v.parse().unwrap_or(vol);
            if n != vol {
                vol = n;
                let _ = melee_audio::output::save_volume(n);
            }
        } else if crate::cfg::text(&k).as_deref() != Some(v.as_str()) {
            s.set_str(&k, &v);
        }
    }
    vol
}

const KEY_CODES: &[(&str, u32)] = &[("a", 1), ("enter", 1), ("b", 2), ("esc", 2), ("up", 4), ("down", 5), ("left", 6), ("right", 7), ("tab", 8)];

struct SettingsWindow {
    o: Overlay,
    ini_path: Option<PathBuf>,
    ini: PortIni,
    presenter: Option<Presenter>,
    quit: bool,
    exit: u8,
    tick: u32,
    script: Vec<u32>,
    shot: Option<PathBuf>,
    shot_at: u32,
    max_ticks: Option<u32>,
}

impl SettingsWindow {
    fn apply(&mut self) {
        for c in self.o.take_changes() {
            match c.key.as_str() {
                "action:quit" | "action:menu" => {
                    self.quit = true;
                    continue;
                }
                "volume" => {
                    let v: i32 = c.value.parse().unwrap_or(100);
                    let _ = melee_audio::output::save_volume(v);
                }
                _ => {}
            }
            let his = port_ini::to_port(&c.key, &c.value, &self.ini);
            if his.is_empty() {
                continue;
            }
            let mut changed = false;
            for (k, v) in his {
                changed |= self.ini.set(k, &v);
            }
            if let (true, Some(p)) = (changed, self.ini_path.as_ref()) {
                match self.ini.save(p) {
                    Ok(()) => eprintln!("settings window: {} = {} -> {}", c.key, c.value, p.display()),
                    Err(e) => eprintln!("settings window: cannot write {}: {e}", p.display()),
                }
            }
        }
    }
}

impl Game for SettingsWindow {
    fn tick(&mut self, pads: &mut PadSystem) {
        self.tick += 1;
        if pads.ui_keys.is_none() {
            pads.ui_keys = Some(Vec::with_capacity(64));
        }
        let mut keys: Vec<u32> = pads.ui_keys.as_mut().map(std::mem::take).unwrap_or_default();
        if self.tick >= 20 && (self.tick - 20) % 6 == 0 {
            let i = ((self.tick - 20) / 6) as usize;
            if let Some(&k) = self.script.get(i) {
                keys.push(1 << 24 | k);
            }
        }
        let mut live = [melee_types::PADStatus::zeroed(); 4];
        pads.read(&mut live);
        let mut four = live;
        self.o.input(&live, &mut four, false, false, &keys);
        self.apply();
        if !self.o.open {
            self.quit = true;
        }
        if self.max_ticks.is_some_and(|m| self.tick >= m) {
            eprintln!("settings window: MELEE_SETTINGS_MAX_TICKS reached");
            self.exit = 4;
            self.quit = true;
        }
    }
    fn present(&mut self, window: &Arc<Window>, _alpha: f64) {
        if self.presenter.is_none() {
            match Presenter::new(window.clone()) {
                Ok(p) => self.presenter = Some(p),
                Err(e) => {
                    eprintln!("settings window: {e}");
                    self.exit = 1;
                    self.quit = true;
                    return;
                }
            }
        }
        let p = self.presenter.as_mut().unwrap();
        // drawn while open; after the close the frame is not shown again (the process ends)
        if let Some((f, a)) = self.o.own_frame() {
            p.renderer.set_ui(Some(f), a);
        }
        p.present(&melee_engine::gxbridge::shim().gx.state, None);
        if self.shot.is_some() && self.tick >= self.shot_at {
            let path = self.shot.take().unwrap();
            let (w, h, px) = p.screenshot_window();
            match write_png(&path, w, h, &px) {
                Ok(()) => eprintln!("settings window: shot {} ({w}x{h}, {} quads)", path.display(), self.o.last_quads),
                Err(e) => eprintln!("settings window: shot: {e}"),
            }
        }
    }
    fn resized(&mut self, w: u32, h: u32) {
        if let Some(p) = self.presenter.as_mut() {
            p.resize(w, h);
        }
    }
    fn should_quit(&self) -> bool {
        EXIT.store(self.exit, std::sync::atomic::Ordering::Relaxed);
        self.quit && self.shot.is_none()
    }
}

#[cfg(windows)]
mod standby {
    use std::ffi::c_void;
    type H = *mut c_void;
    #[link(name = "kernel32")]
    extern "system" {
        fn CreateEventW(attrs: *const c_void, manual: i32, initial: i32, name: *const u16) -> H;
        fn OpenProcess(access: u32, inherit: i32, pid: u32) -> H;
        fn WaitForMultipleObjects(n: u32, handles: *const H, all: i32, ms: u32) -> u32;
        fn CloseHandle(h: H) -> i32;
    }
    const SYNCHRONIZE: u32 = 0x0010_0000;
    const INFINITE: u32 = 0xFFFF_FFFF;

    fn wide(s: &str) -> Vec<u16> {
        s.encode_utf16().chain(std::iter::once(0)).collect()
    }

    /// Block until the launcher asks to show the panel (true), or to quit / exits (false).
    pub fn wait(pid: u32) -> bool {
        unsafe {
            let show = CreateEventW(std::ptr::null(), 0, 0, wide(&format!("Local\\MeleeUnlockedSettingsShow-{pid}")).as_ptr());
            let quit = CreateEventW(std::ptr::null(), 0, 0, wide(&format!("Local\\MeleeUnlockedSettingsQuit-{pid}")).as_ptr());
            let parent = OpenProcess(SYNCHRONIZE, 0, pid);
            let mut shown = false;
            if !show.is_null() && !quit.is_null() && !parent.is_null() {
                let hs = [show, quit, parent];
                shown = WaitForMultipleObjects(3, hs.as_ptr(), 0, INFINITE) == 0;
            }
            for h in [show, quit, parent] {
                if !h.is_null() {
                    CloseHandle(h);
                }
            }
            shown
        }
    }
}

#[cfg(not(windows))]
mod standby {
    pub fn wait(_pid: u32) -> bool {
        false
    }
}

pub fn run(args: &[String]) -> ExitCode {
    let opts = parse(args);
    if let Some(pid) = opts.standby {
        eprintln!("settings window: standby for launcher {pid}");
        if !standby::wait(pid) {
            eprintln!("settings window: standby ended (quit or the launcher closed)");
            return ExitCode::SUCCESS;
        }
        eprintln!("settings window: shown");
    }
    // read after the wait: what the game or an earlier panel saved meanwhile
    let ini = opts.ini.as_deref().map(PortIni::load).unwrap_or_default();
    let vol = sync_from_port(&ini);
    let mut o = Overlay::new(vol).into_window(opts.tab.as_deref());
    o.refresh_msaa();
    eprintln!("settings window: {} ({}), page {}", opts.ini.as_ref().map_or("no port-settings.ini".into(), |p| p.display().to_string()), if ini.pairs().is_empty() { "empty" } else { "read" }, o.page_name());
    let script = std::env::var("MELEE_SETTINGS_SCRIPT")
        .unwrap_or_default()
        .split(',')
        .filter_map(|t| KEY_CODES.iter().find(|(n, _)| n.eq_ignore_ascii_case(t.trim())).map(|k| k.1))
        .collect::<Vec<_>>();
    let shot = std::env::var("MELEE_SETTINGS_SHOT").ok().filter(|s| !s.is_empty()).map(PathBuf::from);
    let shot_at = if script.is_empty() { 40 } else { 20 + 6 * (script.len() as u32 - 1) - 1 };
    let win = SettingsWindow {
        o,
        ini_path: opts.ini,
        ini,
        presenter: None,
        quit: false,
        exit: 0,
        tick: 0,
        script,
        shot,
        shot_at,
        max_ticks: std::env::var("MELEE_SETTINGS_MAX_TICKS").ok().and_then(|v| v.parse().ok()),
    };
    let (w, h) = std::env::var("MELEE_WINDOW").ok().and_then(|v| v.split_once(['x', 'X']).and_then(|(a, b)| Some((a.trim().parse().ok()?, b.trim().parse().ok()?)))).unwrap_or((800, 600));
    let title = std::env::var("MELEE_WINDOW_TITLE").unwrap_or_else(|_| "Melee Unlocked - Rust Port settings".into());
    let mut pads = PadSystem::interactive();
    pads.ui_keys = Some(Vec::with_capacity(64));
    crate::launcher_compat::apply_input(&mut pads);
    let active = !melee_platform::window_hidden() && std::env::var_os("MELEE_WINDOW_UNFOCUSED").is_none();
    let cfg = WindowConfig { title, width: w, height: h, active, ..Default::default() };
    match run_windowed(win, pads, cfg) {
        Ok(_) => {
            eprintln!("settings window: closed");
            ExitCode::from(EXIT.load(std::sync::atomic::Ordering::Relaxed))
        }
        Err(e) => {
            eprintln!("settings window: event loop: {e}");
            ExitCode::FAILURE
        }
    }
}

static EXIT: std::sync::atomic::AtomicU8 = std::sync::atomic::AtomicU8::new(0);
