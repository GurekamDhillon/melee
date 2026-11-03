//! Netplay for the HUD overlay and the input-delay choice: a read-only stats snapshot of the rollback session
//! ([`NetHud`], from `NetDriver::hud`), its overlay lines, and `--np-delay auto` (the delay from the measured ping).
//!
//! Nothing here writes to the session or the engine; the overlay reads, the delay is chosen before a room opens.
use melee_platform::overlay::{Line, BAD, DIM, GOOD, WARN, WHITE};
use std::sync::atomic::AtomicBool;

/// `--np-delay auto`: each match's delay comes from the ping measured on the connection.
pub static AUTO_DELAY: AtomicBool = AtomicBool::new(false);

/// The fallback (and first-match) delay under `--np-delay auto`.
pub const AUTO_FALLBACK: i32 = 2;

/// Frames of input delay for a round trip of `rtt_ms`: half the round trip (one way) plus a few ms of jitter margin,
/// in 60 Hz frames, rounded up, kept in 1..=7 (the session allows 0..=8; 0 rolls back every remote input).
pub fn auto_delay(rtt_ms: u32) -> i32 {
    if rtt_ms == 0 {
        return AUTO_FALLBACK;
    }
    let one_way = rtt_ms as f64 * 0.5 + 5.0;
    ((one_way * 60.0 / 1000.0).ceil() as i32).clamp(1, 7)
}

/// `--np-delay N|auto` -> (the delay to start with, auto).
pub fn parse_delay(v: Option<&str>) -> (i32, bool) {
    match v.map(str::trim) {
        Some("auto") => (AUTO_FALLBACK, true),
        Some(s) => (s.parse::<i32>().map_or(2, |d| d.clamp(0, 8)), false),
        None => (2, false),
    }
}

/// One read of the netplay state.
#[derive(Clone, Debug, Default)]
pub struct NetHud {
    pub host: bool,
    /// smoothed round trip (0: not measured yet)
    pub ping_ms: u32,
    /// ms since the last packet from the peer (0: unknown)
    pub silent_ms: u32,
    pub delay: i32,
    pub auto: bool,
    pub rollbacks: i32,
    pub resim: i32,
    pub depth_max: i32,
    pub stall_ticks: i32,
    /// "menus", "connecting", "match", "match over", "error"
    pub phase: &'static str,
    /// the game frame of the match (from 0)
    pub frame: Option<i32>,
    /// the menus' lockstep: frame, stalls
    pub lockstep: Option<(u32, u64)>,
    pub error: Option<String>,
}

/// Rates between two overlay refreshes.
#[derive(Default)]
pub struct HudRates {
    last: Option<(std::time::Instant, i32, i32, i32)>,
    /// rollbacks / s, resimulated frames / s, stall ticks / s, mean depth of the period's rollbacks
    pub now: (f64, f64, f64, f64),
}

impl HudRates {
    pub fn update(&mut self, h: &NetHud) {
        let t = std::time::Instant::now();
        if let Some((t0, rb, rs, st)) = self.last {
            let dt = (t - t0).as_secs_f64().max(1e-3);
            // a new match's session starts its counters from 0
            let (drb, drs, dst) = ((h.rollbacks - rb).max(0), (h.resim - rs).max(0), (h.stall_ticks - st).max(0));
            self.now = (drb as f64 / dt, drs as f64 / dt, dst as f64 / dt, if drb > 0 { drs as f64 / drb as f64 } else { 0.0 });
        }
        self.last = Some((t, h.rollbacks, h.resim, h.stall_ticks));
    }
}

/// good / fair / poor from the ping, the rollback rate and depth, stalls and silence.
pub fn health(h: &NetHud, r: &HudRates) -> (&'static str, [u8; 4]) {
    if h.error.is_some() {
        return ("LOST", BAD);
    }
    if h.silent_ms > 1500 {
        return ("NO PACKETS", BAD);
    }
    let (rbs, _, stalls, depth) = r.now;
    if h.ping_ms > 160 || stalls > 6.0 || depth > 5.0 || h.silent_ms > 400 {
        ("poor", BAD)
    } else if h.ping_ms > 90 || stalls > 0.5 || rbs > 30.0 || depth > 3.0 {
        ("fair", WARN)
    } else {
        ("good", GOOD)
    }
}

pub fn lines(h: &NetHud, r: &HudRates) -> Vec<Line> {
    let (hl, hc) = health(h, r);
    let ping = if h.ping_ms == 0 { "--".to_string() } else { format!("{} ms", h.ping_ms) };
    let mut v = vec![(format!("netplay {} {} [{hl}]", if h.host { "host" } else { "guest" }, h.phase), hc)];
    let auto = if h.auto {
        let next = auto_delay(h.ping_ms);
        if next != h.delay && h.ping_ms > 0 { format!(" (auto, next {next})") } else { " (auto)".into() }
    } else {
        String::new()
    };
    v.push((format!("ping {ping}  delay {}f{auto}", h.delay), WHITE));
    let (rbs, rss, sts, depth) = r.now;
    v.push((format!("rollback {rbs:.0}/s depth {depth:.1} max {}", h.depth_max), if depth > 3.0 { WARN } else { WHITE }));
    v.push((format!("total {} resim {rss:.0}/s stall {sts:.1}/s", h.rollbacks), DIM));
    if let Some((f, s)) = h.lockstep {
        v.push((format!("menus lockstep {f} stalls {s}"), DIM));
    }
    if let Some(f) = h.frame {
        v.push((format!("frame {f}"), DIM));
    }
    if let Some(e) = &h.error {
        v.push((e.chars().take(44).collect(), BAD));
    }
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auto_from_ping() {
        assert_eq!(auto_delay(0), AUTO_FALLBACK);
        assert_eq!(auto_delay(2), 1); // LAN
        assert_eq!(auto_delay(40), 2);
        assert_eq!(auto_delay(100), 4);
        assert_eq!(auto_delay(2000), 7);
    }

    #[test]
    fn parse() {
        assert_eq!(parse_delay(Some("auto")), (AUTO_FALLBACK, true));
        assert_eq!(parse_delay(Some("4")), (4, false));
        assert_eq!(parse_delay(Some("99")), (8, false));
        assert_eq!(parse_delay(None), (2, false));
    }

    #[test]
    fn health_levels() {
        let h = NetHud { ping_ms: 20, phase: "match", ..Default::default() };
        assert_eq!(health(&h, &HudRates::default()).0, "good");
        let h2 = NetHud { ping_ms: 200, ..h.clone() };
        assert_eq!(health(&h2, &HudRates::default()).0, "poor");
    }
}
