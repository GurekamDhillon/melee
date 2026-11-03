//! `melee play` display options: widescreen, fullscreen, window size, render scale, MSAA, vsync.
//!
//! Each option is resolved as: the command-line flag, else the environment (`MELEE_<KEY>`), else `settings.cfg`
//! beside the executable (`MELEE_SETTINGS_CFG` moves it; the C build's key=value file, see cfg.rs), else the default.
//! `--save-display` writes the resolved values back to settings.cfg. The result is exported to the environment so
//! the layers below (melee_gx's renderer, melee_platform's presenter) read one answer.
//!
//! None of these reach the simulation: the headless engine (golden, gate) installs fixed view values of its own, and
//! the windowed view hooks (melee_engine view.rs) only shape cameras' projections and the render target, as the C
//! build's `gw_view_*` policy does. Notes: docs/rust-port/notes/display.md.
use melee_platform::FullscreenMode;

pub const HELP: &str = "\
display options (flag > MELEE_<KEY> env > settings.cfg key > default):
  --widescreen | --no-widescreen   16:9 (Hor+) in matches, the C build's gw_view policy; the menus and the CSS stay
                                   4:3 (pillarboxed). Key `widescreen` (0/1), default 0.
  --fullscreen[=borderless|exclusive] | --windowed
                                   start fullscreen; Alt+Enter toggles at any time. Key `fullscreen`
                                   (off|borderless|exclusive), default off. Exclusive uses the monitor's largest mode.
  --window WxH                     window client size, default 1280x960 (1280x720 with --widescreen). Key `window`.
  --render-scale N|auto            internal resolution, N x 480 lines (1..8); auto follows the window. Key
                                   `render_scale`, default auto.
  --msaa 1|2|4|8                   multisample anti-aliasing of the EFB pass (falls back to what the GPU supports).
                                   Key `msaa`, default 1 (off; auto render scale already supersamples small windows).
  --vsync on|off                   Key `vsync`, default on.
  --save-display                   write the resolved display options to settings.cfg.
The content keeps its aspect: letterbox / pillarbox bars fill the rest of the window.";

#[derive(Clone, Debug, PartialEq)]
pub struct Display {
    pub widescreen: bool,
    pub fullscreen: FullscreenMode,
    pub window: (u32, u32),
    /// None: auto.
    pub render_scale: Option<u32>,
    pub msaa: u32,
    pub vsync: bool,
}

fn parse_bool(v: &str) -> Option<bool> {
    match v.trim().to_ascii_lowercase().as_str() {
        "1" | "on" | "yes" | "true" => Some(true),
        "0" | "off" | "no" | "false" => Some(false),
        _ => None,
    }
}

fn parse_size(v: &str) -> Option<(u32, u32)> {
    let (a, b) = v.trim().split_once(['x', 'X'])?;
    let (w, h) = (a.trim().parse::<u32>().ok()?, b.trim().parse::<u32>().ok()?);
    (w >= 160 && h >= 120 && w <= 16384 && h <= 16384).then_some((w, h))
}

fn parse_scale(v: &str) -> Option<Option<u32>> {
    let v = v.trim();
    if v.eq_ignore_ascii_case("auto") || v == "0" {
        return Some(None);
    }
    v.parse::<u32>().ok().filter(|&n| n >= 1).map(|n| Some(n.min(melee_gx::render::Renderer::MAX_SCALE)))
}

fn parse_msaa(v: &str) -> Option<u32> {
    match v.trim().trim_end_matches(['x', 'X']) {
        "0" | "1" | "off" => Some(1),
        "2" => Some(2),
        "4" => Some(4),
        "8" => Some(8),
        _ => None,
    }
}

impl Display {
    /// Resolve from `args` over `lookup` (a settings.cfg / env value by key; None when unset).
    pub fn resolve(args: &[String], lookup: &mut dyn FnMut(&str) -> Option<String>) -> Result<Self, String> {
        let get = |k: &str| args.iter().position(|a| a == k).and_then(|i| args.get(i + 1)).cloned();
        let has = |k: &str| args.iter().any(|a| a == k);
        let bad = |k: &str, v: &str| format!("display: bad {k} value '{v}' (see melee play --help)");
        let widescreen = if has("--widescreen") {
            true
        } else if has("--no-widescreen") {
            false
        } else {
            match lookup("widescreen") {
                Some(v) => parse_bool(&v).ok_or_else(|| bad("widescreen", &v))?,
                None => false,
            }
        };
        let fs_flag = args.iter().find_map(|a| {
            if a == "--fullscreen" {
                Some("borderless".to_string())
            } else {
                a.strip_prefix("--fullscreen=").map(str::to_string)
            }
        });
        let fullscreen = if has("--windowed") {
            FullscreenMode::Off
        } else if let Some(v) = fs_flag.or_else(|| lookup("fullscreen")) {
            FullscreenMode::parse(&v).ok_or_else(|| bad("fullscreen", &v))?
        } else {
            FullscreenMode::Off
        };
        let window = match get("--window").or_else(|| lookup("window")) {
            Some(v) => parse_size(&v).ok_or_else(|| bad("window", &v))?,
            None if widescreen => (1280, 720),
            None => (1280, 960),
        };
        let render_scale = match get("--render-scale").or_else(|| lookup("render_scale")) {
            Some(v) => parse_scale(&v).ok_or_else(|| bad("render-scale", &v))?,
            None => None,
        };
        let msaa = match get("--msaa").or_else(|| lookup("msaa")) {
            Some(v) => parse_msaa(&v).ok_or_else(|| bad("msaa", &v))?,
            None => 1,
        };
        let vsync = match get("--vsync").or_else(|| lookup("vsync")) {
            Some(v) => parse_bool(&v).ok_or_else(|| bad("vsync", &v))?,
            None => true,
        };
        Ok(Display { widescreen, fullscreen, window, render_scale, msaa, vsync })
    }

    /// Resolve against the real environment and settings.cfg.
    pub fn from_args(args: &[String]) -> Result<Self, String> {
        let mut s = crate::cfg::open();
        Self::resolve(args, &mut |k| {
            let v = s.str_or(k, "");
            (!v.trim().is_empty()).then_some(v)
        })
    }

    fn fullscreen_name(&self) -> &'static str {
        match self.fullscreen {
            FullscreenMode::Off => "off",
            FullscreenMode::Borderless => "borderless",
            FullscreenMode::Exclusive => "exclusive",
        }
    }

    /// The key=value rows settings.cfg keeps.
    pub fn rows(&self) -> Vec<(&'static str, String)> {
        vec![
            ("widescreen", (self.widescreen as u8).to_string()),
            ("fullscreen", self.fullscreen_name().to_string()),
            ("window", format!("{}x{}", self.window.0, self.window.1)),
            ("render_scale", self.render_scale.map_or("auto".to_string(), |n| n.to_string())),
            ("msaa", self.msaa.to_string()),
            ("vsync", if self.vsync { "on" } else { "off" }.to_string()),
        ]
    }

    /// Export to the environment the lower layers read, and to melee_engine's view state.
    pub fn apply(&self) {
        std::env::set_var("MELEE_WIDESCREEN", if self.widescreen { "1" } else { "0" });
        match self.render_scale {
            Some(n) => std::env::set_var("MELEE_RENDER_SCALE", n.to_string()),
            None => std::env::remove_var("MELEE_RENDER_SCALE"),
        }
        std::env::set_var("MELEE_MSAA", self.msaa.to_string());
        std::env::set_var("MELEE_VSYNC", if self.vsync { "on" } else { "off" });
        melee_engine::view::set_widescreen(self.widescreen);
        melee_engine::view::set_window(self.window.0, self.window.1);
    }

    /// `--save-display`: write [`Display::rows`] to settings.cfg.
    pub fn save(&self) -> Result<std::path::PathBuf, String> {
        let mut s = melee_data::settings::Settings::new(melee_data::settings::default_path());
        for (k, v) in self.rows() {
            s.set_str(k, &v);
            if !s.last_save_ok() {
                return Err(format!("could not write {}", s.path().display()));
            }
        }
        Ok(s.path().to_path_buf())
    }

    pub fn summary(&self) -> String {
        self.rows().iter().map(|(k, v)| format!("{k}={v}")).collect::<Vec<_>>().join(" ")
    }

    /// The platform window config for `title`.
    pub fn window_config(&self, title: String, active: bool) -> melee_platform::WindowConfig {
        melee_platform::WindowConfig { title, width: self.window.0, height: self.window.1, active, fullscreen: self.fullscreen, overlay: crate::cfg::flag("overlay", false) }
    }
}

static CURRENT: std::sync::OnceLock<Display> = std::sync::OnceLock::new();

/// The options [`setup`] resolved (the defaults before it ran).
pub fn current() -> Display {
    CURRENT.get().cloned().unwrap_or_else(|| Display::resolve(&[], &mut |_| None).unwrap())
}

/// Resolve, apply and log the display options for `melee play` (and `--save-display`). Err: a bad value.
pub fn setup(args: &[String]) -> Result<Display, String> {
    let d = Display::from_args(args)?;
    d.apply();
    let _ = CURRENT.set(d.clone());
    eprintln!("play: display {}", d.summary());
    if args.iter().any(|a| a == "--save-display") {
        match d.save() {
            Ok(p) => eprintln!("play: display options saved to {}", p.display()),
            Err(e) => eprintln!("play: {e}"),
        }
    }
    Ok(d)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn a(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn defaults_and_precedence() {
        let none = |_: &str| None;
        let d = Display::resolve(&a(&[]), &mut { none }).unwrap();
        assert_eq!(d, Display { widescreen: false, fullscreen: FullscreenMode::Off, window: (1280, 960), render_scale: None, msaa: 1, vsync: true });
        // the file / env says wide + exclusive + 4x; the flags win
        let mut file = |k: &str| match k {
            "widescreen" => Some("1".to_string()),
            "fullscreen" => Some("exclusive".to_string()),
            "msaa" => Some("4".to_string()),
            "render_scale" => Some("3".to_string()),
            _ => None,
        };
        let d = Display::resolve(&a(&[]), &mut file).unwrap();
        assert!(d.widescreen && d.fullscreen == FullscreenMode::Exclusive && d.msaa == 4 && d.render_scale == Some(3) && d.window == (1280, 720));
        let d = Display::resolve(&a(&["--no-widescreen", "--windowed", "--msaa", "2", "--render-scale", "auto", "--window", "800x600"]), &mut file).unwrap();
        assert!(!d.widescreen && d.fullscreen == FullscreenMode::Off && d.msaa == 2 && d.render_scale.is_none() && d.window == (800, 600));
        let d = Display::resolve(&a(&["--fullscreen"]), &mut { none }).unwrap();
        assert_eq!(d.fullscreen, FullscreenMode::Borderless);
        let d = Display::resolve(&a(&["--fullscreen=exclusive", "--vsync", "off"]), &mut { none }).unwrap();
        assert!(d.fullscreen == FullscreenMode::Exclusive && !d.vsync);
        assert!(Display::resolve(&a(&["--msaa", "3"]), &mut { none }).is_err());
        assert!(Display::resolve(&a(&["--window", "big"]), &mut { none }).is_err());
    }

    #[test]
    fn rows_round_trip() {
        let d = Display { widescreen: true, fullscreen: FullscreenMode::Borderless, window: (1600, 900), render_scale: Some(2), msaa: 4, vsync: false };
        let rows = d.rows();
        let mut lk = |k: &str| rows.iter().find(|(r, _)| *r == k).map(|(_, v)| v.clone());
        assert_eq!(Display::resolve(&[], &mut lk).unwrap(), d);
    }
}
