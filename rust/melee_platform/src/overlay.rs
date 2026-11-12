//! The in-game HUD overlay: fps, frame time, sim rate, and what the game adds (netplay: ping, input delay,
//! rollbacks, health; offline: the practice input delay). Toggled with F3, on at start with `overlay=1` in
//! settings.cfg / `MELEE_OVERLAY=1`.
//!
//! Render-only by construction: the text is laid out here and drawn by melee_gx's renderer over the swapchain image
//! after the EFB blit ([`melee_gx::render::Renderer::overlay_to`]). It never enters the GX shim, a draw list, the EFB
//! or anything the simulation reads, so frames, hashes and replays are the same with it on or off.
use melee_gx::render::OverlayQuad;

/// A line of overlay text and its colour.
pub type Line = (String, [u8; 4]);

pub const WHITE: [u8; 4] = [235, 238, 245, 255];
pub const DIM: [u8; 4] = [150, 156, 170, 255];
pub const GOOD: [u8; 4] = [110, 225, 135, 255];
pub const WARN: [u8; 4] = [250, 205, 80, 255];
pub const BAD: [u8; 4] = [245, 95, 80, 255];

// The lines live in the Presenter (`Presenter::hud`, handed over through `Game::set_hud`), never in a static: a
// rollback run's snapshots restore the program's statics, which would bring back a freed Vec.

/// The overlay's quads for a `tw` x `th` target. Top-left panel; the glyph scale follows the target height (one
/// 8x8 cell per 480 lines, at least 1).

pub fn layout(lines: &[Line], _tw: u32, th: u32) -> Vec<OverlayQuad> {
    let s = ((th as f32 / 480.0).round()).max(1.0);
    let (pad, lh) = (4.0 * s, 10.0 * s);
    let cols = lines.iter().map(|l| l.0.chars().count()).max().unwrap_or(0) as f32;
    let mut out = vec![OverlayQuad { x: 6.0 * s, y: 6.0 * s, w: cols * 8.0 * s + 2.0 * pad, h: lines.len() as f32 * lh + 2.0 * pad - 2.0 * s, c: [0, 0, 0, 150] }];
    for (i, (text, c)) in lines.iter().enumerate() {
        text_quads(&mut out, text, 6.0 * s + pad, 6.0 * s + pad + i as f32 * lh, s, *c);
    }
    out
}

/// One line of 8x8 text at (x0, y0), scale `s`.
fn text_quads(out: &mut Vec<OverlayQuad>, text: &str, x0: f32, y0: f32, s: f32, c: [u8; 4]) {
    for (k, ch) in text.chars().enumerate() {
        let glyph = font8x8::UnicodeFonts::get(&font8x8::BASIC_FONTS, if ch.is_ascii() { ch } else { '?' }).unwrap_or([0; 8]);
        let cx = x0 + k as f32 * 8.0 * s;
        for (row, bits) in glyph.iter().enumerate() {
            // runs of lit pixels in a row (bit 0 is the leftmost column) as one rectangle
            let mut col = 0;
            while col < 8 {
                if bits >> col & 1 == 1 {
                    let start = col;
                    while col < 8 && bits >> col & 1 == 1 {
                        col += 1;
                    }
                    out.push(OverlayQuad { x: cx + start as f32 * s, y: y0 + row as f32 * s, w: (col - start) as f32 * s, h: s, c });
                } else {
                    col += 1;
                }
            }
        }
    }
}

/// The frame profiler's readout (MELEE_PROFILE_OVERLAY=1): a graph of the last presented-frame intervals and the
/// hitch count, top right. Render-only, like the rest of the overlay.
pub fn profile_readout(out: &mut Vec<OverlayQuad>, tw: u32, th: u32) {
    let Some(r) = melee_core::frameprof::readout() else { return };
    let s = ((th as f32 / 720.0).round()).max(1.0);
    let n = r.intervals.len() as f32;
    let (gw, gh) = (n * s, 64.0 * s);
    let (pad, lh) = (4.0 * s, 10.0 * s);
    let pw = gw.max(36.0 * 8.0 * s);
    let x0 = tw as f32 - pw - 2.0 * pad - 6.0 * s;
    let y0 = 6.0 * s;
    out.push(OverlayQuad { x: x0, y: y0, w: pw + 2.0 * pad, h: gh + 2.0 * lh + 2.0 * pad, c: [0, 0, 0, 160] });
    let exp = r.expected_us.max(1000) as f32;
    // the graph spans 0 .. 3x the expected interval (at least 33.3 ms); bars past the top are clipped
    let top = (3.0 * exp).max(33_333.0);
    let gy = y0 + pad + 2.0 * lh;
    for (k, &us) in r.intervals.iter().enumerate() {
        if us == 0 {
            continue;
        }
        let h = (us as f32 / top).min(1.0) * gh;
        let c = if us as f32 > 1.5 * exp && us >= 25_000 { BAD } else if us as f32 > 1.5 * exp { WARN } else { GOOD };
        out.push(OverlayQuad { x: x0 + pad + k as f32 * s, y: gy + gh - h, w: s, h: h.max(s), c });
    }
    // the expected interval and 16.7 ms (one tick) as thin lines
    for (v, c) in [(exp, [200, 200, 200, 140]), (16_667.0, [120, 160, 255, 140])] {
        if v < top {
            out.push(OverlayQuad { x: x0 + pad, y: gy + gh - v / top * gh, w: gw, h: s, c });
        }
    }
    let last = r.last.map_or(String::new(), |(ago, ms)| format!("  last {ms} ms {ago:.0}s ago"));
    text_quads(out, &format!("hitches {}{last}", r.hitches), x0 + pad, y0 + pad, s, if r.last.is_some_and(|l| l.0 < 2.0) { WARN } else { WHITE });
    let cur = r.intervals[r.intervals.len() - 1] as f32 / 1e3;
    text_quads(out, &format!("frame {cur:.1} ms (target {:.1})", exp / 1e3), x0 + pad, y0 + pad + lh, s, DIM);
}

/// Frame-time statistics over the last overlay period.
#[derive(Default)]
pub struct FrameWindow {
    us: Vec<u32>,
}

impl FrameWindow {
    pub fn push(&mut self, us: u32) {
        if self.us.len() < 4096 {
            self.us.push(us);
        }
    }
    /// (mean ms, worst ms) since the last take.
    pub fn take(&mut self) -> (f64, f64) {
        let v = std::mem::take(&mut self.us);
        if v.is_empty() {
            return (0.0, 0.0);
        }
        let mean = v.iter().map(|&x| x as f64).sum::<f64>() / v.len() as f64 / 1e3;
        (mean, *v.iter().max().unwrap() as f64 / 1e3)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layout_panel_then_glyphs() {
        let q = layout(&[("I".into(), WHITE)], 640, 480);
        assert_eq!(q[0].c[3], 150);
        assert!(q.len() > 1 && q[1..].iter().all(|r| r.c == WHITE));
        // scale 2 at 960 lines
        let q2 = layout(&[("I".into(), WHITE)], 1280, 960);
        assert_eq!(q2[0].x, 12.0);
    }

    #[test]
    fn frame_window() {
        let mut f = FrameWindow::default();
        f.push(16000);
        f.push(18000);
        assert_eq!(f.take(), (17.0, 18.0));
        assert_eq!(f.take(), (0.0, 0.0));
    }
}
