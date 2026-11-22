//! The host app's master volume over the game's own (the YouGame page's `embed volume <gain>`, Hero's
//! `audio_set_page_volume`). Written by the bridge's stdin thread, read by the audio driver once per frame.
//! The gain is the page's own amplitude gain in thousandths (0..=1000), multiplied into the mixer gain as it is:
//! the page already shaped its slider into that gain (it is what it applies to a web game's audio), so applying
//! another curve here would bend it twice.

use std::sync::atomic::{AtomicU32, Ordering};

static GAIN_MILLI: AtomicU32 = AtomicU32::new(1000);
static GENERATION: AtomicU32 = AtomicU32::new(0);

/// Set the host gain (thousandths, clamped to 0..=1000). Returns whether it changed.
pub fn set_milli(g: u32) -> bool {
    let g = g.min(1000);
    let changed = GAIN_MILLI.swap(g, Ordering::SeqCst) != g;
    if changed {
        GENERATION.fetch_add(1, Ordering::SeqCst);
    }
    changed
}

/// The host gain in thousandths (1000 = no change).
pub fn milli() -> u32 {
    GAIN_MILLI.load(Ordering::SeqCst)
}

/// Bumped on every change, so the audio driver re-applies its gain only when needed.
pub fn generation() -> u32 {
    GENERATION.load(Ordering::SeqCst)
}

/// `volume <gain>` with a plain decimal gain in [0, 1] -> thousandths; anything else -> None. By hand, not
/// `f32::parse`, as Hero's yg_embed_volume: digits past the third decimal are dropped (and must be 0 at 1).
pub fn parse_embed(cmd: &str) -> Option<u32> {
    let v = cmd.strip_prefix("volume ")?.trim_end();
    let (whole, frac) = v.split_once('.').unwrap_or((v, ""));
    if whole.is_empty() && frac.is_empty() || !whole.bytes().chain(frac.bytes()).all(|b| b.is_ascii_digit()) || whole.len() > 2 {
        return None;
    }
    let w: u32 = if whole.is_empty() { 0 } else { whole.parse().ok()? };
    let mut milli = 0u32;
    for (i, b) in frac.bytes().enumerate() {
        if i < 3 {
            milli += (b - b'0') as u32 * [100, 10, 1][i];
        } else if b != b'0' && w == 1 {
            return None;
        }
    }
    if w > 1 || w == 1 && milli != 0 {
        return None;
    }
    Some(w * 1000 + milli)
}

#[cfg(test)]
mod tests {
    use super::parse_embed;
    #[test]
    fn parses_page_gain() {
        assert_eq!(parse_embed("volume 0.000"), Some(0));
        assert_eq!(parse_embed("volume 0.500"), Some(500));
        assert_eq!(parse_embed("volume 0.7049"), Some(704));
        assert_eq!(parse_embed("volume 1"), Some(1000));
        assert_eq!(parse_embed("volume 1.000"), Some(1000));
        assert_eq!(parse_embed("volume .25"), Some(250));
        for bad in ["volume 1.001", "volume 2", "volume -0.5", "volume ", "volume 0,5", "volume x", "volumes 0.5"] {
            assert_eq!(parse_embed(bad), None, "{bad}");
        }
    }
}
