//! The optional second disc (Super Smash Bros. Brawl, RSBE01) and the first-run build step that would turn it into
//! Brawl-derived characters under the data folder. Input and hook only: the conversion itself is not written yet.
//!
//! YouGame gives a native build exactly one disc (yougame-native.json `"disc": true`, `{disc}` in its args: the
//! listing's one `base_game`), so on YouGame a Brawl disc cannot arrive today (notes/yougame-listing.md, open
//! decisions). Outside YouGame, or once a route exists, it comes as `--brawl-disc PATH` or `MELEE_BRAWL_DISC`.
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
}

/// The first-run hook: called once at start, before the game boots. `progress(fraction, label)` is the player's
/// progress line; an `Err` is a message for the player (the game still plays Melee without Brawl characters).
pub fn run(brawl: Option<&str>, progress: &mut dyn FnMut(f32, &str)) -> Result<Step, String> {
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
