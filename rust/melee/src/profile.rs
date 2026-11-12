//! `melee play` with the frame profiler (melee_core::frameprof): `MELEE_PROFILE=1` or `profile=1` in settings.cfg.
//! Options (each `MELEE_<KEY>` env > settings.cfg key > default):
//! - `profile_dir`: the reports' folder (default `profiles` beside the executable; when that cannot be written, a
//!   `profiles` folder beside settings.cfg, then the temp folder);
//! - `profile_overlay=1`: the on-screen readout (frame-time graph and hitch counter), off by default;
//! - `profile_frame_mult` (default 1.5) / `profile_tick_mult` (default 2.0): the hitch thresholds, x the display's
//!   frame interval and x 16.7 ms;
//! - test knobs: `MELEE_PROFILE_TEST_SLEEP=<tick>:<ms>` (one slow tick), `MELEE_PROFILE_TEST_STAGE=<tick>:<ground kind>`.
//!
//! The report never holds an absolute path: the folder is named relative to the game folder.
use std::path::{Path, PathBuf};

fn num(key: &str, dflt: f64) -> f64 {
    crate::cfg::text(key).and_then(|v| v.trim().parse::<f64>().ok()).filter(|v| *v > 0.0).unwrap_or(dflt)
}

/// `p` relative to `base` when inside it, else just its last component (never a home path).
fn shown(p: &Path, base: &Path) -> String {
    match p.strip_prefix(base) {
        Ok(r) => r.display().to_string(),
        Err(_) => format!("(outside the game folder)/{}", p.file_name().map_or(String::new(), |n| n.to_string_lossy().into_owned())),
    }
}

/// Start recording when asked (call once, before the engine boots and before any mod loads).
pub fn start(display: &crate::display::Display, args: &[String]) {
    if !crate::cfg::flag("profile", false) {
        return;
    }
    let exe_dir = std::env::current_exe().ok().and_then(|p| p.parent().map(Path::to_path_buf)).unwrap_or_else(|| PathBuf::from("."));
    let mut dirs: Vec<PathBuf> = Vec::new();
    if let Some(d) = crate::cfg::text("profile_dir") {
        dirs.push(PathBuf::from(d));
    }
    dirs.push(exe_dir.join("profiles"));
    if let Some(p) = melee_data::settings::default_path().parent() {
        dirs.push(p.join("profiles"));
    }
    dirs.push(std::env::temp_dir().join("melee-profiles"));
    let test_sleep = std::env::var("MELEE_PROFILE_TEST_SLEEP").ok().and_then(|v| {
        let (a, b) = v.split_once(':')?;
        Some((a.trim().parse().ok()?, b.trim().parse().ok()?))
    });
    for dir in dirs {
        let cfg = melee_core::frameprof::Config {
            dir: dir.clone(),
            frame_mult: num("profile_frame_mult", 1.5),
            tick_mult: num("profile_tick_mult", 2.0),
            overlay: crate::cfg::flag("profile_overlay", false),
            test_sleep,
        };
        match melee_core::frameprof::start(cfg) {
            Ok(stem) => {
                let where_ = shown(&dir, &exe_dir);
                eprintln!("profile: recording; the report is {where_}/{stem}.txt (beside the game), rewritten every 5 s");
                use melee_core::frameprof::info;
                info("build", &format!("{} {}{}", env!("MELEE_GIT_WHERE"), env!("MELEE_GIT_HASH"), crate::play_net::dirty_suffix()));
                info("display", &display.summary());
                info("command", &args.iter().filter(|a| !a.contains('/') && !a.contains('\\') && !a.to_ascii_lowercase().ends_with(".iso")).cloned().collect::<Vec<_>>().join(" "));
                let mods = std::env::var("MELEE_WASM_MODS").ok().filter(|v| !v.is_empty()).map(|v| {
                    let p = PathBuf::from(&v);
                    let names: Vec<String> = std::fs::read_dir(&p).map(|r| r.filter_map(|e| e.ok()).map(|e| e.file_name().to_string_lossy().into_owned()).filter(|n| n.ends_with(".wasm")).collect()).unwrap_or_default();
                    format!("{} ({})", p.file_name().map_or(String::new(), |n| n.to_string_lossy().into_owned()), if names.is_empty() { "no .wasm files".into() } else { names.join(", ") })
                });
                info("mods folder", &mods.unwrap_or_else(|| "none (vanilla)".into()));
                if let Some(t) = test_sleep {
                    info("test", &format!("induced slow tick at tick {} ({} ms)", t.0, t.1));
                }
                if let Ok(v) = std::env::var("MELEE_PROFILE_TEST_STAGE") {
                    info("test stage", &format!("forced stage switch {v} (tick:ground kind)"));
                }
                return;
            }
            Err(e) => eprintln!("profile: {}: {e}; trying the next folder", shown(&dir, &exe_dir)),
        }
    }
    eprintln!("profile: no folder could be written; not recording");
}
