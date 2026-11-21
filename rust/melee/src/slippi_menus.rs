//! `--slippi-menus on|off` (Hero's switch, Melee Unlocked `port/app/main.cpp` / `source_host.h`): serve Slippi's
//! patched menu files over the disc so the main menu has Slippi's Online rows. Default off for now (the default is
//! the owner's call). Off, or any failure, boots the disc's own menus exactly as before: the layer is all or nothing.
//!
//! The files come from `melee_data::sysfiles` (fetched Slippi files, hash-checked, plus the player's disc). Forced
//! off for replays, like Hero's `--replay`.

use std::sync::{Arc, OnceLock};

use melee_data::sysfiles::SysFiles;

static LAYER: OnceLock<Option<Arc<SysFiles>>> = OnceLock::new();

/// The run asked for the layer: `--slippi-menus on` (or `MELEE_SLIPPI_MENUS=on`), and nothing forces it off.
pub fn requested(args: &[String]) -> Result<bool, &'static str> {
    let get = |k: &str| args.iter().position(|a| a == k).and_then(|i| args.get(i + 1)).cloned();
    let v = get("--slippi-menus").or_else(|| std::env::var("MELEE_SLIPPI_MENUS").ok()).unwrap_or_else(|| "off".into());
    if !matches!(v.as_str(), "on" | "1" | "true") {
        return Ok(false);
    }
    for f in ["--replay", "--vanilla-only", "--online-test"] {
        if args.iter().any(|a| a == f) {
            return Err(match f {
                "--replay" => "--replay",
                "--vanilla-only" => "--vanilla-only",
                _ => "--online-test",
            });
        }
    }
    Ok(true)
}

/// Whether the layer is loaded for this run (the online menu code checks this).
pub fn active() -> bool {
    LAYER.get().is_some_and(Option::is_some)
}

/// Build the layer once (first call wins) and return it. Logs one line per file; never logs a disc path.
pub fn load(args: &[String], iso: Option<&str>) -> Option<Arc<SysFiles>> {
    LAYER
        .get_or_init(|| {
            match requested(args) {
                Ok(false) => {
                    eprintln!("slippi menus: off");
                    return None;
                }
                Err(by) => {
                    eprintln!("slippi menus: off ({by})");
                    return None;
                }
                Ok(true) => {}
            }
            let Some(iso) = iso else {
                eprintln!("slippi menus: off (no disc)");
                return None;
            };
            let mut gcm = match melee_engine::gcm::Gcm::open(iso) {
                Ok(g) => g,
                Err(_) => {
                    eprintln!("slippi menus: off (cannot open the disc)");
                    return None;
                }
            };
            let cache = melee_data::sysfiles::default_cache_dir();
            let mut disc = |n: &str| gcm.read_vanilla(n).map_err(|_| format!("{n}: not on the disc"));
            let mut fetch = |u: &str| melee_data::sysfiles::fetch_curl(u);
            match SysFiles::build(&cache, &mut disc, &mut fetch) {
                Ok(s) => {
                    for (n, size, h) in &s.built {
                        eprintln!("slippi menus: /{} {size} {h:08x}", n.to_ascii_lowercase());
                    }
                    eprintln!("slippi menus: on ({} files)", s.len());
                    Some(Arc::new(s))
                }
                Err(e) => {
                    eprintln!("slippi menus: off ({e})");
                    None
                }
            }
        })
        .clone()
}

/// Put the layer over `under` (the mods overlay): Slippi's copy answers first, as in Hero's build (a pack's own copy
/// of a menu file is replaced by Slippi's).
pub fn wrap(s: Arc<SysFiles>, under: Option<melee_engine::gcm::Overlay>) -> melee_engine::gcm::Overlay {
    Box::new(move |p: &str| s.get(p).map(<[u8]>::to_vec).or_else(|| under.as_ref().and_then(|u| u(p))))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn a(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn default_is_off_and_replay_forces_off() {
        if std::env::var_os("MELEE_SLIPPI_MENUS").is_some() {
            return;
        }
        assert_eq!(requested(&a(&["play"])), Ok(false));
        assert_eq!(requested(&a(&["play", "--slippi-menus", "off"])), Ok(false));
        assert_eq!(requested(&a(&["play", "--slippi-menus", "on"])), Ok(true));
        assert_eq!(requested(&a(&["play", "--slippi-menus", "on", "--replay", "x.slp"])), Err("--replay"));
    }

    #[test]
    fn wrap_prefers_slippi_then_mods() {
        let mut s = SysFiles::default();
        s.insert("MnMaAll.usd", vec![7]);
        let under: melee_engine::gcm::Overlay = Box::new(|p: &str| (p == "/PlFx.dat").then(|| vec![9]));
        let o = wrap(Arc::new(s), Some(under));
        assert_eq!(o("/MnMaAll.usd"), Some(vec![7]));
        assert_eq!(o("/PlFx.dat"), Some(vec![9]));
        assert_eq!(o("/MnMaAll.dat"), None);
    }
}
