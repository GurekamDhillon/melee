//! Early inputs (netplay prediction hints).
//!
//! Credit: the design follows the early-input approach of melee-unlocked, by Hero, which in turn builds on Arte's
//! subframe rollback (JulienBernard3383279). This file is a clean-room rewrite written from our own tests and notes
//! (docs/rust-port/notes/net.md, "Early inputs"); no code or text was taken from melee-unlocked.
//!
//! Timeline of one local pad:
//! - tick N schedules the pad for frame F; it goes out in the normal input packets;
//! - between ticks a sampler thread keeps reading the controller; a read that changes what the game would do is
//!   sent straight away as a hint for frame F+1 (or F+2 once the controller has been polled for F+1 but that pad has
//!   not been scheduled yet);
//! - the remote side keeps those hints per slot and, while frame G is still unconfirmed, predicts it from the
//!   highest-target hint in (last confirmed, G]; without one it falls back to repeating the last confirmed pad.
//!
//! Hints only steer prediction. The confirmed pad replaces any guess, so the simulation, replays and checksums are
//! identical with hints on or off.
//!
//! Contents: significance test ([`matters`], [`clamp_rest`]), datagram ([`encode`] / [`decode`]), per-slot hint
//! store ([`Buffer`]), misbehaving-peer cut-off ([`Trust`]) and the sending decision ([`SenderState`]; plain data,
//! the game crate wraps it in a mutex shared by the game and sampler threads).

use crate::pad::NP_PAYLOAD;

/// Wire form of one pad: the `np_encode` bytes. Index 0-1 buttons (big-endian), 2-3 main stick X/Y, 4-5 C-stick X/Y,
/// 6-7 analog L/R, then analog A, analog B, err.
pub type Pad = [u8; NP_PAYLOAD];

/// Hint capacity per remote slot. Rollback never reaches further back than about a dozen frames, so a longer backlog
/// would only mean the confirmed stream has stalled.
pub const MAX_ENTRIES: usize = 8;

/// Datagram layout: tag `MEI1`, room channel (1 byte), target frame (i32, big-endian), in-game port (1 byte), pad.
pub const MAGIC: &[u8; 4] = b"MEI1";
pub const PACKET_LEN: usize = 4 + 1 + 4 + 1 + NP_PAYLOAD;

/// Bit in the HELLO / ACCEPT capability byte announcing hint support.
pub const CAP_EARLY: u8 = 0x01;

/// Dead zone used only when deciding significance: a stick whose both axes are within +-2 is treated as centred.
/// (What is transmitted is always the raw read.)
pub fn clamp_rest(p: &mut Pad) {
    for base in [2usize, 4] {
        let near = |b: u8| (b as i8).unsigned_abs() <= 2;
        if near(p[base]) && near(p[base + 1]) {
            p[base] = 0;
            p[base + 1] = 0;
        }
    }
}

/// Per-axis decision lines in raw stick units (game value = raw / 80). Index order matches the pad bytes 2..6.
/// Lines below zero are reached by going at or under them, lines above zero by going at or over them.
const AXIS_LINES: [[i8; 4]; 4] = [
    [-64, -23, 23, 64], // main X: walk/tilt, dash/smash
    [-56, -23, 23, 53], // main Y: crouch, tilt, tap jump
    [-64, -23, 23, 64], // C X
    [-53, -23, 23, 53], // C Y
];
/// Main-stick Y line that drops a held shield.
const SHIELD_DROP: i8 = -53;
/// Analog trigger depth counted as engaged.
pub const TRIGGER_LINE: u8 = 43;
/// Buttons that act on the edge (A, B, Start) versus buttons whose held state matters (X, Y, Z, R, L).
/// D-pad changes are never significant.
const EDGE_MASK: u16 = 0x1300;
const LEVEL_MASK: u16 = 0x0C70;
/// Digital L / R / Z bits used to tell whether the player is shielding.
const SHIELD_BUTTONS: u16 = 0x0070;

/// Which transitions count. `All` (default): any line crossed in either direction and any edge button change.
/// `Presses`: only crossings away from centre and button presses; level buttons still count both ways. Picked with
/// `MELEE_EARLY_RULES=presses` for measurements.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Rules {
    #[default]
    All,
    Presses,
}

fn past(v: i8, line: i8) -> bool {
    if line < 0 {
        v <= line
    } else {
        v >= line
    }
}

/// Did the value move across `line`? With `entering_only` only a move from the near side to the far side counts.
fn moved_across(now: i8, before: i8, line: i8, entering_only: bool) -> bool {
    match (past(now, line), past(before, line)) {
        (true, false) => true,
        (false, true) => !entering_only,
        _ => false,
    }
}

fn engaged(trigger: u8) -> bool {
    trigger >= TRIGGER_LINE
}

/// Significance test: would the game plausibly react differently to `now` than to `before`? Callers pass both
/// through [`clamp_rest`] first.
pub fn matters(now: &Pad, before: &Pad, rules: Rules) -> bool {
    let entering_only = rules == Rules::Presses;
    let b_now = u16::from_be_bytes([now[0], now[1]]);
    let b_old = u16::from_be_bytes([before[0], before[1]]);
    let flipped = b_now ^ b_old;

    if flipped & LEVEL_MASK != 0 {
        return true;
    }
    let edge = if entering_only { flipped & b_now } else { flipped };
    if edge & EDGE_MASK != 0 {
        return true;
    }
    for (i, lines) in AXIS_LINES.iter().enumerate() {
        let (n, o) = (now[2 + i] as i8, before[2 + i] as i8);
        if lines.iter().any(|&l| moved_across(n, o, l, entering_only)) {
            return true;
        }
    }
    let shielding = b_now & SHIELD_BUTTONS != 0 || engaged(now[6]) || engaged(now[7]);
    if shielding && moved_across(now[3] as i8, before[3] as i8, SHIELD_DROP, entering_only) {
        return true;
    }
    engaged(now[6]) != engaged(before[6]) || engaged(now[7]) != engaged(before[7])
}

pub fn encode(ch: u8, target: i32, port: u8, pad: &Pad) -> [u8; PACKET_LEN] {
    let mut out = [0u8; PACKET_LEN];
    let mut w = 0;
    for chunk in [&MAGIC[..], &[ch], &target.to_be_bytes(), &[port], &pad[..]] {
        out[w..w + chunk.len()].copy_from_slice(chunk);
        w += chunk.len();
    }
    out
}

/// Parse a hint datagram into (channel, target, port, pad); anything else gives None.
pub fn decode(b: &[u8]) -> Option<(u8, i32, u8, Pad)> {
    if b.len() < PACKET_LEN || !b.starts_with(MAGIC) {
        return None;
    }
    let target = i32::from_be_bytes(b[5..9].try_into().ok()?);
    let pad: Pad = b[10..PACKET_LEN].try_into().ok()?;
    Some((b[4], target, b[9], pad))
}

/// Hints received for one remote slot, kept sorted by target frame. Read by prediction only.
#[derive(Clone, Debug, Default)]
pub struct Buffer {
    pub entries: Vec<(i32, Pad)>,
}

impl Buffer {
    /// Store a hint. A repeat for a target already held overwrites it. At capacity the lowest target is evicted to
    /// make room, unless the incoming target is lower still, in which case it is discarded.
    pub fn add(&mut self, target: i32, pad: &Pad) {
        match self.entries.binary_search_by_key(&target, |e| e.0) {
            Ok(i) => self.entries[i].1 = *pad,
            Err(i) => {
                if self.entries.len() < MAX_ENTRIES {
                    self.entries.insert(i, (target, *pad));
                } else if i > 0 {
                    self.entries.remove(0);
                    self.entries.insert(i - 1, (target, *pad));
                }
            }
        }
    }
    /// Forget hints for frames the confirmed stream now covers (target <= `latest`).
    pub fn drop_through(&mut self, latest: i32) {
        let keep_from = self.entries.partition_point(|e| e.0 <= latest);
        self.entries.drain(..keep_from);
    }
    /// Hint to predict `frame` with, given confirmed inputs up to `latest`: the highest target in (latest, frame].
    pub fn best(&self, frame: i32, latest: i32) -> Option<&Pad> {
        self.entries.iter().rev().find(|e| e.0 <= frame && e.0 > latest).map(|e| &e.1)
    }
    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

/// Cut-off for a peer whose hints are unreliable: once at least [`TRUST_MIN_GUESSES`] hinted predictions have been
/// checked and more than [`TRUST_MAX_WRONG_PCT`] % were wrong, its hints are ignored until the match ends.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Trust {
    pub guesses: u32,
    pub wrong: u32,
    pub distrusted: bool,
}

pub const TRUST_MIN_GUESSES: u32 = 64;
pub const TRUST_MAX_WRONG_PCT: u32 = 60;

impl Trust {
    /// Record whether one hinted prediction turned out right. Returns true exactly once: on the call that trips the
    /// cut-off.
    pub fn judge(&mut self, right: bool) -> bool {
        self.guesses += 1;
        self.wrong += u32::from(!right);
        let over = self.guesses >= TRUST_MIN_GUESSES && self.wrong * 100 > self.guesses * TRUST_MAX_WRONG_PCT;
        let tripped = over && !self.distrusted;
        self.distrusted |= tripped;
        tripped
    }
}

/// Counters reported in the match summary.
#[derive(Clone, Copy, Debug, Default)]
pub struct SendStats {
    pub samples: u64,
    pub changes: u64,
    pub follow_ups: u64,
    pub resent_after_skip: u64,
    pub packets: u64,
}

/// Sending decision. Game-thread events: [`note_sampled`](Self::note_sampled) when a local pad is scheduled,
/// [`note_poll`](Self::note_poll) when the controller is read for the next tick, [`note_skipped`](Self::note_skipped)
/// when a tick scheduled nothing. Sampler-thread event: [`judge`](Self::judge) per controller read.
#[derive(Clone, Debug, Default)]
pub struct SenderState {
    /// Last pad the peer is known to have or be about to get (scheduled or hinted), dead-zoned.
    known: Pad,
    /// Highest local frame whose pad has been scheduled, if any.
    pub last_frame: Option<i32>,
    /// Controller already read for the coming tick, pad not yet scheduled: new changes target one frame later.
    poll_pending: bool,
    /// Previous tick scheduled nothing: send the next read on any byte difference.
    skipped: bool,
    /// Target of the last significant hint; enables follow-ups for that same target.
    crossed_target: Option<i32>,
    last_follow_up_us: u64,
    pub rules: Rules,
    pub stats: SendStats,
}

/// Minimum spacing between follow-up hints, in microseconds.
pub const FOLLOW_UP_US: u64 = 2000;

impl SenderState {
    pub fn new(rules: Rules) -> Self {
        SenderState { rules, ..Default::default() }
    }
    /// Frame a change seen now would first land in.
    fn target(&self) -> Option<i32> {
        let ahead = if self.poll_pending { 2 } else { 1 };
        self.last_frame.map(|f| f + ahead)
    }
    pub fn note_poll(&mut self) {
        self.poll_pending = true;
    }
    pub fn note_skipped(&mut self) {
        self.poll_pending = false;
        self.skipped |= self.last_frame.is_some();
    }
    /// Local pad for `frame` scheduled; the regular input stream carries it.
    pub fn note_sampled(&mut self, frame: i32, pad: &Pad) {
        let mut p = *pad;
        clamp_rest(&mut p);
        self.known = p;
        self.last_frame = Some(frame);
        self.poll_pending = false;
    }
    /// Decide on one controller read taken at `now_us`. Returns the target frame when `pad` should be sent as a hint.
    ///
    /// Three reasons to send: (a) right after a skipped tick, any difference; (b) a significant change per
    /// [`matters`]; (c) a follow-up: after (b) for target T, further raw differences while T is still the target,
    /// rate-limited by [`FOLLOW_UP_US`], because the receiver compares the whole pad, not only the lines.
    pub fn judge(&mut self, pad: &Pad, now_us: u64) -> Option<i32> {
        self.stats.samples += 1;
        let next = self.target()?;
        let mut p = *pad;
        clamp_rest(&mut p);
        let after_skip = std::mem::take(&mut self.skipped);
        let differs = p != self.known;

        let reason_skip = after_skip && differs;
        let reason_sig = !after_skip && matters(&p, &self.known, self.rules);
        let reason_follow = !after_skip
            && !reason_sig
            && differs
            && self.crossed_target == Some(next)
            && now_us.saturating_sub(self.last_follow_up_us) >= FOLLOW_UP_US;
        if !(reason_skip || reason_sig || reason_follow) {
            return None;
        }

        self.known = p;
        self.stats.changes += 1;
        if after_skip {
            self.stats.resent_after_skip += 1;
        }
        if reason_sig {
            self.crossed_target = Some(next);
        }
        if reason_follow {
            self.stats.follow_ups += 1;
            self.last_follow_up_us = now_us;
        }
        Some(next)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pad(btn: u16, sx: i8, sy: i8, cx: i8, cy: i8, l: u8, r: u8) -> Pad {
        let b = btn.to_be_bytes();
        [b[0], b[1], sx as u8, sy as u8, cx as u8, cy as u8, l, r, 0, 0, 0]
    }
    fn m(a: Pad, b: Pad, r: Rules) -> bool {
        let (mut a, mut b) = (a, b);
        clamp_rest(&mut a);
        clamp_rest(&mut b);
        matters(&a, &b, r)
    }

    #[test]
    fn thresholds() {
        let z = pad(0, 0, 0, 0, 0, 0, 0);
        for r in [Rules::All, Rules::Presses] {
            // buttons
            assert!(m(pad(0x0100, 0, 0, 0, 0, 0, 0), z, r), "A press");
            assert_eq!(m(z, pad(0x0100, 0, 0, 0, 0, 0, 0), r), r == Rules::All, "A release");
            assert!(m(pad(0x0040, 0, 0, 0, 0, 0, 0), z, r) && m(z, pad(0x0040, 0, 0, 0, 0, 0, 0), r), "L both ways");
            assert!(!m(pad(0x0001, 0, 0, 0, 0, 0, 0), z, r), "d-pad never");
            // main stick X 23 / 64 both signs
            for v in [23i8, 64, -23, -64] {
                assert!(m(pad(0, v, 0, 0, 0, 0, 0), pad(0, v - v.signum(), 0, 0, 0, 0, 0), r), "x {v}");
                assert_eq!(m(pad(0, v - v.signum(), 0, 0, 0, 0, 0), pad(0, v, 0, 0, 0, 0, 0), r), r == Rules::All, "x back {v}");
            }
            assert!(!m(pad(0, 40, 0, 0, 0, 0, 0), pad(0, 30, 0, 0, 0, 0, 0), r), "inside one zone");
            // main Y
            for v in [23i8, 53, -23, -56] {
                assert!(m(pad(0, 0, v, 0, 0, 0, 0), pad(0, 0, v - v.signum(), 0, 0, 0, 0), r), "y {v}");
            }
            assert!(!m(pad(0, 0, -53, 0, 0, 0, 0), pad(0, 0, -52, 0, 0, 0, 0), r), "-53 without shield");
            assert!(m(pad(0x0020, 0, -53, 0, 0, 0, 0), pad(0x0020, 0, -52, 0, 0, 0, 0), r), "shield drop with R");
            assert!(m(pad(0, 0, -53, 0, 0, 50, 0), pad(0, 0, -52, 0, 0, 50, 0), r), "shield drop with trigger");
            // c-stick
            for v in [23i8, 64, -23, -64] {
                assert!(m(pad(0, 0, 0, v, 0, 0, 0), pad(0, 0, 0, v - v.signum(), 0, 0, 0), r), "cx {v}");
            }
            for v in [23i8, 53, -23, -53] {
                assert!(m(pad(0, 0, 0, 0, v, 0, 0), pad(0, 0, 0, 0, v - v.signum(), 0, 0), r), "cy {v}");
            }
            // trigger 43 both ways
            assert!(m(pad(0, 0, 0, 0, 0, 43, 0), pad(0, 0, 0, 0, 0, 42, 0), r));
            assert!(m(pad(0, 0, 0, 0, 0, 42, 0), pad(0, 0, 0, 0, 0, 43, 0), r));
            assert!(m(pad(0, 0, 0, 0, 0, 0, 43), z, r));
            // rest clamp
            assert!(!m(pad(0, 2, -2, 1, 0, 0, 0), z, r));
        }
    }

    #[test]
    fn wire() {
        let p = pad(0x1234, -5, 7, 9, -11, 40, 200);
        let b = encode(3, -100, 1, &p);
        assert_eq!(b.len(), PACKET_LEN);
        assert_eq!(decode(&b), Some((3, -100, 1, p)));
        assert_eq!(decode(&b[..PACKET_LEN - 1]), None);
        assert_eq!(decode(b"GN............................"), None);
    }

    #[test]
    fn buffer() {
        let mut b = Buffer::default();
        let p = |v: u8| [v; NP_PAYLOAD];
        b.add(10, &p(1));
        b.add(8, &p(2));
        b.add(12, &p(3));
        b.add(10, &p(4)); // replaces
        assert_eq!(b.entries.iter().map(|e| e.0).collect::<Vec<_>>(), vec![8, 10, 12]);
        assert_eq!(b.best(11, 7), Some(&p(4)));
        assert_eq!(b.best(11, 10), None);
        assert_eq!(b.best(7, 5), None);
        assert_eq!(b.best(20, 5), Some(&p(3)));
        b.drop_through(10);
        assert_eq!(b.entries.len(), 1);
        for t in 20..40 {
            b.add(t, &p(t as u8));
        }
        assert_eq!(b.entries.len(), MAX_ENTRIES);
        assert_eq!(b.entries[0].0, 32);
        b.add(5, &p(0));
        assert_eq!(b.entries[0].0, 32, "older than everything: ignored when full");
    }

    #[test]
    fn trust() {
        let mut t = Trust::default();
        for i in 0..63 {
            assert!(!t.judge(i % 10 == 0));
        }
        assert!(t.judge(false), "64th guess, >60 % wrong");
        assert!(t.distrusted);
        let mut h = Trust::default();
        for i in 0..200 {
            h.judge(i % 4 != 0); // 25 % wrong
        }
        assert!(!h.distrusted);
    }

    #[test]
    fn sender() {
        let mut s = SenderState::new(Rules::All);
        let z = pad(0, 0, 0, 0, 0, 0, 0);
        assert_eq!(s.judge(&pad(0x100, 0, 0, 0, 0, 0, 0), 0), None, "nothing scheduled yet");
        s.note_sampled(10, &z);
        assert_eq!(s.judge(&z, 100), None, "at rest");
        assert_eq!(s.judge(&pad(0x100, 0, 0, 0, 0, 0, 0), 200), Some(11));
        assert_eq!(s.judge(&pad(0x100, 0, 0, 0, 0, 0, 0), 300), None, "already known");
        // follow-up: a stick byte change after the crossing, rate limited
        assert_eq!(s.judge(&pad(0x100, 10, 0, 0, 0, 0, 0), 500), None, "inside 2 ms");
        assert_eq!(s.judge(&pad(0x100, 12, 0, 0, 0, 0, 0), 2300), Some(11));
        // a poll read the controller: the change belongs to the frame after its pad
        s.note_poll();
        assert_eq!(s.judge(&pad(0x100, 30, 0, 0, 0, 0, 0), 3000), Some(12));
        s.note_sampled(11, &pad(0x100, 12, 0, 0, 0, 0, 0));
        // a skipped tick: any difference goes
        s.note_skipped();
        assert_eq!(s.judge(&pad(0x100, 13, 0, 0, 0, 0, 0), 9000), Some(12));
    }
}
