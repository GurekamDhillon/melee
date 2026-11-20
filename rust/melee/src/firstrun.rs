//! The optional second disc (Super Smash Bros. Brawl, RSBE01): Brawl-derived characters and Brawl audio.
//!
//! Two ways in. YouGame's dual-disc listing (`kind: "patch"` with `also_requires`, `yougame-native.json`
//! `"discs": {"brawl": "optional"}`) has the app unpack the player's own Brawl disc and hand us the folder as
//! `{disc:brawl:files}` / `YOUGAME_DISC_BRAWL_FILES`; that folder is read-only, shared and already extracted
//! (`files/...`), so nothing is converted here. Outside YouGame, or with the app's raw image, it comes as
//! `--brawl-disc PATH` / `MELEE_BRAWL_DISC` (a plain .iso, RSBE01 header-checked; the conversion is still a stub).
//! Nothing read from either disc is ever written outside the data folder, shipped or committed.
use std::path::{Path, PathBuf};

pub const BRAWL_WANT: &str = "Super Smash Bros. Brawl (USA) (RSBE01), as a plain .iso";

/// Header check for the Brawl disc: game id RSBE01, the Wii magic at 0x18. Never names the path.
pub fn identify_brawl_header(h: &[u8; 0x20]) -> Result<(), String> {
    if h[0x18..0x1C] != [0x5D, 0x1C, 0x9E, 0xA3] {
        return Err(format!("the second disc is not a plain Wii disc image: it needs {BRAWL_WANT}"));
    }
    if &h[..6] != b"RSBE01" {
        let shown: String = h[..6].iter().map(|&b| if b.is_ascii_alphanumeric() { b as char } else { '?' }).collect();
        return Err(format!("the second disc is a different game (id {shown}): it needs {BRAWL_WANT}"));
    }
    Ok(())
}

/// The YouGame dual-disc folder (`{disc:brawl:files}`): the `--brawl-files PATH` argument or the app's
/// `YOUGAME_DISC_BRAWL_FILES`. It holds the player's own Brawl DATA partition, already unpacked (`files/...`).
/// Returns it only when that `files/` folder is really there.
pub fn yougame_files() -> Option<PathBuf> {
    if let Some(p) = std::env::var_os("YOUGAME_DISC_BRAWL_FILES").map(PathBuf::from) {
        if p.join("files").is_dir() {
            return Some(p);
        }
    }
    let args: Vec<std::ffi::OsString> = std::env::args_os().collect();
    let mut i = 0;
    while i < args.len() {
        if args[i].to_str() == Some("--brawl-files") {
            if let Some(p) = args.get(i + 1).map(|s| PathBuf::from(s.as_os_str())) {
                if p.join("files").is_dir() {
                    return Some(p);
                }
            }
        } else if let Some(rest) = args[i].to_str().and_then(|s| s.strip_prefix("--brawl-files=")) {
            let p = PathBuf::from(rest);
            if p.join("files").is_dir() {
                return Some(p);
            }
        }
        i += 1;
    }
    None
}

/// Where the first-run step's output lives: `<YOUGAME_DATA or .>/brawl`.
pub fn out_dir() -> PathBuf {
    melee_platform::yougame::data_dir().map(PathBuf::from).unwrap_or_else(|| PathBuf::from(".")).join("brawl")
}

/// What happened at first run.
#[derive(Debug, PartialEq)]
pub enum Step {
    /// No Brawl disc given: nothing to do; Melee plays without Brawl characters.
    NoDisc,
    /// Built on an earlier run (the marker is there).
    AlreadyBuilt,
    /// The build ran now.
    Built,
    /// YouGame handed us the player's already-unpacked Brawl folder (`{disc:brawl:files}`): nothing to convert.
    FilesReady,
}

/// The first-run hook: called once at start, before the game boots. `progress(fraction, label)` is the player's
/// progress line; an `Err` is a message for the player (the game still plays Melee without Brawl characters).
pub fn run(brawl: Option<&str>, progress: &mut dyn FnMut(f32, &str)) -> Result<Step, String> {
    // YouGame dual-disc: the app already unpacked the player's Brawl disc. Point the Brawl audio reader at the
    // folder (the reader also reads YOUGAME_DISC_BRAWL_FILES itself) and skip the disc-image path entirely.
    if let Some(dir) = yougame_files() {
        if std::env::var_os("MELEE_BRAWL_LOCAL").is_none() {
            std::env::set_var("MELEE_BRAWL_LOCAL", &dir);
        }
        progress(1.0, "Brawl files from your disc are ready");
        return Ok(Step::FilesReady);
    }
    let Some(p) = brawl else { return Ok(Step::NoDisc) };
    let out = out_dir();
    if out.join("built.txt").is_file() {
        return Ok(Step::AlreadyBuilt);
    }
    progress(0.0, "Checking your Brawl disc");
    let mut h = [0u8; 0x20];
    {
        use std::io::Read;
        let mut f = std::fs::File::open(p).map_err(|e| format!("the Brawl disc image could not be opened ({})", e.kind()))?;
        f.read_exact(&mut h).map_err(|_| "the Brawl disc image is empty or cut short".to_string())?;
    }
    identify_brawl_header(&h)?;
    build(Path::new(p), &out, progress)?;
    std::fs::create_dir_all(&out).map_err(|e| format!("could not write the data folder ({})", e.kind()))?;
    std::fs::write(out.join("built.txt"), "brawl-build v0 (empty: conversion not implemented)\n").map_err(|e| format!("could not write the data folder ({})", e.kind()))?;
    progress(1.0, "Brawl characters ready");
    Ok(Step::Built)
}

/// The conversion itself: NOT implemented (design only). It must write only under `out`.
fn build(_disc: &Path, _out: &Path, progress: &mut dyn FnMut(f32, &str)) -> Result<(), String> {
    progress(0.5, "Building Brawl characters (not implemented yet: nothing is built)");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn brawl_header() {
        let mut h = [0u8; 0x20];
        h[..6].copy_from_slice(b"RSBE01");
        h[0x18..0x1C].copy_from_slice(&[0x5D, 0x1C, 0x9E, 0xA3]);
        assert!(identify_brawl_header(&h).is_ok());
        h[3] = b'P';
        assert!(identify_brawl_header(&h).unwrap_err().contains("RSBP01"));
        assert!(identify_brawl_header(&[0u8; 0x20]).is_err());
        assert_eq!(run(None, &mut |_, _| {}), Ok(Step::NoDisc));
    }
}
