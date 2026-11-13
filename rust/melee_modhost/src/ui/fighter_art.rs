//! Fighter icons and portraits for the presentation layer, read by the engine from the player's own character-select
//! file at boot (melee_engine::fighter_art) and handed here. Presentation only: never read by a sim, never stored.

use std::sync::Mutex;

/// One RGBA picture, row-major, `w * h * 4` bytes.
#[derive(Clone)]
pub struct Pic {
    pub w: u32,
    pub h: u32,
    pub rgba: Vec<u8>,
}

/// A fighter's art by port CharacterKind.
pub struct Art {
    pub ck: i32,
    pub icon: Option<Pic>,
    pub portrait: Option<Pic>,
}

static ART: Mutex<Vec<Art>> = Mutex::new(Vec::new());

/// Replace the art (the engine, once at boot).
pub fn set(v: Vec<Art>) {
    *ART.lock().unwrap_or_else(|e| e.into_inner()) = v;
}

/// Fighter `ck`'s icon (`which` 0) or portrait (1), cloned; None when the disc had none.
pub fn get(ck: i32, which: i32) -> Option<Pic> {
    let a = ART.lock().unwrap_or_else(|e| e.into_inner());
    let e = a.iter().find(|a| a.ck == ck)?;
    match which {
        0 => e.icon.clone(),
        1 => e.portrait.clone(),
        _ => None,
    }
}
