//! Texture overrides: a decoded RGBA picture drawn in place of a game texture whose payload (texels, and palette for
//! the indexed formats) has a given identity. The cosmetics layer (melee `cosmetics.rs`) registers a skin's portrait
//! and stock icon here. Render-only: the game's memory, its draw lists and their hashes are untouched; the renderer
//! only uploads different pixels when it decodes a texture for the GPU.
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering::Relaxed};
use std::sync::{Arc, RwLock};

/// A texture's identity: format, size, FNV-1a 64 of the base level's texels, and of the palette (0: none).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Key {
    pub fmt: u32,
    pub w: u32,
    pub h: u32,
    pub texels: u64,
    pub tlut: u64,
}

/// A replacement picture, RGBA8, top row first.
#[derive(Debug)]
pub struct Picture {
    pub w: u32,
    pub h: u32,
    pub rgba: Vec<u8>,
}

static MAP: RwLock<Option<Arc<HashMap<Key, Arc<Picture>>>>> = RwLock::new(None);
static GEN: AtomicU64 = AtomicU64::new(0);

pub fn fnv64(b: &[u8]) -> u64 {
    let mut h = 0xcbf2_9ce4_8422_2325u64;
    for &c in b {
        h ^= c as u64;
        h = h.wrapping_mul(0x100_0000_01b3);
    }
    h
}

/// Bytes of the base level of a `w` x `h` texture in GX format `fmt` (block-padded), None for an unknown format.
pub fn level0_size(fmt: u32, w: u32, h: u32) -> Option<usize> {
    use crate::consts::*;
    let (bw, bh, bytes) = match fmt {
        GX_TF_I4 | GX_TF_C4 | GX_TF_CMPR => (8, 8, 32),
        GX_TF_I8 | GX_TF_IA4 | GX_TF_C8 => (8, 4, 32),
        GX_TF_IA8 | GX_TF_RGB565 | GX_TF_RGB5A3 | GX_TF_C14X2 => (4, 4, 32),
        GX_TF_RGBA8 => (4, 4, 64),
        _ => return None,
    };
    Some((w.div_ceil(bw) * h.div_ceil(bh) * bytes) as usize)
}

/// The identity of a texture payload (`tlut`: the palette bytes, for the indexed formats).
pub fn key(fmt: u32, w: u32, h: u32, texels: &[u8], tlut: Option<&[u8]>) -> Key {
    let n = level0_size(fmt, w, h).unwrap_or(texels.len()).min(texels.len());
    Key { fmt, w, h, texels: fnv64(&texels[..n]), tlut: tlut.map_or(0, fnv64) }
}

/// Replace the whole set (None or empty: no overrides). The renderer drops its decoded textures on the next frame.
pub fn set(map: Option<HashMap<Key, Arc<Picture>>>) {
    *MAP.write().unwrap() = map.filter(|m| !m.is_empty()).map(Arc::new);
    GEN.fetch_add(1, Relaxed);
}

/// The change counter (the renderer compares it once a frame).
pub fn generation() -> u64 {
    GEN.load(Relaxed)
}

pub fn active() -> bool {
    MAP.read().unwrap().is_some()
}

pub fn lookup(k: &Key) -> Option<Arc<Picture>> {
    MAP.read().unwrap().as_ref()?.get(k).cloned()
}
