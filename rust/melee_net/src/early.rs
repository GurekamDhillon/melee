//! Early inputs: a controller change that matters is sent to the other player the moment it happens, stamped with
//! the pad frame it will first appear in, instead of waiting for the next frame's input packet. The receiver
//! predicts with the newest early pad until the confirmed pad arrives; the confirmed pad always decides, so the
//! simulation, replays and checksums never change, only how often a prediction is right.
//!
//! The design is Arte's subframe rollback (JulienBernard3383279: the Kristal build and the "Subframe rollback"
//! draft for Slippi's Dolphin), as Hero's Melee Unlocked carries it (`docs/early-inputs.md` there). This is our own
//! implementation over the gw_net wire; nothing is shared with either. See docs/rust-port/notes/net.md.
//!
//! Here: which changes matter ([`matters`]), the packet ([`encode`] / [`decode`]), the receiver's per-slot buffer
//! ([`Buffer`]), the trust rule ([`Trust`]) and the sender's judgement ([`SenderState`], thread-agnostic: the
//! sampler thread in the game crate owns one behind a mutex).

use crate::pad::NP_PAYLOAD;

/// The pad as it travels: the 11 bytes of `np_encode` (buttons big-endian, main stick, C-stick, L, R, analog A,
/// analog B, err).
pub type Pad = [u8; NP_PAYLOAD];

/// Entries kept per remote slot (a rollback window is 7..12 frames; more unconfirmed early pads than that would mean
/// the confirmed inputs stopped arriving).
pub const MAX_ENTRIES: usize = 8;

/// The packet: `MEI1` | room channel u8 | target frame i32 BE | in-game port u8 | pad[11].
pub const MAGIC: &[u8; 4] = b"MEI1";
pub const PACKET_LEN: usize = 4 + 1 + 4 + 1 + NP_PAYLOAD;

/// Handshake capability bit (HELLO / ACCEPT trailing caps byte): this side sends and uses early pads.
pub const CAP_EARLY: u8 = 0x01;

/// Sticks within two of neutral read as zero (for the judgement only; the bytes sent are the pad as read).
pub fn clamp_rest(p: &mut Pad) {
    let (sx, sy, cx, cy) = (p[2] as i8, p[3] as i8, p[4] as i8, p[5] as i8);
    if (-2..=2).contains(&sx) && (-2..=2).contains(&sy) {
        p[2] = 0;
        p[3] = 0;
    }
    if (-2..=2).contains(&cx) && (-2..=2).contains(&cy) {
        p[4] = 0;
        p[5] = 0;
    }
}

// The stick lines the game acts on, in the controller's units (the game divides by 80): walk / tilt 23 (0.2875),
// dash / smash 64 (0.8), tap jump 53 (0.6625), crouch 56 (0.7). A negative line is crossed at or below it, a
// positive one at or above it.
const MAIN_X: [i8; 4] = [-64, -23, 23, 64];
const MAIN_Y: [i8; 4] = [-56, -23, 23, 53];
const C_X: [i8; 4] = [-64, -23, 23, 64];
const C_Y: [i8; 4] = [-53, -23, 23, 53];
/// Shield drop: main stick Y -53 while a shield input is held.
const SHIELD_DROP: i8 = -53;
/// The analog trigger value read as a light shield.
pub const TRIGGER_LINE: u8 = 43;
/// A B Start: act on press. X Y Z R L: held for a state. The D-pad is left out.
const PRESS_BUTTONS: u16 = 0x1300;
const HELD_BUTTONS: u16 = 0x0C70;

/// `all`: every line both ways, every button both ways; `presses`: outward crossings and presses only (plus the
/// held buttons both ways). `MELEE_EARLY_RULES=presses` picks the second (measurement).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Rules {
    #[default]
    All,
    Presses,
}

fn inside(v: i8, line: i8) -> bool {
    if line < 0 {
        v <= line
    } else {
        v >= line
    }
}
fn crossed(now: i8, before: i8, line: i8, outward: bool) -> bool {
    let (a, b) = (inside(now, line), inside(before, line));
    if outward {
        a && !b
    } else {
        a != b
    }
}
fn axis(now: u8, before: u8, lines: &[i8; 4], outward: bool) -> bool {
    lines.iter().any(|&l| crossed(now as i8, before as i8, l, outward))
}

/// True when `now` differs from `before` in a way that changes what the game does. Both rest-clamped.
pub fn matters(now: &Pad, before: &Pad, rules: Rules) -> bool {
    let out = rules == Rules::Presses;
    let bn = u16::from_be_bytes([now[0], now[1]]);
    let bb = u16::from_be_bytes([before[0], before[1]]);
    if (bn ^ bb) & HELD_BUTTONS != 0 {
        return true;
    }
    let press = if out { bn & !bb & PRESS_BUTTONS } else { (bn ^ bb) & PRESS_BUTTONS };
    if press != 0 {
        return true;
    }
    if axis(now[2], before[2], &MAIN_X, out)
        || axis(now[3], before[3], &MAIN_Y, out)
        || axis(now[4], before[4], &C_X, out)
        || axis(now[5], before[5], &C_Y, out)
    {
        return true;
    }
    let shielding = bn & 0x0070 != 0 || now[6] >= TRIGGER_LINE || now[7] >= TRIGGER_LINE;
    if shielding && crossed(now[3] as i8, before[3] as i8, SHIELD_DROP, out) {
        return true;
    }
    (now[6] >= TRIGGER_LINE) != (before[6] >= TRIGGER_LINE) || (now[7] >= TRIGGER_LINE) != (before[7] >= TRIGGER_LINE)
}

pub fn encode(ch: u8, target: i32, port: u8, pad: &Pad) -> [u8; PACKET_LEN] {
    let mut o = [0u8; PACKET_LEN];
    o[..4].copy_from_slice(MAGIC);
    o[4] = ch;
    o[5..9].copy_from_slice(&target.to_be_bytes());
    o[9] = port;
    o[10..].copy_from_slice(pad);
    o
}

/// (channel, target, port, pad), or None for anything that is not an early packet.
pub fn decode(b: &[u8]) -> Option<(u8, i32, u8, Pad)> {
    if b.len() < PACKET_LEN || &b[..4] != MAGIC {
        return None;
    }
    let mut pad = [0u8; NP_PAYLOAD];
    pad.copy_from_slice(&b[10..PACKET_LEN]);
    Some((b[4], i32::from_be_bytes([b[5], b[6], b[7], b[8]]), b[9], pad))
}

/// The early pads held for one remote slot, oldest target first. Only prediction reads it.
#[derive(Clone, Debug, Default)]
pub struct Buffer {
    pub entries: Vec<(i32, Pad)>,
}

impl Buffer {
    /// Add or replace (the newest word about one target stands). Full: the oldest goes for a newer target, a target
    /// older than everything held is ignored.
    pub fn add(&mut self, target: i32, pad: &Pad) {
        if let Some(e) = self.entries.iter_mut().find(|e| e.0 == target) {
            e.1 = *pad;
            return;
        }
        if self.entries.len() >= MAX_ENTRIES {
            if target < self.entries[0].0 {
                return;
            }
            self.entries.remove(0);
        }
        let at = self.entries.iter().position(|e| e.0 > target).unwrap_or(self.entries.len());
        self.entries.insert(at, (target, *pad));
    }
    /// The confirmed inputs reach `latest`: every early pad for a frame at or before it is spent.
    pub fn drop_through(&mut self, latest: i32) {
        self.entries.retain(|e| e.0 > latest);
    }
    /// The pad to predict `frame` with when the confirmed inputs reach `latest` (< frame): the newest early pad for a
    /// target in (latest, frame].
    pub fn best(&self, frame: i32, latest: i32) -> Option<&Pad> {
        self.entries.iter().filter(|e| e.0 > latest && e.0 <= frame).last().map(|e| &e.1)
    }
    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

/// Early pads are data a peer volunteers: one whose guesses are mostly wrong (over 60 % after 64) is ignored for the
/// rest of the match.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Trust {
    pub guesses: u32,
    pub wrong: u32,
    pub distrusted: bool,
}

pub const TRUST_MIN_GUESSES: u32 = 64;
pub const TRUST_MAX_WRONG_PCT: u32 = 60;

impl Trust {
    /// Count one judged guess; returns true when this one made the peer untrusted.
    pub fn judge(&mut self, right: bool) -> bool {
        self.guesses += 1;
        if !right {
            self.wrong += 1;
        }
        if !self.distrusted && self.guesses >= TRUST_MIN_GUESSES && self.wrong * 100 > self.guesses * TRUST_MAX_WRONG_PCT {
            self.distrusted = true;
            return true;
        }
        false
    }
}

/// Sender statistics.
#[derive(Clone, Copy, Debug, Default)]
pub struct SendStats {
    pub samples: u64,
    pub changes: u64,
    pub follow_ups: u64,
    pub resent_after_skip: u64,
    pub packets: u64,
}

/// The sender's judgement (behind a mutex, shared by the game thread and the sampler thread). The game thread tells
/// it each local pad it schedules (`note_sampled`), when it reads the controller (`note_poll`) and when a tick ran
/// no new frame (`note_skipped`); the sampler hands it each controller read (`judge`).
#[derive(Clone, Debug, Default)]
pub struct SenderState {
    /// the newest pad the other side has or will have (scheduled, or sent early), rest-clamped
    known: Pad,
    /// the newest local frame with a scheduled pad (None: none yet)
    pub last_frame: Option<i32>,
    /// the controller was read for a tick whose pad is not scheduled yet: a change now belongs to the frame after it
    poll_pending: bool,
    /// a tick ran without scheduling a pad: the next read is sent if it differs at all
    skipped: bool,
    crossed_target: Option<i32>,
    last_follow_up_us: u64,
    pub rules: Rules,
    pub stats: SendStats,
}

/// At most one follow-up every 2 ms.
pub const FOLLOW_UP_US: u64 = 2000;

impl SenderState {
    pub fn new(rules: Rules) -> Self {
        SenderState { rules, ..Default::default() }
    }
    fn target(&self) -> Option<i32> {
        self.last_frame.map(|f| f + if self.poll_pending { 2 } else { 1 })
    }
    pub fn note_poll(&mut self) {
        self.poll_pending = true;
    }
    pub fn note_skipped(&mut self) {
        self.poll_pending = false;
        if self.last_frame.is_some() {
            self.skipped = true;
        }
    }
    /// The pad for local `frame` is scheduled (it travels in the regular input packets).
    pub fn note_sampled(&mut self, frame: i32, pad: &Pad) {
        let mut p = *pad;
        clamp_rest(&mut p);
        self.known = p;
        self.last_frame = Some(frame);
        self.poll_pending = false;
    }
    /// One controller read at `now_us`: Some(target) when `pad` goes out now as an early pad for `target`.
    /// Follow-ups: once a change crossed a line for a frame, later byte changes before that frame's poll go too
    /// (a guess is judged on every byte), at most one per [`FOLLOW_UP_US`].
    pub fn judge(&mut self, pad: &Pad, now_us: u64) -> Option<i32> {
        self.stats.samples += 1;
        let next = self.target()?;
        let mut p = *pad;
        clamp_rest(&mut p);
        let force = std::mem::take(&mut self.skipped);
        let differs = p != self.known;
        let judged = !force && matters(&p, &self.known, self.rules);
        let follow_up = !force && !judged && differs && self.crossed_target == Some(next) && now_us.saturating_sub(self.last_follow_up_us) >= FOLLOW_UP_US;
        if !((force && differs) || judged || follow_up) {
            return None;
        }
        self.known = p;
        self.stats.changes += 1;
        if force {
            self.stats.resent_after_skip += 1;
        }
        if judged {
            self.crossed_target = Some(next);
        }
        if follow_up {
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
